# Landscape graphics — 0.94.0

Terrain, paths and planting follow the detailed classic isometric style of the rides.

- Grass has quieter irregular turf patches and fine blades instead of the repeated bright oval spots.
- Sand has shallow ripples and small mineral grains. Exposed terrain has a turf lip and layered soil/stone markings.
- Ground-level public paths use staggered warm stone paving. Raised paths and slopes use timber with fine grain. Queue paths retain their blue surface and clear gold dashed markings.
- Curbs appear only at the closed sides of a path. Connections at corners and intersections remain open.
- Flower beds have pale stone edging, dark soil, leaves and denser coordinated blooms.
- Conifers have layered, irregular crowns and needle highlights. Existing oak artwork has a range of greens and proportions. Duplicate solid foliage behind the oak image has been removed.
- Water edges have subtle bank colors and small stones, while water highlights remain clipped to the water surface.

These are presentation changes. Terrain heights, path connectivity, construction prices, scenery identities and saved parks keep their existing behavior. Save schema/content is 51/51.

## Play and inspect

Extract `dist/Parkbound-0.94.0-windows-x64.zip` after verification and launch `Parkbound.exe`. Choose **Play Sunhaven Gardens** to explore the changes. Existing parks also use the new graphics. Rotate with Q/E and use the normal zoom, path and terrain tools.

The isolated `--capture-scene landscape` diagnostic builds real raised terrain, a sandy pond, stone intersections, sloped timber bridges, blue queues, trees and flower beds. It does not alter player saves. Combine with `--capture-rotation 0` through `3`, `--capture-detail` and an absolute `--capture-frame` PNG path.

## Resume on another PC

Shared local checkout: `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.

Branch `codex/landscape-graphics` starts at `f20a568bf0f896bda4aaf63fbdf25a961fc009d9`. Keep the existing repository/Drive registration and use the toolchain in `INSTALLATIE.md`. No new dependencies, external art or downloads are needed.

Material code lives in `world/landscape.rs`; shoreline detail in `world/atmosphere.rs`; integration and tree trunks in `world.rs`; oak tint and size variation in `visual_style.rs`. Static details stay in the existing chunk system. Rich ground and cliff detail is limited to maps through 64×64; larger maps keep a simpler treatment.

Run formatting, strict Clippy, the game tests, release build, material/height/rotation captures, and the extracted-package demo continuation. Read `LANDSCAPE_GRAPHICS_VERIFICATION.md` for the completed checks. This PC is excluded from performance acceptance.
