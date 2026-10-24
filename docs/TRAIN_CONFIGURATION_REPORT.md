# Configurable coaster cars — 0.35.0

Version **0.35.0 is locally verified and packaged**, runtime source `8f2dc0454b32c170ed943b1cca36bbffe830cd46` on `codex/coaster-car-count` in [draft PR #34](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/34), based on verified 0.34.1 checkpoint `fe5125652df71e16e0e73c07ebc8563228d443bd` / PR #33. The final 0.34.1 source `a9217be3e366033a9f04adc0576b64d5f67db01d` passed all 35 steps of exact-source Windows CI 40121394869. No milestone PRs have been merged.

## Player behavior

Open **Rides [Y] → Operations**. A coaster supports **one train with 1–3 cars**, each with two seats: total capacity **2, 4 or 6**. The initial setting and imported parks use three cars. The carousel has six seats. Car count changes the visible train, boarding seats, occupancy displays and loading-policy limits. The compact cars fit the existing station.

Close the ride and allow its current trip and passenger exits to finish before changing cars. Removing cars is refused if the configured minimum load would exceed the new capacity; lower the minimum first. Opening, transfers and passenger ownership retain their existing safeguards. Repeating an already-selected setting is a harmless no-op. Changes cost nothing and preserve track, cash, completed trips, fares, reviews and other ride settings. Reset operating defaults restores three cars together with the selected coaster's operating and loading defaults.

The mechanics test becomes stale when car count changes. Its motion remains the existing approximate model; car count does not introduce a changed mass, forces, multiple trains or block operation.

## Saves and verification

Save schema/content is **19/19**. `coaster_cars` is a required bounded integer in current saves. A separate exact version-18 DTO preserves all old fields and initializes three cars; earlier explicit readers also initialize three cars. Current saves reject absent, malformed or out-of-range counts, passengers outside configured seats and minimum loads above current capacity. Historical financial receipt limits retain the old maximum of six seats, so shortening a train does not invalidate earlier revenue.

Source tests cover all three train lengths with two independently operating rides, paid passengers, unique seat ownership, close/stop/empty guards, minimum-load protection, current-save strictness and occupied version-18 migration. UI tests use generated controls and real save/load systems, verify held clicks, selected-ride ownership, dynamic capacity, reset and test-result invalidation. Production `write-trains` / `verify-trains` modes retain a parked one-car backup and paid occupied one-car primary, then replay 7,200 ticks, close and drain it, grow to two cars and reset to three. Source slots are preserved.

Capture scenes are `cars-one`, `cars-two`, `cars-three`, `cars-riding`, `cars-minimum` and `cars-map`. They use fresh ride fixtures before scene construction, actual operating commands and real paid boarding. The minimum-size Rides panel accommodates the additional row.

All **706 release tests (418 game / 288 simulation)**, formatting, strict all-target release Clippy and production build passed. The final executable is **49,849,856 bytes**, SHA-256 **`bf46c1da787c71b5dcbdae5e2778a4c9daef196429ccc00fa1f3083dca6b406f`**. Production build time was 67 seconds on this development PC. Receipts are `tests-final.log`, `clippy.log` and `build.log` under `artifacts/train-configuration-0.35.0/`.

Fresh extraction to `C:\Users\strij\AppData\Local\Temp\Parkbound 0.35 car count delivery é`, launched from `C:\Windows`, passed **51 payload hashes / 42 current guides**, sixteen separate proof processes (trains, operations, dispatch, appeal, design-transfer, track-construction, access and coaster-testing writer/verifier pairs) and eleven captured scenes. The six car scenes above, `operations-carousel`, `testing-complete`, `exit-path-prepared`, `library-gallery` and `help` were directly inspected at 1024 × 680. Settings, capacity, paid occupancy and minimum-load refusal text fit the available controls. The gallery retains a known midword caption wrap without clipping. `package-verification.log` and `visual-review.txt` retain the receipts.

Eight further processes generated genuine **schema-18** dispatch/paid and track/draft saves using the verified 0.34.1 executable, migrated all **eight original primary/backup slots** with original fields and bytes preserved, and replayed 3,600 current ticks per slot. An actual 0.34.1 reader refused new schema-19 slots without changing them. Both executable hashes remained unchanged. `migration.log` and its named `migration-*` directory retain these results. The 291 dependency entries and all five runtime/notice files match 0.34.1 byte for byte (`notices.txt`).

Exact-source Windows [CI run 40139500981](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40139500981), job 121008034529, is still running at this local delivery checkpoint. No CI artifact was downloaded or executed. The delivered executable is the independently verified local build.

The download is `dist/Parkbound-0.35.0-windows-x64.zip`; its adjacent `.zip.sha256` identifies the current archive. A final guide refresh preserves the verified executable and checks all payloads/current guides in another fresh extraction (`package-final-verification.log`). Older packages remain available.

Physical mouse/keyboard, clean-PC and representative performance acceptance remain separate. This development PC remains excluded as a reference; no reference PC is selected. The normal map/save caps, all three continuation paths, dependency policy and original-content requirements are preserved.

## Following work

RID-04 remains open for multiple trains and broader operating modes. Further construction, complete closed/test/open workflows, realistic forces, additional ride families, staff, maintenance, research, scenarios, audio and complete management remain unfinished.

The next bounded construction slice is an original starter-design pack: a small flat oval, a compact dogleg and a modest lift/drop loop. Use the existing version-1 design import, thumbnail, transform, quote and Apply workflow. Include the files in the portable package, document footprints and construction/access steps, and verify all rotations, real paid replacement, import/export identity and save continuation. Finish the car-count checkpoint before beginning that content stage.
