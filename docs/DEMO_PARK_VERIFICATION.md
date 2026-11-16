# Sunhaven Gardens verification

Version 0.91.0, branch `codex/playable-demo-park`, based on Phase 11 commit `8d6ed23c1c4a95e0163182bae1394515d94ec0aa`. The source is published through PR 83. Save schema/content remains 51/51.

## Observed gameplay

The park opens at simulation time 05:00 with 60 real visitors in 20 parties, five working staff members, seven open rides, eight connected facilities and $9,065 remaining from its $15,000 development budget. All research is available and automatic stock is enabled. Opening weather is sunny. The park is a sandbox with ordinary costs and paid tickets.

The fresh-process proof loads the opening save and advances 18,000 ticks. Its final state matches the uninterrupted run exactly and preserves the original save bytes. At simulation time 15:00, all seven rides have completed paid visits.

| Ride | Completed visits | Ticket revenue |
| --- | ---: | ---: |
| Sunburst Coaster | 6 | $12 |
| Willow Falls | 3 | $6 |
| Sunhaven Railway | 9 | $10 |
| Garden Carousel | 41 | $42 |
| Blossom Cups | 20 | $20 |
| Golden Wheel | 12 | $13 |
| Marigold Voyager | 15 | $16 |

Revenue is charged at departure. A paid passenger can still be riding or exiting when the report is taken. At that endpoint there are 70 active guests, zero turned-away arrivals and $8,767 cash. The five staff remain working and have each received $30 wages. Food, drink, toilet, rest, souvenir and information facilities have served visitors. Ice cream and first aid remain connected and available but received no completed use in this bounded run.

## Checks

- All 1,031 release tests passed, consisting of 567 game tests and 464 simulation tests. The demo-specific tests cover paid operation, connected access, save continuation, title entry and independent library saves that preserve an existing park.
- Strict release Clippy and formatting passed for the tested source. The final presentation-only correction updates the status bar to the current park name and passed a fresh lint/build check.
- Production `write-demo-park` and `verify-demo-park` ran in separate executable processes. The original slot remained byte-identical.
- Inspected 1920×1080 full-park captures from opposite rotations and the 1024×680 title screen. The demo button is visible and keyboard-focusable. The park uses ground-level paths and stations, a central lake and original ride/scenery rendering.

Local evidence is retained under ignored `artifacts/demo-*` paths. The Windows distribution is `dist/Parkbound-0.91.0-windows-x64.zip`. Its adjacent `.sha256` file records the archive checksum. The package verification report records the exact executable checksum, payload integrity and isolated extracted-package runs.

The shipping executable SHA256 is `55802dd42b858a03a1acd778bb2083fe60ee2d15313e601ac4b7a1a93f55884d`. The initial extracted package passed all 153 payload hashes and 139 current guides. Its `front` and `demo-park` write/verify pairs passed from `C:\Windows` with isolated profiles, and its title/demo captures passed and were inspected. The final package refresh changes the documentation and keeps this executable. Raw receipts remain in `artifacts/demo-package-verified/verification.txt` and the final refresh verification folder.

Runtime source `e0239b841c9df60d7a5bb76e7d4719b59002eeb1` is verified on GitHub in PR 83. Its Windows workflow is `43857209308`; the remote full workflow was still running at the documentation checkpoint. Earlier source workflow `43855141565` passed the new seven-ride fresh-process proof. The local checks and package evidence above are complete independently of the remote workflow.

## Scope

The deterministic simulation and the actual save/library paths are exercised. The rendering captures use the in-engine capture mode. Physical mouse/keyboard playtesting, subjective visual feedback and representative-hardware performance acceptance remain separate. This PC's timings are excluded from performance requirements.

The first local fixture failed on a track/path overlap. A second run exposed overlapping flower placement, and a subsequent queue experiment failed the existing branch rule. The corrected layout passed the final operation and continuation tests.

The preceding Drive archive remains incomplete after a readback download error. Queue `q-1981abefda07ea3b445a106c9e472de6` has 69/147 verified receipts. Queue `q-e6c25399542a7d74945073bf34b4ff8f` remains pending. No incomplete queue was acknowledged.
