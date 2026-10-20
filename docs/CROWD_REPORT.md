# Path and crowd proof report

Build: **Parkbound 0.4.0**, 20 October 2026. The Stage 2 / **PRF-02 path-connectivity and crowd-recovery proof is verified** through the automated, rendered, and native-button evidence below. The physical Delete shortcut and distribution checks remain separate open items. The 0.3.0 overlap proof remains the regression baseline.

## Delivered behavior

- A small original network extends the elevated walkway into a loop, junctions, and branches: 37 path tiles at height 8. The nine track tiles at height 0 remain in place.
- Twelve seeded guests choose reachable destinations, walk between neighboring path tiles, and wait briefly after arrival before choosing another destination. The interface reports walking guests, completed visits, and lost guests.
- **Delete** or **Remove path** removes the selected path. Navigation references are reconciled immediately; guests reroute where possible or enter a defined lost state.
- **R** or **Restore paths** restores missing fixture paths without resetting simulation time, guests, their stable IDs, or the IDs of surviving objects. Restored objects receive new IDs.
- Lost guests retain their last location. Recovery uses a path at that tile or a cardinal neighbor at the same height; it does not teleport a guest to an arbitrary distant path.
- Normal, 2×, and 4× speed execute complete authoritative simulation steps. Pause performs no simulation steps.
- A separate dynamic mesh presents guests above their supporting path while terrain and nearer surfaces still occlude them. Guests are not selectable objects in this proof.
- Terrain and underground controls remain available. The original stable surface identity, clearance, and hidden-layer picking rules are retained.

This is a small navigation demonstration. It does not add general path construction, sloped connections, ride queues or boarding, guest needs or spending, operating rides, finances, saves, loading, or audio. Completed visits are arrivals at demonstration path destinations, not paying attraction visits.

## Simulation contract

`park_sim` remains independent of Bevy. Its guest state, route choice, random-generator state, travel progress, and counters are authoritative. Presentation does not advance guest state.

Paths connect through cardinal tile adjacency at the same base height. Vertically stacked paths and diagonal neighbors do not connect. The existing X/Y `Axis` controls the visual plank orientation; path connectivity is determined by valid neighboring path objects. Track objects do not become walking paths. Shared occupied-space validation still applies before placement.

Routing uses deterministic breadth-first search with stable ordering and a stored SplitMix64 seed state. A guest chooses another reachable path node, takes 30 simulation ticks per path edge, then waits 30 ticks on arrival. The path network is intentionally small; this implementation does not establish the proposed large-park performance target.

Guest identities persist. A guest's `current`, `next`, destination, and route references must resolve or be cleared when paths change. Removing any path in a guest's active edge/destination/route cancels the route, returns partial movement to the retained starting anchor, clears invalid references, and marks Lost immediately. A subsequent tick may replan within the surviving component. If its current path vanished, recovery checks only the retained tile and cardinal same-height neighbors, ordered by distance then stable ID. An isolated guest remains lost until a usable connection becomes available. Removal and fixture restoration preserve unrelated objects and simulation time.

`Park::advance(speed)` runs the complete tick body once per speed multiplier at a 30 Hz application fixed update. Equivalent counts of normal and accelerated ticks must produce equal authoritative state for the same seed and command sequence. Display frame rate must not change route choice or travel speed.

The current fixture is deterministic and restoration is atomic. The original x8..19/y16 walkway is retained; loop sides at x8/x19, y17..20 and a bottom edge at y20 join the added branch/crossbar around x13. All coordinates in this paragraph are zero-based. Terrain presets retain paths at height 8 and track at height 0.

## Presentation and controls

Guest positions interpolate only between authoritative path endpoints and their integer travel fraction. The separate guest mesh avoids rebuilding the terrain mesh solely because a guest moved. Path decks extend toward connected neighbors so visible junctions correspond to walkable connections. Lost guests have a distinct orange marker at their retained location.

Path removal and restoration must invalidate scene and selection geometry. A removed selected object must no longer appear in the inspector or as a pick target. Restore must not make an old removed ID resolve to the new object at that coordinate.

For the manual demonstration:

1. Start at normal speed and watch guests traverse the loop and junctions. Confirm visits increase.
2. Pause, select a path segment, and remove it. Check that removal is immediate and the selected reference clears.
3. Resume and observe rerouting or lost guests. Delete a destination or occupied segment as separate cases.
4. Remove enough paths to isolate a remaining node. Guests must remain defined and must not walk across missing tiles.
5. Restore paths. Check that time and guest identities continue, surviving path IDs remain stable, and locally stranded guests recover when a connection is restored.
6. Compare pause, 1×, 2×, and 4× behavior. Repeat relevant selection and rendering checks through all four rotations, both terrain presets, and both visibility modes.

The existing camera pan/drag and physical DPI/multiple-display acceptance gaps remain separate. Engine captures and synthetic input tests do not establish normal operating-system input.

Diagnostic presets extend the existing capture interface:

```text
parkbound.exe --capture-frame <absolute-output.png>
  --capture-scene overview|selection|path|track|crowd|lost|recovered|help
  --capture-ticks 0..36000
  --capture-map flat|terraced
  --capture-view surface|underground
  --capture-rotation 0|1|2|3
  [--capture-small-window]
```

The default tick count is 600 before the scene is paused. Crowd presets frame the network at 160% zoom. `lost` removes the neighbors of the first guest's current path and advances 60 additional ticks; `recovered` then restores fixture paths and advances 600 ticks. These presets directly prepare a controlled state; they do not simulate a normal keyboard or button action.

## Verification record

The local release build passed formatting, release Clippy with warnings denied, all **72 tests**, and release compilation on 20 October 2026. The test total consists of 30 simulation tests, 20 world/rendering tests, 18 input-system tests, and four main-module tests.

Passing source-check commands:

```powershell
cargo fmt --all -- --check
cargo clippy --release --locked --workspace --all-targets -j 2 -- -D warnings
cargo test --release --locked --workspace -j 2
cargo build --release --locked -p park_game --bin parkbound -j 2
```

The verified executable SHA-256 is:

```text
78615c2de086b6cf1e2f7db1e4eac2804d9fb2b9c81314f870cca525db061ee7
```

Automated coverage includes deterministic seeded continuation; complete-step speed equivalence and pause; same-height/cardinal-only connections; junctions and disconnected components; removal of occupied, next, destination, and later-route paths; immediate reference cleanup while paused; retained lost anchors and local recovery; fresh IDs after restoration; atomic restoration failure; guest depth/visibility; separate moving meshes; and UI input handling.

### Engine captures

Eleven captures under `artifacts/crowd-0.4.0/` completed with exit code 0 and were visually inspected. These generated images are local evidence, not game assets or a substitute for native input.

| Capture | Observed result |
| --- | --- |
| `crowd-start` | Zero ticks: twelve waiting guests and zero destinations reached. |
| `crowd-r0`, `crowd-r1`, `crowd-r2`, `crowd-r3` | At 600 ticks: ten walking, two waiting, zero lost, and 29 arrivals in every rotation. |
| `lost` | At 660 ticks: Path #5 isolated, one lost guest, and 31 arrivals. |
| `recovered` | At 1260 ticks: restored connectivity, zero lost guests, and 55 arrivals. |
| `small-help`, `small-path` | Help and selected-path layouts fit the 1024 × 680 minimum window without clipped controls. |
| `flat-crowd` | The crowd and connected path network render under the Flat terrain preset. |
| `underground-track` | Underground inspection retains the track and elevated-path/crowd layers. |

### Native input

The `@oai/sky` desktop helper was available for this verification. Normal application controls were exercised at 1280 × 800:

1. Space paused the simulation at **00:21**, with 29 destinations reached. Mouse-wheel input changed zoom from 100% to 162%.
2. Clicking occupied **Path #37**, displayed tile **17,21**, selected it. The **Remove path** button removed the deck, cleared selection, and showed an orange lost guest. Time and the total of twelve guests were preserved.
3. **R** restored the path while paused. The lost count remained one until simulation resumed, as expected. Clicking the restored coordinate showed fresh **Path #47**.
4. The **Resume** button restarted movement and the lost count returned to zero. A later Space pause at **00:42** showed 59 destinations reached.
5. Removing selected Path #47 through its button produced two lost guests. The **Restore** button restored the network without changing the paused time.
6. **H** opened help; **Back to the park** closed it without selecting through the overlay or changing paused time. The native window-close control ended the application process.

Two attempts to inject **Delete** through the helper had no effect. The headless `KeyCode::Delete` regression passes, but that does not prove the physical shortcut. **Retest Delete using a physical keyboard or a verified native key-injection mapping; use Remove path for the demonstrated native deletion flow.** No keyboard-shortcut fix is inferred from this ambiguous helper result.

The core PRF-02 acceptance is supported by native removal/restoration and observed recovery, the isolated/recovered engine scenes, and the simulation invariants. Keyboard panning, right/middle dragging, physical DPI/multiple-display coverage, and clean-machine compatibility remain open from earlier milestones. No large-park or varying-display-frame-rate performance claim is made.

### Package and hosted CI

The local Windows package was generated successfully. Its archive checksum and all **18 payload checksums** matched after extraction into a fresh temporary directory containing spaces and `é`. The extracted executable has the SHA-256 recorded above. It launched from `C:\Windows`, rendered the recovered scene, and exited with code 0. Its recovered PNG was byte-for-byte identical to the original build's inspected recovered capture.

The package includes one imported MSVC runtime DLL (`VCRUNTIME140.dll`), notices for 288 dependencies and the default font, Rust standard-library notices, and the ten current Markdown documents. This is a relocation test on the development PC, not a clean-machine test. The final archive hash lives beside the ZIP in `.zip.sha256`; it cannot be embedded in a report inside that same archive without changing the hash.

These are local verification results. The repository's Windows workflow also checks formatting, linting, tests, and packaging on pushes and pull requests. Consult the [current branch's workflow runs](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions?query=branch%3Acodex%2Fpath-crowds) for hosted status and the exact commit; the older overlap branch's green runs do not verify this change.

Packaging command:

```powershell
pwsh -NoProfile -File scripts/package-windows.ps1 -SkipBuild
```

Repeat archive and payload verification after any repackaging. Keep packaged Markdown files identical to their source counterparts. Local evidence for this checkpoint is under `artifacts/crowd-0.4.0/`, including `relocated-recovered.png` and the recorded temporary package path.

## Delivery and continuation

Local output is `dist/Parkbound-0.4.0-windows-x64.zip`, with its adjacent `.sha256` and an extracted `Parkbound-0.4.0-windows-x64` directory. The package contains the current README, resume guide, development plan, this report, notices, and runtime dependencies. This is an unsigned local checkpoint; a source push or CI artifact is not a public binary release.

The current checkout is the original shared OneDrive folder recorded in [the resume guide](../RESUME_DEVELOPMENT.md). The separate second-PC path remains documented. Work on branch `codex/path-crowds` starts from `codex/overlapping-surfaces`; preserve the overlap branch's commits and open review until their merge status is confirmed. A stacked pull request should use that branch as its base while it remains unmerged.

PRF-02 is **V** for the demonstrated seeded routing, deletion, and recovery proof. The next implementation is PRF-03: a representative station → lift → drop → brake → station train circuit. Preserve the crowd and overlap regressions, and carry the explicit Delete-shortcut and distribution checks forward. Save/reload continuation and performance benchmarking remain later independent proofs.

Clean-machine/offline/standard-user installation, native pan/drag and physical display coverage, final public naming and source license, and full classic-content scope remain open.
