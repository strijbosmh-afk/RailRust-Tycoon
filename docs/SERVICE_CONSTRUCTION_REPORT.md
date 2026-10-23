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

Runtime source **`c22cc214a04451d5af7b5f5ae77568f5966f12b6`** is pushed in [draft PR #21](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/21), stacked on open PR #20 / `codex/ride-construction-economy`. All **467 release tests (276 game / 191 simulation)**, formatting, final strict all-target release Clippy and the production Windows build pass. Ten new simulation regressions and seven game regressions cover fixed prices, exact/stale budgets, immutable previews, actual commands, cancellation, saved provenance, corrupt schemas, archives, capture state and deterministic continuation. Independent reviews found no remaining actionable accounting or layout defect.

Exact-source Windows [push CI 39942277276](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39942277276) and [PR CI 39942317453](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39942317453) passed all **27 steps**, including all 467 tests, packaging and the new facility construction continuation check. Later documentation commits retain this runtime identity and have separate CI receipts. CI reporting smoke is not performance acceptance.

All **87 local production checks** pass: **71 save/proof processes** and **16 visually inspected capture processes**. Authentic versions 1–11 were written by retained 0.12–0.22 executables with verified hashes. Occupied journey, economy, feedback, service, finance and ride states preserve every original field; authentic v9/v11 archived finances, v10 rebuilt-ride history and v11 paid-carousel/refund archives also continue. Old 0.22 rejects schema12 without changing either slot. Thirteen specialized writer/verifier pairs and ordinary continuation pass. The new **12,000-tick facility proof** covers all three paid kinds alongside a synthetic free legacy facility, unfinished-use cancellation with unchanged positions/wallets/needs, replacement IDs, actual service reuse, real upkeep archive rollover, refund ownership, repeated loss-making build/removal, insufficient funds, strict round-trips and paused primary/backup restoration. Synthetic legacy setup is separately labelled; authentic migration uses old binaries.

All sixteen PNGs were inspected across four rotations, Surface/Underground, Flat/Terraced and normal/minimum windows. Nine new variants show price, red/green placement, $40 shortfall at $40 cash, $40 refund, removed geometry and facility Finance. The $380 construction/$40 refunds/$660 cash example has five signed owner rows. Retained Services clearly separates operating profit from net cash; old carousel Finance, dual rides, archived pages, Rides and Help remain readable. Captures establish rendered states, not native input or continuous-motion acceptance.

The executable is **48,275,968 bytes**, SHA-256 **`8034396ac7c8edc6c0466c76161bb3a955d42102cf8c83aca1200bcdd65a3161`**. The portable 0.23 package verifies **38 payload hashes and 29 current Markdown guides**. All **291 dependency entries**, dependency/native manifests, third-party/Rust notices and the required MSVC runtime DLL are unchanged from 0.22. A fresh spaces/é extraction launched from `C:\Windows` passes facility continuation and a byte-identical minimum-window Finance capture. Final guide/manifest/ZIP refreshes preserve this exact executable; the adjacent `.zip.sha256` identifies the final archive. Same-PC relocation is not clean-machine acceptance.

Native checks passed **F9 paused load**, the **Services button**, **next-facility buttons**, free legacy and paid Food/Drinks cost/refund labels, **Escape**, actual **Drink#3 body selection**, **Remove**, **F5 save**, **Build Drinks** and an actual replacement placement, the **Finance button**, and **Alt+F4 exit**. Cash changed **$700 → $740 → $660 at tick0**. Independent primary/backup comparisons prove one $40 refund for Drink#3, one $80 purchase for fresh Drink#5 at internal (15,21), height8, unchanged guests/wallets/needs/reviews, coaster/train, RNG, terrain, unrelated facilities and public paths. The original ledger prefix remains; backup records demolition and primary records replacement. The isolated process and window were confirmed closed; default player saves were untouched.

The **M shortcut attempt was inconclusive** (no visible response); the Services button passed separately. Native X/Delete, insufficient-cash placement, removal during active use and placement of every facility kind are not claimed; source/process tests cover those rules. An initial capture showed unrelated pixels, so no input used that observation; fresh selection/activation recovered the correct game before any interaction. No unrelated content was used as project evidence. Clean-machine/offline/standard-user compatibility, signing, licensing, final title and representative hardware remain separate gates. **This PC is excluded as a performance reference; no local performance batch ran.**

Raw evidence is in ignored `artifacts/service-construction-0.23.0/`: source logs, `runtime-verification.txt`, migration/proof directory references, capture inspection/hashes, native instance/save verification, notice comparison, relocated execution and CI JSON. `final-handoff.txt` records final documentation HEAD, package identity and remote/local equality after push. Git carries source and guides; ignored builds/raw evidence need OneDrive or deliberate transfer. Both local PC paths in the resume guide are preserved.

## Remaining scope

The carousel retains its $300 purchase/$150 paid refund. Paths, gates, terrain and the fixed coaster keep their existing construction rules. General track construction, depreciation, ride operating costs, wages, stock, energy/nausea/resting, loans, admission and scenario finances remain later work. The next cohesive stage is a minimal station/straight/curve track builder with endpoint and circuit validation. Broad management criteria remain in progress.

Clean-machine offline/standard-user compatibility, signing, source licensing, final title and representative performance hardware remain separate release gates. No local performance batch or performance acceptance is claimed.
