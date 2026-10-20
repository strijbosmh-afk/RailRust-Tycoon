# Guest rendering and path-cache report — 0.8.0

Date: 20 October 2026. This checkpoint is in development. Its scope is reducing repeated guest-geometry work and rebuilding path connectivity only when required, while preserving gameplay and save continuation. Implementation details and evidence below remain pending until verified.

**No reference PC is selected.** The user explicitly rejected this development PC as a performance reference because it performs poorly. Its 0.7.0 measurements and any new runs here are local diagnostics for investigating work and comparing implementations under stated conditions. They do not set minimum hardware, establish representative FPS, or pass/fail a hardware target. Select an agreed representative PC before evaluating the proposed 60 FPS at 1080p target.

PRF-05 remains **in progress**. The proposed 256 × 256 / 5,000-guest / 50-ride workload is still beyond the one-circuit model and 32 × 32 player presentation. Optimization correctness and local diagnostic results are separate from this acceptance gate.

## Intended implementation and boundaries

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

The simulation's 65 release tests passed, including five new graph/cache regressions. Full-workspace checks, renderer/integration tests, and runtime evidence remain pending. Historical 147-test evidence belongs to 0.7.0 and must not be reused as current verification.

## Local comparison method

Keep the existing baseline, save-scale, and stress fixture definitions, seed, complete-tick rules, and save limits. Use the same release compiler/dependency graph and record source/executable hashes for each candidate. Run sequentially, with the same warmup and sample settings, and compare repeated distributions and completed work rather than selecting a best run.

This checkpoint plans a small local batch covering baseline/save-scale/stress and one 1,800-tick stress observation. Actual settings/results will be recorded below; no repeated-hardware comparison or representative performance conclusion is implied. Longer observation includes more route completions/replanning. Preserve incomplete outcomes. Operation counts, geometry equivalence, and cross-version continuation are primary evidence that the intended repeated work was removed safely.

The CPU report adds `geometry.guest_renderer.cold` (five samples), `moving_update` (sixty), and `legacy_moving` (sixty). The animated paths follow sixty untimed warmup updates, then advance one complete simulation tick before each rendering timer. The retained and optimized renderer consume identical cloned states; completion requires equal final authoritative state and emitted vertex totals. Cold includes template/deck/buffer preparation, one update, and destruction; moving updates retain one renderer and mesh. Do not label these different workloads interchangeably.

`geometry.guests` retains the old frozen-state constructor for continuity, but its path-neighbor calls now benefit from the new simulation cache. A comparison against 0.7.0 therefore includes that shared optimization. The paired animated probes in the same build provide a clearer comparison of the two rendering paths, while fixed run order and active-host noise still limit timing conclusions. Reports retain graph-cache before/after counters, renderer counters, warmup counts, and actual ticks.

Optional baseline frame profiles retain requested/actual resolution, present mode, focus, visibility, occlusion, and clock-cap data. They measure CPU main-loop pacing, not GPU or physically presented-frame timing. On this explicitly unsuitable development PC they remain local observations, even if they improve. A missing GPU/render-thread measurement cannot establish the cause of a frame-rate change.

The original 0.7.0 diagnostic dataset remains byte-for-byte at `benchmarks/results/2026-10-20-reference/`, including its original JSON, checksums, and path names. The word `reference` in that historical directory name does not designate the PC as a reference machine. See the clarification in [PERFORMANCE_REPORT.md](PERFORMANCE_REPORT.md) and the dataset README. No old timing value is rewritten.

## Evidence

| Item | Result |
| --- | --- |
| Runtime source / branch / review | `codex/guest-rendering-path-cache`, continuing from 0.7.0 checkpoint `ffd8f1e`; implementation commit/review pending |
| Final simulation/cache contract | Implementation reviewed read-only; 65 simulation release tests passed |
| Final guest-rendering contract | Persistent renderer and integration implemented; runtime/test verification pending |
| Formatting / lint / tests / release build | Full-workspace checks pending for 0.8.0; partial simulation result above |
| Local before/after CPU measurements | Pending; diagnostic use only |
| Render inspection / frame observations | Pending |
| Save continuation / package relocation | Pending |
| Executable identity / notices / guide hashes | Pending |
| Hosted Windows CI | Pending |
| Representative PC and hardware-target acceptance | Not selected; deferred until agreed |

## Next decision and handoff

After correctness and measurements are available, record what work was removed, what remains expensive, and the next bounded change. Keep GPU/presentation investigation separate from CPU cache/mesh results. Selecting a representative PC and implementing the missing full-target scale remain open decisions; this checkpoint must not silently close PRF-05.

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
