# Gameplay checklist

Updated: 21 October 2026 for the 0.13.0 one-step sloped-path checkpoint; original baseline: 19 October 2026. Reference: classic RCT1/RCT2 isometric construction and management. Read with [the development plan](../DEVELOPMENT_PLAN.md), [product decisions](PRODUCT_DECISIONS.md), and current [sloped paths report](SLOPED_PATHS_REPORT.md).

This is the implementation inventory for an original game with complete classic gameplay coverage. It is not a claim that the first executable contains these systems. The first playable park is an intermediate checkpoint; 1.0 requires every agreed entry below and the content audit. Original scenario files, artwork, sounds, historical bugs, and original save formats are not assumed deliverables.

**Tracking rules.** `P` = planned; `I` = in progress; `V` = verified. An entry becomes `V` only when its acceptance example passes in a named build, with evidence recorded in the [foundation report](FOUNDATION_REPORT.md), [terrain report](TERRAIN_REPORT.md), [overlap report](OVERLAP_REPORT.md), [crowd report](CROWD_REPORT.md), [train report](TRAIN_REPORT.md), [save report](SAVE_REPORT.md), [performance report](PERFORMANCE_REPORT.md), [optimization report](OPTIMIZATION_REPORT.md), [large-map/GPU report](LARGE_MAP_GPU_REPORT.md), [static visibility report](STATIC_VISIBILITY_REPORT.md), [path construction report](PATH_CONSTRUCTION_REPORT.md), [terrain editing report](TERRAIN_EDITING_REPORT.md), [sloped paths report](SLOPED_PATHS_REPORT.md), or a later milestone report. Earlier verified IDs retain their named-build evidence; new code still requires regression verification. The stage is the completion milestone, not a restriction on earlier prototypes. Every feature must persist through save/load and work through normal player controls where applicable; version 0.6.0 verifies technical continuation for the existing simulation, with native save-control retests separate.

Technical-proof acceptance is limited to the stated behavior. It does not close later gameplay, persistence, hardware, or unverified control requirements; those gaps remain explicit in the milestone report.

## Foundation and technical proofs

| ID | Stage | State | Intended behavior and observable acceptance |
| --- | --- | --- | --- |
| DEF-01 | 0 | P | Reconcile this inventory against the selected classic baseline; every gap has an ID and owner stage, and expansion-only items remain visibly separate. |
| DEF-02 | 0 | P | Record supported Windows versions, an agreed representative PC (none selected; this development PC excluded), original visual direction, content targets, and naming decision; unmeasured performance stays labelled a target. |
| FND-01 | 1 | V | A Rust Windows build opens an isometric tile map and closes cleanly; the packaged executable runs outside the source folder. Reverified in 0.13.0 with 28 payload hashes/nineteen guides, relocated 3,600-tick slope continuation/backup/paused load, and a byte-identical minimum-window slope-built capture from C:\Windows. Native construction, height/shape controls, F5/F9, and Save/Backup buttons passed. Terrain B/V/buttons and the separate Load button remain unverified. Historical terrain/foundation checks remain in their named reports. Clean-machine compatibility is separate. |
| FND-02 | 1 | I | Pan and zoom the camera and select a map tile; the highlighted tile matches the pointer and selection details match the world coordinate. Native 0.3.0 selection, four rotations, wheel zoom (100% → 113%), and Fit (100%) passed. Native 0.13.0 wheel zoom (100% → 162%) and targeted ramp selection/placement passed. Keyboard panning was not clearly established, right/middle drag was unavailable through the helper, and physical DPI/display checks remain open. |
| FND-03 | 1 | V | Toolbar and status area expose the current tool, surface, simulation time, and pause/speed state; pause stops authoritative ticks and speed changes their rate. Native speed/pause/resume evidence is retained from 0.2.0. In 0.3.0 the inspector, U/T buttons, pause at 02:14, rotation, and help-close behavior passed, supported by current authoritative clock tests; see the overlap report. |
| FND-04 | 1 | V | Simulation rules run without graphics; tests, formatting, linting, and release compilation pass. Reverified in 0.13.0 through 260 release tests (175 game/85 simulation), local rustfmt/strict Clippy/build, and [Windows CI run 39583267705](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39583267705) for exact runtime source `b1d26a04cc1282bb71defcb59557309b79252f1d`. CI passed source checks, all tests, packaging, ordinary/path/terrain/slope continuation, CPU-reporting smoke, and artifact uploads. Earlier results remain in their named reports; later documentation heads have separate CI status. |
| PRF-01 | 2 | V | Render and pick stacked terrain, elevated paths, and underground track; hidden and visible surfaces do not select the wrong height. Verified in 0.3.0 through layered geometry/clearance tests, all-rotation captures, and native inspection of Path #7 and Track #18. U revealed/hid track while retaining identity; Flat exposed the same track; returning to hills hid it and clicking its old projection selected foreground terrain. The packaged proof passed relocation and all 17 payload hashes. FND-02 and clean-machine checks remain separate open gates; see the overlap report. |
| PRF-02 | 2 | V | A seeded crowd reaches destinations and recovers from path deletion; disconnected guests enter a defined lost/recovery behavior without invalid references. Verified in 0.4.0 through 72 tests, eleven inspected captures, and native Remove/Restore buttons plus R. Native deletion preserved twelve guests/time, cleared selection, and showed lost guests; restored Path #37 received fresh ID #47, then resume returned the lost count to zero. Isolated/recovered captures and deterministic full-tick tests cover connectivity and reference invariants. Physical Delete shortcut retesting remains open after helper injection had no effect. The 0.11.0 level-path tool adds player edits, and 0.13.0 adds one-step slopes; complete construction, queues, and guest needs remain later scope. See the crowd report. |
| PRF-03 | 2 | V | A train completes a representative station/lift/drop/brake circuit with stable movement at different display frame rates. Verified for automatic circuit/fixed-update behavior in 0.5.0: 99 tests, whole-Park equivalence over 120 simulated seconds at 30/60/144 Hz update schedules, sixteen inspected captures, and normal launch showing lap 1. Space paused at 01:15. Native O/F/buttons remain unverified after desktop activity interrupted testing; physical hardware frame-rate performance is unmeasured. See the train report for separate input/delivery gaps. Track editing, passengers, real physics, and general ride operations remain later scope. |
| PRF-04 | 2 | V | Save and reload a seeded simulation; continuing the same commands reproduces the authoritative state for the same build. Verified narrowly in 0.6.0 through 131 tests, separate production/extracted-process continuation for 3,600 ticks plus commands, explicit backup/paused load, twelve inspected captures, and package relocation with 20 payload hashes/twelve guides matching. The 0.6.0 native attempt issued no input; 0.11.0 later verified F5/F9, and 0.13.0 verified F5/F9 plus Save/Backup buttons and exact restored-save bytes. The separate Load button and Shift+F9 gesture remain open. The 0.13.0 slice also verifies explicit v1→v2 migration; unrelated future migrations, autosave, future gameplay saves, and overall release acceptance remain open. See the save and sloped paths reports. |
| PRF-05 | 2 | I | Measure the proposed 256 × 256 tile, 5,000-guest, 50-ride workload on an agreed representative PC and approve/revise architecture and targets. No reference PC is selected; this development PC is explicitly excluded. The bounded 0.8 cache/render optimization, 0.9 large-map/GPU diagnostics, and 0.10 static visibility are complete, with named evidence in their reports. Version 0.10 passed 193 tests, 74 wrapper checks, eight exact image pairs outside version text, bidirectional continuation, six GPU profiles, CI, and relocated package checks. Focused stress selection is 13/212 chunks and 149,556/2,251,764 static vertices; this is a final CPU-eligibility snapshot, not actual GPU draws, upload traffic, or an FPS gain. One circuit/train, the 256-guest save cap, normal editable play at 32 × 32, and unselected hardware keep full-target acceptance open. |

## Land, paths, and construction

The 0.11.0 implementation adds level-path placement/demolition, selected-height preview/rejection feedback, guest reconciliation, input conflict/save ordering, and construction continuation diagnostics. All 217 release tests, strict source checks, nine inspected captures, and production construction/cross-version continuation passed. Native P/X, height buttons, F5/F9 and rejection/reset checks passed; source CI and relocated packaging passed with 26 payload hashes/seventeen guides. The bounded level-path milestone is complete. Physical Page Up/Down and other listed native/release gaps remain in the path construction report. BLD-01/02/04 are in progress because their full costs/refunds/queue criteria are not implemented. Version 0.12.0 completes its bounded single-tile raise/lower implementation, with preview, shared clearance, and persistence; BLD-06 remains in progress for the full editor. All 236 local release tests, strict release Clippy, release build, ten inspected captures, and terrain/cross-version continuation passed. Native terrain controls remain unverified after activation/capture errors prevented input. Exact-source CI and package relocation passed, including all 27 payload hashes/eighteen guides and byte-identical terrain rendering; see the terrain editing report. Those results belong to their named historical builds.

Version 0.13.0 completes the bounded slope slice under BLD-08: four one-step rise directions, matching opposing endpoints, guest slope motion, and strict v2 saves with explicit v1 migration. All 260 release tests (175 game/85 simulation), formatting, strict release Clippy, release build, production migration/continuation, and eleven inspected captures passed. Runtime source `b1d26a04cc1282bb71defcb59557309b79252f1d` is pushed in [draft PR #11](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/11), stacked on PR #10/`codex/terrain-editing`. [Exact-source Windows CI run 39583267705](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39583267705) passed source checks, all tests, packaging, ordinary/path/terrain/slope continuation, reporting smoke, and uploads. All 28 payload hashes/nineteen guides matched; relocated slope continuation and byte-identical rendering passed. Native Space, C/shape, wheel zoom, South-ramp placement, +/- buttons, F5, X demolition, Save button, F9 paused reset, and Backup restoration passed, including exact restored-save bytes. PageUp and Shift+F9 automation were unconfirmed; physical Page Up/Down, the separate Load button, Delete, terrain/ride controls, pan/drag/DPI, and clean-PC checks remain open. BLD-08 remains in progress for broader bridges/tunnels/ownership/economics. Earlier counts remain historical.

| ID | Stage | State | Intended behavior and observable acceptance |
| --- | --- | --- | --- |
| BLD-01 | 3 | I | Placement preview reports the exact charge and rejection reason; an invalid placement changes neither world nor cash. |
| BLD-02 | 3 | I | Build, connect, and remove level paths and ride queues; visitors traverse junctions and join only the intended ride queue. |
| BLD-03 | 3 | P | Place a ride, entrance, exit, shop, and park entrance; unreachable or incomplete facilities cannot silently operate as valid destinations. |
| BLD-04 | 3 | I | Demolish a placed object with a visible refund/cost; affected guests and facilities recover without disappearing passengers or dangling references. |
| BLD-05 | 6 | P | Buy land or construction rights; ownership and allowed vertical space control construction and demolition consistently. |
| BLD-06 | 6 | I | Raise/lower terrain, form supported slopes, and change surfaces/edges using brush tools; preview the full cost before committing. The 0.12.0 slice covers only single visible Grass/Sand columns, +/-1 within 0–32; Water/Underground edits are blocked. All 236 local release tests, strict release Clippy/build, ten inspected captures, and continuation passed. Native terrain controls remain unverified after activation/capture errors; exact-source CI and package relocation passed with 27 payload hashes/eighteen guides. Brushes, slopes, material editing, and costs remain open. |
| BLD-07 | 6 | P | Add and remove water at valid heights; water, terrain, and ride clearance rules produce understandable accepted/rejected results. |
| BLD-08 | 6 | I | Build sloped/elevated paths, bridges, and tunnels; guests connect only matching heights and structures obey shared clearance rules. The completed 0.13.0 slice provides four one-step slopes, no side entry, endpoint-matched routes, conservative clearance, and explicit v1→v2 save migration. All 260 release tests, strict source checks, eleven inspected captures, production migration/continuation, native construction/save checks, exact-source CI, and relocated-package checks passed. Full bridges/tunnels/ownership/economics and the listed native/release gaps remain open. |
| BLD-09 | 6 | P | Place walls, fences, small/large scenery, vegetation, and signs; rotate, recolor where supported, and remove them with correct costs and boundaries. |
| BLD-10 | 6 | P | Place benches, bins, and path additions on compatible edges; guests/staff use reachable objects and path edits update them safely. |
| BLD-11 | 7 | P | Rotate through four isometric views and toggle underground/height/visibility aids; placement and picking remain correct in each view. |

## Ride operations and coaster design

| ID | Stage | State | Intended behavior and observable acceptance |
| --- | --- | --- | --- |
| RID-01 | 3 | P | Build one original flat ride, set its price, open/close it, board queued guests, complete a cycle, and release guests through its exit. |
| RID-02 | 4 | P | Edit track incrementally using compatible stations, curves, slopes, transitions, and banking; connection failures identify the affected endpoint. |
| RID-03 | 4 | P | Add lifts and brakes and tune supported settings; trains accelerate/decelerate according to track, gravity, and operating equipment. |
| RID-04 | 4 | P | Configure train count, cars, dispatch waits, and loading policy within each ride's limits; passengers, seats, and trains stay accounted for. |
| RID-05 | 4 | P | Separate closed/test/open states; unfinished track, invalid stations, inadequate clearance, or missing access prevent opening with a reason. |
| RID-06 | 4 | P | Run a full test, show speed/height/force telemetry, and calculate excitement, intensity, and nausea from the tested layout; editing invalidates stale results. |
| RID-07 | 4 | P | Ratings influence guest ride choice alongside preferences, money, queue, and price; extreme layouts produce visibly different demand. |
| RID-08 | 4 | P | Save an original coaster design and place it elsewhere with previewed cost and footprint; incompatible terrain/ownership is rejected safely. |
| RID-09 | 6 | P | Add launches, inversions, reversals, special track pieces, and block brakes only to capable families; illegal combinations cannot be constructed. |
| RID-10 | 6 | P | Operate supported continuous, shuttle, launched, synchronized, and block-section modes; dispatch respects occupancy and route-specific safety rules. |
| RID-11 | 6 | P | Set supported ride colors, vehicle colors, names, music, and operating parameters; changes remain visible after reloading. |
| RID-12 | 6 | P | Report capacity, queue time, popularity, satisfaction, age, reliability, income, and costs; values reconcile with underlying events. |

## Mechanical ride families

These are behavior families, not final catalogue counts or a promise to reuse original ride names. Each family needs at least one original validated definition and a recorded catalogue audit before Stage 6 closes. Additional classic variants discovered in that audit become explicit checklist entries; expansion variants need a separate inclusion decision.

| ID | Stage | State | Intended behavior and observable acceptance |
| --- | --- | --- | --- |
| FAM-01 | 3 | P | Cyclic gentle flat ride: a complete loading, operating, and unloading cycle runs with configurable timing and price. |
| FAM-02 | 6 | P | Thrill flat rides: rotation, swinging, and inversion cycles use the appropriate capacity, motion, and ratings. |
| FAM-03 | 6 | P | Vertical attractions: tower/drop/launch behavior respects height, travel bounds, loading, and operating mode. |
| FAM-04 | 6 | P | Walking attractions: maze and course-based attractions route guests through their internal layout and return them to the exit. |
| FAM-05 | 6 | P | Guided gentle rides and powered coasters: vehicles follow player-built routes with correct propulsion, spacing, and stations. |
| FAM-06 | 6 | P | Transport: rail and suspended/cable-style families provide appropriate station stops and passenger transfers across reachable stations. |
| FAM-07 | 6 | P | Competitive circuits: kart-style racing completes laps, determines results, and unloads without vehicles blocking subsequent races. |
| FAM-08 | 6 | P | Boat attractions: free-roaming and guided boats remain in their allowed water/route and return passengers to a valid station. |
| FAM-09 | 6 | P | Flume/rapids and water-coaster families: water sections, drops, lifts, spacing, and splash-related ratings behave as defined. |
| FAM-10 | 4 | P | Conventional coaster: a gravity-driven seated train completes a custom station/lift/drop/brake circuit and carries guests. |
| FAM-11 | 6 | P | Wooden/side-friction/bobsled-style families: track and vehicle constraints differ appropriately and unsafe tests cannot silently pass. |
| FAM-12 | 6 | P | Steel looping, large-drop, and compact/wild-mouse families: each exposes its supported geometry and operating limits. |
| FAM-13 | 6 | P | Suspended, inverted, stand-up, flying, and multi-orientation families: vehicle orientation, clearance, and transitions match the authored family. |
| FAM-14 | 6 | P | Spinning and reversing families: vehicle rotation or reversal changes motion, forces, and ratings while retaining valid track attachment. |
| FAM-15 | 6 | P | Shuttle, impulse, and reverse-freefall families: forward/reverse travel and launch/return behavior complete the configured operating cycle. |

## Guests, employees, and management

| ID | Stage | State | Intended behavior and observable acceptance |
| --- | --- | --- | --- |
| GST-01 | 3 | P | Guests arrive through the entrance, pay applicable admission, choose reachable attractions, queue, board, exit, and eventually leave. |
| GST-02 | 3 | P | Guests carry money, make purchases, and react to unaffordable prices; every payment reconciles with park revenue. |
| GST-03 | 5 | P | Hunger, thirst, toilet need, energy, happiness, and nausea change with time and activities; appropriate services/rest improve the affected state. |
| GST-04 | 5 | P | Ride preferences, thoughts, recent experiences, and price/value judgments affect choices; inspection explains a guest's current action and concerns. |
| GST-05 | 5 | P | Guests abandon excessive queues, get lost, litter, vomit, and seek assistance/rest as applicable; counters and visible world effects agree. |
| GST-06 | 6 | P | Crowds respond to scenery, rain/shelter, incidents, and park conditions; reactions influence measurable happiness and demand. |
| STF-01 | 5 | P | Hire/fire and inspect handymen, mechanics, security, and entertainers; wages and headcount update correctly. |
| STF-02 | 5 | P | Set staff patrol areas and enabled duties; tasks are chosen within reachable permitted areas, and uncovered work remains visible. |
| STF-03 | 5 | P | Handymen clean litter/vomit, empty bins, and perform configured grounds work; cleaning restores the affected condition. |
| STF-04 | 5 | P | Mechanics inspect, respond to breakdowns, reach service access, repair, and return rides to operation; unreachable work is reported. |
| STF-05 | 6 | P | Security responds to vandalism and entertainers improve nearby guest experience; their effects are observable in incidents and guest states. |
| MGT-01 | 3 | P | Cash tracks construction, demolition, admission, ride/shop sales, and operating costs in integer minor units; the ledger balances. |
| MGT-02 | 5 | P | Operate food, drink, merchandise, toilets, and supporting facilities; prices, supplies/costs, capacity, and guest needs affect transactions. |
| MGT-03 | 5 | P | Choose permitted admission/ride charging and close/open the park; scenario restrictions prevent disallowed settings. |
| MGT-04 | 5 | P | Borrow/repay within scenario limits and accrue interest/wages/running costs; reports match the actual balance over a simulated month. |
| MGT-05 | 5 | P | Fund and prioritize research; elapsed funded work unlocks defined content and reports completion. |
| MGT-06 | 5 | P | Launch permitted advertising campaigns for a stated price/duration; guest arrivals or preferences change and effects expire. |
| MGT-07 | 5 | P | Inspect finance history, park value, company value, park rating, guest totals, and ride profitability with explained calculations. |
| MGT-08 | 6 | P | Reliability, wear, inspections, breakdown types, and repairs influence downtime/costs; supported safety failures have defined accident consequences. |
| MGT-09 | 6 | P | Weather transitions affect comfort, demand, and operations as specified; scenario climate and saves preserve weather state. |
| MGT-10 | 6 | P | Vandalism, accidents, and park awards/penalties have visible causes, notifications, and bounded management consequences. |

## Scenarios, interface, and release

| ID | Stage | State | Intended behavior and observable acceptance |
| --- | --- | --- | --- |
| MOD-01 | 7 | P | Play original scenarios with guest, rating, value, income/profit, and ride-related objectives; deadlines and completion/failure trigger correctly. |
| MOD-02 | 7 | P | Complete an original campaign with recorded progression and scenario restrictions; success persists across restarts. |
| MOD-03 | 7 | P | Start sandbox with chosen funds/research/settings; all included content can be explored without campaign unlocks. |
| MOD-04 | 7 | P | Create terrain manually or from a seed, assign land/rights, climate, finances, available content, and objectives; export/reload/play the custom scenario. |
| MOD-05 | 7 | P | Browse, rename, remove, and place saved track designs with previews and validation; missing/incompatible definitions produce useful messages. |
| UI-01 | 3 | P | Inspect rides, guests, facilities, and tiles and operate the basic park entirely through visible controls. |
| UI-02 | 7 | P | Management windows, map overlays, lists, sorting/filtering, search, and notifications locate the affected guest/ride/place and expose relevant actions. |
| UI-03 | 7 | P | Tutorial and contextual help teach construction, queue/access, coaster tests, finances, staff, and saving using the shipped controls. |
| UI-04 | 8 | P | Controls can be remapped; readable scaling, keyboard interaction, and non-color-only status indicators pass the agreed accessibility checks. |
| UI-05 | 8 | P | Music, ambient sounds, ride effects, and alerts have separate adjustable levels and persist in settings; original assets have recorded provenance. |
| SAV-01 | 3 | P | Save a playable park, exit, reload, and continue; world, money, guests, rides, clock, and random state restore correctly. |
| SAV-02 | 7 | P | Autosave, backup rotation, schema/content version checks, and supported migrations work; corrupt/unsupported files never overwrite the player's original. |
| QA-01 | 8 | P | Long mixed-park simulations and repeated construction/save cycles pass invariants, balance checks, performance targets, and release-blocker review. |
| REL-01 | 1 | I | Produce a versioned ZIP containing executable and required resources, launch instructions, and known limitations. |
| REL-02 | 9 | P | A clean supported Windows system installs/unpacks and plays offline without development tools, an account, or original RCT files. |
| REL-03 | 9 | P | Installer and ZIP upgrades preserve saves/settings; standard-user use, spaces/non-ASCII paths, and uninstall behavior pass verification. |
| REL-04 | 9 | P | Publish the free full-function download with checksums, credits/notices, license decisions, support instructions, and save-compatibility notes. |

## Scope decisions still required

- **Catalogue audit:** confirm the complete classic ride/track/operating-mode inventory against the intended RCT1/RCT2 reference. The mechanical families above establish implementation coverage; exact ride variants, piece counts, scenery themes, and scenario counts remain undecided.
- **Expansions:** catalogue expansion-specific content separately, including Added Attractions/Corkscrew Follies, Loopy Landscapes, Wacky Worlds, and Time Twister references. Inclusion is undecided; no expansion catalogue is silently promised or silently treated as complete.
- **Compatibility:** importing original RCT parks, scenarios, designs, or saves is undecided and unnecessary for the standalone original-content target.
- **Later extensions:** multiplayer, mobile/console versions, and a scripting API are outside the current single-player Windows plan unless the user expands scope.

## Milestone evidence

Each report should record the build/version, completed IDs, player demonstration, relevant automated checks, Windows/hardware used, measured performance where relevant, known limitations, and next IDs. Packaging success on the development PC is not evidence of clean-machine compatibility. Update states after evidence exists; never mark a whole family complete from a single shared-system prototype.
