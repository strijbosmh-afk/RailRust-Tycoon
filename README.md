# Parkbound

A free Windows theme park simulation in Rust, inspired by the classic isometric construction and management genre. **Parkbound is a temporary working title.**

Source repository: [strijbosmh-afk/RailRust-Tycoon](https://github.com/strijbosmh-afk/RailRust-Tycoon). **Continuing on another PC? Start with [RESUME_DEVELOPMENT.md](RESUME_DEVELOPMENT.md)** for setup, build commands, architecture, current status, and the next implementation steps.

Version 0.4.0 adds a small seeded crowd to the 32 by 32 park. Twelve guests choose reachable destinations and walk a connected elevated path network with junctions, a loop, and branches. Delete a selected path to observe rerouting or a defined lost state, then restore the fixture paths to let guests recover. Flat/Terraced terrain and Surface/Underground inspection remain available.

The PRF-02 navigation proof passed 72 automated tests, eleven inspected engine captures, and native path-removal/restoration controls; evidence is tracked in [the crowd report](docs/CROWD_REPORT.md). The physical Delete shortcut still needs a retest; use the Remove path button for the verified deletion flow. Full construction, ride queues and boarding, operating rides, finances, and park saves remain later milestones in [the gameplay checklist](docs/GAMEPLAY_CHECKLIST.md).

## Play the Windows build

Extract the entire `Parkbound-0.4.0-windows-x64.zip` archive into a folder and open `Parkbound.exe`. The build targets Windows 11 x64 and is packaged to run without Rust, Visual Studio, an internet connection, or an RCT installation. The game includes its procedural landscape, guest art, and default font. Check the crowd report for this version's actual package verification. A separate clean Windows machine without development tools remains untested; generated binaries stay in the ignored local `dist` folder.

The build is unsigned. Minimum hardware and additional supported Windows versions will be established through further testing. The [overlap report](docs/OVERLAP_REPORT.md), [terrain report](docs/TERRAIN_REPORT.md), and [foundation report](docs/FOUNDATION_REPORT.md) preserve earlier results; they do not verify changed 0.4.0 code.

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
| Delete / Remove path button | Remove the selected path segment; guests reroute or enter a lost state |
| R / Restore paths button | Restore missing demonstration paths without resetting time or guests |
| Space | Pause or resume the previous simulation speed |
| 1 / 2 / 3 | Set simulation speed to 1x / 2x / 4x |
| H / F1 | Open or close help |
| Escape | Close help, or clear selection |
| Window close button / Alt+F4 | Exit |

Tile coordinates in the inspector start at 1. Terrain heights are simulation steps; the Terraced preset uses levels 0, 2, 4, and 6. The demonstration contains 37 path tiles at height 8 and nine track tiles at height 0, each with two steps of clearance. Guests connect only cardinally adjacent paths at the same height. The interface reports walking guests, visits, and lost guests. Guests are visual simulation markers and are not individually selectable yet.

Flat terrain exposes the same track that Terraced terrain buries. Switching presets preserves elapsed time, materials, object identities, and object heights. Underground view removes terrain from rendering and picking; its reference floor is decorative and cannot be selected. Hidden selections retain their identity but do not show through occluding surfaces.

Delete and Restore paths exercise the navigation proof; they are not a full path editor. Restored segments receive new IDs, while surviving paths and guests retain theirs. A lost guest keeps its last location and can recover onto a path at that location or an adjacent tile at the same height. The displayed timer is elapsed simulation time, not a park calendar. Trees and the entrance arch remain decorative. Every launch starts a fresh seeded demonstration.

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

For an automated render check, run the executable with `--capture-frame` followed by an absolute `.png` output path. It waits for rendering, saves a frame, and exits. Use `--capture-scene overview|selection|path|track|crowd|lost|recovered|help`, `--capture-map flat|terraced`, `--capture-view surface|underground`, and `--capture-rotation 0|1|2|3`. `--capture-ticks` accepts 0–36000 complete ticks (default 600) before the paused scene. Add `--capture-small-window` for the 1024 by 680 minimum window. Lost/recovered presets exercise a controlled disconnection and restoration. These checks inspect output and do not replace normal mouse and keyboard testing; detailed commands are in the resume guide.

`park_sim` owns engine-independent terrain, stable identities, clearance, path connectivity, seeded guest state, routing, and complete fixed simulation ticks. `park_game` owns the isometric projection, shared surface rendering/picking, a separate guest mesh, visibility, controls, and UI. A separate content crate will be introduced when there are authored ride/scenario definitions to validate.

## Project documents

- [Resume development](RESUME_DEVELOPMENT.md): complete handoff and setup on another Windows PC.
- [Development plan](DEVELOPMENT_PLAN.md): milestones and completion gates.
- [Gameplay checklist](docs/GAMEPLAY_CHECKLIST.md): feature inventory and acceptance criteria.
- [Product decisions](docs/PRODUCT_DECISIONS.md): confirmed choices, working defaults, and open decisions.
- [Crowd report](docs/CROWD_REPORT.md): current 0.4.0 path and crowd proof, verification, and remaining checks.
- [Overlap report](docs/OVERLAP_REPORT.md): historical 0.3.0 overlap proof and evidence.
- [Terrain report](docs/TERRAIN_REPORT.md): historical 0.2.0 terrain proof and verification.
- [Foundation report](docs/FOUNDATION_REPORT.md): historical 0.1.0 evidence and acceptance gaps.
- [Asset provenance](docs/ASSET_PROVENANCE.md): original assets and third-party attribution.

## Distribution and licensing

All planned gameplay will be included in the free download. The project's source-code license has not yet been selected; free binary distribution does not imply an open-source license. Dependency and embedded-font notices accompany the packaged build. No original RollerCoaster Tycoon code, graphics, music, or scenario files are included.

Packaging creates a Windows ZIP and SHA-256 checksums under `dist`. It does not upload or publish a release. The [Windows CI workflow](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions) runs checks and prepares a build artifact; it does not create a public release. Check the run for the exact source commit rather than assuming an earlier successful run covers a new build.
