# Parkbound graphics, large sandbox and coaster test guide (0.74.0)

From **New Park**, choose Sandbox and click **Map** to cycle 32, 64, 128 and 256 tiles per side. The 32-tile choice retains the familiar flat starter paths, guests and coaster. Larger maps start empty with flat heights, $1,000 cash and the existing lake/shore materials. Click **Landscape** to cycle Flat or rolling hills from seeds 1–3. Challenge parks remain the fixed 32-tile scenario layout. Press Enter on New Park to create the selected park; the Create action remains the initial keyboard focus.

On an empty map, build a public path, set a visitor entry on a path, then enable arrivals. Build access, queues, facilities and rides as usual. The larger map and its edited tile heights are saved in the park library. The camera allows proportionally closer zoom and wider panning on larger maps; **R** restores the whole-map view.

For coaster engineering, close and empty the installed coaster, open **Rides [Y] → Test**, and run one isolated lap. The report now lists the three fastest track pieces by piece number, kind, tile and speed. It uses the same motion rules as the live train but is a derived estimate. Changing the track or operating settings invalidates the old result. The test leaves guests, the park clock, finances and save bytes unchanged.

Guest clothing now has 48 deterministic appearances, including occasional caps. This art is generated from guest IDs and adds no save fields. The [visual bible](VISUAL_BIBLE.md) and bundled `assets/manifest.json` record the original art rules and shipped asset families.

The full Phase 3–5 roadmap gates remain open. In particular, map ownership, water tools, terrain brushes, building previews and undo; more coaster piece types, multiple independent coasters/trains and breakdowns; and full native display/rotation checks are future work. The 256-tile choice is supported by save and edit tests, but this development PC is not a performance benchmark.
