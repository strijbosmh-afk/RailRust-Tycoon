# Queue paths and ride entrance report — 0.14.0

Updated: 21 October 2026. The bounded queue ownership and ride entrance connectivity slice is complete. Runtime source `4c8d42f00fe43a48ad5586240f153b833419d991` is pushed on `codex/queue-connectivity` in [draft PR #12](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/12), stacked on unmerged PR #11/`codex/sloped-paths` from completed checkpoint `effe6a8abe07b62be395c5efe72ffc1ce55d54a3`. All 285 release tests (190 game/95 simulation), formatting, strict all-target release Clippy, release build, production migration/continuation, eleven inspected captures, exact-source CI, and relocated-package checks passed. Native Load-button use passed; new queue/gate gestures remain unverified. This completes the stated connectivity implementation, not full queueing/ride gameplay or release acceptance.

Read with [the resume guide](../RESUME_DEVELOPMENT.md), [gameplay checklist](GAMEPLAY_CHECKLIST.md), and [product decisions](PRODUCT_DECISIONS.md). Historical reports and raw diagnostic datasets retain their original evidence. **This PC is excluded as a performance reference; no reference PC is selected.** No local performance measurement batch was run for this stage; CI CPU smoke checks verify reporting only.

## Bounded scope

The implementation adds blue queue paths owned by one stable ride identity, a physical entrance beside the existing Meadow Flyer station, and derived connectivity feedback. Queue paths support Flat and all four one-step slopes. There is no automatic gate or queue in a fresh demonstration: the player builds them.

`Ready` means the owned queue forms the required connection from one public path edge to the entrance. It does not admit visitors, create a line of waiting guests, board cars, or make money. The automatic train continues to obey its existing Open/Close controls independently of queue status. Public guests continue wandering on public paths and never use queue paths or the gate as ordinary destinations or recovery surfaces. BLD-02 and BLD-03 remain in progress for their full gameplay criteria.

## Controls and placement

| Input | Behavior |
| --- | --- |
| P / Path | Build a public path. |
| J / Queue | Build a blue queue owned by the existing ride. |
| I / Gate | Place the ride entrance beside a lateral edge of its station. |
| C / shape or facing button | In Queue, cycle Flat → North → East → South → West → Flat and remain in Queue. In Gate, rotate the entrance facing North → East → South → West. Outside these modes, select public Path and cycle its shape. |
| Page Up/Down or height buttons | Set Flat/entrance height 0–30, or slope low height 0–29. The default is 8. |
| Left click | Preview/commit one deliberate placement or demolition; holding does not paint. |
| X / Demolish, or Delete on a visible selected object | Remove a visible public path, queue path, or ride entrance. Track editing remains unavailable. |
| Escape / Inspect | Return to inspection, with the existing help-first behavior. |

The entrance facing points toward the station; its opposite edge faces the queue. It must sit at the station's height on a lateral side, not on the track's entry/exit ports. Preview and commit use shared immutable simulation validation. Clearance, occupied tiles, invalid owner/height/facing, hidden target planes, capacity, and allocator exhaustion reject atomically. R retains its public demonstration-path restoration purpose; it does not convert queues or overwrite player pieces.

Help/UI/panning/focus and capture/diagnostic gates remain in force. Save executes after a same-update world edit. Pending Load or Backup suppresses edits even if loading fails; successful loading resets the tool to Inspect. Rendering distinguishes blue queues and the entrance, while selection retains stable object identity.

## Authoritative model and topology

`RideId` and its allocator are authoritative. The train's ride identity survives a circuit fault and repair. `Ride` stores an optional `RideEntrance { object, facing }`; a `RideEntrance` object occupies a Flat two-step clearance column. Queue paths use `SurfaceObject.queue_owner: Some(RideId)`; public paths use `None`. A queue has the same endpoint geometry as an ordinary Flat/ramp path: Flat exposes four edges, ramps only two opposite ends, and connected edges must have equal endpoint heights.

Station attachment is derived from the current circuit and station ports. A gate is not permanently bound to a station object ID. A circuit fault makes access unavailable while retaining the physical gate/metadata; repairing the valid station can reconnect that same entrance. Demolishing the gate clears its entrance reference and rebuilding allocates a fresh object ID without replacing the ride identity.

`ride_access(ride)` evaluates every owned queue fragment. `Ready` requires one simple chain, the gate's outward connection at one end, and exactly one compatible public boundary edge at the other. The result includes queue tile count, station/public entry identities, and an ordered entrance-to-public queue only when Ready. Public contacts are counted as edges, not deduplicated path IDs. A branch, loop, extra fragment, foreign contact, or public entry at the wrong location cannot silently pass.

| Status | Meaning |
| --- | --- |
| RideUnavailable | The ride or usable circuit is unavailable. |
| NoEntrance / StationDisconnected | No gate exists, or it no longer attaches to the current station. |
| ForeignQueue | The topology touches another queue owner. |
| NoQueue | The ride has no owned queue tiles. |
| Branch / Loop | Owned topology contains a junction or a cycle. |
| Disconnected | Owned fragments do not form the required gate-connected chain. |
| NoPublicEntry / MultiplePublicEntries / PublicEntryNotAtEnd | Public access is absent, ambiguous, or joins the wrong part of the chain. |
| Ready | One gate-to-public chain satisfies the rules; boarding is still absent. |

The evaluator prioritizes unavailable ride/entrance/station, foreign ownership, missing queue, branch, loop, disconnected topology, then public-boundary errors. Incomplete construction is legitimate saveable state. Public guest graph/cache membership excludes queue/gate objects. Queue/gate-only edits do not invalidate the public graph; public path edits retain normal invalidation and guest reconciliation.

## Reproducible construction fixture

Use the fresh 32 × 32 demonstration, preferably paused. Coordinates below show both internal zero-based positions and the one-based inspector labels. All directions are world directions, independent of view rotation. Build the gate facing North and the four queue pieces in any convenient order; no public piece needs replacing.

| Piece | Internal tile | Inspector tile | Shape/facing | Low height |
| --- | --- | --- | --- | --- |
| Existing station | (8,10) | (9,11) | Track West–East | 8 |
| Entrance | (8,11) | (9,12) | Facing North | 8 |
| Queue nearest gate | (8,12) | (9,13) | Flat | 8 |
| Queue bridge | (8,13) | (9,14) | South | 8 |
| Queue upper landing | (8,14) | (9,15) | Flat | 9 |
| Queue return ramp | (8,15) | (9,16) | North | 8 |
| Existing public path | (8,16) | (9,17) | Flat | 8 |

Feedback after completing the fixture is four linked queue tiles with no boarding. Removing the bridge produces Disconnected; rebuilding it produces Ready with a fresh object ID. Adding a Flat queue at internal (9,14), height 9, creates a branch. Removing the gate produces NoEntrance; rebuilding it preserves the RideId. Removing public (8,16) produces NoPublicEntry; rebuilding the public piece restores Ready. These outcomes passed the source regressions and production capture/continuation checks below; ordinary queue construction gestures still need native retesting.

## Save compatibility and continuation

New saves write strict **schema 3/content 3**. Required fields include nullable object `queue_owner`, park `ride`, and `next_ride_id`. Owner/entrance references and public-only guest references are validated. Mismatched/future version pairs and unknown fields reject without replacing the running park or rewriting files.

Strict 1/1 and 2/2 saves migrate explicitly. Existing paths become public, no entrance/queue is invented, and an existing train—including a Fault train—receives deterministic RideId 1. Existing IDs, slopes/anchors where present, object order, guest routes, RNG, train state, and allocator state retain their defined continuation. The older 1/1 conversion additionally supplies Flat shapes. Old-format DTOs reject new-format fields instead of silently accepting a hybrid document.

Compatibility is directional: **0.13 intentionally rejects 3/3**. Keep closed-game copies of old saves if returning to older binaries. Loading does not rewrite the source save; the next Save writes 3/3. Saves remain in `%LOCALAPPDATA%\Parkbound\saves` by default, outside the shared checkout. The player app still accepts only 32 × 32 parks and retains one primary slot plus explicit backup.

The new production proof modes are `write-queues` and `verify-queues`. The writer creates a Ready backup, then saves a Disconnected primary with Custom terrain and a closing train. Verification loads exact primary state paused, continues 3,600 ticks with queue/public bridge removal and replacement, gate demolition/rebuild, and train close/open commands, and checks exact final state and backup. Every tick checks that public guests reference only public paths; fresh IDs and stable ride identity are asserted. Verification leaves primary/backup bytes unchanged and emits `verification-queues.txt`.

```powershell
$exe = (Resolve-Path '.\target\release\parkbound.exe').Path
$qa = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound queues é ' + [guid]::NewGuid())
& $exe --save-check write-queues --save-dir $qa
if ($LASTEXITCODE -ne 0) { throw 'Queue writer failed' }
& $exe --save-check verify-queues --save-dir $qa
if ($LASTEXITCODE -ne 0) { throw 'Queue verifier failed' }
Get-Content -LiteralPath (Join-Path $qa 'verification-queues.txt')
```

Run each mode in a separate process. The writer requires a new absolute directory whose parent exists and refuses an existing directory. Use the extracted `Parkbound.exe` path to repeat the proof outside the checkout; starting from `C:\Windows` tests independence from the working directory. Do not direct diagnostics at a player's real save directory.

## Capture recipes and verification gates

The six production scenes are `queue-entrance`, `queue-connected`, `queue-disconnected`, `queue-branch`, `queue-blocked`, and `queue-rebuilt`. They prepare the fixture through real tool commands where applicable, including visible demolition and a no-mutation blocked entrance click. QueueEntrance shows a valid preview without committing it. Rebuilt asserts fresh bridge identity. Capture mode suppresses normal input and preserves the prepared preview.

```powershell
$exe = (Resolve-Path '.\target\release\parkbound.exe').Path
$images = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound queue captures ' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $images | Out-Null
& $exe --capture-frame (Join-Path $images 'connected.png') --capture-scene queue-connected --capture-rotation 0 --capture-map terraced --capture-view surface --capture-ticks 600
& $exe --capture-frame (Join-Path $images 'rebuilt-small.png') --capture-scene queue-rebuilt --capture-rotation 2 --capture-map terraced --capture-view underground --capture-small-window
```

| Check | Current evidence |
| --- | --- |
| Exact source commit and PR | `4c8d42f00fe43a48ad5586240f153b833419d991`, pushed in [draft PR #12](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/12), stacked on unmerged PR #11. |
| Release tests, formatting, strict lint and build | All 285 release tests passed: 190 game/95 simulation. Formatting, strict all-target release Clippy, and production release build passed. |
| Topology, migration, guest exclusion, rejected edits and input ordering | Regression coverage passed in the release suite, including all-rotation/view/preset fixture preparation. Public guest isolation and fresh-ID topology repair also passed production continuation. |
| Captures and visual inspection | Eleven final captures exited 0 and were inspected: eight at 1280 × 800, three at 1024 × 680. The six queue scenes plus help/overview cover all four rotations, Surface/Underground and Flat/Terraced. No blocking visual defect was found. |
| Production migration and continuation | 0.12 v1 and 0.13 v2 saves migrated and continued successfully in 0.14. The old 0.13 reader rejected v3, preserving primary/backup hashes; new v3 verification still passed. Separate path, terrain, slope, and queue write/verify processes passed. Queue proof checks exact primary/final/backup state, 3,600 ticks plus commands, stable ride/fresh object identities, public-only guest references, and paused loading. |
| Native controls | The separate Load button restored an isolated seeded connected park at 00:24 paused, four linked queue tiles, twelve guests, and Inspect mode. J/I/C/queue/gate/demolition gestures remain unverified after automation stopped on detected desktop input. Historical public-path/slope/F5/F9/Save/Backup checks remain separate. |
| Exact-source Windows CI | [Run 39588999746](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39588999746) succeeded for runtime source `4c8d42f00fe43a48ad5586240f153b833419d991`, including formatting, strict lint, all tests, packaging, ordinary/path/terrain/slope/queue continuation, reporting smoke, and artifact uploads. Later documentation heads require their own CI status. |
| Executable | 46,488,576 bytes; SHA-256 `d5a2a741466ade1cab5d1649e9dfdc7f1fa7fdfa168375a4c232bfe62db75737`. |
| Package and relocation | All 29 payload hashes and twenty bundled guides matched. A fresh extraction into a Temp path containing spaces and é, launched from `C:\Windows`, passed queue continuation and a byte-identical minimum-window connected cutaway capture. The exact executable identity matched. |
| Attribution/runtime | Packaging includes 291 dependency notices, default-font/Rust notices, native manifests, and one required MSVC DLL. `DEPENDENCIES.json`, `NATIVE_DEPENDENCIES.json`, `THIRD_PARTY_NOTICES.txt`, `RUST_STANDARD_LIBRARY_NOTICES.html`, and `VCRUNTIME140.dll` are byte-identical to 0.13.0. |

Capture files under `artifacts/queue-connectivity-0.14.0/captures` are queue-entrance r0 Surface/Terraced; queue-connected r0/r1 Surface/Terraced, r2 Underground/Terraced/small, and r3 Surface/Flat; queue-disconnected r0 Surface/Terraced/small; queue-branch r2 Surface/Terraced; queue-blocked r3 Surface/Terraced; queue-rebuilt r1 Surface/Flat; help r0 Surface/Terraced/small; and overview r0 Surface/Terraced. Runtime, native, package, relocation, and notice receipts are `runtime-verification.txt`, `native-verification.txt`, `package-verification.txt`, `relocated-runtime-verification.txt`, and `notice-verification.txt` in the same milestone artifact directory.

The native session used the extracted executable and an isolated Temp save directory. Its primary was seeded from the queue proof's connected backup. After the Load-button success, two input-detected responses and refreshed state showed additional zoom/tool/selection changes outside the intended actions. Automation stopped; no cause or individual new control was inferred. The isolated window remained available, and no default player saves were accessed.

The local archive is `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon\dist\Parkbound-0.14.0-windows-x64.zip`. Its adjacent `.zip.sha256` is authoritative after documentation refresh; a ZIP self-hash is deliberately not embedded in bundled guides. The final 29-payload/twenty-guide package/documentation checkpoint is retained in `artifacts/queue-connectivity-0.14.0/final-handoff.txt`; later documentation-head CI is recorded separately in PR #12. This is an unsigned review build, not a published release.

Static image checks establish presentation, not continuous motion or native input. Headless input tests establish update/gating behavior, not physical shortcut handling. The separate Load-button gap is narrowly closed by the native result above. J/I/C in the new queue/gate modes, gate/queue placement and demolition, physical Page Up/Down, Delete/modifier gestures, terrain/ride controls, pan/drag/DPI, clean-PC offline/standard-user, and reference-hardware selection remain open. No FPS gain or hardware acceptance follows from this stage.

## Handoff and next bounded slice

The shared checkout remains `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. The recorded second-PC location remains `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`. See the resume guide for Rust/toolchain setup, Git synchronization, and separate save transfer. Do not run two PCs against the shared checkout simultaneously.

The bounded connectivity implementation is complete. Next, specify deliberate guest admission to the one ride's queue: eligibility, stable queue membership/order, capacity, cancellation when access changes, and exact save continuation. Boarding, seats/dispatch, exits, spending, ride ratings, multiple rides, and a track editor remain separate work. Do not expand map/save caps or evaluate the full hardware target on this excluded PC. Retest the unverified new native controls alongside the next stage rather than treating automated coverage as physical-input proof.
