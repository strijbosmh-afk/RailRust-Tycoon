# Project decisions

## 2026-11-19 — Menu theme and fullscreen

- Use shared forest, cream and gold colors throughout the menu system. Retain readable button text when the background changes to gold.
- Interpret the requested menu icon as the top-right park-menu control. Draw three scalable bars and show its label on hover and keyboard focus.
- Create new windows borderless by default. Honor explicit saved display choices and retain decorated-window recovery through F11 and safe-windowed launch.
- Preserve display settings version 1 and park saves 51/51. Keep Google Drive disabled.

## 2026-11-18 — Soundtrack, spatial sound and Drive synchronization

- Disable Google Drive synchronization for Parkbound at the owner's request. Retain local exports and existing destination IDs. Keep the project flag in Git so another PC can respect the choice.
- Write three original instrumental arrangements with percussion, bass, harmony and contrasting sections. Crossfade between tracks and retain the existing independent volume controls.
- Use the real rendering camera and moving ride positions for four nearby sound sources. Water and lift-chain effects follow the ride model and motion state.
- Add AuraVoice's CC0 group cheer, retaining the original recording, converted WAV, provenance and full terms. Keep remaining music and effects original.
- Preserve save schema/content 51/51 and audio settings version 2. Keep the Windows toolchain and runtime dependencies unchanged.

## 2026-11-18 — Parkbound application icon

- Use the user-approved original red coaster, golden loop and green badge for the Windows application identity.
- Retain the transparent PNG master and a reproducible seven-size ICO in Git. Record ImageGen provenance and hashes with the other artwork.
- Embed the ICO and version properties using the existing Windows SDK. Assign large and small native window icons on Bevy's main thread, with dimensions selected for the window DPI.
- Use the installed ICO explicitly for Start menu and optional desktop shortcuts, setup and uninstall. Keep version 0.96.1 on the current Windows test channel and retain save schema/content 51/51.

## 2026-11-18 — Park detail graphics

- Extend the existing original geometry for rides, paths, planting, services and visitor clothing. Reuse the physical transforms and surface clipping.
- Keep visitor appearance deterministic with the existing 48-look cache cycle. Keep static additions in the existing chunk renderer.
- Allow twice the previous close zoom through normal camera controls so players can inspect the new detail.
- Retain park schema/content 51/51 and the Windows test distribution. Use this PC for correctness and visual inspection.

## 2026-11-18 — Windows test distribution

- Build the Phase 12 Windows installer using Inno Setup 6.7.4 and the same verified payload as the portable ZIP.
- Install per user under LocalAppData/Programs with a stable application identity. Keep player data in its existing separate directory.
- Hold a shared named Windows mutex for running game instances. Setup requires them to close before replacing or removing program files.
- Use the installer's recorded file ownership for removal so player-created files and save data survive.
- Record executable, package and compiler hashes. Exercise prior-version upgrade and save/settings continuation using isolated Unicode paths.
- The owner explicitly deferred the open-source license decision while finishing the Windows test version. Keep independent clean-PC, player and hardware acceptance visible in the release record.

## 2026-11-16 — Landscape graphics

- Use original procedural material geometry and the existing oak image. Keep saved terrain, paths, scenery and construction prices unchanged.
- Clip every ground/path/border detail to its authoritative surface plane, including ramps. Preserve open edges at path junctions.
- Distinguish ground-level stone paving from raised timber and blue queue paths. Keep richer material details on maps through 64×64; larger parks retain simpler details and the existing chunk visibility system.
- Remove duplicate solid oak crowns behind the existing texture; vary its green tint and scale.
- Keep this PC excluded from performance acceptance.

## 2026-11-16 — Detailed ride geometry

- Continue the selected classic isometric style with physical geometry for ride details; retain the same world-space depth system in all four camera directions.
- Use open bowls and gondolas, bevelled hulls and small faceted tubes instead of replacing moving attractions with fixed pictures.
- Keep authoritative seat positions, physical ride footprints, paint controls, existing simulation and 51/51 saves.
- Keep added static details in the existing terrain/object chunks. Use bounded segments for moving detail. This PC supplies correctness evidence only.


## 2026-11-16 — Classic graphics direction

- Follow the user's selected detailed classic isometric concept. Keep ride motion and object selection anchored to the existing simulation geometry.
- Embed original title and oak artwork, retaining alpha on the foliage. Bundle both source images and licensed Barlow/Gloock fonts for offline distribution.
- Use live menu controls over the opening art and preserve keyboard navigation through every title action.
- Extend scenery and water details to the 48×48 demo and 64×64 parks. Continue using the existing bounded rendering paths for large-map diagnostics.
- Keep saves at schema/content 51/51 and verify the real demo continuation after presentation changes.

## 2026-11-16 — Phase 11

- Preserve schema/content 51/51 and established prices while collecting reproducible balance data. Actual player sessions and representative hardware determine later tuning.
- Retain historical diagnostic fixtures and add current-content starter, medium mixed, large mixed and active-capacity cases. Large mixed parks use the existing full catalogue district plus a long connected avenue.
- Run beta checks in a new explicit output directory. Compare uninterrupted simulation, repeated save/load and a separate process replay, including backup rotation and independent ledger reconciliation.
- Add an Operations page with targeted inspection using the existing management panels. Derive issues from live simulation state.
- Use one breadth-first distance traversal per ride-choice or staff-job decision. Compare distance results to existing route reconstruction and retain destination scoring and tie ordering.
- Keep this development PC excluded from hardware acceptance. Phase 11's complete gate includes independent display, audio, usability and campaign-balance sessions.

## 2026-11-13 — Phase 9

- Use six campaign scenarios. Retain Meadow and Boardwalk and add Cedar Valley, Research Ridge, Lakeside and Grand Festival, each with distinct goals.
- Provide ten objective families. Operating-profit goals use the current lifetime operating result, including template history. Sales goals count new receipts from scenario start.
- Extend sandbox setup with budget, research availability, arrivals, loans and charging rules.
- Store portable scenarios as versioned `.parkscenario.json` documents containing validated park state, objectives and available content. Keep authoring drafts separate from test play state.
- Preserve the existing public GitHub repository and existing Drive project IDs. The registration rules for creating a new private repository apply when a new project is created.
- Record correctness evidence from this PC and GitHub checks. Exclude this PC from performance recommendations.
- The user requested the full build and codebase in Codex Cloud and explicitly approved a new cloud chat. Preserve this handoff request while completing Phase 9. Cloud provisioning requires recovery after the initial launch stalled and the command-line service reported an unavailable environment.

## 2026-11-16 — Phase 10

- Keep park schema/content 51/51. Store accessibility independently with unique bounded shortcut assignments. Migrate version-1 sound preferences into version 2 with an independent reactions level.
- Guide users through the existing paid construction and access tools. A skipped guide preserves every park field. Completion checks inspect actual public paths, a newly built ride, connected access, completed visits and a successful save.
- Keep physical vehicle and guest positions visible in reduced motion. Freeze decorative weather/water animation, guest walk poses and intro pulsing.
- Retain original procedural assets and the locked dependency set. Add a second score and synthesized staff/guest responses with bounded voices, independent volume, focus fades and retryable Windows PCM output.

## 2026-11-16 — Sunhaven Gardens demo, 0.91.0

- Add an original 48×48 landscaped sandbox accessible directly from the title screen. Its seven attractions span coaster, water, transport and gentle/thrill rides.
- Build rides, paths, queues, scenery and facilities through production purchase methods. Record the $15,000 opening budget and all costs in the ordinary ledger.
- Provide all eight facilities, automatic stock, five staff roles, $1 admission and affordable tickets. Use sunny weather and unlocked research for the demo's opening conditions.
- Start each title-screen demo as a fresh unsaved park. Save it through the existing library without replacing the player's existing parks. Retain save schema/content 51/51.
- Advance five minutes of real simulation with an opening flyer campaign to populate the park. Continue normal arrivals, spending, needs, maintenance and staffing during play.
