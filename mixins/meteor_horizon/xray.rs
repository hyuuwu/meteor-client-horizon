// Uses Horizon's existing model, tint and face builders on a private padded buffer.
fn xray(mesher: &mut Mesher<'_>, region: &[[Option<&crate::chunk::Chunk>; 3]; 3], sy: i32) -> std::ops::ControlFlow<SectionMesh> {
    if !meteor_horizon::config().xray {
        return std::ops::ControlFlow::Continue(());
    }
    let center = region[1][1].expect("centre chunk");
    let mut pad = Padded::new(region, sy);
    let mut selected = std::collections::HashMap::new();
    meteor_horizon::filter_blocks(&mut pad.blocks, |id| {
        *selected.entry(id).or_insert_with(|| {
            mesher.assets.registry.state(horizon_world::registry::BlockStateId(id))
                .is_some_and(|state| meteor_horizon::is_ore(&state.name))
        })
    });
    pad.light.fill(0xff);
    // Every section is traversable after hiding terrain, including enclosed ores.
    let mut out = SectionMesh { visibility: u64::MAX, ..SectionMesh::default() };
    let base_y = center.min_y() + sy * 16;
    let mut tints = Tints::new(region, mesher.biomes, mesher.zoom, mesher.blend_radius,
        [center.x, center.z, base_y]);
    let mut models = Vec::new();
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                let id = pad.blocks[pi(x, y, z)];
                if id == 0 { continue; }
                let state = &mesher.tables.info[id as usize];
                let world = horizon_core::position::BlockPos::new(center.x * 16 + x, base_y + y, center.z * 16 + z);
                if mesher.render.model_catalog.models_at(horizon_world::registry::BlockStateId(id), world, &mut models).is_err() {
                    continue;
                }
                for model in &models {
                    for face in &model.faces {
                        mesher.face(&pad, &mut tints, &mut out, [x, y, z],
                            [x as f32, y as f32, z as f32], face, false, state);
                    }
                }
            }
        }
    }
    std::ops::ControlFlow::Break(out)
}
