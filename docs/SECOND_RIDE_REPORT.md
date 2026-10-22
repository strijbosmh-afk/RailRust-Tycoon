# Buildable second ride and guest ride choice — 0.21.0

This stage adds the original Meadow Carousel, independent ride operation and guest choice. Work is in [draft PR #19](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/19) on `codex/second-ride-choice`, based on completed 0.20 checkpoint `ba9ccb555660677d14a1704aeacdd503f5977aa2`, stacked on `codex/service-finances` / [PR #18](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/18), verified open and unmerged at stage start. Do not assume `main` contains these dependent stages.

Parkbound remains an original Rust game with Windows-only free distribution intended. This stage adds procedural geometry and original code without dependencies or imported artwork. No performance reference PC has been selected; the current development PC is explicitly excluded.

## Building and managing the carousel

Open **Rides / Y**, choose **Build carousel**, and place the highlighted three-by-three footprint at the chosen height. All nine cells must fit within the map and clear terrain and other objects. The whole command succeeds or fails together; a rejected footprint consumes no identity or money. The circular ride has no facing. The existing station circuit remains Meadow Flyer.

The carousel needs its own owned queue, entrance and public exit. Select the intended ride in Rides, then close with Use ride/Y/Escape; J/Queue, I/Gate, K/Exit, F/Find, O/Open and F6/F7 ticket controls target that selection. Gates face toward a perimeter tile of the carousel. C rotates gates and paths as before. Opening/closing the Rides panel suppresses underlying world edits and save/load; loading resets transient ride selection and pauses the park.

There may be one active carousel alongside the original coaster. The save retains up to eight lifetime carousel records, including removed rides, so past receipts and reviews stay attributable. A removed ride does not appear in the active selection list. Removing an empty carousel removes its nine body objects and both gates atomically. Its queue remains for returning guests and deliberate removal. Active passengers or transfers protect the ride from demolition. Rebuilding allocates a new RideId and starts fresh operating counters while retaining the old history. Construction charges and demolition refunds are outside this stage.

The carousel has six independent seats. Boarding and unloading each take 60 ticks, station dwell takes 90 ticks, and an operating cycle takes 720 ticks (24 simulation seconds at 30 ticks/second). Occupants use authoritative sampled moving seats. Closing cancels waiting admissions but lets an occupied cycle finish. A disconnected exit holds completed passengers until repaired. Pause freezes both rides, transfers, needs and costs; faster speeds execute complete identical ticks.

### Reproducible layout

Use the coaster queue/exit fixture in the resume guide. The following second-ride pieces fit the fresh demonstration at height 8. Coordinates below are internal zero-based; the in-game inspector adds one to each coordinate.

| Piece | Tile(s) | Shape or facing |
| --- | --- | --- |
| Carousel northwest origin | (19,22), covering x19..21/y22..24 | Three-by-three body |
| Owned queue | (17,21), (17,22), (17,23) | Flat; owned by carousel |
| Entrance | (18,23) | East, toward body (19,23) |
| Public exit extension | (20,20) | Flat; joins existing (19,20) |
| Exit | (20,21) | South, toward body (20,22) |

Set each ride to $1.00, open both and resume. Guests can use separate queues and ride concurrently. The carousel fixture has three queue positions and six seats; seat numbers are local to each ride.

## Guest choice and owner-specific behavior

Service visits retain priority. Guests already approaching, waiting, boarding, riding or leaving a ride keep their chosen owner; a price or selection change does not silently retarget them. On a new choice, candidates must be reachable, open and otherwise eligible. Affordable and acceptable-value candidates are preferred. The lower score wins:

```text
10 × public route edges
+ 10 × current queue members
+ ticket price in cents / 25 (integer division)
+ 80 if the ride kind differs from this guest's preference
+ 20 × this guest's completed visits to that ride
```

Equal scores use stable RideId order. Preferred Coaster/Carousel kind derives from the park seed and GuestId without consuming random numbers. Read-only inspection also consumes none. Existing price and queue comfort rules remain; a cheaper reachable alternative can be chosen without falsely charging or reviewing the other ride.

Queues, admission tickets, patience, transfers, seats, departure flags, completions and reviews are filtered by RideId. Ticket prices lock at boarding; actual departure charges once. Returning to a public path after a completed cycle earns one review for that ride. Zero-price travel is valid. The guest inspector shows the active ride and preferred kind; Rides shows selected-ride access, passengers, price, income and reviews. Finance ride-income rows identify their ride source.

Every guest retains bounded per-ride spending, completed-trip and review totals alongside existing aggregate counters. Each ride's totals must equal the sums of its guest records. Park cash still equals opening cash plus all ride/service income minus service costs. Departure income is grouped in stable RideId order after both runtimes have updated passenger departure flags. Service cost and completion ordering remains unchanged.

## Persistence and compatibility

Version 0.21 writes strict **schema 10/content 10**. It requires carousel runtime/footprints, guest ride histories and ride sources for finance opening, archived and recent records. Validation rejects partial footprints, duplicate/unknown owners or seats, impossible runtime/passenger states, inconsistent receipts/reviews, over-cap records and current/legacy hybrids before replacing live state.

Explicit versions 1–9 migration preserves every original park field. Existing guest spending/completions/reviews belong to the original coaster. Version 9 ledger IDs, ticks, order, amounts, balances, costs, archives and opening history survive unchanged; added attribution assigns historical ride income to that coaster. Versions 1–8 retain the prior opening-baseline migration without inventing past expenses. Migration adds no carousel and draws no new randomness. Future behavior follows current rules. Loading does not rewrite either slot; older 0.20 readers reject schema 10. Preserve closed-game legacy copies for rollback.

Normal maps remain 32 × 32. Existing byte, object, path, guest, route, circuit, service and recent-ledger caps are unchanged. The eight-carousel-record bound is additional, not an increase in normal map capacity.

## Verification and handoff

Runtime source **`b70edd79e3ab25ed4aa8f705d5a63f0dfcc438a5`** is pushed in [draft PR #19](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/19), stacked on #18. All **436 release tests (263 game / 173 simulation)**, formatting, strict all-target release Clippy and production build pass. Tests cover atomic full-footprint placement, independent queues/seats/transfers, real departures and reviews, closure/exit repair, removal/rebuilding, deterministic choice, pause/speed equivalence, owner-aware input/rendering and strict save corruption/migration. Runtime validation accounts for both running time and each cycle's initial dwell.

[Exact-source Windows push CI 39777803850](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39777803850) and [PR CI 39777812491](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39777812491) passed all **25 steps**, including packaging and the new fresh-process proof. The CI reporting smoke is not hardware acceptance. Later guide commits retain this runtime identity and have their own CI status.

The executable is **47,982,080 bytes**, SHA-256 **`9ae2649d227cc7fc6c13560825c1cdfb6da079c3e60aad9821e6488399d103ab`**. The portable package verifies **36 payload hashes and 27 current Markdown guides**. All **291 dependency entries** and five dependency/native/notices/runtime files are byte-identical to 0.20. A fresh spaces/é extraction launched from `C:\Windows` passed the 24,000-tick rides proof. Final guide/manifest/ZIP refreshes preserve this executable; the adjacent ZIP sidecar identifies the archive without self-referential hashes in bundled guides. Same-PC relocation does not establish clean-machine compatibility.

All **72 local production processes** passed: **54 save/proof processes** plus **18 inspected captures**. Authentic schema1–9 files were written by retained 0.12–0.20 executables. Occupied journey/economy/feedback/service/finance cases and an authentic v9 archived ride/service ledger preserve every original field and continue deterministically. Version0.20 rejects schema10 without altering either slot. All eleven retained/new continuation pairs pass. The new 24,000-tick oracle independently checks each owner's actual departure, locked fare, public-return review formula, history deltas and ledger source/balance; it also verifies price-sensitive alternative choices, closure and fresh-ID rebuilding.

All eighteen PNGs were visually inspected: fourteen second-ride variants across four rotations, both terrain/visibility modes and normal/minimum windows, plus retained Services, oldest Finance page, Help and overview. Rides, owner-aware Guests and Finance fit 1024×680 without clipping. The relocated package produced a byte-identical minimum-window Rides capture. Still images establish presentation separately from native gestures and continuous motion.

Native checks used the exact extracted executable and isolated copied slots. **F9** loaded both occupied rides paused at tick **1,977 / 01:05**, with **$1,009 cash**, four coaster passengers and five carousel passengers. **Y** opened Rides; the **next-ride button** selected Carousel #2; **Escape** returned to the park; **F7** changed only its fare from $1.00 to $1.50; **O** closed that carousel while retaining its paid cycle; **F5** saved. Comparing saved data proved unchanged time, coaster/train state, finances, geometry, RNG/allocators, all guest wallets/spending/history/reviews, and ten unaffected passenger/other-ride guest records. Backup bytes exactly match the original primary. **Alt+F4** closed the isolated instance; default player saves were untouched.

The **Down arrow in Rides was inconclusive** (no visible selection change); the button passed separately. F7 while the selector was open correctly left the fare unchanged: close with Use ride/Y/Escape before operating the selected ride. Native construction/demolition, remaining price gestures, direct continuous carousel motion and earlier unverified native controls remain open. Source tests and production proofs retain their distinct scope.

No local performance batch was run. This PC remains excluded from performance acceptance. Clean-PC/offline/standard-user compatibility, signing, source licensing, final naming and representative-hardware acceptance remain separate release gates. `final-handoff.txt` records the final documentation HEAD, archive identity and clean remote/local equality after push; the adjacent ZIP sidecar identifies the final archive.

The new isolated production commands are `--save-check write-rides` and `--save-check verify-rides`, using a fresh absolute `--save-dir`. They save real mid-boarding and concurrent paid-running states and replay 24,000 ticks with owner-specific closure, price changes, choices, removal and rebuilding. An independent oracle checks actual departure payments, locked fares, guest/ride receipts, ledger transaction sources and balances, unique seats and exact continuation. Retained proofs continue to cover terrain, paths, services and the finance ledger.

New capture scenes are `carousel-valid`, `carousel-blocked`, `carousel-built`, `dual-boarding`, `dual-riding`, `ride-choice`, `rides`, `ride-closed`, `ride-finance` and `carousel-rebuilt`. They use real simulation states and preserve all rotation/view/minimum-window options. Captures establish rendered presentation; they do not establish actual keyboard/mouse gestures or continuous motion.

Raw output belongs in ignored `artifacts/second-ride-choice-0.21.0/`. The portable target is `dist/Parkbound-0.21.0-windows-x64.zip`, with adjacent `.zip.sha256`. The resume guide retains both exact PC paths. Git carries source and guides; raw artifacts and binaries require OneDrive or deliberate transfer. Same-PC relocation is separate from clean-machine compatibility.

## Next cohesive stage

Add construction charges and demolition refunds for the bounded ride/facility construction tools, with clear previews, insufficient-cash rejection and attributed finance entries, before broadening track editing. General coaster design, multiple active carousels, ride running costs, stock, wages, loans, park admission and scenario finances remain future work. Finite guest wallets still are not replenished. The second ride does not complete the full construction or management acceptance criteria.
