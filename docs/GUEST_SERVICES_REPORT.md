# Guest needs and services — 0.19.0

This milestone adds a complete, bounded service loop to the classic isometric park simulation: growing hunger, thirst and toilet need; player-built food stalls, drink stalls and toilets; public-path access, waiting, service, payment and relief. The working title remains Parkbound. Original code, procedural buildings and text are used throughout; no dependency or imported asset is added.

Work starts on `codex/guest-services` from completed 0.18 checkpoint `2414a6ba0647c45261ff2f63d09d52753563dcbb`, stacked on `codex/guest-experience` / [PR #16](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/16). Do not assume the preceding branch is merged or that `main` contains these stages. Exact final source, verification and package identity are recorded below after checks finish.

## Player demonstration

Open Services with **M** or its Build-panel button. Choose Food, Drink or Toilet, then place one tile at the selected height. **C** rotates the entrance facing, which points outward toward an adjacent public path. Page Up/Down changes building height within 0–29; the building reserves three height steps. Its entrance must meet the public path's facing edge at the same height. A matching ramp endpoint works; queues are not service access paths. Disconnected buildings are valid construction and show their access problem.

For a reproducible fresh-park demonstration, use height 8 and North facing at internal zero-based tiles Food `(10,21)`, Drink `(15,21)`, Toilet `(18,21)`. The corresponding one-based tiles are `(11,22)`, `(16,22)`, `(19,22)`. These face the existing public loop along y=20. No ride entrance or exit is needed to use services. Ordinary fresh startup has no facilities, queue or ride portals; the player builds them.

Reopen Services to browse active facilities, inspect access and customer counts, change the price in $0.50 steps, or open/close the selected facility. Up/Down browses; F6/F7 changes its price; O toggles that facility; Space pauses. Help, Guests and Services are mutually exclusive and consume opening/closing input so commands cannot reach the world behind them. Save/load is available after closing the modal. X or Delete demolishes a visible service building through the existing tools.

L/Guests displays each guest's hunger, thirst, toilet need, service activity, spending and completed uses alongside existing preferences, thoughts and ride reviews. Needs and service progress stop when simulation is paused. There is no fast-forward based on wall-clock time.

## Deterministic rules

- Initial needs are independently derived from crowd seed and stable GuestId in the range 10–30, without consuming the crowd RNG. Hunger grows by one every 90 complete simulation ticks, thirst every 60, and toilet need every 120, capped at 100. Needs advance before service/admission/journey review processing, including for passengers.
- A need at 60 can motivate a service visit. Choose the highest qualifying need with a reachable, open, affordable service; ties use Food, Drink, Toilet order, route length and stable ServiceId. An existing ride visit finishes before service seeking. Service and ride visits are mutually exclusive.
- Crossing 85 costs five satisfaction points once per need per crossing. An already-critical loaded value does not manufacture a new crossing. Unmet needs continue to grow; no affordable/reachable facility produces a thought and retry cooldown while ordinary walking and rides remain possible.
- Guests physically walk over public paths. They remain on the public access tile while waiting/using; there is no indoor movement or ordered spatial service queue. Each facility serves one guest for 90 ticks. Arrived waiters are considered in stable GuestId order; waiting times out after 900 ticks and retries use a 300-tick cooldown.
- The price is checked again and locked when Using starts. Approaching/waiting guests have no locked charge. Only successful completion transfers money once, resets the relevant need to 20, adds five satisfaction points up to 100, and records a completed use. Later price edits leave a current user's quote intact.
- Closing, removing or disconnecting a facility cancels unfinished visits immediately without payment or relief. Existing valid public walking edges finish normally. Repairs use fresh object/service identities; no stale active references survive a command.

Default prices are Food $2.00, Drink $1.00 and Toilet free; all three allow exact integer-cent prices from $0–$100 through the simulation API. UI steps are $0.50. Guests have the existing finite $20 wallet; there is no replenishment, supply stock, operating expense, construction cost/refund or income other than rides/services in this stage.

## State and accounting

`ServiceId` has its own monotonically increasing allocator. A service references a one-tile `ObjectKind::Service`, with kind, outward facing, price/open state and served/revenue/rejection totals. Demolition clears its object reference but retains the service record and financial history. A park may retain at most 256 records, including demolished facilities; IDs are never reused.

Guests add `GuestNeeds`, an optional `ServiceVisit` (Approaching/Waiting/Using, service and access identities, quote, progress and waited ticks), service cooldown, `service_spent_cents` and `services_used`. Existing `spent_cents` remains total spending. The independent accounting identities are:

```text
guest wallet + guest total spent = 2,000 cents
sum(guest total spent − guest service spent) = ride revenue
sum(guest service spent) = sum(all retained service revenue)
park cash = 100,000 cents + ride revenue + all service revenue
sum(guest services used) = sum(all retained service served)
```

Free services increment completion counts while moving zero money. Active-use validation reserves counter capacity, and strict validation rejects inconsistent totals, illegal phases/quotes, duplicate users, broken route/access references and overflows before replacing live state. Public navigation excludes service building objects; topology is derived from current public geometry rather than saved caches.

## Save format and compatibility

Version 0.19 writes strict **schema 8/content 8**. Needs, service visits/cooldowns, split spending, use counts, retained service records and the service allocator are required. Existing bounds remain 8 MiB, 4,096 objects, 1,024 paths, 256 guests, 2,048 circuit pieces and 65,536 route IDs. The player app still uses 32 × 32 maps.

Explicit versions 1–7 migrations preserve all original fields, IDs, RNG, money, active fares/journeys and existing v7 preferences/satisfaction/thoughts/reviews. They add deterministic initial needs, no services or visits, zero service spending/uses and allocator 1. Earlier experience defaults apply only to versions 1–6; migrated v7 feedback is not reset. No historical service purchase is invented. Old schemas reject new service object/thought variants instead of accepting hybrid data.

Loading alone leaves primary and backup bytes unchanged. Saving writes 8/8. Old 0.18 intentionally rejects 8/8; retain closed-game legacy copies for rollback. Future behavior follows current rules, so legacy checks compare original state and then current-version deterministic futures, not old executable futures.

## Verification and handoff

All 398 release tests pass (244 game / 154 simulation), as do formatting and strict all-target release Clippy. The production release build also passes. Separate process, capture, CI and package checks are still in progress; no result is claimed until its receipt exists. Production service checks use a fresh isolated directory and separate `--save-check write-services` / `verify-services` processes, retaining the other continuation proofs. Service capture scenes are `service-needs`, `service-approaching`, `service-using`, `service-served`, `service-blocked`, and `service-recovered`; controlled captures do not establish native mouse/keyboard acceptance.

Raw receipts belong in ignored `artifacts/guest-services-0.19.0/`; the portable archive target is `dist/Parkbound-0.19.0-windows-x64.zip`. The adjacent `.zip.sha256` identifies the final archive after documentation refreshes. The resume guide retains both exact local PC paths, setup, branch, source map and commands. Raw artifacts/dist require OneDrive or explicit transfer because they are not in Git.

This development PC is explicitly excluded as a performance reference. No local performance batch or hardware acceptance is part of this milestone. Native service gestures, clean-PC/offline/standard-user compatibility, signing, source licensing, public-name selection and representative-hardware acceptance remain separate gates. Food/drink/toilet service is a bounded addition: energy, nausea, resting, litter, merchandise, stock, costs and staff remain incomplete, as do multiple rides and track construction.

## Next cohesive stage

Build service operating costs and a readable finance ledger, with exact money sources/sinks, operating controls and save continuation. Keep the finite-wallet limitations visible. After that, add a buildable second ride and explicit ride choice before general track construction. GST-03 and MGT-02 remain in progress for their broader criteria; this slice does not complete the management game.
