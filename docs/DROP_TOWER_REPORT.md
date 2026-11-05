# Beacon Drop stage report — 0.63.0

## Scope

Beacon Drop is an original, player-built vertical flat ride in the existing 3×3 footprint and queue/passenger system. It has a separate $420 construction charge, $210 paid-body demolition refund, $0.95 operating cost per 900 ticks, six seats, 72-point static challenge, ride-specific reviews and 18-point completion nausea. The carriage lifts, pauses, drops and brakes using integer positions derived from saved ride progress. Detailed guide rails, striped platform, crown and moving chairs share the same footprint used for previews and selection.

New normal launches begin with flat terrain, a matching Flat control and build height 0. The existing demonstration structures remain available at their authored elevations. Loading a saved park keeps its saved terrain; it is not flattened.

Strict park save schema/content advances to **33/33**. Version 32 reads without transformation; a tower cannot be smuggled into a version-32 file. The new production writer/verifier pair requires paid visits on both a carousel and a tower, saves a paid tower with its carriage raised, and compares an exact 3,600-tick continuation in a separate process while preserving the original primary file. Windows CI runs the pair.

## Acceptance

All **853 release tests (487 game / 366 simulation)** passed. Formatting, strict release all-target Clippy and the Windows release build passed. Executable SHA-256: **`ef3107add85dd9d54e48409cd6ef4c665cf702ac44aea9efe7310116086d7bba`**. A fresh Unicode/space extraction in `artifacts/drop-tower-0.63.0/first proof é/` verified **110 payload hashes and 98 current guides**. Four writer/verifier pairs passed in the extracted executable: drop tower, swing, patrol and wellness. The drop tower pair compared exactly 3,600 future ticks from a raised paid carriage and left the original primary save unchanged.

The 1024×680 `drop-tower-map`, `drop-tower-rides` and `flat-start` captures were directly inspected. The tower is visible at ordinary play zoom; the ride panel's three build buttons and details fit; flat start shows level ground, Flat terrain and path build height 0. Capture SHA-256 values: `04dcc6d18559d655fab2b9a41b8900424a2013237f9fbdead6be8f40004fdd20`, `d6e52e9ded076148be6970b0c354a05f928e904bb18f6af3e5c5e29888135dc5` and `503c0662a646fff9feb39dad19120a80a9a44e1dbddccba24151e3c66b770db9`. The unsigned local ZIP is `dist/Parkbound-0.63.0-windows-x64.zip`; refresh and recheck after guide changes. This development PC is excluded as a performance reference. Native controls, clean target-PC acceptance and taller/custom tower modes remain future work.
