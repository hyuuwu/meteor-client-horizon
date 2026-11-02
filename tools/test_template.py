#!/usr/bin/env python3
"""Compile the real bundle on a tiny source fixture; no Horizon/assets required."""
import json
import os
from pathlib import Path
import subprocess
import shutil
import tempfile
import tomllib
import horizon


def run(command, **kwargs):
    result = subprocess.run(command, capture_output=True, text=True, **kwargs)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    return result


def snapshot(root):
    return {str(p.relative_to(root)): (horizon.digest(p.read_bytes()), p.stat().st_mtime_ns)
            for p in root.rglob('*') if p.is_file()}


def audit(original, stage, plan):
    run(['cargo', 'run', '--quiet', '--locked', '--offline', '-p', 'native-mod-source-audit',
         '--', str(original), str(stage), str(plan)], cwd=horizon.ROOT)


def main():
    root = horizon.ROOT
    (root/'target').mkdir(exist_ok=True)
    run(['cargo', 'build', '--locked', '--offline', '-p', 'horizon-mixins-cli'], cwd=horizon.MIXINS)
    cli = horizon.MIXINS/'target/debug/horizon-mixins'
    manifests = list((root/'mixins').glob('*/horizon-mixin.toml'))
    assert len(manifests) == 1, 'this template test expects one example bundle'
    manifest = tomllib.loads(manifests[0].read_text())
    with tempfile.TemporaryDirectory(prefix='template-test-', dir=root/'target') as scratch:
        scratch = Path(scratch)
        original, stage = scratch/'original', scratch/'stage'
        source = original/'crates/horizon-client-render/src'
        source.mkdir(parents=True)
        (original/'Cargo.toml').write_text('[workspace]\nmembers=["crates/horizon-client-render", "crates/horizon-client", "crates/winit"]\nresolver="2"\n')
        (source.parent/'Cargo.toml').write_text('[package]\nname="horizon-client-render"\nversion="0.1.0"\nedition="2024"\n')
        (source/'lib.rs').write_text('''extern crate self as horizon_world;
extern crate self as horizon_core;
pub mod position { pub struct BlockPos; impl BlockPos { pub fn new(_: i32, _: i32, _: i32) -> Self { Self } } }
pub mod registry {
    pub struct BlockStateId(pub u32);
    pub struct State { pub name: String }
    pub struct Registry;
    impl Registry { pub fn len(&self) -> usize { 3 } }
    impl Registry {
        pub fn state(&self, id: BlockStateId) -> Option<State> {
            Some(State { name: match id.0 {
                1 => "minecraft:stone", 2 => "minecraft:diamond_ore", _ => return None,
            }.into() })
        }
    }
}
pub mod assets { pub struct Assets { pub registry: &'static crate::registry::Registry } }
pub mod chunk {
    pub const MIN_Y: i32 = -64;
    pub struct Chunk { pub x: i32, pub z: i32 }
    impl Chunk {
        pub fn min_y(&self) -> i32 { MIN_Y }
        pub fn get(&self, x: i32, y: i32, z: i32) -> u32 {
            if x == 1 && y == MIN_Y && z == 1 { 2 } else { 1 }
        }
    }
}
pub mod mesher;
pub mod gfx;
''')
        (source/'mesher.rs').write_text('''use crate::{assets::Assets, chunk::Chunk};
const B: i32 = 1;
const P: usize = 18;
fn pi(x: i32, y: i32, z: i32) -> usize {
    ((y + B) as usize * P + (z + B) as usize) * P + (x + B) as usize
}
struct Padded { blocks: Vec<u32>, light: Vec<u8> }
impl Padded {
    fn new(region: &[[Option<&Chunk>; 3]; 3], sy: i32) -> Self {
        assert_eq!(sy, 0);
        assert!(region[1][1].is_some());
        let mut blocks = vec![1; P * P * P];
        blocks[pi(1, 0, 1)] = 2;
        Self { blocks, light: vec![0; P * P * P] }
    }
}
#[derive(Default)]
pub struct SectionMesh { pub visibility: u64, pub ores: Vec<[i32; 3]> }
pub struct Tables { info: Vec<()> }
pub struct Catalog;
pub struct Render { model_catalog: Catalog }
pub struct Model { faces: Vec<()> }
impl Catalog {
    fn models_at(&self, _: crate::registry::BlockStateId, _: crate::position::BlockPos, models: &mut Vec<Model>) -> Result<(), ()> {
        models.clear(); models.push(Model { faces: vec![(); 6] }); Ok(())
    }
}
struct Tints;
impl Tints { fn new(_: &[[Option<&Chunk>;3];3], _: &(), _: (), _: i32, _: [i32;3]) -> Self { Self } }
pub struct Mesher<'a> {
    pub assets: &'a Assets, tables: Tables, render: Render, biomes: &'a (), zoom: (), blend_radius: i32,
}
impl Mesher<'_> {
    pub fn mesh(&mut self, region: &[[Option<&Chunk>; 3]; 3], sy: i32) -> SectionMesh {
        let pad = Padded::new(region, sy);
        assert_eq!(pad.light[0], 0);
        assert_eq!(pad.blocks[pi(0, 0, 0)], 1);
        SectionMesh { visibility: 123, ores: vec![] }
    }
    fn face(&self, pad: &Padded, _: &mut Tints, out: &mut SectionMesh, pos: [i32;3], _: [f32;3], _: &(), ao: bool, _: &()) {
        assert!(!ao);
        assert_eq!(pad.light[0], 255);
        assert_eq!(pad.blocks[pi(-1, 0, 0)], 0);
        out.ores.push(pos);
    }
}
#[test] fn original_body_and_input_are_preserved() {
    static REGISTRY: crate::registry::Registry = crate::registry::Registry;
    let assets = Assets { registry: &REGISTRY };
    let mut mesher = Mesher { assets: &assets, tables: Tables { info: vec![(); 3] }, render: Render { model_catalog: Catalog }, biomes: &(), zoom: (), blend_radius: 0 };
    let chunk = Chunk { x: 0, z: 0 };
    let region = [[Some(&chunk); 3]; 3];
    let out = mesher.mesh(&region, 0);
    let xray = std::env::var("HORIZON_XRAY").as_deref() == Ok("1");
    assert_eq!(out.visibility, if xray { u64::MAX } else { 123 });
    assert_eq!(out.ores, if xray { vec![[1, 0, 1]; 6] } else { vec![] });
    assert_eq!(chunk.get(0, crate::chunk::MIN_Y, 0), 1);
}
''')
        (source/'gfx.rs').write_text('''pub struct Renderer { fullbright: bool }
impl Renderer {
    fn update_lightmap(&mut self) -> u8 {
        if self.fullbright { 255 } else { 42 }
    }
}
#[test] fn existing_fullbright_option_is_preserved() {
    assert_eq!(Renderer { fullbright: true }.update_lightmap(), 255);
    let forced = std::env::var("HORIZON_FULLBRIGHT").as_deref() == Ok("1");
    assert_eq!(Renderer { fullbright: false }.update_lightmap(), if forced { 255 } else { 42 });
    let state = crate::mesher::meteor_horizon::state();
    let mut renderer = Renderer { fullbright: false };
    if !state.config().fullbright { state.toggle(1); }
    assert_eq!(renderer.update_lightmap(), 255);
    state.toggle(1);
    assert_eq!(renderer.update_lightmap(), 42);
}
''')
        # Minimal host interfaces compile the actual menu adapters without assets.
        client = original/'crates/horizon-client/src'
        client.mkdir(parents=True)
        (client.parent/'Cargo.toml').write_text('[package]\nname="horizon-client"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nhorizon-client-render={path="../horizon-client-render"}\nwinit={path="../winit"}\n')
        (client/'lib.rs').write_text('extern crate self as horizon_client;\npub use horizon_client_render::{mesher, hud};\npub type Result<T> = std::result::Result<T, ()>;\npub mod game;\n')
        shutil.copyfile(root/'tools/fixtures/menu/game.rs', client/'game.rs')
        shutil.copytree(root/'tools/fixtures/menu/game', client/'game')
        winit = original/'crates/winit/src'
        winit.mkdir(parents=True)
        (winit.parent/'Cargo.toml').write_text('[package]\nname="winit"\nversion="0.1.0"\nedition="2024"\n')
        (winit/'lib.rs').write_text('pub mod keyboard { #[derive(Clone, Copy, PartialEq, Eq, Hash)] pub enum KeyCode { ShiftRight, Escape, ArrowUp, ArrowDown, Tab, Enter, NumpadEnter, Space, KeyA } }')
        with (source/'lib.rs').open('a') as lib:
            lib.write('''pub mod hud {
                pub struct HudAtlas;
                pub struct HudBatch { pub width: f32, pub height: f32, pub labels: Vec<String> }
                impl HudBatch {
                    pub fn fill(&mut self, _: &HudAtlas, _: f32, _: f32, _: f32, _: f32, _: u32) {}
                    pub fn text(&mut self, _: &HudAtlas, text: &str, _: f32, _: f32, _: u32, _: bool) -> f32 { self.labels.push(text.into()); 0.0 }
                }
            }''')
        (original/'rust-toolchain.toml').write_bytes((root/'rust-toolchain.toml').read_bytes())
        (original/'.horizon-revision').write_text(horizon.PIN+'\n')
        run(['cargo', 'generate-lockfile', '--offline'], cwd=original)
        before = snapshot(original)
        base = ['--workspace', str(original), '--mods', str(root/'mixins')]
        report = json.loads(run([str(cli), 'check', *base, '--out', str(stage), '--trust-native-mods']).stdout)
        assert report['injections'] == 7 and report['modules'] == 2
        assert '7 passed' in report['cargo_stdout'] and '1 passed' in report['cargo_stdout']
        for xray, fullbright in [('0', '0'), ('1', '0'), ('0', '1'), ('1', '1')]:
            run(['cargo', 'test', '--workspace', '--locked', '--offline'], cwd=stage,
                env={**os.environ, 'HORIZON_XRAY': xray, 'HORIZON_FULLBRIGHT': fullbright})
        first = snapshot(stage)
        second = json.loads(run([str(cli), 'check', *base, '--out', str(stage), '--trust-native-mods']).stdout)
        assert not second['stage_changed'] and snapshot(stage) == first
        assert snapshot(original) == before
        plan_path = scratch/'plan.json'
        run([str(cli), 'plan', *base, '--out', str(plan_path)])
        plan = json.loads(plan_path.read_text())
        assert plan['schema'] == horizon.COMPAT['mixin_schema']
        assert plan['injections'][0]['mod_id'] == manifest['mod']['id']
        audit(original, stage, plan_path)
        (original/'.horizon-revision').write_text('incompatible\n')
        bad = subprocess.run([str(cli), 'apply', *base, '--out', str(stage)], capture_output=True, text=True)
        assert bad.returncode and json.loads(bad.stderr)['code'] == 'E_REVISION'
        assert snapshot(stage) == first
        (original/'.horizon-revision').write_text(horizon.PIN+'\n')
        source = original/'crates/horizon-client-render/src/mesher.rs'
        source.write_text(source.read_text().replace('fn mesh(', 'fn renamed_mesh('))
        bad = subprocess.run([str(cli), 'apply', *base, '--out', str(stage)], capture_output=True, text=True)
        assert bad.returncode and json.loads(bad.stderr)['code'] == 'E_TARGET_NOT_FOUND'
        assert snapshot(stage) == first
    print(json.dumps({'status': 'ok', 'compiled_runtime_and_original_body': True,
                      'only_planned_asts_changed': True, 'deterministic_bytes_and_mtimes': True,
                      'incompatible_revision_and_target_rejected': True}, indent=2))


if __name__ == '__main__':
    main()
