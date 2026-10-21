# Resume development on another Windows PC

Updated 21 October 2026 for the completed bounded **0.14.0 queue paths and ride entrance connectivity** implementation. Source is on `codex/queue-connectivity`, based on completed 0.13.0 checkpoint `effe6a8abe07b62be395c5efe72ffc1ce55d54a3`. Runtime source `4c8d42f00fe43a48ad5586240f153b833419d991` is pushed in [draft PR #12](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/12), stacked on open PR #11/`codex/sloped-paths`. Formatting, strict all-target release Clippy, all 285 release tests (190 game/95 simulation), release build, production migration/continuation, eleven inspected captures and package relocation passed. [Windows CI 39588999746](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39588999746) passed for that source. Native Load-button verification passed narrowly; new queue/gate controls still need native checks. See [the queue report](docs/QUEUES_REPORT.md) and section 10 for exact evidence and remaining gaps. The final documentation/package checkpoint is recorded in `artifacts/queue-connectivity-0.14.0/final-handoff.txt`; later documentation-head CI is recorded in PR #12 separately from the runtime-source run.

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

That second path is a **separate clone**, not evidence that OneDrive syncs it. Preserve both historical paths. Verify the actual folder, branch, and commit before continuing. The current stage starts from [draft PR #11](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/11), which was still open/unmerged when queue work began; that PR targets the preceding terrain branch. Do not assume `main` contains the latest playable stage or that dependent PRs have merged.

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
git switch --track origin/codex/queue-connectivity
```

If the branch already exists locally, use `git switch codex/queue-connectivity` instead. If it is absent from `origin`, finish/push the first PC's checkpoint before switching to an older branch unintentionally. Preserve local edits before branch changes; inspect divergence before any pull. Do not force-reset a checkout or overwrite another PC's work.

Use this handoff sequence:

1. On the active PC, finish the bounded change and its checks; update the milestone report and this guide. Inspect the diff and commit the intended files with a message describing the actual change. Push the branch and confirm the exact commit/PR/CI status.
2. Close the game, Cargo, editor tasks and Git operations using the shared folder. Let OneDrive finish syncing before the other PC opens it. Use only one active editing/build/Git session against that shared checkout.
3. On a separate clone, fetch and inspect the branch before `git pull --ff-only`. A non-fast-forward failure requires reconciling the two histories, not discarding one.
4. On the next PC, verify source identity, install the pinned toolchain, rebuild locally, and run the checks below. Copying `target` is unnecessary. Keep `target`, `dist`, and raw `artifacts` out of Git as configured.
5. Transfer saves separately with the game closed, preserving primary and backup together. Saves are outside the checkout by default; Git or OneDrive synchronization of source does not transfer them automatically.

Retain historical reports, raw benchmark data, evidence paths and notices. In particular, `benchmarks/results/2026-10-20-reference/` is a historical name, **not** designation of this PC as reference hardware. Do not rename or rewrite old evidence to imply current acceptance.

## 2. Read the project records

Start with [QUEUES_REPORT.md](docs/QUEUES_REPORT.md), [GAMEPLAY_CHECKLIST.md](docs/GAMEPLAY_CHECKLIST.md), [DEVELOPMENT_PLAN.md](DEVELOPMENT_PLAN.md), [PRODUCT_DECISIONS.md](docs/PRODUCT_DECISIONS.md), and [ASSET_PROVENANCE.md](docs/ASSET_PROVENANCE.md). The checklist's full feature criteria remain authoritative: a bounded implementation does not automatically complete BLD-02/03, the broader sloped-path criterion BLD-08, or the complete game.

Historical reports retain exact source, test, capture, native, CI, package, and checksum evidence without duplicating it all here:

| Version | Retained report |
| --- | --- |
| 0.13 | [One-step sloped paths](docs/SLOPED_PATHS_REPORT.md) |
| 0.12 | [Terrain editing](docs/TERRAIN_EDITING_REPORT.md) |
| 0.11 | [Path construction](docs/PATH_CONSTRUCTION_REPORT.md) |
| 0.10 | [Static visibility](docs/STATIC_VISIBILITY_REPORT.md) |
| 0.9 | [Large-map/GPU diagnostics](docs/LARGE_MAP_GPU_REPORT.md) |
| 0.8 / 0.7 | [Optimization](docs/OPTIMIZATION_REPORT.md) / [performance diagnostics](docs/PERFORMANCE_REPORT.md) |
| 0.6 / 0.5 | [Persistence](docs/SAVE_REPORT.md) / [train](docs/TRAIN_REPORT.md) |
| 0.4 / 0.3 | [Crowds](docs/CROWD_REPORT.md) / [overlap](docs/OVERLAP_REPORT.md) |
| 0.2 / 0.1 | [Terrain](docs/TERRAIN_REPORT.md) / [foundation](docs/FOUNDATION_REPORT.md) |

The completed 0.13 runtime source was `b1d26a04cc1282bb71defcb59557309b79252f1d`: 260 release tests, strict source gates, eleven inspected captures, continuation/migration, native construction/recovery, and relocated packaging passed. [CI 39583267705](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39583267705) verifies that runtime source; [CI 39584568173](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39584568173) verifies its later checkpoint. Keep those historical identities separate from the new 0.14 binary and documentation head.

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

Normal play remains a 32 × 32 deterministic demonstration, initially Terraced (grass/water/sand at heights 0/2/4/6), with Flat and player-edited Custom states. There are 37 public paths around a loop/junction/branches, twelve seeded guests, nine legacy track pieces, and the 46-piece Meadow Flyer circuit at x7..24/y4..10 with rail heights 8..16 and three cars. Fresh startup has **no queue or entrance**. Larger 256 × 256 scenes are controlled diagnostics, not general player-map/save support.

| Source | Responsibility and invariants |
| --- | --- |
| `crates/park_sim` | Window-independent deterministic park, bounded terrain/clearance, stable IDs, routes/RNG, crowds, train, ride/queue topology and strict save codec. Never make correctness depend on Bevy or wall-clock time. |
| `park_sim/src/path_geometry.rs` | Flat/four one-step ramps; endpoint matching; integer `PATH_UNITS = 60` positions. Guest center→edge→center heights and renderer share this geometry. |
| `park_sim/src/ride_access.rs` | Stable ride identity, entrance placement and derived owned-queue access. Public guest graph is separate. |
| `park_game/src/main.rs`, `path_tools.rs`, `path_tools/entrance.rs`, `terrain_tools.rs` | Input ordering, tool state, immutable preview validation, commit/gating, pause/view and scene refresh. |
| `park_game/src/world.rs`, `world/*` | Shared physical faces for painting/picking/selection, slope/gate/terrain ghosts, persistent guest rendering and sampled train geometry. |
| `park_game/src/static_landscape.rs` | Persistent static chunk slots and conservative visibility. Complete emission groups preserve internal geometry order; small maps retain one mesh. |
| `park_game/src/ui.rs` | Fixed control columns, inspector, train/access feedback, shared Build Tools panel and help. Minimum layout is 1024 × 680. |
| `persistence.rs`, `save_store.rs`, `save_check.rs`, `save_check/*` | Paused load/snapshot integration, bounded atomic file operations, isolated production continuation proofs. |
| `capture.rs`, `capture/*`, `*_input_tests.rs` | Controlled scenes, native-independent input regression harness and retained visual proofs. |
| `benchmark.rs`, `frame_probe.rs`, `gpu_probe.rs`, `render_scene.rs` | Opt-in diagnostics only; CPU time, main-loop intervals and actual GPU timestamp spans remain distinct. |

Public path appearance is retained. Owned queue paths use original blue procedural decks/marks and the same Flat/ramp surfaces. The gate is a platform, two posts, header and station-facing floor arrow within its two-step clearance; picking uses those actual faces, not a solid invisible clearance box. A doorway remains open. Selection/ghosts use physical depth; foreground terrain/decks can hide them. Gate action focus uses its visible header because the deck can be hidden. `geometric_paths_connect` supplies deck joins, including the gate's outward edge, independently of whole-chain readiness.

## 5. Controls and reproducible queue construction

| Control | Current behavior |
| --- | --- |
| P / Path | Public path tool. |
| J / Queue | Blue queue tool owned by the existing ride. |
| I / Gate | Entrance tool, facing toward the station. |
| C / shape or facing button | Queue: cycle Flat/North/East/South/West while staying Queue. Gate: cycle North/East/South/West facing. Other modes: enter public Path and cycle shape. |
| Page Up/Down / height buttons | Flat/gate height 0–30; ramp low height 0–29, high end low+1. Default 8. |
| Left click | One placement/removal; holding does not paint. Hidden or invalid targets reject without partial edits. |
| X / Demolish | Remove visible public/queue paths or the gate. Delete acts on a currently visible selected path/gate. Track editing is absent. |
| B / V | Raise/lower one visible dry terrain column by one within 0–32; Water and Underground reject. |
| Escape / Inspect | Return to Inspect, with help closed first when open. |
| H / F1 | Help overlay; world edits are blocked while it is open. |
| F5 / F9 / Shift+F9 | Save / explicit primary Load / explicit Backup. Successful load pauses and clears transient tool/selection/camera state. |
| Space / 1 / 2 / 3 | Pause / 1× / 2× / 4× simulation. |
| O / F, ride buttons | Open/Close automatic train / find ride. Queue readiness does not dispatch or board it. |
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

This is four queue tiles. It reports linked access with **no boarding**. Removing bridge (8,13) gives Disconnected; rebuilding restores Ready with a fresh ObjectId. Adding a Flat queue at (9,14), height 9, creates Branch. Removing public (8,16) gives NoPublicEntry. Removing the gate gives NoEntrance; rebuilding keeps the RideId and allocates a fresh gate ObjectId. Regression tests, production continuation and the inspected diagnostic fixtures cover these transitions; see the queue report for exact execution evidence. Native construction controls remain a separate open check.

## 6. Queue/entrance API and navigation contract

`SurfaceObject.queue_owner: Option<RideId>` distinguishes public Path objects (`None`) from owned queue Path objects (`Some`). `ObjectKind::RideEntrance` is a separate physical Flat object with clearance 2; it is not a public path. `RideId(pub u64)` and its allocator are authoritative. `Park::ride()` exposes `Ride { id, entrance: Option<RideEntrance> }`, with `RideEntrance { object, facing }`. Facing points toward the station; the opposite edge connects to the owned queue. Entrance placement requires the station height and a lateral station edge rather than its track entry/exit ports.

Use `validate_queue_path`/`place_queue_path(pos, height, slope, ride)` and `validate_ride_entrance`/`place_ride_entrance(pos, height, facing, ride)`. Validation is immutable; rejected commands leave the whole park unchanged. Incomplete and disconnected construction is legitimate persistent state, not a reason to reject a save. `SurfaceObject::is_public_path()` is the membership predicate for ordinary guest navigation.

`Park::ride_access(ride)` derives `RideAccess { status, queue_tiles, ordered_queue, public_entry, station }`. Ready requires one simple owned chain from the gate's outer edge to exactly one compatible public boundary edge at the far end. Public contacts count as **edges**, not unique public ObjectIds. `ordered_queue` is populated only for Ready, in gate→public order. All owned fragments participate in validation. Statuses are Ready, RideUnavailable, NoEntrance, StationDisconnected, NoQueue, Disconnected, Branch, Loop, NoPublicEntry, MultiplePublicEntries, PublicEntryNotAtEnd, and ForeignQueue; their concise Display text explains incomplete access.

Keep the two graphs separate:

- `path_neighbors` and the cached public guest graph exclude all queues and gates. Guests must never wander onto them, target them, or recover onto them while admission is absent. Queue/gate-only edits do not rebuild the public graph; public edits retain reconciliation/invalidation.
- `geometric_path_neighbors` and `geometric_paths_connect` drive physical deck joins. Public↔queue pieces can geometrically connect with compatible endpoint heights; an owned queue can meet its entrance's outward edge. This does not assert whole-chain Ready or guest permission.

Flat paths expose four edges and reserve two height steps. Slopes expose only the opposite low/high ends, rise one step and reserve three; low height is at most 29. Both queue and public shapes use the shared fixed-point sampler. Keep original vector order and stable-ID lookup semantics: valid saves can reorder object/guest arrays, so raw slot indices or binary search on unproven ordering are unsafe. Renderer caches must handle removal and same-tick loads that reuse IDs but change geometry.

Station linkage is re-derived from the current usable circuit; it is not a stale stored station ID. Circuit faults retain ride identity and physical entrance metadata. Repair can reconnect the same gate. Removing the gate clears its reference; rebuilding allocates a new object ID while preserving the ride. Open/Close train behavior remains independent of access. **Ready is connectivity only:** no admission, waiting line, boarding, exits, payments or ratings have been implemented.

## 7. Saves, migration and recovery

Default user directory: `%LOCALAPPDATA%\Parkbound\saves`. Missing LOCALAPPDATA fails clearly; there is no silent working-directory/OneDrive fallback. `--save-dir` accepts an explicit absolute override for isolated tests. Normal startup neither loads nor saves automatically.

| File | Purpose |
| --- | --- |
| `park.park.json` | Primary slot, selected by Load/F9. |
| `park.park.json.bak` | Previous valid primary, selected explicitly by Backup/Shift+F9. |
| `park.park.json.lock` | Exclusive writer lock. Readers do not take it. |

Save serializes and validates before touching the slots, uses same-directory unique temporary files with write/sync, then atomic replacement without a delete-first gap. Only a **valid** old primary rotates into backup; a corrupt primary cannot displace a valid backup. Backup rollback preserves prior bytes on a failed primary replacement where possible. This is not one atomic transaction spanning both filenames. Failed load never mutates the live park or silently chooses backup. Successful Load/Backup pauses, clears selection/tools and forces static/dynamic refresh even when ticks match.

After a crash, close all Parkbound instances before deciding that the exact lock is stale and removing that lock. Never delete a lock belonging to a running writer. If the UI says `Recovery: see save folder`, preserve `.parkbound-rollback-*.tmp` files before manual recovery. They contain previous backup bytes, which may themselves require validation. Do not sweep unrelated temporary files or replace a valid slot blindly; keep copies and validate with the codec/isolated application first.

New saves are strict **schema 3/content 3**. Required fields include nullable object `queue_owner`, park `ride` and `next_ride_id`. The decoder rejects unsupported version pairs, unknown fields, invalid owners/entrance references, and ordinary guest references to queues or gates. Source bounds remain 8 MiB, 4,096 objects, 1,024 total paths including queues, 256 guests, 2,048 circuit pieces, and 65,536 total route IDs. The player application still accepts only 32 × 32 parks.

Explicit strict **1/1 and 2/2 migration** makes old paths public, invents no queue/gate, and gives an existing train (including Fault) deterministic RideId 1. The 1/1 migration additionally supplies Flat shapes. Defined IDs/order/routes/RNG/train state/allocators and existing 2/2 slopes/anchors retain continuation. Old DTOs reject new-format fields rather than accepting hybrid documents. Loading does not rewrite the original file; the next Save writes 3/3. **0.13 intentionally rejects 3/3**, so keep closed-game copies before moving back to older binaries. Compatibility here is forward migration, not bidirectional new-format loading.

### Fresh-process continuation proof

Use the production executable, a new absolute directory with an existing parent, and separate writer/verifier processes. Never use a player's real save folder. This PowerShell pattern waits for the GUI executable and checks its actual exit code:

```powershell
$gameExecutable = (Resolve-Path '.\target\release\parkbound.exe').Path
$queueProof = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound queue proof é ' + [guid]::NewGuid())
foreach ($proofMode in @('write-queues', 'verify-queues')) {
    $proofArguments = '--save-check {0} --save-dir "{1}"' -f $proofMode, $queueProof
    $proofProcess = Start-Process -FilePath $gameExecutable -ArgumentList $proofArguments -WindowStyle Hidden -Wait -PassThru
    if ($proofProcess.ExitCode -ne 0) { throw "Proof failed: $proofMode ($($proofProcess.ExitCode))" }
}
Get-Content -LiteralPath (Join-Path $queueProof 'verification-queues.txt')
```

The writer refuses an existing directory. It makes a Ready backup and a Disconnected primary with Custom terrain and a closing train. Verification checks exact paused primary/backup state, 3,600 ticks and queue/public removal/replacement, gate demolition/rebuild and train commands. Every tick checks public-only guest references; fresh ObjectIds, stable RideId and unchanged slot bytes are asserted.

Retain ordinary `write`/`verify`, `write-paths`/`verify-paths`, `write-terrain`/`verify-terrain` and `write-slopes`/`verify-slopes` proofs, each in its own fresh directory. Repeat against the packaged executable. Migration evidence must also include authentic old 0.12 v1 and 0.13 v2 writers read by the new binary, and old-reader rejection of 3/3 without file loss. Synthetic malformed-document tests and same-process roundtrips are additional checks, not replacements for production migration/continuation.

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

Use `-SkipBuild` only for a binary built from the exact intended source. Otherwise omit it and let packaging build. Version 0.14.0 outputs under `dist\Parkbound-0.14.0-windows-x64` with a matching `.zip` and `.zip.sha256`. The verified runtime executable is **46,488,576 bytes**, SHA-256 `d5a2a741466ade1cab5d1649e9dfdc7f1fa7fdfa168375a4c232bfe62db75737`. The package verification matched 29 payload hashes and 20 guides; all 291 dependency entries/notices and the required runtime inventory were verified, retaining the preceding milestone's notice/runtime bytes. A fresh extraction in a Temp path containing spaces and `é`, launched from `C:\Windows`, passed queue continuation and produced a byte-identical minimum-window connected-queue Underground capture. The final guide/manifest/archive verification is recorded in `artifacts/queue-connectivity-0.14.0/final-handoff.txt`; the adjacent ZIP checksum remains authoritative.

The package includes the executable, required redistributable runtime, third-party and Rust standard-library notices, current README/resume/development plan/docs, measurement wrapper and payload manifest. Preserve notice sources, attribution and `.gitattributes` byte rules; newline normalization can invalidate recorded hashes. Do not drop dependency/license files because a local launch succeeds. There is no imported RCT art, sound or original game data. Source licensing and final naming/trademark review remain unresolved; free binary distribution does not itself grant an open-source license.

Verify a fresh extraction to a path containing spaces and a non-ASCII character, outside the checkout. Check every payload manifest hash and bundled guide against current sources, executable size/SHA-256 and runtime/notices inventory. Launch the extracted executable from `C:\Windows`, repeat relevant continuation/migration proofs and a matched capture, and record results. Do not call a same-PC relocation test a clean-PC test. Actual offline, standard-user installation/start/save behavior on a clean PC remains a separate release gate.

When documentation changes after runtime verification, retain the runtime source commit and binary identity, refresh package guides/manifest/ZIP, and reverify them. Do **not** place a mutable ZIP's own hash inside a bundled guide: that creates self-reference. The adjacent `.zip.sha256` is the canonical archive checksum.

`.github/workflows/windows.yml` runs on `windows-2022` with Rust 1.93.1: formatting, strict all-target release Clippy, workspace tests, packaging, ordinary/path/terrain/slope/queue fresh-process continuation, CPU-reporting smoke and artifact uploads. Permissions are read-only contents; uploads currently retain 14 days. Read the exact run for the actual source/PR rather than inferring green from configuration or an older run. Preserve useful receipts before hosted artifacts expire. A stacked draft PR is not a public release or proof that the branch has merged.

Before ending a stage, confirm the source branch is pushed, the draft PR targets the correct predecessor, exact CI completion is recorded, local source/docs are current, and final package guides match those docs. Commit only intended source/docs/locked dependency changes; generated builds and local raw captures stay in ignored output directories unless a curated evidence record is deliberately tracked.

## 10. Completed evidence, remaining checks and next bounded stage

The bounded queue ownership/entrance-connectivity implementation is complete for runtime source `4c8d42f00fe43a48ad5586240f153b833419d991`; the full gameplay criteria and native/release gaps remain open. Exact local receipts are retained under `artifacts/queue-connectivity-0.14.0/` and summarized in the tracked queue report. Raw artifacts are ignored by Git, so another PC may need the shared folder or deliberately transferred evidence to inspect them.

| Verification | Result for the runtime source |
| --- | --- |
| Source gates and build | Formatting, strict all-target release Clippy, 285 release tests (190 game/95 simulation), and production release build passed. |
| Production saves/migration | Authentic 0.12 schema 1 and 0.13 schema 2 writers migrated successfully. The old 0.13 reader rejected schema 3 without changing files; new schema 3 continuation still passed afterward. |
| Continuation | Ordinary/path/terrain/slope/queue fresh-process proofs passed. Queue verification covered exact primary/backup/paused states, 3,600 ticks plus mutations, stable RideId, fresh object identities and public-only guests. |
| Captures | Eleven PNGs were inspected: eight at 1280 × 800 and three at 1024 × 680. They cover all four rotations, Surface/Underground, Flat/Terraced, gate preview, connected/disconnected/branch/blocked/rebuilt states, help and overview. |
| Portable package | 29 payload hashes and 20 guides matched; dependency/notices/runtime verification and relocated queue proof passed. The relocated small connected cutaway PNG matched the original byte-for-byte. |
| Hosted CI | [Run 39588999746](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39588999746) completed successfully, including source gates, packaging, every continuation family, reporting smoke and uploads. |
| Native Load button | The extracted executable loaded a seeded Ready primary at 00:24, paused, with four linked queues, twelve public guests and Inspect tools. This closes the separate Load-button gap narrowly. |

The native session used an isolated Temp save directory and left default player files untouched. Automation stopped after two input-detected responses; refreshed state showed additional scene/zoom/tool/selection changes outside the intended actions. No cause or successful individual queue/gate action was inferred. The isolated test window was left available, without further automated input or forced closure. New native J/I/facing/queue/gate/demolition checks remain unverified. The final documentation/package receipt and documentation-head CI are tracked separately from the passing runtime source in `final-handoff.txt` and PR #12.

Source tests verify input ordering and geometry, captures verify rendered states, and native tests verify actual shortcuts/buttons; none substitutes for all three. Static images do not prove continuous motion, and this PC remains excluded from hardware acceptance.

Historical 0.13 native checks passed Space pause, wheel zoom, C/shape cycling, South-ramp placement, height buttons, X plus click, F5 save, Save button, F9 paused/reset load and explicit Backup restoration. A subsequent F5 reproduced the ramp save bytes exactly. These are useful retained evidence, not automatic verification of changed 0.14 controls.

Keep these gaps visible until directly checked: new J/I and Gate-mode C/facing/queue placement/demolition controls; physical Delete, Page Up/Down and Shift+F9; B/V and terrain buttons/clicks; O/F/ride controls; pan/drag/DPI/display behavior; clean-PC offline/standard-user operation; release signing/licensing/title decisions; and selection of a representative performance PC. Earlier automation attempts for Page Up/Shift+F9 were inconclusive, not proof that the shortcuts work or that native interaction is permanently blocked. The new Load-button result does not close those other gaps.

The next sensible bounded slice is **deliberate guest admission to this one ride's queue**: specify eligibility, a stable ordered membership model, capacity, waiting behavior, cancellation/rerouting when queue access or ride state changes, and exact save continuation. Public graph wandering must remain public-only; admission should be an explicit transition. Define mutation/fault/removal rules before animating visitors in a queue. Boarding/seats/dispatch, exits, spending, ride ratings, multiple rides, and track construction remain separate stages. Keep current map/save caps and original-art requirements until a later scoped change deliberately revisits them.

For the next coding session: read this guide and the current queue report, verify branch/source identity and clean handoff, inspect the checklist's remaining acceptance IDs, complete any unfinished current verification, then implement one bounded next stage with matching tests/runtime evidence. Update all affected records and package guides before the next PC handoff.
