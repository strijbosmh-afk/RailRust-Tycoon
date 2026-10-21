# Guest admission and ordered queues — 0.15.0

Updated: 21 October 2026. This stage implements deliberate ride seeking, capacity-limited admission, ordered visible queue movement, patience, and physical return after cancellation. Work is on `codex/guest-admission`, based on completed 0.14.0 checkpoint `9e5a8c19354a24134bb25604c43914ee811d97df`. [PR #12](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/12) remains open/unmerged. New source/PR identity, tests, final build, runtime/native checks, CI, and packaging are pending; previous milestone totals do not verify this implementation.

Read with [the resume guide](../RESUME_DEVELOPMENT.md), [gameplay checklist](GAMEPLAY_CHECKLIST.md), and [product decisions](PRODUCT_DECISIONS.md). The [0.14 queue report](QUEUES_REPORT.md) preserves the historical public-only guest contract; this stage deliberately extends it for admitted guests. Historical reports, raw datasets, and notices remain unchanged. **No reference PC is selected; this development PC is explicitly excluded.** No local performance batch is planned, and reporting smoke checks do not establish hardware acceptance.

## Player-visible change

Building a valid owned queue and entrance now gives guests a reachable ride destination. They approach through public paths, join only at the public end when space is available, and walk in ticket order toward the station. A queue holds at most one admitted guest per queue tile; approaching guests do not reserve a place. Ramps retain their shared fixed-point height sampling.

This stage has no boarding, dedicated exit, train passengers, payments, or ride-completion events. The front guest waits on the queue path rather than entering the gate or train. Once the oldest admitted guest reaches 1,800 simulation ticks of patience—60 seconds at 30 Hz—the entire line returns tail-first to public paths. The whole-line return avoids opposing flows in a single-file queue. New admission stops until that return completes. A failed visit/return uses a 300-tick, ten-second cooldown before another attempt.

Closing or faulting the train, removing the gate, or making access invalid also starts return. Surviving reserved movement completes before reversal; there is no arbitrary teleport to a distant public path. A removed connection can strand a guest at its retained local owned-path anchor until repair enables local recovery. Paused edits reconcile visit/queue state immediately, while elapsed patience, cooldown, movement, and train ticks remain frozen.

## Controls and feedback

Construction controls remain P for public paths, J for queues, I for the entrance, C for shape/facing, X for visible demolition, and Page Up/Down or height buttons. O/Open/Close now also determines whether the existing ride accepts visitors: a valid, open, non-draining queue can admit them. The automatic train still follows its circuit and finish-lap close behavior without passengers. Space pauses all complete simulation steps; 1/2/3 select 1x/2x/4x. Save/Load/Backup retain their existing shortcuts and single-slot behavior.

The ride panel distinguishes topology errors, closed admission, available capacity, full queues, and returning/stranded guests. Queue length/capacity and oldest wait are separate from cumulative cancellation counts. Guest markers use shapes as well as colors: a diamond for approaching, bars for queueing, and a chevron for returning. They follow the same world depth and occlusion as the guest mesh; markers are not selection targets or a new guest-management window.

Existing help/UI/pan/focus and capture/diagnostic gates remain. Save includes a same-update edit, while Load/Backup suppresses pending edits even on failure. Successful load pauses and resets camera, selection, view, and tools; authoritative visit membership, tickets, patience, cooldown, and return progress remain saved.

## Simulation and state contract

`GuestState` remains the movement state (Walking, Waiting, Lost). `Guest.ride_visit` independently stores optional `RideVisit { ride, phase, ticket, waited_ticks }`; phases are Approaching, Queueing, and Leaving. `ride_cooldown` is authoritative. Approaching uses only public routes with no ticket reservation. Admitted/returning guests may use their ride's owned queue and public connection, never another owner's queue or a gate as a walking tile.

Tickets are stable and increasing. `ride_queue_order(ride)` returns ticket-ascending admitted identities, including Leaving members. Current/next queue cells are reserved so followers cannot overtake or share the same reserved queue tile. The ride stores draining state, the next ticket allocator, and cumulative accepted/turned-away/abandoned counts. Queue access remains derived from the existing physical gate and all owned fragments; a branch, loop, missing edge, wrong owner, or invalid station prevents admission.

| `ride_queue_stats` field | Meaning |
| --- | --- |
| seeking | Guests approaching through public paths; no reserved queue place. |
| queueing | Admitted moving or waiting guests. |
| leaving | Members returning after cancellation or patience expiry. |
| stranded | Leaving guests whose movement state is Lost; a subset of leaving. |
| capacity | Capacity derived from queue tiles. |
| moving / waiting | Movement subsets of queueing. |
| oldest_wait_ticks | Oldest admitted wait; authoritative ticks, not wall-clock time. |
| draining / admitting | Whole-line return and current eligibility for new admission. |
| joined | Cumulative accepted admissions. |
| turned_away | Canceled seeking attempts, counted once each. |
| abandoned | Admitted guests beginning return, counted once each. |

These counts describe visits and queue events, not income, completed rides, or passengers. Their subsets must not be summed as independent populations. Whole-Park equality/save continuation includes visit state, order, counters, allocator, RNG, and cooldown; derived graph/render caches remain excluded.

## Reproduce the four-place queue

Start the fresh 32 × 32 demonstration. It has twelve guests and an initially open train but no queue/gate. Use the following production construction fixture; displayed coordinates are one-based and directions remain world directions through camera rotation.

| Piece | Internal tile | Inspector tile | Shape/facing | Low height |
| --- | --- | --- | --- | --- |
| Existing station | (8,10) | (9,11) | West–East track | 8 |
| Entrance | (8,11) | (9,12) | North toward station | 8 |
| Queue nearest gate | (8,12) | (9,13) | Flat | 8 |
| Queue bridge | (8,13) | (9,14) | South | 8 |
| Upper landing | (8,14) | (9,15) | Flat | 9 |
| Return ramp | (8,15) | (9,16) | North | 8 |
| Existing public path | (8,16) | (9,17) | Flat | 8 |

Resume to observe public approaches and queue filling. The oldest wait eventually triggers physical return. Pause and close the ride or remove an occupied queue piece to observe immediate cancellation without advancing time. Rebuild the removed piece with the same shape/height; it receives a fresh object ID, and surviving guests must return/recover before new admission. Reopening alone does not skip a drain or cooldown. These are intended demonstrations; final runtime/native evidence is still pending below.

## Persistence and compatibility

New saves use strict **schema 4/content 4**. Required visit/cooldown and ride queue state distinguish valid ordinary guests from approaching, admitted, returning, or stranded guests. Validation rejects wrong owners, duplicate/orphan membership or tickets, invalid routes/progress/allocator state, and malformed version/field combinations before replacing the live park. Incomplete topology and valid return/Lost states remain saveable.

Strict 1/1, 2/2, and 3/3 saves migrate explicitly. Existing guests start without an admitted visit and with valid default cooldown/queue state; existing terrain, path shapes, ride identities, guests, RNG, and train continuation are preserved according to the migration contract. Older formats first acquire the already-defined public-path/ride defaults. Loading alone does not rewrite files; saving writes 4/4. The old 0.14 executable must reject v4, so retain closed-game legacy copies if reverting.

A connected legacy queue can now attract guests after loading: that is a deliberate new behavior. Migration checks compare the migrated initial state and deterministic replay under the new rules, rather than requiring future queue behavior to match 0.14's non-admitting simulation. Existing ordinary/path/terrain/slope fixtures without usable queues retain their legacy behavior. Default saves remain outside the shared checkout in `%LOCALAPPDATA%\Parkbound\saves`; the normal player map and save caps do not expand.

The new `write-admission`/`verify-admission` production proof saves an approaching backup, then a Custom-terrain primary with multiple members and actual mid-ramp movement. It continues exactly 3,600 complete ticks with close/open commands, occupied queue deletion/fresh-ID repair, gate demolition/rebuild, and public-edge removal/replacement. It checks role-aware references, reservation exclusivity, ticket order/capacity, conserved guest/ride identities, cancellation and renewed admission, full state equality, paused/Inspect loading, explicit backup, and unchanged primary/backup bytes. Save roundtrips run at every command boundary. Patience expiration is separately demonstrated by the natural leaving capture, rather than falsely attributing command-driven returns to patience.

```powershell
$exe = (Resolve-Path '.\target\release\parkbound.exe').Path
$qa = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound admission é ' + [guid]::NewGuid())
& $exe --save-check write-admission --save-dir $qa
if ($LASTEXITCODE -ne 0) { throw 'Admission writer failed' }
& $exe --save-check verify-admission --save-dir $qa
if ($LASTEXITCODE -ne 0) { throw 'Admission verifier failed' }
Get-Content -LiteralPath (Join-Path $qa 'verification-admission.txt')
```

The writer requires a new absolute directory with an existing parent; it refuses an existing directory before touching files. Each command is a separate process. Repeat with the extracted executable from `C:\Windows` to verify relocation. Never point proof commands at a player's save directory. The retained queue topology proof now validates roles rather than incorrectly requiring admitted guests to stay on public paths; older historical receipts are unchanged.

## Capture recipes and evidence

New scenes are `admission-approach`, `admission-moving`, `admission-full`, `admission-leaving`, `admission-cancelled`, and `admission-recovered`. They install the real queue/gate via production commands and seek actual state predicates for at most 12,000 extra complete ticks. They never inject guest positions, tickets, wait times, or counters. Moving captures a nonzero ramp pose; leaving requires actual return motion after natural patience expiry; cancellation removes a genuinely occupied queue tile; recovery requires a fresh ID and renewed admission.

```powershell
$exe = (Resolve-Path '.\target\release\parkbound.exe').Path
$images = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound admission captures ' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $images | Out-Null
& $exe --capture-frame (Join-Path $images 'full.png') --capture-scene admission-full --capture-rotation 0 --capture-map terraced --capture-view surface
& $exe --capture-frame (Join-Path $images 'return-small.png') --capture-scene admission-leaving --capture-rotation 2 --capture-view underground --capture-small-window
```

| Verification | Status for 0.15.0 |
| --- | --- |
| Source commit / new PR / exact-source CI | Pending. Branch is `codex/guest-admission` from `9e5a8c19354a24134bb25604c43914ee811d97df`. |
| Release tests, formatting, strict lint, production build | Pending. |
| State-machine, capacity, order, patience, cancellation, migration and input regressions | Implemented/in integration; execution pending. |
| Production admission and retained continuation modes; legacy migration and old-reader rejection | Pending on the final executable. |
| Six phase captures, rotations/views, normal/minimum windows, visual inspection | Fixtures implemented; runtime/image review pending. |
| Native build/queue/Open-Close/pause/save/load checks | Pending; historical native checks do not prove the new admission behavior. |
| Exact executable identity, dependency/notices, package hashes/guides and relocation | Pending. Use the archive's adjacent `.sha256`; do not embed a mutable ZIP self-hash. |

Native J/I/C queue/gate construction, physical Page Up/Down/Delete/modifier gestures, terrain/ride controls, pan/drag/DPI, clean-PC offline/standard-user operation, and reference-hardware selection remain open unless new receipts close them. The separate Load-button check passed narrowly in 0.14.0. Headless input tests and still images do not substitute for native interaction or continuous motion verification.

## Handoff and next major milestone

The shared checkout remains `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. The second-PC path remains `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`. Use the resume guide for toolchain/Git setup and transfer closed-game saves separately. Do not operate two PCs against the shared checkout at once.

After this stage passes its evidence gates, build **dedicated ride exits and a complete board–ride–unload loop** for the existing circuit. Specify exit placement/public connectivity, stable seat/passenger membership and dispatch, completed-lap unloading, and walking back to public paths. Test close/fault/demolition recovery without guest loss and exact saving while queued, boarding, riding, and unloading. This is the next substantial gameplay step; finance, multiple rides, guest needs, and a full track editor remain later scope. BLD-02, GST-01 and GST-05 remain broader in-progress families; RID-01 is not complete without the full ride operation criteria.
