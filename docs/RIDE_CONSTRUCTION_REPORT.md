# Ride construction costs and demolition refunds — 0.22.0

Date: 22 October 2026. This stage adds purchase costs and refunds to the buildable Meadow Carousel. It builds on the original two-ride simulation in [SECOND_RIDE_REPORT.md](SECOND_RIDE_REPORT.md). The current branch is `codex/ride-construction-economy`, based on `codex/second-ride-choice` checkpoint `489bfff52c3ab44242c157d04d84d9d36e0906bb`; draft PR #19 was open and unmerged at stage start. The full game and broader MGT-01/MGT-02 criteria remain in progress.

## Player contract

- A complete three-by-three carousel costs **$300.00**. Previewing or selecting the tool costs nothing. A successful deliberate placement deducts the price once, allocates the complete body and stable ride identity, and records one ride-attributed construction transaction.
- An empty carousel refunds **50% of its actual paid construction cost**, currently **$150.00**. The demolition preview and selected-ride details show the amount before commitment. Refunds are paid only when removal succeeds.
- Demolition removes the complete body and its entrance/exit, while retaining queue paths and the removed ride's trip, ticket and construction history. A replacement is a new purchase with a new RideId. Buying and demolishing reduces cash by $150; repetition cannot generate money.
- An occupied ride or active transfer prevents demolition. A blocked footprint, unavailable record/object capacity or insufficient cash prevents construction. Rejected commands preserve the entire park, including cash, ledger, geometry, identity allocators, guests and random state.
- Pausing stops simulation time; deliberate construction and demolition still work while paused and are recorded at the current tick. No charge occurs merely for displaying a tool or panel.
- The fixed demonstration coaster, paths, gates, terrain and services retain their existing construction rules. This stage prices the buildable carousel body. It does not introduce track construction, service purchase costs, ride upkeep, depreciation, loans or general construction economics.

## Money and history

All money uses integer cents. Construction and demolition are separate ledger categories from ticket revenue and guest spending. Lifetime park cash reconciles as:

```text
100000 + ride ticket revenue + service sales + demolition refunds
       - service upkeep - service supplies - ride construction
```

Guest wallet conservation and paid-departure accounting remain unchanged. Finance shows construction expense and demolition refunds separately, includes construction in total costs, and includes refunds in the overall net result. Refunding a body never rewrites its original purchase or changes historical guest spending. The bounded 128-row recent ledger retains older category and ride totals when rows roll into its archive.

`CAROUSEL_BUILD_COST_CENTS` is 30000 and `CAROUSEL_REFUND_PERCENT` is 50. `carousel_build_quote` and `carousel_demolition_quote` are immutable validated quotes; placement/removal revalidate at commitment. `CarouselRide` retains `construction_cost_cents` and `demolition_refund_cents`, including after removal. `FinanceKind::RideConstruction` and `RideDemolitionRefund` carry the same stable ride source as the retained record. Finance totals and per-ride archive totals retain construction and refund fields independently of ticket revenue.

## Save compatibility

New saves use strict **schema 11/content 11**. Explicit versions 1–10 migration preserves every original park field, historical cash and transaction order, amounts, balances and sources. Migrated carousels receive a **zero paid basis and zero refund**, since no purchase occurred under the old rules. Migration never charges the player retrospectively or grants a refund windfall. A newly purchased replacement gets the normal paid basis. A paid ride cannot claim legacy opening earnings; its construction must precede its receipts and refund, including when purchase entries have already archived.

New fields are required in current saves; malformed amounts, inconsistent refund/body state, incorrect ownership or ledger totals and mixed schema fields reject before live state changes. Loading alone leaves both save slots unchanged. The previous 0.21 reader must reject schema 11 without altering either file. Keep closed-game copies for rollback. Existing map, object, guest, ride-record, byte and ledger caps remain unchanged.

## Reproduction and verification

Read [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md) for both exact PC paths, pinned toolchain, build commands and safe shared-OneDrive continuation. Open Y/Rides, choose Build carousel, set height 8 and hover a clear three-by-three area. The retained example origin is internal (19,22); inspector coordinates add one. Check the price before clicking. Select the empty body or use X/Demolish to inspect its refund before removal. N/Finance shows both attributed transactions.

The new production proof uses `--save-check write-construction` and `verify-construction` with a fresh absolute `--save-dir`. Its paid-body backup starts at tick 0 with $700 cash; its primary contains two concurrently paid rides. A 7,200-tick replay closes and unloads the rides, verifies occupied refusal and body/gate removal, retains queues and history, rebuilds a fresh identity, archives the original purchase/refund through real service upkeep and repeatedly buys/removes until cash refusal. Exact independent transaction amounts, sources, balances and unchanged guest wallets are checked at commands; saved paid basis and exact future/backup equality are verified through the production save commands. Retained dual-ride and service-finance proofs check guest payments and operating costs alongside the new categories. Authentic old executables create migration fixtures; preserved original fields are checked independently of the migration implementation.

Controlled scenes are `construction-price`, `construction-refund`, `construction-unaffordable`, `construction-finance` and `construction-removed`. They prepare actual production state and assert that showing a preview does not change money. Static images establish presentation; they do not establish native input or frame-rate acceptance.

All **450 release tests (269 game / 181 simulation)**, formatting and strict all-target release Clippy pass. Tests cover exact and insufficient cash, immutable quotes, failed-command state/allocator preservation, all nine body cells, occupied transfer rejection, repeated-refund prevention, negative-net rebuilding, archive rollover, corruption rejection, zero-basis migration and actual game input. A regression specifically rejects invented opening earnings for a paid ride after its purchase has archived. Production build, runtime, visual, packaging and GitHub evidence follow after completion. Raw receipts live in ignored `artifacts/ride-construction-0.22.0/`; source and this guide are tracked in Git. No local performance batch is required. **This PC is excluded as a performance reference**, and functional timing is not hardware acceptance.

## Remaining scope

Native controls require direct interaction evidence separate from static captures and source input tests. Same-PC relocation does not establish clean-PC, offline or standard-user compatibility. Representative hardware, signing, final title and source-license/public-release decisions remain open. The next cohesive construction stage can price service facilities and their demolition, then introduce the minimal station/straight/curve track builder with explicit valid-circuit feedback. Broader park finances and ride families remain later work.
