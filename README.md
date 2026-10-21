# Parkbound

A free Windows theme park simulation in Rust, inspired by the classic isometric construction and management genre. **Parkbound is a temporary working title.**

Source repository: [strijbosmh-afk/RailRust-Tycoon](https://github.com/strijbosmh-afk/RailRust-Tycoon). **Continuing on another PC? Start with [RESUME_DEVELOPMENT.md](RESUME_DEVELOPMENT.md)** for setup, build commands, architecture, current status, and the next implementation steps.

Version 0.12.0 completes the bounded single-tile terrain raising/lowering implementation in the existing 32 × 32 park. B/Raise and V/Lower change a visible Grass or Sand column by one step, with a preview and atomic clearance rejection. All 236 tests, formatting/strict lint/build, ten inspected captures, production continuation, exact-source Windows CI, and package relocation passed. Native terrain controls remain unverified because Windows activation/capture errors prevented input. See [the terrain editing report](docs/TERRAIN_EDITING_REPORT.md). BLD-06 remains in progress for brushes, slopes, water editing, and construction economics.

PRF-05 remains in progress. **No reference PC is selected:** the user rejected this poorly performing development PC as a reference. Its [historical 0.7.0 measurements](docs/PERFORMANCE_REPORT.md) are local diagnostics only. Hardware requirements and 60 FPS acceptance wait for an agreed representative PC. The full 5,000-guest/50-ride target remains unproven: diagnostics currently use one circuit, and normal play retains its 32 × 32 map. The [save report](docs/SAVE_REPORT.md) preserves accepted 0.6.0 continuation evidence. Native Backup, Save/Load buttons, O/F/buttons, physical Delete and Page Up/Down still need retests; F5/F9 and height buttons passed in 0.11.0. Track editing, boarding, ride revenue/ratings, and finances remain later milestones.

## Play the Windows build

Extract the complete `Parkbound-0.12.0-windows-x64.zip` archive and open `Parkbound.exe`. The Windows 11 x64 package includes dependency/font/Rust notices and its required MSVC runtime. Verification matched 27 payload hashes/eighteen guides; relocated terrain continuation and a byte-identical minimum-window terrain capture passed from an unrelated working directory. The [terrain editing report](docs/TERRAIN_EDITING_REPORT.md) records its full local path. A clean Windows machine without development tools remains untested. Binaries stay in ignored `dist`; the adjacent `.zip.sha256` is authoritative after final guide refresh.

The build is unsigned. Minimum hardware and additional supported Windows versions will be established through further testing. Earlier milestone reports preserve their own results; they do not verify changed 0.12.0 code.

## Controls

| Input | Action |
| --- | --- |
| Left click | Inspect, place/remove one path, or raise/lower one terrain column according to the current tool |
| P / Path button | Enter level-path placement mode |
| X / Demolish button | Enter visible-path demolition mode |
| B / Raise button | Raise the visible Grass/Sand terrain column by one step |
| V / Lower button | Lower the visible Grass/Sand terrain column by one step |
| Page Up / Page Down / height buttons | Raise/lower the path construction plane; range 0–30, default 8; independent of terrain height |
| Right or middle mouse drag | Pan the camera |
| WASD or arrow keys | Pan with the keyboard |
| Mouse wheel, + or - | Zoom |
| Q / E | Rotate left / right through four views |
| Home or Fit button | Fit the park in the available viewport |
| G | Toggle the tile grid |
| T / terrain preset button | Explicitly reset heights to the other terrain preset when clearance permits |
| U / visibility button | Switch between Surface and Underground views |
| Delete / Remove path button | Remove the selected path if it is still visible; guests reroute or enter a lost state |
| R / Restore paths button | Restore missing demonstration paths without resetting time or guests |
| O / Open or Close button | Open the train, or let its current lap finish and park at the station |
| F / Find button | Center the train circuit at 150% zoom |
| F5 / Save button | Save the current simulation to the single slot |
| F9 / Load button | Load the primary slot and pause |
| Shift+F9 / Backup button | Explicitly load the previous valid backup and pause |
| Space | Pause or resume the previous simulation speed |
| 1 / 2 / 3 | Set simulation speed to 1x / 2x / 4x |
| H / F1 | Open or close help |
| Escape / Inspect button | Close help first, otherwise return to Inspect; Escape also clears selection |
| Window close button / Alt+F4 | Exit |

Tile coordinates in the inspector start at 1. Terrain heights are simulation steps; the Terraced preset uses levels 0, 2, 4, and 6. The crowd demonstration retains 37 path tiles at height 8 and nine original track tiles at height 0. The new circuit's rails range from height 8 to 16. Guests connect only cardinally adjacent paths at the same height. The interface reports walking guests, visits, and lost guests. Guests and train cars are not individually selectable; their supporting track/path surfaces are.

Flat terrain exposes the same track that Terraced terrain buries. Switching presets preserves elapsed time, materials, object identities, and object heights. Underground view removes terrain from rendering and picking; its reference floor is decorative and cannot be selected. Hidden selections retain their identity but do not show through occluding surfaces.

P places level paths on the chosen height plane with green/red preview and rejection text; X removes visible paths with an orange preview. Click once per tile: holding or dragging does not paint. A hidden target has no ghost; rotate or use Underground view to inspect it. New and rebuilt segments receive fresh IDs. R restores only missing demonstration paths, preserving player paths, time, and guests. A lost guest retains its location and can recover onto a same-height path there or on a cardinal neighbor. Paths at different heights do not connect without a future slope tool. Construction has no cost/refund or queue rules yet.

B/Raise and V/Lower target the picked terrain column, including a visible cliff face. Each click changes height by one step within 0–32 while preserving Grass/Sand material. Water and Underground terrain editing are blocked. Green plus/blue minus indicators show the direction; red X indicates rejection. Paths, rails, and guests stay in place, and invalid clearance leaves the park unchanged. Successful edits display Custom; T is an explicit preset reset. No brushes, water editing, slopes, costs, or refunds are included.

The displayed timer is elapsed simulation time, not a park calendar. Trees and the entrance arch remain decorative. Every launch starts a fresh seeded demonstration; use Load to continue a saved park. Startup does not overwrite an existing save.

The train initially waits 90 ticks at the station, then runs while open. Closing at the station parks it immediately; closing during travel finishes the current lap. Space freezes both the train and guests, so a paused train cannot complete its requested return until resume. Reopening starts another station wait. Motion uses tuned integer rules along the track; this is not a realistic coaster physics model or a complete ride system.

## Save and continue

Saves live in `%LOCALAPPDATA%\Parkbound\saves`, separate from the executable and shared OneDrive source folder. `park.park.json` is the primary slot; `park.park.json.bak` is the previous valid backup. Saving replaces the one slot, including player-built/removed paths and custom terrain heights. Backup loading is explicit, and a corrupt primary is never rotated into the backup. A successful load resets camera/selection/view and tool mode, pauses simulation, and resumes at 1x when requested; authoritative train and guest state is retained. Camera, tool height/ghost, UI settings, and pause/speed preference are not saved. A same-update Save includes the world click; Load/Backup takes precedence over pending edits even when loading fails.

The format is versioned JSON for the current demonstration. The app accepts 32 × 32 parks; unsupported versions, invalid references, and corrupt files report an error without replacing the running park. There is no autosave or save browser yet. Copy closed-game save files separately to continue on another PC; GitHub and the shared source checkout do not transfer LocalAppData automatically. For QA or an explicitly chosen portable folder, launch with `--save-dir` followed by an absolute path. See the [resume guide](RESUME_DEVELOPMENT.md) for backup and lock recovery.

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

Save diagnostics use `--save-check write` and a separate `--save-check verify` process with a new absolute `--save-dir`. Persistence render scenes `saved|loaded|backup|load-error` also require an explicit isolated save directory; `saved` writes its slot. See [the save report](docs/SAVE_REPORT.md) for exact setup and expected evidence.

Path-tool capture scenes are `path-valid|path-blocked|path-built|path-demolish|path-removed|path-recovered`. They call production tool commands and preserve the existing rotation/view/minimum-window options. `--save-check write-paths` followed by a separate `--save-check verify-paths` process tests construction/demolition continuation in a new isolated directory and writes `verification-paths.txt`. See [the completed path report](docs/PATH_CONSTRUCTION_REPORT.md) for that contract and evidence.

New terrain scenes are `terrain-raise|terrain-lower|terrain-raised|terrain-lowered|terrain-blocked|terrain-water|terrain-underground`. Terrain continuation uses `--save-check write-terrain` then a separate `--save-check verify-terrain` process with a new isolated absolute save directory, producing `verification-terrain.txt`. These terrain and retained cross-version continuation checks passed in the production executable; [the terrain editing report](docs/TERRAIN_EDITING_REPORT.md) explains the intended proof.

For repeatable measurements, build Release and run `scripts/measure-performance.ps1 -OutputDirectory <absolute-new-directory>` through PowerShell 7. It runs baseline/save-scale/stress CPU cases and two baseline frame profiles sequentially, records safe hardware metadata and sampled process memory, and preserves unsupported/incomplete outcomes. `-SkipFrames`, `-Cases`, and repetition/sample options allow targeted runs. Frame values describe main-loop pacing, not GPU or physically presented-frame timing. See [the performance report](docs/PERFORMANCE_REPORT.md) before interpreting results.

The Windows ZIP also includes the wrapper. With PowerShell 7 installed, run `.\measure-performance.ps1 -Executable .\Parkbound.exe -OutputDirectory <absolute-new-directory>` from the extracted folder. PowerShell is only needed for this optional measurement tool.

`park_sim` owns engine-independent terrain, stable identities, clearance, path connectivity, seeded guests, sampled circuit geometry, train operation, complete fixed ticks, and validated save encoding/decoding. `park_game` owns filesystem storage, projection, shared surface rendering/picking, separate guest/train meshes, visibility, controls, and UI. A separate content crate will be introduced when authored ride/scenario definitions justify it.

## Controlled rendering and GPU diagnostics

Launch `parkbound.exe --render-case stress --render-camera crowd` for the isolated large fixture. Cases are `baseline`, `save-scale`, and `stress`; cameras are `overview`, `crowd`, and `ride`. `--render-rotation 0..3` and `--render-underground` select the view. Camera/inspection/speed remain available; editing and persistence are disabled in previews. Normal launches and saves stay 32 × 32.

Add `--profile-gpu` to a `--profile-frames <absolute-new.json>` run to request actual instrumented GPU timestamps where supported. Missing/unsupported data remains explicit. GPU spans, render-span CPU, main-loop pacing, and unmeasured presentation/upload are distinct. With the wrapper, `-SkipCpu -RenderCase stress -RenderCamera crowd -GpuDiagnostics -ProfileSamples 120` runs only the two frame modes; provide a new absolute `-OutputDirectory`. The historical [large-map/GPU report](docs/LARGE_MAP_GPU_REPORT.md) records the completed 0.9.0 evidence and diagnostic methods. Version 0.10.0 targets reduced static geometry submission in focused large-map views. Reports retain all generated landscape geometry and separately expose a final `render_work.static_visibility` snapshot with selected/hidden chunks and geometry. These are conservative CPU eligibility counts, not actual GPU draw calls or visible pixels; counters include startup/warmup. All 193 release tests, strict source checks, eight exact image comparisons outside version text, bidirectional 0.9/0.10 continuation, six GPU profiles, CI, and package relocation passed. The focused stress camera selects 13 of 212 chunks: 149,556 of 2,251,764 generated static vertices. This is reduced CPU eligibility, not a measured FPS or upload gain; see [the historical report](docs/STATIC_VISIBILITY_REPORT.md).

## Project documents

- [Resume development](RESUME_DEVELOPMENT.md): complete handoff and setup on another Windows PC.
- [Development plan](DEVELOPMENT_PLAN.md): milestones and completion gates.
- [Gameplay checklist](docs/GAMEPLAY_CHECKLIST.md): feature inventory and acceptance criteria.
- [Product decisions](docs/PRODUCT_DECISIONS.md): confirmed choices, working defaults, and open decisions.
- [Terrain editing report](docs/TERRAIN_EDITING_REPORT.md): completed 0.12.0 single-tile tools, verification, and remaining native/release gaps.
- [Path construction report](docs/PATH_CONSTRUCTION_REPORT.md): completed 0.11.0 controls, command/persistence contract, and verification.
- [Static visibility report](docs/STATIC_VISIBILITY_REPORT.md): completed 0.10.0 scope, chunk/viewport contract, and verification.
- [Large-map/GPU report](docs/LARGE_MAP_GPU_REPORT.md): completed 0.9.0 preview, diagnostic methods, CLI, and evidence.
- [Optimization report](docs/OPTIMIZATION_REPORT.md): completed 0.8.0 cache/rendering work and verification.
- [Performance report](docs/PERFORMANCE_REPORT.md): historical 0.7.0 local diagnostics, with the corrected hardware designation.
- [Save report](docs/SAVE_REPORT.md): historical 0.6.0 continuation proof and format/storage contract.
- [Train report](docs/TRAIN_REPORT.md): historical 0.5.0 circuit proof, verification, and remaining checks.
- [Crowd report](docs/CROWD_REPORT.md): historical 0.4.0 navigation proof and evidence.
- [Overlap report](docs/OVERLAP_REPORT.md): historical 0.3.0 overlap proof and evidence.
- [Terrain report](docs/TERRAIN_REPORT.md): historical 0.2.0 terrain proof and verification.
- [Foundation report](docs/FOUNDATION_REPORT.md): historical 0.1.0 evidence and acceptance gaps.
- [Asset provenance](docs/ASSET_PROVENANCE.md): original assets and third-party attribution.

## Distribution and licensing

All planned gameplay will be included in the free download. The project's source-code license has not yet been selected; free binary distribution does not imply an open-source license. Dependency and embedded-font notices accompany the packaged build. No original RollerCoaster Tycoon code, graphics, music, or scenario files are included.

Packaging creates a Windows ZIP and SHA-256 checksums under `dist`. It does not upload or publish a release. The retained 0.6.0 implementation is in [draft PR #4](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/4), with [passing implementation CI](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39422425034). The historical 0.7.0 implementation `e93eb99` is in [draft PR #5](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/5), stacked on the save review. Its 147 local release tests and strict source checks passed; [implementation CI](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39425219126) passed for that exact commit. CI prepares a review artifact; it does not create a public release. Check the exact commit's run after further code changes.

Historical 0.8.0 runtime implementation `9460a9` and the PowerShell-only validator correction `3f1ea88` are pushed in [draft PR #6](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/6), stacked on PR #5. All 159 release tests, formatting, strict linting, and release compilation passed; [Windows CI run 39429282120](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39429282120) passed for `3f1ea88`. The bounded optimization is complete.

Historical 0.9.0 implementation `5d49b7e` is in [draft PR #7](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/7), stacked on PR #6. All 178 release tests and strict source checks passed, as did [Windows CI run 39435077589](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39435077589), including packaging, production save continuation, CPU reporting, and artifact upload. Relocated package checks passed; the final refresh verified all fifteen current guides and all 24 payload hashes. Full-target hardware acceptance remains open. Later heads need their own CI status, and the final ZIP hash is in its adjacent `.sha256` file.

Historical 0.10.0 source `ff2857e` is on `codex/static-terrain-visibility` in [draft PR #8](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/8), stacked on PR #7. [Windows CI run 39440918757](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39440918757) passed for that source. The subsequent checkpoint passed [run 39442909052](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39442909052). Historical 0.11.0 source `eed600b1f86efb6c92dccb1a589167caeb44951f` is pushed in [draft PR #9](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/9), stacked on PR #8. [Windows CI run 39447839922](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39447839922) passed for that source, along with local package/hash/relocation checks. Later documentation heads have separate CI status. Full-target hardware acceptance remains open.

Current 0.12.0 work is on `codex/terrain-editing` from completed 0.11.0 checkpoint `13c4063`. Runtime source `e5dbc0e61a9fdc3870f89a9c664ed13fbf4c4fc1` is pushed in [draft PR #10](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/10), stacked on still-unmerged PR #9/`codex/path-construction` at `13c4063`. All 236 local release tests passed (162 game/74 simulation), and final strict all-target release Clippy passed. The final source also passed all 236 tests in Windows CI; final local rustfmt passed. The production release build passed. Bidirectional 0.11/0.12 continuation and write-terrain/verify-terrain passed, including exact primary/backup state, Custom/paused loading, 3,600 ticks, commands, and clearance rejection. All ten captures exited 0 and were inspected without a blocking visual defect. [Windows CI run 39557465779](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39557465779) passed for exact runtime source `e5dbc0e61a9fdc3870f89a9c664ed13fbf4c4fc1`, including formatting, strict lint, all 236 tests, packaging, ordinary/path/terrain continuation, CPU-reporting smoke, and artifact uploads. Package verification matched all 27 payload hashes and eighteen guides. A fresh extraction path containing spaces and é, launched from `C:\Windows`, passed terrain primary/backup/Custom/paused continuation for 3,600 ticks plus commands/clearance. Its terrain-raised/rotation-2/Surface/Terraced/1024 × 680 PNG matched the source image byte-for-byte. Native terrain controls remain unverified after capture/activation errors prevented input. The bounded terrain slice is verified through source/runtime/delivery checks; native controls retain the explicit gap. The recommended next design is one-step sloped paths: explicit endpoint heights, shared clearance, guest traversal, and save compatibility before implementation. This is a bounded BLD-08 proposal; queues, boarding, and the complete staged game remain later work.
