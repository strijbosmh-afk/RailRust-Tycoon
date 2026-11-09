# Phase 2 checkpoint report: display and interface scale (0.72.0)

This is the first bounded Phase 2 implementation from the [full-game roadmap](FULL_GAME_ROADMAP.md). It adds saved scale, window mode, connected display choice and recovery; a shared capped scrollable dialog; responsive park chrome; and scale-aware camera/picking. Park save schema/content remains **39/39**. The separate display-settings file is version 1 and has a 4 KiB read limit.

The shared dialog is used by the front screens, Help and Finance. The sidebar and Build/Track panels scroll within available height. Finance/Rides buttons remain in the sidebar when compact header shortcuts hide. Scale-aware exclusion rectangles keep the left bar, ride card and Build/Track panel from sending clicks to the map. The full camera target remains available to static terrain visibility, with the same scale used by camera framing.

Auto uses the window's logical width, so Windows DPI is not multiplied twice. Manual values are capped when needed to reserve a map viewport; the requested and effective values are both visible. Fullscreen uses the display's current video mode. A missing display selection falls back to primary; F11 resets preferences, and `--safe-windowed` bypasses a saved fullscreen mode for recovery.

## Verification

The release build passed formatting and strict Clippy checks. The final complete release test suite passed: 508 game tests and 389 simulation tests (897 total). The extracted Windows package passed its 129 payload hash checks and 117-guide inventory. Its runtime `front` writer/verifier, `flat-start` capture and Settings capture passed from a fresh Unicode/space extraction path.

I inspected captures at 1024×680/125%, 1920×1080/150%, 2560×1440/150% and 3840×2160/200%. Title, Finance, Settings and park layouts stayed readable. At 1024×680/125%, Settings needs vertical scrolling to reveal Back; this was verified with the live mouse wheel on a 1280×800 window. Live checks also changed Auto → 100% → 125%, switched Windowed → Borderless → Fullscreen and used F11 to recover Windowed/Auto. The isolated profile ended with `display-settings.json` version 1, `auto`, `windowed`, display 0. These checks do not establish behavior on a second monitor or at 200% Windows DPI. Do not use this development PC for a frame-time target.

The checked executable SHA-256 is `877e095adc8c355142a2b2f3d24a94aa57cea0ccff6209419d6efccafc3e2b68`. The final ZIP checksum is in the adjacent `.sha256` sidecar after the package is refreshed with this report.

## Remaining Phase 2 work

Convert the remaining Guests, Services, Rides, track design and other management panels to the shared layout; group park navigation; test and repair every row of the roadmap's display matrix, including mixed-DPI monitor moves, 4K and 200% Windows scaling. A clean target Windows PC remains a separate acceptance gate.
