# Single-tile terrain editing — 0.12.0

Updated: 21 October 2026. Current branch: `codex/terrain-editing`, based on completed 0.11.0 checkpoint `13c4063`. This is a draft implementation/handoff contract. New source identity, tests, runtime captures, native controls, continuation, CI, executable hash, and package verification are **pending**. The [path construction report](PATH_CONSTRUCTION_REPORT.md) preserves the completed 0.11.0 baseline; its results do not verify changed terrain code.

This bounded slice starts **BLD-06**, which remains in progress. It adds one-tile raising/lowering to the normal 32 × 32 park. It does not complete terrain brushes, terrain slopes, material/water tools, costs/refunds, or the full editor. No reference PC is selected: this development PC is explicitly excluded. No local performance measurement batch is planned; CI CPU smoke checks concern reporting rather than hardware acceptance.

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

Schema version 1/content version 1 already stores terrain heights and materials. No schema, content version, save limits, or dependency change is planned. Normal app load remains 32 × 32. Tool mode, preview, hints, camera, and pause/speed preference remain transient. Startup still creates the fresh demonstration without overwriting saves.

## Planned verification

These are acceptance requirements, not completed results. Record the exact tested source and binary before replacing pending entries.

| Area | Required evidence | Current status |
| --- | --- | --- |
| Source | Published implementation commit, review/base, and exact CI identity | Pending |
| Simulation | Immutable validation/setter parity; bounds/material preservation; path/track clearance; rejected-state equality; no object/guest relocation | Pending |
| Player input | B/V/buttons; one click only; top/cliff selection; Water/Underground rejection; UI/help/drag/focus/diagnostic gates; same-update Save and Load/Backup priority | Pending |
| Rendering | Valid/invalid directional previews, edited terrain, all rotations, surface occlusion, minimum window/help, unchanged retained path tools | Pending |
| Persistence | Fresh-process terrain primary/backup equality, Custom/paused loading, 3,600 continued ticks plus terrain/path commands, compatible historical saves | Pending |
| Source gates | Release workspace tests, rustfmt, strict all-target release Clippy, release build | Pending |
| Native controls | Normal Windows B/V/buttons, clicks, preset behavior, save/reload, and rejection feedback | Pending; earlier native results do not verify new tools |
| Delivery | Final executable size/hash, notices/runtime, all payload/guide hashes, extracted execution from unrelated working directory, and final archive sidecar | Pending |

### Terrain captures

Use the production executable's existing `--capture-frame`, rotation/map options, and `--capture-small-window`. `capture/terrain_scenes.rs` prepares production commands with ordinary input suppressed; these are not native keyboard/mouse checks. All terrain scenes except `terrain-underground` require Surface, and an explicit Underground option is rejected. The Underground scene forces cutaway mode and rejection. Empty dry targets start at height 4; the blocked fixture raises ground to a real path base before rejecting +1. Water remains unchanged. Implemented scenes awaiting verification:

| Scene | Intended proof |
| --- | --- |
| `terrain-raise` | Valid raise preview with no terrain mutation. |
| `terrain-lower` | Valid lower preview with no terrain mutation. |
| `terrain-raised` | One successful raise, resulting terrain, and Custom state. |
| `terrain-lowered` | One successful lower and resulting cliff/top geometry. |
| `terrain-blocked` | Shared-clearance rejection and unchanged terrain. |
| `terrain-water` | Player Water rejection. |
| `terrain-underground` | Underground edit suppression and no editable reference-floor ghost. |

Example after building the current source; actual exit/image checks are pending:

```powershell
$terrainCapture = Join-Path $env:TEMP ('Parkbound terrain-' + [Guid]::NewGuid().ToString('N') + '.png')
$terrainExe = (Resolve-Path -LiteralPath 'target\release\parkbound.exe').Path
$terrainRun = Start-Process -FilePath $terrainExe -ArgumentList @('--capture-frame', ('"' + $terrainCapture + '"'), '--capture-scene', 'terrain-raised', '--capture-small-window') -WindowStyle Hidden -PassThru -Wait
if ($terrainRun.ExitCode -ne 0) { throw 'Terrain capture failed' }
```

A valid PNG/zero exit is necessary but does not prove correct preview, depth, or layout. Inspect images and record actual dimensions/state. Preserve historical captures; keep new evidence under `artifacts/terrain-editing-0.12.0/`.

### Fresh-process terrain continuation

`--save-check write-terrain` requires a new absolute save directory whose parent exists. A separate `--save-check verify-terrain` process reads it and writes `verification-terrain.txt`, leaving the primary/backup unchanged. This diagnostic uses production save actions and terrain validation without a renderer.

The implemented proof fixture edits Grass at zero-based `(3,24)`, Sand at `(24,3)`, and the overlap column `(13,16)`. It stores custom terrain in primary/backup, compares complete loaded Park state and paused/Custom presentation, and continues 3,600 ticks with branch-path commands. The overlap sequence lowers height 8 to 2 and rejects 1 against buried track, then raises 2 to 8 and rejects 9 against the path. Every terrain command changes at most one height step. These proof cases are implemented but have not yet been run for this checkpoint.

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

Intended 0.12.0 archive location, **not yet built or verified**:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon\dist\Parkbound-0.12.0-windows-x64.zip
```

The adjacent `.zip.sha256` is authoritative after packaging. Do not place a ZIP's self-referential hash in its bundled guides. Refresh the package after final Markdown updates, verify every payload and current guide, then record the final source/CI/relocation evidence. Prior payload counts are historical and must not be copied into this checkpoint as new results.

## Remaining scope and next handoff

New B/V and terrain-button native verification is pending. Existing gaps remain: physical Page Up/Down and Delete, O/F/ride buttons, Backup and separate Save/Load buttons, panning/dragging and physical display/DPI, clean-PC offline/standard-user execution, and final hardware targets. Native 0.11.0 P/X, height buttons, F5/F9, and rejection/Inspect reset are retained historical results.

After this slice passes, the recommended next bounded design is a one-step sloped path connecting different heights under BLD-08. First specify endpoint heights, occupied clearance, compatible path edges, guest movement, preview/picking, and save-version handling; existing path objects have a single height. Do not silently treat slopes as geometry only or bundle queues, ride boarding, or the full terrain editor. BLD-06 and PRF-05 remain open for their broader criteria.
