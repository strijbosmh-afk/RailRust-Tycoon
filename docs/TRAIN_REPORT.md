# Train circuit proof report

Build: **Parkbound 0.5.0**, 20 October 2026. **PRF-03 is verified for the automatic train circuit and fixed-update behavior** through 99 tests, sixteen inspected engine captures, and a normal launch showing a completed lap. Native O/F/button operation and physical hardware/display-rate performance remain unverified. This does not accept every 0.5.0 control, the whole foundation, or a complete ride system. The accepted 0.3.0 overlap and 0.4.0 crowd proofs remain regression baselines.

## Implemented scope

One original closed circuit carries a three-car train through a station, chain lift, descent, brake section, and return to the station. The train follows the circuit's sampled centerline. Its operation, movement, phase, lap count, and station wait belong to the Rust simulation.

- **O / Open or Close** controls the demonstration train. Closing at the station parks it; closing during a lap lets that lap finish before parking.
- **F / Find** centers the circuit at 150% zoom.
- The ride panel reports operation/phase, speed in tiles per second, lap count, and three cars. Its buttons and panel bounds exclude map input.
- **Space** pauses the whole park. Closing a running train while globally paused changes the requested operating state but cannot move it to the station until the simulation resumes.
- Reopening a parked train starts a full 90-tick station wait before dispatch.
- The twelve seeded guests, path-removal/restoration controls, terrain presets, and visibility modes remain available.

This is a deterministic movement demonstration. It does not add a track editor, guest boarding, queues, ride revenue, ratings, general ride testing/opening rules, multiple trains, block sections, realistic coaster physics, saves, or audio. Speed is a simulation measurement in tiles per second, not a claim about metres per second or real ride safety.

## Simulation and geometry contract

`Park::install_train_fixture()` installs the 46-piece circuit atomically and is a no-op while healthy. `Park::circuit()`, `train()`, and `train_cars()` expose read-only circuit, operation, and presentation state; `set_train_open(bool)` changes the requested operation. Existing crowd/time/RNG and surviving object identities are preserved. Reinstalling after missing circuit track repairs objects with fresh IDs where required, preserves completed laps, and parks the train **closed**. It does not restart automatically.

Track pieces use stable `ObjectId` values and `TrackKind::{Station, Lift, Drop, Brake, Curve, Straight}`. Their shared sampled integer centerlines and lengths drive both movement and rendering. Every adjacent endpoint, including the final return to the first piece, must connect before operation is valid. The fixture occupies the rectangle x7..24/y4..10, with its station at x8/y10 and rail heights 8..16; these coordinates are zero-based.

`install_train_circuit(&[TrackPieceSpec])` provides headless installation/validation with explicit tile-edge ports and heights. Exactly one station is required. Curves are level quarter turns; lifts rise one step per tile; drops descend one or two; other pieces are level straights. This API is not a player construction tool.

`TRACK_UNITS` is 1024 per tile and per height step. `TrackPoint` stores integer x, y, and height coordinates. The distance metric treats a height step as one quarter of a tile, matching the demonstration's chosen proportions. Train speed is integer distance units per tick. The lift uses a bounded speed of 24; descent can reach 96; brakes reduce it toward 20. These are tuned demonstration rules.

Three compact cars are spaced 320 distance units apart, with half-body length 120. The station stopping point is 768 units into its piece, so all cars fit when parked. Initial operation is open with a 90-tick dwell. Global 1×/2×/4× speeds run complete authoritative steps at the existing 30 Hz application fixed update; crowd and train must remain equivalent for equal tick/command histories.

Each circuit piece reserves its rail envelope from its minimum rail height to its maximum rail height plus two steps through the existing placement and terrain validation. Terrain presets must respect it. Camera fit includes the actual sampled rail maximum. Decorative supports are not reserved physical collision columns in this proof.

Removing a circuit track object faults/stops the train and clears its circuit geometry/reference set safely. There is no player-facing track demolition tool yet; the existing Remove path action still targets paths only. Invalid/deleted identities must not produce stale train indexing or selectable circuit geometry.

## Presentation and controls

Static circuit rails use the authoritative samples. Rendering, picking, and selection share actual sampled faces, never clearance-envelope roofs. The three original red/gold cars use authoritative center/front/rear poses in a separate nonselectable mesh; train motion alone must not rebuild static terrain or alter crowd state. Train position must preserve height and depth through all four rotations and both visibility modes. Foreground terrain and decks should occlude it according to shared depth. Decorative support posts are omitted where they intersect another object's occupied vertical range.

Manual acceptance demonstration:

1. Use Find and inspect a station, lift, descent, brake, and curve. Confirm the inspector identifies the track piece and its rail-height range.
2. Observe station wait, dispatch, lift, descent, braking, and return. A completed circuit increments laps once and starts another station wait.
3. Close during a lap. The train must continue to the station, park, and remain there while guests/time continue.
4. Open again; verify a complete station wait and a new dispatch. Close while at the station as a separate case.
5. Pause during travel, issue Close, and confirm the train stays still until resume. Repeat pause and 1×/2×/4× checks while guests continue to use complete ticks.
6. Exercise O/F and their buttons, help close without click-through, rotations, terrain presets, visibility modes, and the minimum window. Preserve path-removal/recovery behavior.

The older physical Delete shortcut, keyboard pan/drag, physical display, and clean-machine gaps remain separate unless newly demonstrated in this version.

## Diagnostic captures

Existing capture options remain available. The additional `--capture-scene` values are:

```text
train | train-lift | train-drop | train-brake | train-return | train-closed
```

The preset first advances `--capture-ticks` complete ticks (0..36000, default 600). `train` captures that state directly. Other train presets seek behavior inside their named section using complete simulation ticks, with a bound of 36000 additional ticks:

| Preset | Required state |
| --- | --- |
| `train-lift` | Lift phase at speed 24 distance units/tick, with lead-car height at least 12 and below 16 steps. |
| `train-drop` | Drop phase at speed at least 60 distance units/tick, with lead-car height above 8 and at most 14 steps. |
| `train-brake` | Brake phase at speed 20 distance units/tick, with the lead car at least one third of a tile past the current brake piece's entry. |
| `train-return` | Station phase, zero speed, and at least one completed lap. |
| `train-closed` | Close requested, then Closed phase at zero speed. |

These conditions avoid capturing the first boundary tick before the section's lift, descent, or braking behavior takes effect. A fault or missing target state fails diagnostics rather than waiting indefinitely. The resulting scene is paused, focused on the circuit, and selects the current track piece.

Use explicit rotation/map/view/tick options when comparing results. These controlled presets are engine evidence, not proof that normal keyboard or mouse input works.

## Verification record

The complete release test suite passed **99 tests**: 43 simulation, 27 world/rendering, 23 input-system, four main-module, and two capture-preset tests. Formatting, strict release Clippy with warnings denied, and release compilation also passed. The suite includes the retained overlap/crowd regressions and the train's circuit validation, motion, operation, sample-based picking/depth, and UI/capture behavior.

A Bevy fixed-update regression ran 120 simulated seconds with 30, 60, and 144 Hz display-update schedules and obtained equal whole-Park state. This verifies schedule-independent simulation, including both guests and train; it is a synthetic scheduling test, not a measured hardware frame-rate result. Rendering regressions also cover degenerate edge-on faces and picking against the triangles actually emitted to the mesh.

The checked release executable SHA-256 is:

```text
0cd409dc5e9574507bc7bcc93111dfb3c0d5415bdddde35cb16c62b54c491d64
```

Passing source checks:

```powershell
cargo fmt --all -- --check
cargo clippy --release --locked --workspace --all-targets -j 2 -- -D warnings
cargo test --release --locked --workspace -j 2
cargo build --release --locked -p park_game --bin parkbound -j 2
```

Passed regressions cover cyclic endpoint validation, invalid geometry, atomic installation failure, sample/arc-length consistency, lap counting, speed equivalence, station dwell, close-at-station and close-mid-lap, paused controls, reopening, missing-track fault behavior, crowd preservation, rendering/picking/depth, and panel click exclusion.

### Engine captures

All **16 PNG captures** under `artifacts/train-0.5.0/` completed with exit code 0 and were visually inspected. Track and all three cars were coherent across four rotations, including shared depth and legible controls. The minimum-window train/help layouts fit at 1024 × 680.

| Capture | Observed result |
| --- | --- |
| `station.png` | Initial station scene at zero ticks. |
| `lift-ne.png`, `lift-se.png`, `lift-sw.png`, `lift-nw.png` | Lift behavior in all four rotations; elapsed 1 min 1 s, lap 1, speed 0.70 tiles/s. |
| `drop.png` | Descent at elapsed 30 s, lap 0, speed 2.61 tiles/s. |
| `brake.png` | Brake section at elapsed 46 s, lap 0, speed 0.59 tiles/s. |
| `return.png` | Station wait after return at elapsed 51 s, lap 1, zero speed. |
| `closed.png` | Parked after finish-lap closure at elapsed 51 s, lap 1, zero speed. |
| `small-train.png`, `small-help.png` | Readable ride panel and help at the minimum 1024 × 680 window. |
| `flat-drop.png` | Descent scene with Flat terrain. |
| `underground-lift.png` | Lift scene in Underground view. |
| `crowd.png`, `path.png`, `underground-track.png` | Retained crowd, selected path, and original underground-track rendering. |

These generated captures are local evidence, not shipped game assets or normal-input verification.

### Native controls

The normal application ran at 1280 × 800. At elapsed **01:04**, the ride panel showed lap 1 and the chain-lift phase. Space successfully paused at **01:15**, lap 1, Running at 0.94 tiles/s. The crowd still contained twelve guests: eight walking and four waiting, with 116 destinations reached.

Further native testing stopped when human desktop activity obscured the game. An F attempt had no unambiguous observed result, so **F/Find is not verified through native input**. O/Open/Close and their buttons were not demonstrated in this version. Headless tests and engine presets cover their internal behavior but do not establish normal input. Retest these controls when the game can be observed throughout the action.

Physical display-rate/hardware performance, the earlier Delete shortcut, keyboard pan/drag, physical DPI/multiple-monitor coverage, and clean-machine compatibility remain open.

### Package and hosted CI

**Pending:** final package checksums/payload verification/relocation and hosted CI for the delivered commit. These delivery checks are separate from PRF-03's demonstrated automatic-circuit behavior. The 0.4.0 captures, package hashes, and successful workflow remain historical evidence.

Package with `pwsh -NoProfile -File scripts/package-windows.ps1 -SkipBuild` after building the exact checked binary. Verify the ZIP checksum and all payload checksums, then launch the extracted executable from an unrelated working directory after relocation into a folder containing spaces/non-ASCII characters. Packaged Markdown must match the final source documents.

## Delivery and continuation

Expected local output after successful packaging is `dist/Parkbound-0.5.0-windows-x64.zip`, its `.sha256`, and the extracted `Parkbound-0.5.0-windows-x64` directory. No current package or hosted-CI success is claimed until recorded above. This remains an unsigned local checkpoint; a source push or CI artifact is not a public binary release.

The current checkout is the original OneDrive folder in [the resume guide](../RESUME_DEVELOPMENT.md), which retains both recorded PC paths. Branch `codex/train-circuit` starts from crowd checkpoint `51a79e8`. While [crowd PR #2](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/2) remains unmerged, the train review is stacked on `codex/path-crowds`; the overlap review remains its earlier dependency. Inspect actual merge state before changing bases.

PRF-03 is **V** for its stated technical criterion: completing the station/lift/drop/brake circuit with simulation independent of display-update scheduling. Normal launch showed lap 1 and Space paused successfully; O/F/buttons still need native retesting. The next implementation is **PRF-04 save/reload continuation**: version all authoritative terrain/object/guest/circuit/train state, identities, clocks, timers, and RNG; validate before mutation; and compare uninterrupted execution with saved/reloaded continuation. Carry the explicit manual-input and delivery gaps forward. Performance measurement (PRF-05) and the first complete park remain later milestones.
