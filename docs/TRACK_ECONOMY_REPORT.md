# Track construction costs and refunds — 0.26.0

Updated 24 October 2026. Work on `codex/track-economy` starts from the 0.25 documentation checkpoint `2343f813d5e79bbac71eebe7eeedc91cc74d58e1` on `codex/sloped-track` / [draft PR #23](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/23). The [sloped track report](SLOPED_TRACK_REPORT.md) retains the separate 0.25 runtime identity, successful CI and package evidence. Those results do not verify this milestone.

Parkbound remains an original, free Windows theme park game developed in playable stages. Normal play remains 32 × 32. No representative performance PC has been selected; development machines remain excluded from performance acceptance.

## Player contract

Planning, erasing a draft tile, clearing a draft and saving a plan are free. Apply purchases the completed track plan only after all geometry, clearance, capacity, train and guest checks pass. The coaster must be closed and stopped with no active visitors. An unaffordable or invalid command leaves the complete authoritative state unchanged, including cash, IDs, live rails, paid basis, guests, ledger and draft.

| Piece | Purchase price |
| --- | ---: |
| Station | $100 |
| Straight | $10 |
| Left or right curve | $15 |
| Chain lift +1 | $25 |
| Drop −1 | $15 |
| Drop −2 | $20 |
| Level brakes | $20 |

Replacement compares exact world specifications: tile, kind, incoming/outgoing edges and heights. Matching installed pieces retain their object IDs and original paid basis. New or changed pieces are charged at the table above. Removed paid pieces refund 50% of their actual purchase price. Original demonstration rails and all track imported from versions 1–14 have zero paid basis and refund $0. A matching free legacy rail remains free when retained. Reapplying an identical circuit is a complete no-op.

The editor shows purchase cost, refund, net payment or receipt, current cash and resulting cash. Quotes are read-only and are refreshed from the current park. Apply computes its own current quote before committing; displaying a quote never reserves money. A guaranteed replacement refund can fund the purchase, and the ledger records the refund before the construction debit. An insufficient-cash notice keeps the plan available for later editing or funding.

Outside the draft editor, Inspect a live rail to see its refund and use Delete or Remove. Existing occupied-ride protection still applies. Removing one owned rail faults the circuit, preserving the remaining rails and their paid basis. Those remnants can be removed separately for their own one-time refunds, including after save/load. A missing rail cannot be refunded again. Faulted remnants are not silently adopted by a new circuit; remove obstructing remnants before rebuilding. Clear draft and draft Erase never demolish live track or generate refunds.

Construction preserves ride identity, portals, guests, earned income, reviews, completed trips and lap history. Relocating a station can still require reconnecting access. Track prices do not change guest wallets. Buy/remove cycles lose half the purchase price and cannot generate money; refunding free legacy track creates neither money nor ledger entries.

## Accounting and persistence

`Park::track_construction_quote()` returns a `TrackConstructionQuote` containing construction/refund amounts, retained/new/removed piece counts and available cash. Its `required_cents()`, `affordable()` and `balance_after()` methods expose the net transaction. `TrackBuildKind::cost_cents()` exposes palette prices. `Park::track_demolition_refund_cents(ObjectId)` exposes an individual rail's paid refund without mutation.

`TrackEconomy` retains cumulative construction/refund totals and sorted paid-piece records containing object ID, exact specification and paid cents. Only positive paid basis is recorded. It survives circuit faults independently of `TrainCircuit`. The invariant is:

```text
lifetime purchases = remaining paid basis + 2 × lifetime refunds
```

Distinct `TrackConstruction` and `TrackRefund` entries belong to the persistent coaster RideId. They feed the existing construction/refund summary, bounded chronological ledger, running balances and owner-attributed archive. Track refunds can precede later ticket income; carousel lifetime-removal rules remain separate. The finance checker reconciles totals against both remaining basis and archived/recent transactions.

Strict schema 15/content 15 requires the track economy state. Explicit migrations accept versions 1–14 and initialize a zero track basis without changing any original saved fields or inventing past charges. Version 13 retains its level-only draft constraints; version 14 retains graded drafts. Earlier versions still receive the established defaults. Loading does not rewrite primary or backup. Version 0.25 rejects schema 15, so preserve closed-game old slots for rollback.

The codec rejects missing/unknown fields, malformed records, non-track or missing object references, duplicate/unsorted paid IDs, mismatched piece prices or geometry, invented free-track refunds, inconsistent lifetime totals and invalid financial owners/balances. Existing byte, object, guest and 256-piece draft caps remain in effect.

## Reproduction

The established commands remain:

```powershell
Parkbound.exe --save-check write-track-construction --save-dir <new-absolute-directory>
Parkbound.exe --save-check verify-track-construction --save-dir <same-directory>
Parkbound.exe --save-check verify-legacy --save-dir <copied-legacy-slot-directory>
```

Run writer and verifier in separate processes. The graded 28-piece fixture retains its original station access and verifies a paid passenger midway up a chain lift, then 7,200 ticks of exact continuation, closure, a changed descending/brake profile and another completed paid journey. Quote checks use independently stated prices and compare full state before and after preview/refusal. Changed rails receive fresh IDs; retained rails keep their IDs and basis. The proof also covers ledger rollover, refund-first balances, unaffordable replacement, no-op Apply, partial demolition through a circuit fault, saved remnants, repeated-removal refusal and free legacy demolition.

Capture scenes include the retained `track-draft`, `track-invalid`, `track-ready`, `track-built` and `track-occupied`, plus:

| Scene | Evidence |
| --- | --- |
| `track-replacement` | Replace a paid $10 straight with $20 brakes; $5 refund and $15 net charge |
| `track-unaffordable` | Complete saved plan, low cash and atomic Apply refusal |
| `track-refund` | Selected paid rail with its demolition refund |
| `track-finance` | Actual construction, replacement refund and demolition ledger entries |

Review normal and 1024 × 680 windows, four rotations, Flat/Terraced and Surface/Underground. Captures establish presentation; input regressions establish ordering and simulation commands. Neither establishes native mouse/keyboard acceptance. Exactly edge-on steep rails still require rotating the view to expose their picking surface.

## Verification status

Implementation verification is in progress. Record the exact tested source, test counts, CI run, executable checksum, capture inspection, authentic migration and relocated-package results here before handoff. The current delivered 0.25 package remains identified in its historical report until a verified 0.26 package is produced. Do not attribute older test results to this source.

Authentic schema-14 primary and backup were produced by the retained 0.25 executable with SHA-256 `4377c734df042fac7a5731853b4ba3feef94337519df64d00d016e5161587e52`. The fixture includes an occupied graded circuit and an incomplete draft. Raw receipts are under ignored `artifacts/track-economy-0.26.0/`; they require OneDrive or explicit transfer between PCs. The [resume guide](../RESUME_DEVELOPMENT.md) retains all three checkout locations.

Native input, clean-PC offline/standard-user compatibility, signing/title/licensing decisions and representative-hardware performance remain open. No performance batch is required for this bounded gameplay milestone.

## Next stage

Improve the coaster editing workflow with copying the installed circuit into a draft and reversible draft edits, keeping the quoted accounting and atomic Apply contract. Further geometry, multiple coasters, broader ride families, realistic physics and full ratings remain later milestones. This bounded purchase/refund slice does not complete general construction or management acceptance.
