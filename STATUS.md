# Project status

## Current work — Ride graphics, 0.93.0

The ride graphics follow-up is implemented on `codex/ride-graphics-overhaul`, based on `69044094a68bad6d1f00cd07db4bcac1352efafa`. It improves ride bodies, moving vehicles, framing and track detail while preserving simulation and saves. See `docs/RIDE_GRAPHICS_GUIDE.md`. All 1,034 tests passed (570 game, 464 simulation), together with strict Clippy, formatting, fourteen ride captures, four wheel views and production demo/catalogue continuation. The extracted Windows package passed 166 payload hashes, 146 bundled guides, independent front/demo write/verify pairs and inspected title/demo captures. Runtime commit `dabbd2e6f99a6518a24e71ef2a78878103e375a3` is verified on GitHub in PR 85. The final documentation refresh preserves the same executable. No new external assets or dependencies.



Project ID `railrust-tycoon`.

## Current work

Version 0.92.0 classic graphics overhaul is implemented on `codex/classic-graphics-overhaul`, based on demo commit `2b0df01b22013e4a913a2f363b9963256efefcc7`, in PR 84. The selected direction adds original opening art, live menu controls, embedded fonts, detailed foliage, services, rides and visitors. Read `docs/CLASSIC_GRAPHICS_GUIDE.md` and `docs/CLASSIC_GRAPHICS_VERIFICATION.md`. Strict checks, all 568 game tests, four-view inspection and extracted-package play/save verification pass. All 464 simulation tests also pass, completing 1,032 tests. The remote workflow is still running at this checkpoint.

The 0.91.0 Sunhaven Gardens demo is delivered on `codex/playable-demo-park`, based on Phase 11 commit `8d6ed23c1c4a95e0163182bae1394515d94ec0aa`, in PR 83. The title screen starts a fresh 48×48 landscaped sandbox with seven rides, eight facilities and five staff roles. It uses production construction, guest arrivals, ride operation, money and saves. All 1,031 local tests pass. All seven rides complete paid visits and the exact 18,000-tick continuation passes across separate processes. Strict checks, release build and extracted-package gameplay/save checks pass. Runtime source `e0239b841c9df60d7a5bb76e7d4719b59002eeb1` is verified on the remote. See `docs/DEMO_PARK_GUIDE.md` and `docs/DEMO_PARK_VERIFICATION.md`. The final full GitHub workflow remains in progress at this checkpoint.

Phase 11 implementation is delivered in 0.90.0 on `codex/phase-11-beta-stability`, based on verified Phase 10 commit `3cecab22fe24aa6063ff4cb9d8b0dc53e3bfd07d`. Runtime `3f3bf6da1ff57b861ad4d6bb1358a144dfd4e97a` in PR 82 adds operational issue inspection, four current-content beta workloads, repeated save/backup checks, independent accounting reconciliation, fresh-process replay and balance reports. Shared path-distance searches reduce repeated ride/staff traversal. All 1,029 tests, the 70-step Windows build/gameplay job and all four one-hour endurance cases pass. See `docs/PHASE_11_BETA_GUIDE.md` and `docs/PHASE_11_VERIFICATION.md` for controls and evidence.

Park saves remain schema/content 51/51. Accessibility preferences are separate version 1. Sound settings advance to version 2 with explicit version-1 migration.

## Handoff registration

The existing GitHub repository and project registration were verified on 2026-11-16. The standard prompt was read online, version 2.1.0. The shared local checkout remains `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.

The native Stop export `q-38bbf860d212ca6448ffe4344b56e24b` was uploaded and all 144 files verified through fresh downloads under `strijbosmh@gmail.com` on 2026-11-13. Its accepted receipt is in the local Softwareprojecten receipts directory. That evidence applies to PC1830 and its recorded snapshot.

The later export `q-1981abefda07ea3b445a106c9e472de6` remains locally queued. 124 of 147 files have verified readback receipts after an interrupted tool session. Exports `q-e6c25399542a7d74945073bf34b4ff8f` and `q-be73a30431959a9891e9a44ae8c33f28` are also queued. No acknowledgement was submitted for these incomplete exports.

## Remaining work

The approved Codex Cloud handoff awaits a usable repository-connected environment. Previous creation attempts returned no task ID. Keep `docs/CODEX_CLOUD_HANDOFF.md` with the source and Windows package for that transfer.

Phase 11's full acceptance gate includes representative-hardware validation. This PC remains excluded from performance acceptance. Independent player playtesting, hardware/display coverage and subjective audio feedback remain open.
