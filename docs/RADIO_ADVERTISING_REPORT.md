# Radio advertising stage — 0.67.0

## Scope

The starter park now installs a separate ground-level version of the original coaster circuit and its path/guest fixture. The station, low track, paths and guests begin at height zero on flat terrain. The lift rises to height eight, then the drop returns to ground. The elevated overlap demonstration fixture stays available for diagnostic scenes; existing saves retain their actual terrain and object heights.

The second marketing product is a fixed $40 radio campaign lasting 9,000 simulation ticks. It adds 300 cents to a candidate's deterministic entry-price willingness, without changing the guest's wallet or the arrival interval. Local flyers remain a separate $25 product that halves the arrival interval; both can overlap and expire independently. The radio candidate count records observed candidates while active. The extra-entry count is a direct decision difference against the same candidate's unadvertised willingness, not a broader causal estimate of campaign profitability.

Each radio purchase writes an ownerless **RadioAdvertising** expense. Finance keeps separate flyer and radio receipts, combines their costs for operating profit and cash, and checks total advertising against the two purchase counters. Strict park saves advance to **37/37** with version-36 migration. Current saves require four radio state fields; older schemas reject them and radio receipts. The `write-radio` / `verify-radio` production proof saves overlapping paid campaigns with 900 ticks left, loads paused in another process, compares 3,600 future ticks across expiry and the normal arrival pace, and checks that the original primary file remains byte-identical.

## Verification and limits

All **872 release workspace tests (491 game / 381 simulation)** passed after the starter-ground and active-flyer migration checks. Formatting, strict release all-target Clippy and the Windows release build passed. Executable SHA-256: `f55bf7ae1b62d16a3407ade700fac5d43a33fa0e27408be40e6e9178f0f794b6`. A fresh Unicode/space ZIP extraction verified **118 payload hashes and 106 current guides**. Eight extracted-game processes passed the radio, flyers, admission-rules and wellness writer/verifier pairs. The radio proof compared exactly 3,600 ticks from 900 overlapping campaign ticks remaining across expiry and ordinary arrivals; the primary save remained unchanged.

The 1024×680 `flat-start`, `visitor-radio` and `visitor-radio-finance` captures passed and were directly inspected. The starter station and low track sit at ground height with visible climbing coaster track. The Visitor flow panel shows both clocks, six candidates, three direct extra entries and all controls without clipping. Finance shows the combined $65 ad expense and reconciled cash without clipped rows. Capture SHA-256 values: `7bbdb5d0d480cb83ccaaf0db0fdd23c5f349400a17fdcf069722b05cef99b230`, `6a8a1213e9e5cbfc6b7b5a916050f04e846376e80e668b0c3f529b69fe538a06` and `67e2543dd5bb872a36a7b214df0caf0daff9c41508d2e2add2a56d11a2515eb7`. The unsigned local ZIP is `dist/Parkbound-0.67.0-windows-x64.zip` with an adjacent `.zip.sha256`; recheck it after final guide edits.

This development PC is not a performance reference. Native input, a clean target PC, scenario-authored campaign restrictions, campaign profitability analysis, research and objectives remain separate work.
