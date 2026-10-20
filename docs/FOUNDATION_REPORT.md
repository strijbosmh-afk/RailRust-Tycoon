# Foundation build report

Historical 0.1.0 checkpoint. The current 0.2.0 implementation and verification are recorded in [the terrain report](TERRAIN_REPORT.md).

Build: Parkbound 0.1.0, 19 October 2026. Parkbound is a temporary working title. This is the first executable foundation, not a completed park management game.

## Delivered behavior

- A 32 by 32 isometric site with grass, a lake, a shore, decorative trees, and an entrance arch. Geometry and colors are original and generated inside the application.
- Four camera orientations, zoom, camera panning, a fit command, and a grid toggle.
- Tile selection and a terrain inspector. Rendering and picking share the same projection, including all four rotations.
- A toolbar, simulation timer, pause/resume, three simulation speeds, and an in-game controls panel.
- A separate Rust simulation library that runs without Bevy or a window.
- A Windows packaging script that collects native runtime dependencies, dependency/font licenses, and SHA-256 checksums. A Windows CI configuration performs checks and builds an artifact; it has not yet run on a hosted repository.

Construction, elevation editing, paths, guests, rides, coaster construction, finances, sound, settings persistence, and park saves are not implemented yet. Each launch starts a fresh session.

## Verification evidence

The development host is Windows 11 Enterprise x64 (10.0.26200), Intel Core i5-9400T, Intel UHD Graphics 630, and approximately 8 GB RAM. Exact specifications are recorded in [product decisions](PRODUCT_DECISIONS.md). These are not minimum requirements.

| Check | Result |
| --- | --- |
| Release compilation | Passed with Rust 1.93.1, Bevy 0.17.3, MSVC, and the locked dependency graph. |
| Automated tests | 16 passed: seven simulation tests, five geometry/picking tests, and four camera/UI-input-state tests. |
| Formatting | Passed. |
| Clippy | Passed for the workspace and all targets with warnings treated as errors. |
| Native startup | The executable created a responsive Windows window titled `Parkbound - Meadow Lake`. |
| Rendered output | Three 1280 by 800 images captured from Bevy's render target were inspected: overview, rotated/zoomed selection, and help. Terrain, selection highlight, inspector, text, and modal layout rendered correctly. Every diagnostic process exited with code 0. |
| Relocated ZIP launch | Passed after extraction outside the repository into a path with spaces and an accented character, with an unrelated working directory. The rotated selection and help captures ran from that extracted executable. |
| Package integrity | All 14 payload file hashes matched the packaged SHA-256 manifest after extraction. |
| Native dependency inspection | The executable imports `VCRUNTIME140.dll`, which is bundled from the Visual Studio redistributable. Its remaining imports are Windows system/UCRT libraries. This does not replace clean-machine testing. |

Tests cover map dimensions and bounds, deterministic initial terrain, clock pause and speed multipliers, saturating time, every tile center in every view orientation, shared tile-edge ownership, invalid picks, grid geometry, camera bounds, selection preservation, and preventing a help-close click from reaching the map.

The desktop automation helper returned an unrelated Codex image when asked to capture the Parkbound window, including after refreshing the window selection. Therefore native mouse/keyboard interaction has not been manually verified. Engine-rendered captures and automated logic tests are recorded separately and must not be mistaken for that manual check.

Checklist acceptance: FND-01 (startup, rendering, diagnostic clean exit, and relocation) and FND-04 (automated checks) are verified. FND-02 and FND-03 are implemented but remain in progress until normal-input checks are recorded. Stage 1 is not fully accepted yet.

## Remaining acceptance checks

- Play through pan, zoom, rotation, selection, grid, pause/speed, help, and close using normal input. Verify right/middle dragging and display scaling in particular.
- Launch the extracted package on a clean Windows machine without Rust or Visual Studio, with networking unavailable, using a standard user account.
- Check the minimum window size and multiple DPI/display configurations. Establish measured frame-rate and memory targets before large-park claims.
- Complete the detailed classic-content audit, expansion scope, public name, source-license decision, and minimum Windows/hardware requirements.

The build is unsigned and has not been published. No original RollerCoaster Tycoon files are required or bundled.

## Next implementation slice

Begin Stage 2 with height-aware terrain and picking, then prove path connectivity and guest navigation on a small test park. Add the first representative train circuit and save/reload determinism before scaling to the proposed large-park benchmark. Each proof has its own acceptance ID in [the gameplay checklist](GAMEPLAY_CHECKLIST.md).
