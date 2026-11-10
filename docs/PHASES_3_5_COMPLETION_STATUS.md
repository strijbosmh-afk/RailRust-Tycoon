# Phase 3–5 completion status (0.78.0)

This is an implementation checkpoint on the road to the Phase 3, 4 and 5 exit gates in [FULL_GAME_ROADMAP.md](FULL_GAME_ROADMAP.md). **None of the three full phase gates is complete yet.** Do not describe this branch as a finished graphics, construction or coaster phase in a release announcement.

Source is in [draft PR #76](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/76), stacked on the 0.74.0 checkpoint.

## Player-visible work in this checkpoint

- Large Sandbox maps now start with a central 32×32 owned area and a narrow owned route to the west border. Unowned ground has muted color and the boundary is visible. The Build panel buys one adjacent tile for $5; building, scenery and terrain editing reject unowned land. Ownership persists in strict save schema/content 40/40. Existing 39/39 parks migrate with all their land owned.
- Build offers paid square brushes with radii 0–2 for Raise, Lower, Grass, Sand and Water. Hover uses the same planning and price as commit. A failed brush leaves all tiles, objects, money and identifiers unchanged. Water is level-zero terrain, cannot submerge ground objects, and permits a raised bridge. A new ground path on water is refused. The old terrain preset shortcut is restricted to an untouched empty, fully owned park.
- The track editor adds banked left/right curves and a launch straight. Geometry and clearance use the existing closed-circuit validator; they have construction prices, saved piece kinds and distinct rendering. Train movement allows greater speed on a banked curve and accelerates on launch. A coaster test includes those pieces in the speed and intensity estimate.
- Finance records land purchases and landscaping as construction. Archived receipts retain cash conservation and save validation. The new features have no third-party artwork or audio dependency.
- Paths and queues can now be fully buried below dry ground. The same half-open clearance rule rejects pieces that cross the terrain surface; Underground view permits placement and selection, while Surface view hides buried paths and their guests. Terrain lowering exposes the same path and ID. The read-only access planner may suggest a valid tunnel without claiming the ride is connected yet.
- Large-park limits rise together: 2,048 paths, 6,144 objects, 1,024 scenery objects, 512 saved guests, 256 active arrivals and a 16 MiB save file. A 128×128 test builds past the former path and scenery limits, checks path connectivity, reloads, demolishes, and reloads again. The 32×32 map remains a small fixture.
- Player-built trees and flower beds now cost $2.50 and $1.50, with a 50% refund. A selected public path accepts a $1 bench; passing guests regain energy, and a removed bench or its demolished path refunds $0.50. Free older scenery retains a zero-cost basis. Construction and refunds enter the finance ledger and recorded park value, including archived receipt checks.
- Selected public paths now accept $2 signs and $3 fences alongside benches and bins. Their original geometry follows the path height and visibility; each refunds half its paid cost on direct removal or path demolition. These additions are visual and do not alter guest navigation.

## Save and behavior contract

- Current park save schema/content is **42/42**. Schema 41 parks load with their paid scenery and benches and no signs/fences; current files require both new arrays. Schema 40 parks load with no paid decoration records. Older files cannot claim decoration receipts or ownership before their introduced versions. The version-39 reader still defaults missing per-tile ownership to owned. Existing older migrations remain supported.
- A land purchase quotes and charges $5 before marking its tile owned. A terrain brush quotes changed tiles, validates the entire footprint, checks funds, then applies all changes and records a single receipt. Price per changed tile: height $1, grass/sand $0.50, water $2.
- Buildable water has a clearance rule: a ground object cannot be placed at height 0 or 1 on water; a path or track at height 2 or more still needs ordinary collision and terrain validation. Save validation rejects objects on unowned land.
- Buried public and queue paths use the existing path identity, slope, connectivity and save fields. The tunnel feature introduced no new fields in schema **40/40**; the current save is **42/42** for later furniture records. The larger bounds are read/write limits, not new serialized fields. No old file is rewritten merely by opening it.

## Exit-gate audit

| Phase | Implemented and retained | Still needed for its roadmap gate |
| --- | --- | --- |
| 3 Graphics | Original visual bible and provenance manifest, procedural terrain/ride/service/guest art, four-rotation depth tests, 48 guest appearances, and a [16-view resolution/rotation screenshot audit](PHASE_3_VISUAL_AUDIT_0_76.md) with four near views. | Richer original art in a representative multi-ride busy park at every scale; consistent near/far ride and service detail; night/weather treatment; native inspection of picking and animation. |
| 4 World | Editable 64/128/256 maps, deterministic starting terrain, land purchase, paid brushes and water, raised paths/bridges and basic tunnels, queues, paid trees/flowers and functional paid benches, signs and fences, raised capacity with a 128×128 reload/demolition stress case, atomic rejection and save/load. | Tunnel portals and wider scenery; consistent cost/refund/undo across all build actions; an end-to-end large-park construction/demolition playthrough with guests and gates. |
| 5 Coasters | One closed coaster, paid track draft/design import-export and test telemetry, station/straight/lift/drop/brake/curve plus bank and launch, dispatch and passenger accounting. | Inversions/transitions/special pieces, multiple independent coasters and trains, block operation, richer force/safety telemetry, breakdown and repair, and complete custom-build → test → operate → save → export → rebuild playthrough. |

The development PC is explicitly **not** a performance reference. The above gaps are product work, not a claim that only testing remains.

## Local verification

The 0.78.0 release suite passed **522 game and 406 simulation tests (928 total)**. Its focused additions cover combined bench/sign/fence charges and one-time path demolition refunds, 41→42 migration with strict new fields, and archived mixed furniture receipts. The 1024×680 furniture capture, strict build and portable ZIP are checked separately below in the resume guide. These tests do not establish a representative performance target or the full phase gates.

After a lint-only condition cleanup, strict release all-target Clippy and the final Windows build passed. The 1024×680 and 1920×1080/150% furniture captures were inspected: the minimum Build panel scrolls to the new buttons, and the HD view shows Sign/Fence in full. Final executable SHA-256: `637daa42ad23ea85dd6a6b80684a4db8cb26c8b884bec25b9906bb43f36bca1e`. Package validation follows in the handoff.

The first 0.78.0 ZIP extracted with **137 matching payload hashes and 124 guides**, passed the front and scenery save writer/verifier pairs as four independent executable processes, and produced a scenery capture identical to the inspected minimum-window PNG. The final guide refresh and archive checksum are retained in the resume guide and local `dist/` sidecar. Native input and mixed-DPI acceptance remain open.

The 0.77.0 release suite passed **522 game and 403 simulation tests (925 total)**. New checks cover paid scenery hover/commit/refund, bench energy, one-time path/scenery refunds, save migration and strict record rejection, and archived decoration receipts. This is local correctness evidence, not a performance comparison or completion of the Phase 3–5 gates.

After the full suite, a 1024×680 scenery capture revealed a stale “free” footer hint. The wording was corrected and the final Windows release executable rebuilt. Formatting and strict release all-target Clippy passed. The final capture shows the $1.50 flower price and reachable Bench control. Executable SHA-256: `c62a1eaf1f3be30080c1ee9c2977cd0cf28c584265982a1376c5d5348d7523f8`. Package verification is recorded in the resume guide after extraction. Native mixed-DPI checks remain open.

The first 0.77.0 ZIP extracted with **137 matching payload hashes and 124 guides**, passed the front and scenery save writer/verifier pairs as four independent executable processes, and produced a scenery capture identical to the inspected local PNG. The final guide refresh and archive checksum are recorded in the resume guide and local `dist/` sidecar. Native pointer and mixed-DPI acceptance remains open.

The 0.76.0 release test suite passed **522 game and 398 simulation tests (920 total)** before a later label-only correction in the Build menu. New checks cover underground build-tool placement, four-rotation tunnel picking, connected buried paths across a cliff, save/reload/exposure, the larger capacity bounds and 128×128 construction/reload/demolition. After that correction, formatting, strict release all-target Clippy, the Windows build and all 20 visual captures passed. The original-asset manifest validator passed. Package validation is recorded in the handoff. These checks do not establish the three full roadmap exit gates or a hardware performance target.

The Windows 0.76.0 executable SHA-256 is `f1b8e9a64acc24909f41c3cdb2c71332521be7f80d1af530303437f989d4330d`. A fresh Unicode/space extraction verified 137 payload hashes and 124 current guides, passed three independent save writer/verifier pairs (`front`, `coaster-testing`, `scenery`), and captured `overview` and `terrain-underground`. The latter was inspected directly and its short Brush label no longer overlaps adjacent buttons. The package and final checksum are retained locally under `dist/`; the proof is under `artifacts/phase-3-5-0.76.0/`. Native pointer and mixed-DPI checks remain open.

## Continue from another PC

Use [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md) for the exact shared OneDrive path, Git branch, source and save transfer, toolchain, package verification, and latest check results. Continue Phase 4 with consistent construction economics, undo and a complete large-park player flow; Phase 5 with the multi-coaster model and safe blocks; and Phase 3 with original art and native visual acceptance. Keep each save migration and player flow reviewable.
