# Phase 3–5 completion status (0.75.0)

This is an implementation checkpoint on the road to the Phase 3, 4 and 5 exit gates in [FULL_GAME_ROADMAP.md](FULL_GAME_ROADMAP.md). **None of the three full phase gates is complete yet.** Do not describe this branch as a finished graphics, construction or coaster phase in a release announcement.

## Player-visible work in this checkpoint

- Large Sandbox maps now start with a central 32×32 owned area and a narrow owned route to the west border. Unowned ground has muted color and the boundary is visible. The Build panel buys one adjacent tile for $5; building, scenery and terrain editing reject unowned land. Ownership persists in strict save schema/content 40/40. Existing 39/39 parks migrate with all their land owned.
- Build offers paid square brushes with radii 0–2 for Raise, Lower, Grass, Sand and Water. Hover uses the same planning and price as commit. A failed brush leaves all tiles, objects, money and identifiers unchanged. Water is level-zero terrain, cannot submerge ground objects, and permits a raised bridge. A new ground path on water is refused. The old terrain preset shortcut is restricted to an untouched empty, fully owned park.
- The track editor adds banked left/right curves and a launch straight. Geometry and clearance use the existing closed-circuit validator; they have construction prices, saved piece kinds and distinct rendering. Train movement allows greater speed on a banked curve and accelerates on launch. A coaster test includes those pieces in the speed and intensity estimate.
- Finance records land purchases and landscaping as construction. Archived receipts retain cash conservation and save validation. The new features have no third-party artwork or audio dependency.

## Save and behavior contract

- Current park save schema/content is **40/40**. The version-39 reader defaults missing per-tile ownership to owned. Version-40 files require an explicit ownership field per tile. Version-39 files cannot claim the new track kinds or land receipts. Existing older migrations remain supported.
- A land purchase quotes and charges $5 before marking its tile owned. A terrain brush quotes changed tiles, validates the entire footprint, checks funds, then applies all changes and records a single receipt. Price per changed tile: height $1, grass/sand $0.50, water $2.
- Buildable water has a clearance rule: a ground object cannot be placed at height 0 or 1 on water; a path or track at height 2 or more still needs ordinary collision and terrain validation. Save validation rejects objects on unowned land.

## Exit-gate audit

| Phase | Implemented and retained | Still needed for its roadmap gate |
| --- | --- | --- |
| 3 Graphics | Original visual bible and provenance manifest, procedural terrain/ride/service/guest art, four-rotation depth tests and captures, 48 guest appearances. | Detailed art acceptance for a representative busy park at every supported scale and rotation; consistent near/far ride and service detail; night/weather treatment; native inspection of picking and animation. |
| 4 World | Editable 64/128/256 maps, deterministic starting terrain, land purchase, paid brushes and water, raised paths/bridges, queues, basic scenery, atomic rejection and save/load. | Path/object/guest/file limits expanded and stressed together; tunnels, benches, signs, fences and wider scenery tools; consistent cost/refund/undo across all build actions; end-to-end large-park construction/demolition playthrough. |
| 5 Coasters | One closed coaster, paid track draft/design import-export and test telemetry, station/straight/lift/drop/brake/curve plus bank and launch, dispatch and passenger accounting. | Inversions/transitions/special pieces, multiple independent coasters and trains, block operation, richer force/safety telemetry, breakdown and repair, and complete custom-build → test → operate → save → export → rebuild playthrough. |

The development PC is explicitly **not** a performance reference. The above gaps are product work, not a claim that only testing remains.

## Local verification

The 0.75.0 release test suite passed **520 game and 396 simulation tests (916 total)**. Formatting, strict release all-target Clippy and original-asset manifest validation passed. Tests include atomic brush quotes/commits, water clearance, land purchase/save continuation, schema-39 ownership migration, archived finance reconciliation, banked/launch track apply/test/save, large-map library continuation and old-save regression. These checks do not establish the three full roadmap exit gates or a hardware performance target.

The Windows executable hash is `a4ce8c001816009ad405d800e64a6f69185635aac44f748c713a6c2ce053da47`. A fresh extraction with a Unicode/space path verified 136 package payloads and 123 current guides and passed separate `write-front`/`verify-front` processes. The final ZIP checksum is in the local `dist/` sidecar, with the retained final proof under `artifacts/phase-3-5-completion/final package proof é/`. Native pointer and mixed-DPI checks remain open.

## Continue from another PC

Use [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md) for the exact shared OneDrive path, Git branch, source and save transfer, toolchain, package verification, and latest check results. Continue Phase 4 with a large-park construction and capacity pass, Phase 5 with the multi-coaster model and safe blocks, and Phase 3 with original art and native visual acceptance. Keep each save migration and player flow reviewable.
