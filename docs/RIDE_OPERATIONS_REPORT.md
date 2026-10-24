# Ride operations and design search — 0.31.0

Current branch: `codex/ride-operations-and-design-search`, based on verified 0.30 checkpoint `39d25cdc0dd00ace721f5b3c815021f278283b17` / [PR #28](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/28). The new source implements persistent ride operating settings and transient saved-design search/sort. **Verification is pending.** The latest verified technical delivery remains 0.30; [TRACK_DESIGN_TRANSFER_REPORT.md](TRACK_DESIGN_TRANSFER_REPORT.md) preserves its exact source, executable, CI, runtime, visual and package evidence.

This milestone advances bounded portions of RID-03, RID-04, RID-08, RID-11, FAM-01, MOD-05 and UI-02. Those complete feature families remain in progress. Normal play retains its 32 × 32 map, one coaster, current carousel limits and existing save caps. No development PC is a performance reference; representative-hardware, clean-PC and full-game acceptance remain open.

## Player behavior

Open **Rides [Y]**, select a ride, and use its Open/Close control. Close the ride and wait until it has stopped and all passengers have left before adjusting its operating settings. A paid trip already in progress finishes and unloads normally; closing does not eject its passengers or charge them again. A missing exit that holds passengers also prevents tuning. The panel exposes only settings supported by the selected ride family.

| Setting | Supported ride | Range and step | Original default |
| --- | --- | --- | --- |
| Station wait | Coaster and carousel | 1–30 seconds, step 1 second | 3 seconds |
| Lift speed | Coaster | 0.35–1.41 tiles/s, steps of about 0.12 tiles/s | 0.70 tiles/s |
| Brake speed | Coaster | 0.23–1.17 tiles/s, steps of about 0.12 tiles/s | 0.59 tiles/s |
| Cycle duration | Carousel | 12–60 seconds, step 6 seconds | 24 seconds |

The panel displays equipment speed in tiles per second, rounded to two decimals. It converts fixed simulation distance units per tick by multiplying by 30 and dividing by 1,024; this is not km/h or an inferred physical measurement. The simulation runs at 30 ticks per second. Settings apply to the next relevant wait/cycle and do not reset existing timers. **Reset operating defaults** restores only the selected ride. Changes are free; ticket prices, cash, fare locks, guest ownership, paid construction basis, draft/history, simulation speed and ride identity remain separate. Save the park to persist settings.

Open **Designs [Ctrl+D]** from Track tools. The new search field matches saved names case-insensitively, trims surrounding query whitespace and accepts up to 32 Unicode characters. **Clear** shows all entries. **Sort: Name/Pieces** toggles ascending alphabetical or piece-count order; ties use case-insensitive name, then exact name. Four-card pages follow selection among matching entries. Match counts and an explicit zero-results state explain the current view.

Typing focus belongs to either the design name or search field. Ctrl+A selects the focused field; Backspace deletes a character and Enter/Tab ends typing. Search, sorting and browsing never reorder or write the catalog, edit the draft, change Undo/Redo or alter the park. Filters survive panel close/reopen and reset at application restart. When a result remains visible, its selection and placement controls remain intact. If it disappears, the first matching result becomes selected; no matches clears selection and disables load, export and placement controls.

Save, confirmed rename and import select their resulting saved entry. Search clears only when that entry would otherwise be hidden. Confirmed deletion retains the query and selects the first remaining match. Existing confirmation and pending-transfer barriers disable search/sort actions as well as other library controls. Import/export still uses the strict independent version-1 design format. Loading into the draft remains a free undoable change; Apply alone constructs track and settles money.

## Simulation and save contract

`park_sim::RideOperatingSettings` contains `station_wait_ticks`, `lift_speed`, `brake_speed` and `cycle_ticks`. Exact lift bounds are 12–48 and brake bounds are 8–40 fixed distance units per tick, both in steps of 4; defaults are 24 and 20. Station wait is 30–900 ticks in steps of 30, default 90. Carousel duration is 360–1,800 ticks in steps of 180, default 720. `Park::set_ride_operating_settings` validates range, step and ride-family compatibility before mutation. Actual changes require an available, closed, stationary ride without seated/transfer passengers. Repeating the already-active valid settings returns an unchanged result even while operating. Carousel equipment fields and coaster cycle duration must retain their unsupported-field defaults.

Coaster lift/brake motion uses its configured equipment values; carousel angular presentation scales progress to its configured cycle duration. Passenger accounting, admission/access rules, exact-once departure payment, completion counts, closing/draining behavior and finance reconciliation remain authoritative. Train count/cars, minimum-load policy, full physics/forces, test mode, telemetry and additional operating modes are outside this slice.

New park files use strict **schema 16/content 16**. Nondefault settings are stored as a bounded, sorted, unique list keyed by existing ride IDs, including retained demolished-ride identities where applicable. The maximum remains nine owners: one coaster and eight retained carousel records. Default settings omit the override record. Validation rejects unknown/duplicate/unsorted owners, explicit default records, unsupported fields, wrong steps/ranges and missing/unknown JSON fields. Current historical-cycle bounds use the minimum legal duration because previous cycles may have used other settings.

Exact readers for schema/content 1–15 preserve their original fields and supply an empty override list, restoring original operating defaults. They retain their own historical validation, including the fixed legacy carousel-cycle bound; a schema-15 document containing the newer settings field is rejected. Loading does not rewrite primary or backup files. Saving writes 16/16, which 0.30 and earlier readers cannot load; preserve legacy copies before saving with the new version. Design/catalog formats remain version 1. Search/sort, text focus and draft Undo/Redo are transient and absent from park saves.

## Source and verification scope

The simulation lives in `crates/park_sim/src/ride_operations.rs` with train/carousel and strict save integration. `crates/park_game/src/ride_operations.rs` and `ui/rides.rs` implement selected-ride commands and visible controls. Same-frame browse, modal close/load, hidden panels, diagnostics and captures block unintended edits. The operations production proof uses `save_check/operations.rs`.

Design filtering derives a vector of actual catalog indices in `design_library.rs`; selection never becomes a display-slot index. `ui/designs.rs` maps matching cards, captions, actions and selected borders through that vector. `design_preview.rs` maps the same visible indices into the existing geometry cache, keeping thumbnails aligned after filtering/sorting without rerasterizing unchanged geometry.

Four added real-gallery UI tests exercise Unicode/case-insensitive typing, focus and length limits, zero-result actions/images, Clear, sorted filtered pages, actual catalog-index selection, rendered thumbnail pixels, retained placement, close/reopen state, confirmation/transfer barriers, deletion refresh, renamed/saved result visibility and numbered import selection. Whole-park snapshots, history and catalog bytes establish that browsing does not edit persistent state. These tests are implemented; their execution results are pending.

| Check | Current result |
| --- | --- |
| Formatting, workspace release tests, strict all-target release Clippy | Pending |
| Production Windows release build and exact executable identity | Pending |
| Separate `write-operations` / `verify-operations` production processes | Pending |
| Retained design/transfer, paid-track/history and broader continuation proofs | Pending |
| Authentic schema-15 migration and unchanged-slot rejection by 0.30 | Pending |
| Direct visual inspection and final-source capture comparison | Pending |
| Exact-source Windows CI | Pending |
| Dependency/native notice comparison and portable payload integrity | Pending |
| Fresh outside-checkout spaces/Unicode extraction and runtime/render checks | Pending |
| Physical native controls, file pickers and keyboard gestures | Not verified for 0.31 |
| Offline standard-user clean-PC launch/save and representative performance | Open release gates |

Run proof writer/verifier modes as separate production processes against a fresh absolute `--save-dir`, never the player's save directory. The writer refuses an existing directory. The operations proof saves independent closed-ride settings in backup and both tuned rides carrying paid passengers in primary. Replay checks occupied-edit refusal, closing/unloading, resetting one owner, retuning, reopening and 7,200 ticks with strict save round-trips and paid journeys on both rides. The verifier compares exact post-restart state, independent backup settings, paused load with stale UI controls cleared, and unchanged primary/backup bytes. Its receipt is `verification-operations.txt`. These are the implemented proof assertions; execution remains pending. Preserve exact source and executable identity with the results, and distinguish migration against authentic old writers from transformed current fixtures.

New controlled capture scenes are `operations-coaster`, `operations-carousel`, `operations-running`, `operations-limits`, `library-search`, `library-no-results` and `library-sorted`. Inspect the minimum 1024 × 680 window, disabled running/occupied controls, boundaries, both ride families and filtered/sorted/empty gallery states. Retain the existing library/transfer/editor/Help scenes. A controlled capture disables normal input and does not verify physical mouse/keyboard interaction.

The intended portable output is `dist/Parkbound-0.31.0-windows-x64.zip`. Its production build, payload count, notices, exact binary identity and fresh extraction checks remain pending. After documentation edits, refresh bundled guides, payload manifest and archive, then verify the extracted bytes. The adjacent `.zip.sha256` identifies the archive; do not embed a mutable archive's self-hash in its own bundled report. Same-PC relocation is separate from clean-PC acceptance.

## Following work

First finish the current verification matrix and record scoped results here. Then prioritize station/access construction assistance, train/loading policies, test-mode telemetry and layout-driven ratings. Multiple coasters, broader track/ride families, staff, research, scenarios and full management remain open. Preserve immutable previews, safe operating edits, collision-safe imports, exact saved-snapshot exports and the draft/live-accounting boundary through later changes.
