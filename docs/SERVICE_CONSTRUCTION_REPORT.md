# Service construction costs and demolition refunds — 0.23.0

Updated 23 October 2026. Work is on `codex/service-construction-economy`, based on completed 0.22 checkpoint `f167b352cd741638a758608f121f967562e1f590` / open [PR #20](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/20). Parkbound remains an original, staged Rust theme park game for free Windows distribution. This PC is excluded as a performance reference.

## Player contract

| Facility | Purchase | Refund after paid purchase |
| --- | ---: | ---: |
| Food stall | $120.00 | $60.00 |
| Drink stall | $80.00 | $40.00 |
| Toilets | $100.00 | $50.00 |

The immutable quote validates footprint, capacity and park cash. A failed command leaves cash, ledger, IDs, guests, random state and world unchanged. A successful purchase debits exactly once, retains its actual paid basis and records the new ServiceId. Removing the active object refunds half its paid basis once and retains its financial record. Rebuilding allocates a fresh ID and loses half the purchase amount, preventing a cash-generating loop.

Demolition keeps the existing service policy: cancel unfinished visits without a customer payment or need relief. Guests remain on their public paths. Completed sales and operating charges remain earned/incurred. Pausing stops simulation time and recurring charges, while intentional construction/removal commands still record cash changes at the current tick.

Controls show purchase costs, available cash, shortfall and refund before commitment. Services identifies actual paid cost and refund. Finance distinguishes facility purchases/refunds from ride purchases/refunds and customer sales, including archived history. Net cash includes capital transactions; operating profit excludes them.

## Persistence and ownership

Strict schema 12/content 12 requires each Service's `construction_cost_cents` and `demolition_refund_cents`, new `ServiceConstruction`/`ServiceDemolitionRefund` finance kinds and per-service capital totals. Global construction/refund totals include both rides and services. Explicit versions 1–11 migration preserves all original fields and gives free old facilities zero basis/refund. Version 11's existing paid carousel history remains intact. Older readers reject version 12 without rewriting either slot.

Validation reconciles actual paid basis, active/removed objects, one purchase/refund per source, opening and archived totals, chronological entries and exact cash. New fields or transaction kinds cannot be smuggled into old schemas. Paid facilities cannot claim earnings from before their purchase. Current map/save/ledger limits are unchanged.

## Reproduction

Use the pinned Rust 1.93.1 MSVC toolchain and existing lockfile. Follow [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md), including both PC paths, source checks, separate save folders and safe shared-OneDrive handoff.

Build the production executable, then run `--save-check write-facility-construction` and a separate `--save-check verify-facility-construction` process with the same new absolute `--save-dir`. Retain the writer's directory, primary, backup, expected-state and verification files. These processes exercise all prices, actual unfinished service use, removal/refund, replacement IDs, archive rollover, insufficient-cash refusal and deterministic continuation.

Capture scenes are `facility-price`, `facility-refund`, `facility-unaffordable`, `facility-finance` and `facility-removed`. Verify minimum-window readability, all rotations, both views and unchanged ordinary operation. Captures do not establish native input acceptance.

## Verification

All 467 release tests (276 game / 191 simulation), formatting, strict all-target release Clippy and the production Windows build pass. Production migration/continuation, visual, package and Git verification are in progress. Raw evidence is retained under ignored `artifacts/service-construction-0.23.0/`; this section will be finalized before handoff.

## Remaining scope

The carousel retains its $300 purchase/$150 paid refund. Paths, gates, terrain and the fixed coaster keep their existing construction rules. General track construction, depreciation, ride operating costs, wages, stock, energy/nausea/resting, loans, admission and scenario finances remain later work. The next cohesive stage is a minimal station/straight/curve track builder with endpoint and circuit validation. Broad management criteria remain in progress.

Clean-machine offline/standard-user compatibility, signing, source licensing, final title and representative performance hardware remain separate release gates. No local performance batch or performance acceptance is claimed.
