# Resume development on another Windows PC

Updated 22 October 2026 for **0.22.0 ride construction costs and demolition refunds**, on `codex/ride-construction-economy` from completed 0.21 checkpoint `489bfff52c3ab44242c157d04d84d9d36e0906bb`. Carousel purchases and paid-basis refunds now affect park cash, with visible quotes, atomic refusal and retained ledger history. Section 10 and the [ride construction report](docs/RIDE_CONSTRUCTION_REPORT.md) record current verification; earlier named reports remain historical.

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

That second path is a **separate clone**, not evidence that OneDrive syncs it. Preserve both historical paths. Verify the actual folder, branch and commit before continuing. The current branch is `codex/ride-construction-economy`, based on `codex/second-ride-choice` / [PR #19](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/19), which was open and unmerged at stage start. The current draft and exact runtime/verification identities are recorded in section 10. Do not assume `main` contains the latest playable stage or dependent PRs have merged.

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
git switch --track origin/codex/ride-construction-economy
```

If the branch already exists locally, use `git switch codex/ride-construction-economy` instead. If it is absent from `origin`, finish/push the first PC's checkpoint before switching to an older branch unintentionally. Preserve local edits before branch changes; inspect divergence before any pull. Do not force-reset a checkout or overwrite another PC's work.

Use this handoff sequence:

1. On the active PC, finish the bounded change and its checks; update the milestone report and this guide. Inspect the diff and commit the intended files with a message describing the actual change. Push the branch and confirm the exact commit/PR/CI status.
2. Close the game, Cargo, editor tasks and Git operations using the shared folder. Let OneDrive finish syncing before the other PC opens it. Use only one active editing/build/Git session against that shared checkout.
3. On a separate clone, fetch and inspect the branch before `git pull --ff-only`. A non-fast-forward failure requires reconciling the two histories, not discarding one.
4. On the next PC, verify source identity, install the pinned toolchain, rebuild locally, and run the checks below. Copying `target` is unnecessary. Keep `target`, `dist`, and raw `artifacts` out of Git as configured.
5. Transfer saves separately with the game closed, preserving primary and backup together. Saves are outside the checkout by default; Git or OneDrive synchronization of source does not transfer them automatically.

Retain historical reports, raw benchmark data, evidence paths and notices. In particular, `benchmarks/results/2026-10-20-reference/` is a historical name, **not** designation of this PC as reference hardware. Do not rename or rewrite old evidence to imply current acceptance.

## 2. Read the project records

Start with [RIDE_CONSTRUCTION_REPORT.md](docs/RIDE_CONSTRUCTION_REPORT.md), [SECOND_RIDE_REPORT.md](docs/SECOND_RIDE_REPORT.md), [SERVICE_FINANCE_REPORT.md](docs/SERVICE_FINANCE_REPORT.md), [GUEST_SERVICES_REPORT.md](docs/GUEST_SERVICES_REPORT.md), [GUEST_EXPERIENCE_REPORT.md](docs/GUEST_EXPERIENCE_REPORT.md), [ECONOMY_REPORT.md](docs/ECONOMY_REPORT.md), [RIDE_JOURNEY_REPORT.md](docs/RIDE_JOURNEY_REPORT.md), [GAMEPLAY_CHECKLIST.md](docs/GAMEPLAY_CHECKLIST.md), [DEVELOPMENT_PLAN.md](DEVELOPMENT_PLAN.md), [PRODUCT_DECISIONS.md](docs/PRODUCT_DECISIONS.md), and [ASSET_PROVENANCE.md](docs/ASSET_PROVENANCE.md). The checklist's full feature criteria remain authoritative: a bounded implementation does not automatically complete BLD-02/03, the broader sloped-path criterion BLD-08, or the complete game.

Historical reports retain exact source, test, capture, native, CI, package, and checksum evidence without duplicating it all here:

| Version | Retained report |
| --- | --- |
| 0.22 | [Ride construction costs and demolition refunds](docs/RIDE_CONSTRUCTION_REPORT.md) |
| 0.21 | [Buildable second ride and guest choice](docs/SECOND_RIDE_REPORT.md) |
| 0.20 | [Service costs and finance ledger](docs/SERVICE_FINANCE_REPORT.md) |
| 0.19 | [Guest needs and services](docs/GUEST_SERVICES_REPORT.md) |
| 0.18 | [Guest preferences, satisfaction and ride ratings](docs/GUEST_EXPERIENCE_REPORT.md) |
| 0.17 | [Ticket prices, wallets and ride revenue](docs/ECONOMY_REPORT.md) |
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

Local tooling note for 0.20: an incidental editing command triggered Windows' Python install manager and installed Python 3.15.1 on this PC. Python is not a project dependency or build requirement; further edits used PowerShell/file patches. Do not install Python on another PC merely to build Parkbound.

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
| `park_sim/src/services.rs` | Needs, service access/lifecycle, retained financial records, deterministic selection and strict visit/accounting validation. |
| `park_sim/src/experience.rs` | Deterministic preferences, thoughts, satisfaction, review events and review validation. |
| `park_sim/src/finances.rs` | Recurring costs, supplies, debt-free closure, bounded transaction history, archive/opening balances and strict finance validation. |
| `park_sim/src/economy.rs` | Integer-cent wallets, fare validation/locking, departure payments, affordability and conserved revenue/cash. |
| `park_game/src/main.rs`, `path_tools.rs`, `path_tools/entrance.rs`, `path_tools/service.rs`, `terrain_tools.rs` | Input ordering, tool state, immutable preview validation, commit/gating, pause/view and scene refresh. |
| `park_game/src/world.rs`, `world/*` | Shared physical faces for painting/picking/selection, slope/gate/terrain ghosts, persistent guest rendering and sampled train geometry. |
| `park_game/src/static_landscape.rs` | Persistent static chunk slots and conservative visibility. Complete emission groups preserve internal geometry order; small maps retain one mesh. |
| `park_game/src/ui.rs`, `ui/economy.rs`, `ui/guests.rs`, `ui/services.rs`, `ui/finance.rs` | Fixed controls, cash/revenue/wallet and price/affordability displays, inspector, train/access feedback, shared Build Tools panel and help. Minimum layout is 1024 × 680. |
| `persistence.rs`, `save_store.rs`, `save_check.rs`, `save_check/*` | Paused load/snapshot integration, bounded atomic file operations, isolated production continuation proofs. |
| `capture.rs`, `capture/*`, `*_input_tests.rs` | Controlled scenes, native-independent input regression harness and retained visual proofs. |
| `benchmark.rs`, `frame_probe.rs`, `gpu_probe.rs`, `render_scene.rs` | Opt-in diagnostics only; CPU time, main-loop intervals and actual GPU timestamp spans remain distinct. |

Public path appearance is retained. Owned queue paths use original blue procedural decks/marks and the same Flat/ramp surfaces. The gate is a platform, two posts, header and station-facing floor arrow within its two-step clearance; picking uses those actual faces, not a solid invisible clearance box. A doorway remains open. Selection/ghosts use physical depth; foreground terrain/decks can hide them. Gate action focus uses its visible header because the deck can be hidden. `geometric_paths_connect` supplies deck joins, including the gate's outward edge, independently of whole-chain readiness.

## 5. Controls and reproducible ride construction

| Control | Current behavior |
| --- | --- |
| P / Path | Public path tool. |
| Y / Rides | Select an active ride, inspect it, or start building the carousel. Up/Down browses. |
| J / Queue | Blue queue tool owned by the selected ride (initially Meadow Flyer). |
| I / Gate | Entrance tool for the selected ride, facing toward its station or carousel edge. |
| K / Exit | Teal exit tool for the selected ride, facing toward its station or carousel edge; default South, independent of entrance facing. |
| C / shape or facing button | Queue: cycle Flat/North/East/South/West while staying Queue. Entrance/Exit/Service: cycle that building's facing. Other modes: enter public Path and cycle shape. |
| Page Up/Down / height buttons | Flat/gate height 0–30; service/carousel base 0–29; ramp low end 0–29, high end low+1. Default 8. |
| Left click | One placement/removal; holding does not paint. Hidden or invalid targets reject without partial edits. |
| X / Demolish | Remove visible public/queue paths, entrance/exit/service or an empty carousel (whole body and gates). Delete acts on a currently visible selected path/gate/service. Active transfer walkways reject demolition. Track editing is absent. |
| B / V | Raise/lower one visible dry terrain column by one within 0–32; Water and Underground reject. |
| Escape | Close Help/Guests/Services/Finance/Rides first; otherwise return to Inspect and clear selection. |
| Inspect | Return to Inspect when no overlay is open; use the overlay's Close button to dismiss it. |
| N / Finance | Cash/income/costs/net and paged transactions; Page Up/Down changes page; N/Escape closes. |
| M / Services | Facility construction/management. Up/Down browses, F6/F7 edits price, O opens/closes selected facility. C rotates outward service facing in build mode; height 0–29. |
| L / Guests | Guest inspection. Click/Up/Down selects; Page Up/Down changes the eight-row page; L/Escape closes. |
| H / F1 | Help overlay, mutually exclusive with Guests, Services and Finance. All suppress underlying edits and save/load, including opening/closing frames; Space can pause in Guests/Services/Finance. |
| F5 / F9 / Shift+F9 | Save / explicit primary Load / explicit Backup. Successful load pauses and clears transient tool/selection/camera state. |
| Space / 1 / 2 / 3 | Pause / 1× / 2× / 4× simulation. |
| O / F, ride buttons | Open/close the selected ride and admission / find selected ride. Closing starts queue return; departed passengers finish the lap then unload. Paused commands never move guests. |
| F6 / F7, Ticket −/+ | Lower/raise the selected ride fare by $0.50, within $0.00–$100.00. Existing boarders keep their locked fare. |
| Q/E, arrows/WASD, wheel or +/−, right/middle drag | Rotate, pan, zoom and drag camera; Home/Fit restores framing. |
| G / T / U / R | Grid / Flat–Terraced preset / Surface–Underground / restore public demonstration paths. |

T applies a terrain preset rather than undoing one edit. R retains explicit fixture-restoration semantics; it does not convert queue ownership or overwrite arbitrary player pieces. Help/Guests/Services/Finance/Rides, UI hit rectangles, focus/drag, diagnostic/capture gates and pending Load/Backup suppress conflicting edits. Save runs after a same-update valid edit; pending Load/Backup suppresses edits even if loading fails.

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

### 0.22 carousel construction economy

The full body costs 30000 cents. `carousel_build_quote(origin,height)` validates the full footprint and available cash; `carousel_demolition_quote(id)` validates that the body exists and is empty, returning half of its persisted paid basis. Place/remove revalidate before mutation. `CarouselRide::construction_cost_cents` and `demolition_refund_cents` survive removal; the new `RideConstruction`/`RideDemolitionRefund` ledger categories remain separate from ticket income, and opening/archived totals retain their owner. See the current report for strict save invariants and exact verification. The fixed coaster, paths, gates, terrain and service purchase/removal remain under their prior free-construction rules.

### Current carousel operation and ride choice

Y/Rides selects the ride targeted by J/I/K, O, F and F6/F7. Build carousel starts a three-by-three placement preview. There is one active carousel, six seats, 720-tick running cycles, 90-tick dwell and 60-tick transfers. Closing finishes its current cycle and unloads; a disconnected exit holds passengers. Empty removal clears the whole body and both gates atomically, retaining queue paths and historical accounts. Eight lifetime carousel records are retained; rebuilding uses a fresh RideId. Construction costs $300; successful empty demolition refunds 50% of the actual paid basis, currently $150. Migrated free bodies refund $0. Quotes are shown before commitment and rejected commands leave the entire park unchanged.

For a reproducible height-8 layout, place the carousel origin at internal (19,22), covering x19..21/y22..24. Give it Flat queue paths (17,21), (17,22), (17,23), an entrance (18,23) facing East, a public extension (20,20), and an exit (20,21) facing South. Inspector coordinates add one. The [second ride report](docs/SECOND_RIDE_REPORT.md) has the complete contract and verification fixture.

Guest choice considers eligible reachable rides using distance, queue length, fare, deterministic Coaster/Carousel preference and prior visits. Ties use RideId. Needs/services retain priority; existing trips retain ownership. Each ride has separate admission, seats, fare settlement, completion and ratings. Guest history sums and attributed opening/archived/recent ledger entries reconcile per ride, including removed records. UI selection does not change simulation ownership or draw randomness.

Core additions are `park_sim/src/carousel.rs` (atomic construction, runtime, samples, validation) and `ride_choice.rs` (candidate scoring). Existing `ride()`/`train()` explicitly mean the original coaster; use `all_rides`, `ride_by_id`, `ride_kind`, `object_ride`, `set_ride_open` and owner-specific stats for general behavior. The game adds `ui/rides.rs`, `world/carousel_render.rs`, `path_tools/carousel.rs`, `save_check/ride_choice.rs` and matching controlled scenes. Moving carousel geometry shares the existing dynamic ride mesh; static footprint geometry uses the existing landscape chunks.

## 6. Queue/entrance API and navigation contract

`SurfaceObject.queue_owner: Option<RideId>` distinguishes public Path objects (`None`) from owned queue Path objects (`Some`). `ObjectKind::RideEntrance` is a separate physical Flat object with clearance 2; it is not a public path. `RideId(pub u64)` and its allocator are authoritative. `Park::ride()` exposes `Ride { id, entrance: Option<RideEntrance> }`, with `RideEntrance { object, facing }`. Facing points toward the station; the opposite edge connects to the owned queue. Entrance placement requires the station height and a lateral station edge rather than its track entry/exit ports.

Use `validate_queue_path`/`place_queue_path(pos, height, slope, ride)` and `validate_ride_entrance`/`place_ride_entrance(pos, height, facing, ride)`. Validation is immutable; rejected commands leave the whole park unchanged. Incomplete and disconnected construction is legitimate persistent state, not a reason to reject a save. `SurfaceObject::is_public_path()` is the membership predicate for ordinary guest navigation.

`Park::ride_access(ride)` derives `RideAccess { status, queue_tiles, ordered_queue, public_entry, station }`. Ready requires one simple owned chain from the gate's outer edge to exactly one compatible public boundary edge at the far end. Public contacts count as **edges**, not unique public ObjectIds. `ordered_queue` is populated only for Ready, in gate→public order. All owned fragments participate in validation. Statuses are Ready, RideUnavailable, NoEntrance, StationDisconnected, NoQueue, Disconnected, Branch, Loop, NoPublicEntry, MultiplePublicEntries, PublicEntryNotAtEnd, and ForeignQueue; their concise Display text explains incomplete access.

Keep the two graphs separate:

- `path_neighbors` and the cached public guest graph exclude all queues and gates. Ordinary and approaching guests use only public paths. Admission is an explicit transition granting access to that ride's queue; it never adds queue nodes to the public wandering graph. Queue/gate-only edits preserve that graph; public edits retain reconciliation/invalidation.
- `geometric_path_neighbors` and `geometric_paths_connect` drive physical deck joins. Public↔queue pieces can geometrically connect with compatible endpoint heights; an owned queue can meet its entrance's outward edge. This does not assert whole-chain Ready or guest permission.

Flat paths expose four edges and reserve two height steps. Slopes expose only the opposite low/high ends, rise one step and reserve three; low height is at most 29. Both queue and public shapes use the shared fixed-point sampler. Keep original vector order and stable-ID lookup semantics: valid saves can reorder object/guest arrays, so raw slot indices or binary search on unproven ordering are unsafe. Renderer caches must handle removal and same-tick loads that reuse IDs but change geometry.

Station linkage is re-derived from the current usable circuit; it is not a stale stored station ID. Circuit faults retain ride identity and physical gate metadata. Repair can reconnect a gate. Removing a gate clears its reference; rebuilding allocates a new object ID while preserving the ride. Ready describes topology. Admission additionally requires an open usable ride, capacity, sufficient money and no ongoing queue return. Boarding separately requires a usable exit and rechecks affordability. Feedback ratings are tracked after completed public returns; layout/intensity ratings remain absent.

### Guest admission and return contract

`GuestState` remains Walking/Waiting/Lost. Orthogonal `Guest.ride_visit: Option<RideVisit>` stores the ride, Approaching/Queueing/Leaving phase, optional ticket and waited ticks; `ride_cooldown` limits retries. Seeking follows a public BFS route to the entry and reserves no queue space. Admission assigns a monotonically increasing ticket. Capacity is one guest per owned queue tile, with current/next tile reservations preventing overlaps and overtaking. `ride_queue_order` reports ascending tickets; `ride_queue_stats` reports seeking, queueing, leaving, stranded, capacity, movement/waiting, oldest wait, admitting/draining and cumulative joined/turned-away/abandoned counts.

The oldest waiting guest's 1,800-tick (60 simulation second) patience limit, ride closure/fault or invalid access starts return for the single-file queue. Admissions stay suspended while it empties tail-first. A surviving movement edge finishes before reversal; cancellation alone does not teleport guests. Missing paths retain the existing local anchor rules, and admitted guests may recover only onto public paths or that ride's queue. Repairs use fresh ObjectIds. Returning guests leave by a physically reachable public connection, then have a 300-tick (10 simulation second) retry cooldown. Pause freezes movement and timers; an explicit edit/Close still changes intent immediately. Station boarding is the explicit transition beyond ordinary queue movement.

The renderer uses the authoritative positions, clipped path shadows and cached body/visit marker templates. Seeking, queueing and leaving markers distinguish purpose; Lost retains its warning. UI explains capacity, oldest wait, queue return and passenger status. Without usable admission, guests can still wander and seek services; active needs intentionally change future choices and RNG consumption relative to older versions.

### Passenger journey contract

`ObjectKind::RideExit` and `Ride.exit: Option<RideEntrance>` describe the separate exit. Its facing points toward the station; its outward edge meets a matching public path. `validate_ride_exit` and `place_ride_exit` share immutable placement checks. `ride_exit_access` reports Ready, RideUnavailable, NoExit, StationDisconnected, NoPublicExit or PublicDisconnected. Boarding requires Ready, including a public route back to the queue's public entry. Physical unloading only requires the healthy station, exit and outward public landing: a broken entrance/queue must not trap riders. A severed distant public network can leave an unloaded visitor following ordinary local/Lost navigation.

`RideVisitPhase` now also includes Boarding/Riding/Exiting. Its `passenger: Option<PassengerJourney>` stores a stable seat 0..5, departure/completed-lap flags, and an optional `RideTransfer { path, portal, progress }`. Transfers take 60 simulation ticks through the path center, gate center and stopped seat; the boarding source remains reserved for all 60 ticks. Passengers clear ordinary current/next/destination/route references and use `Park::ride_guest_positions()` / `guest_position()` for authoritative positions. `Ride` counts boarded, exited and completed journeys; `Guest.rides_completed` counts completed laps followed by return to a public path. Predeparture cancellation is not a completed ride.

`object_removal_block_reason` protects occupied circuit pieces and active transfer paths/gates. `remove_object` preserves its Option signature and returns None without mutation when protected. While a passenger is seated on a departed train, the unused exit may be removed; they finish the lap and wait at the station until a valid local exit is restored. Transfers hold the train. Closing never teleports passengers, and paused saves retain exact seats and transfer progress. The standing guest mesh renders transfers; the train mesh renders seated riders from the same sampler, so `rendered_guests` counts standing emitted bodies only. No-exit diagnostic fixtures retain their boarding restrictions and automatic empty-train behavior; evolving needs still change guest state.

### Guest experience contract

Price limits are $4/$6/$8/$10; queue comfort is 15/25/35/45 seconds. Seed/ID hashing initializes traits without consuming RNG. Wallet checks precede value checks. The interested count requires both, independent of reachability/capacity. Front refusal safely drains the existing queue; a reserved fare stays locked. Comfort crossing costs five satisfaction points once; joined-visit cancellation costs eight once and gives no review. Refusal thoughts use cooldown without repeated satisfaction loss.

Only a completed physical public return creates one 1–5-star review, based on locked fare and frozen queue wait. It updates guest and ride counts/sums and satisfaction. Migration invents no historical reviews. `Guest.thought` and its timestamp are event state; the read-only Guests panel exposes their age, preferences, wallet, activity and review history. See the guest experience report for formulas, strict bounds and overflow handling. Needs and services are covered below; intensity preferences and rating-driven ride choice remain future work.

### Ticket economy contract

Money uses integer cents. Every fresh guest starts with $20.00 and zero spending; a fresh park starts with $1,000.00. Fresh rides and migrations from pre-economy schemas 1–5 start free; schema 6–8 migrations preserve saved fares, wallets and revenue. The player chooses a ticket price from $0.00 to $100.00 with $0.50 UI steps. A guest must afford the current fare to seek/join the ride, and the front of the queue is checked again before boarding. A price increase can therefore stop a guest who has not yet boarded; safe queue return and cooldown rules still apply.

Boarding locks the fare in the passenger state but does not transfer money. On the train's first actual departure, the locked amount moves once from the guest wallet to their spent total, the ride revenue and park cash. Price edits after boarding do not reprice existing passengers. Predeparture closure/cancellation costs nothing, so there is no refund transaction; a departed fare remains earned while the lap finishes or a missing exit is repaired. Free trips create no money. Save/load preserves the locked fare and paid flag and must never charge twice.

Accounting includes service expenses: `wallet + spent = 2000`; `sum(spent − service_spent) = ride revenue`; `sum(service_spent) = sum(all service revenue)`; `cash = 100000 + ride revenue + service revenue + demolition refunds − upkeep − supplies − construction`; `sum(services_used) = sum(service served)`. Retained demolished records participate. Payments occur only at actual departure or successful service completion; costs occur at running-cost deadlines and successful supply use. Carousel purchases/refunds occur on successful player commands even while paused; wallet replenishment, loans and a finance calendar remain absent.

### Guest needs and service contract

`services.rs` owns `ServiceId`, `ServiceKind`, `GuestNeeds`, `ServiceVisit` and derived access/stats. Initial needs 10–30 use seed/GuestId hashing without RNG draws. Growth is hunger/90, thirst/60, toilet/120 complete ticks, capped 100 and processed before journeys/reviews. At 60 a guest without a ride visit may seek the highest qualifying reachable/open/affordable service; ties use Food/Drink/Toilet, route length then stable ID. Critical 85 crossing costs 5 satisfaction once; unavailable services use thoughts/cooldown without blocking ordinary walking/rides.

Use `validate_service(pos,height,facing,kind)` / `place_service` with outward facing toward a matching PUBLIC path edge. Buildings reserve one tile and 3 height steps, maximum height 29; they are not public graph nodes. Use `services`, `service(id)`, `service_by_object`, `service_access`, `service_stats`, `guest_needs_summary`, `set_service_price` and `set_service_open`. Demolition leaves a retained record with `object=None`, preserving revenue/counts; maximum 256 total records including removed facilities.

Visitors follow public routes through Approaching/Waiting/Using; only one Using guest per facility, stable-ID waiter selection,90 use ticks, 900 wait timeout, 300 retry cooldown. Guests stay on the public access tile; no ordered physical service queue or indoor animation is claimed. Using locks an affordable quote; completion alone charges, resets that need to 20 and adds 5 satisfaction. Closure/removal/access loss cancels without payment/relief immediately. Valid walking edges finish normally. `Guest.spent_cents` remains total; `service_spent_cents` and `services_used` split receipts. Ride/service visits are mutually exclusive.

Try food (10,21), drink (15,21), toilet (18,21), height 8, facing North to the existing y=20 loop (inspector coordinates add 1). Defaults $2/$1/free; UI price steps 50 cents in 0–10,000. M manages facilities, L inspects needs. No facilities are prebuilt at ordinary startup. `world/service_render.rs` supplies original physical geometry for paint/picking/selection/ghosts; no external art is required. See the service report for limitations and evidence.
### Service operating costs and finance ledger

N/Finance shows current cash, ride/service income, running/supplies expenses, net result and paged transactions. Food/Drinks/Toilets cost $6/$4/$2 per simulation minute, paid $0.30/$0.20/$0.10 every 90 open ticks, plus $0.60/$0.25/$0.10 supplies per completed use. Disconnected open facilities still incur costs; closed/removed facilities stop accruing. Closing preserves partial progress; pausing freezes costs. Running costs precede service processing and ride departures in each complete tick. A locked sale can fund its own supplies, including from zero cash; free uses still consume supplies.

Insufficient cash closes the facility, cancels unfinished visits without payment/relief, and requires manual reopening. There is no debt. Failed upkeep retains due progress for the next open tick. Per-service expenses and funds-closure status persist, including demolished histories. The ledger keeps 128 recent entries plus opening and archived category/per-service history, preserving exact cash reconciliation after rollover. Zero-value income is omitted; free completed-use counts remain. See [SERVICE_FINANCE_REPORT.md](docs/SERVICE_FINANCE_REPORT.md) for strict rules, API, accounting and evidence.

## 7. Saves, migration and recovery

Default user directory: `%LOCALAPPDATA%\Parkbound\saves`. Missing LOCALAPPDATA fails clearly; there is no silent working-directory/OneDrive fallback. `--save-dir` accepts an explicit absolute override for isolated tests. Normal startup neither loads nor saves automatically.

| File | Purpose |
| --- | --- |
| `park.park.json` | Primary slot, selected by Load/F9. |
| `park.park.json.bak` | Previous valid primary, selected explicitly by Backup/Shift+F9. |
| `park.park.json.lock` | Exclusive writer lock. Readers do not take it. |

Save serializes and validates before touching the slots, uses same-directory unique temporary files with write/sync, then atomic replacement without a delete-first gap. Only a **valid** old primary rotates into backup; a corrupt primary cannot displace a valid backup. Backup rollback preserves prior bytes on a failed primary replacement where possible. This is not one atomic transaction spanning both filenames. Failed load never mutates the live park or silently chooses backup. Successful Load/Backup pauses, clears selection/tools and forces static/dynamic refresh even when ticks match.

After a crash, close all Parkbound instances before deciding that the exact lock is stale and removing that lock. Never delete a lock belonging to a running writer. If the UI says `Recovery: see save folder`, preserve `.parkbound-rollback-*.tmp` files before manual recovery. They contain previous backup bytes, which may themselves require validation. Do not sweep unrelated temporary files or replace a valid slot blindly; keep copies and validate with the codec/isolated application first.

New saves are strict **schema 11/content 11**, requiring paid carousel construction/refund amounts and aggregate/per-ride finance categories alongside all existing carousel runtime, guest histories, needs, services and money. Explicit versions 1–10 migration preserves every original park field. Legacy free carousels get zero paid basis/refund; all old cash, ledger IDs/ticks/order/balances and archive sources survive unchanged. Versions 1–8 retain their opening-baseline migration. No past purchase or refund entitlement is invented. Validation reconciles active/removed bodies, actual paid basis, refunded totals, ride ownership, opening/archived/recent ledger amounts and all existing relationships before replacing live state. Unknown/missing fields and hybrid old/new schemas reject. Bounds remain 8 MiB, 4,096 objects, 1,024 paths, 256 guests, 2,048 circuit pieces, 65,536 route IDs, 256 services and eight retained carousels; recent finance entries remain capped at 128. Normal play remains 32 × 32.

Explicit strict **1/1 through 10/10 migration** preserves original IDs, allocators, RNG, cash, active journeys, feedback, needs, services, carousels and existing finance data. Earlier-version defaults apply only where fields did not exist. Compare original state and deterministic current-rule continuations, not the old executable future. Loading leaves both files unchanged; Save writes 11/11. **0.21 intentionally rejects 11/11**; retain closed-game legacy copies.

### Fresh-process continuation proof

Use the production executable, a new absolute directory with an existing parent, and separate writer/verifier processes. Never use a player's real save folder. This PowerShell pattern waits for the GUI executable and checks its actual exit code:

```powershell
$gameExecutable = (Resolve-Path '.\target\release\parkbound.exe').Path
$rideProof = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound construction proof é ' + [guid]::NewGuid())
foreach ($proofMode in @('write-construction', 'verify-construction')) {
    $proofArguments = '--save-check {0} --save-dir "{1}"' -f $proofMode, $rideProof
    $proofProcess = Start-Process -FilePath $gameExecutable -ArgumentList $proofArguments -WindowStyle Hidden -Wait -PassThru
    if ($proofProcess.ExitCode -ne 0) { throw "Proof failed: $proofMode ($($proofProcess.ExitCode))" }
}
Get-Content -LiteralPath (Join-Path $rideProof 'verification-construction.txt')
```

The writer refuses an existing directory. The construction proof checks paid placement, refund/rebuild history, atomic refusals, saved paid basis and exact future continuation. The retained rides proof saves actual mid-boarding and concurrent paid-running states, then checks 24,000 ticks of separate fares, reviews, guest choices, closure and rebuilding. It verifies paused exact restoration and unchanged slots. The retained finance proof saves actual midperiod state and replays at least 24,000 complete ticks. It independently checks recurring costs and completion supplies, ledger rollover, balances, closure/reopening, retained demolition history, insufficient-funds closure and recovery through real income. The verifier checks exact state, paused loading, modal reset and unchanged primary/backup bytes. Retained services, feedback, journey and economy proofs continue to exercise needs, reviews, passenger repair and departure accounting under current rules.

Retain ordinary `write`/`verify`, `write-paths`/`verify-paths`, `write-terrain`/`verify-terrain`, `write-slopes`/`verify-slopes`, `write-queues`/`verify-queues`, `write-admission`/`verify-admission`, `write-journey`/`verify-journey` and `write-economy`/`verify-economy`, each in a fresh directory. Add `write-feedback`/`verify-feedback` for 18,000 ticks of experience and review-event continuation, `write-services`/`verify-services` for needs/service receipts, and `write-finance`/`verify-finance` for operating costs and ledger continuity. Repeat against the packaged executable. For authentic 0.12–0.21 writers use current `verify-legacy`, including occupied v5 journey, paid/reserved v6 economy, reviewed v7 feedback, occupied v8 service slots v9 finance/occupied-history slots and v10 dual-ride/removed-history slots. It compares every original park field and runs deterministic current-version continuations with round-trips, leaving both files unchanged and writing `verification-legacy.txt`. Old 0.21 rejection of 11/11 must leave both slots intact. Do not compare the new rules against an old executable's future.

The service proof saves real mid-approach backup and mid-use primary state. A 24,000-tick replay checks completion charges/relief, locked quotes, cancellation without purchase, access repair with fresh IDs, funds refusal and recovery, plus eventual ride return. Retained feedback checks include need-critical penalties before review updates.

## 8. Runtime captures and diagnostics

Construction scenes: `construction-price|construction-refund|construction-unaffordable|construction-finance|construction-removed`. They use actual quote/commit commands, including repeated loss-making build/removal until funds are insufficient. Cover all rotations and inspect the minimum-window price/refund/Finance text. Source scene tests distinguish state preparation from read-only presentation.

Second-ride scenes: `carousel-valid|carousel-blocked|carousel-built|dual-boarding|dual-riding|ride-choice|rides|ride-closed|ride-finance|carousel-rebuilt`. Use all rotations and both visibility modes; the Rides and owner-aware Guests/Finance panels must fit the minimum window.

Service scenes: `service-needs|service-approaching|service-using|service-served|service-blocked|service-recovered`. Needs shows the world, Using shows facility management, and the other scenes inspect the actual involved guest. Use all four rotations and Surface/Underground/minimum-window variants. Captures are functional/visual evidence only.

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
| Guest experience | `feedback-value`, `feedback-queue`, `feedback-satisfied`, `feedback-cancelled`, `feedback-recovered`, `feedback-guests` |
| Ride economy | `economy-reserved`, `economy-price-change`, `economy-earned`, `economy-cancelled`, `economy-unaffordable`, `economy-free` |
| Services | `service-needs`, `service-approaching`, `service-using`, `service-served`, `service-blocked`, `service-recovered` |
| Finance | `finance-ledger`, `finance-profit`, `finance-loss`, `finance-closed`, `finance-reopened`, `finance-removed`, `finance-history` |
| Second ride | `carousel-valid`, `carousel-blocked`, `carousel-built`, `dual-boarding`, `dual-riding`, `ride-choice`, `rides`, `ride-closed`, `ride-finance`, `carousel-rebuilt` |
| Construction economy | `construction-price`, `construction-refund`, `construction-unaffordable`, `construction-finance`, `construction-removed` |

Example, from an existing output parent:

```powershell
$gameExecutable = (Resolve-Path '.\target\release\parkbound.exe').Path
$captureFile = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound queue é ' + [guid]::NewGuid() + '.png')
$captureArguments = '--capture-frame "{0}" --capture-scene queue-rebuilt --capture-rotation 2 --capture-map terraced --capture-view underground --capture-small-window' -f $captureFile
$captureProcess = Start-Process -FilePath $gameExecutable -ArgumentList $captureArguments -WindowStyle Hidden -Wait -PassThru
if ($captureProcess.ExitCode -ne 0) { throw 'Capture failed' }
```

For queues, inspect all four rotations, Surface/Underground, Flat/Terraced, connected/disconnected/branch/blocked/rebuilt cases, exclusion of ordinary public wanderers, permitted admitted/returning guest movement, physical entrance occlusion, and the compact access/tool/help panel at minimum size. Verify complete pixels and source identity, not just a PNG filename. Preserve source-capture pairs and compare retained normal scenes when rendering changes.

Controlled large-map previews use `--render-case baseline|save-scale|stress`, `--render-camera overview|crowd|ride`, `--render-rotation 0..3`, and optional `--render-underground`. They permit inspection/camera interaction but disable construction and persistence. Frame profiling additionally locks human input. Match fixture, view, camera, actual resolution and simulation workload before comparing.

`scripts/measure-performance.ps1` runs sequential child processes and records memory/process identity alongside JSON reports. CPU reports measure simulation/mesh work; `--profile-frames <new absolute .json>` records real main-loop intervals after warmup (default 600; `--profile-samples 120..3600`). `--profile-no-vsync` requests AutoNoVsync; otherwise AutoVsync is requested. `--profile-gpu` adds actual supported GPU timestamp diagnostics. The requested 1920 × 1080 window must be checked against actual size/scale/focus/visibility; occlusion can remain unknown.

Main-loop pacing, main-world CPU hooks, render-pass CPU spans and GPU timestamp spans are different quantities. Asynchronous GPU reports can have fewer samples than main frames; do not sum nested spans or invent a full-frame/presentation GPU time. Selected static chunks/vertices measure eligible geometry, not GPU draw calls or FPS improvement. Disabled diagnostics add no normal-play measurement work. No local performance batch or hardware acceptance is implied by the queue milestone.

## 9. Packaging, Git/CI and release checks

After the final release build, create the portable Windows package:

```powershell
pwsh -NoProfile -File scripts/package-windows.ps1 -SkipBuild
if ($LASTEXITCODE -ne 0) { throw 'Packaging failed' }
```

Use `-SkipBuild` only for a binary built from the exact intended source. Otherwise omit it and let packaging build. Version 0.22.0 outputs under `dist\Parkbound-0.22.0-windows-x64` with a matching `.zip` and `.zip.sha256`. Exact executable identity, payload/guide counts, notices and relocation evidence belong in the ride construction report and `artifacts/ride-construction-0.22.0/final-handoff.txt`. The adjacent ZIP checksum remains authoritative after documentation refresh.

The package includes the executable, required redistributable runtime, third-party and Rust standard-library notices, current README/resume/development plan/docs, measurement wrapper and payload manifest. Preserve notice sources, attribution and `.gitattributes` byte rules; newline normalization can invalidate recorded hashes. Do not drop dependency/license files because a local launch succeeds. There is no imported RCT art, sound or original game data. Source licensing and final naming/trademark review remain unresolved; free binary distribution does not itself grant an open-source license.

Verify a fresh extraction to a path containing spaces and a non-ASCII character, outside the checkout. Check every payload manifest hash and bundled guide against current sources, executable size/SHA-256 and runtime/notices inventory. Launch the extracted executable from `C:\Windows`, repeat relevant continuation/migration proofs and a matched capture, and record results. Do not call a same-PC relocation test a clean-PC test. Actual offline, standard-user installation/start/save behavior on a clean PC remains a separate release gate.

When documentation changes after runtime verification, retain the runtime source commit and binary identity, refresh package guides/manifest/ZIP, and reverify them. Do **not** place a mutable ZIP's own hash inside a bundled guide: that creates self-reference. The adjacent `.zip.sha256` is the canonical archive checksum.

`.github/workflows/windows.yml` runs on `windows-2022` with Rust 1.93.1: formatting, strict all-target release Clippy, workspace tests, packaging, ordinary/path/terrain/slope/queue/admission/journey/economy/feedback/services/finance/rides/construction fresh-process continuation, CPU-reporting smoke and artifact uploads. Permissions are read-only contents; uploads currently retain 14 days. Read the exact run for the actual source/PR rather than inferring green from configuration or an older run. Preserve useful receipts before hosted artifacts expire. A stacked draft PR is not a public release or proof that the branch has merged.

Before ending a stage, confirm the source branch is pushed, the draft PR targets the correct predecessor, exact CI completion is recorded, local source/docs are current, and final package guides match those docs. Commit only intended source/docs/locked dependency changes; generated builds and local raw captures stay in ignored output directories unless a curated evidence record is deliberately tracked.

## 10. Current verification and remaining checks

Current 0.22 verification is in progress. Final source, executable, tests, production proofs, captures, package and GitHub results will be recorded here. Raw receipts are in ignored `artifacts/ride-construction-0.22.0/`; both PC paths above remain authoritative. No local performance batch is required and this PC is excluded as a reference.

### Historical 0.21 evidence

Runtime source **`b70edd79e3ab25ed4aa8f705d5a63f0dfcc438a5`** is pushed in [draft PR #19](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/19), stacked on #18. All **436 release tests (263 game / 173 simulation)**, formatting, strict all-target release Clippy and production build pass. Tests cover atomic full-footprint placement, independent queues/seats/transfers, real departures and reviews, closure/exit repair, removal/rebuilding, deterministic choice, pause/speed equivalence, owner-aware input/rendering and strict save corruption/migration. Runtime validation accounts for both running time and each cycle's initial dwell.

[Exact-source Windows push CI 39777803850](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39777803850) and [PR CI 39777812491](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39777812491) passed all **25 steps**, including packaging and the new fresh-process proof. The CI reporting smoke is not hardware acceptance. Later guide commits retain this runtime identity and have their own CI status.

The executable is **47,982,080 bytes**, SHA-256 **`9ae2649d227cc7fc6c13560825c1cdfb6da079c3e60aad9821e6488399d103ab`**. The portable package verifies **36 payload hashes and 27 current Markdown guides**. All **291 dependency entries** and five dependency/native/notices/runtime files are byte-identical to 0.20. A fresh spaces/é extraction launched from `C:\Windows` passed the 24,000-tick rides proof. Final guide/manifest/ZIP refreshes preserve this executable; the adjacent ZIP sidecar identifies the archive without self-referential hashes in bundled guides. Same-PC relocation does not establish clean-machine compatibility.

All **72 local production processes** passed: **54 save/proof processes** plus **18 inspected captures**. Authentic schema1–9 files were written by retained 0.12–0.20 executables. Occupied journey/economy/feedback/service/finance cases and an authentic v9 archived ride/service ledger preserve every original field and continue deterministically. Version0.20 rejects schema10 without altering either slot. All eleven retained/new continuation pairs pass. The new 24,000-tick oracle independently checks each owner's actual departure, locked fare, public-return review formula, history deltas and ledger source/balance; it also verifies price-sensitive alternative choices, closure and fresh-ID rebuilding.

All eighteen PNGs were visually inspected: fourteen second-ride variants across four rotations, both terrain/visibility modes and normal/minimum windows, plus retained Services, oldest Finance page, Help and overview. Rides, owner-aware Guests and Finance fit 1024×680 without clipping. The relocated package produced a byte-identical minimum-window Rides capture. Still images establish presentation separately from native gestures and continuous motion.

Native checks used the exact extracted executable and isolated copied slots. **F9** loaded both occupied rides paused at tick **1,977 / 01:05**, with **$1,009 cash**, four coaster passengers and five carousel passengers. **Y** opened Rides; the **next-ride button** selected Carousel #2; **Escape** returned to the park; **F7** changed only its fare from $1.00 to $1.50; **O** closed that carousel while retaining its paid cycle; **F5** saved. Comparing saved data proved unchanged time, coaster/train state, finances, geometry, RNG/allocators, all guest wallets/spending/history/reviews, and ten unaffected passenger/other-ride guest records. Backup bytes exactly match the original primary. **Alt+F4** closed the isolated instance; default player saves were untouched.

The **Down arrow in Rides was inconclusive** (no visible selection change); the button passed separately. F7 while the selector was open correctly left the fare unchanged: close with Use ride/Y/Escape before operating the selected ride. Native construction/demolition, remaining price gestures, direct continuous carousel motion and earlier unverified native controls remain open. Source tests and production proofs retain their distinct scope.

No local performance batch was run. This PC remains excluded from performance acceptance. Clean-PC/offline/standard-user compatibility, signing, source licensing, final naming and representative-hardware acceptance remain separate release gates. `final-handoff.txt` records the final documentation HEAD, archive identity and clean remote/local equality after push; the adjacent ZIP sidecar identifies the final archive.

Full contract and evidence are in [SECOND_RIDE_REPORT.md](docs/SECOND_RIDE_REPORT.md). Raw receipts are in ignored `artifacts/second-ride-choice-0.21.0/`. Both PC paths above remain authoritative handoff references; transfer raw artifacts/dist through OneDrive or deliberately, since Git carries source and guides. Prior named passes below remain historical.

### Historical 0.20 evidence

Runtime source **`06b1f0843775ad85553aa2fdc99e830d165a33fb`** is pushed on `codex/service-finances` in [draft PR #18](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/18), stacked on `codex/guest-services` / #17. All **414 release tests (252 game / 162 simulation)**, formatting, strict all-target release Clippy and the production build pass. [Exact-source Windows push CI 39768678035](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39768678035) and [PR CI 39768759947](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39768759947) both passed **24 steps**. Later documentation commits have separate CI status and preserve this runtime source/binary identity.

All **63 production processes** passed: authentic schema 1–8 migration, occupied legacy journey/economy/feedback/service preservation, unchanged-file rejection by 0.19, ten current continuation proof pairs and sixteen inspected captures. The **24,000-tick finance proof** independently checks ordered transactions and exact balances, running/supply costs, partial periods, locked sales, rollover, closure, demolition history, insolvency and recovery from real ride earnings. All sixteen PNGs are legible, including minimum-window Finance/Services and the oldest retained history page.

The executable is **47,597,568 bytes**, SHA-256 **`abc744b599c7da455362e080318bb7d8fd82bfa49d8c6a73900bb2ac79eb7dce`**. The **0.20 package verifies 35 payload hashes and 26 current Markdown guides**. All 291 dependency entries and five notices/runtime files are byte-identical to 0.19. Fresh spaces/é extraction launched from `C:\Windows` passed finance continuation and produced a byte-identical minimum-window history capture. Final documentation refreshes preserve the executable and rebuild the guide hashes, manifest and ZIP; the adjacent `.zip.sha256` identifies the archive.

Native **F9** loaded paused at tick 2,358 with $984.40 cash, 78 ledger entries and 18 open ticks toward each facility's next charge. **N** opened Finance and clicking **Older** showed page 2/10 with unchanged totals. A **Page Up** attempt detected user input; the next observation no longer showed the game, so desktop input stopped and the isolated QA instance was left open for the user. Keyboard paging, closing Finance, native modal save blocking and live expense/pause/re-save/closure/reopen gestures remain unverified here; source tests and process proofs have separate scope. Default player saves were untouched. The exact QA executable/PID/save directory are in `native-instance.json`.

The [service finance report](docs/SERVICE_FINANCE_REPORT.md) records the full contract and evidence. Raw receipts are in ignored `artifacts/service-finances-0.20.0/`: source-check logs, `runtime-verification.txt`, both capture-inspection receipts, capture hashes/PNGs, `native-verification.txt`, package/notices/relocation receipts and CI JSON. `final-handoff.txt` records the final documentation HEAD, package checksum and clean local/remote equality after push. Transfer raw artifacts/dist through OneDrive or deliberately; Git retains the source and guides. No local performance batch was run and this PC remains excluded. Broader native, clean-PC/offline/standard-user, signing/licensing/title and representative-hardware release acceptance remain open.

### Historical 0.19 evidence

Runtime source **`e0a54e9f6cf5b93ca4d5248c98427b49ab99082d`** is pushed on `codex/guest-services` in [draft PR #17](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/17), stacked on `codex/guest-experience` / PR #16, verified open and unmerged during this handoff. All **398 release tests (244 game / 154 simulation)**, formatting, strict all-target release Clippy and the production build pass. [Exact-source Windows push CI 39758297300](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39758297300) passed all **23 steps**; [PR CI 39758334277](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39758334277) also passed. Later documentation commits have separate CI status and retain this runtime identity.

All **55 production processes** passed: authentic v1–v7 migration, occupied legacy journey/economy/feedback migration, unchanged-file schema-8 rejection by 0.18, all nine current continuation pairs and fourteen captures. The new **24,000-tick service proof** checks exact mid-use/mid-approach loading, natural need growth, completion-only locked payments/relief, critical-crossing satisfaction, cancellation/access repair, tombstones/fresh IDs, funds refusal/free recovery, ride return and combined accounting. All **fourteen PNGs were visually inspected**, covering all rotations, both terrain/view modes and normal/minimum windows; the new Services and extended Guests panels fit. Captures establish presentation separately from native input.

The executable is **47,412,224 bytes**, SHA-256 **`7af3439ceb6e4d1a643d8fc97ed61d0e96bbc4555e72712c03caa89f798907d8`**. The portable **0.19 package verifies 34 payload hashes and 25 current Markdown guides**. All 291 dependency entries and five dependency/native/notices/runtime files remain byte-identical to 0.18. Fresh spaces/é extraction, launched from `C:\Windows`, passed service continuation and a byte-identical minimum-window served-guest capture. Final guide/manifest/ZIP refreshes preserve the executable; the adjacent `.zip.sha256` identifies the archive. Same-PC relocation is not clean-PC acceptance.

Native checks passed **F9 paused primary load**, the **Services button and next-facility button**, **F7 service price increase**, **O closure/cancellation**, **Escape**, **F5 save**, **L/Guests** and **Guest #5 row selection**. At tick 2,329 Drinks #2 changed from $1.00 to $1.50 and closed; its Using and Approaching visits cancelled. Comparing saved slots confirmed no tick advance, unchanged needs/satisfaction/wallets/spending/use counts for all twelve guests, $1,000 cash, no active service visits and an exact original primary in backup. The isolated QA instance then exited; default player saves were untouched. **M and Down attempts were inconclusive** with no visible response; native placement/facing/height/demolition, F6, free-use completion, modal save blocking and guest paging remain unverified. Source input tests and captures passed independently.

See [GUEST_SERVICES_REPORT.md](docs/GUEST_SERVICES_REPORT.md) for the full contract and scoped results. Raw receipts are in ignored `artifacts/guest-services-0.19.0/`: `tests.txt`, `clippy.txt`, `build.txt`, `runtime-verification.txt`, `capture-inspection.txt`, `capture-hashes.txt`, `captures/`, `native-instance.json`, `native-verification.txt`, `native-save-verification.txt`, `notices-comparison.txt`, `relocated-runtime-verification.txt`, `ci-runs.json`, `ci-jobs.json` and package receipts. `final-handoff.txt` records final documentation HEAD, package identity and remote/local equality after the final push. These raw files and `dist` require OneDrive or explicit transfer; Git retains source and guides. No local performance batch was run; this PC remains excluded. Remaining native, clean-PC/offline/standard-user, signing/licensing/title and representative-hardware release gates remain open.

### Historical 0.18 evidence

Runtime source **`efda12f22c292cc43dc21ba8c3246e4f77882b62`** is pushed on `codex/guest-experience` in [draft PR #16](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/16), stacked on unmerged `codex/ride-economy` / PR #15. All **367 release tests (230 game / 137 simulation)** passed. The last-review regression was strengthened with a valid matching-aggregate control, followed by another passing 137-test simulation run; production source did not change. Formatting, final strict all-target release Clippy and production build passed. The executable is **47,141,888 bytes**, SHA-256 **`b8c05e05dbab244ef0e1ca1d147e660fd1314c7bc04c70de4b59f6807e17a98c`**. [Exact-source push CI 39753833991](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39753833991) passed all **22 steps**, including tests, packaging, retained/new continuation proofs, reporting smoke and uploads. [PR CI 39753861050](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39753861050) also passed for that exact source. Later documentation commits retain this runtime identity and need their own CI status.

All **47 production process checks passed**. Authentic schemas 1–6 were written by the retained 0.12–0.17 binaries; additional authentic occupied v5 journey and paid/reserved v6 economy saves were included. The new reader preserves every original primary/backup park field, initializes experience without historical reviews, and runs 3,600 current-version ticks per slot with commands and repeated round-trips. Both original files remain unchanged. Version 0.17 rejects v7 without changing either slot. All retained construction/admission/journey/economy proofs pass, and the new feedback proof reproduces **18,000 complete ticks**, checking actual public-return review deltas, frozen fare/wait scoring, satisfaction, stable traits, cancellation without review, value refusal and recovery. Its backup restores real queue discomfort paused; its primary restores a reviewed park paused/Custom/Inspect with the guest panel reset.

All **twelve production PNGs were visually inspected**: eight feedback variants plus occupied journey, entrance preview, help and overview. Four rotations, Flat/Terraced, Surface/Underground and normal/minimum windows are represented. The Guests panel shows readable traits, money, activity, event age, satisfaction and reviews on both pages; blank final-page rows cannot select a guest. Queue discomfort shows 65/100 without a review; joined cancellation shows 62/100 without a completion/review; completed return shows 77/100 and 5/5; recovery shows six reviews and a 4.8/5 ride average. Help and construction controls still fit at 1024 × 680. Pictures establish presentation, not native gestures or continuous motion.

The portable **0.18 package verifies 33 payload hashes and 24 current Markdown guides**. All 291 dependency entries and the five dependency/native/notices/runtime files remain byte-identical to 0.17. A fresh extraction under a spaces/é path, launched from `C:\Windows`, passed the 18,000-tick feedback proof and produced a byte-identical minimum-window Guests capture. Guide/manifest/ZIP checks are refreshed after the final documentation update; the adjacent `.zip.sha256` is the final archive identity. Same-PC relocation does not establish clean-machine compatibility.

**New native controls remain unverified.** The extracted 0.18 QA instance opened with isolated temporary copied slots. The first F9 attempt reported detected desktop input; the subsequent observation did not show the game. All automated input stopped, and no load or new control outcome was inferred. The QA window was left open. L/Guests, row selection, paging, closing and native modal save blocking therefore have no current native pass; source input regressions and captures passed independently. Prior native results remain historical, including the unfinished 0.17 price gestures. Default player saves were never accessed.

Raw receipts are in `artifacts/guest-experience-0.18.0/`: `tests.txt`, `sim-tests-final.txt`, `clippy-final.txt`, `build-final.txt`, `runtime-verification.txt`, `capture-inspection.txt`, `capture-hashes.txt`, `captures/`, `native-instance.json`, `native-verification.txt`, `notices-comparison.txt`, `relocated-runtime-verification.txt`, `ci-jobs.json`, and package receipts. This ignored directory requires OneDrive or explicit transfer to another PC; it is not in Git. `final-handoff.txt` records exact runtime identity, final documentation HEAD, package checks and remote/local equality after the final push. No local performance batch was run; this PC remains excluded. Native, clean-PC/offline/standard-user, signing/licensing/title and representative-hardware release gates remain open.

### Historical 0.17 evidence

Historical 0.17 runtime source **`f9bfad2774989554cacb332163cc934063bcc84d`** is pushed on `codex/ride-economy` in [draft PR #15](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/15), stacked on unmerged `codex/ride-journeys` / PR #14. All **345 release tests (219 game / 126 simulation)**, formatting, strict all-target release Clippy and the production build passed. The 46,949,888-byte executable has SHA-256 **`5dc66d55ad9e8a702eb4f508dff5041bd388da772b3d690e6009addf9ab6b38c`**. [Push Windows CI 39740526253](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39740526253) passed all 21 steps; [PR CI 39740538860](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39740538860) also passed for that exact source. Later documentation commits retain this runtime identity; do not attribute their CI status without reading their own runs.

All **41 production process checks passed**: authentic schema 1–5 saves written by the retained 0.12–0.16 binaries migrate and continue; an authentic occupied v5 journey continues for 7,200 ticks; 0.16 rejects a v6 save without changing either slot; all retained path/terrain/slope/queue/admission/journey proofs pass; and the new economy proof matches 10,800 complete ticks with exact per-guest payment deltas, fare locking, paid completion, no-charge cancellation, unaffordable rejection, affordable recovery and an unpaid boarding backup. The speed-equivalence regression separately compares 14,400 complete ticks at all supported speeds. Its command schedule and the economy replay allow enough time for actual public returns after fare changes; completion assertions are retained.

All **twelve current production captures were visually inspected**: eight economy variants plus the retained occupied journey, gate preview, help and overview. Four rotations, Flat/Terraced, Surface/Underground and normal/minimum windows are represented. Prices, wallet ranges, revenue, affordability and help remain legible; the Build Tools panel fits above the footer at 1024 × 680. Still images verify presentation, not native gestures or continuous motion.

The portable 0.17 package verifies **32 payload hashes and 23 current Markdown guides**. Its 291 dependency entries, native manifest, third-party/Rust notices and bundled MSVC runtime are byte-identical to 0.16. A fresh spaces/é extraction launched from `C:\Windows` passed the 10,800-tick economy proof and produced a byte-identical minimum-window price-change capture. Final guide/manifest/ZIP checks are refreshed after documentation changes. The adjacent `.zip.sha256` identifies the current archive; it is intentionally not copied into a bundled guide.

Native verification is narrower: the extracted executable launched with an isolated temporary save directory, and actual **F9** loaded the paid primary paused with Custom/Inspect, clock 01:11, six riders, a $5 ticket, $28 revenue, $1,028 cash and a $15–$20 wallet range. The attempted F6 check was interrupted by detected user input; subsequent observation showed user-controlled resume/zoom. Automated input stopped and the QA window was left open for the user. New native F6/F7, Ticket buttons, save/cancellation gestures and native payment deltas remain **unverified**. Source input tests and the process proofs passed independently; they are not substituted for those manual checks. The QA instance's exact executable and temporary save directory are in `artifacts/ride-economy-0.17.0/native-instance.json`; it does not use default player saves.

Raw receipts are under `artifacts/ride-economy-0.17.0/`: `tests.txt`, `clippy.txt`, `build.txt`, `runtime-verification.txt`, `capture-inspection.txt`, `captures/`, `native-verification.txt`, `notices-comparison.txt`, `relocated-runtime-verification.txt`, `ci-jobs.json`, and package receipts. This folder is ignored by Git; another PC needs the shared folder or an explicit transfer for raw files. `final-handoff.txt` records runtime source, exact executable, final documentation HEAD, package verification and the remote branch matching local HEAD. No local performance batch was performed; this PC remains excluded from performance acceptance. Clean-PC/offline/standard-user compatibility and the broader release gates remain open.

### Historical 0.16 evidence

Historical 0.16 runtime source `25ed0a993f1c6b63dad2e659bc6b6c9dc93d5d38` is pushed in draft PR #14. All **326 release tests (210 game / 116 simulation)**, formatting, strict all-target release Clippy and production build passed. The 46,813,184-byte executable has SHA-256 `6815f2df25c0eb8e1fd2de06cc66a1d7be973927df8ab96d7da2b547b746a9a7`. [Exact-source Windows CI 39621993525](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39621993525) passed formatting, strict lint, tests, Windows packaging, all continuation proofs, reporting smoke and uploads. See [RIDE_JOURNEY_REPORT.md](docs/RIDE_JOURNEY_REPORT.md) for the complete scope and reproduction commands.

All 37 final-executable process checks passed: authentic v1/v2/v3/v4 migration, an occupied v4 admission save continued for 3,600 ticks, v5 rejection by 0.15 without changing either slot, all retained construction/admission proofs, and the new 7,200-tick passenger journey proof. That proof covers closure, missing-exit station holds, fresh-ID exit repair, protected occupied demolition, empty-train fault/repair, a boarding backup and another completed trip. Twelve final production PNGs are byte-identical to the twelve visually inspected originals: eight journey scene variants plus admission, entrance preview, help and overview, across all four rotations, both terrain/view variants, and normal/minimum windows.

The portable package contains 31 verified payload hashes and 22 Markdown guides. Its 291 dependency notices, native dependency manifest, third-party and Rust notices, and bundled MSVC runtime remain unchanged from 0.15. A fresh extraction launched from `C:\Windows` passed the 7,200-tick journey proof and produced a byte-identical minimum-window occupied-train capture. The final ZIP identity is in the adjacent `.zip.sha256`; guide updates require refreshing the manifest and archive as described above.

Native checks used the final extracted executable and isolated saves. F9 restored the completed primary at 03:37 paused; rapid left-Shift+F9 restored the occupied backup at 01:11 paused with six riders. A subsequent ordinary F9 loaded the primary again, and right-Shift+F9 restored the backup. K selected Exit facing South; C rotated it West. O opened and closed the ride while paused. The Save button persisted tick 2,159 with all twelve guest objects unchanged and rotated the previous primary intact. Escape returned to Inspect; 3 resumed at 4x, with the occupied train observed moving on the lift and later parked closed with zero seats and six completed trips. Space paused at 02:38; F5 saved tick 4,751 with boarded/exited/completed counts of six, six individual guest completions, and no remaining active visits. The Backup button then restored the occupied park paused with Inspect reset. Default player saves were untouched.

The rapid backup shortcut preserves held, just-pressed and just-released Shift state for the current update. This intentionally tolerates a modifier release in the same update as F9; aggregated input does not retain exact event ordering. Regression cases cover both Shift keys, same-update chords and a Shift held from the previous update, and establish that backup selection does not latch into a later ordinary F9.

Raw evidence is retained under `artifacts/ride-journeys-0.16.0/`: `tests-final.txt`, `clippy-final.txt`, `runtime-verification-final.txt`, `capture-inspection-final.txt`, `final-captures/`, `native-instance-final.json`, `native-verification-final.txt`, `notices-comparison-final.txt`, and package/relocation receipts. This directory is ignored by Git, so another PC needs the shared folder or a deliberate transfer for raw receipts. `final-handoff.txt` records exact runtime source, binary identity, final guide/manifest/ZIP validation and remote cleanliness after the final documentation push.

Historical 0.15 runtime source was `0c34f8d13139d3dc676505248dba0a9fa872cf65`, followed by checkpoint `965e3267b2ea8350067a12761fe339051b711e8d`. Its **306 release tests (200 game / 106 simulation)**, source/build checks, retained continuation proofs, authentic v1/v2/v3 migration, old-reader rejection and eleven inspected captures passed. [Exact-source Windows CI 39614500323](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39614500323) passed. Its executable was 46,683,648 bytes, SHA-256 `301a970a3b25ee20855a6d9181b4dc4862f94fffb639ed400d6a659a86c5ea30`; 30 payload hashes, 21 guides and 291 dependency notices were verified. Its extracted executable passed admission continuation and a byte-identical capture. These are historical results, not verification of the changed 0.16 source.

Historical native results remain narrowly scoped. 0.13 passed Space, wheel zoom, C/shape cycling, South-ramp placement, height buttons, X+click, F5, Save, F9 paused/reset load and Backup. 0.14's Load button restored an isolated connected queue fixture paused at 00:24, four linked tiles, twelve guests and Inspect; subsequent desktop input interrupted checks. In 0.15 the new isolated instance opened, but its first F9 attempt reported user input and automated inputs stopped. No 0.15 native control pass was claimed. Paths and receipts remain in their original reports/artifact directories; default player saves were not accessed.

Keep unverified native checks visible until directly exercised: J/I, actual entrance/exit placement and demolition, boarding from a player-built queue and live exit repair; physical Delete and Page Up/Down; B/V terrain controls and F ride focus; pan/drag/DPI/display behavior; clean-PC offline/standard-user operation; signing/licensing/title decisions; and representative performance hardware. Current K/C exit selection/facing, O ride operation, passenger lap completion and both Shift+F9 chords passed as described above. Inconclusive automation is neither a pass nor proof that a control is broken. Do not erase a historical limitation without named current evidence.

Source input tests establish ordering and gating; captures establish rendered states; native checks establish actual keyboard/mouse interaction. Static pictures do not establish continuous motion. Same-PC relocation does not establish clean-machine compatibility. This PC remains excluded from hardware acceptance, and no local performance batch is required for this gameplay stage.

## 11. Next major gameplay milestone

Extend the same purchase/refund rules to service facilities, with per-kind build prices, immutable previews, atomic cash rejection, persisted paid basis and attributed ledger entries. Then add a minimal station/straight/curve coaster track builder with explicit circuit validation. Carousel purchase/refunds are implemented in 0.22; broad construction, GST-03/MGT-01/MGT-02 and full-game criteria remain incomplete. Multiple active carousels, general track design, ride running costs, stock, wages, energy/nausea/resting, admission, loans and scenario finances remain absent.

For the next session: read this guide and the ride construction report; verify current branch/source, local edits and remote state; inspect the acceptance checklist; complete one cohesive stage with matching source/runtime evidence. Update source, guides and the portable package together. Preserve map/save caps and original-content requirements unless the next scoped change deliberately revisits them. This PC remains excluded from performance acceptance.