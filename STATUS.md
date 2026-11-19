# Project status

## Current work — Structured construction menus, 0.98.1

Branch `codex/themed-menus-borderless` continues from `18d9ee8fee24dcab33cfff85977ba8b28e82dc6b`. The Build panel now groups Paths, Landscape and Scenery/park items. Construction icons use colored filled artwork, with distinct signs, patrol routes and brush controls. The title, new-park, pause and settings screens regain their original text buttons. Verification and distribution are in progress. Read `docs/MENU_THEME.md` and `docs/sessions/2026-11-19-structured-build-menu.md`.

The shared Git root is `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. Google Drive remains disabled and local standard prompt 2.1.0 is in use. Saves remain 51/51 and display settings remain version 1.

## Current work — Menu theme and borderless startup, 0.98.0

Branch `codex/themed-menus-borderless` starts at `4933b0ec75fa33c4d62c1a251a2803fb61674213`. Forest-green menu panels, cream text and gold accents are implemented. The owner's clarification applies icon-only controls and live hover/focus labels to every menu button. New display settings start in borderless fullscreen. Verification is in progress. See `docs/MENU_THEME.md` and the current session report.

The shared Git root remains `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. Google Drive synchronization is disabled. The cached standard prompt is 2.1.0. Saves remain 51/51 and display settings remain version 1.

## Current work — Soundtrack and spatial audio, 0.97.0

Branch `codex/soundtrack-and-spatial-audio` starts at `7c41a3ef5b9e7c55cf3a35d4a857ce0717e28911`. Runtime `44c25003d7bb7172175ec85f385c2c2394c9e56c` is published in PR 90. Three arranged original tracks, zoom-aware ride sound and recorded visitor cheering are complete. All 1,045 tests, strict Clippy, formatting, eleven independent WAV re-renders, Windows audio output and extracted-package demo/front/audio continuation passed. See `docs/AUDIO_OVERHAUL.md` and `docs/sessions/2026-11-18-soundtrack.md`.

Version 0.97.0 is installed on PC1830. All 191 payload hashes and 164 bundled guides were checked. Both existing player files were preserved. Executable SHA256 is `e86078439abe0730613dfc7cc7682043fcac8075ecaba6b11cdd5ccddabd75f4`. ZIP and installer hashes accompany the files in `dist/`. At 2026-11-18 15:35 UTC, GitHub run `44333819884` had passed tests and packaging and was continuing integration checks. Physical speaker/headphone listening remains separate from the verified PCM/device checks.

Google Drive synchronization is disabled at the owner's request. The project flag and local registration are set to false, and the archive helper suppresses Drive continuations. Local files and GitHub remain in use. Existing exports are retained locally. The cached standard prompt is 2.1.0. Saves remain 51/51 and audio settings remain version 2.

## Current work — Game icon, 0.96.1

Branch `codex/game-icon` starts at `589a09b6da98131ffb729499c41f895880755aab`. Runtime source `c18174696b34460eb48db76d4f015c5876a1c483` is published in PR 89. The approved original icon is integrated into the executable, native window/taskbar, setup, uninstall and both shortcuts. All 1,038 tests, strict Clippy, formatting, resource-byte checks, native-window checks and extracted-package demo/front continuation passed. The 0.96.0 installation was upgraded to 0.96.1 with all 185 payload files verified and both player files preserved. Read `docs/GAME_ICON.md` and `docs/sessions/2026-11-18-game-icon.md`.

The executable SHA256 is `4cb731db5641fb677be3fb6781863d577fb4d87ad306d61ee70df60645c9052c`. ZIP and installer checksums accompany the local files in `dist/`. At the 2026-11-18 14:48 UTC checkpoint, GitHub run `44325579555` had passed tests, packaging and embedded-icon verification and was continuing gameplay integration checks.

The shared checkout remains `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. Saves remain 51/51. The preceding native Drive export `q-2cd0c0a02dc9bcfa808be6b1fb4b7a64` has an accepted receipt for 173 verified files. Three older exports remain queued. This PC supplies correctness evidence only.

## Current work — Detailed park graphics, 0.96.0

Branch `codex/park-detail-overhaul` starts at `886aa7c318b3cd11c399c7b9033aa7586f0ed579`. Graphics source `202a9bdb039a45140f5d0d83aa161df54b37421f` is published in PR 88. Physical flower heads and stems, fern fronds and pine clusters, paving bevels and timber fixings, rounded shop signs/products, platform inlays, ride ornaments, round coaster wheels and varied clothing are implemented. The maximum camera zoom is doubled. All 1,038 local tests, strict Clippy, formatting and the release build passed. Twenty after-captures, production demo continuation and the full catalogue continuation were checked. Read `docs/PARK_DETAIL_GRAPHICS.md`. The current GitHub run passed workspace tests and was continuing integration checks at this checkpoint.

The source and original artwork remain in the shared OneDrive checkout. Saves remain 51/51. The previous native Drive export `q-3786d3de32c0aa387b5c1a4f74ae4c1a` has 171 verified files and an accepted receipt. Three older exports remain queued.

## Current work — Windows distribution, 0.95.0

Branch `codex/phase-12-windows-distribution`, based on `b84dd85ba2296715fc7d8cfcf3108be0d288d7f0`. Runtime `c55c0b7894771e58813277f8c2453ecb7b94fef5` is in PR 87. The per-user installer, running-game guard, version query and download identity records are implemented. Formatting, strict Clippy, the mutex regression, release build, title capture and 14-process installation lifecycle passed. The real 0.94.0 payload upgrades to 0.95.0 while retaining 21 park, campaign and settings files. Read `docs/PHASE_12_WINDOWS_DISTRIBUTION.md` and `docs/PHASE_12_DISTRIBUTION_VERIFICATION.md`.

The owner deferred the open-source license choice while the Windows test version is completed. Phase 12's public 1.0 gate retains the independent clean-PC, hardware and player checks. Save schema/content is 51/51. Shared checkout is `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.

Corrected source `10f5215aeb39ddf460936c2b3e80dddc0d1da9a2` passed the Windows build, all 1,038 tests, production continuation checks and the complete installer lifecycle in GitHub run `44298610393`. The verifier resolves the registered uninstaller after rapid reinstall. The local ZIP's seven-ride demo continuation and title capture also passed. The starter endurance case passed; three longer cases were still running at the closing checkpoint. Installer evidence is retained under `artifacts/phase12/ci-corrected-installer`.

The 0.94.0 DropSync transfer has eight files verified through independent downloads. Native Drive export `q-d4882dccb560c1de91b9a01c5d681d98` has an accepted receipt for 164 files. Three earlier exports remain queued.

## Current work — Landscape graphics, 0.94.0

Branch `codex/landscape-graphics`, runtime `5488cf973ec5b4222fadd0c7d192036f24db3ac1`, in [PR 86](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/86). Terrain materials, stone promenades, timber paths, flower borders, conifers and oak variation are implemented. Strict checks, GitHub workspace tests, actual landscape/height/rotation captures and extracted-package demo continuation passed. Read `docs/LANDSCAPE_GRAPHICS_GUIDE.md` and `docs/LANDSCAPE_GRAPHICS_VERIFICATION.md`. Shared OneDrive checkout: `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. Save schema/content stays 51/51; this PC is not a performance reference. DropSync handoff: `Rust builds/Parkbound`; see `docs/DROPSYNC_HANDOFF.md` for contents and verification status.


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

Export `q-1981abefda07ea3b445a106c9e472de6` is now acknowledged after all 147 files passed remote readback. The later queue `q-ec36afe8b0f661ac064d300a83a347e2` has 56/160 verified receipts; its upload stopped when the required account check failed. Exports `q-e6c25399542a7d74945073bf34b4ff8f` and `q-be73a30431959a9891e9a44ae8c33f28` remain queued. No incomplete export has been acknowledged.

## Remaining work

The approved Codex Cloud handoff awaits a usable repository-connected environment. Previous creation attempts returned no task ID. Keep `docs/CODEX_CLOUD_HANDOFF.md` with the source and Windows package for that transfer.

Phase 11's full acceptance gate includes representative-hardware validation. This PC remains excluded from performance acceptance. Independent player playtesting, hardware/display coverage and subjective audio feedback remain open.
