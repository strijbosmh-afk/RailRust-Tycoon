# Local flyer advertising stage — 0.66.0

## Scope

The player can purchase one fixed local flyer campaign for $25.00 from Visitor flow while the park is open and ready for arrivals. It lasts 9,000 simulation ticks and halves the visitor-candidate interval from 900 to 450 ticks. The clock is part of authoritative simulation time, including during a later closure or disabled arrivals; pause freezes it. The ordinary interval resumes at expiry. Entry price, visitor willingness, capacity and connectivity continue to determine actual admission. The UI distinguishes candidates seen during the campaign from incremental customers.

Each purchase reduces park cash once and writes an ownerless **Advertising** ledger receipt. Finance reports a distinct advertising expense, includes it in total costs, operating profit, net cash and archived history, and validates it against the campaign purchase count. Strict park save schema/content advances to **36/36** with explicit version-35 migration. Missing or forged current marketing fields and older-version marketing fields/receipts are rejected. A new separate-process `write-flyers` / `verify-flyers` proof saves an active paid campaign, loads it paused, and compares an exact 3,600-tick replay across expiry without changing the primary file.

## Verification and limits

All **866 release tests (490 game / 376 simulation)**, formatting, strict release all-target Clippy and the Windows release build passed. Executable SHA-256: `77042388ac6f6213bdf6bd5e759ea7407518b5d7a3d12d330241deab86b57f12`. A fresh Unicode/space extraction verified **116 payload hashes and 104 current guides**, then passed eight production game processes: flyers, admission-rules, park-open and wellness writer/verifier pairs. The flyer pair resumed a paid campaign with 900 ticks left, compared an exact 3,600-tick replay across expiry and normal arrivals, and left the original primary save unchanged.

The 1024×680 `visitor-flyers` and `visitor-flyers-finance` captures passed and were directly inspected. The campaign clock, candidates, $25 button and existing visitor controls fit; Finance shows the separate flyer cost and the reconciled operating result without clipped rows. Capture SHA-256 values: `b6ab307d9e4570fecc0255c88125b913c4189e5d7a916f9c38329a65c33dfff4` and `1fdcd0582a3d2c1389df570d6ce3ebcd01019755bbed29b100999f4ffb00ba2f`. The unsigned local ZIP is `dist/Parkbound-0.66.0-windows-x64.zip` with an adjacent `.zip.sha256`; it is rechecked after the final guide edits. Windows CI runs the new proof after the branch is pushed.

This development PC is not a performance reference. Native input and a clean target PC remain separate checks. This stage is a single original advertising product; broader marketing types, measured campaign attribution, scenario-authored restrictions, research and objectives remain open.
