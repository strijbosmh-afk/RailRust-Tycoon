# Single-tile terrain editing — 0.12.0

Updated: 21 October 2026. Current branch: `codex/terrain-editing`, based on completed 0.11.0 checkpoint `13c4063`. Runtime source `e5dbc0e61a9fdc3870f89a9c664ed13fbf4c4fc1` is pushed in [draft PR #10](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/10), stacked on still-unmerged PR #9/`codex/path-construction` at `13c4063`. Local tests, strict release Clippy, production build, ten inspected captures, and terrain/cross-version continuation passed. Native controls remain unverified after activation/capture errors prevented input. [Windows CI run 39557465779](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39557465779) passed for exact runtime source `e5dbc0e61a9fdc3870f89a9c664ed13fbf4c4fc1`, including formatting, strict lint, all 236 tests, packaging, ordinary/path/terrain continuation, CPU-reporting smoke, and artifact uploads. Package verification matched all 27 payload hashes and eighteen guides. A fresh extraction path containing spaces and é, launched from `C:\Windows`, passed terrain primary/backup/Custom/paused continuation for 3,600 ticks plus commands/clearance. Its terrain-raised/rotation-2/Surface/Terraced/1024 × 680 PNG matched the source image byte-for-byte. The [path construction report](PATH_CONSTRUCTION_REPORT.md) preserves the completed 0.11.0 baseline; its results do not verify changed terrain code.

The bounded single-tile terrain implementation is verified through source, automated input, runtime rendering/continuation, CI, and package checks. **BLD-06 remains in progress** for the full editor; native terrain input remains unverified. It adds one-tile raising/lowering to the normal 32 × 32 park. It does not complete terrain brushes, terrain slopes, material/water tools, costs/refunds, or the full editor. No reference PC is selected: this development PC is explicitly excluded. No local performance measurement batch was run; CI CPU smoke checks concern reporting rather than hardware acceptance.

## Player contract

| Input or state | Behavior |
| --- | --- |
| B / Raise in BUILD TOOLS | Select single-tile terrain raising. |
| V / Lower in BUILD TOOLS | Select single-tile terrain lowering. |
| Deliberate left click | Change the picked visible Grass or Sand column by exactly one height step. A visible top or cliff can identify the same column. No held-button or drag painting. |
| Height limits | Terrain remains within 0–32. Raising 32 or lowering 0 is rejected, not reported as a successful saturated edit. |
| Material and visibility | Preserve Grass/Sand material. Player Water edits and all Underground terrain edits are blocked. The decorative reference floor is never editable. |
| Preview | Current-top footprint and exposed-cliff markers remain visible when their actual faces are visible. Green plus means raise, blue minus means lower, and red X means rejection. A proposed-height outline uses the target height/depth; foreground geometry may occlude it. |
| Path-height controls | Page Up/Down and height buttons still select the independent path plane, 0–30/default 8. They do not set terrain height. |
| Inspect / Escape | Return to inspection; Escape closes help first and clears selection. Existing P/X path tools remain available. |
| Terrain preset T | Explicitly apply a preset when shared clearance permits. A successful manual terrain edit displays Custom; failed edits/preset changes preserve the current terrain. |

Paths, track, and guests are not moved by the terrain command. A raised surface cannot intersect a path or a rail envelope. A lowering operation cannot partially expose a previously buried track through its reserved interval. The preview and command must use the same candidate height and validation result. While simulation is running, normal ticks can still advance; command atomicity is checked against a paused snapshot or isolated simulation state.

## Simulation, input, and rendering contract

`Park::validate_terrain_height(&self, TilePos, u8) -> Result<(), TerrainHeightError>` performs immutable bounds/shared-clearance validation. `Park::set_terrain_height(&mut self, TilePos, u8)` delegates before assigning one tile height. The low-level simulation API continues to support all materials; Water rejection belongs to the player tool. This separation preserves existing fixture/preset behavior.

A rejected command leaves tiles, materials, objects, IDs/allocators, guests, train, clock, and RNG unchanged. A valid edit changes the chosen terrain height without relocating or reallocating objects. The path graph still connects same-height path objects; terrain-only edits do not create path connections. Whole-Park continuation and cache regressions must cover these invariants.

The player tool uses the normally picked terrain identity, including cliff hits, rather than intersecting the path construction plane. `terrain_tools.rs` dispatches `RaiseTerrain`/`LowerTerrain` through `ToolState.terrain_preview`. It reuses the existing translucent preview mesh slot with no additional entity. Actual depth/occlusion and all four rotations remain authoritative. A terrain edit refreshes the static scene, selection, and preview; camera-only changes retain the existing static-visibility behavior. This is not incremental terrain-chunk rebuilding or a claimed rendering-speed improvement.

UI, help, drag/pan, focus, capture, profile, and diagnostic-preview gates continue to suppress unintended edits. H/F1 help toggles consume the click. Delete wins Restore and consumes a competing world click. Pending Load/Backup gates terrain/path/preset/train edits even when loading fails. Save executes after a same-update world edit, so it includes the new height. A successful load resets tool mode to Inspect and path height to 8, resets presentation, and starts paused at 1x resume speed.

Schema version 1/content version 1 already stores terrain heights and materials. No schema, content version, save limits, or dependency changed. Normal app load remains 32 × 32. Tool mode, preview, hints, camera, and pause/speed preference remain transient. Startup still creates the fresh demonstration without overwriting saves.

## Verification

All 236 local release tests passed (162 game/74 simulation), and final strict all-target release Clippy passed. The final source also passed all 236 tests in Windows CI; final local rustfmt passed. The production release build passed. Bidirectional 0.11/0.12 continuation and write-terrain/verify-terrain passed, including exact primary/backup state, Custom/paused loading, 3,600 ticks, commands, and clearance rejection. All ten captures exited 0 and were inspected without a blocking visual defect. [Windows CI run 39557465779](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39557465779) passed for exact runtime source `e5dbc0e61a9fdc3870f89a9c664ed13fbf4c4fc1`, including formatting, strict lint, all 236 tests, packaging, ordinary/path/terrain continuation, CPU-reporting smoke, and artifact uploads. Package verification matched all 27 payload hashes and eighteen guides. A fresh extraction path containing spaces and é, launched from `C:\Windows`, passed terrain primary/backup/Custom/paused continuation for 3,600 ticks plus commands/clearance. Its terrain-raised/rotation-2/Surface/Terraced/1024 × 680 PNG matched the source image byte-for-byte. Native terrain controls remain unverified after capture/activation errors prevented input. Tests exercise supplied engine input/resources and geometry; they do not establish native Windows control behavior.

| Area | Required evidence | Current status |
| --- | --- | --- |
| Source | Published implementation commit, review/base, and exact CI identity | Source `e5dbc0e61a9fdc3870f89a9c664ed13fbf4c4fc1` is in draft PR #10, stacked on PR #9 at `13c4063`. [Windows CI run 39557465779](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39557465779) passed for exact runtime source `e5dbc0e61a9fdc3870f89a9c664ed13fbf4c4fc1`, including formatting, strict lint, all 236 tests, packaging, ordinary/path/terrain continuation, CPU-reporting smoke, and artifact uploads. |
| Simulation | Immutable validation/setter parity; bounds/material preservation; path/track clearance; rejected-state equality; no object/guest relocation | Local release suite passed: 74 simulation tests. |
| Player input | B/V/buttons; one click only; top/cliff selection; Water/Underground rejection; UI/help/drag/focus/diagnostic gates; same-update Save and Load/Backup priority | Headless regressions passed in the 162-game-test suite. Native controls remain unverified; no native input was sent. |
| Rendering | Valid/invalid directional previews, edited terrain, all rotations, surface occlusion, minimum window/help, unchanged retained path tools | Ten captures exited 0 and were visually inspected: six 1280 × 800, four 1024 × 680. No blocking defect; raise-r0 plus is mostly occluded, so no unobstructed-plus legibility claim. |
| Persistence | Fresh-process terrain primary/backup equality, Custom/paused loading, 3,600 continued ticks plus terrain/path commands, compatible historical saves | Production write-terrain/verify-terrain and bidirectional 0.11/0.12 continuation passed with exact state, backup, Custom/paused loading, 3,600 ticks/commands, and clearance rejection. |
| Source gates | Release workspace tests, rustfmt, strict all-target release Clippy, release build | All 236 tests passed locally and in final-source CI (162 game/74 simulation). Final local rustfmt, strict all-target release Clippy, and production build passed. |
| Native controls | Normal Windows B/V/buttons, clicks, preset behavior, save/reload, and rejection feedback | Native B/V/buttons/clicks remain unverified: activation failed at `GetCursorPos` with access denied `0x80070005`; one refreshed-window retry failed at `IGraphicsCaptureItemInterop.CreateForMonitor` with `0x80070057`. No input was sent, and only the owned isolated process was stopped. |
| Delivery | Final executable size/hash, notices/runtime, all payload/guide hashes, extracted execution from unrelated working directory, and final archive sidecar | Executable 46,266,368 bytes, SHA-256 `344e627608bc5bab1f8bd87943dd301be20552e62861e11d7bc4af5d207480a7`. Package verification matched all 27 payload hashes and eighteen guides. A fresh extraction path containing spaces and é, launched from `C:\Windows`, passed terrain primary/backup/Custom/paused continuation for 3,600 ticks plus commands/clearance. Its terrain-raised/rotation-2/Surface/Terraced/1024 × 680 PNG matched the source image byte-for-byte. Includes 291 dependency notices, font/Rust notices, and one MSVC DLL; notices/metadata/runtime match 0.11.0 exactly. |

### Terrain captures

Use the production executable's existing `--capture-frame`, rotation/map options, and `--capture-small-window`. `capture/terrain_scenes.rs` prepares production commands with ordinary input suppressed; these are not native keyboard/mouse checks. All terrain scenes except `terrain-underground` require Surface, and an explicit Underground option is rejected. The Underground scene forces cutaway mode and rejection. Empty dry targets start at height 4; the blocked fixture raises ground to a real path base before rejecting +1. Water remains unchanged. All seven terrain scenes passed and were inspected:

| Scene | Intended proof |
| --- | --- |
| `terrain-raise` | Valid raise preview with no terrain mutation. |
| `terrain-lower` | Valid lower preview with no terrain mutation. |
| `terrain-raised` | One successful raise, resulting terrain, and Custom state. |
| `terrain-lowered` | One successful lower and resulting cliff/top geometry. |
| `terrain-blocked` | Shared-clearance rejection and unchanged terrain. |
| `terrain-water` | Player Water rejection. |
| `terrain-underground` | Underground edit suppression and no editable reference-floor ghost. |

All ten final images in `artifacts/terrain-editing-0.12.0/captures/` exited 0 and were inspected:

| Scene | Rotation | View / map | Size |
| --- | --- | --- | --- |
| terrain-raise | 0 | Surface / Terraced | 1280 × 800 |
| terrain-lower | 1 | Surface / Flat | 1280 × 800 |
| terrain-raised | 2 | Surface / Terraced | 1024 × 680 |
| terrain-lowered | 3 | Surface / Flat | 1280 × 800 |
| terrain-blocked | 0 | Surface / Terraced | 1280 × 800 |
| terrain-water | 1 | Surface / Terraced | 1280 × 800 |
| terrain-underground | 3 | Underground / Terraced | 1024 × 680 |
| overview | 0 | Surface / Terraced | 1280 × 800 |
| help | 0 | Surface / Terraced | 1024 × 680 |
| path-built | 2 | Surface / Terraced | 1024 × 680 |

The panel, help, and footer fit. Rejected Water/clearance reasons and red X markers are readable; lowering retains its current-cliff marker; Underground shows no terrain ghost. The raise-r0 plus is mostly occluded by surrounding terrain/path while the green proposed outline remains visible. That image verifies occlusion, not unobstructed plus-marker legibility. Static inspection does not establish native interactions or motion.

Example for repeating the verified production capture:

```powershell
$terrainCapture = Join-Path $env:TEMP ('Parkbound terrain-' + [Guid]::NewGuid().ToString('N') + '.png')
$terrainExe = (Resolve-Path -LiteralPath 'target\release\parkbound.exe').Path
$terrainRun = Start-Process -FilePath $terrainExe -ArgumentList @('--capture-frame', ('"' + $terrainCapture + '"'), '--capture-scene', 'terrain-raised', '--capture-small-window') -WindowStyle Hidden -PassThru -Wait
if ($terrainRun.ExitCode -ne 0) { throw 'Terrain capture failed' }
```

A valid PNG/zero exit is necessary but does not prove correct preview, depth, or layout. Inspect images and record actual dimensions/state. Preserve historical captures; keep new evidence under `artifacts/terrain-editing-0.12.0/`.

### Fresh-process terrain continuation

`--save-check write-terrain` requires a new absolute save directory whose parent exists. A separate `--save-check verify-terrain` process reads it and writes `verification-terrain.txt`, leaving the primary/backup unchanged. This diagnostic uses production save actions and terrain validation without a renderer.

The implemented proof fixture edits Grass at zero-based `(3,24)`, Sand at `(24,3)`, and the overlap column `(13,16)`. It stores custom terrain in primary/backup, compares complete loaded Park state and paused/Custom presentation, and continues 3,600 ticks with branch-path commands. The overlap sequence lowers height 8 to 2 and rejects 1 against buried track, then raises 2 to 8 and rejects 9 against the path. Every terrain command changes at most one height step. These cases passed in separate production processes. The runtime verification script also passed both old-0.11-write/new-0.12-verify and reverse continuation.

```powershell
$terrainProof = Join-Path $env:TEMP ('Parkbound terrain proof é ' + [Guid]::NewGuid().ToString('N'))
$terrainExe = (Resolve-Path -LiteralPath 'target\release\parkbound.exe').Path
foreach ($terrainMode in @('write-terrain', 'verify-terrain')) {
    $terrainRun = Start-Process -FilePath $terrainExe -ArgumentList @('--save-check', $terrainMode, '--save-dir', ('"' + $terrainProof + '"')) -WorkingDirectory $env:SystemRoot -WindowStyle Hidden -PassThru -Wait
    if ($terrainRun.ExitCode -ne 0) { throw "Terrain proof failed: $terrainMode" }
}
Get-Content -LiteralPath (Join-Path $terrainProof 'verification-terrain.txt')
```

Use isolated QA directories, never the player's default save slot. Retain the existing `write`/`verify` and `write-paths`/`verify-paths` proofs and cross-version continuation where compatible. Package relocation must rerun the terrain proof against the extracted binary.

## Local files and delivery

Current shared checkout:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
```

The second-PC checkout remains `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`, a separate clone with no verified OneDrive sync. Use only one active edit/build/Git session in the shared folder. Source and documentation belong on GitHub; ignored binaries/captures/Cargo caches do not. LocalAppData saves need separate transfer.

Verified local 0.12.0 archive location:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon\dist\Parkbound-0.12.0-windows-x64.zip
```

The adjacent `.zip.sha256` is authoritative for the final archive. Do not place a ZIP's self-referential hash in its bundled guides. The initial final-code archive verified 27 payload hashes and eighteen guides; refresh after final Markdown updates and repeat every payload/current-guide check. Exact runtime source and CI identity above remain fixed across documentation-only updates, whose later heads have separate CI status.

Local receipts are `runtime-verification.txt`, `native-verification.txt`, `package-verification.txt`, `notice-verification.txt`, and `relocated-runtime-verification.txt` under `artifacts/terrain-editing-0.12.0/`. The retained native receipt records the pre-input activation/capture failures. `THIRD_PARTY_NOTICES.txt`, `RUST_STANDARD_LIBRARY_NOTICES.html`, `VCRUNTIME140.dll`, `DEPENDENCIES.json`, and `NATIVE_DEPENDENCIES.json` match 0.11.0 byte-for-byte.

## Remaining scope and next handoff

Native B/V/buttons/clicks remain unverified: activation failed at `GetCursorPos` with access denied `0x80070005`; one refreshed-window retry failed at `IGraphicsCaptureItemInterop.CreateForMonitor` with `0x80070057`. No input was sent, and only the owned isolated process was stopped. Existing gaps remain: physical Page Up/Down and Delete, O/F/ride buttons, Backup and separate Save/Load buttons, panning/dragging and physical display/DPI, clean-PC offline/standard-user execution, and final hardware targets. Native 0.11.0 P/X, height buttons, F5/F9, and rejection/Inspect reset are retained historical results.

The recommended next bounded design is a one-step sloped path connecting different heights under BLD-08. First specify endpoint heights, occupied clearance, compatible path edges, guest movement, preview/picking, and save-version handling; existing path objects have a single height. Do not silently treat slopes as geometry only or bundle queues, ride boarding, or the full terrain editor. BLD-06 and PRF-05 remain open for their broader criteria.
