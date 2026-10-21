# One-step sloped paths — 0.13.0

Updated: 21 October 2026. The bounded slope implementation is complete on `codex/sloped-paths`, from completed 0.12.0 checkpoint `ef64609`. Runtime source `b1d26a04cc1282bb71defcb59557309b79252f1d` is pushed in [draft PR #11](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/11), stacked on still-open PR #10/`codex/terrain-editing`. All 260 release tests (175 game/85 simulation), formatting, strict all-target release Clippy, and the production release build passed. [Windows CI run 39583267705](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39583267705) passed for that exact runtime source, including ordinary/path/terrain/slope continuation, packaging, reporting smoke, and uploads. Later documentation heads have separate CI status. The [terrain editing report](TERRAIN_EDITING_REPORT.md) preserves the historical 0.12.0 results.

This completes the bounded one-step slope slice under **BLD-08**, which remains in progress for its broader bridges/tunnels/construction criteria. Normal play/load remains 32 × 32, with existing object/path/guest limits. Queues, boarding, costs/refunds, ownership, multi-tile construction, and a full terrain editor are outside this slice. No reference PC is selected; the user excluded this development PC. No local performance measurement batch was run; CI CPU smoke verifies reporting only.

## Agreed player and geometry contract

| Choice | Meaning |
| --- | --- |
| Flat | Existing level path at height 0–30, occupying two height steps. |
| North / East / South / West | One-step uphill direction in world coordinates: North is −Y, East is +X; camera rotation does not change the world direction. |
| Slope low height | 0–29; high endpoint is low + 1. The conservative occupied column spans three height steps. |
| C / shape button | Cycle Flat → North → East → South → West → Flat and enter Path build mode. Selecting a slope clamps a previous height 30 to 29. |
| Page Up/Down / height buttons | Change the low-end height for a slope; retain the existing flat-height meaning for Flat. |
| Placement/demolition | One deliberate click per piece, shared validation/preview, fresh IDs on placement, visible-only demolition, and unchanged UI/help/drag/focus/save conflict gates. No hold-to-paint. |

These controls passed headless input regressions. Native C/shape selection, height buttons, South-ramp construction/demolition, F5/F9, and Save/Backup buttons were also verified as detailed below. Physical Page Up/Down and modifier gestures remain retests. Existing P/X/Inspect, B/V terrain editing, camera/rotation, and save controls remain. The build panel/help distinguish world direction of rise and low/high endpoints.

`PathSlope` variants are `Flat`, `North`, `East`, `South`, and `West`. A ramp joins only through its two opposite ends; it has no side entrance. Two cardinal neighbors connect only when their facing edges both exist and have identical endpoint heights. Flat paths retain their compatible four-edge behavior. Diagonal proximity, base-height coincidence, or visually touching side faces do not create a connection. Track is never a guest path.

`Park::validate_sloped_path(pos, height, slope)` and `place_sloped_path(pos, height, slope)` share the player command contract. Existing `validate_path`/`place_path` remain Flat wrappers. Conservative clearance reserves the whole tile column, including the one-step rise. Placement, terrain edits/presets, and save validation must agree on that reservation. Rejected geometry, collision, capacity, or allocator checks leave all authoritative state and cache counters unchanged. Stable object IDs are not reused, and successful topology edits invalidate the derived path cache exactly as required.

Preview, emitted deck geometry, picking, selection, connected edges, and guest positions must use the same surface definition. All four world directions must remain correct in all four camera rotations, Surface and Underground views, at different heights and with foreground occlusion. The current generated art/geometry is original; no external assets are planned.

## Guest motion, demolition, and recovery

Shared simulation helpers are `PathDirection`, `paths_connect`, `path_edge_height`, `path_height_at`, and `path_center`, using `PATH_UNITS = 60`. `GuestPosition.point` samples the current path center, matching edge, and next path center piecewise. A ramp's height varies across its footprint, so simply interpolating the two object base heights is incorrect. Shared-edge position and height must be continuous in both travel directions, including Flat↔slope and slope↔slope joins. Exact old flat positions and the existing complete 30 Hz ticks, progress cadence, deterministic route order, seed/RNG, and wait behavior must remain.

`Guest.anchor_slope` retains a removed path's footprint orientation and base for lost-state positioning and incident-edge recovery. Arrival updates the guest's lower height and slope. Recovery stays local and uses edge/surface compatibility, not an arbitrary nearest path. Release motion/removal regressions and production occupied-bridge removal/fresh-ID recovery passed; separate production processes verified complete authoritative continuation through these commands.

## Save version and migration contract

New saves use strict **schema version 2/content version 2**. Loading strict **1/1** explicitly migrates legacy paths to `Flat`, preserving authoritative terrain, objects/IDs/order/allocators, clock, guests/routes/progress/waits/RNG, and train state. A legacy walking guest mid-edge must continue the same flat route; Lost guests without a current path and faulted trains remain legitimate states. Migration must not allocate new path IDs or reinstall fixtures.

Mismatched version pairs, unknown/future versions, v1 documents containing new v2 fields, malformed slope representations, wrong-edge routes, invalid height/clearance combinations, and invalid references are rejected before replacing the live park. Existing bounded input/collection limits, explicit backup loading, lock/recovery behavior, and normal 32 × 32 app guard remain. A migrated file is not rewritten merely because it was loaded; explicit Save writes v2 through the production store.

**Compatibility is directional:** production 0.12 writes → 0.13 migration/continuation passed. Production 0.13 writes → 0.12 rejection returned the expected exit 1; subsequent 0.13 verification passed. Do not treat this intentional old-reader rejection as a regression. Keep a closed-game v1 primary/backup copy if returning to the older executable is necessary. The ordinary one-slot backup is not an unlimited version archive.

The verification target is equivalent authoritative state and deterministic continued behavior after migration, not byte identity between v1 and v2 JSON. V2 same-build roundtrips must preserve every new slope/anchor field and complete continuation state. Save includes same-update edits; pending Load/Backup blocks competing edits even on failure. Successful load resets transient tools/camera/selection and pauses while retaining authoritative state.

## Acceptance and evidence

| Area | Required proof | Status |
| --- | --- | --- |
| Source and API | Final slope/edge/guest APIs, implementation commit, review/base | Runtime b1d26a0 is pushed in draft PR #11, stacked on PR #10/codex/terrain-editing. |
| Placement | Four directions; low/high bounds; conservative clearance; terrain/preset interaction; exact rejected-state/cache identity; existing caps | Passing 85-simulation-test suite and observed native South-ramp construction. |
| Connectivity | Opposing matching endpoints; flat↔slope and slope↔slope; wrong height/side/diagonal rejection; stable route ordering | Release simulation regressions passed. |
| Guest movement | Continuous fixed-point positions/heights; both travel directions; deterministic full ticks/speeds; local removal/recovery | Release regressions and production mid-ramp/3,600-tick continuation passed. |
| UI and rendering | C/shape/height controls; all-direction/all-rotation preview/pick/selection/depth; minimum window/help; input conflict gates | Release geometry, six-scene command matrix, and six slope-input regressions passed. Eleven final captures exited 0 and were inspected. |
| Saves | Strict v1 migration; strict v2 roundtrip; malformed/future/version-pair rejection; primary/backup; new-process continuation through slope edits | Codec/store tests, production forward migration/old-reader rejection, and separate ordinary/terrain/slope proofs passed. |
| Native controls | Normal Windows slope selection/build/remove and save/reload | C/shape, height buttons, South build, X demolition, F5/F9, Save and Backup buttons passed; remaining gestures are listed below. |
| Source gates | Release tests, rustfmt, strict Clippy, production build | All 260 release tests (175 game/85 simulation), formatting, strict release Clippy, and build passed. |
| Delivery | Executable identity, CI, notices/runtime, payload/guide hashes, relocated continuation/rendering | Exact-source CI passed; 28 payload hashes/nineteen guides matched; relocated slope continuation and byte-identical minimum-window built-slope capture passed. Reverify guides after documentation refresh. |

The executable is **46,345,728 bytes**, SHA-256 `249e2225998949197f42a8f08e1a447c62d36c9fa9d7b7baab2ff8c1f1dfa5f9`. Local receipts are under `artifacts/sloped-paths-0.13.0/`: `runtime-verification.txt`, `runtime.log`, `visual-verification.txt`, `native-verification.txt`, `package-verification.txt`, `relocated-runtime-verification.txt`, and `notice-verification.txt`.

### Production capture scenes

`capture/slope_scenes.rs` implements six scenes covered by passing command-matrix tests and the final production capture matrix:

| Scene | Prepared state |
| --- | --- |
| `slope-valid` | Valid South-ramp preview; the prepared Park remains unchanged by preview. |
| `slope-built` | Production click builds the South bridge to the higher landings. |
| `slope-blocked` | A real track envelope rejects the attempted bridge; prepared Park remains unchanged. |
| `slope-crowd` | Bounded complete-tick search finds a real walking guest on the bridge at progress 7. |
| `slope-removed` | Demolish that occupied bridge; the guest retains a South anchor and enters Lost. |
| `slope-recovered` | Rebuild with a fresh ID, advance 120 complete ticks, and require local guest recovery. |

The branch uses zero-based tiles: South ramp `(13,21)` low 8, Flat `(13,22)` height 9, East ramp `(14,22)` low 9, and Flat `(15,22)` height 10. Guest seeking is bounded to 18,000 complete ticks, not a timing threshold. Existing rotation/view/map/minimum-window flags remain. Cover all four rise directions and camera rotations in source tests plus a bounded capture matrix; do not infer North/West runtime coverage from a South/East fixture. Include minimum-window help. A PNG/zero exit is not visual or native-input proof.

All eleven final images in `artifacts/sloped-paths-0.13.0/captures/` exited 0 and were visually inspected without a blocking defect:

| Window | Scene / rotation / view / map |
| --- | --- |
| 1280 × 800 | slope-valid / 0 / Surface / Terraced |
| 1280 × 800 | slope-valid / 1 / Underground / Flat |
| 1280 × 800 | slope-blocked / 3 / Surface / Terraced |
| 1280 × 800 | slope-crowd / 0 / Surface / Terraced |
| 1280 × 800 | slope-crowd / 2 / Underground / Terraced |
| 1280 × 800 | slope-recovered / 3 / Surface / Flat |
| 1280 × 800 | overview / 0 / Surface / Terraced |
| 1024 × 680 | slope-built / 2 / Surface / Terraced |
| 1024 × 680 | slope-removed / 1 / Surface / Terraced |
| 1024 × 680 | help / 0 / Surface / Terraced |
| 1024 × 680 | terrain-raised / 2 / Surface / Terraced |

The Underground direction arrow, joins/planks, guest foot placement, and expanded panel/help were coherent. Removal showed a gap and one Lost guest; recovery showed zero Lost and 39 destinations. The blocked ghost was mostly occluded by the obstructing track, with readable rejection text. Static inspection establishes neither continuous motion nor exact cross-version pixel equality. Only the relocated built-slope capture was compared byte-for-byte.

```powershell
$slopeExe = (Resolve-Path -LiteralPath 'target\release\parkbound.exe').Path
$slopeCapture = Join-Path $env:TEMP ('Parkbound slope-' + [Guid]::NewGuid().ToString('N') + '.png')
$slopeRun = Start-Process -FilePath $slopeExe -ArgumentList @('--capture-frame', ('"' + $slopeCapture + '"'), '--capture-scene', 'slope-built', '--capture-rotation', '2', '--capture-view', 'surface', '--capture-map', 'terraced', '--capture-small-window') -WindowStyle Hidden -PassThru -Wait
if ($slopeRun.ExitCode -ne 0) { throw 'Slope capture failed' }
```

### Fresh-process slope continuation

`--save-check write-slopes` and a separate `--save-check verify-slopes` process write/check the production slot in a new absolute directory, producing `verification-slopes.txt`. `save_check/slopes.rs` prepares the same South/Flat/East/Flat branch, seeks a real mid-ramp guest at progress 7, rejects overlapping/out-of-bounds slope placement without mutation, and saves exact primary/backup states. The primary also contains a Custom terrain edit.

Continuation runs 3,600 ticks, removes the South bridge at tick 1,200, rebuilds it with a fresh ID at 1,500, closes the train at 1,800, and reopens it at 2,400. Actual ramp travel/high-landing arrival, complete primary/continued/backup equality, rejected-placement identity, and paused loading passed in release regressions, separate production processes, and the relocated package.

```powershell
$slopeProof = Join-Path $env:TEMP ('Parkbound slope proof é ' + [Guid]::NewGuid().ToString('N'))
$slopeExe = (Resolve-Path -LiteralPath 'target\release\parkbound.exe').Path
foreach ($slopeMode in @('write-slopes', 'verify-slopes')) {
    $slopeRun = Start-Process -FilePath $slopeExe -ArgumentList @('--save-check', $slopeMode, '--save-dir', ('"' + $slopeProof + '"')) -WorkingDirectory $env:SystemRoot -WindowStyle Hidden -PassThru -Wait
    if ($slopeRun.ExitCode -ne 0) { throw "Slope proof failed: $slopeMode" }
}
Get-Content -LiteralPath (Join-Path $slopeProof 'verification-slopes.txt')
```

The existing ordinary/path/terrain capture and continuation tools remain regression inputs. Migration QA uses isolated absolute directories and separate production processes. The verified recipe creates v1 with archived 0.12 and verifies it with 0.13; a separate v2 write is rejected by 0.12 with expected exit 1 and successfully verified afterward by 0.13. Headless control tests and the following native observations remain distinct evidence.

## Native Windows verification

The isolated packaged 0.13 session used a fresh QA save folder. Activation and screenshots succeeded. Space paused; C selected North, the shape button selected East, then C selected South. Wheel zoom changed 100% → 162%. One click built Path #93 at zero-based `(8,21)` (display `9,22`), lower height 8, upper 9, clearance 3; the inspector and saved v2 data agreed. Height buttons changed the selected range 8/9 → 9/10 → 8/9 without altering the built path.

F5 saved the ramp. X plus a map click removed only #93; the Save button persisted demolition. F9 loaded that demolished state paused and reset camera/tool to Inspect/Flat. The Backup button restored the ramp. F5 then produced bytes identical to the first native ramp save; its previous backup exactly lacked #93. Only the owned process was closed.

Page Up injection and two Shift+F9 helper spellings produced no visible change. These physical gestures remain unconfirmed; no cause is inferred. The separate Load button was not deliberately reverified. Successful native F9 and Backup-button actions do not imply that every keyboard modifier or separate button passed.

## Local handoff and package

Current shared workspace:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
```

The second-PC clone remains `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`. It is a separate path with no verified OneDrive synchronization. Use one active editing/build/Git session in the shared checkout, push source and current guides before switching PCs, and transfer LocalAppData saves separately.

Verified local archive location:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon\dist\Parkbound-0.13.0-windows-x64.zip
```

Extraction matched all 28 payload hashes and nineteen guides. A fresh temporary folder containing spaces and é, launched from `C:\Windows`, passed the slope proof and produced a byte-identical slope-built/rotation-2/Surface/Terraced/1024 × 680 PNG. Packaging includes 291 dependency notices, default-font/Rust notices, and one required MSVC DLL. `DEPENDENCIES.json`, `NATIVE_DEPENDENCIES.json`, `THIRD_PARTY_NOTICES.txt`, `RUST_STANDARD_LIBRARY_NOTICES.html`, and `VCRUNTIME140.dll` match 0.12.0 byte-for-byte. Updated guides require a final archive refresh and repeat payload/guide hash verification. The adjacent `.zip.sha256` is authoritative; do not embed the ZIP's self-hash in bundled guides.

The historical 0.12 native attempt sent no input after `GetCursorPos` access denied `0x80070005` and `IGraphicsCaptureItemInterop.CreateForMonitor` `0x80070057`. Activation/capture succeeded in 0.13; this is not a current blanket native blocker. Terrain B/V/buttons remain untested. Physical Page Up/Down/Delete, modifier shortcuts, the separate Load button, O/F/ride controls, pan/drag/DPI/display, and clean-PC offline/standard-user checks remain open. No local result establishes reference hardware or 60 FPS acceptance.

The recommended next bounded design is queue-path ownership and entry/connectivity rules for one explicit ride access point, contributing to BLD-02/03 and the Stage 3 park loop. Specify minimal ride identity/entrance rules first; boarding, a complete ride catalogue, and economy remain separate. This is a recommendation, not implemented scope.
