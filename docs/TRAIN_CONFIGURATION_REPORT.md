# Configurable coaster cars — 0.35.0

Implementation is in progress on `codex/coaster-car-count`, based on verified 0.34.1 checkpoint `fe5125652df71e16e0e73c07ebc8563228d443bd` / PR #33. The final 0.34.1 source `a9217be3e366033a9f04adc0576b64d5f67db01d` has now passed all 35 steps of exact-source Windows CI 40121394869.

## Player behavior

Open **Rides [Y] → Operations**. A coaster supports **one train with 1–3 cars**, each with two seats: total capacity **2, 4 or 6**. The initial setting and imported parks use three cars. The carousel has six seats. Car count changes the visible train, boarding seats, occupancy displays and loading-policy limits. The compact cars fit the existing station.

Close the ride and allow its current trip and passenger exits to finish before changing cars. Removing cars is refused if the configured minimum load would exceed the new capacity; lower the minimum first. Opening, transfers and passenger ownership retain their existing safeguards. Repeating an already-selected setting is a harmless no-op. Changes cost nothing and preserve track, cash, completed trips, fares, reviews and other ride settings. Reset operating defaults restores three cars together with the selected coaster's operating and loading defaults.

The mechanics test becomes stale when car count changes. Its motion remains the existing approximate model; car count does not introduce a changed mass, forces, multiple trains or block operation.

## Saves and verification

Save schema/content is **19/19**. `coaster_cars` is a required bounded integer in current saves. A separate exact version-18 DTO preserves all old fields and initializes three cars; earlier explicit readers also initialize three cars. Current saves reject absent, malformed or out-of-range counts, passengers outside configured seats and minimum loads above current capacity. Historical financial receipt limits retain the old maximum of six seats, so shortening a train does not invalidate earlier revenue.

Source tests cover all three train lengths with two independently operating rides, paid passengers, unique seat ownership, close/stop/empty guards, minimum-load protection, current-save strictness and occupied version-18 migration. UI tests use generated controls and real save/load systems, verify held clicks, selected-ride ownership, dynamic capacity, reset and test-result invalidation. Production `write-trains` / `verify-trains` modes retain a parked one-car backup and paid occupied one-car primary, then replay 7,200 ticks, close and drain it, grow to two cars and reset to three. Source slots are preserved.

Capture scenes are `cars-one`, `cars-two`, `cars-three`, `cars-riding`, `cars-minimum` and `cars-map`. They use fresh ride fixtures before scene construction, actual operating commands and real paid boarding. The minimum-size Rides panel accommodates the additional row.

Verification receipts will be retained in `artifacts/train-configuration-0.35.0/`. Release tests, strict checks, production build, genuine old-save migration, unsupported-new-save refusal by 0.34.1, separate-process proofs, visual checks and relocated portable packaging must finish before this version is marked verified.

Physical mouse/keyboard, clean-PC and representative performance acceptance remain separate. This development PC remains excluded as a reference; no reference PC is selected. The normal map/save caps, all three continuation paths, dependency policy and original-content requirements are preserved.

## Following work

RID-04 remains open for multiple trains and broader operating modes. Further construction, complete closed/test/open workflows, realistic forces, additional ride families, staff, maintenance, research, scenarios, audio and complete management remain unfinished.
