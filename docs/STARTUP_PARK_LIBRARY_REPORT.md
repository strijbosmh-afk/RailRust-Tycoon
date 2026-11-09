# Phase 1 report: startup and park library (0.71.0)

## Keyboard follow-up

A player report exposed a New Park keyboard mistake: Enter activated the first mode row, which was already selected, and did not create the park. The opening keyboard focus now points to **Create Park**. This also applies when Campaign opens the selector with Meadow preselected. The action hint at the bottom of the screen now reads **Enter selects Create Park** on entry. A focused regression test checks the default and a changed mode; the [player guide](STARTUP_PARK_LIBRARY_GUIDE.md) now states the Enter shortcut.

The initial correction passed formatting, six focused release tests and the production release build. A fresh package capture exposed the same focus default in the separate `--capture-front new-park` path. That path now shares the corrected focus rule, and the regression test checks it. Final executable SHA-256: `0d4eb24009acf6c5143de530210f9c5f73fe5444ae65d72b84d87a375e4f59e8`.

The corrected extracted package passed 127 payload hashes, 115 guides, `write-front` / `verify-front`, `flat-start`, and `front-new-park`. The inspected New Park capture shows **Enter selects Create Park**. An isolated native launch from the package pressed Enter on Title to open New Park and pressed Enter again to create the live Sandbox park. The QA process exited; default player saves were untouched. The resulting ZIP is local and unsigned.

## Scope delivered

Normal launch now has explicit intro, title, New Park, library, loading/error, park, pause, and confirmation screens. The original Parkbound intro lasts three seconds and can be skipped. The title offers Continue, New Park, Campaign, Load Park, Settings, Credits, and Quit. New Park chooses Sandbox or one of two original campaign challenges, with goals and rules displayed before creation. Boardwalk remains locked until the existing campaign award makes it available.

The game keeps up to 32 named parks, with stable generated IDs independent of display names. Each park uses the existing validated primary/backup save store. A small atomic metadata file holds its name and last-save time. The library can load a primary or backup, rename, and archive a park after confirmation. Both old single-slot files can be imported explicitly as separate parks, with their original bytes retained. The root campaign record is still shared across parks. Park save schema and content remain 39/39.

In the park, Menu or a clear Escape opens a pause menu with Resume, Save, Save As, Load, Settings, Return to Title, and Quit. The menu states whether the park has unsaved changes. Return, Quit, and Load confirm before discarding changes; a loaded backup remains unsaved until the player saves it. Underlying map and edit input are blocked while a front screen is open. F5/F9/Shift+F9 use the active library park on normal launch. Diagnostic and render modes retain their previous isolated behavior.

## Acceptance checks

| Check | Expected observation |
| --- | --- |
| Intro and title | Intro skips by button or keyboard and settles on title; Continue is unavailable with a clear message when no healthy primary exists. |
| New Park and campaign | Sandbox creates only after Create Park; Meadow starts immediately; Boardwalk refuses while locked and can be created after an award. |
| Save and Continue | New named park saves into a generated slot; Return to Title then Continue loads the same park paused. |
| Multiple parks | Save As makes a separate named slot; list, page, rename, load primary/backup, and archive work without changing another park. |
| Legacy preservation | Primary and backup import separately; source bytes remain unchanged. |
| Unsaved protection | Return, Quit, or Load present a confirmation; Cancel keeps the current park. |
| Input isolation | Menu click does not select terrain behind it; keyboard menu controls do not reach park tools. |
| Restart | The fresh-process `front` proof verifies the saved library, backup, archive and legacy files. The existing `campaign` proof verifies award persistence and unlock. |
| Display | Front screens are captured at 1024×680 and 1920×1080 and inspected for clipping. |

The source tests cover library file handling and a new/save/edit/load/backup state transition. The package verifier runs the production executable in isolated directories and records output hashes. This development PC is not a performance reference.

## Verification result

The full release suite passed **501 game tests and 389 simulation tests** before the final pointer-release guard. After that guard, five focused release tests passed, including the guard and the new/save/load flow. Formatting, strict release all-target Clippy, and the Windows release build passed on the final source. The final executable SHA-256 is `4612c3c98773e3ff4f21ed074c78f5c5e80da18abedb0bbdda774626048bc42f`.

A fresh Unicode/space extraction in `artifacts/startup-park-library-0.71.0/corrected proof é/` verified **127 payload hashes, 115 current guides, and no extra files**. The `front`, `campaign`, `scenarios`, and `wellness` writer/verifier pairs all passed: eight separate production processes. The `flat-start` capture and six front-screen captures (`title`, `new-park`, `park-library`, `pause-menu`, `settings`, `credits`) completed at 1024×680 and were inspected directly. No controls or text were clipped. A separate 1920×1080 park-library capture from the corrected extracted executable was inspected; its list, actions, and Back control remained visible.

Native interaction used an isolated `--save-dir`, never the player's default saves. Mouse New Park and Create Park, keyboard Tab navigation, Save, Return to Title, keyboard Continue, paused restoration, F5, Menu and Quit all worked. The first Create Park test exposed a click-through: its click selected terrain behind the closing screen. A guard lasting through button release fixed it. On the corrected executable, the same Create click left the inspector at **Pick a surface**; a separate deliberate click selected **Tile 22, 23**. The corrected test park was saved while paused and closed. Native checks are limited to these gestures; they are not a clean-PC or broad display-input acceptance.

## Remaining work

Phase 2 of the [full-game roadmap](FULL_GAME_ROADMAP.md) adds responsive UI layout and display scaling. Phase 1 keeps the established 1024×680 minimum and existing fixed-size park management panels. The ZIP is unsigned; clean target-PC release acceptance belongs to the later public release phase.
