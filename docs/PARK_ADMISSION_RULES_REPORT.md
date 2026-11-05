# Park admission and charging rules — 0.65.0

## Delivered behavior

The visitor arrival clock now creates a candidate first, evaluates a stable willingness against the park entry price, and either counts a turned-away visitor or admits a guest. A paid entrant starts with the reduced wallet and increased lifetime spending. Park cash and a dedicated **ParkAdmission** finance receipt increase by exactly the same amount. The visitor-flow panel shows the rule, price, cumulative entry income and refusal count. Finance separates entry income from ride tickets and service sales while including it in cash and operating totals.

The player can select Flexible, Free entry or Free rides. Rules prevent forbidden future prices and clear prices that become forbidden. Existing locked passenger fares continue to completion. Price changes do not charge current guests again. The current willingness rule spans $3.00–$20.00 per candidate, is deterministic from the crowd seed and candidate ID, and is intended as a simple demand foundation.

Strict park save schema/content advances to **35/35**. Version-34 and older parks migrate with free entry, Flexible charging and zero entry history. Version 35 requires all new visitor and archived finance fields; earlier versions reject them. Save validation reconciles guest and retired wallet spending with ride/service/entry revenue and the finance ledger. A separate-process `write-admission-rules` / `verify-admission-rules` proof loads a paid entry park with a refused candidate, switches to Free rides during a previously paid passenger trip and compares an exact 3,600-tick replay without rewriting the primary save. Windows CI includes the proof.

## Verification

All **861 release tests (489 game / 372 simulation)**, formatting, strict release all-target Clippy and the Windows release build passed. Executable SHA-256: `9d06df24a8c0f60bf78aec236f0e59cef2ab89d253de099c0d5bc4af76316e29`. A fresh Unicode/space extraction verified **114 payload hashes and 102 current guides**, then passed eight production game processes: admission-rules, park-open, drop-tower and wellness writer/verifier pairs. The admission proof loaded a park after a high-price refusal and paid entry with a paid tower passenger, compared exactly 3,600 future ticks and left the original primary save unchanged.

Three 1024×680 captures were directly inspected: `visitor-pricing`, `visitor-demand` and `visitor-free-rides`. Price, policy and refusal counts fit the Visitor flow panel; the top Revenue number includes park entry income. Capture SHA-256 values: `89488a75d4ca440a03ec71707ad1a50b7f16601f7277e9fccda1e1109a14ef70`, `dc79ce5daa3d563a3a8eeff0cfadedd9afbe8160fdb8979c01b0feee4f3e6844` and `30a89535a2470c6607aea7373b355212b7b79264e378b1cd272bf8596c88902a`. The unsigned local ZIP is `dist/Parkbound-0.65.0-windows-x64.zip` with an adjacent `.zip.sha256` file; the package is rechecked after the final documentation edits. Windows CI runs the new proof when the branch is pushed.

This PC is not a performance reference. Native controls and clean target-PC acceptance remain to be done. This stage provides player-selectable charging rules; it does not yet provide scenario-authored locked rules, objectives or completion awards.
