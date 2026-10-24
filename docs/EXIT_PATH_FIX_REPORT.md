# Exit path connection fix — 0.34.1

This hotfix addresses the reported difficulty connecting the coaster exit. It is based on the verified 0.34.0 checkpoint `c7a3534b24805308594844f011126f9a8b8bef52`, on branch `codex/exit-path-connection`. Verification is in progress; the previous verified package remains 0.34.0 until the checks below finish.

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

Pending: release tests, strict Clippy, production build, fresh-process access/journey proofs, minimum-window capture inspection, portable package integrity and relocation checks. Evidence will be retained in `artifacts/exit-path-0.34.1/`.

Native physical mouse/keyboard acceptance and clean-PC checks are separate from automated input tests and controlled captures. This development PC remains excluded as a representative performance reference. The wider roadmap remains open; train composition is the next planned milestone after this repair.
