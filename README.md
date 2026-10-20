# Parkbound

A free Windows theme park simulation in Rust, inspired by the classic isometric construction and management genre. **Parkbound is a temporary working title.**

Source repository: [strijbosmh-afk/RailRust-Tycoon](https://github.com/strijbosmh-afk/RailRust-Tycoon). **Continuing on another PC? Start with [RESUME_DEVELOPMENT.md](RESUME_DEVELOPMENT.md)** for setup, build commands, architecture, current status, and the next implementation steps.

Version 0.5.0 adds a three-car train on a closed station/lift/drop/brake circuit. Open it to run laps, close it to finish the current lap and park, and use Find to center the circuit. Twelve seeded guests continue exploring the elevated path network, with path removal and recovery retained. Flat/Terraced terrain and Surface/Underground inspection remain available.

The PRF-03 automatic circuit proof is verified through 99 tests, sixteen inspected captures, and a normal launch showing a completed lap; details are in [the train report](docs/TRAIN_REPORT.md). Native O/F/button checks remain open after desktop activity interrupted testing, and physical Delete still needs a retest. These gaps are separate from the demonstrated circuit behavior. Save/reload is next; track editing, boarding, ride revenue/ratings, and finances remain later milestones in [the gameplay checklist](docs/GAMEPLAY_CHECKLIST.md).

## Play the Windows build

Extract the entire `Parkbound-0.5.0-windows-x64.zip` archive into a folder and open `Parkbound.exe`. The build targets Windows 11 x64 and is packaged to run without Rust, Visual Studio, an internet connection, or an RCT installation. The game includes its procedural landscape, guests, train art, and default font. Its ZIP passed relocation with all 19 payload hashes and eleven guides verified; the extracted executable rendered successfully from an unrelated working directory. A separate clean Windows machine without development tools remains untested. Generated binaries stay in the ignored local `dist` folder; see [the train report](docs/TRAIN_REPORT.md) for exact evidence.

The build is unsigned. Minimum hardware and additional supported Windows versions will be established through further testing. The crowd, overlap, terrain, and foundation reports preserve earlier results; they do not verify changed 0.5.0 code.

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
| O / Open or Close button | Open the train, or let its current lap finish and park at the station |
| F / Find button | Center the train circuit at 150% zoom |
| Space | Pause or resume the previous simulation speed |
| 1 / 2 / 3 | Set simulation speed to 1x / 2x / 4x |
| H / F1 | Open or close help |
| Escape | Close help, or clear selection |
| Window close button / Alt+F4 | Exit |

Tile coordinates in the inspector start at 1. Terrain heights are simulation steps; the Terraced preset uses levels 0, 2, 4, and 6. The crowd demonstration retains 37 path tiles at height 8 and nine original track tiles at height 0. The new circuit's rails range from height 8 to 16. Guests connect only cardinally adjacent paths at the same height. The interface reports walking guests, visits, and lost guests. Guests and train cars are not individually selectable; their supporting track/path surfaces are.

Flat terrain exposes the same track that Terraced terrain buries. Switching presets preserves elapsed time, materials, object identities, and object heights. Underground view removes terrain from rendering and picking; its reference floor is decorative and cannot be selected. Hidden selections retain their identity but do not show through occluding surfaces.

Delete and Restore paths exercise the navigation proof; they are not a full path editor. Restored segments receive new IDs, while surviving paths and guests retain theirs. A lost guest keeps its last location and can recover onto a path at that location or an adjacent tile at the same height. The displayed timer is elapsed simulation time, not a park calendar. Trees and the entrance arch remain decorative. Every launch starts a fresh seeded demonstration.

The train initially waits 90 ticks at the station, then runs while open. Closing at the station parks it immediately; closing during travel finishes the current lap. Space freezes both the train and guests, so a paused train cannot complete its requested return until resume. Reopening starts another station wait. Motion uses tuned integer rules along the track; this is not a realistic coaster physics model or a complete ride system.

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

For an automated render check, run the executable with `--capture-frame` followed by an absolute `.png` output path. It waits for rendering, saves a frame, and exits. Existing `--capture-scene overview|selection|path|track|crowd|lost|recovered|help` presets remain; train presets are `train|train-lift|train-drop|train-brake|train-return|train-closed`. Use `--capture-map flat|terraced`, `--capture-view surface|underground`, and `--capture-rotation 0|1|2|3`. `--capture-ticks` accepts 0–36000 initial complete ticks (default 600); named train phases may advance additional bounded ticks before capture. Add `--capture-small-window` for the 1024 by 680 minimum window. These checks inspect output and do not replace normal mouse and keyboard testing; detailed commands are in the resume guide.

`park_sim` owns engine-independent terrain, stable identities, clearance, path connectivity, seeded guests, sampled circuit geometry, train operation, and complete fixed ticks. `park_game` owns projection, shared surface rendering/picking, separate guest/train meshes, visibility, controls, and UI. A separate content crate will be introduced when authored ride/scenario definitions justify it.

## Project documents

- [Resume development](RESUME_DEVELOPMENT.md): complete handoff and setup on another Windows PC.
- [Development plan](DEVELOPMENT_PLAN.md): milestones and completion gates.
- [Gameplay checklist](docs/GAMEPLAY_CHECKLIST.md): feature inventory and acceptance criteria.
- [Product decisions](docs/PRODUCT_DECISIONS.md): confirmed choices, working defaults, and open decisions.
- [Train report](docs/TRAIN_REPORT.md): current 0.5.0 circuit proof, verification, and remaining checks.
- [Crowd report](docs/CROWD_REPORT.md): historical 0.4.0 navigation proof and evidence.
- [Overlap report](docs/OVERLAP_REPORT.md): historical 0.3.0 overlap proof and evidence.
- [Terrain report](docs/TERRAIN_REPORT.md): historical 0.2.0 terrain proof and verification.
- [Foundation report](docs/FOUNDATION_REPORT.md): historical 0.1.0 evidence and acceptance gaps.
- [Asset provenance](docs/ASSET_PROVENANCE.md): original assets and third-party attribution.

## Distribution and licensing

All planned gameplay will be included in the free download. The project's source-code license has not yet been selected; free binary distribution does not imply an open-source license. Dependency and embedded-font notices accompany the packaged build. No original RollerCoaster Tycoon code, graphics, music, or scenario files are included.

Packaging creates a Windows ZIP and SHA-256 checksums under `dist`. It does not upload or publish a release. The train implementation is saved in [draft PR #3](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/3), with [passing Windows CI](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39417086959). CI prepares a review artifact; it does not create a public release. Check the exact commit's run after further code changes.
