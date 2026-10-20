# Overlapping surface proof report

Build: **Parkbound 0.3.0**, 20 October 2026. This checkpoint completes the Stage 2 / PRF-01 overlapping-surface proof with independently selectable elevated paths and underground track. Remaining foundation input/display and clean-machine checks are recorded separately below.

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

Verified implementation commit: **`0220c7b550228df26996523009fbeef80263dc5e`**. Its [Windows CI run 39406042854](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39406042854) passed formatting, release-profile workspace/all-targets Clippy with warnings denied, all tests, Windows packaging, and artifact upload. Later documentation-only commits retain this implementation; compare the executable hash below when reusing its runtime evidence.

The current development host reports Windows 11 Enterprise x64 (10.0.26200), Intel UHD Graphics 630 driver 31.0.101.2115, and approximately 8 GB RAM. Rust 1.93.1 and its pinned components are installed, and `cargo fetch --locked` passed. The Visual Studio C++ Build Tools installer could not obtain elevation, so compilation and packaging used the existing hosted Windows workflow. The downloaded executable was tested on this host. Local simulation/test type checks and Clippy also passed; they are separate from the hosted release-profile execution.

| Check | Current evidence |
| --- | --- |
| Simulation regressions | **19 passed** in release-profile CI. |
| Geometry and input regressions | **32 passed:** 16 geometry/picking, 12 input-system, and four existing application controls. Total workspace: **51 passed**. |
| Formatting and linting | Local formatting and whitespace checks passed; hosted formatting and release-profile workspace/all-targets Clippy passed with warnings denied. |
| Windows release build and package | Passed for the exact implementation commit. |
| Render captures and visual inspection | **14 diagnostic captures** plus one relocated-package capture inspected; all processes exited with code 0. |
| Ordinary keyboard and mouse input | Object/terrain selection, U/button visibility, T/button presets, four rotations, wheel zoom, Fit, pause, help, and native close verified below. Sustained pan and right/middle dragging remain unverified. |
| Physical DPI/multiple displays | Unverified. |
| Clean Windows machine without development tools | Unverified; local-host execution alone does not establish it. |

New regressions cover stable IDs, atomic placement and terrain rejection, touching/overlapping height intervals, valid burial, fixture repeatability, preset preservation, all four rotations, depth agreement, underground visibility, invalid references, logical DPI conversion, mode controls, and camera fitting above flat terrain.

## Render and normal-input evidence

The diagnostic matrix contains Surface and Underground overview captures in all four rotations, Flat in both modes, and four 1024 × 680 minimum-window captures: selected path, selected underground track, hidden selected track, and help. All were visually inspected. Terrain, path/track orientation, overlap depth, hidden outlines, camera framing, inspector, and controls were consistent. Selection captures intentionally use 135% zoom, so map edges can extend beyond the viewport.

Generated matrix: `artifacts/overlap-0.3.0/`, ignored by Git. The executable contains the capture presets needed to reproduce it. The elevated walkway has no support columns or ground shadow yet; this is a presentation limitation of the proof fixture.

The Windows Computer Use helper initially timed out capturing the window; refreshing the returned window binding succeeded. Subsequent normal-input observations were made in the actual 1280 × 800 game window:

| Action | Observed result |
| --- | --- |
| Click the elevated walkway | Selected Path #7, display tile (15, 17), height 8, clearance 2. |
| Press U | Revealed the track and retained Path #7. |
| Click visible track | Selected Track #18, display tile (14, 18), height 0, clearance 2, Underground. |
| Click Underground button | Hid track and its outline, retained Track #18, and displayed “Buried / U to reveal.” |
| Space, then T | Paused at 02:14; Flat exposed the same Track #18 at height 0 without changing time. |
| E through NE → SE → SW → NW | Retained Track #18, its world tile, and the paused time. |
| Wheel, then Fit button | Zoom changed 100% → 113% → 100%, retaining selection. |
| Click terrain button | Restored hills, buried the track, and retained its identity and 02:14 time. |
| Click the track's former screen location | Selected foreground terrain at display tile (12, 19), height 6; did not select the hidden track. |
| H, then Back to the park | Help opened and closed; terrain selection and paused time were unchanged. |
| Native window close | The game process exited. |

An injected ArrowRight press did not produce a clear visible pan. The helper exposes no right/middle drag gesture. These native checks remain open under FND-02; automated transformed-picking, panning, and logical 2× DPI tests do not establish physical mouse/display behavior.

## Package and relocation evidence

The hosted artifact and inner ZIP matched their SHA-256 checksums. All **17 payload files** matched `SHA256SUMS.txt` in both the workspace extraction and a fresh temporary extraction whose path contains spaces and `é`.

The relocated executable ran with `C:\Windows` as its working directory, rendered the selected underground track in rotation 3, and exited with code 0. That 1280 × 800 image was visually inspected. The original and relocated executables were identical:

```text
5c022763e3d4534816c8f8e3027c48ec3562532a589ba4d329dc6a8cb9bcca9e
```

The local versioned ZIP is `dist/Parkbound-0.3.0-windows-x64.zip`; its adjacent `.sha256` records the checksum of that archive, including its packaged documentation. The tested build is unsigned. A draft pull request and CI artifact are available; no public binary release has been published.

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

PRF-01 is **verified** for 0.3.0: terrain, elevated paths, and underground track render and select consistently, supported by current regression, capture, native-input, and package evidence. This does not close FND-02's remaining native pan/drag and physical display checks, or clean-machine distribution acceptance.

Next is PRF-02: height-aware path connectivity and a small seeded crowd that recovers from path deletion. Before moving entities, speed controls must execute complete authoritative simulation steps rather than only incrementing time. Train operation, save/reload continuation, and performance measurement remain later independent proofs.
