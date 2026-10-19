# Gameplay checklist

Baseline: 19 October 2026. Reference: classic RCT1/RCT2 isometric construction and management. Read with [the development plan](../DEVELOPMENT_PLAN.md) and [product decisions](PRODUCT_DECISIONS.md).

This is the implementation inventory for an original game with complete classic gameplay coverage. It is not a claim that the first executable contains these systems. The first playable park is an intermediate checkpoint; 1.0 requires every agreed entry below and the content audit. Original scenario files, artwork, sounds, historical bugs, and original save formats are not assumed deliverables.

**Tracking rules.** `P` = planned; `I` = in progress; `V` = verified. An entry becomes `V` only when its acceptance example passes in a named build, with evidence recorded in the [foundation report](FOUNDATION_REPORT.md) or a later milestone report. The stage is the completion milestone, not a restriction on earlier prototypes. Every feature must persist through save/load and work through normal player controls where applicable; foundation 0.1.0 does not yet implement persistence.

## Foundation and technical proofs

| ID | Stage | State | Intended behavior and observable acceptance |
| --- | --- | --- | --- |
| DEF-01 | 0 | P | Reconcile this inventory against the selected classic baseline; every gap has an ID and owner stage, and expansion-only items remain visibly separate. |
| DEF-02 | 0 | P | Record supported Windows versions, reference PC, original visual direction, content targets, and naming decision; unmeasured performance stays labelled a target. |
| FND-01 | 1 | V | A Rust Windows build opens an isometric tile map and closes cleanly; the packaged executable runs outside the source folder. Verified in 0.1.0 through native startup, rendered captures, diagnostic exit code 0, and extracted-ZIP relocation. |
| FND-02 | 1 | I | Pan and zoom the camera and select a map tile; the highlighted tile matches the pointer and selection details match the world coordinate. |
| FND-03 | 1 | I | Toolbar and status area expose the current tool, tile, simulation time, and pause/speed state; pause stops authoritative ticks and speed changes their rate. |
| FND-04 | 1 | V | Simulation rules run without graphics; relevant tests, formatting, linting, and release compilation pass. Verified in 0.1.0: 16 tests, formatting, Clippy with warnings denied, and release compilation. |
| PRF-01 | 2 | P | Render and pick stacked terrain, elevated paths, and underground track; hidden and visible surfaces do not select the wrong height. |
| PRF-02 | 2 | P | A seeded crowd reaches destinations and recovers from path deletion; disconnected guests enter a defined lost/recovery behavior without invalid references. |
| PRF-03 | 2 | P | A train completes a representative station/lift/drop/brake circuit with stable movement at different display frame rates. |
| PRF-04 | 2 | P | Save and reload a seeded simulation; continuing the same commands reproduces the authoritative state for the same build. |
| PRF-05 | 2 | P | Measure the proposed 256 x 256 tile, 5,000 guest, 50 ride benchmark on a recorded PC; use results to approve or revise architecture and targets. |

## Land, paths, and construction

| ID | Stage | State | Intended behavior and observable acceptance |
| --- | --- | --- | --- |
| BLD-01 | 3 | P | Placement preview reports the exact charge and rejection reason; an invalid placement changes neither world nor cash. |
| BLD-02 | 3 | P | Build, connect, and remove level paths and ride queues; visitors traverse junctions and join only the intended ride queue. |
| BLD-03 | 3 | P | Place a ride, entrance, exit, shop, and park entrance; unreachable or incomplete facilities cannot silently operate as valid destinations. |
| BLD-04 | 3 | P | Demolish a placed object with a visible refund/cost; affected guests and facilities recover without disappearing passengers or dangling references. |
| BLD-05 | 6 | P | Buy land or construction rights; ownership and allowed vertical space control construction and demolition consistently. |
| BLD-06 | 6 | P | Raise/lower terrain, form supported slopes, and change surfaces/edges using brush tools; preview the full cost before committing. |
| BLD-07 | 6 | P | Add and remove water at valid heights; water, terrain, and ride clearance rules produce understandable accepted/rejected results. |
| BLD-08 | 6 | P | Build sloped/elevated paths, bridges, and tunnels; guests connect only matching heights and structures obey shared clearance rules. |
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
