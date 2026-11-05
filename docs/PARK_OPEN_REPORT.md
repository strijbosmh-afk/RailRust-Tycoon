# Park opening stage report — 0.64.0

## Scope

The visitor-flow panel has a separate Open / close park control and explicit park and arrival status. Closure resets the pending arrival interval, marks every active guest for departure and keeps rides and services in progress. Reopening allows opt-in arrivals after a full interval while existing departures continue. The park's ride and service controls, upkeep and finance ledger are independent of park admission. Missing entry paths continue to hold departures under the existing visitor rules.

Strict park save schema/content advances to **34/34**. The new `park_open` field is required in version 34 and forbidden in earlier visitor-state saves. Version 33 and older parks default to open. Validation rejects a closed park containing a guest who is not leaving or a nonzero pending arrival interval. The production `write-park-open` / `verify-park-open` pair saves a closed park carrying a paid passenger mid-ride, loads it paused in another process and compares an exact 3,600-tick continuation without rewriting the primary save. Windows CI runs the pair.

## Acceptance

Local source, package, capture and GitHub evidence is recorded in [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md), section 31. The package is unsigned. This PC is not a performance reference. Scenario permission rules, park admission pricing and a park-wide ride shutdown remain future work.
