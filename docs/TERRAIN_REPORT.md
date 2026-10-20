# Terrain proof report

Build: **Parkbound 0.2.0**, 20 October 2026. This is the first terrain slice of Stage 2 / PRF-01. The complete overlapping terrain, elevated-path, and underground-track proof remains in progress.

## Delivered behavior

- The 32 × 32 site starts with an original, repeatable terraced hill: levels 0, 2, 4, and 6. The shore, water, and outer tree belt remain level.
- Press **T** or the terrain button to compare Terraced and Flat presets. Materials, simulation time, and selected tile coordinates are preserved. This is a preset switch; individual terrain editing comes later.
- Top surfaces and exposed front cliffs render with per-vertex depth. Equal-height neighbors do not create internal cliff faces. Scenery and its ground shadows follow the supporting terrain.
- Pointing at a visible top or cliff selects that terrain column. Hidden lower terrain cannot be picked through a foreground cliff. Selection outlines follow the actual surfaces and can be hidden by nearer terrain.
- All four camera rotations use the same surface rules. Fit accounts for elevation, and the inspector displays height in simulation steps.
- Diagnostic capture options now include map profile, rotation, and minimum window size. Capture mode pauses time and ignores ordinary game input and button hover tinting to keep comparison images stable.

No construction tools, path objects, underground tracks, guests, operating rides, finances, saves, or audio are implemented in this checkpoint.

## Implementation contract

`park_sim` remains independent of Bevy. `Park::new` still creates flat terrain. `apply_terrain_profile` changes heights only. `set_terrain_height` validates the coordinate and the inclusive 0–32 height range before changing anything; rejected edits leave the park unchanged. Preset generation uses integer arithmetic, including on rectangular simulation maps.

The current game renderer expects a square park. It converts one height step to 16 world units. Shared top/cliff surfaces drive rendering, picking, and selection. Screen depth derives from rotated ground coordinates on each vertex, rather than a single rank for an entire tall column. Pointer hits interpolate the same surface depth and choose the nearest visible terrain column. `TilePos` remains the selected identity while each tile contains only one terrain column.

Terrain changes are applied before pointer conversion. Rebuilding terrain also invalidates the selection mesh, even when the selected coordinate and rotation have not changed. Landscape and selection meshes opt out of cached frustum bounds because their vertices change. Both use opaque vertex-colored materials and actual depth testing.

The next overlapping-surface slice must introduce stable identity for separately selectable path/track objects and an explicit underground visibility mode. It must extend shared geometry, clearance, rendering, and picking together.

## Verified checks

The development host is the Windows 11 Enterprise / Intel UHD Graphics 630 machine recorded in [product decisions](PRODUCT_DECISIONS.md). Rust 1.93.1, Bevy 0.17.3, the MSVC toolchain, and `Cargo.lock` were used.

| Check | Result |
| --- | --- |
| Automated tests | **34 passed:** 12 simulation, 10 world geometry, 8 input-system integration, and 4 existing application-control tests. |
| Formatting | `cargo fmt --all -- --check` passed. |
| Linting | Release-profile workspace/all-targets Clippy passed with warnings denied. |
| Release executable | Release-profile locked Windows build passed with two compilation jobs. |

Commands:

```powershell
cargo fmt --all -- --check
cargo clippy --release --locked --workspace --all-targets -j 2 -- -D warnings
cargo test --release --locked --workspace -j 2
cargo build --release --locked -p park_game --bin parkbound -j 2
```

The tests cover:

- Repeatable terrain profiles, preserved materials/time/dimensions, small/rectangular/maximum map sizes, valid height extremes, and atomic rejection of invalid edits.
- Flat-map regression, shared edges, all rotations, roof and cliff ownership, hidden low roofs, emitted mesh depth, selection occlusion, finite geometry, and equal-height neighbors.
- Real application input systems with synthetic Bevy input: elevated picking after rotation/pan/zoom at 1× and 2× DPI, visible-cliff picking, UI/help exclusion, wheel handling, focus/cursor-loss drag recovery, and camera fitting up to height 32.
- Terrain button actions preserving selection and time, and selection vertices rebuilding at their new height even when tile identity and rotation are unchanged.

Synthetic input tests do not establish native Windows mouse/keyboard or physical multi-monitor behavior. The following normal-input checks were performed separately.

## Render and native input evidence

Seven engine captures were visually inspected: Terraced overview at rotations 0, 1, 2, and 3 and Flat overview at 1280 × 800, plus selected raised terrain and help at the 1024 × 680 minimum window. Terrain, cliff faces, scenery, inspector, selection, and controls rendered correctly. All capture processes exited with code 0. The selection preset uses 135% zoom, so map edges may intentionally extend beyond the viewport. Fit remains 100%.

Local capture directory: `artifacts/terrain-0.2.0/`. These generated images are ignored by Git and can be reproduced using the commands in the resume guide.

The desktop helper successfully targeted the actual Parkbound window in this session. Normal Windows input was observed:

| Action | Observed result |
| --- | --- |
| Click a raised top | Inspector selected display tile (15, 18), height 6. Display coordinates are one-based. |
| Press T, then click the terrain button | Selected tile stayed (15, 18); height changed 6 → 0 → 6. The map changed with it. |
| Click a visible cliff | Inspector selected display tile (16, 15), height 6; its top/cliff outline followed the terrain column. |
| Press E | View changed NE → SE while retaining display tile (16, 15). |
| Mouse wheel, then Fit button | Zoom changed 100% → 113% → 100%; selected identity stayed unchanged. |
| Press G | Grid switched off visibly. |
| Press 3 | Toolbar showed 4×; simulation time advanced. |
| Press Space, open Help, close with Back to the park | Time remained paused at 02:36 through help; closing did not select the map beneath the button. |
| Click Resume | Time resumed at the previous 4× speed. |
| Click the native window close button | Parkbound's process exited. |

Sustained keyboard panning and right/middle dragging were not established by this native session. The available helper does not expose a right/middle drag gesture. Their application logic, including focus/cursor-loss recovery, has automated coverage. Physical 2× DPI and multi-monitor transitions remain unverified; 2× DPI calculations were tested headlessly. Do not infer those manual results from the captures.

## Acceptance and next step

- PRF-01 remains **I**: terrain heights are implemented; overlapping elevated paths and underground tracks are still outstanding.
- FND-02 remains **I** pending the remaining native pan/drag and display checks. FND-03 is **V** in 0.2.0 from the observed toolbar, timer, pause/resume, and speed behavior plus authoritative simulation-clock tests.
- Clean-machine, offline, standard-user launch without Rust/Visual Studio remains open. Testing an extracted ZIP on this development PC cannot establish it.
- Performance targets, Windows minimum version, physical DPI configurations, public name, source license, signing, and public release decisions remain open.

Next: implement the small elevated-path/underground overlap fixture described in [the resume guide](../RESUME_DEVELOPMENT.md), before crowd navigation. Preserve this terrain baseline and its tests.

## Delivery validation

`scripts/package-windows.ps1 -SkipBuild` produced:

- `dist/Parkbound-0.2.0-windows-x64/Parkbound.exe`
- `dist/Parkbound-0.2.0-windows-x64.zip`
- The ZIP's `.sha256` file and the package's `SHA256SUMS.txt`.

Packaging collected notices for 288 dependencies and the embedded font, Rust standard-library notices, and the imported MSVC runtime DLL from the Visual Studio redistributable. The package includes the updated README, development plan, resume guide, and milestone documents. There were no dependency changes in this milestone.

The ZIP was extracted outside the repository under a temporary folder whose name contains spaces and `é`. All **16 payload hashes** matched its manifest. Its executable ran from an unrelated working directory (`C:\Windows`), rendered the selected raised tile in rotation 3, and exited with code 0. That 1280 × 800 capture was visually inspected. The extracted executable matched the source build.

Verified executable SHA-256:

```text
7550a9d6e660a21b5f7ae7d8c94058b05489a663ad9e903812abc0e1aaaf3903
```

The documentation was then refreshed in the package and the final archive's manifest rechecked. The runtime binary remained the same. The ZIP checksum is recorded in the adjacent `.zip.sha256` file; it is not embedded here because this report is itself packaged.

This is a local unsigned checkpoint, not a published public binary release. The old 0.1.0 package remains available for comparison. Source and documentation belong in the dedicated GitHub repository; generated binaries and captures stay in the ignored local `dist` and `artifacts` folders under the shared OneDrive workspace.

Hosted CI is a separate check: inspect [GitHub Actions](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions) for the exact delivered commit. The earlier successful foundation runs linked in the resume guide do not prove success for this new commit.
