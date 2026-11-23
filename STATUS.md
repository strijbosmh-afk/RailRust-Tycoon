# Project status

## 0.99.3 — Graphics across parks

Crossroads and waterfront walks pick their surface finish from the layout.
The first clear, rectangular sand-bordered pond on normal-size maps can
display an ornamental fountain. Tracked stations and several flat ride
platforms have model-specific finishes. See
[the graphics session](docs/sessions/2026-11-23-park-wide-graphics.md).

## 0.99.3 — Park districts

Sunhaven has patterned pond paths, benches and timber viewing walks.
Willow Falls, Garden Carousel and Marigold Voyager have distinct surroundings.
Service silhouettes, raised path rails and dusk shadows improve readability.
See [the district graphics session](docs/sessions/2026-11-23-park-districts.md).

## 0.99.3 — Sunhaven plaza

Sunhaven's pond has an animated fountain and scattered reeds at its edge.
Visitors use distinct counter, bench and queue poses. Evening lamps light
their path decks and open service counters glow warmly. See
[the plaza session](docs/sessions/2026-11-23-sunhaven-plaza.md).

## 0.99.3 — Park planting, shadows and motion

Low planting grows in visual clusters on open lawns. Trees, stations and
raised paths cast ground-clipped contact shadows. Ride entrances have animated
pennants, waiting visitors shift their gaze, and water glints are more visible at park scale. See
[the park atmosphere session](docs/sessions/2026-11-23-park-atmosphere.md).

## 0.99.3 — Classic coaster graphics

The Classic Coaster has pitched platform roofs, brighter rails and train
trim, more legible timber supports, and gold accents on its queue. Sunburst
Coaster in Sunhaven Gardens is the reference ride. See
[the coaster graphics session](docs/sessions/2026-11-23-sunburst-graphics.md).

## 0.99.3 — Water graphics

Water has a deeper teal base, lighter shallows at land edges, and varied
ripples. The surface depth and clipping rules remain in effect. See
[the water graphics session](docs/sessions/2026-11-23-water-graphics.md).

## 0.99.3 — Park startup

Sunhaven Gardens prepares its deterministic opening state on a background
worker during the intro. The title screen reuses that state for independent
demo starts in the same process. Creating a new park or starting the demo
shows a progress message while the park is prepared. See
[the startup session](docs/sessions/2026-11-19-park-startup.md).

## Current checkpoint — Complete PC transfer, 0.99.3

The requested handoff is in `dist/Parkbound-Transfer-2026-11-19`, with a
companion ZIP of the same name. Start with its `START_HERE.md`. It contains the
complete committed source and all tracked Markdown, the Windows installer,
the portable game ZIP, an offline Git-history bundle and a separate player-data
backup. `docs/TRANSFER_TO_ANOTHER_PC.md` explains playing, restoring saves and
resuming development. The source commit and distribution build commit are
recorded separately in the transfer manifest.

The runtime/documentation baseline `e2b86b6965fbcf43da9346ea5e79b67c67f3dd33`
now has a fully successful GitHub CI result: run `44536241835`, all six jobs
including the Windows installer and four endurance cases. This supersedes the
previous in-progress CI note. The runtime remains 0.99.3 / schema-content 52/52;
there is no gameplay change in this handoff. Source and Git bundle restore
checks, package hashes and player-data copy checks are recorded with the
transfer. Use `Verify-Transfer.ps1` after copying it to the destination PC.

Continue on `codex/right-click-settings` / PR #93. The transfer's source
snapshot is for browsing; clone its Git bundle into a new folder to preserve
history. Install the Rust/C++ build tools only when developing; playing needs
only the supplied installer or portable ZIP. The destination PC has not been
installed or tested by this transfer operation.

Original shared Git root:
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Project `railrust-tycoon`, PC1830, local standard prompt 2.1.0. Preserve the
existing project identity and remote, and register the actual path on the new
PC. Google Drive remains disabled; existing exports stay local. This PC is
not a performance reference.

## Current checkpoint — Native guide and English Help verified, 0.99.3

On PC1830, the installed game passed the complete seven-step first-park guide.
A fresh Sandbox used an isolated profile, and Next advanced through steps 1–7
without requiring construction, a guest trip or a save. The last Next closed
the guide. During navigation, the park stayed paused at 00:11 with $1,000.00.
The test park was saved separately after these checks and closed normally.

Help (H) → the document icon opened the installed
`docs/PLAYER_MANUAL.html` in Microsoft Edge. The browser showed the English
“Parkbound — complete player manual”, version 0.99.3, its contents panel and
English chapters. The game also displayed its successful manual-launch
notice. This resolves the earlier outstanding native opener check. A browser
translation offer is browser UI; the shipped manual remains English.

No code fix or save migration was needed in this acceptance pass. Runtime
source remains `16292a10ce2bb7f1bb2467c2398078c93eb2a0db`; tested documentation
baseline was `11949ddabbfa640837ce93d31a40f1ddd27f870c`. Continue from the latest
`codex/right-click-settings` commit / PR #93. Installed/runtime SHA256:
`f3c847f4927fec5f5485f212db4f571eaa341a5866a16a0ae11453fb2006f2f7`.

GitHub CI run `44533828792` was inspected on 2026-11-19: formatting, manual
freshness, asset validation, strict lints, workspace tests, Windows packaging,
embedded icons, persistent Tips and the demo-park check passed. Remaining
continuation/installer/endurance jobs were still running at this checkpoint;
this is not a claim that the entire workflow is green. Check the latest
[PR #93 checks](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/93/checks)
and local `artifacts/native-guide-ci-*.json` receipts before release.

The refreshed documentation uses the same 0.99.3 executable. Distribution and
installed-payload verification receipts are retained under
`artifacts/native-guide-*`; package checksums remain beside the files in `dist`.
See `docs/sessions/2026-11-19-native-guide-manual-acceptance.md` for the test
sequence. Further browser search/filter interaction is not claimed by this
pass; the requested Help-to-English-manual launch is verified.

Shared Git root: `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Project `railrust-tycoon`, PC1830, local standard prompt 2.1.0. Drive remains
disabled and existing exports stay local. This PC is not a performance reference.

## Current checkpoint — First-park guide navigation fixed, 0.99.3

Runtime source: `16292a10ce2bb7f1bb2467c2398078c93eb2a0db`, branch
`codex/right-click-settings`. The following documentation commit records the
completed validation and installation. Resume from the latest remote branch.

Next no longer stops at 2/7. Every step can be browsed without completing its
practice task. The guide shows explicit optional-practice status, updates tool
and navigation tooltips, and finishes normally after step 7. It detects a new
connected public-path ID even if another path was removed, and can inspect an
existing ride's access in Sunhaven or another established park.

All six guide regressions passed, including the complete seven-step flow in
starter and demo parks with byte-identical saved park state, actual practice
with construction/guests/saving, connected-path detection, existing-ride access,
replay and modal guards. Formatting, strict workspace/all-target Clippy and the
Windows release build passed. The extracted package passed 204 payload hashes,
175 current guides and separate park-library writer/restart verification.

PC1830 was upgraded to 0.99.3 at `%LOCALAPPDATA%\Programs\Parkbound`. All 204
installed payloads matched, four player files stayed byte-identical and both
shortcut targets/icons were verified. Runtime SHA256:
`f3c847f4927fec5f5485f212db4f571eaa341a5866a16a0ae11453fb2006f2f7`.
The final documentation refresh uses the same runtime. Current ZIP/installer
checksums are beside the distribution files in `dist`; evidence is in
`artifacts/guide-*`.

The English manual and quick-start guide explain optional practice. Save
schema/content stays 52/52; guide progress is transient and no migration is
needed. The earlier native manual-opening retest is a separate outstanding
acceptance item and is not claimed by this guide fix.

Shared Git root: `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Project `railrust-tycoon`, PC1830, local standard prompt 2.1.0. Drive remains
disabled; keep existing local exports. This PC is not a performance reference.

## Current checkpoint — English manual and context controls, 0.99.2

Continue on `codex/right-click-settings` / PR #93. English-manual source:
`34cf0fdb7311d1c06234e833432f069176731856`. Windows-opener fix:
`42c7f09fb3bb57ce888d2b7db72c3c1ad0b7396b`. The subsequent documentation
commit records this acceptance checkpoint. The complete manual and all related
Help text are English. `docs/PLAYER_MANUAL.md` is canonical; its HTML has search,
25 chapters, 80 headings, 120 local links and 14 tables. The older HTML address
is an identical English compatibility copy.

Native checks passed on the 0.99.2 package: right-click Garden Carousel #4 and
Drinks #2 selected the correct individual settings. Earlier checks also opened
Golden Wheel #6 from its queue and changed/restored the drinks price through
actual buttons. The isolated saved park reloaded paused with its state intact.
No real player data was used.

The manual-opening test found the earlier Explorer launcher opening Documents.
It now uses `ShellExecuteW`, literal UTF-16 filenames with native separators
and checked error codes. The corrected build and strict all-target Clippy
passed. The eight focused resolver/pointer tests passed before the final opener
replacement; the new Windows call was compiled and linted. HTML generation,
all local links and JavaScript syntax passed. The package passed 203 payload
hashes and 174 current guides.

IMPORTANT: the user stopped Computer Use with physical Escape before the
corrected opener could be visually retested. Do not mark that final native
manual-opening/search check complete. The running isolated test process was
left alone after the stop. Close the game before installing the prepared
0.99.2 update; the installer refuses an active game. Installation status and
final package receipts are under `artifacts/english-*` and `dist`.

Runtime SHA256:
`d6f68d4fcc2df1e9c922cf4cff34c16a2e8687bbf43ec2c101df66f60eb9c67b`.
GitHub CI is inspected through [PR #93 checks](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/93/checks).
Local passing tests do not imply a green remote workflow. See the latest
commit's checks and local CI receipt for the publication-time state.

Remaining: visually verify that Help opens the English manual, exercise its
search and chapter links, and confirm all remote jobs have completed. Native
input was stopped by the user; do not resume it without a new request.

Shared Git root: `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Project `railrust-tycoon`, PC1830, local standard prompt 2.1.0. Park schema/content
stays 52/52. Drive remains disabled and existing exports remain local. This PC
is not a performance reference.

## Current checkpoint — Right-click settings and Dutch manual, 0.99.1

Runtime and manual source: `38179d701e229ab38fe4d844c026c7c06b04611e`,
branch `codex/right-click-settings`, based on the previous menu/advice phase
at `3d08981d99f8f3373941a2ffe72deaeaf2e66bd6`. Resume from this branch's latest
remote commit, which also includes the final validation record.

A short right-click opens the visible ride or service's own settings. Track,
owned queues and gates resolve to their ride. Right/middle dragging still pans;
focus, modal, camera and loaded-park changes cancel pending clicks. See
`docs/OBJECT_SETTINGS.md`. Park schema/content remains 52/52.

The Dutch player manual contains 25 chapters, about 9,500 words, 80 heading
anchors and 14 reference tables. `docs/HANDLEIDING_NL.md` is the source;
`docs/HANDLEIDING_NL.html` is the offline searchable version. Help (H/F1) offers
its document icon. All 120 local links resolve. Build with
`python scripts/build-player-manual.py`; `--check` validates the output in CI.
Python is a development tool only; the player uses a normal browser.

All 599 application tests, formatting, strict workspace Clippy, the release
build and seven-size embedded-icon checks passed. The extracted package passed
200 payload hashes, 172 current guides and separate write/restart checks for
ride operations, Tips and the park library. Manual HTML/link and JavaScript
syntax checks passed. Logs are retained under `artifacts/context-*` and
`artifacts/manual-audit.json`.

PC1830 is updated to 0.99.1 at `%LOCALAPPDATA%\Programs\Parkbound`.
All 200 installed payloads matched; four existing player files were preserved
byte-for-byte, along with both shortcut targets/icons. Runtime SHA256:
`2bff66774ed77e6b542762fbbdcc2455ddddc6ef1e9d8c6e14d36e5b2f275ffd`.
The final guide-only package refresh uses this same executable; download
checksums and installer records live beside the files in `dist`.

Native input testing reached the real title screen in an isolated profile, then
the user stopped Computer Use with Escape. No further UI automation followed.
Native right-click and browser-opening acceptance remain unverified; the input
behavior is covered by automated application regressions. The next manual
check is a ride click, a service click and Help opening the offline manual.
This PC supplies correctness evidence, never a product performance reference.

Shared Git root: `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Project `railrust-tycoon`; local standard prompt 2.1.0. Google Drive remains
disabled. Preserve existing local exports and the complete AppData player
folder when continuing on another PC.

## Current work — Organized menus and park advice, 0.99.0

Runtime source `27e9eb714dfceab786561225a6a97301e57c83d9` is on
`codex/park-advice-and-menus`. Remaining game menus have labelled sections,
clearer facility and management icons, and readable form/list entries. The main
menu retains its original text buttons. The third header shortcut is Tips,
which returns two current-state suggestions with three uses per park. The
allowance survives saving, primary/backup loading, restarting, copied parks
and construction undo. New parks start with three uses. Read `docs/PARK_TIPS.md`.

All 1,057 workspace tests passed, followed by all 35 UI tests after the final
icon refinements. Formatting, strict Clippy, original asset records, Windows
release compilation and embedded icon verification passed. Fifteen engine
captures cover the menus, Tips, 1024x680 through 3840x2160 and main-menu text.
The extracted package passed 196 payload hashes, 169 current guides and separate
Tips, park-library and playable-demo write/restart processes.

PC1830 is upgraded to 0.99.0 in `%LOCALAPPDATA%\Programs\Parkbound`. All installed
payloads and both shortcuts were verified; both existing player files retained
their exact hashes. Executable SHA256: `6d317b31dc11a43eedec9075fd490a72e8a568e7d59b9390abd0f150e78d3683`. The final documentation
refresh uses the same executable. ZIP and installer checksums accompany their
files in `dist/`. See `docs/sessions/2026-11-19-park-advice-menus.md` for evidence.

Shared Git root: `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Park saves are 52/52; supported older parks migrate with zero tip uses spent.
Copy the complete `%LOCALAPPDATA%\Parkbound\saves` folder to carry park-library
metadata to another PC. Google Drive remains disabled; local standard prompt
2.1.0 is in use. This PC supplies correctness checks only.

The requested menu and advice work is complete. Resume from the latest commit
on this branch and consult `docs/FULL_GAME_ROADMAP.md` before selecting another
phase. Independent player acceptance and representative-hardware performance
remain separate roadmap work. Check the new GitHub workflow's outcome before
merging; publication alone is not a CI pass.

## Current work — Structured construction menus, 0.98.1

Runtime `726dcfc8ce400eb452ca3d0aedbf952227f1fef9` on `codex/themed-menus-borderless` is published in PR 91. The Build panel groups Paths, Landscape and Scenery/park items. Construction icons use colored filled artwork, with distinct signs, patrol routes and brush controls. The title, new-park, pause and settings screens regain their original text buttons. All 44 relevant UI/front/display tests, formatting, strict Clippy, release compilation, screen captures and extracted-package front continuation passed. Read `docs/MENU_THEME.md` and `docs/sessions/2026-11-19-structured-build-menu.md`.

PC1830 is upgraded to 0.98.1. All 194 installed payloads match the package, both existing player files are preserved and both shortcuts were checked. Executable SHA256 is `b7c9ba207b448d02a8b1d79222a0354aa0168946553320243e2cdbc5d0adf214`. ZIP and installer checksums accompany the downloads in `dist/`. The GitHub run was still in progress at the verification checkpoint.

The shared Git root is `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. Google Drive remains disabled and local standard prompt 2.1.0 is in use. Saves remain 51/51 and display settings remain version 1.

## Current work — Menu theme and borderless startup, 0.98.0

Branch `codex/themed-menus-borderless` starts at `4933b0ec75fa33c4d62c1a251a2803fb61674213`. Forest-green menu panels, cream text and gold accents are implemented. The owner's clarification applies icon-only controls and live hover/focus labels to every menu button. New display settings start in borderless fullscreen. This intermediate build is superseded by 0.98.1 above. See `docs/MENU_THEME.md` and the current session report.

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
