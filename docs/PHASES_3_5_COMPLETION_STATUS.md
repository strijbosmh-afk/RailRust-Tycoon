# Phase 3–5 completion status (0.84.0)

**Phases 3, 4 and 5 meet their published exit gates in 0.84.0.** The delivered graphics, construction and coaster systems are checked against [FULL_GAME_ROADMAP.md](FULL_GAME_ROADMAP.md). The remaining full-game catalogue, simulation, campaign and distribution work continues in later phases.

Source is in [PR #76](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/76), stacked on the 0.74.0 checkpoint.

## Delivered player features

- One to three independently moving trains per coaster, whole-train block occupancy, station depot dispatch, block signals, per-train passenger seats and fares, safe closing and service. Eight independent coasters are supported.
- Smooth one-step rise/descent transitions and a 360-degree heartline roll, with shared draft, installed, train and passenger geometry. Banked curves now bank the rail bed and vehicles. Older banked parks migrate train progress to the revised centerline.
- Isolated speed, height and approximate vertical/lateral force telemetry, with existing layout ratings. Format-2 design exchange covers the new pieces, while older-piece designs retain format 1.


- Eight independent coaster records, each with its own saved construction draft, track, train, car count, gates, prices, guests, paint, name and finances. New coaster starts at height zero. Next coaster and the ride list target the intended workspace. Rendering and sound handle operating coasters independently.

- Saved Day/Sunset/Night lighting and Clear/Rain presentation, with original lamps and cliff-boundary tunnel portals.
- Logical Windows keyboard shortcuts repair Ctrl+Z on AZERTY layouts. Native checks cover path/terrain undo, saving and flat 128×128 sandbox creation.
- A complete 128×128 simulation playthrough covers land purchase, terrain, slopes, queues, guests, ticket revenue, reload continuation and one-time demolition refunds.

- Large Sandbox maps now start with a central 32×32 owned area and a narrow owned route to the west border. Unowned ground has muted color and the boundary is visible. The Build panel buys one adjacent tile for $5; building, scenery and terrain editing reject unowned land. Ownership persists in strict save schema/content 40/40. Existing 39/39 parks migrate with all their land owned.
- Build offers paid square brushes with radii 0–2 for Raise, Lower, Grass, Sand and Water. Hover uses the same planning and price as commit. A failed brush leaves all tiles, objects, money and identifiers unchanged. Water is level-zero terrain, cannot submerge ground objects, and permits a raised bridge. A new ground path on water is refused. The old terrain preset shortcut is restricted to an untouched empty, fully owned park.
- The track editor adds banked left/right curves and a launch straight. Geometry and clearance use the existing closed-circuit validator; they have construction prices, saved piece kinds and distinct rendering. Train movement allows greater speed on a banked curve and accelerates on launch. A coaster test includes those pieces in the speed and intensity estimate.
- Finance records land purchases and landscaping as construction. Archived receipts retain cash conservation and save validation. The new features have no third-party artwork or audio dependency.
- Paths and queues can now be fully buried below dry ground. The same half-open clearance rule rejects pieces that cross the terrain surface; Underground view permits placement and selection, while Surface view hides buried paths and their guests. Terrain lowering exposes the same path and ID. The read-only access planner may suggest a valid tunnel without claiming the ride is connected yet.
- Large-park limits rise together: 2,048 paths, 6,144 objects, 1,024 scenery objects, 512 saved guests, 256 active arrivals and a 16 MiB save file. A 128×128 test builds past the former path and scenery limits, checks path connectivity, reloads, demolishes, and reloads again. The 32×32 map remains a small fixture.
- Player-built trees and flower beds now cost $2.50 and $1.50, with a 50% refund. A selected public path accepts a $1 bench; passing guests regain energy, and a removed bench or its demolished path refunds $0.50. Free older scenery retains a zero-cost basis. Construction and refunds enter the finance ledger and recorded park value, including archived receipt checks.
- Selected public paths now accept $2 signs and $3 fences alongside benches and bins. Their original geometry follows the path height and visibility; each refunds half its paid cost on direct removal or path demolition. These additions are visual and do not alter guest navigation.
- The track editor now offers a $35 camelback hill. It has matching level ports and a one-step internal crest, uses the same sampled geometry for draft, installed rails and train position, reserves three steps of clearance, changes ascent/descent speed, and appears in the isolated test metrics. Its paid basis, demolition refund, design thumbnail and schema-43 save continuation are covered. This is a special hill, not an inversion or a G-force simulation.
- Paused park edits now have 16 bounded undo/redo snapshots in the active session. The left sidebar and Ctrl+Z/Ctrl+Y restore an exact park snapshot, including tiles, objects, guests, identifiers, cash and finance history. A new edit clears redo; resuming time or changing parks clears both stacks. Draft track history remains separate while its editor is active. Save schema stays 43/43 because history is not serialized.

- Player-built public paths and queue pieces now have paid basis: $1/$1.50 for flat/sloped public paths and $2/$2.50 for flat/sloped queues. Demolition returns half the path price plus any separately paid furniture refund. Pre-44 and fixture paths stay free. Preview checks both geometry and available cash before allocation. Strict schema/content 44/44 carries paid IDs, prices and totals; finance entries and archived receipts reconcile with cash.

## Save and behavior contract

- Current park save schema/content is **47/47**. Schema 45 introduced independent coaster workspaces, schema 46 introduced fleets, and schema 47 introduces transitions/inversions and banked geometry migration. Earlier parks gain an empty collection of additional coaster workspaces. Current saves require it, and older schemas cannot claim populated independent records. Each circuit, draft, train and paid track owner is validated. Schema 43 parks load with free existing paths and can add newly paid paths. Schema 42 parks retain their signs and fences. A camelback in a schema-42 or older draft, circuit or paid track record is rejected. Schema 41 parks load with their paid scenery and benches and no signs/fences; current files require both new arrays. Schema 40 parks load with no paid decoration records. Older files cannot claim decoration receipts or ownership before their introduced versions. The version-39 reader still defaults missing per-tile ownership to owned. Existing older migrations remain supported.
- A land purchase quotes and charges $5 before marking its tile owned. A terrain brush quotes changed tiles, validates the entire footprint, checks funds, then applies all changes and records a single receipt. Price per changed tile: height $1, grass/sand $0.50, water $2.
- Buildable water has a clearance rule: a ground object cannot be placed at height 0 or 1 on water; a path or track at height 2 or more still needs ordinary collision and terrain validation. Save validation rejects objects on unowned land.
- Buried public and queue paths use the existing path identity, slope, connectivity and save fields. The tunnel feature introduced no new fields in schema **40/40**; the current save is **47/47** for the subsequent furniture, track and fleet features. The larger bounds and session edit history are not serialized fields; paid path basis is serialized in schema 44. No old file is rewritten merely by opening it.

## Exit-gate audit

| Phase | Status | Exit-gate evidence |
| --- | --- | --- |
| 3 Graphics | Complete | Original visual bible and eleven-family provenance manifest. Shared per-face depth and picking tests. The 0.82 four-rotation minimum/HD/QHD/4K matrix is supplemented by thirteen inspected 0.84 captures covering both coasters, inverted vehicles, block controls, all rotations and the same resolution range. |
| 4 Construction | Complete | Editable large maps, ownership, paid terrain/water brushes, raised/underground paths, queues, furniture, scenery and safe history. The 128×128 empty-park playthrough builds terrain, routes and ride access, earns guest revenue, reloads and demolishes with conserved money and references. |
| 5 Coasters | Complete | Independent coasters, three-train block operation, all planned piece classes, tests and force estimates, wear/shutdown/service, design exchange and compatibility. The custom lifecycle builds, rejects and corrects layouts, tests, runs paid guests, closes/services, reloads and exports/rebuilds a rotated design in a different park. |


The development PC is explicitly **not** a performance reference. The phase gates describe the delivered scope. Later phases expand the full-game catalogue and simulation.

## Current verification

The 0.84 release checks passed **533 game and 423 simulation tests, 956 total**, strict all-target Clippy, formatting, the eleven-family asset validator and the Windows build. The strengthened coaster acceptance tests include rebuilding a rotated design in a different park. Thirteen production captures were inspected directly. The fresh-process coaster writer/verifier preserves two paid coasters, a three-train fleet, passengers and 900 subsequent ticks. Fresh package extraction passed 137 payload hashes, 124 current guides and six independent coaster, startup and track-construction writer/verifier processes. Evidence is under `artifacts/phase-3-5-0.84.0/`.

The final manual Windows interaction check was stopped with the physical Escape key before inspection. Current button and keyboard flows are covered by the automated game tests. Earlier native checks retain their named-build scope. The development PC is excluded from performance references.

## Earlier checkpoint verification

The 0.83.0 release suite passed **531 game and 416 simulation tests (947 total)**, including two paid coasters with independent guests, fares, car counts, movement, save continuation, closed editing, demolition and faulted-remnant ownership. Legacy schema comparison fixtures explicitly account for the new empty workspace collection. Logs are under `artifacts/phase-3-5-0.83.0/`.

The 0.82.0 suite passed **529 game and 413 simulation release tests (942 total)**. Formatting, asset validation and the release build passed. Native Windows checks passed Enter-to-create, paid path placement, logical-keyboard undo/redo, terrain raise/undo, saving, persisted Night/Rain and flat 128×128 sandbox creation. Sixteen directly inspected production captures cover four rotations at 1024×680/100%, 1920×1080/125% Night, 2560×1440/150% Sunset/Rain and 3840×2160/200% Day. Evidence is under `artifacts/phase-3-5-0.82.0/`. This PC remains excluded from performance references.

The 0.81.0 paid-path checkpoint passed **525 game and 412 simulation release tests (937 total)**. Its new tests cover paid public/queue construction, atomic rejection, a one-time half refund after reload, schema-43 free-path migration, strict schema-44 field presence, and archived path receipts. Existing player-input and migration cases were updated to expect the new prices. Formatting, original-asset validation, strict all-target lint, the Windows build, inspected 1024×680 captures and four fresh-extraction save pairs passed; hashes and paths are in the resume guide. This advances Phase 4 without closing its full gate.

The 0.80.0 paused-edit checkpoint passed **525 game and 408 simulation release tests (933 total)**. The added tests cover exact cash and ledger restoration for a paid build, button-triggered undo/redo in the real update order, redo invalidation after a new branch, and clearing history when simulation resumes. Formatting and original-asset validation passed. Save schema remains 43/43. Strict all-target lint, the Windows release build, an inspected 1024×680 capture and fresh extracted-package save proofs also passed; hashes and paths are recorded in the handoff. This remains a Phase 4 construction checkpoint; the phase gates above are open.

The 0.79.0 camelback checkpoint passed **522 game and 408 simulation release tests (930 total)**. After a validator-order fix, all 408 simulation tests and 45 track-focused game tests passed again. Focused checks cover geometry, price, clearance, isolated test metrics, live train/save continuation, strict schema-42 rejection, four-rotation capture preparation and player-input placement. Strict release all-target Clippy, formatting and original-asset validation passed. The final executable and package details are recorded in [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md). This advances Phase 5 but does not close any Phase 3–5 exit gate.

The final 0.79.0 executable SHA-256 is `ad43919e4261bb7ec25cdb4cdd8a545e62abd979391b3b383bb25f2fe9769a62`. The 1024×680 and 1920×1080/rotation-2 camelback captures were inspected directly, including the full editor in HD. Package validation is recorded in the handoff after extraction. Native pointer and mixed-DPI acceptance remain open.

The first 0.79.0 ZIP passed **137 payload hashes and 124 guides** from a fresh Unicode/space extraction, the track-construction writer/verifier pair, and an extracted `track-camelback` capture identical to the inspected minimum-window PNG. The final guide refresh and ZIP checksum are retained in the resume guide and local `dist/` sidecar.

The 0.78.0 release suite passed **522 game and 406 simulation tests (928 total)**. Its focused additions cover combined bench/sign/fence charges and one-time path demolition refunds, 41→42 migration with strict new fields, and archived mixed furniture receipts. The 1024×680 furniture capture, strict build and portable ZIP are checked separately below in the resume guide. These tests do not establish a representative performance target or the full phase gates.

After a lint-only condition cleanup, strict release all-target Clippy and the final Windows build passed. The 1024×680 and 1920×1080/150% furniture captures were inspected: the minimum Build panel scrolls to the new buttons, and the HD view shows Sign/Fence in full. Final executable SHA-256: `637daa42ad23ea85dd6a6b80684a4db8cb26c8b884bec25b9906bb43f36bca1e`. Package validation follows in the handoff.

The first 0.78.0 ZIP extracted with **137 matching payload hashes and 124 guides**, passed the front and scenery save writer/verifier pairs as four independent executable processes, and produced a scenery capture identical to the inspected minimum-window PNG. The final guide refresh and archive checksum are retained in the resume guide and local `dist/` sidecar. Native input and mixed-DPI acceptance remain open.

The 0.77.0 release suite passed **522 game and 403 simulation tests (925 total)**. New checks cover paid scenery hover/commit/refund, bench energy, one-time path/scenery refunds, save migration and strict record rejection, and archived decoration receipts. This is local correctness evidence, not a performance comparison or completion of the Phase 3–5 gates.

After the full suite, a 1024×680 scenery capture revealed a stale “free” footer hint. The wording was corrected and the final Windows release executable rebuilt. Formatting and strict release all-target Clippy passed. The final capture shows the $1.50 flower price and reachable Bench control. Executable SHA-256: `c62a1eaf1f3be30080c1ee9c2977cd0cf28c584265982a1376c5d5348d7523f8`. Package verification is recorded in the resume guide after extraction. Native mixed-DPI checks remain open.

The first 0.77.0 ZIP extracted with **137 matching payload hashes and 124 guides**, passed the front and scenery save writer/verifier pairs as four independent executable processes, and produced a scenery capture identical to the inspected local PNG. The final guide refresh and archive checksum are recorded in the resume guide and local `dist/` sidecar. Native pointer and mixed-DPI acceptance remains open.

The 0.76.0 release test suite passed **522 game and 398 simulation tests (920 total)** before a later label-only correction in the Build menu. New checks cover underground build-tool placement, four-rotation tunnel picking, connected buried paths across a cliff, save/reload/exposure, the larger capacity bounds and 128×128 construction/reload/demolition. After that correction, formatting, strict release all-target Clippy, the Windows build and all 20 visual captures passed. The original-asset manifest validator passed. Package validation is recorded in the handoff. These checks do not establish the three full roadmap exit gates or a hardware performance target.

The Windows 0.76.0 executable SHA-256 is `f1b8e9a64acc24909f41c3cdb2c71332521be7f80d1af530303437f989d4330d`. A fresh Unicode/space extraction verified 137 payload hashes and 124 current guides, passed three independent save writer/verifier pairs (`front`, `coaster-testing`, `scenery`), and captured `overview` and `terrain-underground`. The latter was inspected directly and its short Brush label no longer overlaps adjacent buttons. The package and final checksum are retained locally under `dist/`; the proof is under `artifacts/phase-3-5-0.76.0/`. Native pointer and mixed-DPI checks remain open.

## Continue from another PC

Use [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md) for the exact shared OneDrive path, Git branch, source and save transfer, toolchain, package verification, and latest check results. Continue with Phase 6, the full ride and facility catalogue. Preserve the completed graphics, construction and coaster contracts, original-asset provenance and save compatibility.
