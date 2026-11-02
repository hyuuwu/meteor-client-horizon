# Validation

Run on Linux on 2026-11-01 with the installed Rust/Cargo **1.99.1**.
The repository requests Rust 1.95.0; that toolchain was not installed locally,
so local results do not establish a Rust 1.95.0 pass. CI installs that version.
Horizon and horizon-mixins revisions remain pinned in `compatibility.json`.

| Check | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Pass |
| `cargo test --workspace --locked --offline` | Three standalone runtime tests pass |
| `python3 tools/test_template.py` | Pass: actual helpers compile; all four launch-switch combinations, normal execution/cancellation, ore faces, border filtering, unchanged world input, exact AST audit, deterministic staging and revision/target rejection |
| `python3 tools/test_stage.py --horizon target/upstream/horizon` | Pass: complete pinned export, exact cancellable/noncancellable HEAD and module audit, repeatability, unchanged upstream checkout |
| `python3 tools/horizon.py check --horizon target/upstream/horizon` | Pass: actual client binary type-checks |
| `python3 tools/horizon.py test --horizon target/upstream/horizon` | Pass: six embedded runtime tests in the actual renderer library |
| `python3 tools/package.py` | Source-only bundle created under ignored `target/` |

The native check reports two unused-function warnings from the shared runtime
copy in the lightmap module; the standalone Clippy check with warnings denied passes.

For this run, `HORIZON_MIXINS` pointed to the pinned checkout under
`target/upstream/horizon-mixins`. Upstream sources were exported with `git archive`
into ignored trees. No Minecraft assets or player resources were accessed.

No game was launched. Rendering, shader packs, distant LOD terrain and gameplay
remain manual checks. Xray only filters section terrain meshes; entities,
particles and distant LOD meshes remain unchanged. It cannot reveal blocks that
a server has not sent. Launch switches are read once and require a restart.
Fullbright retains the original renderer lightmap path and existing option.
These results apply only to the pinned Horizon sources, not other revisions or
arbitrary combinations of third-party mods.
