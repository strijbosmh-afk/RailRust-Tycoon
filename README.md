# Parkbound

A free Windows theme park simulation in Rust, inspired by the classic isometric construction and management genre. **Parkbound is a temporary working title.**

Source repository: [strijbosmh-afk/RailRust-Tycoon](https://github.com/strijbosmh-afk/RailRust-Tycoon). **Continuing on another PC? Start with [RESUME_DEVELOPMENT.md](RESUME_DEVELOPMENT.md)** for setup, build commands, architecture, current status, and the next implementation steps.

Version 0.16.0 connects queues to six reserved passenger seats, boarding, an occupied train lap, a dedicated exit, and completed public-path returns. K/Exit builds the new station-side portal; passenger/transfer state persists in strict v5 saves with explicit legacy migration. This implementation is in integration: final tests/build, runtime/native checks, captures, CI and packaging are pending. See [the ride journey report](docs/RIDE_JOURNEY_REPORT.md). The completed 0.15.0 queue-admission evidence remains in [the admission report](docs/ADMISSION_REPORT.md).

PRF-05 remains in progress. **No reference PC is selected:** the user rejected this poorly performing development PC as a reference. Its [historical 0.7.0 measurements](docs/PERFORMANCE_REPORT.md) are local diagnostics only. Hardware requirements and 60 FPS acceptance wait for an agreed representative PC. The full 5,000-guest/50-ride target remains unproven: diagnostics currently use one circuit, and normal play retains its 32 × 32 map. The [save report](docs/SAVE_REPORT.md) preserves accepted 0.6.0 continuation evidence. Native F5/F9, Save/Backup buttons, shape controls, and height buttons passed in 0.13.0. The separate Load button passed in 0.14.0, restoring a connected queue park paused with Inspect reset. New J/I/C queue/gate gestures, Page Up/Down, Shift+F9, physical Delete, terrain/ride controls, and pan/drag/DPI checks remain open. New K/Exit and passenger controls also need native verification. Track editing, ride revenue/ratings, and finances remain later milestones.

## Play the Windows build

The last verified archive is `dist\Parkbound-0.15.0-windows-x64.zip`; the new `Parkbound-0.16.0-windows-x64.zip` is pending verification. Extract the complete ZIP and open `Parkbound.exe`. Historical 0.15.0 packaging matched 30 payload hashes/21 guides and passed relocated admission continuation/rendering; those results do not verify changed 0.16.0 code. Clean-machine compatibility remains open. Binaries stay in ignored `dist`; the adjacent `.zip.sha256` identifies each final archive.

The build is unsigned. Minimum hardware and additional supported Windows versions will be established through further testing. Each milestone report records its own named-build evidence.

## Controls

| Input | Action |
| --- | --- |
| Left click | Inspect, build/demolish one public path/queue/entrance/exit, or raise/lower one terrain column according to the tool |
| P / Path button | Enter public-path placement mode |
| J / Queue button | Build a queue owned by the existing ride |
| I / Gate button | Place the ride entrance beside its station |
| K / Exit button | Place a dedicated exit beside the station; default facing South |
| C / shape or facing button | In Queue, cycle Flat → N → E → S → W and remain Queue; in Gate or Exit, rotate that portal's independent facing; otherwise select public Path and cycle its shape |
| X / Demolish button | Remove a visible public path, queue path, entrance or exit when not protected by an active transfer |
| B / Raise button | Raise the visible Grass/Sand terrain column by one step |
| V / Lower button | Lower the visible Grass/Sand terrain column by one step |
| Page Up / Page Down / height buttons | Set Flat/entrance/exit height 0–30 or ramp low height 0–29, default 8; independent of terrain height |
| Right or middle mouse drag | Pan the camera |
| WASD or arrow keys | Pan with the keyboard |
| Mouse wheel, + or - | Zoom |
| Q / E | Rotate left / right through four views |
| Home or Fit button | Fit the park in the available viewport |
| G | Toggle the tile grid |
| T / terrain preset button | Explicitly reset heights to the other terrain preset when clearance permits |
| U / visibility button | Switch between Surface and Underground views |
| Delete / Remove button | Remove the visible selected public path, queue path, entrance or exit when unoccupied; reconcile affected guest/access state |
| R / Restore paths button | Restore missing public demonstration paths without replacing player pieces or resetting time/guests |
| O / Open or Close button | Open the ride for queue admission, or stop admission and return queued guests while the train finishes its current lap |
| F / Find button | Center the train circuit at 150% zoom |
| F5 / Save button | Save the current simulation to the single slot |
| F9 / Load button | Load the primary slot and pause |
| Shift+F9 / Backup button | Explicitly load the previous valid backup and pause |
| Space | Pause or resume the previous simulation speed |
| 1 / 2 / 3 | Set simulation speed to 1x / 2x / 4x |
| H / F1 | Open or close help |
| Escape / Inspect button | Close help first, otherwise return to Inspect; Escape also clears selection |
| Window close button / Alt+F4 | Exit |

Tile coordinates in the inspector start at 1. Terrain heights are simulation steps; the Terraced preset uses levels 0, 2, 4, and 6. The crowd demonstration retains 37 path tiles at height 8 and nine original track tiles at height 0. The new circuit's rails range from height 8 to 16. The 0.13.0 connection contract joins only compatible opposing cardinal edges at the same endpoint height; ramps have no side entrance. The interface reports guests/lost, seeking/queued/returning, cancellation events, queue capacity, oldest wait, passenger phases and completed trips. Guests and train cars are not individually selectable; their supporting track/path surfaces are.

Flat terrain exposes the same track that Terraced terrain buries. Switching presets preserves elapsed time, materials, object identities, and object heights. Underground view removes terrain from rendering and picking; its reference floor is decorative and cannot be selected. Hidden selections retain their identity but do not show through occluding surfaces.

P selects public-path construction; J selects a queue for Meadow Flyer. C cycles Flat and the four world directions of rise, retaining Queue mode when active. Slopes use low height 0–29 and conservative clearance 3; Flat uses height 0–30/clearance 2. Matching opposing endpoints connect; ramps have no side entry. I/Gate places a Flat entrance at station height facing a lateral station side, and C rotates its facing. Preview/commit share validation. X removes visible paths or gates one piece per deliberate click. R restores public fixture paths without replacing player pieces. The ride panel requires one unbranched, acyclic owned queue from the gate to exactly one public edge. Ready means connected topology. With the ride open, guests approach on public paths and join only at the public queue end. Capacity is one admitted guest per queue tile; approaching guests reserve no place. Guests walk in ticket order without overtaking, including on slopes. K/Exit adds a station-side exit with a public landing connected to the queue-entry component. This enables passenger journeys; costs/refunds remain absent.

B/Raise and V/Lower target the picked terrain column, including a visible cliff face. Each click changes height by one step within 0–32 while preserving Grass/Sand material. Water and Underground terrain editing are blocked. Green plus/blue minus indicators show the direction; red X indicates rejection. Paths, rails, and guests stay in place, and invalid clearance leaves the park unchanged. Successful edits display Custom; T is an explicit preset reset. No terrain brushes, water editing, terrain slopes, costs, or refunds are included.

The displayed timer is elapsed simulation time, not a park calendar. Trees and the entrance arch remain decorative. Every launch starts a fresh seeded demonstration; use Load to continue a saved park. Startup does not overwrite an existing save.

The empty train retains its automatic station/lap behavior. With a valid connected exit and ready open queue, guests can reserve six seats, board over 60 ticks, ride a real lap, and exit over 60 ticks onto the public landing. Passenger transfers and finished-lap occupants hold the train while they resolve. Closing in flight finishes the occupied lap; a missing exit holds returned passengers until repair. Occupied circuit track and active transfer paths/portals reject demolition atomically. Space freezes train, queue and transfers. The 60-second queue patience, tail-first cancellation return and ten-second retry cooldown remain for queue visitors. Completed trips count only real lap completion followed by public return; no income is collected. The train uses tuned integer motion rather than a realistic coaster physics model.

## Save and continue

Saves live in `%LOCALAPPDATA%\Parkbound\saves`, separate from the executable and shared OneDrive source folder. `park.park.json` is the primary slot; `park.park.json.bak` is the previous valid backup. Saving replaces the one slot, including player-built/removed paths and custom terrain heights. Backup loading is explicit, and a corrupt primary is never rotated into the backup. A successful load resets camera/selection/view and tool mode, pauses simulation, and resumes at 1x when requested; authoritative train and guest state is retained. Camera, tool height/ghost, UI settings, and pause/speed preference are not saved. A same-update Save includes the world click; Load/Backup takes precedence over pending edits even when loading fails.

Version 0.16.0 writes strict schema 5/content 5 and explicitly migrates versions 1/1 through 4/4. Exit metadata, passenger seats, transfers, journey flags and completed-trip counts persist. Migrated v4 parks acquire no automatic exit or passenger and retain no-exit queue behavior. Loading alone does not rewrite files; old 0.15 readers intentionally reject v5. Keep closed-game legacy copies for rollback. Normal play remains 32 × 32 with existing save caps, no autosave/save browser, and saves outside OneDrive. Use `--save-dir <absolute-path>` for isolated QA. New compatibility/runtime verification is pending; see [the ride journey report](docs/RIDE_JOURNEY_REPORT.md).

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

Retained terrain scenes are `terrain-raise|terrain-lower|terrain-raised|terrain-lowered|terrain-blocked|terrain-water|terrain-underground`. Terrain continuation uses `--save-check write-terrain` then a separate `--save-check verify-terrain` process with a new isolated absolute save directory, producing `verification-terrain.txt`. These terrain and cross-version checks passed historically in 0.12.0; [the terrain editing report](docs/TERRAIN_EDITING_REPORT.md) explains the intended proof.

Retained slope scenes are `slope-valid|slope-built|slope-blocked|slope-crowd|slope-removed|slope-recovered`. `--save-check write-slopes` and `verify-slopes` use a new absolute QA directory and produce `verification-slopes.txt`. Historical 0.13.0 migration/continuation evidence is in [the sloped paths report](docs/SLOPED_PATHS_REPORT.md). New queue scenes are `queue-entrance|queue-connected|queue-disconnected|queue-branch|queue-blocked|queue-rebuilt`; `--save-check write-queues` then a separate `verify-queues` process produces `verification-queues.txt`. The [queue report](docs/QUEUES_REPORT.md) records the fixture, commands, topology, final verification, and remaining native/release gaps. Old readers must reject newer save versions; reverse-version loading is not a compatibility promise.

Admission scenes are `admission-approach|admission-moving|admission-full|admission-leaving|admission-cancelled|admission-recovered`. They seek real simulation states through at most 12,000 extra ticks. Separate `--save-check write-admission` and `verify-admission` processes use a new absolute isolated directory and produce `verification-admission.txt`, checking an approaching backup, occupied-ramp primary, and 3,600-tick command continuation. These admission proofs and eleven inspected captures passed on the historical 0.15.0 executable; their 0.16.0 regression run is pending. The existing queue proof now validates ordinary versus admitted path roles instead of requiring all guests to remain public.

New journey scenes are `journey-boarding|journey-riding|journey-unloading|journey-completed|journey-exit-blocked|journey-recovered`. The `write-journey`/`verify-journey` fresh-process proof uses an actual boarding backup, occupied moving primary, and 7,200 ticks with close, missing-exit hold/repair, protected demolition and empty fault/repair. Execution evidence is pending. See the current report for isolated commands and fixture coordinates.

For repeatable measurements, build Release and run `scripts/measure-performance.ps1 -OutputDirectory <absolute-new-directory>` through PowerShell 7. It runs baseline/save-scale/stress CPU cases and two baseline frame profiles sequentially, records safe hardware metadata and sampled process memory, and preserves unsupported/incomplete outcomes. `-SkipFrames`, `-Cases`, and repetition/sample options allow targeted runs. Frame values describe main-loop pacing, not GPU or physically presented-frame timing. See [the performance report](docs/PERFORMANCE_REPORT.md) before interpreting results.

The Windows ZIP also includes the wrapper. With PowerShell 7 installed, run `.\measure-performance.ps1 -Executable .\Parkbound.exe -OutputDirectory <absolute-new-directory>` from the extracted folder. PowerShell is only needed for this optional measurement tool.

`park_sim` owns engine-independent terrain, stable identities, clearance, public/queue path geometry, ride entrance/access topology, seeded guests, admission/ordered queues/patience, passenger seats/transfers/completion, sampled circuit geometry, train operation, complete fixed ticks, and validated saves. `park_game` owns filesystem storage, projection, shared rendering/picking, guest/train meshes, visibility, controls, and UI. A separate content crate will be introduced when authored ride/scenario definitions justify it.

## Controlled rendering and GPU diagnostics

Launch `parkbound.exe --render-case stress --render-camera crowd` for the isolated large fixture. Cases are `baseline`, `save-scale`, and `stress`; cameras are `overview`, `crowd`, and `ride`. `--render-rotation 0..3` and `--render-underground` select the view. Camera/inspection/speed remain available; editing and persistence are disabled in previews. Normal launches and saves stay 32 × 32.

Add `--profile-gpu` to a `--profile-frames <absolute-new.json>` run to request actual instrumented GPU timestamps where supported. Missing/unsupported data remains explicit. GPU spans, render-span CPU, main-loop pacing, and unmeasured presentation/upload are distinct. With the wrapper, `-SkipCpu -RenderCase stress -RenderCamera crowd -GpuDiagnostics -ProfileSamples 120` runs only the two frame modes; provide a new absolute `-OutputDirectory`. The historical [large-map/GPU report](docs/LARGE_MAP_GPU_REPORT.md) records the completed 0.9.0 evidence and diagnostic methods. Version 0.10.0 targets reduced static geometry submission in focused large-map views. Reports retain all generated landscape geometry and separately expose a final `render_work.static_visibility` snapshot with selected/hidden chunks and geometry. These are conservative CPU eligibility counts, not actual GPU draw calls or visible pixels; counters include startup/warmup. All 193 release tests, strict source checks, eight exact image comparisons outside version text, bidirectional 0.9/0.10 continuation, six GPU profiles, CI, and package relocation passed. The focused stress camera selects 13 of 212 chunks: 149,556 of 2,251,764 generated static vertices. This is reduced CPU eligibility, not a measured FPS or upload gain; see [the historical report](docs/STATIC_VISIBILITY_REPORT.md).

## Project documents

- [Resume development](RESUME_DEVELOPMENT.md): complete handoff and setup on another Windows PC.
- [Development plan](DEVELOPMENT_PLAN.md): milestones and completion gates.
- [Gameplay checklist](docs/GAMEPLAY_CHECKLIST.md): feature inventory and acceptance criteria.
- [Product decisions](docs/PRODUCT_DECISIONS.md): confirmed choices, working defaults, and open decisions.
- [Ride journey report](docs/RIDE_JOURNEY_REPORT.md): current 0.16.0 exit/boarding/passenger/return contract, reproduction, and pending verification.
- [Admission report](docs/ADMISSION_REPORT.md): completed bounded 0.15.0 seeking/ordered queues/return/migration implementation, reproduction, verification, and remaining manual checks.
- [Queue report](docs/QUEUES_REPORT.md): completed bounded 0.14.0 ownership/entrance/topology slice, strict v3 migration, reproduction, evidence, and remaining gaps.
- [Sloped paths report](docs/SLOPED_PATHS_REPORT.md): historical completed 0.13.0 edge/height/save-migration slice and verification.
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

Historical source `4c8d42f00fe43a48ad5586240f153b833419d991` is pushed on `codex/queue-connectivity` in [draft PR #12](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/12), stacked on unmerged PR #11/`codex/sloped-paths` from checkpoint `effe6a8abe07b62be395c5efe72ffc1ce55d54a3`. [Exact-source Windows CI run 39588999746](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39588999746) passed all steps, including 285 tests, packaging, ordinary/path/terrain/slope/queue proofs, reporting smoke, and uploads. The executable is 46,488,576 bytes, SHA-256 `d5a2a741466ade1cab5d1649e9dfdc7f1fa7fdfa168375a4c232bfe62db75737`; later documentation heads require their own CI status. That bounded connectivity implementation is complete; its evidence does not verify the current admission changes.

Historical 0.15.0 runtime source `0c34f8d13139d3dc676505248dba0a9fa872cf65` is in [draft PR #13](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/13), with passing [exact-source Windows CI run 39614500323](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39614500323). Its 46,683,648-byte executable has SHA-256 `301a970a3b25ee20855a6d9181b4dc4862f94fffb639ed400d6a659a86c5ea30`. Its native attempt stopped on the first F9 input-detected response; no new native control passed. Current 0.16.0 work is on `codex/ride-journeys` from checkpoint `965e326`; final source/PR/tests/runtime/native/CI/package identity and evidence are pending. No local performance measurement batch is planned.
