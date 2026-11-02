# Meteor Horizon

A native Horizon mod providing ore Xray and Fullbright, inspired by the two
Meteor Client features. This is an independent Rust implementation, not the
Java Meteor client, a Fabric mod, or a complete Meteor port. No Meteor source
or artwork is included. The template and this implementation use the MIT license.

## Features

- **Xray:** renders vanilla ores (including deepslate variants, Nether ores and
  ancient debris) through hidden terrain. Ores are lit and section visibility
  stays open so enclosed deposits can render. Only temporary mesh buffers change;
  world blocks, collision, saves and server packets remain unchanged.
- **Fullbright:** forces Horizon's existing white lightmap. With the launch switch
  off, Horizon's own Fullbright setting continues to work normally.

Both launch switches default to off. Enable either independently with `1` or
`true`; restart the client to change them. There is no custom in-game menu yet.

```sh
HORIZON_XRAY=1 HORIZON_FULLBRIGHT=1 \
  ./target/horizon-cargo/fast/horizon-client --resources /absolute/path/to/prepared/resources
```

Xray only reveals blocks the client has received. Server-side ore obfuscation
still applies. Distant LOD terrain, entities, particles, fog, shader packs and
block-entity rendering are not filtered by this initial implementation; keep
LOD disabled when using Xray. Interactive rendering has not been verified.

## Build

Linux, Python 3.12+, Git, Cargo and Rust 1.95.0 are required. Exact pins are in
`compatibility.json`: Horizon Nightly 2026-10-30 (`ba6344a59cb9e755b730dcf02285fc0d33b97d3c`)
and horizon-mixins `d9c83d7096b7ea5b83678da2efdfd6161f74997e` (schema 6).

```sh
git clone https://github.com/horizon-rust/horizon ../horizon
git clone https://github.com/horizon-rust/horizon-mixins ../horizon-mixins
git -C ../horizon-mixins checkout d9c83d7096b7ea5b83678da2efdfd6161f74997e
cargo fetch --locked
cargo fetch --locked --manifest-path ../horizon-mixins/Cargo.toml
python3 tools/horizon.py stage --horizon ../horizon
cargo fetch --locked --manifest-path target/mixins/modded/Cargo.toml
python3 tools/horizon.py check --horizon ../horizon
python3 tools/horizon.py test --horizon ../horizon
python3 tools/horizon.py build --horizon ../horizon
```

Set `HORIZON_MIXINS=/absolute/path/to/pinned/horizon-mixins` for another tool
location. The Horizon Git history must contain the pin; its checked-out branch
is not changed. Only committed sources are exported into ignored `target/`
trees. All AST edits use horizon-mixins plan/apply. Original upstream checkouts
and player resources are never modified. Supply your own prepared resources
compatible with the pinned release when launching.

## Development and validation

`mixins/meteor_horizon/runtime.rs` is the standalone Cargo library and the
injected runtime source. `xray.rs` is a cancellable HEAD adapter around Horizon's
existing meshing/model helpers; `fullbright.rs` sets the renderer flag before
its original lightmap update. Compatibility hashes cover both modified files.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
python3 tools/test_template.py
python3 tools/test_stage.py --horizon ../horizon
python3 tools/horizon.py check --horizon ../horizon
python3 tools/horizon.py test --horizon ../horizon
python3 tools/package.py
```

The fixture compiles the actual helpers, exercises all four launch-switch
combinations, checks cancellation and normal execution, and audits the original
ASTs after removing only exact planned additions. The native test command runs
the embedded runtime tests in `horizon-client-render`. Packaging creates a
reproducible source-only `target/meteor_horizon-0.1.0.hnmod`; no binaries,
exported upstream sources or Minecraft assets belong in Git. See `VALIDATION.md`
for actual results and limitations.

Based on the MIT-licensed Horizon native mod template and its Veinminer-derived
tooling. [Meteor Client](https://github.com/MeteorDevelopment/meteor-client)
is a separate GPL-3.0 project; this project is not affiliated with Meteor Development.
