# Overlapping surface proof report

Build: **Parkbound 0.3.0**, 20 October 2026. This checkpoint extends Stage 2 / PRF-01 with independently selectable elevated paths and underground track. Verification is in progress; the evidence below must be completed before treating the milestone as accepted.

## Delivered behavior

- The existing 32 × 32 Flat/Terraced park includes 12 elevated path segments and nine track segments. The original path crosses the hill at height 8; the perpendicular track stays at height 0 with two steps of occupied clearance.
- Every segment has a stable identity separate from its terrain tile. The inspector shows its kind, identity, tile, base height, clearance, and burial state.
- **U** or **Underground** switches between normal terrain and a cutaway. The cutaway removes terrain and decorative scenery, revealing the track over a muted context grid. That grid is deliberately not selectable.
- Normal viewing excludes buried track from rendering, picking, and outlines. A selected buried object remains identified in the inspector with an instruction to reveal it. An elevated path can still occlude track in the cutaway.
- **T** retains the same object identities and elevations. Flat exposes the track at ground level; Terraced buries it again. Terrain materials, simulation time, and selection remain intact.
- Shared geometry drives object rendering, picking, and selection. Camera fitting includes elevated objects.
- Simulation placement checks bounds, height limits, terrain intersections, and occupied vertical intervals before changing state or allocating an identity. Invalid terrain changes and partially intersecting track are rejected atomically.

These fixed fixtures demonstrate surfaces and clearance. They are not player construction tools, connected walkable paths, operating rides, or a coaster editor. No guests, finances, persistent settings, park saves, or audio are implemented.

## Implementation contract

`park_sim` remains independent of Bevy. `SurfaceId` distinguishes terrain coordinates from `ObjectId`; removed object IDs are not reused. `SurfaceObject` records its kind, tile, axis, height, and clearance. Rendering entities and depth values remain presentation details.

Occupied space is conservatively the whole tile over the half-open range `[height, height + clearance)`. Adjacent ranges may touch. Paths must be on or above terrain. Track may be on/above terrain or fully buried, but may not straddle the terrain surface. Tunnel excavation, supporting structures, ownership, and prices are outside this proof.

`try_apply_terrain_profile` validates the complete proposed change before committing. Normal application controls use this fallible API and keep the previous profile on rejection. `set_terrain_height` also validates existing objects. Fixture installation is atomic and idempotent.

The renderer uses shallow path decks and rail beds within their validated clearance. Their visible tops sit slightly above the simulation base to avoid coplanar fighting with ground surfaces. Cosmetic support columns are not added without a matching occupied-space model.

Visibility is shared by scene meshes, hit testing, and outlines. Underground mode has no selectable terrain surfaces. Hidden valid selections persist; removed or invalid references clear safely. UI pointer consumption and terrain-before-picking ordering remain in place.

## Verification record

The baseline source was `dc832fa`. Its [Windows CI run 39381924927](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39381924927) passed, but does not verify this new implementation.

The current development host reports Windows 11 Enterprise x64 (10.0.26200), Intel UHD Graphics 630 driver 31.0.101.2115, and approximately 8 GB RAM. Rust 1.93.1 and its pinned components are installed, and `cargo fetch --locked` passed. The Visual Studio C++ Build Tools installer could not obtain elevation, so compilation and packaging will use the existing hosted Windows workflow; downloaded binaries will be checked on this host.

| Check | Current evidence |
| --- | --- |
| Simulation regressions | Added; execution pending. |
| Geometry and input regressions | Added; execution pending. |
| Formatting and linting | Local `cargo fmt --all --check` and `git diff --check` passed; Clippy is pending on Windows CI. |
| Windows release build and package | Pending on the exact implementation commit. |
| Render captures and visual inspection | Pending. |
| Ordinary keyboard and mouse input | Pending. |
| Physical DPI/multiple displays | Unverified. |
| Clean Windows machine without development tools | Unverified; local-host execution alone does not establish it. |

New regressions cover stable IDs, atomic placement and terrain rejection, touching/overlapping height intervals, valid burial, fixture repeatability, preset preservation, all four rotations, depth agreement, underground visibility, invalid references, logical DPI conversion, mode controls, and camera fitting above flat terrain.

## Reproduce the demonstration

Run the packaged `Parkbound.exe`, inspect the elevated path, press **U**, and inspect the revealed track. Rotate with **Q/E**, pan and zoom, then switch views and terrain with **U/T**. The inspector should retain the same selected object and report its current burial state.

Diagnostic options now include:

```text
parkbound.exe --capture-frame <absolute-output.png>
  --capture-scene overview|selection|path|track|help
  --capture-map flat|terraced
  --capture-view surface|underground
  --capture-rotation 0|1|2|3
  [--capture-small-window]
```

Use a fresh output path for each capture. Inspect Surface and Underground in every rotation, a Flat comparison, path/track selection, hidden selection, and the minimum-window help layout. Engine captures suppress normal input and do not establish native control behavior.

## Acceptance and next work

PRF-01 remains **in progress** until the complete rendered and normal-input overlap demonstration is recorded. FND-02 still needs the remaining native pan/drag and physical display checks. Clean-machine distribution acceptance remains open.

After overlap acceptance, continue PRF-02: height-aware path connectivity and a small seeded crowd that recovers from path deletion. Before moving entities, speed controls must execute complete authoritative simulation steps rather than only incrementing time. Train operation, save/reload continuation, and performance measurement remain later independent proofs.
