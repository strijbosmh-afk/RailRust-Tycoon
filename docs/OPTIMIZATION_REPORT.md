# Guest rendering and path-cache report — 0.8.0

Date: 20 October 2026. The bounded optimization scope is complete: guest rendering reuses geometry and buffers, and unchanged crowd ticks reuse path connectivity. Regression tests, bidirectional save continuation, paired rendered images, local diagnostic counters, hosted CI, and relocated-package execution verify the change. This does not complete PRF-05 or establish hardware requirements.

**No reference PC is selected.** The user explicitly rejected this development PC as a performance reference because it performs poorly. Its 0.7.0 measurements and any new runs here are local diagnostics for investigating work and comparing implementations under stated conditions. They do not set minimum hardware, establish representative FPS, or pass/fail a hardware target. Select an agreed representative PC before evaluating the proposed 60 FPS at 1080p target.

PRF-05 remains **in progress**. The proposed 256 × 256 / 5,000-guest / 50-ride workload is still beyond the one-circuit model and 32 × 32 player presentation. Optimization correctness and local diagnostic results are separate from this acceptance gate.

## Implementation and boundaries

`park_sim` owns authoritative objects, guest state, integer ticks, stable IDs, and deterministic routing. Its new private `PathCache` retains an immutable `Arc<PathGraph>`. Successful path placement/removal invalidates it; the next crowd tick builds it lazily. Unchanged ticks reuse it. Clones may share the immutable graph, but edits and counters remain local to each `Park`. Rejected edits, unchanged fixture restoration, terrain presets, and track-only edits do not discard valid walkable topology.

Graph construction indexes path objects by `(x, y, height)` and looks up four cardinal neighbors, then sorts neighbor IDs. This preserves the old all-pairs graph's stable traversal order. `path_neighbors` uses a warm graph when available; cold read-only queries still derive their answer without changing counters.

`Park::path_cache_stats()` reports `builds`, `reuses`, `invalidations`, and `cached`. Reuses count crowd ticks; invalidations count discarded warm graphs, so multiple edits before a rebuild do not count multiple discarded graphs. These diagnostic values and graph warmth are excluded from authoritative equality and saves. Loading starts with an empty cache; the first format remains schema 1/content 1. `Park` now has explicit equality over its authoritative fields: keep this list and the save DTO aligned when adding future gameplay state.

`park_game` owns guest geometry and its presentation caches. `world::GuestRenderer` persists beside the existing guest mesh. It prepares 192 body templates—six appearances across thirty walking poses plus Waiting and Lost—and retains all five mesh vectors: positions, colors, normals, UVs, and triangle indices. Buffers keep their high-water capacities. This is CPU geometry/buffer reuse, not GPU instancing or proof of reused GPU uploads.

An ID lookup and exact projected path roofs avoid repeated guest/object scans and per-guest neighbor queries. Full object snapshots, dimensions, and rotation invalidate projected roofs even when a replacement park has the same IDs/counts. The HashMap is only an ID lookup; guest emission order is retained. Every update reads current guest state and visibility. Contact shadows use sixteen-point stack clipping buffers. The retained `guest_mesh` constructor provides an independently generated geometry oracle.

`GuestRenderer::stats()` reports `updates`, `deck_rebuilds`, `template_builds`, `buffer_growths`, `rendered_guests`, and `vertices`. Buffer growth counts vector-capacity increases, not all allocations or GPU memory. Rendering equivalence checks require identical indices, colors, normals, UVs, and physical depth; XY permits less than 0.002 world-presentation units for floating-point translation order. No changed save format, larger save limit, extra rides, new player controls, or full path/track editor is included.

## Correctness checks

The meaningful regression checks for this slice are:

- Compare cached routing against fresh production graph construction over long deterministic continuation and path-removal/restoration commands. Include disconnected components, stacked paths at different heights, stable shortest-path ties, newly allocated restored IDs, and paused queries immediately after edits.
- Verify successful changes invalidate the needed caches and rejected atomic edits leave the original park intact. Check fixture repair, cloning, save roundtrip after cache warmup, and loading a valid object vector whose ordering differs from ID order.
- Compare guest geometry/appearance and support clipping over Walking, Waiting, and Lost states, current/next crossings, missing-path recovery, all rotations, and both visibility modes. Preserve foreground terrain/deck occlusion and non-selectable guest behavior.
- Verify normal scene updates reuse intended allocations without accidentally rebuilding static terrain. Same-tick path edits and successful load must refresh all affected presentation state; empty or replaced guest collections must not retain old geometry.
- Keep save continuation, fixed-step equivalence, selection/picking, train, input ordering, and package relocation regressions.

All **159 release workspace tests passed: 65 simulation and 94 game tests**. The twelve additions cover five graph/cache cases, six retained-renderer cases, and one scene-update integration case. Formatting, release Clippy with warnings denied, and release compilation passed. Eleven PowerShell helper checks also passed, including rejection of contradictory counters and compatibility with historical 0.7.0 reports.

Separate production processes verified saves in both directions: 0.7.0 write → 0.8.0 verify, and 0.8.0 write → 0.7.0 verify. Both passed 3,600 continued ticks with commands, full authoritative-state comparison, explicit backup, and paused loading. This verifies compatibility of these two checkpoints with unchanged schema 1/content 1; it is not a promise for future formats.

Four paired 0.7.0/0.8.0 capture scenes completed in eight successful processes: crowd Surface rotation 0, crowd Underground rotation 2, lost Surface rotation 1, and recovered Surface rotation 3. The four new images were visually inspected for guests, shadows, occlusion, recovery, and UI. Decoded-image comparison found exactly 44 changed pixels in each pair, all in the version digit at x32–37/y739–746. The entire world and all other UI pixels were identical. The local comparison receipt is `artifacts/optimization-0.8.0/capture-comparison.json`.

## Local comparison method

The comparison retains the baseline, save-scale, and stress fixtures, seed 20261020, complete-tick rules, release compiler/dependency graph, and save limits. Processes ran sequentially with recorded source/executable identities. Single-run distributions and completed work are reported without selecting a best sample or claiming repeatability.

The completed batch contains one 300-tick run for each case and one 1,800-tick stress observation, each following 60 untimed simulation warmup ticks. The longer observation includes more route completions/replanning. All four completed their requested work. Operation counts, geometry equivalence, and cross-version continuation are the primary evidence that repeated work was removed safely.

The CPU report adds `geometry.guest_renderer.cold` (five samples), `moving_update` (sixty), and `legacy_moving` (sixty). The animated paths follow sixty untimed warmup updates, then advance one complete simulation tick before each rendering timer. The retained and optimized renderer consume identical cloned states; completion requires equal final authoritative state and emitted vertex totals. Cold includes template/deck/buffer preparation, one update, and destruction; moving updates retain one renderer and mesh. Do not label these different workloads interchangeably.

`geometry.guests` retains the old frozen-state constructor for continuity, but its path-neighbor calls now benefit from the new simulation cache. A comparison against 0.7.0 therefore includes that shared optimization. The paired animated probes in the same build provide a clearer comparison of the two rendering paths, while fixed run order and active-host noise still limit timing conclusions. Reports retain graph-cache before/after counters, renderer counters, warmup counts, and actual ticks.

No frame-performance comparison was performed for this checkpoint. The packaged wrapper's two 120-sample baseline profiles were functional render/upload smoke checks only. Both actual windows were 1920 × 1080, visible and focused; occlusion remained unknown for all 120 intervals in each run. Frame profiles measure CPU main-loop pacing, not GPU or physically presented-frame timing. A missing GPU/render-thread measurement cannot establish the cause of a frame-rate change.

The original 0.7.0 diagnostic dataset remains byte-for-byte at `benchmarks/results/2026-10-20-reference/`, including its original JSON, checksums, and path names. The word `reference` in that historical directory name does not designate the PC as a reference machine. See the clarification in [PERFORMANCE_REPORT.md](PERFORMANCE_REPORT.md) and the dataset README. No old timing value is rewritten.

## Local diagnostic results

The six raw JSON files and their hash manifest are retained in [the optimization dataset](https://github.com/strijbosmh-afk/RailRust-Tycoon/tree/codex/guest-rendering-path-cache/benchmarks/results/2026-10-20-optimization-local). These are four CPU reports and two wrapper summaries, copied without altering their bytes. Both batches recorded clean checkout `3f1ea886fea965a80bc5d3438ae80ecc4de4543d` and the same executable. That checkpoint adds a PowerShell-only report-validator correction to runtime implementation `9460a90180f94a1d8f73f5a35d3480caf43f0b14`.

The local host is Windows 11 Enterprise 10.0.26200, Intel Core i5-9400T, Intel UHD 630 driver 31.0.101.2115, and 8,453,365,760 bytes of physical RAM. Power, temperature, and background load were not controlled. **This host is explicitly excluded as reference hardware.**

| Case | Guests / paths / map | Measured ticks | Simulation mean / p99 ms | Retained moving-render mean ms | Legacy moving-render mean ms |
| --- | --- | --- | --- | --- | --- |
| Baseline | 12 / 37 / 32 × 32 | 300 | 0.00050 / 0.00780 | 0.02138 | 0.07452 |
| Save-scale | 256 / 1,024 / 256 × 256 | 300 | 0.04579 / 0.66890 | 0.47380 | 3.10717 |
| Stress | 5,000 / 1,024 / 256 × 256 | 300 | 0.99691 / 3.51510 | 14.80183 | 214.01023 |
| Extended stress | 5,000 / 1,024 / 256 × 256 | 1,800 | 2.46539 / 6.54630 | 20.53611 | 104.42832 |

Each renderer pair completed sixty animated samples and reported `matched_moving_workloads: true`, including identical final authoritative state and emitted vertex totals. The short and extended runs reach different states before rendering and are not repeated samples of the same workload. In both stress runs the timings were noisy: the retained-render p99 was 28.57470/95.68950 ms, while the legacy mean varied substantially. No portable speedup factor or FPS conclusion follows from these numbers.

Every simulation measurement added **zero graph builds and exactly 300 or 1,800 graph reuses**. The single graph had already been built during warmup. Every retained-render measurement advanced updates from 60 to 120 while template builds stayed at 192, deck rebuilds at 1, and buffer growths at 5. These counters demonstrate reuse across the measured unchanged topology and animated states; tests separately cover invalidation after edits and loads.

Fresh indexed graph construction was also measured independently; its mean was 0.01396 ms for baseline and 0.47256–0.60181 ms for the 1,024-node cases. These fresh-build costs are separate from cached simulation tick costs. CPU scene construction remains a headless probe; it does not demonstrate interactive rendering of a 256 × 256 park or fifty rides.

## Evidence

| Item | Result |
| --- | --- |
| Runtime source / branch / review | Runtime `9460a90180f94a1d8f73f5a35d3480caf43f0b14`; wrapper validator `3f1ea886fea965a80bc5d3438ae80ecc4de4543d`; branch `codex/guest-rendering-path-cache`; [draft PR #6](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/6) stacked on PR #5's performance branch |
| Simulation and renderer correctness | 159 release tests passed, including deterministic cache/invalidation, full geometry/oracle, allocation reuse, and same-tick load integration cases |
| Formatting / lint / release build | Formatting, release Clippy with warnings denied, and release compilation passed |
| Local CPU diagnostics | All four CPU reports complete; all four animated-render comparisons matched; counters and timings above |
| Render inspection | Eight paired capture processes exited 0; four new images inspected; only 44 version-digit pixels differed per pair |
| Save continuation | Both cross-version directions passed 3,600 ticks/commands, primary and backup, full state, and paused loading |
| Package relocation | All 23 extracted payload hashes matched in a fresh path containing spaces and `é`; execution from `C:\Windows` passed continuation/backup/paused load and sibling-wrapper baseline 30 ticks plus both 120-sample frame smoke modes |
| Executable identity | 45,947,392 bytes; SHA-256 `f907dbfc3effdcd460ab5e84ef15bf58cf623f9321231f89dd4a2c2f99ba6c24` |
| Notices / final guide refresh | Packaging collected 291 dependency notices, font/Rust notices, and one MSVC DLL. The final refreshed ZIP was extracted again: all 23 payload hashes and fourteen current Markdown guides matched the workspace; the executable identity stayed unchanged |
| Hosted Windows CI | [Run 39429282120 passed](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39429282120) for `3f1ea886fea965a80bc5d3438ae80ecc4de4543d`, including build, packaging, continuation, and CPU-wrapper checks |
| Representative PC and hardware-target acceptance | Not selected; deferred until agreed |

CI success belongs to its recorded source checkpoint, not automatically to a later documentation-only head. The archive changes when guides are refreshed; its adjacent `.zip.sha256` is authoritative. Do not embed the archive's own changing hash in a bundled guide.

## Next decision and handoff

Retain the cache and renderer changes: tests, continuation, images, and counters establish the intended behavior and removed work. The bounded optimization slice is complete. The next performance investigation should collect render-thread/GPU/upload/presentation evidence and support a controlled large-map render scene before selecting another rendering strategy; the existing probes cannot identify a GPU bottleneck. Keep the current geometry oracle, deterministic continuation, and counter checks as regressions.

Representative hardware must be agreed before performance acceptance. Fifty rides, large-map player presentation, and larger save capacity remain separate implementation/measurement work. Keep PRF-05 open; do not let this local optimization result silently revise those targets or block unrelated small gameplay slices on this PC's FPS.

Native Save/Load/Backup, O/F/Open/Close/Find, physical Delete, panning/dragging/physical display, and clean-PC offline/standard-user checks remain open unless new direct evidence closes them. Preserve the current source-license/title and content-scope decisions.

The current shared OneDrive source path remains:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
```

The separate historical second-PC clone remains:

```text
C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon
```

Use only one active editing/build/Git session in the shared folder. Commit and push source plus handoff records before changing PCs; generated outputs and LocalAppData player saves have their own transfer/verification requirements. Measure a new PC afresh and seek agreement before designating it as a performance reference.
