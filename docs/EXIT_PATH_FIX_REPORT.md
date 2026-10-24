# Exit path connection fix — 0.34.1

Version **0.34.1 is locally verified and packaged**. It addresses the reported difficulty connecting the coaster exit. Runtime source is `a9217be3e366033a9f04adc0576b64d5f67db01d` on `codex/exit-path-connection`, in [draft PR #33](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/33), based on verified 0.34.0 checkpoint `c7a3534b24805308594844f011126f9a8b8bef52` / PR #32. No PRs have been merged.

## What was wrong

A green exit gate is only accepted beside a station at its deck height, facing the station. Once placed, the remaining connection is a public path from the gate's outward edge to the public network used by the queue. The screenshot alone does not establish the player's exact saved topology or version.

The path helper had a reproducible failure in the demonstration coaster: with its queue connected and an exit landing at tile 9,9, height 8, it returned no suggested public path. A legal route exists eastward and then under the raised lift back to the public network. The old search required every step to reduce distance; it rejected the initial sideways steps needed for that route.

The revised helper searches a bounded legal route and prepares its next tile. It retains ordinary collision, height and queue connection rules. It also searches for an exposed, clickable part of the tile when the coaster covers its center, and caches the result. If no part is visible, the panel explains that the view must be rotated instead of arming an invisible ghost. Covered centers still obey normal obstruction rules. The checklist separately reports entrance-to-ride, queue, exit-to-ride, outward path and return route. A missing outward tile includes its coordinates and height. The route search is cached while the Access panel is open, rather than repeated on every simulation tick.

## Player steps

1. Open **Rides [Y]**, select the coaster and choose **Access**.
2. If **Exit to ride: connected** is shown, the gate is already attached to the station.
3. Use **Prepare suggested public path**. Move to an exposed edge of the highlighted tile, away from the track, then click once to build it. Reopen **Y → Access** for the next tile until the return route is connected.
4. When building manually, use a brown public path from the green gate's outward arrow, at the same deck height. Join it to the main public paths. A path ending beside a track or on a different height does not form a connection.

Suggestions prepare one tile; they do not build a whole route automatically. The bounded search may still require manual construction for very large or obstructed routes.

## Compatibility and verification

Save schema/content remains **18/18** and track design format remains **1**. No saved simulation fields, guest choice rules, construction prices or dependencies are changed. Diagnostic runs use separate temporary profiles and never write to the player's saves.

All **697 release tests passed (414 game / 283 simulation)**, including the final guidance wording. Formatting and strict all-target release Clippy passed. Regressions cover ten existing walkway lengths, blocked routes, height changes, foreign queue isolation, an explicitly track-obscured tile center, all four camera rotations through generated buttons and the pointer system, and plan cache invalidation. The original failing reproduction is retained as `repro-before.log`.

The final production build passed in **44.54 seconds**. Its executable is **49,778,176 bytes**, SHA-256 **`f478992de8071c3c9c0f0319fb461fe81dcb554ec797df4bdca7c11037b581e6`**. Source receipts are `tests-delivery.log`, `clippy-final.log` and `build-delivery.log` under `artifacts/exit-path-0.34.1/`.

Fresh extraction to `C:\Users\strij\AppData\Local\Temp\Parkbound 0.34.1 exit delivery é`, launched from `C:\Windows`, passed **50 payload hashes / 41 current guides**, ten separate production proof processes (access, journey, operations, dispatch and track construction writer/verifier pairs), and six minimum-window captures. The extended access proof repairs both a single exit landing and a longer dead-end walkway using prepared path tools, saves/reloads after each tile, completes real guest journeys and reproduces the result after a process restart. Original save slots remain unchanged. `package-delivery-verification.log` retains the receipt.

Four additional processes verified **0.34.0 → 0.34.1 and 0.34.1 → 0.34.0 operations continuation**, preserving the source slots and both executable hashes. `compatibility-delivery.log` identifies the complete receipt directory. The 291 dependency entries and five native/runtime/notice files are unchanged from 0.34.0.

The six inspected 1024 × 680 captures are `exit-path-missing`, `exit-path-disconnected`, `exit-path-prepared`, `access-ready`, `access-unavailable` and `help`. The final prepared-path hint fits its three visible lines; the other five final captures match directly inspected preliminary captures byte for byte. The preliminary executable `5867750f…` and its clipped hint are superseded by the final executable above. Capture proof does not establish physical mouse input acceptance.

The functional checkpoint `2bd8d6f26de2ea3708affbfc25031beadd54ad9f` passed all **35 Windows CI steps**, including 697 tests, in [run 40117560902](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40117560902), job 120941939668. The final source changes only the hint wording to fit the minimum window and passed the complete local checks again. Its separate exact-source [CI run 40121394869](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40121394869), job 120953498168, is still in progress at this delivery checkpoint. No CI artifact was downloaded or executed; the delivered executable is the independently verified local build.

The download is `dist/Parkbound-0.34.1-windows-x64.zip`. Its adjacent `.zip.sha256` identifies the current archive. A final documentation refresh keeps the exact verified executable and rechecks all payloads and current guides in a new extraction; its receipt is `package-final-verification.log`. Earlier 0.34.0 and older packages are retained.

Native physical mouse/keyboard acceptance and clean-PC checks are separate from automated input tests and controlled captures. This development PC remains excluded as a representative performance reference. The wider roadmap remains open; train composition is the next planned milestone after this repair.
