# Original starter designs — 0.36.0

Version **0.36.0 is locally verified and packaged**, runtime source `93ffb17bc680c213b3b89aa0cf88e0e14eb8950d` on `codex/starter-track-designs` in [draft PR #35](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/35), based on verified 0.35.0 checkpoint `5e4356e6f6366ea5478145e4c91cbe171bd4800d` / PR #34. The car-count package remains retained. This bounded construction slice makes complete original layouts available through the existing import/preview/quote/Apply workflow. No milestone PRs have been merged.

The package includes **Pocket Oval** (16 pieces, 5 × 5), **Meadow Winder** (18 pieces, 6 × 5) and **Meadow Climb** (24 pieces, 8 × 6, three-step climb with drops and a brake). On fresh empty terrain their track quotes are $270, $300 and $420. Existing track, refunds, obstructions and placements can change the actual quote/readiness. [STARTER_DESIGNS_GUIDE.md](STARTER_DESIGNS_GUIDE.md) covers player construction and access.

`crates/park_sim/examples/generate-starter-designs.rs` authors these original layouts using production track specifications and the design codec. It validates every loop in four rotations before writing canonical JSON into a new absolute directory; it refuses an existing destination. No external game assets or layouts were used. The three tracked JSON files are the package content; regeneration should be compared in a separate directory before changing them.

Saves remain **19/19**, and design/catalog formats remain **1**. Runtime simulation rules are unchanged from 0.35. The library explains where to import the starters. Small car-label and capture-camera improvements are included. No player catalog is filled or replaced automatically.

Source tests verify canonical codecs, all rotations, actual purchase quotes and settlement, complete laps, strict save roundtrips and paid journeys. For each layout the actual existing path planner builds a valid return route, including slopes/bridges, without damaging the queue. Production `write-starters` / `verify-starters` import and export all three files through the existing transfer completion boundary, preserve canonical bytes and the catalog, build all layouts for their quotes, save a paid occupied primary and parked backup per layout, and verify 3,600 ticks of future state per layout after restart. Source slots remain unchanged. Native file-picker gestures are a separate acceptance check.

Capture scenes `starter-pocket`, `starter-winder`, `starter-climb` and `starter-paid` use actual bundled content. The preview scenes are read-only in-memory catalogs; filesystem import/export is proved independently. Tests exercise the real PostStartup boundary and fresh fixtures. Package generation copies the three named files, and package verification checks every bundled design against its current source in addition to manifest and guide checks.

## Verification

Formatting, strict all-target release Clippy and all **711 release tests (421 game / 290 simulation)** passed. The final production build completed in 39.18 seconds. The independently verified local executable is **49,872,896 bytes**, SHA256 **`ca7f16629819488bccb405542ee635bc8851cc8a8d589c109edc6406d8bd8534`**. No runtime source changed after that build.

Fresh extraction outside the checkout to `C:\Users\strij\AppData\Local\Temp\Parkbound 0.36 starter delivery é`, launched from `C:\Windows`, passed **56 payload hashes / 44 current guides**, with all three bundled design files matching source. All **18 production proof processes** passed: writer/verifier pairs for starters, trains, operations, dispatch, appeal, design transfer, track construction, access and coaster testing. A separate **four-process 0.35 ↔ 0.36 continuation proof** passed with actual old/new executables and unchanged source slots. Schema/content stays 19/19.

Ten final 1024 × 680 captures cover the three selected starter previews, a paid two-seat starter journey, one-car settings, the enlarged train map, completed testing, prepared exit path, library gallery and Help. Direct visual inspection checks names, shapes, prices, footprints, control/notice fit and retained access guidance. These controlled captures and production transfer proofs do not claim physical file-picker/mouse acceptance.

All **291 dependency entries** and five runtime/notice files match 0.35 byte for byte. Logs and visual receipts are retained in `artifacts/starter-designs-0.36.0/`. Final guide refresh uses the same verified executable and another fresh extraction; the archive's adjacent `.zip.sha256` identifies the refreshed ZIP.

Exact-source Windows [CI run 40141146431](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40141146431), job 121012967542, remains in progress at this local checkpoint. No CI artifact was downloaded or executed. Previous 0.35 runtime source passed all 36 steps of CI 40139500981.

## Remaining work

The broader construction, multiple-coaster, ride-family, staff, maintenance, progression, scenario, audio and management roadmap remains open. RID-08 remains in progress. No reference PC is selected, and this development PC remains excluded. Preserve all three continuation paths and the current map/save caps. Physical native input, clean-PC/offline/standard-user and representative performance acceptance remain separate gates.
