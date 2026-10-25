# Checked ride opening — 0.37.0

Version **0.37.0 is locally verified and packaged**, runtime `d63c7c4acd4c19f45e452bf3132994d29673ba5e` on `codex/ride-opening-checks` in [draft PR #36](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/36), based on verified 0.36.0 checkpoint `b861b75`. Previous exact-source Windows CI 40141146431 passed all 37 steps. The three original starter designs and configurable car-count packages remain retained. No milestone PRs have been merged.

This bounded RID-05 step checks the installed ride, entrance/queue and public exit return path before player opening. Operations, the main Open button and O use the same simulation `set_ride_open_checked` boundary. Refusals are atomic and display a reason; the Operations status describes readiness before a click. Every open request derives its answer from current geometry and access. No cached certificate can survive a path edit.

`ride_opening_status` checks known ride identity, installed availability, current coaster occupied envelopes and train availability, then the existing entrance/queue and exit-network rules. Construction and strict saves already validate installed geometry/clearance. The low-level `set_ride_open` switch remains for fixtures and historical continuation; normal player controls use the checked command. Closing is permitted with damaged access so existing trips can finish and unload after repair.

Mechanics tests remain isolated from access readiness. They can run on a closed, stopped, empty installed coaster before gates are built. No test certificate, new live test mode, force simulation, automatic path repair or auto-closing on access damage is added. Existing saves keep their open state and paid journeys. Saves remain schema/content **19/19** and design/catalog formats **1**.

[RIDE_OPENING_GUIDE.md](RIDE_OPENING_GUIDE.md) explains the player flow. Source tests cover both ride families, absent and damaged access, queue and exit-path repairs, atomic refusal, closing after damage, real Operations/main-button/O input paths, paid journeys and strict save continuation. `write-opening` / `verify-opening` exercise production controls and fresh-process continuation with an occupied paid primary and a closed ready backup for both rides.

The retained empty-dispatch fixture now builds valid access for both rides and uses the maximum permitted fare to keep its existing guests out. It still proves positive empty loading waits and timeout departure without charging a fare, through the checked player opening command. The paid fixture retains ordinary affordable fares. Existing shortcut tests now expect a refusal when reopening an unconnected demonstration coaster.

## Verification

Formatting, strict all-target release Clippy and all **718 release tests (424 game / 294 simulation)** passed. The final production build completed in **50.04 seconds**. The independently verified local executable is **49,907,712 bytes**, SHA256 **`8f1f88a87a2014446bd72cef73c45ba672b470cd162c0e558b8eb132da22007e`**. Runtime source remained unchanged after that build.

Fresh extraction outside the checkout to `C:\Users\strij\AppData\Local\Temp\Parkbound 0.37 opening delivery é`, launched from `C:\Windows`, passed **58 payload hashes / 46 current guides** and exact starter-design checks. All **20 production proof processes** passed: writer/verifier pairs for opening, starters, trains, operations, dispatch, appeal, design transfer, track construction, access and coaster testing. A separate **four-process 0.36 ↔ 0.37 continuation proof** passed with authentic old/new executables and unchanged source slots.

All **eight final 1024 × 680 captures** passed direct inspection: missing entrance, missing exit public path, ready coaster, ready carousel, paid occupied checked-open coaster, connected empty loading wait with an unaffordable fare, completed mechanics test without access, and the retained Pocket Oval design preview. Status, notices, controls and lower rows fit. Controlled system input tests establish the button/O command paths; these captures do not claim physical native input acceptance.

All **291 dependency entries** and five runtime/notice files match 0.36 byte for byte. Receipts are retained in `artifacts/ride-opening-0.37.0/`. The archive is `dist/Parkbound-0.37.0-windows-x64.zip`; its adjacent `.zip.sha256` identifies the current archive after the final guide-only refresh, which uses the same verified executable and a separate fresh extraction.

Exact-source Windows [CI 40182744749](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40182744749), job 121137064063, remains in progress at this local checkpoint. No CI artifact was downloaded or executed.

## Following work

Next expose per-ride recorded ticket income, construction spending, refunds and net cash in Finance, including retained demolished rides and archived history. These are cashflow figures; park valuation and recurring ride running costs need their own implementation.

Native physical input/file-picker, clean-PC and representative hardware acceptance remain separate. No reference PC is selected; this development PC is excluded. The full game, broader RID-05 test modes, multiple coasters, staff, maintenance, research, scenarios, audio and complete management remain unfinished. Preserve the normal map/save caps and all three continuation paths.
