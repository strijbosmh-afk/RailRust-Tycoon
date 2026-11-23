# Native guide and English manual acceptance

- Project: `railrust-tycoon`.
- Session: `01a15441-13d3-7142-b742-e896afad208c`.
- PC: `PC1830`; checkpoint UTC: 2026-11-19 13:42.
- Branch: `codex/right-click-settings`; PR #93.
- Baseline commit: `11949ddabbfa640837ce93d31a40f1ddd27f870c`.
- Runtime source: `16292a10ce2bb7f1bb2467c2398078c93eb2a0db` (0.99.3).
- Executable SHA256: `f3c847f4927fec5f5485f212db4f571eaa341a5866a16a0ae11453fb2006f2f7`.
- The containing documentation commit records this acceptance; use the latest
  remote branch head when resuming.

## Native Windows checks

The owner requested a new native test after the earlier Computer Use stop.
The installed `%LOCALAPPDATA%\Programs\Parkbound\Parkbound.exe` was started
with `--safe-windowed` and an isolated profile at
`artifacts/native-guide-0993-acceptance`. Sound was muted for the test. Existing
player saves were not used.

1. Opened New Park and Create Park to start a fresh Sandbox.
2. Paused at 00:11, cash $1,000.00 and 12 guests.
3. Observed step 1/7 (welcome), clicked Next and observed 2/7 (public path).
4. Clicked Next without placing a path: 3/7 (gentle ride) appeared.
5. Continued without construction through 4/7 (entrance/exit), 5/7 (guests),
   6/7 (save) and 7/7 (resume).
6. Clicked the final Next. The guide disappeared, leaving the park view.
   Cash and simulation time stayed unchanged throughout the guide traversal.
7. Pressed H. Help displayed Detailed player manual in English.
8. Clicked its document icon. Microsoft Edge opened the installed
   `docs/PLAYER_MANUAL.html`; the address bar pointed into the Parkbound
   installation. The English title, version 0.99.3, contents and chapter text
   were visible. No web-hosted manual or translation was needed.
9. Returned to the test game and observed the successful manual-launch notice.
10. Closed Help, saved the isolated park with F5, observed Park saved, and
    closed the game normally. A process check confirmed that the test game
    had exited.

The requested seven-step navigation and Help-to-English-manual checks passed.
This pass does not claim a separate native test of every browser search/filter
control. The earlier focused guide regressions, strict Clippy, formatting and
save continuation tests remain applicable; no runtime source changed here.
Save schema/content remains 52/52. This PC is not a performance reference.

## GitHub CI and publication

Run `44533828792` for the baseline commit was inspected. Formatting, generated
manual freshness, original assets, strict lints, workspace tests, Windows
packaging, embedded icons, Tips persistence and demo-park continuation passed.
The workflow was still performing the remaining continuation checks at this
checkpoint. Installer and four endurance jobs follow the main Windows job.
Preserve the distinction between completed checks and a fully green workflow.

Current check snapshots are saved as `artifacts/native-guide-ci-*.json`.
The documentation-only update is committed and pushed on the existing branch;
verify that its remote head equals the local commit. Do not merge or change
branches as part of this acceptance task. Final ZIP/installer and installed
payload receipts live under `artifacts/native-guide-*` and `dist`.

## Resume and storage

No functional fix was needed. The previous outstanding native manual-opening
check is resolved. Before a wider release, confirm the latest GitHub workflow
has completed successfully. Development continues from PR #93's branch.

Shared Git root:
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Local standard prompt 2.1.0 and the existing project registration were used.
Google Drive synchronization stays disabled; existing exports remain local.
No manual snapshot or hook command was run.
