# Coaster testing and loading policies — 0.33.0

This milestone starts from 0.32 checkpoint `303cd66a0e1789cb0bf4b3d3365233440efefebb` / PR #30 on `codex/coaster-tests-and-dispatch`. It advances RID-04/RID-05/RID-06 with bounded mechanics testing and loading policies. The bounded 0.33 implementation is locally verified: 674 release tests, strict source/build checks, production continuation, inspected captures and fresh package checks passed. Authentic schema-16 migration and unchanged-slot old-reader refusal passed; exact-source CI is pending. [RIDE_ACCESS_ASSISTANCE_REPORT.md](RIDE_ACCESS_ASSISTANCE_REPORT.md) preserves the locally verified 0.32 package; its push CI [40108342235](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40108342235) passed all 32 steps and 634 tests.

## Coaster tests

Open **Rides [Y]**, select the coaster and choose **Test**. Close the ride and let it stop and unload, then run an isolated lap of the installed circuit. Apply draft changes first. The test uses the same sampled geometry, station wait, lift speed, brakes and approximate motion rules as the live train. It leaves the park clock, guests, cash, live train, other rides, draft/history and saved files unchanged.

The result shows sampled track length, piece counts, moving/station/total time, minimum and maximum moving speed, height range, curves/lifts/drops/brakes, total ascent/descent and longest continuous drop. Length and speed use game tiles and tiles per second; heights use terrain steps. These are not physical metres or km/h. Original excitement/intensity/nausea estimates are bounded 0–100 layout heuristics. They are separate from guest reviews and do not change guest demand. G-forces, full rigid-body physics and safety certification are not provided.

The mechanics test is empty: it excludes passenger minimum-load thresholds and extra loading waits while retaining the ordinary station dwell. Missing, incomplete, open, moving or occupied coasters are refused. A bounded 36,000-tick limit reports failure if a lap cannot finish; it produces no partial rating. Carousel tests are not included.

Results are transient. A changed installed layout or motion settings, opening/occupying the ride, changing selected ride, or loading a park invalidates the displayed result. Editing an unapplied draft leaves a valid installed-track result intact. Loading policies do not affect the mechanics-only result.

Operations, Access and Test are exclusive tabs in the existing 820 × 540 Rides panel, with a visible active tab even in controlled captures. A deliberate Run press executes once; opening Test alone does not run a lap. Modal transitions, ride browsing, F9/Shift+F9 intent and read-only capture/diagnostic modes consume pending actions. A successful Load/Backup clears the cached report, notices and pending actions even when restored inputs match. Ordinary panel closure retains a still-valid report.

## Passenger loading policies

The **Operations** tab adds **Minimum load** (0–6, where 0 means no minimum) and **Extra loading wait** (1–60 seconds, in whole-second steps). Defaults are no minimum and 30 seconds of extra wait. Each ride keeps its own policy. Policy edits require a closed, stationary, empty ride. Reset prechecks both edit guards before restoring operating settings and loading policy in one uninterrupted operation; a refusal cannot partially reset the ride. An unchanged reset preserves state. While waiting for load, Operations displays seated/minimum passengers and elapsed/maximum extra wait.

The existing station wait runs first. If the minimum load is unmet at an otherwise safe departure opportunity, the ride waits up to its extra-wait limit, then may leave below the requested load. Only fully seated fresh passengers count. Boarding, unloading, mandatory station dwell and blocked exits are excluded from the extra-wait timer; therefore it is not a wall-clock departure deadline. Boarding can restart ordinary dwell without erasing already accumulated extra wait. Closing/draining retains passenger safety and bypasses the optional minimum where needed; it never forces departure through an existing transfer/exit hold. Actual departure, close, fault and rebuild reset the accumulated timer.

Fare reservation, charge-once departure, refunds, guest ownership and financial reconciliation keep their existing rules. Changing policy does not reprice reservations or spend money. Multiple trains and configurable car/seat counts remain later work.

## Save contract

New park saves use strict schema/content **17/17**. A bounded sorted list of unique ride owners stores nondefault policies and elapsed extra-wait ticks, including retained ride identities where appropriate. Invalid ranges, unsupported owners, duplicate/default records and inconsistent timer/runtime combinations are rejected. Default records are omitted.

Explicit readers for versions 1–16 preserve original fields and initialize no-minimum policies with zero elapsed extra wait. Version-16 operating settings retain their variable station/cycle timing; older fixed-cycle validation applies only through version 15. Loading alone does not rewrite old slots. Versions 0.32 and earlier refuse schema 17, so retain older save copies before saving with 0.33. Design/catalog formats remain version 1. No new dependency or external asset is added.

## Verification

| Check | Current result |
| --- | --- |
| `cargo check --release --workspace --locked` | Passed on the integrated source |
| Release tests | Passed: **674 tests (403 game / 271 simulation)**; receipt `tests-delivery.log` |
| Formatting, strict release Clippy and production build | Passed; `clippy-delivery.log`; production build 47.48 seconds |
| Separate coaster-testing and dispatch writer/verifier processes | Passed — four extracted-executable processes |
| Retained access, operations, track-construction and design-transfer proofs | Passed — eight extracted-executable processes |
| Authentic version-16 migration and unchanged-slot refusal by 0.32 | Passed — seven processes; six legacy slots each preserve original fields/source bytes through 3,600 current-rule ticks, and all four new dispatch slots remain unchanged after old-reader refusal |
| Minimum-window capture inspection | Passed — all thirteen 1024 × 680 PNGs directly inspected, readable without clipping or overlap |
| Exact-source Windows CI | Pending |
| Archive/payload/current-guide integrity and fresh relocated runtime checks | Passed — **48 payload hashes / 39 current guides**, twelve proof processes and thirteen captures |
| Dependency/runtime/notices comparison with 0.32 | Passed — **291 dependency entries / five unchanged files** |
| Physical native gestures and clean-PC/offline standard-user acceptance | Not verified |
| Representative hardware performance | Open; this development PC is not a reference |

Use isolated absolute save directories for all proof modes. `write-coaster-testing` / `verify-coaster-testing` check independent tuned/default saved states, deterministic report reproduction, stale-result invalidation and load-time cache clearing without changing source slots. `write-dispatch` / `verify-dispatch` exercise persisted waits, timeout and occupied/paying operation. The receipts below identify the exact source and executable. Controlled images establish rendering, not physical pointer interaction.

The locally verified runtime source is **`b4523bb8eab5365947759a6643241a3da83400a4`** in [draft PR #31](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/31), based on PR #30. The final executable is **49,667,584 bytes**, SHA-256 **`77e4cb7c8f5ce70608e7e434bbe479c0bfe60ae0ff5a66347805c1ec01625532`**. Local receipts are under `artifacts/coaster-testing-dispatch-0.33.0/`, including `tests-delivery.log`, `clippy-delivery.log`, `notices.txt` and `visual-review.txt`. Earlier 670/672-test passes and preliminary packages are superseded by this executable and the 674-test pass.

Fresh extraction to `C:\Users\strij\AppData\Local\Temp\Parkbound 0.33 verified delivery é`, launched from `C:\Windows`, validated all payloads and current guides before running twelve proof processes: testing, dispatch, access, operations, design-transfer and track-construction writer/verifier pairs. All thirteen extracted-executable captures passed direct inspection; `visual-review.txt` records their hashes. The portable archive is `dist/Parkbound-0.33.0-windows-x64.zip`. Refresh guides, manifest and archive after documentation edits while preserving the verified executable, then recheck a fresh extraction. Its adjacent `.zip.sha256` identifies the mutable ZIP.

Seven separate migration/refusal processes passed, recorded in `artifacts/coaster-testing-dispatch-0.33.0/migration-final.log` and `migration-final/migration.txt`. Authentic 0.32 operations primary/backup slots and track-construction main plus editing-session primary/backups cover **six legacy slots**. Each preserves every original field and source byte through **3,600 current-rule ticks**. The 0.32 reader rejects confirmed schema/content **17/17** with its exact `Save version is unsupported` message; all **four new dispatch slots** stay unchanged. Both old/current executable hashes remain unchanged. This is current-rule migration continuity, not a claim that new dispatch rules reproduce an older build's future.

Exact-source push [Windows CI 40110125468](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40110125468) is still pending. Native physical gestures, clean-PC/offline/standard-user acceptance and representative-hardware performance remain open; this PC is not a performance reference.

## Following work

Next priorities include layout-informed guest choice, wider coaster/ride construction, configurable train composition and deeper management systems. Staff, maintenance/breakdowns, research, scenarios, audio, additional ride families and full-game acceptance remain unfinished. Broad checklist entries remain in progress after these bounded milestones.
