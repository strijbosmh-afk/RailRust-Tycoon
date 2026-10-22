# Service operating costs and finance ledger — 0.20.0

This stage adds recurring service running costs, supplies costs on completed use, and a readable cash ledger. It builds on 0.19 guest needs and services at `e18e9aa296eafd13957722485061b5cfdf38087f`, on `codex/service-finances`, stacked on `codex/guest-services` / [PR #17](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/17). The predecessor was verified open and unmerged at stage start. Do not assume `main` contains these stages.

The working title remains Parkbound. This is original Rust code, interface text and procedural content; no dependency or imported asset is added. Windows-only free distribution remains the intended outcome. The current development PC is explicitly excluded as a performance reference.

A local editing helper unexpectedly triggered Windows' Python install manager, installing Python 3.15.1. It adds no project dependency or runtime requirement; remaining edits used file patches/PowerShell. The other-PC setup remains Rust/MSVC.

## Player behavior

Open **Finance** with its header button or **N**. It shows cash, ride and service income, running costs, supplies, net result and recent transactions. Pages show older retained transactions. Opening balances and earlier totals preserve the explanation of current cash after older transactions leave the recent list. These are simulation-time accounts, not real dates or a monthly accounting calendar.

**Services / M** retains construction, price and open/close controls and adds service costs and profit. Running costs are fixed by service kind; the player controls prices and whether to operate. An open facility costs money even without visitors or a connected path. Closed and demolished facilities incur no new cost. Demolition retains earned revenue and expenses. There is no construction charge or demolition refund in this slice.

| Facility | Running cost per simulation minute | Charge every 90 open ticks | Supplies per completed use |
| --- | --- | --- | --- |
| Food | $6.00 | $0.30 | $0.60 |
| Drinks | $4.00 | $0.20 | $0.25 |
| Toilets | $2.00 | $0.10 | $0.10 |

There are 30 simulation ticks per second. Running-cost progress belongs to each facility, is preserved when closing/reopening or saving/loading, and advances only while open. Pausing stops time, needs, service progress and running costs. Fast speeds execute the same complete ticks. A free service still consumes supplies on successful completion.

At a running-cost deadline, insufficient park cash closes the facility and cancels unfinished visits without charging guests or relieving needs. It keeps its due progress so a reopen retries on the next tick. If park cash plus a user's locked payment cannot cover supplies, that use likewise cancels without guest payment or relief and the facility closes. The player must reopen it after making funds available; it never reopens automatically. Cash cannot become negative, and this stage introduces no debt or loans.

## Accounting and deterministic order

At each complete tick, running costs are considered in stable ServiceId order before needs, service completions and ride departure payments. A successful service records its locked income before its supplies expense; the sale can therefore fund its own supplies. Income still occurs only on ride departure or successful service completion. A later price edit never changes the locked quote of a current user.

Guest money remains conserved independently of park expenses:

```text
guest wallet + guest total spent = 2,000 cents
sum(guest total spent − guest service spent) = ride revenue
sum(guest service spent) = all retained service revenue
park cash = 100,000 cents + ride revenue + service revenue − running costs − supplies
sum(guest completed uses) = all retained service served counts
```

All amounts are integer cents. Each retained service owns running-cost progress, cumulative running and supplies costs and its funds-closure status. Demolished records keep their history and participate in every accounting total.

The ledger retains at most 128 chronological transactions, with stable increasing IDs, simulation tick, category, optional service identity, amount and resulting cash. Categories are ride income, service income, running costs and supplies. Older entries are accumulated into validated category and per-service totals; cumulative history is not discarded when the recent list rolls over. Zero-money uses still increment served counts but create no zero-value income transaction.

## Persistence

Version 0.20 writes strict **schema 9/content 9**. Explicit versions 1–8 migration preserves original state, including v8 needs, service visits, locked quotes, cooldowns, money and demolished-service history. It starts accounting at the loaded tick/cash, retains old revenue and served counts as opening history, and adds zero expenses/progress. It invents no past costs or transactions. Subsequent simulation follows current cost rules.

The strict reader validates ledger balances, chronology, categories, service identities, archived totals, opening history and expense relationships before replacing live state. Primary and backup remain unchanged on load or rejection. Version 0.19 intentionally rejects schema 9; retain closed-game copies of older saves for rollback. Existing size, map, object, guest, route and retained-service caps remain unchanged.

## Verification and handoff

Implementation and verification are in progress. No pass is inferred from the preceding stage. Final runtime identity, source tests, production proofs, visual/native checks, CI and portable-package evidence will be recorded here after checking.

The isolated production commands are `--save-check write-finance` and `--save-check verify-finance`. Raw receipts belong in ignored `artifacts/service-finances-0.20.0/`. The portable target is `dist/Parkbound-0.20.0-windows-x64.zip`; its adjacent `.zip.sha256` identifies the final archive after guide refreshes. Source and Markdown are pushed to GitHub; raw receipts and portable builds require OneDrive or explicit transfer. The resume guide retains both exact PC paths.

No local performance batch or hardware acceptance is part of this gameplay stage. Same-PC relocation, source tests, controlled images and native input checks establish different things and must remain separately scoped. Clean-PC/offline/standard-user compatibility, signing, source licensing, final name and representative-hardware acceptance remain open.

## Next cohesive stage

Add a buildable second ride and explicit guest ride choice, preserving per-ride financial history and deterministic saves before a general track builder. Stock deliveries, configurable operating budgets, wages, loans, construction/refunds, park admission, monthly reporting and scenario finances remain future work. Finite guest wallets are still not replenished. This bounded ledger does not complete MGT-01/MGT-02 or the management game.
