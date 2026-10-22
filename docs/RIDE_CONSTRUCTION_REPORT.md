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

## Completed verification

Runtime source **`e387de6032289c526e547df6cf8540581364fa2b`** is pushed in [draft PR #20](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/20), stacked on open #19 / `codex/second-ride-choice`. All **450 release tests (269 game / 181 simulation)**, formatting, strict all-target release Clippy and the production build pass. The eight new simulation regressions cover exact/insufficient funds, immutable quotes, all body cells, occupied protection, refund-once, loss-making rebuilding, archived ownership and strict migration/chronology. Six game regressions cover actual input, finance presentation, fresh-process continuation and capture fixtures. Paid rides cannot claim earnings from before their purchase, including after purchase entries archive.

[Exact-source Windows push CI 39794086171](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39794086171) and [PR CI 39794127684](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39794127684) passed all **26 steps**, including the new construction continuation check. CI reporting smoke is separate from performance acceptance. Later documentation commits retain this runtime identity and have separate CI receipts.

All **82 local production process checks** passed: **64 save/proof processes** and **18 inspected captures**. Authentic schemas 1–10 were written by retained 0.12–0.21 executables. Occupied journey/economy/feedback/service/finance cases, removed/rebuilt v10 rides and authentic v9/v10 archived ledgers preserve every original park field and continue deterministically. Old 0.21 rejects schema11 without changing either slot. Twelve specialized continuation pairs pass alongside the ordinary proof. The new 7,200-tick construction proof checks exact costs, single refunds, retained queues/history, occupied and cash refusal, rebuilding, archive rollover, saved basis and paused backup restoration.

All eighteen production PNGs were inspected across four rotations, both terrain/visibility modes and normal/minimum windows. Nine new construction variants show the $300 quote, $50 shortfall at $250 cash, $150 refund, removed body and distinct Finance categories. Minimum-window Rides shows paid cost and occupied demolition protection; Guests, Services, old Finance pages, Help and overview remain readable. Captures establish presentation separately from native interaction and continuous motion.

The executable is **48,142,336 bytes**, SHA-256 **`a93d8a25269c9ff120c0e4078116745917516611a43946cf6f91a1f459c56856`**. The portable **0.22 package verifies 37 payload hashes and 28 current Markdown guides**. All **291 dependency entries** and five dependency/native/notices/runtime files are byte-identical to 0.21. Fresh spaces/é extraction, launched from `C:\Windows`, passed construction continuation and a byte-identical minimum-window construction Finance capture. Final documentation/manifest/ZIP refreshes preserve the exact executable; the adjacent `.zip.sha256` identifies the archive. Same-PC relocation is not clean-PC acceptance.

Native checks passed **F9 paused load**, **Y/Rides and next-ride button**, **Escape**, **F/Find**, **body selection**, the **Remove button**, **F5 save**, the **Build carousel button and actual placement click**, **N/Finance** and **Alt+F4 exit**. At tick0, cash changed from **$700 → $850 → $550** through demolition and rebuilding. Saved data confirms exactly one $150 refund for Ride#2 and one new $300 purchase for Ride#3, removal of the old body and both gates, retained owned queues/unrelated objects, and unchanged guests/wallets/needs/reviews, coaster/train, RNG, terrain, services and time. The replacement is closed at internal origin (19,24). Backup is the saved demolition state; primary is the replacement. The isolated instance was confirmed closed; default player saves were untouched.

The **Delete-key attempt was inconclusive**, with no visible response; the Remove button passed separately. Initial window capture timed out, then one reselection/retry succeeded before input. Native X-tool demolition, insufficient-cash clicking, occupied refusal and remaining historical keyboard/portal controls are not claimed here; source/process tests cover their respective rules. No local performance batch ran and **this PC remains excluded as a performance reference**. Clean-PC/offline/standard-user compatibility, signing, source licensing, final title and representative hardware remain separate release gates.

Full contract and reproduction are in [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md). Raw receipts are in ignored `artifacts/ride-construction-0.22.0/`: source logs, `runtime-verification.txt`, process logs, capture inspection/hashes, migration/proof directory references, native instance/save verification, notices, relocated execution and CI JSON. `final-handoff.txt` records final documentation HEAD, package identity and clean remote/local equality after push. Both PC paths in the resume guide remain authoritative. Transfer ignored artifacts/dist through OneDrive or deliberately; Git carries source and guides.

## Remaining scope

Native controls require direct interaction evidence separate from static captures and source input tests. Same-PC relocation does not establish clean-PC, offline or standard-user compatibility. Representative hardware, signing, final title and source-license/public-release decisions remain open. The next cohesive construction stage can price service facilities and their demolition, then introduce the minimal station/straight/curve track builder with explicit valid-circuit feedback. Broader park finances and ride families remain later work.
