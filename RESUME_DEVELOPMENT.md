# Resume development on another Windows PC

Updated 21 October 2026 for **0.16.0 complete ride journeys**, on `codex/ride-journeys` from completed 0.15.0 checkpoint `965e3267b2ea8350067a12761fe339051b711e8d`. This stage adds a separate exit, physical station transfers, six passenger seats, completed rides, protected occupied construction and exact passenger continuation. Verification is in progress; section 10 and [the journey report](docs/RIDE_JOURNEY_REPORT.md) record current evidence. Historical [admission](docs/ADMISSION_REPORT.md) and [queue](docs/QUEUES_REPORT.md) reports retain their original evidence.

Canonical repository: <https://github.com/strijbosmh-afk/RailRust-Tycoon>. **Parkbound** is a temporary title for an original Rust theme park simulation inspired by classic RCT1/RCT2. The user chose Windows-only free distribution with all functionality included, developed in playable stages. It is not yet a complete management game. **No reference PC is selected; this development PC is explicitly excluded.** Its measurements remain local diagnostics, not hardware requirements or 60 FPS acceptance.

## 1. Exact folders and safe two-PC continuation

The original PC's shared OneDrive checkout is:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
```

Its relative OneDrive location is `Documenten\ChatGPT\Railroad Tycoon`. The Windows username and OneDrive root may differ on another PC. The folder name `Railroad Tycoon`, repository name `RailRust-Tycoon`, and temporary game title `Parkbound` intentionally differ.

The previously used second-PC checkout is:

```text
C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon
```

That second path is a **separate clone**, not evidence that OneDrive syncs it. Preserve both historical paths. Verify the actual folder, branch, and commit before continuing. The current branch is `codex/ride-journeys`, stacked on `codex/guest-admission` / [draft PR #13](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/13), which was still unmerged when this stage began. The new draft PR and exact runtime source will be recorded after verification. Do not assume `main` contains the latest playable stage or that dependent PRs have merged.

For an existing shared checkout:

```powershell
Set-Location -LiteralPath 'C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon'
git status --short --branch
git remote -v
git log -5 --oneline
git fetch origin
git branch -a
```

For a fresh separate clone, choose a new folder rather than overwriting either recorded checkout:

```powershell
$projectDirectory = Join-Path $env:USERPROFILE 'source\RailRust-Tycoon'
git clone https://github.com/strijbosmh-afk/RailRust-Tycoon.git $projectDirectory
Set-Location -LiteralPath $projectDirectory
git fetch origin
git branch -a
# Only after the current branch is listed on origin:
git switch --track origin/codex/ride-journeys
```

If the branch already exists locally, use `git switch codex/ride-journeys` instead. If it is absent from `origin`, finish/push the first PC's checkpoint before switching to an older branch unintentionally. Preserve local edits before branch changes; inspect divergence before any pull. Do not force-reset a checkout or overwrite another PC's work.

Use this handoff sequence:

1. On the active PC, finish the bounded change and its checks; update the milestone report and this guide. Inspect the diff and commit the intended files with a message describing the actual change. Push the branch and confirm the exact commit/PR/CI status.
2. Close the game, Cargo, editor tasks and Git operations using the shared folder. Let OneDrive finish syncing before the other PC opens it. Use only one active editing/build/Git session against that shared checkout.
3. On a separate clone, fetch and inspect the branch before `git pull --ff-only`. A non-fast-forward failure requires reconciling the two histories, not discarding one.
4. On the next PC, verify source identity, install the pinned toolchain, rebuild locally, and run the checks below. Copying `target` is unnecessary. Keep `target`, `dist`, and raw `artifacts` out of Git as configured.
5. Transfer saves separately with the game closed, preserving primary and backup together. Saves are outside the checkout by default; Git or OneDrive synchronization of source does not transfer them automatically.

Retain historical reports, raw benchmark data, evidence paths and notices. In particular, `benchmarks/results/2026-10-20-reference/` is a historical name, **not** designation of this PC as reference hardware. Do not rename or rewrite old evidence to imply current acceptance.

## 2. Read the project records

Start with [RIDE_JOURNEY_REPORT.md](docs/RIDE_JOURNEY_REPORT.md), [GAMEPLAY_CHECKLIST.md](docs/GAMEPLAY_CHECKLIST.md), [DEVELOPMENT_PLAN.md](DEVELOPMENT_PLAN.md), [PRODUCT_DECISIONS.md](docs/PRODUCT_DECISIONS.md), and [ASSET_PROVENANCE.md](docs/ASSET_PROVENANCE.md). The checklist's full feature criteria remain authoritative: a bounded implementation does not automatically complete BLD-02/03, the broader sloped-path criterion BLD-08, or the complete game.

Historical reports retain exact source, test, capture, native, CI, package, and checksum evidence without duplicating it all here:

| Version | Retained report |
| --- | --- |
| 0.16 | [Complete ride journeys](docs/RIDE_JOURNEY_REPORT.md) |
| 0.15 | [Guest admission and ordered queues](docs/ADMISSION_REPORT.md) |
| 0.14 | [Queue geometry and entrance connectivity](docs/QUEUES_REPORT.md) |
| 0.13 | [One-step sloped paths](docs/SLOPED_PATHS_REPORT.md) |
| 0.12 | [Terrain editing](docs/TERRAIN_EDITING_REPORT.md) |
| 0.11 | [Path construction](docs/PATH_CONSTRUCTION_REPORT.md) |
| 0.10 | [Static visibility](docs/STATIC_VISIBILITY_REPORT.md) |
| 0.9 | [Large-map/GPU diagnostics](docs/LARGE_MAP_GPU_REPORT.md) |
| 0.8 / 0.7 | [Optimization](docs/OPTIMIZATION_REPORT.md) / [performance diagnostics](docs/PERFORMANCE_REPORT.md) |
| 0.6 / 0.5 | [Persistence](docs/SAVE_REPORT.md) / [train](docs/TRAIN_REPORT.md) |
| 0.4 / 0.3 | [Crowds](docs/CROWD_REPORT.md) / [overlap](docs/OVERLAP_REPORT.md) |
| 0.2 / 0.1 | [Terrain](docs/TERRAIN_REPORT.md) / [foundation](docs/FOUNDATION_REPORT.md) |

The completed 0.13 runtime source was `b1d26a04cc1282bb71defcb59557309b79252f1d`: 260 release tests, strict source gates, eleven inspected captures, continuation/migration, native construction/recovery, and relocated packaging passed. [CI 39583267705](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39583267705) verifies that runtime source; [CI 39584568173](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39584568173) verifies its later checkpoint. Keep historical identities separate from the current binary and documentation head.

## 3. Toolchain and source checks

Use Windows x64 with Git for Windows and PowerShell 7. Windows 11 x64 is the initial build/test target, not a proven minimum OS requirement. Install Visual Studio 2022 Build Tools with **Desktop development with C++**, x64/x86 MSVC tools, a Windows SDK, and the redistributable/runtime components required by the packaging helper. The helper currently discovers VS 2022 runtime files; an arbitrary newer compiler installation is not automatically an equivalent package source.

The repository pins **Rust 1.93.1**, `x86_64-pc-windows-msvc`, and `rustfmt`, `clippy`, `rust-docs` in `rust-toolchain.toml`. Cargo uses edition 2024; the manifest's `rust-version = 1.88` is a declared lower bound, not the tested toolchain. Bevy is locked to 0.17.3. Preserve `Cargo.lock`; do not upgrade dependencies as an incidental setup step.

```powershell
rustup toolchain install 1.93.1 --profile minimal --component rustfmt --component clippy --component rust-docs --target x86_64-pc-windows-msvc
rustup show active-toolchain
rustc -vV
cargo --version
```

The recorded second-PC setup had Rust under `%USERPROFILE%\.cargo\bin` without a persistent PATH change, and its MSVC/SDK installation was blocked by UAC. Check the current machine; do not assume that old obstacle has been resolved. If Rust is installed but absent from this shell, prepend that existing directory to this session's PATH. Administrator-assisted Build Tools installation may still be required; a Rust-only installation cannot link this MSVC game.

Run from the repository root, sequentially. Limit Cargo to two jobs on this development PC:

```powershell
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { throw 'Formatting failed' }
cargo clippy --release --workspace --all-targets --locked -j 2 -- -D warnings
if ($LASTEXITCODE -ne 0) { throw 'Clippy failed' }
cargo test --release --workspace --locked -j 2 -- --test-threads=2
if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
cargo build --release --locked --package park_game --bin parkbound -j 2
if ($LASTEXITCODE -ne 0) { throw 'Release build failed' }
```

The playable executable is `target\release\parkbound.exe`. For a deliberate interactive run, launch it normally. For native QA use a fresh absolute `--save-dir` so the real player slot remains separate. A successful build does not establish runtime/native/clean-PC acceptance. Capture exit codes explicitly when using the Windows GUI executable; `Start-Process -Wait -PassThru` is useful in PowerShell scripts.

## 4. Current game and source map

Normal play remains a 32 × 32 deterministic demonstration, initially Terraced (grass/water/sand at heights 0/2/4/6), with Flat and player-edited Custom states. There are 37 public paths around a loop/junction/branches, twelve seeded guests, nine legacy track pieces, and the 46-piece Meadow Flyer circuit at x7..24/y4..10 with rail heights 8..16 and three cars. Fresh startup has **no queue, entrance or exit**. Larger 256 × 256 scenes are controlled diagnostics, not general player-map/save support.

| Source | Responsibility and invariants |
| --- | --- |
| `crates/park_sim` | Window-independent deterministic park, bounded terrain/clearance, stable IDs, routes/RNG, crowds, train, ride/queue topology and strict save codec. Never make correctness depend on Bevy or wall-clock time. |
| `park_sim/src/path_geometry.rs` | Flat/four one-step ramps; endpoint matching; integer `PATH_UNITS = 60` positions. Guest center→edge→center heights and renderer share this geometry. |
| `park_sim/src/ride_access.rs` | Stable ride identity, entrance placement and derived owned-queue access. Public guest graph is separate. |
| `park_sim/src/admission.rs` | Seeking, tickets, capacity, reserved queue movement, patience, physical return and local recovery. Motion remains in the shared fixed-point guest sampler. |
| `park_sim/src/journeys.rs` | Exit connectivity, six stable seats, boarding/unloading transfers, physical passenger positions, completion counters and demolition protection. |
| `park_game/src/main.rs`, `path_tools.rs`, `path_tools/entrance.rs`, `terrain_tools.rs` | Input ordering, tool state, immutable preview validation, commit/gating, pause/view and scene refresh. |
| `park_game/src/world.rs`, `world/*` | Shared physical faces for painting/picking/selection, slope/gate/terrain ghosts, persistent guest rendering and sampled train geometry. |
| `park_game/src/static_landscape.rs` | Persistent static chunk slots and conservative visibility. Complete emission groups preserve internal geometry order; small maps retain one mesh. |
| `park_game/src/ui.rs` | Fixed control columns, inspector, train/access feedback, shared Build Tools panel and help. Minimum layout is 1024 × 680. |
| `persistence.rs`, `save_store.rs`, `save_check.rs`, `save_check/*` | Paused load/snapshot integration, bounded atomic file operations, isolated production continuation proofs. |
| `capture.rs`, `capture/*`, `*_input_tests.rs` | Controlled scenes, native-independent input regression harness and retained visual proofs. |
| `benchmark.rs`, `frame_probe.rs`, `gpu_probe.rs`, `render_scene.rs` | Opt-in diagnostics only; CPU time, main-loop intervals and actual GPU timestamp spans remain distinct. |

Public path appearance is retained. Owned queue paths use original blue procedural decks/marks and the same Flat/ramp surfaces. The gate is a platform, two posts, header and station-facing floor arrow within its two-step clearance; picking uses those actual faces, not a solid invisible clearance box. A doorway remains open. Selection/ghosts use physical depth; foreground terrain/decks can hide them. Gate action focus uses its visible header because the deck can be hidden. `geometric_paths_connect` supplies deck joins, including the gate's outward edge, independently of whole-chain readiness.

## 5. Controls and reproducible ride construction

| Control | Current behavior |
| --- | --- |
| P / Path | Public path tool. |
| J / Queue | Blue queue tool owned by the existing ride. |
| I / Gate | Entrance tool, facing toward the station. |
| K / Exit | Teal exit tool, facing toward the station; default South, independent of entrance facing. |
| C / shape or facing button | Queue: cycle Flat/North/East/South/West while staying Queue. Entrance/Exit: cycle that gate's facing. Other modes: enter public Path and cycle shape. |
| Page Up/Down / height buttons | Flat/gate height 0–30; ramp low height 0–29, high end low+1. Default 8. |
| Left click | One placement/removal; holding does not paint. Hidden or invalid targets reject without partial edits. |
| X / Demolish | Remove visible public/queue paths or entrance/exit. Delete acts on a currently visible selected path/gate. Active transfer walkways reject demolition. Track editing is absent. |
| B / V | Raise/lower one visible dry terrain column by one within 0–32; Water and Underground reject. |
| Escape / Inspect | Return to Inspect, with help closed first when open. |
| H / F1 | Help overlay; world edits are blocked while it is open. |
| F5 / F9 / Shift+F9 | Save / explicit primary Load / explicit Backup. Successful load pauses and clears transient tool/selection/camera state. |
| Space / 1 / 2 / 3 | Pause / 1× / 2× / 4× simulation. |
| O / F, ride buttons | Open/Close train and admission / find ride. Closing starts queue return; departed passengers finish the lap then unload. Paused commands never move guests. |
| Q/E, arrows/WASD, wheel or +/−, right/middle drag | Rotate, pan, zoom and drag camera; Home/Fit restores framing. |
| G / T / U / R | Grid / Flat–Terraced preset / Surface–Underground / restore public demonstration paths. |

T applies a terrain preset rather than undoing one edit. R retains explicit fixture-restoration semantics; it does not convert queue ownership or overwrite arbitrary player pieces. Help, UI hit rectangles, focus/drag, diagnostic/capture gates and pending Load/Backup suppress conflicting edits. Save runs after a same-update valid edit; pending Load/Backup suppresses edits even if loading fails.

For a small reproducible connected queue, pause the fresh park and use these **world** directions, independent of camera rotation:

| Piece | Internal zero-based tile | Inspector one-based tile | Shape/facing | Low height |
| --- | --- | --- | --- | --- |
| Existing station | (8,10) | (9,11) | West–East track | 8 |
| Gate | (8,11) | (9,12) | Facing North toward station | 8 |
| Queue at gate | (8,12) | (9,13) | Flat | 8 |
| Queue bridge | (8,13) | (9,14) | South ramp | 8 |
| Queue landing | (8,14) | (9,15) | Flat | 9 |
| Queue return | (8,15) | (9,16) | North ramp | 8 |
| Existing public entry | (8,16) | (9,17) | Flat | 8 |
| Exit | (8,9) | (9,10) | Facing South toward station | 8 |
| Exit landing and return | x8..13, y8 | x9..14, y9 | Public Flat paths | 8 |
| Return to existing network | x13, y9..11 | x14, y10..12 | Public Flat paths | 8 |

This is four queue tiles and capacity for four admitted guests. Resume an open ride to see guests approach on public paths and join the blue queue. Removing bridge (8,13) gives Disconnected; rebuilding restores Ready with a fresh ObjectId. Adding a Flat queue at (9,14), height 9, creates Branch. Removing public (8,16) gives NoPublicEntry. Removing the gate gives NoEntrance; rebuilding keeps the RideId and allocates a fresh gate ObjectId. These edits now also cancel active admissions and start physical return. An isolated guest stays stranded until a local route is restored. Native construction checks remain separate from source/capture evidence.

The exit's return path joins existing public (13,12); at (13,10) it fits under the raised lift. With both access routes connected, guests board at the stopped station, occupy up to six seats, ride and unload through the teal exit before walking on public paths again. Without an exit, the old approach/queue/patience behavior remains usable and the empty train still runs; no guests board. Removing an unused exit during a lap lets passengers finish that lap and wait at the station for repairs.

## 6. Queue/entrance API and navigation contract

`SurfaceObject.queue_owner: Option<RideId>` distinguishes public Path objects (`None`) from owned queue Path objects (`Some`). `ObjectKind::RideEntrance` is a separate physical Flat object with clearance 2; it is not a public path. `RideId(pub u64)` and its allocator are authoritative. `Park::ride()` exposes `Ride { id, entrance: Option<RideEntrance> }`, with `RideEntrance { object, facing }`. Facing points toward the station; the opposite edge connects to the owned queue. Entrance placement requires the station height and a lateral station edge rather than its track entry/exit ports.

Use `validate_queue_path`/`place_queue_path(pos, height, slope, ride)` and `validate_ride_entrance`/`place_ride_entrance(pos, height, facing, ride)`. Validation is immutable; rejected commands leave the whole park unchanged. Incomplete and disconnected construction is legitimate persistent state, not a reason to reject a save. `SurfaceObject::is_public_path()` is the membership predicate for ordinary guest navigation.

`Park::ride_access(ride)` derives `RideAccess { status, queue_tiles, ordered_queue, public_entry, station }`. Ready requires one simple owned chain from the gate's outer edge to exactly one compatible public boundary edge at the far end. Public contacts count as **edges**, not unique public ObjectIds. `ordered_queue` is populated only for Ready, in gate→public order. All owned fragments participate in validation. Statuses are Ready, RideUnavailable, NoEntrance, StationDisconnected, NoQueue, Disconnected, Branch, Loop, NoPublicEntry, MultiplePublicEntries, PublicEntryNotAtEnd, and ForeignQueue; their concise Display text explains incomplete access.

Keep the two graphs separate:

- `path_neighbors` and the cached public guest graph exclude all queues and gates. Ordinary and approaching guests use only public paths. Admission is an explicit transition granting access to that ride's queue; it never adds queue nodes to the public wandering graph. Queue/gate-only edits preserve that graph; public edits retain reconciliation/invalidation.
- `geometric_path_neighbors` and `geometric_paths_connect` drive physical deck joins. Public↔queue pieces can geometrically connect with compatible endpoint heights; an owned queue can meet its entrance's outward edge. This does not assert whole-chain Ready or guest permission.

Flat paths expose four edges and reserve two height steps. Slopes expose only the opposite low/high ends, rise one step and reserve three; low height is at most 29. Both queue and public shapes use the shared fixed-point sampler. Keep original vector order and stable-ID lookup semantics: valid saves can reorder object/guest arrays, so raw slot indices or binary search on unproven ordering are unsafe. Renderer caches must handle removal and same-tick loads that reuse IDs but change geometry.

Station linkage is re-derived from the current usable circuit; it is not a stale stored station ID. Circuit faults retain ride identity and physical gate metadata. Repair can reconnect a gate. Removing a gate clears its reference; rebuilding allocates a new object ID while preserving the ride. Ready describes topology. Admission additionally requires an open usable ride, capacity and no ongoing queue return. Boarding separately requires a usable exit. Payments and ratings remain absent.

### Guest admission and return contract

`GuestState` remains Walking/Waiting/Lost. Orthogonal `Guest.ride_visit: Option<RideVisit>` stores the ride, Approaching/Queueing/Leaving phase, optional ticket and waited ticks; `ride_cooldown` limits retries. Seeking follows a public BFS route to the entry and reserves no queue space. Admission assigns a monotonically increasing ticket. Capacity is one guest per owned queue tile, with current/next tile reservations preventing overlaps and overtaking. `ride_queue_order` reports ascending tickets; `ride_queue_stats` reports seeking, queueing, leaving, stranded, capacity, movement/waiting, oldest wait, admitting/draining and cumulative joined/turned-away/abandoned counts.

The oldest waiting guest's 1,800-tick (60 simulation second) patience limit, ride closure/fault or invalid access starts return for the single-file queue. Admissions stay suspended while it empties tail-first. A surviving movement edge finishes before reversal; cancellation alone does not teleport guests. Missing paths retain the existing local anchor rules, and admitted guests may recover only onto public paths or that ride's queue. Repairs use fresh ObjectIds. Returning guests leave by a physically reachable public connection, then have a 300-tick (10 simulation second) retry cooldown. Pause freezes movement and timers; an explicit edit/Close still changes intent immediately. Station boarding is the explicit transition beyond ordinary queue movement.

The renderer uses the authoritative positions, clipped path shadows and cached body/visit marker templates. Seeking, queueing and leaving markers distinguish purpose; Lost retains its warning. UI explains capacity, oldest wait, queue return and passenger status. No usable admission leaves the historical wandering/RNG behavior unchanged.

### Passenger journey contract

`ObjectKind::RideExit` and `Ride.exit: Option<RideEntrance>` describe the separate exit. Its facing points toward the station; its outward edge meets a matching public path. `validate_ride_exit` and `place_ride_exit` share immutable placement checks. `ride_exit_access` reports Ready, RideUnavailable, NoExit, StationDisconnected, NoPublicExit or PublicDisconnected. Boarding requires Ready, including a public route back to the queue's public entry. Physical unloading only requires the healthy station, exit and outward public landing: a broken entrance/queue must not trap riders. A severed distant public network can leave an unloaded visitor following ordinary local/Lost navigation.

`RideVisitPhase` now also includes Boarding/Riding/Exiting. Its `passenger: Option<PassengerJourney>` stores a stable seat 0..5, departure/completed-lap flags, and an optional `RideTransfer { path, portal, progress }`. Transfers take 60 simulation ticks through the path center, gate center and stopped seat; the boarding source remains reserved for all 60 ticks. Passengers clear ordinary current/next/destination/route references and use `Park::ride_guest_positions()` / `guest_position()` for authoritative positions. `Ride` counts boarded, exited and completed journeys; `Guest.rides_completed` counts completed laps followed by return to a public path. Predeparture cancellation is not a completed ride.

`object_removal_block_reason` protects occupied circuit pieces and active transfer paths/gates. `remove_object` preserves its Option signature and returns None without mutation when protected. While a passenger is seated on a departed train, the unused exit may be removed; they finish the lap and wait at the station until a valid local exit is restored. Transfers hold the train. Closing never teleports passengers, and paused saves retain exact seats and transfer progress. The standing guest mesh renders transfers; the train mesh renders seated riders from the same sampler, so `rendered_guests` counts standing emitted bodies only. No-exit diagnostic fixtures remain unchanged.

## 7. Saves, migration and recovery

Default user directory: `%LOCALAPPDATA%\Parkbound\saves`. Missing LOCALAPPDATA fails clearly; there is no silent working-directory/OneDrive fallback. `--save-dir` accepts an explicit absolute override for isolated tests. Normal startup neither loads nor saves automatically.

| File | Purpose |
| --- | --- |
| `park.park.json` | Primary slot, selected by Load/F9. |
| `park.park.json.bak` | Previous valid primary, selected explicitly by Backup/Shift+F9. |
| `park.park.json.lock` | Exclusive writer lock. Readers do not take it. |

Save serializes and validates before touching the slots, uses same-directory unique temporary files with write/sync, then atomic replacement without a delete-first gap. Only a **valid** old primary rotates into backup; a corrupt primary cannot displace a valid backup. Backup rollback preserves prior bytes on a failed primary replacement where possible. This is not one atomic transaction spanning both filenames. Failed load never mutates the live park or silently chooses backup. Successful Load/Backup pauses, clears selection/tools and forces static/dynamic refresh even when ticks match.

After a crash, close all Parkbound instances before deciding that the exact lock is stale and removing that lock. Never delete a lock belonging to a running writer. If the UI says `Recovery: see save folder`, preserve `.parkbound-rollback-*.tmp` files before manual recovery. They contain previous backup bytes, which may themselves require validation. Do not sweep unrelated temporary files or replace a valid slot blindly; keep copies and validate with the codec/isolated application first.

New saves are strict **schema 5/content 5**, with required exit/passenger/completion fields in addition to visit/cooldown, admission counters, ticket allocator and draining state. Queue ownership, gate metadata, movement anchors, routes and passenger seat/transfer state remain authoritative. The decoder validates role-specific access, unique tickets/seats/reservations, route/transfer geometry, timers and allocator/counter consistency. Unsupported pairs, unknown fields, invalid owners and ordinary guest references to queues or gates reject. Source bounds remain 8 MiB, 4,096 objects, 1,024 total paths including queues, 256 guests, 2,048 circuit pieces, and 65,536 total route IDs. The player app still accepts only 32 × 32 parks.

Explicit strict **1/1, 2/2, 3/3 and 4/4 migration** adds no exit or passengers and initializes completion counters to zero. Version 4 retains existing admission state, cooldowns, tickets and counters; versions 1–3 initialize no visits and zero cooldown. Versions 1/2 also supply earlier ownership/ride fields, and v1 supplies Flat shapes. Existing identities, movement, RNG, train state and allocators preserve their defined continuation. Strict old DTOs reject hybrid fields. Loading leaves the old file unchanged; Save writes 5/5. **0.15 intentionally rejects 5/5**, so keep closed-game copies before returning to older binaries. A migrated pre-0.15 connected queue adopts admission rules when resumed; v4 no-exit admission retains its behavior until the player builds an exit.

### Fresh-process continuation proof

Use the production executable, a new absolute directory with an existing parent, and separate writer/verifier processes. Never use a player's real save folder. This PowerShell pattern waits for the GUI executable and checks its actual exit code:

```powershell
$gameExecutable = (Resolve-Path '.\target\release\parkbound.exe').Path
$journeyProof = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound journey proof é ' + [guid]::NewGuid())
foreach ($proofMode in @('write-journey', 'verify-journey')) {
    $proofArguments = '--save-check {0} --save-dir "{1}"' -f $proofMode, $journeyProof
    $proofProcess = Start-Process -FilePath $gameExecutable -ArgumentList $proofArguments -WindowStyle Hidden -Wait -PassThru
    if ($proofProcess.ExitCode -ne 0) { throw "Proof failed: $proofMode ($($proofProcess.ExitCode))" }
}
Get-Content -LiteralPath (Join-Path $journeyProof 'verification-journey.txt')
```

The writer refuses an existing directory. The journey proof saves natural Boarding as backup and an occupied moving train as primary, then checks 7,200 ticks with closure, exit removal, station holding, fresh exit repair, protected walkway/track demolition, empty-circuit fault/repair and another completed ride. It validates exact state, positions, seats, identities, counters, paused loads and unchanged primary/backup bytes. The retained 3,600-tick admission proof still covers approach, occupied queues, Close/Open, queue/gate/public connection removal and repair.

Retain ordinary `write`/`verify`, `write-paths`/`verify-paths`, `write-terrain`/`verify-terrain`, `write-slopes`/`verify-slopes`, `write-queues`/`verify-queues` and `write-admission`/`verify-admission`, each in a fresh directory. Repeat the journey proof against the packaged executable. Migration checks use authentic 0.12 v1, 0.13 v2, 0.14 v3 and 0.15 v4 writers, plus old-reader rejection of 5/5 without file loss. Synthetic malformed-document tests and same-process roundtrips add coverage. Do not compare an old pre-admission queue's future against the intentional admission rules.

## 8. Runtime captures and diagnostics

Use new absolute PNG output names and inspect the resulting images. Capture mode disables normal input and exits after verified output; it is not native keyboard/mouse evidence. Warmup is at least 90 frames and two seconds, with a 120-second deadline. Normal capture size is 1280 × 800; `--capture-small-window` requests 1024 × 680. Options include `--capture-rotation 0..3`, `--capture-map flat|terraced`, `--capture-view surface|underground`, and `--capture-ticks 0..36000`.

Current production scene names:

| Family | `--capture-scene` values |
| --- | --- |
| Basic/overlap | `overview`, `help`, `selection`, `path`, `track` |
| Crowd | `crowd`, `lost`, `recovered` |
| Train | `train`, `train-lift`, `train-drop`, `train-brake`, `train-return`, `train-closed` |
| Persistence | `saved`, `loaded`, `backup`, `load-error` (use an isolated absolute save directory) |
| Public paths | `path-valid`, `path-blocked`, `path-built`, `path-demolish`, `path-removed`, `path-recovered` |
| Terrain | `terrain-raise`, `terrain-lower`, `terrain-raised`, `terrain-lowered`, `terrain-blocked`, `terrain-water`, `terrain-underground` |
| Slopes | `slope-valid`, `slope-built`, `slope-blocked`, `slope-crowd`, `slope-removed`, `slope-recovered` |
| Queues | `queue-entrance`, `queue-connected`, `queue-disconnected`, `queue-branch`, `queue-blocked`, `queue-rebuilt` |
| Admission | `admission-approach`, `admission-moving`, `admission-full`, `admission-leaving`, `admission-cancelled`, `admission-recovered` |
| Complete journeys | `journey-boarding`, `journey-riding`, `journey-unloading`, `journey-completed`, `journey-exit-blocked`, `journey-recovered` |

Example, from an existing output parent:

```powershell
$gameExecutable = (Resolve-Path '.\target\release\parkbound.exe').Path
$captureFile = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound queue é ' + [guid]::NewGuid() + '.png')
$captureArguments = '--capture-frame "{0}" --capture-scene queue-rebuilt --capture-rotation 2 --capture-map terraced --capture-view underground --capture-small-window' -f $captureFile
$captureProcess = Start-Process -FilePath $gameExecutable -ArgumentList $captureArguments -WindowStyle Hidden -Wait -PassThru
if ($captureProcess.ExitCode -ne 0) { throw 'Capture failed' }
```

For queues, inspect all four rotations, Surface/Underground, Flat/Terraced, connected/disconnected/branch/blocked/rebuilt cases, guest exclusion, physical entrance occlusion, and the compact access/tool/help panel at minimum size. Verify complete pixels and source identity, not just a PNG filename. Preserve source-capture pairs and compare retained normal scenes when rendering changes.

Controlled large-map previews use `--render-case baseline|save-scale|stress`, `--render-camera overview|crowd|ride`, `--render-rotation 0..3`, and optional `--render-underground`. They permit inspection/camera interaction but disable construction and persistence. Frame profiling additionally locks human input. Match fixture, view, camera, actual resolution and simulation workload before comparing.

`scripts/measure-performance.ps1` runs sequential child processes and records memory/process identity alongside JSON reports. CPU reports measure simulation/mesh work; `--profile-frames <new absolute .json>` records real main-loop intervals after warmup (default 600; `--profile-samples 120..3600`). `--profile-no-vsync` requests AutoNoVsync; otherwise AutoVsync is requested. `--profile-gpu` adds actual supported GPU timestamp diagnostics. The requested 1920 × 1080 window must be checked against actual size/scale/focus/visibility; occlusion can remain unknown.

Main-loop pacing, main-world CPU hooks, render-pass CPU spans and GPU timestamp spans are different quantities. Asynchronous GPU reports can have fewer samples than main frames; do not sum nested spans or invent a full-frame/presentation GPU time. Selected static chunks/vertices measure eligible geometry, not GPU draw calls or FPS improvement. Disabled diagnostics add no normal-play measurement work. No local performance batch or hardware acceptance is implied by the queue milestone.

## 9. Packaging, Git/CI and release checks

After the final release build, create the portable Windows package:

```powershell
pwsh -NoProfile -File scripts/package-windows.ps1 -SkipBuild
if ($LASTEXITCODE -ne 0) { throw 'Packaging failed' }
```

Use `-SkipBuild` only for a binary built from the exact intended source. Otherwise omit it and let packaging build. Version 0.16.0 outputs under `dist\Parkbound-0.16.0-windows-x64` with a matching `.zip` and `.zip.sha256`. Exact executable identity, package payload/guide counts, notices and relocation results belong in the journey report and `artifacts/ride-journeys-0.16.0/final-handoff.txt`. The adjacent ZIP checksum remains authoritative after documentation refresh.

The package includes the executable, required redistributable runtime, third-party and Rust standard-library notices, current README/resume/development plan/docs, measurement wrapper and payload manifest. Preserve notice sources, attribution and `.gitattributes` byte rules; newline normalization can invalidate recorded hashes. Do not drop dependency/license files because a local launch succeeds. There is no imported RCT art, sound or original game data. Source licensing and final naming/trademark review remain unresolved; free binary distribution does not itself grant an open-source license.

Verify a fresh extraction to a path containing spaces and a non-ASCII character, outside the checkout. Check every payload manifest hash and bundled guide against current sources, executable size/SHA-256 and runtime/notices inventory. Launch the extracted executable from `C:\Windows`, repeat relevant continuation/migration proofs and a matched capture, and record results. Do not call a same-PC relocation test a clean-PC test. Actual offline, standard-user installation/start/save behavior on a clean PC remains a separate release gate.

When documentation changes after runtime verification, retain the runtime source commit and binary identity, refresh package guides/manifest/ZIP, and reverify them. Do **not** place a mutable ZIP's own hash inside a bundled guide: that creates self-reference. The adjacent `.zip.sha256` is the canonical archive checksum.

`.github/workflows/windows.yml` runs on `windows-2022` with Rust 1.93.1: formatting, strict all-target release Clippy, workspace tests, packaging, ordinary/path/terrain/slope/queue/admission/journey fresh-process continuation, CPU-reporting smoke and artifact uploads. Permissions are read-only contents; uploads currently retain 14 days. Read the exact run for the actual source/PR rather than inferring green from configuration or an older run. Preserve useful receipts before hosted artifacts expire. A stacked draft PR is not a public release or proof that the branch has merged.

Before ending a stage, confirm the source branch is pushed, the draft PR targets the correct predecessor, exact CI completion is recorded, local source/docs are current, and final package guides match those docs. Commit only intended source/docs/locked dependency changes; generated builds and local raw captures stay in ignored output directories unless a curated evidence record is deliberately tracked.

## 10. Current verification and remaining checks

The current 0.16 implementation is undergoing source, continuation, migration, capture, package and hosted CI checks. No pass is implied until recorded here and in [RIDE_JOURNEY_REPORT.md](docs/RIDE_JOURNEY_REPORT.md). Raw evidence is retained under `artifacts/ride-journeys-0.16.0/`; this directory is ignored by Git, so another PC needs the shared folder or a deliberate transfer for raw receipts. The final handoff receipt records exact runtime source, binary identity, final guide/manifest/ZIP validation and remote cleanliness.

Historical 0.15 runtime source was `0c34f8d13139d3dc676505248dba0a9fa872cf65`, followed by checkpoint `965e3267b2ea8350067a12761fe339051b711e8d`. Its **306 release tests (200 game / 106 simulation)**, source/build checks, retained continuation proofs, authentic v1/v2/v3 migration, old-reader rejection and eleven inspected captures passed. [Exact-source Windows CI 39614500323](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39614500323) passed. Its executable was 46,683,648 bytes, SHA-256 `301a970a3b25ee20855a6d9181b4dc4862f94fffb639ed400d6a659a86c5ea30`; 30 payload hashes, 21 guides and 291 dependency notices were verified. Its extracted executable passed admission continuation and a byte-identical capture. These are historical results, not verification of the changed 0.16 source.

Historical native results remain narrowly scoped. 0.13 passed Space, wheel zoom, C/shape cycling, South-ramp placement, height buttons, X+click, F5, Save, F9 paused/reset load and Backup. 0.14's Load button restored an isolated connected queue fixture paused at 00:24, four linked tiles, twelve guests and Inspect; subsequent desktop input interrupted checks. In 0.15 the new isolated instance opened, but its first F9 attempt reported user input and automated inputs stopped. No 0.15 native control pass was claimed. Paths and receipts remain in their original reports/artifact directories; default player saves were not accessed.

Keep unverified native checks visible until directly exercised: J/I/K, gate facing and placement/demolition; live admission/boarding/ride/unload/exit repair; physical Delete, Page Up/Down and Shift+F9; B/V terrain controls; O/F ride controls; pan/drag/DPI/display behavior; clean-PC offline/standard-user operation; signing/licensing/title decisions; and representative performance hardware. Inconclusive automation is neither a pass nor proof that a control is broken. Do not erase a historical limitation without named current evidence.

Source input tests establish ordering and gating; captures establish rendered states; native checks establish actual keyboard/mouse interaction. Static pictures do not establish continuous motion. Same-PC relocation does not establish clean-machine compatibility. This PC remains excluded from hardware acceptance, and no local performance batch is required for this gameplay stage.

## 11. Next major gameplay milestone

Build a cohesive **ride economy and operating controls** stage: guest wallets, an editable ride ticket price, payment at a precisely defined boarding transition, ride revenue and park cash. Give players a clear ride panel with price, passenger totals, income and operation state. Define insufficient funds, closure before departure, refunded/cancelled boarding and save/load semantics before adding finance counters. Keep deterministic integer currency and validate conservation/accounting through complete rides and interrupted transfers. This should turn the existing journey into the first meaningful management decision.

Then add guest needs/satisfaction and ride ratings, multiple rides, a general track builder and park-management systems in further substantial stages. Do not conflate a single fixed circuit with a completed construction game. Keep map/save caps and original-art requirements until a scoped change deliberately revisits them.

For the next coding session: read this guide and the journey report, verify branch/source identity and a clean handoff, inspect remaining acceptance IDs, and complete one cohesive gameplay milestone with matching tests/runtime evidence. Update source, guides and package together before the next PC handoff.
