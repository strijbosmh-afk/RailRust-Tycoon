# Detailed park graphics — 0.96.0

The additional detail follows the existing saturated classic isometric style.
All new artwork is original Rust geometry and ships in the executable.

## What changes in play

- Ground paths have light bevels, darker joints and small stone variations. Timber decks have visible fixings. Blue queues have narrow contrasting edge strips.
- Flower beds contain stems, paired leaves, shaded five-petal flowers and distinct centers at several heights. Conifers have needle clusters, and planted trees have fern undergrowth.
- Catalogue rides have inlaid paving, slim balusters and globe lamps. Ferris-wheel gondolas have gold medallions and roof finials. Cups have floral decoration that follows their seat orientation. The carousel has colored valance medallions and tassels. Coaster cars have round tires and inset metal hubs that follow their banked pose.
- Facilities have glazing, brass window sills, planted boxes and shaped attendants. Food, drinks and ice cream have recognizable rounded roof signs. Rest benches and the other counters have additional small fittings and products.
- Visitors have stripes, vests, buttons, bags, scarves and sun hats. These details are built into the existing cached appearance templates.
- The regular mouse-wheel and zoom-button controls allow twice the previous close zoom. Reset and fit controls retain their usual entry points.

Existing and new parks use these meshes. Ride colors, saved physical positions,
construction footprints, terrain/path connectivity and simulation state continue
through their established code paths. Park schema/content remains 51/51.

## Implementation

`world/landscape.rs` owns surface-clipped paving and planted flower detail.
`world.rs` adds fern geometry and the clothing details shared by the cached and
independent visitor renderers. Static scenery remains in the chunk system.
`world/service_render.rs` projects facility detail through the existing serving
direction and camera transform. The same faces support inspection and previews.
`world/catalogue_render.rs`, `world/carousel_render.rs` and
`world/train_render.rs` add ride-specific detail in physical park coordinates.

## Validation checkpoint

Before-captures of the landscape, wheel, services, visitors and playable demo are
retained under `artifacts/park-detail/`. Formatting, strict lints, workspace
tests, the release build, after-captures and package verification are in progress.
This PC supplies correctness and visual evidence. Representative performance
and independent player/hardware acceptance remain open.

## Continue on another PC

Branch `codex/park-detail-overhaul` starts at
`886aa7c318b3cd11c399c7b9033aa7586f0ed579`. Shared local checkout is
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Use `INSTALLATIE.md` and the pinned toolchain. Build and verify the Windows ZIP
and installer with the existing packaging scripts. Start **Play Sunhaven
Gardens** from the title screen for the seven-ride playable park.
