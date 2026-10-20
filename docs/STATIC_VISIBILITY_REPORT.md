# Static terrain chunk visibility — 0.10.0

Date: 20 October 2026. This checkpoint implements bounded static-scene visibility for the existing large diagnostic maps. Work starts from the verified 0.9.0 checkpoint `96518d9` on `codex/static-terrain-visibility`. The integrated contract and diagnostic schema are recorded below; the corrected implementation passed 193 release tests (128 game/65 simulation) and strict release Clippy. Final binary build, source identity, runtime results, CI, and package evidence are pending. Earlier results remain attached to their named reports; they do not verify changed code.

**No reference PC is selected.** The user explicitly excluded this poorly performing development PC as a performance reference. Measurements here are local diagnostics, not minimum specifications, representative FPS, a portable speedup, or a pass/fail result for the proposed 60 FPS target. PRF-05 remains in progress for the full 256 × 256 / 5,000-guest / 50-ride workload.

## Purpose and scope

The completed [0.9.0 diagnostic report](LARGE_MAP_GPU_REPORT.md) observed that large-map views submitted the full static landscape mesh even when the camera focused on a small area: 2,251,764 vertices and indices, representing 117,091,728 logical CPU payload bytes. This motivates reducing unnecessary submitted geometry. It does not establish terrain as the dominant bottleneck or promise an FPS improvement.

The change partitions static geometry into spatial chunks only for maps larger than 32 × 32. The normal 32 × 32 demonstration retains its single legacy mesh. Existing `build_scene` remains the full-scene correctness oracle. Guest and train meshes, deterministic simulation, path caching, player controls, and persistence stay separate.

`world::build_static_chunks(&Park, rotation, grid, mode)` returns `Vec<StaticChunkGeometry>`, each with a `Mesh` and `Rect` bounds. Large-map geometry is partitioned using atomic emission groups: each terrain tile and its details, each scenery object, and each surface object, including a track piece's supports, base, and rails, stay together. Groups are assigned to deterministic 1024 × 512 world-unit buckets by their actual bounds center; oversized groups use an overflow bucket within their layer. Layer keys deterministically order returned chunks as Shadow → Terrain → Scenery → Object. Static assets are created before dynamic meshes at initial allocation. These are construction-order guarantees, not a universal GPU draw-order guarantee after slots shrink or regrow and mesh asset IDs are reused or newly allocated. Each triangle retains its attributes, winding, and depth with local indices; neither triangles nor emission groups are split between buckets or at a viewport edge. Within each group, original triangle order is preserved, keeping support shafts, footings, and rails together. Bounded initial-view comparisons and resource-reuse tests separately verify rendered output and replacement behavior. The complete source mesh is dropped after partitioning, so it is not a duplicate persistent render mesh.

Conservative bounds include every actual used vertex: terrain tops and cliffs, scenery, tree canopies, entrance arch, path decks, static circuit geometry/supports, and decorative shadows. A tile origin or nominal tile footprint alone is insufficient. Rectangular-map rejection remains consistent with `build_scene`.

The separate `StaticLandscape` resource manages render slots. Camera pan, zoom, and window resize select intersecting chunk entities without regenerating static geometry. Unchanged viewport bounds reuse the prior selection. Content, grid, rotation, and Surface/Underground changes rebuild the affected static presentation. Entity and mesh-handle slots are reused; excess slots are removed so a smaller replacement cannot leave old geometry visible. Startup avoids a redundant second geometry build. Same-tick load invalidation and transitions between large diagnostic maps and the small demonstration remain explicit regression cases.

The inclusive world-XY viewport rectangle uses the same camera/window transform as rendering and picking, covers the entire window including areas behind UI, and adds a two-logical-pixel guard. Chunks intersecting this conservative rectangle remain eligible for submission. GUI coverage is not permission to remove world geometry. An unavailable or invalid window/camera selects all chunks conservatively.

## Preserved behavior and boundaries

Normal startup creates the editable 32 × 32 demonstration. Loading still accepts only 32 × 32 parks. Schema 1/content 1, collection limits, `%LOCALAPPDATA%\Parkbound\saves`, backup/lock recovery, authoritative ticks/IDs/RNG, and train/crowd continuation are unchanged in scope. Derived rendering state is not serialized.

The existing `--render-case baseline|save-scale|stress` preview remains isolated. Optional `--render-camera overview|crowd|ride`, `--render-rotation 0|1|2|3`, and `--render-underground` require that mode. Preview camera/inspection/pause/speed/grid/view/Find remain available; terrain/path edits, restoration, train Open/Close, and persistence remain disabled. Captures/profiles additionally suppress ordinary input.

Large fixtures retain one circuit/train and 1,024 paths. Save-scale has 256 guests; Stress has 5,000 and exceeds the unchanged 256-guest save bound. A large diagnostic renderer does not introduce large editable parks, fifty operating rides, construction tools, guest boarding, or expanded save capacity.

## Diagnostic interpretation

The report distinguishes:

| Quantity | Meaning | Does not establish |
| --- | --- | --- |
| Generated static geometry | All chunk vertices/indices and their logical CPU payload bytes | Geometry submitted in a particular view or physical GPU storage |
| `selected_chunks` / selected vertices | Chunks eligible for submission under the conservative CPU camera test, and their geometry totals | Measured GPU draw calls, exact visible pixels, unoccluded objects, or physical display |
| Static rebuild count | Geometry regeneration after relevant state changes | GPU upload byte count or duration |
| Visibility updates | Camera-dependent selection of existing chunks | Mesh regeneration or GPU execution time |
| Guest/train geometry | Unchanged separate dynamic meshes | Static-chunk savings |

`render_work.static_visibility` is a **final snapshot**, not an interval average or a per-frame series. Its counters are cumulative since initialization, including startup and warmup. The generated aggregate remains available at `render_work.meshes.landscape`.

| Field | Contract |
| --- | --- |
| `method` | `conservative_world_xy_aabb` |
| `chunking` | `single_mesh` for maps at most 32 × 32; otherwise `spatial_groups` |
| `bucket_world_size` | `[1024, 512]` |
| `selection_state` | `camera_bounds`, or `unavailable_all_selected` when a valid viewport is unavailable |
| `viewport_world` | `{ "min": [x,y], "max": [x,y] }`, or null when unavailable |
| `viewport_guard_logical_pixels` | `2`; DPI is not applied twice |
| `total_chunks`, `selected_chunks`, `hidden_chunks` | Final counts; selected plus hidden equals total |
| `generated`, `selected` | Each contains `vertices`, `indices`, and `logical_cpu_payload_bytes`; payload is 48 bytes per vertex plus 4 per index |
| `geometry_builds` | Cumulative static geometry rebuilds, including initial construction |
| `visibility_evaluations` | Cumulative selection evaluations after viewport change or static invalidation; unchanged bounds are skipped |
| `visibility_changes` | Cumulative individual chunk visibility toggles, not changed frames or draw calls |

Only static chunks are counted here. Dynamic guest/train and selection meshes remain separate. Partitioning preserves triangles and shared indices within each chunk; a vertex used by different chunks can be repeated, so general future indexed geometry need not retain an identical vertex count even when its triangles are identical.

Retain the 0.9.0 timing distinction: main-loop intervals, main-world CPU hooks, render-span CPU encoding, actual GPU timestamps, and pipeline counters are different evidence. Bevy diagnostics arrive asynchronously through a latest-only receipt window. Receipt frames are not GPU source-frame identities; missed batches and exact warmup membership remain unknown. Never sum nested spans into a frame total. Physical upload bytes/time and actual presentation timing remain unmeasured.

No threshold on this PC decides acceptance. Primary evidence is exact geometry/output equivalence, correct bounds/picking, reduced selected geometry in focused views, stable resource counts, and unchanged authoritative continuation. Any bounded timing collection must retain workload/configuration, source/binary identity, actual window state, raw samples, unsupported outcomes, and method limitations.

## Validation plan

These checks describe required proof; they are not passing results yet.

1. Compare the union of chunk triangles with the full-scene oracle, including attributes/depth and group/layer ordering. Cover grid on/off, all four rotations, Surface/Underground, large terrain edges, and decoration extending beyond its tile.
2. Check conservative bounds from actual vertices, including negative coordinates, high cliffs, long shadows, related support shaft/footing/rail surfaces, and triangles crossing nominal bucket/viewport edges. A partially intersecting triangle must not disappear because its origin lies outside the viewport.
3. Verify camera pan/zoom/resize selects different existing chunks while static build counts and entity/handle allocation remain unchanged. Include minimum window and logical-pixel/DPI transforms; retain geometry behind UI.
4. Change rotation/view/content and replace a large map with the 32-map demonstration and back. Verify regenerated geometry, removed surplus slots, same-tick replacement, and unchanged dynamic guest/train updates.
5. Compare production captures against the full-scene or previous executable at matching complete ticks. Include overview, focused crowd/ride, clipped edges, rotations, Underground, Flat/Terraced, and the ordinary 32-map baseline. Inspect images; process exit alone is insufficient.
6. Verify ordinary save continuation in separate processes, including cross-version checks when compatible, full Park state/commands, explicit backup, and paused loading. Preview diagnostics must not create or replace player saves.
7. Run formatting, release tests, strict Clippy, release build, and hosted Windows CI. Record exact implementation identity and keep later documentation-head CI separate.
8. Package and extract to a fresh path containing spaces and `é`, run from `C:\Windows`, verify every payload hash and bundled guide, and exercise the extracted executable and sibling wrapper. A final guide refresh changes the ZIP hash; its adjacent `.sha256` is authoritative.

## Commands and bounded runs

The existing preview is sufficient to inspect camera-dependent selection:

```powershell
.\target\release\parkbound.exe --render-case stress --render-camera crowd
```

The existing short wrapper invocation requests two frame modes without repeating CPU benchmarks:

```powershell
pwsh -NoProfile -File scripts/measure-performance.ps1 -OutputDirectory 'C:\Parkbound-QA\new-static-run' -SkipCpu -RenderCase stress -RenderCamera crowd -GpuDiagnostics -ProfileSamples 120
```

Use a fresh absolute output directory and bounded runs. Shared frame options cannot be combined with `-SkipFrames`, and `-SkipCpu -SkipFrames` is invalid. A large capture uses the `overview` capture scene with shared render flags, a new absolute `.png` path, and at most 1,800 requested initial ticks. There is no additional oracle CLI required by this contract.

## Evidence register

| Item | Current evidence |
| --- | --- |
| Implementation / PR / CI | Pending; branch `codex/static-terrain-visibility`, starting checkpoint `96518d9` |
| Partition and bounds contract | API and 1024 × 512 atomic-group/layer buckets described above; corrected implementation regression evidence pending |
| Release tests / formatting / strict lint / build | Corrected implementation passed 193 release tests: 128 game / 65 simulation. Strict release Clippy passed; final formatting/build evidence and runtime checks pending |
| PowerShell report validation | 30 new checks plus 44 historical wrapper checks passed; production-report validation remains pending runtime evidence |
| Viewport/resource reuse and replacement | Pending |
| Production images / picking | Pending |
| Generated versus submitted geometry | Pending actual report values |
| Save compatibility / preview isolation | Pending for 0.10.0 |
| Executable hash/size | Pending |
| Notices / package / final guides | Pending; preserve existing notice bytes |
| Relocated runtime / bundled wrapper | Pending |
| Representative PC / full target | No reference PC selected; PRF-05 remains in progress |

After the bounded change is verified, choose further work from remaining correctness and measured-work evidence. Do not automatically extend it to dynamic crowd culling, a new renderer, new map/save limits, or a hardware target. Native Save/Load/Backup, O/F/buttons, physical Delete, pan/drag/display, and clean-PC offline/standard-user gaps remain open unless directly reverified.

## Resume on another PC

The active shared OneDrive checkout remains:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
```

The separately recorded second-PC clone remains:

```text
C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon
```

Read [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md) for toolchain, commands, current review state, and save transfer. Push source and current guides before changing PCs. Use only one active editing/build/Git session in the shared folder and allow OneDrive to finish syncing. GitHub and the source folder do not transfer LocalAppData saves automatically. Preserve historical reports, raw datasets, manifests, and notice files exactly.
