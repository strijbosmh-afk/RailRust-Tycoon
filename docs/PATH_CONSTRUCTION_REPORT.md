# Level-path construction report — 0.11.0

Updated 20 October 2026. **The bounded 0.11.0 level-path construction/demolition milestone is complete.** All 217 release tests, strict source checks, nine inspected captures, bidirectional 0.10/0.11 continuation, and the new construction save proof passed. Native P/X construction/demolition, height buttons, F5/F9 save/load, and rejection/reset behavior also passed. Source `eed600b1f86efb6c92dccb1a589167caeb44951f` is pushed on `codex/path-construction` in [draft PR #9](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/9), stacked on unmerged PR #8 from checkpoint `dbe650f`. [Windows CI run 39447839922](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39447839922) passed for that source. The package passed all 26 payload hashes, seventeen guide checks, relocated construction continuation, and a byte-identical capture. Physical Page Up/Down remains a separate input gap.

This begins the construction portions of BLD-01, BLD-02, and BLD-04. Their complete acceptance still requires costs/refunds, queues, and the wider construction loop. PRF-05 remains in progress. **No reference PC is selected; this development PC is explicitly excluded as a performance reference.** No FPS measurement or hardware acceptance is part of this stage.

## Player behavior

The path panel provides Inspect, Path, Demolish, height controls, a live pointer hint, and the last command result. Normal launches still create the seeded demonstration, and existing saves load explicitly.

| Input | Behavior |
| --- | --- |
| P / Path | Enter level-path placement mode. |
| X / Demolish | Enter visible-path demolition mode. |
| Escape / Inspect | Return to inspection; Escape closes help first when it is open. |
| Page Up / Page Down or height buttons | Adjust the construction height by one step, clamped to 0–30; default 8. |
| Left click in Build | Attempt one path tile on the selected height plane. |
| Left click in Demolish | Remove the nearest visible path under the pointer. |
| Delete / Remove path | Remove the selected path only if it is still visible at its picked center. |
| R / Restore paths | Restore missing demonstration paths atomically, subject to clearance and capacity. |

Holding the button does not paint repeated tiles. Right/middle dragging pans without placing paths. Terrain, track, decorative scenery, guests, and train cars are not demolition targets.

The preview projects onto the chosen height plane, using the same 0.22-step roof offset as a built path. It is green when placement is valid and red with an X when validation rejects it. A target hidden behind a nearer selectable surface has no ghost and explains the obstruction; rotate or use Underground view to inspect another exposed target. Demolition uses an orange footprint and X. Text and shape supplement color. Status uses one-based tile coordinates, matching the inspector; demolition says guest routes will be recalculated, without promising every disconnected guest can immediately recover.

Paths are level tiles with X-oriented planks and two height steps of clearance. Cardinal neighbors connect only at the same base height. Level 8 reaches the demonstration network; a disconnected path at another height does not create a ramp. Same-height joins update the deck footprint and navigation graph. No stairs, sloped paths, queues, bridges with physical support columns, ownership, construction prices, or refunds are added.

## Authoritative command and rendering contract

`Park::validate_path(pos, height)` and `Park::place_path(pos, height)` share validation. They check map bounds, supported height, terrain/object clearance, object-ID availability, and the existing save limits of 1,024 paths and 4,096 objects. Preview validation does not mutate the park. Insertion occurs only after all checks pass; rejection preserves objects, ID allocators, guest/RNG state, time, and path-cache state. Successful placement receives a fresh stable ID and invalidates derived connectivity. Generic internal fixture placement remains a separate API.

Fixture restoration respects the same remaining capacity and is atomic. It restores only missing demonstration segments, not player-built paths. Demolition reconciles affected guest references immediately, including while paused. Guests retain their local anchor in a lost state, then recover through the existing same-tile/cardinal same-height rules when ticks resume. Rebuilding a removed tile assigns a fresh ID; it never resurrects the old identity.

`path_tools.rs` owns transient tool state, pointer hints, and command dispatch. `world/path_preview.rs` shares projection, physical depth, connected footprints, and visible-surface picking with the world. Ghost meshes are separate, translucent, and nonselectable. Fit reserves both control columns, including the 236 × 284 tool panel on the right. Four separate transient/dynamic meshes hold selection, path preview, guests, and train; the normal landscape is a fifth mesh. Camera/rotation/view/static edits refresh the relevant presentation; successful edits and same-update loads rebuild all required scene/cache state. No terrain chunking or guest-rendering algorithm is changed by this milestone.

Input is ordered deliberately:

```text
UI / keys → terrain preset → selected Delete/Restore → train action
→ pointer construction/demolition → Save/Load/Backup → camera → scene
→ static chunk visibility → UI
```

Pending Load or Backup suppresses world edits before any mutation, even if the load later fails. Save runs after a same-update pointer edit and includes that edit. A successful load resets camera, selection, and tool state to Inspect/default height 8, starts paused, and reconstructs presentation from the loaded park. Delete wins over simultaneous Restore and consumes the world click so one frame cannot remove and then build accidentally. H/F1 help toggles consume their same-update click, including closing the overlay. Help, UI rectangles, dragging, loss of focus, render previews, and capture/profile modes retain their input gates.

The save schema/content versions remain 1. Ordinary saves include added and removed paths and all authoritative continuation state. Tool mode, ghost, height preference, camera, and command hints are transient and are not saved. Normal player maps and app loads remain 32 × 32; the 256-guest save limit is unchanged.

## Production diagnostics

The six path scenes call the same `path_tools::point_at` command helper after normal surface picking. They freeze presentation for a screenshot; they do not establish native mouse/keyboard acceptance.

| `--capture-scene` | Setup and assertion |
| --- | --- |
| `path-valid` | Green preview at zero-based tile (13,21), height 8, adjacent to the fixture. No park mutation. |
| `path-blocked` | Rejected Build preview over a visible occupied guest path. No park mutation. |
| `path-built` | Commit that nonfixture spur through the production tool; select its fresh path ID. |
| `path-demolish` | Orange preview on a visible occupied guest path. No park mutation. |
| `path-removed` | Remove a visible occupied path through Demolish; show the gap/lost state with unchanged simulation time. |
| `path-recovered` | Remove and rebuild that exact tile through the tools, require a fresh ID, then advance 120 complete ticks and require local guest recovery. |

`--capture-ticks` remains the initial tick count (default 600), and `path-recovered` adds 120 ticks. All four rotations, Flat/Terraced, Surface/Underground, and the 1024 × 680 option remain available. Scene coordinates use zero-based simulation tiles; the inspector displays one-based coordinates. Each path scene focuses its target at 200% zoom, clear of the right-side tool panel.

```powershell
$pathCapture = Join-Path $PWD 'artifacts\path-built-small.png'
.\target\release\parkbound.exe --capture-frame $pathCapture --capture-scene path-built --capture-small-window --capture-rotation 1
```

Use a fresh filename and wait for exit before inspecting the PNG. Existing crowd/train/overlap/save captures and isolated large-map preview rules remain regression coverage.

The path-specific fresh-process diagnostic extends the existing save proof with a nonfixture branch, demolition, an alternate connection, and an occupied-path removal. It uses production save/load behavior in an isolated directory and produces `verification-paths.txt`.

```powershell
$pathSaveDirectory = Join-Path $env:TEMP ('Parkbound path proof ' + [guid]::NewGuid())
$gameExecutable = Join-Path $PWD 'target\release\parkbound.exe'
foreach ($mode in @('write-paths', 'verify-paths')) {
    $proofArguments = '--save-check {0} --save-dir "{1}"' -f $mode, $pathSaveDirectory
    $proofProcess = Start-Process -FilePath $gameExecutable -ArgumentList $proofArguments -WindowStyle Hidden -Wait -PassThru
    if ($proofProcess.ExitCode -ne 0) { throw "Path proof failed: $mode" }
}
Get-Content -LiteralPath (Join-Path $pathSaveDirectory 'verification-paths.txt')
```

The writer requires a new absolute directory with an existing parent and refuses to overwrite it. The final executable passed both new processes: exact primary state, 3,600 continued ticks plus commands, fresh IDs, previous backup, and paused loading. Separate 0.10-writes/0.11-verifies and 0.11-writes/0.10-verifies processes also passed 3,600 ticks plus commands, backup, and paused loading.

## Verification register

| Check | Current 0.11.0 evidence |
| --- | --- |
| Implementation source / PR | `eed600b1f86efb6c92dccb1a589167caeb44951f`, pushed on `codex/path-construction` from `dbe650f`, in [draft PR #9](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/9), stacked on unmerged PR #8. |
| Formatting, strict Clippy, release build | The local 217-test full suite passed before the last hint-only text adjustment. After that adjustment, all eleven path-input tests, final-source rustfmt, strict all-target release Clippy, and the final release build passed. Hosted CI then passed the complete source workflow on `eed600b`. |
| Release tests | 217 passed: 146 game and 71 simulation. The 24 additions are eleven input, three capture, three world-preview, six simulation, and one save-check test. Receipt: `artifacts/path-construction-0.11.0/cargo-tests.txt`. |
| Command/input behavior | Passing tests cover preview nonmutation, clearance/capacity/ID rejection, visible-only demolition, input gates, conflict precedence, same-update save/load, paused immediate refresh, and hidden-plane rejection. Existing Fit tests pass with both control columns reserved. These are supplied-input/system checks, not native Windows observations. |
| Rendered diagnostics | Nine final captures exited successfully and were visually reviewed, including the six path scenes and regression views. Captures establish command/render behavior; native input is a separate check. |
| Production continuation | New write-paths/verify-paths proof passed exact primary state, 3,600 continued ticks/commands, fresh IDs, backup, and paused load. Both 0.10→0.11 and 0.11→0.10 ordinary save proofs passed. |
| Executable identity | 46,213,120 bytes; SHA-256 `8e3bfc2971791f0eb85577d0ae8608bf99bba37e16141be00d101dfd7622e6d0`. |
| Hosted Windows CI | [Run 39447839922](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39447839922) succeeded for `eed600b1f86efb6c92dccb1a589167caeb44951f`, including tests, build/package, continuation, CPU-wrapper smoke checks, and artifact upload. Later documentation heads have separate CI status. |
| Package / relocated execution | Package refresh verified all 26 payload hashes plus the manifest and seventeen current guides. A fresh TEMP path containing spaces and `é`, launched from `C:\Windows`, passed write-paths/verify-paths primary equality, 3,600 ticks/commands, fresh IDs, backup, and paused load. Its path-built/rotation-2/Surface/Terraced/1024 × 680 PNG was byte-identical to the final source capture. |
| Native controls | Verified in an isolated temporary save directory: P/Path and X, click placement/removal, height buttons, F5/F9, blocked clicks, Escape, and owned-window close. Receipt: `artifacts/path-construction-0.11.0/native-verification.txt`. Physical Page Up/Down remains unverified; helper injection produced no visible change despite passing supplied-input tests. |

Final images are under `artifacts/path-construction-0.11.0/captures/`. The reviewed matrix is:

| Scene | Rotation / view / terrain | Window |
| --- | --- | --- |
| `path-valid` | 0 / Surface / Terraced | 1280 × 800 |
| `path-blocked` | 1 / Surface / Terraced | 1280 × 800 |
| `path-built` | 2 / Surface / Terraced | 1024 × 680 |
| `path-demolish` | 3 / Underground / Flat | 1280 × 800 |
| `path-removed` | 0 / Surface / Terraced | 1280 × 800 |
| `path-recovered` | 1 / Surface / Terraced | 1280 × 800 |
| `path-valid` | 3 / Underground / Flat | 1024 × 680 |
| `help` | 0 / Surface / Terraced | 1024 × 680 |
| `overview` | 0 / Surface / Terraced | 1280 × 800 |

The native session paused with Space at 02:39, entered Build with P, rejected an outside-plane click, and placed fresh Path #93 at displayed tile (11,19), height 8. F5 saved it; X and a map click removed it. F9 restored the path at the same paused time and reset the tool to Inspect. The + and − buttons changed height 8→9→8. The Path button entered Build; clicking the occupied restored tile was rejected. Escape returned to Inspect, and Alt+F4 closed only the owned test process. These observations establish the tool and F5/F9 flow; they do not verify every save button or Backup.

Physical Page Up/Down, Delete, O/F and related buttons, pan/drag/DPI/display checks, native Backup, and a clean Windows PC remain open unless explicitly covered by separate evidence. The Page Up/Down injection limitation does not invalidate the verified height buttons.

The archive's adjacent `.zip.sha256` is authoritative. Do not place its self-referential final hash in a guide bundled inside that same archive. Historical reports, JSON datasets, checksums, and notice bytes remain unchanged.

The local archive is:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon\dist\Parkbound-0.11.0-windows-x64.zip
```

Its adjacent `Parkbound-0.11.0-windows-x64.zip.sha256` records the current ZIP hash. The package contains 291 dependency notices, default-font and Rust notices, and one required MSVC DLL. Notice and DLL hashes are unchanged from 0.10.0. Receipts are `package-verification.txt`, `relocated-runtime-verification.txt`, and `notice-verification.txt` under `artifacts/path-construction-0.11.0/`. Refresh and recheck all 26 payload hashes and seventeen bundled guides after documentation-only updates; such updates change the archive, not the verified executable identity.

The camera-fit change reserves the new right control column. Current diagnostic framing and selected-chunk counts can therefore differ from historical 0.10.0 records. No local performance measurement batch was run; CI CPU smoke checks verify reporting only; historical raw results remain unchanged and are not new performance claims.

## Next bounded step

Preserve this completed checkpoint and its narrow native gaps. The next recommended slice is a single-tile terrain raise/lower tool on the existing map, using the existing atomic terrain-height setter and shared clearance. Include a preview, understandable rejected edits, and save/continuation in the same change. This starts BLD-06 without claiming brush tools, water editing, supported slopes, prices, or complete construction. Queue/ride/economy work remains in the Stage 3 plan; the final game scope is unchanged.
