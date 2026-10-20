# Parkbound

A free Windows theme park simulation in Rust, inspired by the classic isometric construction and management genre. **Parkbound is a temporary working title.**

Source repository: [strijbosmh-afk/RailRust-Tycoon](https://github.com/strijbosmh-afk/RailRust-Tycoon). **Continuing on another PC? Start with [RESUME_DEVELOPMENT.md](RESUME_DEVELOPMENT.md)** for setup, build commands, architecture, current status, and the next implementation steps.

Version 0.3.0 completes the elevated-path and underground-track overlap proof. Explore a 32 by 32 terraced woodland park, inspect independently identified surfaces, and switch between Surface and Underground views. Flat and Terraced presets preserve the same objects and their heights. The checkpoint passed 51 automated tests, Windows CI, render inspection, native overlap controls, and relocated-package checks. Detailed evidence and remaining foundation checks are recorded in [the overlap report](docs/OVERLAP_REPORT.md).

Next is a small height-aware path graph and seeded crowd that recovers from path deletion (PRF-02). Player construction, operating rides, finances, and park saves remain later milestones in [the gameplay checklist](docs/GAMEPLAY_CHECKLIST.md).

## Play the Windows build

Extract the entire `Parkbound-0.3.0-windows-x64.zip` archive into a folder and open `Parkbound.exe`. The verified Windows CI package ran after relocation outside the repository, with all 17 payload hashes matching. The build targets Windows 11 x64 and is packaged to run without Rust, Visual Studio, an internet connection, or an RCT installation. The game includes its procedural landscape and default font. A separate clean Windows machine without development tools remains untested; generated binaries are not stored in the source checkout.

The build is unsigned. Minimum hardware and additional supported Windows versions will be established through further testing. Current evidence belongs in [the overlap report](docs/OVERLAP_REPORT.md); the [terrain report](docs/TERRAIN_REPORT.md) and [foundation report](docs/FOUNDATION_REPORT.md) preserve the 0.2.0 and 0.1.0 results.

## Controls

| Input | Action |
| --- | --- |
| Left click | Select and inspect the nearest visible terrain, path, or track surface |
| Right or middle mouse drag | Pan the camera |
| WASD or arrow keys | Pan with the keyboard |
| Mouse wheel, + or - | Zoom |
| Q / E | Rotate left / right through four views |
| Home or Fit button | Fit the park in the available viewport |
| G | Toggle the tile grid |
| T / terrain preset button | Switch between Terraced and Flat maps |
| U / visibility button | Switch between Surface and Underground views |
| Space | Pause or resume the previous simulation speed |
| 1 / 2 / 3 | Set simulation speed to 1x / 2x / 4x |
| H / F1 | Open or close help |
| Escape | Close help, or clear selection |
| Window close button / Alt+F4 | Exit |

Tile coordinates in the inspector start at 1. Terrain heights are simulation steps; the Terraced preset uses levels 0, 2, 4, and 6. The fixture contains 12 path tiles at height 8 and nine track tiles at height 0, each with two steps of clearance. Flat terrain exposes the same track that Terraced terrain buries. Switching presets preserves elapsed time, materials, object identities, and object heights. Underground view removes terrain from rendering and picking; its reference floor is decorative and cannot be selected. Hidden selections retain their identity but do not show through occluding surfaces.

These comparison controls are not terrain or path editors. The displayed timer is elapsed simulation time, not a park calendar. Trees and the entrance arch remain decorative. Every launch starts a new exploration session.

## Develop

Use Rust 1.93.1 with the `x86_64-pc-windows-msvc` toolchain, MSVC C++ build tools, and a Windows SDK. `rust-toolchain.toml` pins Rust and the formatting, linting, and documentation components. Bevy is pinned to 0.17.3 and the dependency graph is recorded in `Cargo.lock`.

```powershell
cargo run --release --locked -p park_game
cargo fmt --all --check
cargo clippy --release --locked --workspace --all-targets -- -D warnings
cargo test --release --locked --workspace
pwsh -NoProfile -File scripts/package-windows.ps1
```

The first Bevy build compiles the rendering engine and takes substantially longer than subsequent changes. On a machine with 8 GB RAM, use `-j 2` with Cargo builds if memory becomes constrained.

For an automated render check, run the executable with `--capture-frame` followed by an absolute `.png` output path. It waits for rendering, saves a frame, and exits. Use `--capture-scene overview|selection|path|track|help`, `--capture-map flat|terraced`, `--capture-view surface|underground`, and `--capture-rotation 0|1|2|3` to choose the diagnostic view. Add `--capture-small-window` for the 1024 by 680 minimum window. These checks inspect rendered output and do not replace testing normal mouse and keyboard controls; detailed commands are in the resume guide.

`park_sim` owns engine-independent terrain, stable surface-object identities, atomic clearance validation, and simulation time. `park_game` owns the isometric projection, shared rendering/picking geometry, visibility, controls, and UI. A separate content crate will be introduced when there are authored ride/scenario definitions to validate.

## Project documents

- [Resume development](RESUME_DEVELOPMENT.md): complete handoff and setup on another Windows PC.
- [Development plan](DEVELOPMENT_PLAN.md): milestones and completion gates.
- [Gameplay checklist](docs/GAMEPLAY_CHECKLIST.md): feature inventory and acceptance criteria.
- [Product decisions](docs/PRODUCT_DECISIONS.md): confirmed choices, working defaults, and open decisions.
- [Overlap report](docs/OVERLAP_REPORT.md): current 0.3.0 overlap proof, evidence, and acceptance gaps.
- [Terrain report](docs/TERRAIN_REPORT.md): historical 0.2.0 terrain proof and verification.
- [Foundation report](docs/FOUNDATION_REPORT.md): historical 0.1.0 evidence and acceptance gaps.
- [Asset provenance](docs/ASSET_PROVENANCE.md): original assets and third-party attribution.

## Distribution and licensing

All planned gameplay will be included in the free download. The project's source-code license has not yet been selected; free binary distribution does not imply an open-source license. Dependency and embedded-font notices accompany the packaged build. No original RollerCoaster Tycoon code, graphics, music, or scenario files are included.

Packaging creates a Windows ZIP and SHA-256 checksums under `dist`. It does not upload or publish a release. The [Windows CI workflow](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions) runs checks and prepares a build artifact; it does not create a public release. Check the run for the exact source commit rather than assuming an earlier successful run covers a new build.
