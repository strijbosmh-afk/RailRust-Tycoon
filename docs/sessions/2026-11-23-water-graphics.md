# Water graphics

- Project `railrust-tycoon`
- Session `water-graphics-2026-11-23` on `MICHIEL-BUILD`, 23 November 2026 UTC
- Branch `codex/right-click-settings`, baseline `d687c61cf402015c6adfe98272665c524e115568`

The landscape renderer uses a deeper teal for open water. Ripples vary in
position and length, with shorter highlight crests. A lighter strip follows
land-facing edges and uses the existing water-surface clipping and depth.
The change uses the existing Windows and Rust build requirements.

The Windows release build generated the landscape in all four rotations.
The northeast and southwest captures were visually inspected against the
previous northeast capture. Player data was isolated by capture mode.
