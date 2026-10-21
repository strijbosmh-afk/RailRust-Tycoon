# One-step sloped paths — 0.13.0

Updated: 21 October 2026. Current work: `codex/sloped-paths`, based on completed 0.12.0 checkpoint `ef64609`. The slope/edge/guest and strict-save APIs are implemented. All 260 release tests passed (175 game/85 simulation), including six-scene command coverage and six slope-input regressions; formatting passed. Source/PR identity, strict lint/build, production runtime/native controls, migration/continuation, CI, executable hash, and package evidence remain **pending**. PR #10 is still open/unmerged; the intended new review base is codex/terrain-editing. The [terrain editing report](TERRAIN_EDITING_REPORT.md) retains the completed 0.12.0 results and native-input limitation; it is historical evidence, not verification of changed slope code.

This starts the bounded one-step slope slice under **BLD-08**, which remains in progress for its broader bridges/tunnels/construction criteria. Normal play/load remains 32 × 32, with the existing object/path/guest limits. Queues, boarding, costs/refunds, ownership, multi-tile construction, and a full terrain editor are outside this slice. No reference PC is selected; the user excluded this development PC. No local performance measurement batch is planned.

## Agreed player and geometry contract

| Choice | Meaning |
| --- | --- |
| Flat | Existing level path at height 0–30, occupying two height steps. |
| North / East / South / West | One-step uphill direction in world coordinates: North is −Y, East is +X; camera rotation does not change the world direction. |
| Slope low height | 0–29; high endpoint is low + 1. The conservative occupied column spans three height steps. |
| C / shape button | Cycle Flat → North → East → South → West → Flat and enter Path build mode. Selecting a slope clamps a previous height 30 to 29. |
| Page Up/Down / height buttons | Change the low-end height for a slope; retain the existing flat-height meaning for Flat. |
| Placement/demolition | One deliberate click per piece, shared validation/preview, fresh IDs on placement, visible-only demolition, and unchanged UI/help/drag/focus/save conflict gates. No hold-to-paint. |

These controls are implemented and covered by passing headless input regressions; native behavior remains unverified. Existing P/X/Inspect, B/V terrain editing, camera/rotation, and save controls remain. The build panel and help must distinguish direction of rise from camera direction and low-end height from the high endpoint.

`PathSlope` variants are `Flat`, `North`, `East`, `South`, and `West`. A ramp joins only through its two opposite ends; it has no side entrance. Two cardinal neighbors connect only when their facing edges both exist and have identical endpoint heights. Flat paths retain their compatible four-edge behavior. Diagonal proximity, base-height coincidence, or visually touching side faces do not create a connection. Track is never a guest path.

`Park::validate_sloped_path(pos, height, slope)` and `place_sloped_path(pos, height, slope)` share the player command contract. Existing `validate_path`/`place_path` remain Flat wrappers. Conservative clearance reserves the whole tile column, including the one-step rise. Placement, terrain edits/presets, and save validation must agree on that reservation. Rejected geometry, collision, capacity, or allocator checks leave all authoritative state and cache counters unchanged. Stable object IDs are not reused, and successful topology edits invalidate the derived path cache exactly as required.

Preview, emitted deck geometry, picking, selection, connected edges, and guest positions must use the same surface definition. All four world directions must remain correct in all four camera rotations, Surface and Underground views, at different heights and with foreground occlusion. The current generated art/geometry is original; no external assets are planned.

## Guest motion, demolition, and recovery

Shared simulation helpers are `PathDirection`, `paths_connect`, `path_edge_height`, `path_height_at`, and `path_center`, using `PATH_UNITS = 60`. `GuestPosition.point` samples the current path center, matching edge, and next path center piecewise. A ramp's height varies across its footprint, so simply interpolating the two object base heights is incorrect. Shared-edge position and height must be continuous in both travel directions, including Flat↔slope and slope↔slope joins. Exact old flat positions and the existing complete 30 Hz ticks, progress cadence, deterministic route order, seed/RNG, and wait behavior must remain.

`Guest.anchor_slope` retains a removed path's footprint orientation and base for lost-state positioning and incident-edge recovery. Arrival updates the guest's lower height and slope. Demolition must not snap a guest to an unrelated level or arbitrary nearest path. Recovery stays local and uses edge/surface compatibility; removal/rebuild tests must cover a guest on both halves of a slope transition, at a shared edge, and after save/load. Final runtime proof remains pending.

## Save version and migration contract

New saves use strict **schema version 2/content version 2**. Loading strict **1/1** explicitly migrates legacy paths to `Flat`, preserving authoritative terrain, objects/IDs/order/allocators, clock, guests/routes/progress/waits/RNG, and train state. A legacy walking guest mid-edge must continue the same flat route; Lost guests without a current path and faulted trains remain legitimate states. Migration must not allocate new path IDs or reinstall fixtures.

Mismatched version pairs, unknown/future versions, v1 documents containing new v2 fields, malformed slope representations, wrong-edge routes, invalid height/clearance combinations, and invalid references are rejected before replacing the live park. Existing bounded input/collection limits, explicit backup loading, lock/recovery behavior, and normal 32 × 32 app guard remain. A migrated file is not rewritten merely because it was loaded; explicit Save writes v2 through the production store.

**Compatibility is directional:** 0.13 is intended to load and migrate 0.12 v1 saves; 0.12 intentionally rejects v2. Do not run the old bidirectional-save recipe and call v2→0.12 rejection a regression. Keep a closed-game copy of the v1 primary and backup before testing explicit v2 writes when returning to an old executable is necessary. The ordinary one-slot backup is not an unlimited version archive.

The verification target is equivalent authoritative state and deterministic continued behavior after migration, not byte identity between v1 and v2 JSON. V2 same-build roundtrips must preserve every new slope/anchor field and complete continuation state. Save includes same-update edits; pending Load/Backup blocks competing edits even on failure. Successful load resets transient tools/camera/selection and pauses while retaining authoritative state.

## Acceptance and evidence

| Area | Required proof | Status |
| --- | --- | --- |
| Source and API | Final slope/edge/guest APIs, implementation commit, review/base | Implemented APIs passed release tests. Source/PR identity pending; intended base is still-unmerged PR #10/codex/terrain-editing. |
| Placement | Four directions; low/high bounds; conservative clearance; terrain/preset interaction; exact rejected-state/cache identity; existing caps | Covered by the passing 85-simulation-test suite; production click proof pending. |
| Connectivity | Opposing matching endpoints; flat↔slope and slope↔slope; wrong height/side/diagonal rejection; stable route ordering | Release simulation regressions passed. |
| Guest movement | Continuous fixed-point positions/heights; both travel directions; deterministic full ticks/speeds; local removal/recovery | Release regressions passed; production mid-ramp/continuation proof pending. |
| UI and rendering | C/shape/height controls; all-direction/all-rotation preview/pick/selection/depth; minimum window/help; input conflict gates | Release geometry, six-scene command matrix, and six slope-input regressions passed. Actual images/native controls pending. |
| Saves | Strict v1 migration; strict v2 roundtrip; malformed/future/version-pair rejection; primary/backup; new-process continuation through slope edits | Release codec/store/continuation tests passed. Production old-v1/new-v2 migration and separate executable proofs pending. |
| Native controls | Normal Windows slope selection/build/remove and save/reload, with exact observations | Pending; 0.12 activation/capture failures remain open |
| Source gates | Release tests, rustfmt, strict Clippy, production build | All 260 release tests passed (175 game/85 simulation); formatting passed. Strict release Clippy/build running. |
| Delivery | Exact executable size/hash, CI, notices/runtime, final payload/guide hashes, relocated continuation/rendering | Pending |

### Production capture scenes

`capture/slope_scenes.rs` implements six scenes covered by passing command-matrix tests; production captures remain pending:

| Scene | Prepared state |
| --- | --- |
| `slope-valid` | Valid South-ramp preview; the prepared Park remains unchanged by preview. |
| `slope-built` | Production click builds the South bridge to the higher landings. |
| `slope-blocked` | A real track envelope rejects the attempted bridge; prepared Park remains unchanged. |
| `slope-crowd` | Bounded complete-tick search finds a real walking guest on the bridge at progress 7. |
| `slope-removed` | Demolish that occupied bridge; the guest retains a South anchor and enters Lost. |
| `slope-recovered` | Rebuild with a fresh ID, advance 120 complete ticks, and require local guest recovery. |

The branch uses zero-based tiles: South ramp `(13,21)` low 8, Flat `(13,22)` height 9, East ramp `(14,22)` low 9, and Flat `(15,22)` height 10. Guest seeking is bounded to 18,000 complete ticks, not a timing threshold. Existing rotation/view/map/minimum-window flags remain. Cover all four rise directions and camera rotations in source tests plus a bounded capture matrix; do not infer North/West runtime coverage from a South/East fixture. Include minimum-window help. A PNG/zero exit is not visual or native-input proof.

```powershell
$slopeExe = (Resolve-Path -LiteralPath 'target\release\parkbound.exe').Path
$slopeCapture = Join-Path $env:TEMP ('Parkbound slope-' + [Guid]::NewGuid().ToString('N') + '.png')
$slopeRun = Start-Process -FilePath $slopeExe -ArgumentList @('--capture-frame', ('"' + $slopeCapture + '"'), '--capture-scene', 'slope-crowd', '--capture-rotation', '2', '--capture-small-window') -WindowStyle Hidden -PassThru -Wait
if ($slopeRun.ExitCode -ne 0) { throw 'Slope capture failed' }
```

### Fresh-process slope continuation

`--save-check write-slopes` and a separate `--save-check verify-slopes` process write/check the production slot in a new absolute directory, producing `verification-slopes.txt`. `save_check/slopes.rs` prepares the same South/Flat/East/Flat branch, seeks a real mid-ramp guest at progress 7, rejects overlapping/out-of-bounds slope placement without mutation, and saves exact primary/backup states. The primary also contains a Custom terrain edit.

Continuation runs 3,600 ticks, removes the South bridge at tick 1,200, rebuilds it with a fresh ID at 1,500, closes the train at 1,800, and reopens it at 2,400. It requires real ramp travel and arrival on the high landing. Verification compares complete primary, continued, and backup states and requires paused loading. These modes passed release regressions; separate production-process results remain pending.

```powershell
$slopeProof = Join-Path $env:TEMP ('Parkbound slope proof é ' + [Guid]::NewGuid().ToString('N'))
$slopeExe = (Resolve-Path -LiteralPath 'target\release\parkbound.exe').Path
foreach ($slopeMode in @('write-slopes', 'verify-slopes')) {
    $slopeRun = Start-Process -FilePath $slopeExe -ArgumentList @('--save-check', $slopeMode, '--save-dir', ('"' + $slopeProof + '"')) -WorkingDirectory $env:SystemRoot -WindowStyle Hidden -PassThru -Wait
    if ($slopeRun.ExitCode -ne 0) { throw "Slope proof failed: $slopeMode" }
}
Get-Content -LiteralPath (Join-Path $slopeProof 'verification-slopes.txt')
```

The existing ordinary/path/terrain capture and continuation tools remain regression inputs. New migration QA must use isolated absolute save directories and separate production processes. For legacy migration, first create a v1 file with the archived 0.12 executable, then verify it with 0.13; record logical continuation and v2 write behavior. Old 0.12 should reject the resulting v2 file without changing it. New headless control tests do not close the native Windows input gap.

## Local handoff and package

Current shared workspace:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
```

The second-PC clone remains `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`. It is a separate path with no verified OneDrive synchronization. Use one active editing/build/Git session in the shared checkout, push source and current guides before switching PCs, and transfer LocalAppData saves separately.

Intended archive location, not yet built or verified:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon\dist\Parkbound-0.13.0-windows-x64.zip
```

Final packaging must include the current guides, unchanged required notices/runtime where applicable, and verified payload hashes. Extract to a path with spaces/non-ASCII characters and run continuation/render checks from an unrelated working directory. Record actual counts rather than carrying forward the 0.12 package's 27 payloads/eighteen guides. Use the adjacent `.zip.sha256`; do not embed the archive's self-referential hash in its bundled guides. Source/PR/CI/executable/package identities remain pending.

The 0.12 native attempt sent no input: `GetCursorPos` failed with access denied `0x80070005`, followed by a refreshed-window retry failing `IGraphicsCaptureItemInterop.CreateForMonitor` with `0x80070057`. New slope controls need their own native proof. Carry forward terrain B/V/buttons, physical Page Up/Down/Delete, O/F/ride buttons, Backup and separate Save/Load buttons, pan/drag/DPI, and clean-PC offline/standard-user gaps unless directly reverified. No local result establishes reference hardware or 60 FPS acceptance.

After slope acceptance, the recommended next bounded design is queue-path ownership and entry/connectivity rules for one explicit ride access point, contributing to BLD-02/03 and the Stage 3 park loop. Specify the minimal ride identity/entrance contract first; do not bundle a complete ride catalogue, boarding system, or economy into that slice. This is a recommendation, not implemented scope.
