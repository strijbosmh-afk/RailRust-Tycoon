# Product decisions

Updated: 20 October 2026 for the 0.4.0 path and crowd proof; original planning baseline: 19 October 2026. This records confirmed direction separately from implementation defaults and unresolved decisions. The [gameplay checklist](GAMEPLAY_CHECKLIST.md) is the feature acceptance register; [the development plan](../DEVELOPMENT_PLAN.md) defines milestones; [the crowd report](CROWD_REPORT.md) records current verification.

## Confirmed by the user

| Decision | Direction |
| --- | --- |
| Gameplay reference | Classic RCT1/RCT2 isometric park construction and management. |
| Implementation language | Rust. |
| Platform | Windows only. |
| Business model | Free distribution with all functionality included. |
| Delivery process | Work in stages, retaining playable milestones. Stage 2's overlap proof is verified in 0.3.0 and path-connectivity/crowd-recovery proof in 0.4.0. The train circuit is the next proof; remaining shortcut, foundation, and distribution gaps stay explicit. |
| Source repository | Save the full source and a cross-PC handoff guide in the user's dedicated repository, [strijbosmh-afk/RailRust-Tycoon](https://github.com/strijbosmh-afk/RailRust-Tycoon). The repository is public. This does not select a source-code license. |
| Final scope | Fully functional classic gameplay, rather than treating the first small park as a finished game. Coverage is tracked explicitly in the checklist. |

## Implementation defaults for this milestone

These choices let implementation proceed and can be revised when evidence warrants it. They do not imply user approval of a final public name, source license, or minimum hardware specification.

| Decision | Default and rationale |
| --- | --- |
| Working title | **Parkbound**, a temporary original-name proposal. Availability and trademark clearance have not been checked; do not publish under this name without review. Local directory names are workspace labels, not public branding. |
| Play model | Offline single-player with local saves. A downloadable build should remain playable without a service account or an active server. |
| First build target | Windows 11 x64. The development-machine version and actual executable verification belong in each build report; this is not yet a minimum Windows requirement. |
| Engine candidate | Bevy 0.17.3, deliberately pinned for the selected Rust 1.93.1 toolchain and API compatibility; it is not presented as the newest release. Stage 2 proves rendering, selection, crowd simulation, and coaster behavior before large-scale content production. |
| Core boundary | A window-independent Rust simulation owns authoritative state; the game layer handles rendering, input, UI, and audio. Content definitions use stable identifiers. |
| Simulation contract | Fixed ticks, explicit update order, seeded randomness, integer money, and validated atomic commands. Same-build reproducibility is tested before claiming it. |
| Visual direction | Original isometric park art and interface, with four view orientations as the completed target. Simple geometric placeholder art is acceptable for the first executable. |
| First deliverable | A 32 x 32 isometric map, camera movement/zoom, tile selection, toolbar/status, simulation clock, pause/speed controls, and packaged Windows executable. It does not yet promise rides, visitors, terrain editing, or park saves. |
| Retained overlap baseline | Version 0.3.0 established the original 12-segment elevated walkway and nine track tiles. Version 0.4.0 extends the walkway into 37 path nodes at height 8, with track retained at height 0; clearance remains 2. T preserves time, materials, object IDs/heights, and guests. Flat exposes the same track. |
| Path connectivity | Cardinal adjacency at the same base height connects Path objects. Diagonals, track, and stacked different-height paths do not connect. Axis controls decoration; connected deck edges follow actual neighbors. Slopes, queue ownership/access, and ride routing are later rules. |
| Crowd proof | Twelve seeded guests choose reachable path destinations using deterministic BFS and stored SplitMix64 state. Each edge takes 30 ticks; guests dwell for 30 ticks at arrival. All simulation progress uses integers; rendering only presents it. These are demonstration visits, not admission or ride revenue. |
| Removal and recovery | Delete/the Remove path button removes the selected path. Affected route references clear immediately and guests retain a local anchor in Lost state. Recovery stays on the retained tile or cardinal same-height neighbors. R/Restore paths restores missing fixture segments atomically, assigns new IDs, and preserves guests, time, RNG, and surviving IDs. |
| Complete ticks | The fixed update remains 30 Hz. Normal/fast/fastest run one/two/four complete authoritative steps; pause runs none. New moving systems must join the complete step rather than multiplying only the clock. |
| Guest presentation | A separate dynamic procedural mesh follows authoritative path endpoints/progress and shared depth. Lost guests have an orange marker. Guests are not individually selectable and no external sprite assets are used. |
| Terrain height units | `park_sim` stores integer steps and validates a maximum of 32. The current renderer maps one step to 16 world units. This display conversion belongs to presentation; it is not a physical metre or a promised final clearance rule. |
| Selectable identity and visibility | Terrain uses `SurfaceId::Terrain(TilePos)`; each path/track object uses `SurfaceId::Object(ObjectId)`. IDs are stable and never reused. U/the visibility button switches Surface and Underground views. Buried track is excluded from ordinary picking; Underground removes terrain from rendering and picking and uses a non-selectable reference floor. |
| Clearance proof | Objects occupy one tile and a half-open integer interval `[height, height + clearance)`, within 0–32. Touching intervals may stack; overlapping ones are rejected atomically. Paths must be on/above terrain; track must be on/above terrain or fully buried. Terrain edits/presets must preserve these invariants. This does not yet implement land rights, slopes, water-volume rules, or construction costs. |
| PRF-01 acceptance | Verified in 0.3.0 through 51 tests, passing Windows CI, inspected captures across all four rotations, native overlap/preset/visibility controls, and a relocated package with 17 matching payload hashes. This is historical baseline evidence; current changes need regression checks. FND-02 panning/dragging and physical-display checks and clean-machine testing remain open; see the overlap report. |
| PRF-02 acceptance | Verified in 0.4.0 through 72 tests, eleven inspected engine captures, and native Remove/Restore buttons plus R, with preserved time/guest count, fresh restored-path identity, and recovery on resume. Helper-injected Delete had no effect, so the physical shortcut remains an explicit retest despite its passing headless regression. Package/hosted-CI results and earlier foundation gaps are tracked separately in the crowd report. |
| Content ownership | Write original code and use original assets or assets with documented redistribution rights. Keep an asset/dependency provenance register; do not bundle original RCT files. |
| Persistence | Versioned local saves with safe replacement, backups, validation, and supported migrations. User data remains separate from application files. |
| Distribution path | Versioned ZIP builds first; installer and public download page before 1.0. Proposed first public channel: itch.io. |
| Feature availability | All gameplay in the free download; no paid unlocks. Offline operation is part of the release checks. |

## Development reference machine

The implementation session recorded this machine on 19 October 2026. These are the development/test host specifications, **not validated minimum requirements**:

| Component | Recorded value |
| --- | --- |
| Operating system | Windows 11 Enterprise, 64-bit; version 10.0.26200. |
| CPU | Intel Core i5-9400T, 1.80 GHz. |
| Graphics | Intel UHD Graphics 630; driver 31.0.101.2115. |
| Memory | 8,453,365,760 bytes reported (approximately 7.87 GiB; nominal 8 GB). |

Frame rate, memory consumption, startup time, and graphics compatibility still require actual packaged-build measurements. The proposed large-park benchmark below is a later-stage target.

## Pending decisions and their gates

| Decision | Needed by | How to resolve it |
| --- | --- | --- |
| Exact classic reference inventory | Stage 0 review, refined before Stage 6 content lock | Audit rides, track pieces, operating modes, facilities, scenario objectives, and UI behaviors; assign missing items stable checklist IDs. |
| Expansion inclusion | Before creating expansion-specific content | List expansion-specific variants separately and choose which belong to 1.0. |
| Supported Windows versions and minimum hardware | Provisional in Stage 2; final in Stage 8 | Measure packaged builds on a named reference PC and additional clean systems; record CPU/GPU/RAM, display scale, and graphics behavior. |
| Performance targets | Stage 2 | Evaluate the proposed 256 x 256 map, 5,000 guests, 50 rides, and 60 FPS at 1080p; distinguish design targets from measured capabilities. |
| Original ride/scenery/scenario counts | Before Stage 6 content production closes | Map original content definitions to the completed behavioral inventory and set explicit catalogue/campaign counts. |
| Public game name | Before public release assets | Check availability and relevant naming rights; replace the temporary title if needed. |
| Source-code license | Before granting reuse rights or accepting outside contributions | Source publication in the dedicated GitHub repository is authorized. No project source license has been selected; decide independently from free binary distribution and asset licenses. |
| Third-party code/asset adoption | Before each import | Record the source, exact license, redistribution obligations, and attribution. Existing reference projects are not assumed to permit unrestricted reuse. |
| Code signing and installer technology | Before Stage 9 | Evaluate distribution experience, cost, maintenance, and supported Windows behavior using concrete candidate builds. |
| Original-file compatibility | Before any compatibility implementation | Decide separately whether original parks/scenarios/designs/saves need import support; no import support is currently promised. |
| Accessibility and input targets | During UI development, verified in Stage 8 | Set supported text/display scales, remapping, keyboard reachability, and non-color-only indicators. |

## Release completion rule

Versions 0.1.0 through 0.4.0 are foundation, terrain, overlap, and crowd demonstrations. The [foundation report](FOUNDATION_REPORT.md), [terrain report](TERRAIN_REPORT.md), [overlap report](OVERLAP_REPORT.md), and [crowd report](CROWD_REPORT.md) separate their evidence. Version 1.0 requires the agreed gameplay inventory, the original content catalogue, usable editors and scenarios, tested saves and upgrades, and a distributable Windows package. A clean-machine launch, save recovery, licensing records, and public-release checks remain open until demonstrated. A successful local build alone does not close them.
