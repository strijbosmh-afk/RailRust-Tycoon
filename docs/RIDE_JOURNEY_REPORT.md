# Complete ride journeys — 0.16.0

Updated: 21 October 2026. The bounded passenger journey implementation is complete: the existing queue now connects to six reserved seats, boarding, an occupied train lap, a dedicated exit, and completed public-path returns. Runtime source `25ed0a993f1c6b63dad2e659bc6b6c9dc93d5d38` is pushed on `codex/ride-journeys` from checkpoint `965e326` in [draft PR #14](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/14), stacked on still-open PR #13. All 326 release tests (210 game/116 simulation), formatting, strict all-target release Clippy, production build, migration/continuation, twelve inspected captures, scoped native controls, and relocated-package checks passed. [Exact-source Windows CI run 39621993525](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39621993525) passed all twenty steps; broader input/release gaps remain below.

Read with [the resume guide](../RESUME_DEVELOPMENT.md), [gameplay checklist](GAMEPLAY_CHECKLIST.md), [product decisions](PRODUCT_DECISIONS.md), and historical [admission report](ADMISSION_REPORT.md). Earlier reports preserve their named-build results. **No reference PC is selected; this development PC is explicitly excluded.** No local performance measurement batch was run. CI reporting smoke checks verify diagnostics, not hardware acceptance.

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

Shift+F9 recognizes either Shift key when held or when its press/release edge occurs in the same input update as F9. This bounded tolerance preserves rapid backup chords without a sticky backup mode: an ordinary F9 in a later update loads the primary. The release regression covers both Shift keys, a complete same-update chord, and release of a previously held modifier. Native checks on the final executable confirmed rapid left-Shift backup, an ordinary F9 returning to primary, and right-Shift backup.

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
| Exact source / new PR / Windows CI | Runtime source `25ed0a993f1c6b63dad2e659bc6b6c9dc93d5d38` pushed on `codex/ride-journeys`; [draft PR #14](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/14), stacked on open PR #13. [Windows CI run 39621993525](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39621993525) passed all twenty steps for this source, including formatting/lint, tests, packaging, all continuation proofs, reporting smoke and uploads. |
| Release tests, formatting, strict lint and final build | 326 release tests passed (210 game/116 simulation), formatting check passed, strict all-target release Clippy passed. Production release build passed. |
| Journey state, seat/transfer protection, close/fault/exit recovery, input and save regressions | Passed within the release suite. Input-resource tests verify scheduling/gates, not native Windows gestures. |
| Authentic legacy migration, old-reader rejection, retained/new fresh-process proofs | All 37 runtime processes passed. Authentic v1/v2/v3/v4 parks continued exactly; occupied v4 admission continued for 3,600 ticks after migration. Version 0.15 rejected v5 without changing either save slot. Retained path/terrain/slope/queue/admission proofs and the new 7,200-tick journey proof passed. |
| Six journey phases, retained captures, rotations/views and minimum-window inspection | All twelve final PNG hashes match the twelve inspected originals: eight journey variants plus admission, entrance, help and overview. The matrix covers all four rotations, Flat/Terraced, Surface/Underground, 1280 × 800 and 1024 × 680. |
| Native journey and persistence controls | Final executable passed F9, both Shift+F9 chords, ordinary F9 after backup, K/C exit-tool selection/facing, paused O open/close, Save button, Escape, 4x speed, Space, F5, and Backup button. Six actual completed-lap public returns were observed and confirmed in saved state. Exit placement/demolition and live repair were not exercised natively. |
| Binary identity | Final production executable: 46,813,184 bytes; SHA-256 `6815f2df25c0eb8e1fd2de06cc66a1d7be973927df8ab96d7da2b547b746a9a7`. |
| Notices, package hashes/guides and relocation | All 31 payload hashes and 22 current guides matched. A fresh extraction containing spaces and é, launched from `C:\Windows`, passed the 7,200-tick journey/backup/paused-load proof and produced a byte-identical minimum-window occupied-train PNG. Packaging retains 291 dependency notices, font/Rust attribution and one MSVC DLL; all five notice/manifest/runtime files match 0.15.0 byte-for-byte. |

Native verification used an isolated save directory. F9 loaded a completed primary at 03:37 paused with Done 6 and zero occupied seats; left-Shift+F9 loaded the occupied backup at 01:11 with Done 0 and six seats. A following ordinary F9 restored the primary, then right-Shift+F9 restored the backup. K selected Exit facing South and C rotated it West. O opened and closed while paused; Save preserved all twelve guest objects exactly and rotated the old primary into an identical backup. Escape selected Inspect, 3 resumed at 4x, and the occupied train was seen moving on the chainlift before returning Closed/Parked with zero seats and Done 6. Space and F5 saved tick 4,751 with boarded/exited/completed all 6, six guests with one completed trip each, and all twelve guests free of an active visit. The Backup button restored the occupied 01:11 state paused in Inspect. The owned window closed with Alt+F4, confirmed by a fresh matching-window inventory. Default player saves were untouched.

Actual native exit placement/demolition, live missing-exit repair, physical Delete/Page Up/Down, queue/gate construction gestures, F/Find, terrain tools, pan/drag/DPI and clean-PC offline/standard-user checks remain open. Native O and both Shift+F9 chords are now verified narrowly as above; this is not blanket verification of all ride controls. Licensing, public-name and representative-hardware decisions remain open. Headless tests and still captures do not prove unexercised gestures or performance.

Local receipts are under `artifacts/ride-journeys-0.16.0/`: `runtime-verification-final.txt`, `capture-inspection-final.txt`, `native-verification-final.txt`, `native-instance-final.json`, `package-verification.txt`, `relocated-runtime-verification.txt`, and `notices-comparison-final.txt`. The Windows archive is `dist\Parkbound-0.16.0-windows-x64.zip`; its adjacent `.zip.sha256` is authoritative. Final guide/archive, documentation commit and exact-source CI receipts are recorded separately in `final-handoff.txt` and PR #14. A later documentation head needs its own observed CI result; a mutable archive or documentation-head self-hash does not belong inside bundled guides.

## Handoff and next substantial slice

Shared checkout: `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. Second-PC checkout: `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`. Use the resume guide for the toolchain, Git and package workflows. Do not operate two PCs against the same shared checkout simultaneously. Preserve historical reports/data/notices and record final source and archive receipts separately from mutable documentation-head metadata.

The next substantial milestone is ride economy and operating controls. Specify deterministic guest wallets, ticket prices, ride revenue and park cash, with explicit payment/refund/cancellation semantics and exact saved accounting before implementation. Needs/ratings, shops, multiple rides, a flat ride, track editing and scenario goals follow. This passenger loop alone does not complete RID-01, GST-01 or the management inventory.
