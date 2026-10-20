# Static terrain chunk visibility — 0.10.0

Date: 20 October 2026. The bounded static-scene visibility implementation is complete in source `ff2857e68bfa8217fb85fe3bd107312f385d490e`, on `codex/static-terrain-visibility` from verified 0.9.0 checkpoint `96518d9`. [Draft PR #8](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/8) is stacked on unmerged PR #7. All 193 release tests, strict source checks, eight image comparisons, bidirectional save continuation, six local GPU profiles, [implementation CI run 39440918757](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39440918757), and relocated package execution passed. Earlier results remain attached to their named reports.

**No reference PC is selected.** The user explicitly excluded this poorly performing development PC as a performance reference. Measurements here are local diagnostics, not minimum specifications, representative FPS, a portable speedup, or a pass/fail result for the proposed 60 FPS target. PRF-05 remains in progress for the full 256 × 256 / 5,000-guest / 50-ride workload.

## Purpose and scope

The completed [0.9.0 diagnostic report](LARGE_MAP_GPU_REPORT.md) observed that large-map views submitted the full static landscape mesh even when the camera focused on a small area: 2,251,764 vertices and indices, representing 117,091,728 logical CPU payload bytes. This motivates reducing unnecessary submitted geometry. It does not establish terrain as the dominant bottleneck or promise an FPS improvement.

The change partitions static geometry into spatial chunks only for maps larger than 32 × 32. The normal 32 × 32 demonstration retains its single legacy mesh. Existing `build_scene` remains the full-scene correctness oracle. Guest and train meshes, deterministic simulation, path caching, player controls, and persistence stay separate.

`world::build_static_chunks(&Park, rotation, grid, mode)` returns `Vec<StaticChunkGeometry>`, each with a `Mesh` and `Rect` bounds. Large-map geometry is partitioned using atomic emission groups: each terrain tile and its details, each scenery object, and each surface object, including a track piece's supports, base, and rails, stay together. Groups are assigned to deterministic 1024 × 512 world-unit buckets by their actual bounds center; oversized groups use an overflow bucket within their layer. Layer keys deterministically order returned chunks as Shadow → Terrain → Scenery → Object. Static assets are created before dynamic meshes at initial allocation. These are construction-order guarantees, not a universal GPU draw-order guarantee after slots shrink or regrow and mesh asset IDs are reused or newly allocated. Each triangle retains its attributes, winding, and depth with local indices; neither triangles nor emission groups are split between buckets or at a viewport edge. Within each group, original triangle order is preserved, keeping support shafts, footings, and rails together. Bounded initial-view comparisons and resource-reuse tests separately verify rendered output and replacement behavior. The complete source mesh is dropped after partitioning, so it is not a duplicate persistent render mesh.

Conservative bounds include every actual used vertex: terrain tops and cliffs, scenery, tree canopies, entrance arch, path decks, static circuit geometry/supports, and decorative shadows. A tile origin or nominal tile footprint alone is insufficient. Rectangular-map rejection remains consistent with `build_scene`.

The separate `StaticLandscape` resource manages render slots. Camera pan, zoom, and window resize select intersecting chunk entities without regenerating static geometry. Unchanged viewport bounds reuse the prior selection. Terrain/content, grid, rotation, and Surface/Underground changes still rebuild all static geometry; there is no incremental per-chunk rebuild. Entity and mesh-handle slots are reused; excess slots are removed so a smaller replacement cannot leave old geometry visible. Startup avoids a redundant second geometry build. Same-tick load invalidation and transitions between large diagnostic maps and the small demonstration remain explicit regression cases.

All generated static meshes remain resident, and partitioning temporarily holds the full source mesh. This step reduces camera-selected geometry, not initial construction work, peak memory, or full static-rebuild work. The 256-map overview selects all 212 chunks and may incur extra batching overhead; no overview FPS improvement is claimed.

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

## Validation coverage

The source and runtime checks below support the bounded rendering result. Package relocation/guide verification is recorded separately; ordinary native input and hardware acceptance remain separate gaps.

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

## Observed local results

Six profiles completed with exit code 0 and validated `samples_received` reports: baseline/overview, save-scale/overview, and stress/crowd, each once with AutoVsync and once with AutoNoVsync. Each requested 120 main-loop intervals. All summaries recorded clean source `ff2857e` and the unchanged executable below. Actual windows were 1920 × 1080, visible/focused throughout, with no size/configuration changes, unfocused intervals, or hidden intervals. Occlusion was unknown.

Both modes produced the same static-geometry snapshot for each case:

| Fixture / camera | Selected / total chunks | Hidden chunks | Generated vertices | Selected vertices | Selected logical CPU payload bytes |
| --- | --- | --- | --- | --- | --- |
| Baseline / overview | 1 / 1 | 0 | 99,282 | 99,282 | 5,162,664 |
| Save-scale / overview | 212 / 212 | 0 | 2,251,764 | 2,251,764 | 117,091,728 |
| Stress / crowd | 13 / 212 | 199 | 2,251,764 | 149,556 | 7,776,912 |

Indices matched vertex counts for these fixtures. Every profile recorded one geometry build and one visibility evaluation since initialization. Stress recorded 199 individual visibility changes, hiding the nonintersecting chunks. These fixed-camera profiles demonstrate initial selection and retained geometry; source regressions separately cover subsequent pan/zoom/resize and replacement.

The focused stress view therefore retains the full generated scene while making only 13 chunks eligible for submission. This is a concrete reduction in selected static geometry. It is not a measured GPU draw count, upload saving, portable FPS gain, or acceptance of the full workload on representative hardware. Dynamic crowd/train work remains separate.

Received GPU samples per named pass were 88/82 for baseline, 106/99 for save-scale, and 120/120 for stress (AutoVsync/AutoNoVsync), across four GPU passes. These asynchronous receipts do not identify the corresponding source frames. The nine raw profile/summary JSON files and image comparison record are retained with `SHA256SUMS.txt` in [the local dataset](https://github.com/strijbosmh-afk/RailRust-Tycoon/tree/codex/static-terrain-visibility/benchmarks/results/2026-10-20-static-visibility-local). All ten JSON files were copied byte-for-byte and privacy-scanned. Historical datasets and notices remain unchanged. No reference PC is selected.

## Evidence register

| Item | Current evidence |
| --- | --- |
| Implementation / PR / CI | `ff2857e68bfa8217fb85fe3bd107312f385d490e`; [draft PR #8](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/8), stacked on PR #7. [Windows CI run 39440918757](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39440918757) passed for this exact source; later documentation heads need their own status |
| Partition and bounds contract | Passing tests cover exact triangle attributes over rotations/views/grid settings, unchanged small meshes, shared/custom attributes, atomic support/footing/rail groups, layer metadata without changing the full scene, overflow bounds, focused candidate coverage, and empty/rectangular contracts |
| Release tests / formatting / strict lint / build | All 193 release tests passed: 128 game / 65 simulation. Formatting, strict release Clippy with warnings denied, and release compilation passed |
| PowerShell report validation | 30 new checks plus 44 historical wrapper checks passed; all six production profiles also passed validation |
| Viewport/resource reuse and replacement | Passing tests cover camera movement, resize/DPI and full-window guard, inclusive edges/overhangs, unchanged entity/mesh handles without rebuild, unknown-window fallback, map replacement, excess-asset removal, and same-tick refresh |
| Production images / picking | Eight image pairs match exactly outside 709 version-line pixels per pair. Covers normal 32-map, minimum 1024 × 680 large Ride rotation 2, 256 overview rotation 0, Crowd rotation 1, stress Ride rotation 2, stress Crowd rotation 3 Underground/Flat, Ride rotation 0 Underground/Terraced, and Ride rotation 3 Surface. Eleven capture processes exited successfully: eight current plus three fresh prior-build captures; five retained prior captures complete the pairs. Current images were inspected; picking/depth regressions also pass |
| Generated versus selected geometry | Six completed GPU profiles produced the table above, with one geometry build/evaluation each and reduced focused-view selection. Counts are CPU eligibility, not actual GPU draws/pixels |
| Save compatibility / preview isolation | Production 0.9 writes → 0.10 verifies and 0.10 writes → 0.9 verifies both passed 3,600 ticks/commands, full state, explicit backup, and paused load. Preview with `--save-dir` rejects without creating the directory |
| Executable hash/size | 46,168,064 bytes; SHA-256 `b3a63267dc450f1deab1a8a48aa1f93e93d0c4d6c9ba1e05bc2009f12bdb6c52` |
| Notices / package / guides | Final refresh verified all 25 payload hashes and sixteen current guides; includes 291 dependency notices, default-font/Rust attribution, and one required MSVC DLL. Existing notice bytes are preserved. The adjacent archive checksum is authoritative; future documentation edits require another package refresh |
| Relocated runtime / bundled wrapper | A fresh extraction path containing spaces and `é` ran from `C:\Windows`. Separate processes passed 3,600-tick primary continuation/commands, full state, explicit backup, and paused loading. The 256-map stress Ride rotation-2 Surface capture was byte-identical to the source result. The sibling wrapper selected its adjacent executable and passed one 30-tick CPU baseline plus both 120-interval save-scale Ride rotation-2 Underground GPU profiles with the static-report validator. Source-checkout fields were null as expected outside Git; executable hash/size matched |
| Representative PC / full target | No reference PC selected; PRF-05 remains in progress |

The next recommended bounded gameplay step is interactive level-path placement and demolition in the existing 32 × 32 park. Add a visible placement preview, explicit accepted/rejected feedback, and deliberate placement/removal actions using the existing clearance checks, stable IDs, and path-cache invalidation. Verify guests connect, reroute, become lost, and recover appropriately, and include same-step save/load plus continuation checks for built and removed paths. Existing Remove/Restore fixture controls do not provide construction.

This is an initial slice of BLD-01/BLD-02/BLD-04, not completion of their eventual cost/refund, queue, or broader construction criteria. Keep map/save bounds and one-circuit scope unchanged. Do not turn the next slice into further FPS tuning on this excluded PC. Native Save/Load/Backup, O/F/buttons, physical Delete, pan/drag/display, and clean-PC offline/standard-user gaps remain open unless directly reverified.

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
