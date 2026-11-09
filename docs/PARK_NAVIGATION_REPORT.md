# Phase 2 checkpoint: grouped park navigation and responsive panels (0.73.0)

This checkpoint follows the display-scale work in [draft PR #73](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/73). It provides an in-park Explore route through the roadmap's Build, Manage, Inspect, Park and Help/Settings groups. Existing direct shortcuts remain. Display settings opens directly from Explore and returns through Pause to the park. The routes reuse established build tools and management panels; no park save schema or simulation rule changes are required.

The Guest, Service, Ride and Track Design dialogs now share the capped scrollable layout. Guest header controls wrap, while Ride and Design columns reflow when the logical window is narrow. Wheel input is routed to the open dialog. The modal remains an input boundary for map actions. A deterministic `--capture-panel` option makes the same panel layouts reviewable at multiple resolutions and scale choices.

## Verification

Formatting, strict release all-target Clippy and the Windows release build passed. The final complete release suite passed **516 game and 389 simulation tests (905 total)**. Diagnostic captures at 1024×680/125% showed Build, Manage, Guests, Services, Rides and Track Designs. The initial compact Track Designs capture exposed that its preview appeared before its title and Close control; the form now comes first, and the corrected 1024×680 and capped 200% captures show its title, Close, search and design controls. The 1920×1080/150% Ride and Track Designs captures keep side-by-side content; 1366×768/100%, 2560×1440/150%, 3440×1440/Auto and 3840×2160/200% group/panel captures remain on-screen and readable. The executable SHA-256 is `de322b9443b067f40e4b55e09a4e1edd4f7a3bdb5f44cdca3fb68a1deccf2045`.

An isolated native input attempt was stopped because the Windows computer-use helper timed out twice and then timed out after a reset. The isolated game processes were closed. Native F2 clicks, wheel movement in the new dialogs and mixed-DPI pointer accuracy are therefore **unverified** in this checkpoint. The final extracted package proof is recorded in the resume handoff. This development PC is not a performance reference.

## Remaining Phase 2 gate

The full roadmap display matrix still needs native checks across a real mixed-DPI monitor move and 100%, 150% and 200% Windows display scaling. Remaining fixed controls within complex panels need follow-up where captures or native input reveal crowding. A separate target Windows PC is required for acceptance; screenshot checks alone do not establish pointer accuracy after a DPI move.
