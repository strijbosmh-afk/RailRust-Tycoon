# Complete ride journeys — 0.16.0

Updated: 21 October 2026. This stage connects the existing queue to boarding, six reserved passenger seats, an occupied train lap, a dedicated exit, and a completed return to public paths. Work is on `codex/ride-journeys` from completed 0.15.0 checkpoint `965e326`. Final source/PR identity, release tests/build, production migration/continuation, inspected captures, native controls, CI and package verification remain pending. The implemented contract below is not a claim that those evidence gates have passed.

Read with [the resume guide](../RESUME_DEVELOPMENT.md), [gameplay checklist](GAMEPLAY_CHECKLIST.md), [product decisions](PRODUCT_DECISIONS.md), and historical [admission report](ADMISSION_REPORT.md). Earlier reports preserve their named-build results. **No reference PC is selected; this development PC is explicitly excluded.** No local performance measurement batch is planned. Any CI reporting smoke checks verify diagnostics, not hardware acceptance.

## Player-visible behavior

The existing three-car train now has six seats, two per car. Guests approach and join the owned queue as before. With an open ride, valid entrance/queue, connected exit, and no queue drain, guests at the front can reserve seats and board through the entrance. Boarding and exiting are physical 60-tick transfers, taking two simulation seconds each at 30 Hz. A boarding guest continues to reserve the queue-front tile until that transfer ends. Transfer progress and occupied seats are authoritative state; rendering presents them rather than inventing passenger positions.

A completed trip requires a real departed lap and return through the exit onto a public path. Reaching a seat alone is not a completed ride. `boarded` counts arrivals at seats; `exited` counts public returns, including passengers returned after a predeparture close; `completed` and each guest's `rides_completed` count actual completed-lap public returns. Queue patience/cancellation counters remain separate.

The no-exit demonstration keeps its previous automatic empty-train and queue-admission behavior. Building an exit enables passenger journeys; it does not silently create a gate, public connection, or queue. The existing ride is still a fixed original circuit, not a full track editor or configurable collection of ride types. Pricing, guest spending, income, ride ratings, and general needs are not implemented in this stage.

## Controls and exit connectivity

K or Exit selects the new exit-placement tool. Its default facing is South toward the existing station, independent of the entrance tool's default North. C rotates whichever portal tool is active; public/queue shape cycling remains unchanged. Exit height starts at 8 and uses the existing height controls. The ride panel provides Open/Close, Find, Gate, and Exit actions. Existing inspection, visible demolition, help, camera, and persistence controls retain their conflict gates.

An exit is a physical Flat portal beside a lateral edge of the station at the same height. Its facing points toward the station. The outer edge needs a public landing connected through public paths to the queue entry's public component before new boarding is allowed. Queue paths or another portal are not public landings. Exit statuses distinguish Ready, RideUnavailable, NoExit, StationDisconnected, NoPublicExit, and PublicDisconnected.

Already-carried passengers use the actual station, exit portal, and landing when unloading, even if entrance/queue access has since broken. A surviving but locally isolated public landing can therefore receive them before normal public-path Lost/recovery behavior applies. This prevents an unrelated entrance failure from trapping a finished journey when a physical exit remains usable.

## Passenger safety and authoritative state

The visit phases add Boarding, Riding, and Exiting to Approaching, Queueing, and Leaving. Passenger phases clear ordinary current/next/destination/route references; `RideVisit.passenger` instead stores `PassengerJourney { seat, departed, completed_lap, transfer }`. A transfer stores its path, portal, and progress. Boarding/Exiting require a transfer; Riding has none, including a passenger held at the station. Ordinary path progress and passenger transfer progress must not be confused.

`ride_journey_stats` reports seat capacity, boarding/riding/exiting populations, boarded/exited/completed counters, a held flag, and exit status. `ride_guest_positions` supplies physical poses. Passenger membership and seats are unique; guests are neither duplicated on paths and cars nor deleted after a trip. `ride_queue_order` includes only Queueing/Leaving tickets, excluding passengers; the boarding transfer still reserves its source queue tile.

Active transfer paths and portals, and circuit track while passengers are aboard, reject demolition atomically with a reason. Rejection preserves state, IDs, save bytes and cache counters. An unused exit may be removed while the train is on its lap: passengers finish the lap, then wait at the station until an exit is repaired. They are not teleported to a distant path. Closing in flight preserves the lap and physical unloading; an empty ride can still fault after track demolition, and fixture repair parks it closed with fresh missing-object IDs. Only one guest uses a station walkway at a time. Boarding assigns the lowest free seat; completing a boarding transfer resets the open train's 90-tick dwell. Existing transfers and completed-lap passengers hold the train until resolved. Empty trains retain the legacy automatic behavior.

Pause freezes movement, transfers and timers. Commands still reconcile permitted intent without advancing ticks. Save includes same-update edits; Load/Backup takes precedence over edits even on failure. Successful load pauses, resets camera/selection/tool to Inspect, and retains the exact passenger/seat/transfer state.

## Reproduce the connected journey fixture

Start the normal 32 × 32 park. Directions remain world directions through camera rotation; inspector coordinates are one-based.

| Piece | Internal coordinate(s) | Inspector coordinate(s) | Shape/facing and height |
| --- | --- | --- | --- |
| Existing station | (8,10) | (9,11) | West–East, height 8 |
| Entrance | (8,11) | (9,12) | North, height 8 |
| Queue nearest gate | (8,12) | (9,13) | Flat, height 8 |
| Queue bridge | (8,13) | (9,14) | South ramp, low 8 |
| Upper queue landing | (8,14) | (9,15) | Flat, height 9 |
| Return queue ramp | (8,15) | (9,16) | North ramp, low 8 |
| Existing queue public entry | (8,16) | (9,17) | Flat, height 8 |
| Exit | (8,9) | (9,10) | South, height 8 |
| New public return row | x8 through x13 at y8 | x9 through x14 at y9 | Flat, height 8 |
| New public return column | x13 at y9 through y11 | x14 at y10 through y12 | Flat, height 8 |
| Existing public connection | (13,12) | (14,13) | Flat, height 8 |

The return column passes under the lift at (13,10): public occupied height 8–10 fits below the track's lower height 12. The route avoids adding another public contact to the owned queue. The diagnostic fixture uses production path/portal commands, then advances real simulation states without injecting passengers, tickets, transfer progress, or completion counts.

## Persistence and migration

The format advances to strict **schema 5/content 5**, with explicit strict migration from versions 1/1 through 4/4. Existing v4 queue visits, tickets, patience, cooldown and counters retain their behavior; migration adds no exit or passenger by itself. The retained no-exit admission fixture must still reproduce its previous behavior. New exit metadata, seat/passenger/transfer state and completed-trip counts persist. Unknown or mixed versions and invalid membership/references must reject before replacing live state. Old 0.15 readers intentionally reject v5. Loading alone does not rewrite a file; saving writes the new format.

Default saves remain in `%LOCALAPPDATA%\Parkbound\saves`, outside the app directory and shared checkout. Normal play remains 32 × 32; save count/size limits do not expand. Retain closed-game legacy copies if reverting, and transfer LocalAppData saves separately when switching PCs.

The new `write-journey`/`verify-journey` proof uses production Save/Load/Backup commands. It saves an actual boarding transfer as backup and a moving occupied train with Custom terrain as primary. Continuation runs exactly **7,200 complete ticks**, with bounded actual-phase conditions for these events:

- Reject occupied circuit demolition and boarding/exiting path/portal demolition without mutation.
- Close in flight and remove the unused exit; witness the returned passengers held at the station.
- Rebuild the exit with a fresh identity, unload physically, and count actual completed public returns.
- Remove track only after the closed ride is empty, observe Fault, repair to Closed without changing time or ride identity, reopen, and observe another completed trip.

Every command boundary and held/transfer checkpoint round-trips full state. Every tick checks guest identities, seat uniqueness, permitted path/portal roles, passenger poses and completion accounting. The verifier repeats the exact command sequence, compares the complete final Park, loads the exact boarding backup paused/Inspect, and verifies that primary/backup files remain unchanged. The retained ordinary/path/terrain/slope/queue/admission modes remain regression checks; schema assertions advance to 5/5 without rewriting historical receipts.

```powershell
$exe = (Resolve-Path '.\target\release\parkbound.exe').Path
$qa = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound journey é ' + [guid]::NewGuid())
& $exe --save-check write-journey --save-dir $qa
if ($LASTEXITCODE -ne 0) { throw 'Journey writer failed' }
& $exe --save-check verify-journey --save-dir $qa
if ($LASTEXITCODE -ne 0) { throw 'Journey verifier failed' }
Get-Content -LiteralPath (Join-Path $qa 'verification-journey.txt')
```

The writer requires a new absolute directory and refuses an existing one before touching its contents. Each invocation is a fresh process. Repeat with the extracted executable launched from `C:\Windows`; never point proof commands at a player's save folder.

## Production captures

The six new scenes are `journey-boarding`, `journey-riding`, `journey-unloading`, `journey-completed`, `journey-exit-blocked`, and `journey-recovered`. They seek real phase predicates for at most 18,000 extra ticks per seek. Boarding/unloading capture nonzero transfer progress; riding requires departed occupied movement; completion requires a real public return. Missing-exit and repair scenes remove an unused exit while Riding, witness a station hold, and require renewed completed return after a fresh-ID repair.

```powershell
$exe = (Resolve-Path '.\target\release\parkbound.exe').Path
$images = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound journey images ' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $images | Out-Null
& $exe --capture-frame (Join-Path $images 'riding.png') --capture-scene journey-riding --capture-rotation 0 --capture-map terraced --capture-view surface
& $exe --capture-frame (Join-Path $images 'exit-small.png') --capture-scene journey-unloading --capture-rotation 2 --capture-view underground --capture-small-window
```

## Verification status

| Gate | Evidence for 0.16.0 |
| --- | --- |
| Exact source / new PR / Windows CI | Pending; branch `codex/ride-journeys` from `965e326`. |
| Release tests, formatting, strict lint and final build | Pending. |
| Journey state, seat/transfer protection, close/fault/exit recovery, input and save regressions | Implemented/in integration; execution pending. |
| Authentic legacy migration, old-reader rejection, retained/new fresh-process proofs | Pending on final executable. |
| Six journey phases, retained captures, rotations/views and minimum-window inspection | Implemented; execution/visual review pending. |
| Native exit construction and actual journey controls | Pending; no earlier native check proves the new exit or passenger behavior. |
| Binary identity, notices, package hashes/guides and relocation | Pending; final archive checksum belongs in its adjacent `.zip.sha256`, not inside bundled guides. |

Carry forward physical Page Up/Down/Delete/modifier, queue/gate, terrain/ride, pan/drag/DPI, clean-PC offline/standard-user, licensing/public-name and hardware-selection gaps. The native 0.15 attempt activated the isolated app but stopped at the first F9 input-detected response; it added no native-control pass. Earlier successful gestures retain their named-build scope. Headless input and still captures do not establish native gestures, continuous motion or performance.

## Handoff and next substantial slice

Shared checkout: `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. Second-PC checkout: `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`. Use the resume guide for the toolchain, Git and package workflows. Do not operate two PCs against the same shared checkout simultaneously. Preserve historical reports/data/notices and record final source and archive receipts separately from mutable documentation-head metadata.

Complete this stage's evidence gates, then build the next substantial milestone: ride economy and operating controls. Specify deterministic guest wallets, ticket prices, ride revenue and park cash, with explicit payment/refund/cancellation semantics and exact saved accounting before implementation. Needs/ratings, shops, multiple rides, a flat ride, track editing and scenario goals follow. This passenger loop alone does not complete RID-01, GST-01 or the management inventory.
