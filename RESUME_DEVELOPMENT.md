# Resume development on another Windows PC

This is the handoff guide for **RailRust-Tycoon**, the repository for an original Rust theme park simulation inspired by classic RollerCoaster Tycoon 1 and 2. The application currently displays **Parkbound**, a temporary working title; the repository name does not rename the application. Current milestone: static terrain chunk visibility **0.10.0**, 20 October 2026. Read [the static visibility report](docs/STATIC_VISIBILITY_REPORT.md) for current implementation and evidence. **No reference PC is selected:** this poorly performing development PC was explicitly rejected as a reference; its measurements are local diagnostics only. Hardware requirements and 60 FPS acceptance wait for an agreed representative PC.

Canonical repository: <https://github.com/strijbosmh-afk/RailRust-Tycoon>.

The repository contains the source, locked dependencies, build scripts, notices, and development records needed to rebuild on another PC. Generated executables, ZIPs, screenshots, and Cargo build caches are deliberately ignored by Git. Continue from the synced workspace below or clone the repository, then rebuild as needed; copying a previous PC's `target` folder is unnecessary.

### Shared OneDrive working folder

The current 0.10.0 session uses the original PC's shared OneDrive workspace. Its full local path, recorded on 20 October 2026, is:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
```

The path within that OneDrive root is `Documenten\ChatGPT\Railroad Tycoon`. On another PC, the Windows username or OneDrive root may differ; locate the synced folder and open it as the project workspace. The folder name remains `Railroad Tycoon`, while the GitHub repository is `RailRust-Tycoon` and the application's temporary title is `Parkbound`.

On the original PC, open the workspace in PowerShell with:

```powershell
Set-Location -LiteralPath 'C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon'
```

You can continue from that synced workspace or use a separate GitHub clone. The 0.3.0 checkout on the second PC was `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`; it is a separate local path, not evidence that OneDrive is syncing it. Both paths are retained intentionally. Verify Git state and the expected commit on whichever checkout you use. For a shared workspace, let OneDrive finish syncing and follow the two-PC workflow in section 9. The clone commands below are unnecessary when opening an existing checkout.

The current feature branch is `codex/static-terrain-visibility`, based on verified 0.9.0 checkpoint `96518d9`. The bounded implementation is complete in `ff2857e68bfa8217fb85fe3bd107312f385d490e`, pushed in [draft PR #8](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/8), stacked on unmerged PR #7. [Windows CI run 39440918757](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39440918757) passed for that source. Source/runtime/package evidence is recorded in the static visibility report; later documentation commits retain runtime identity but need their own CI status.

Historical 0.9.0 runtime source `5d49b7e00ce25692aac3ad2d6dda86939afd0a47` is in [draft PR #7](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/7), stacked on PR #6. [Windows CI run 39435077589](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39435077589) passed, along with local profiles/captures/continuation and final package verification of fifteen guides and 24 payloads. Historical 0.8.0 runtime `9460a9` plus wrapper correction `3f1ea88` remain in [draft PR #6](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/6), with successful CI run 39429282120. Inspect actual publication and merge state before switching PCs; cloning `main` may retrieve an older checkpoint.

Historical [Windows CI run 39417086959](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39417086959) passed for the train implementation. Its local ZIP passed all 19 payload hashes and eleven guide checks after extraction into a fresh path containing spaces and `é`. The relocated executable ran from `C:\Windows`, captured the parked train, exited 0, and produced an image byte-for-byte identical to the original. The train report retains that evidence; current measurement code requires its own checks. Documentation-only follow-up commits retain runtime source identity, but must be repackaged and their guides/hashes verified before handoff.

The retained crowd implementation is commit `20cbc76936d698bdb1f7a6dc2e22789d987b3d93`, followed by documentation checkpoint `51a79e8`. [Draft PR #2](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/2) targets the overlap branch and depends on [PR #1](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/1); they were unmerged when the train work began. [Windows CI for the crowd implementation](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39412913892) passed tests, formatting, linting, packaging, and artifact upload. Its local ZIP passed relocation and all 18 payload hashes. This historical evidence does not verify changed 0.10.0 code; record current evidence in the static visibility report.

## 1. Read this first

The user confirmed:

- Rust implementation, classic RCT1/RCT2 isometric construction and park management.
- Windows only, with free distribution and all functionality included.
- Development in playable stages, eventually covering the agreed complete classic gameplay inventory.

Current defaults are offline single-player, an independent simulation library, original assets, four isometric orientations, and versioned local saves. Windows 11 x64 is the first build/test target, not a proven minimum requirement. Multiplayer, mobile/console ports, and scripting are outside the current plan. Expansion content and original-file import compatibility remain undecided.

**The first executable is a foundation, not a finished management game.** Do not infer completion from the visible park or the existence of a package.

Read the project records in this order:

1. [Static visibility report](docs/STATIC_VISIBILITY_REPORT.md): completed 0.10.0 chunk/viewport contract, evidence, and next gameplay slice.
1. [Large-map/GPU report](docs/LARGE_MAP_GPU_REPORT.md): completed 0.9.0 preview, measurement semantics, and verification.
1. [Optimization report](docs/OPTIMIZATION_REPORT.md): completed 0.8.0 cache/rendering implementation and verification.
1. [Performance report](docs/PERFORMANCE_REPORT.md): historical 0.7.0 local diagnostics and corrected hardware designation.
1. [Save report](docs/SAVE_REPORT.md): historical 0.6.0 continuation acceptance.
1. [Train report](docs/TRAIN_REPORT.md): historical 0.5.0 acceptance evidence.
1. [Crowd report](docs/CROWD_REPORT.md): historical 0.4.0 acceptance evidence.
1. [Overlap report](docs/OVERLAP_REPORT.md): historical 0.3.0 overlap proof.
1. [Terrain report](docs/TERRAIN_REPORT.md): historical 0.2.0 evidence.
1. [Foundation report](docs/FOUNDATION_REPORT.md): historical 0.1.0 evidence.
1. [Product decisions](docs/PRODUCT_DECISIONS.md): confirmed direction, defaults, and open decisions.
1. [Gameplay checklist](docs/GAMEPLAY_CHECKLIST.md): acceptance IDs and their states.
1. [Development plan](DEVELOPMENT_PLAN.md): stages, architecture, and completion gates.
1. [Asset provenance](docs/ASSET_PROVENANCE.md): asset ownership and attribution.

This guide is a continuation aid. Update the checklist and milestone reports as evidence changes rather than letting this dated baseline override later results.

## 2. What exists today

| Area | Implementation / current 0.10.0 work |
| --- | --- |
| Park | Deterministic 32 × 32 site with grass, water, and sand. Starts in Terraced mode at heights 0/2/4/6; Flat mode remains available for comparison. |
| Network and overlap | 37 path nodes at height 8 form a loop, junctions, and branches around the original walkway. Nine track tiles remain at x13/y12..20, height 0. Each occupies two height steps. |
| Guests | Twelve seeded guests route over same-height cardinal path connections, walk between nodes, wait at destinations, and enter a defined lost state when disconnected. |
| Circuit and train | A 46-piece loop at x7..24/y4..10, with station at x8/y10, rails at heights 8–16, and three cars. Sampled integer motion, lift/descent/braking, station dwell, and finish-lap closure. |
| Presentation | Original terrain, connected path decks, sampled circuit rails/station/supports, and separate guest/train meshes. Surface mode hides buried track. Underground removes terrain and uses a non-selectable reference floor. No external game art is required. |
| Camera | Four rotations, keyboard and right/middle-drag panning, zoom, surface-height-aware fit, and grid toggle. |
| Inspection | Hover and selected-surface highlights with stable terrain/object identity; inspector distinguishes kind, object ID, tile, and height. Hidden surfaces cannot be picked through ordinary terrain. |
| UI | Pause/resume and 1x/2x/4x speeds, terrain T and underground U, Delete/Remove path, R/Restore paths, O/Open/Close, F/Find circuit, F5/Save, F9/Load, Shift+F9/Backup, help, crowd/train counters, and save status. |
| Simulation | Separate window-independent Rust library with bounded terrain, stable identities, shared clearance, atomic fixtures, deterministic crowds/RNG, validated circuit geometry, complete ticks, and a serde-based validated JSON codec. |
| Persistence | One primary slot and explicit previous-valid backup under LocalAppData; staged replacement, exclusive writer lock, bounded parsing/validation, and load paused with rebuilt presentation. No startup save/load. |
| Diagnostics | Surface/crowd presets plus train/lift/drop/brake/return/closed scenes, bounded tick/phase seeking, map/view/rotation/minimum-window options, PNG verification, and exit status. |
| Measurement | Opt-in CPU baseline/save-scale/stress fixtures; baseline main-loop frame profiling; sequential Windows wrapper with source/binary identity, hardware metadata, bounded processes, and 200 ms memory samples. Historical local diagnostic batches completed in 0.7/0.8. Version 0.9 completed controlled large-map profiles and optional actual GPU timestamp spans. Version 0.10 verifies total generated versus camera-selected static geometry; current evidence is in the static visibility report. |
| Delivery | Windows packaging, native-runtime inspection/copying, dependency/font notices, checksums, and a GitHub Actions workflow. |

**Not implemented:** general terrain/path/track editors, sloped paths, complete player construction, ride queues/boarding, guest needs/spending, general ride operations, coaster design/ratings, staff, economy, scenarios, research, audio, persistent settings, autosave, save browser, or migration support. Delete/Restore paths and Open/Close train are proof controls. Guests and cars are not individually selectable. Preset switching preserves authoritative state and object identities/heights; Flat exposes the original buried track. Each normal launch creates a fresh demonstration without touching a saved park; Load explicitly resumes it. Trees, entrance arch, underground reference floor, and circuit support posts are decorative.

**Version 0.10.0 completes the bounded static visibility slice.** Maps larger than 32 × 32 partition atomic terrain/scenery/surface emission groups into spatial chunks, retaining all their complete triangles and within-group order; the small demonstration retains its legacy mesh. Actual vertex bounds and a guarded full-window viewport select existing chunk entities on camera movement without rebuilding geometry. Static changes reuse slots and remove excess resources. The full-scene build remains the oracle; guest/train meshes and save behavior remain separate. The final diagnostic schema and 74 passing wrapper checks are recorded in the static visibility report; all 193 release tests (128 game/65 simulation), formatting, strict release Clippy, release build, and implementation CI passed. Eight image pairs match outside 709 version-line pixels each. Bidirectional 0.9/0.10 save continuation and all six GPU profiles passed; focused stress selected 13 of 212 chunks, or 149,556 of 2,251,764 generated static vertices. Package relocation also passed. These are correctness/work-selection results, not an FPS or hardware-acceptance claim.

**Version 0.9.0 is a completed historical diagnostic slice.** It adds controlled baseline/save-scale/stress previews, usable overview/crowd/ride cameras, all-rotation/visibility rendering, and optional actual GPU timestamp spans through Bevy. Editing and persistence are disabled in previews; ordinary 32 × 32 play and save/load acceptance remain unchanged. CPU hooks, render-span CPU, GPU spans, and main-loop pacing are distinct; upload and presentation timing remain unmeasured. All 178 release tests (113 game/65 simulation), formatting, strict Clippy, release compilation, and 44 isolated wrapper checks passed. Six 120-interval GPU profiles completed with actual timestamp samples, visible/focused 1920 × 1080 windows, and occlusion unknown. Six captures exited 0; current images were inspected. Bidirectional 0.8/0.9 continuation passed 3,600 ticks/commands, backup, and paused loading. The normal baseline image changed only its version digit. Implementation CI passed; relocated package checks passed; the final refresh verified all fifteen current guides and all 24 payload hashes in the large-map/GPU report. Do not carry historical results forward as verification of changed code.
**The bounded 0.8.0 optimization is complete; PRF-05 remains in progress.** All 159 release tests (65 simulation/94 game), formatting, strict Clippy, and release compilation passed. Four local CPU reports completed with paired animated workloads matched, zero measured graph rebuilds, and unchanged template/deck/buffer-growth counters. Four 0.7/0.8 image pairs differed only in the version digit; bidirectional cross-version continuation passed 3,600 ticks/commands, full state, backup, and paused loading. No reference PC is selected, and this development PC must not be treated as one. Hardware requirements and the proposed 60 FPS at 1080p acceptance wait for an agreed representative machine. The one-circuit model, 32 × 32 player presentation, and 256-guest save cap still limit the full proposed workload.

The current 0.10.0 executable is 46,168,064 bytes with SHA-256 `b3a63267dc450f1deab1a8a48aa1f93e93d0c4d6c9ba1e05bc2009f12bdb6c52`. The final refresh verified all 25 payload hashes and sixteen current guides. A fresh extraction path containing spaces and `é` ran from `C:\Windows`: 3,600-tick primary continuation/commands, full state, explicit backup, paused loading, and a byte-identical stress Ride/rotation-2 Surface capture all passed. The sibling wrapper selected its adjacent executable, reported no source checkout as expected, and passed one 30-tick CPU baseline plus both 120-interval save-scale Ride/rotation-2 Underground GPU profiles. The adjacent `.zip.sha256` is authoritative for the final archive.

The historical 0.9.0 executable is 46,109,696 bytes with SHA-256 `28df016814b5a86e0b9366ea0a380c284bcd9797f4c8c79fff29c61d64e17a88`. All 24 payload hashes matched after extraction to a fresh path containing spaces and `é`. Launched from `C:\Windows`, the extracted binary passed fresh-process 3,600-tick continuation/commands, explicit backup, paused loading, and a stress Ride/rotation-2 capture byte-identical to the original. The sibling wrapper selected its adjacent executable and passed a 30-tick CPU baseline plus two 120-main-interval save-scale Ride/rotation-2/Underground GPU profiles. Both actual windows were 1920 × 1080; 98/95 GPU samples per pass were received for AutoVsync/AutoNoVsync. These are functional checks, not hardware acceptance. The final refresh verified all fifteen current guides and all 24 payload hashes.

The historical 0.8.0 45,947,392-byte executable has SHA-256 `f907dbfc3effdcd460ab5e84ef15bf58cf623f9321231f89dd4a2c2f99ba6c24`. All 23 extracted payload hashes matched in a fresh temporary path containing spaces and `é`. From `C:\Windows`, the extracted executable passed continuation/backup/paused-load verification, and the sibling wrapper passed baseline 30-tick CPU plus two 120-sample frame smoke checks. The frame windows were actually 1920 × 1080, visible/focused, with occlusion unknown; these are functional checks, not an FPS comparison. The final documentation refresh verified all 23 payload hashes and fourteen current guides against the workspace. The optimization report records exact source/CI/package identities and the adjacent archive checksum rule.

Historical 0.7.0 local diagnostics passed 147 release tests, strict source checks, fourteen measurement processes, hosted CI, and package relocation with 22 payload hashes/thirteen guides. They recorded 37.94–40.15 main-loop FPS and 114.61–135.45 ms per stress guest-mesh rebuild on this PC. These remain local observations, not reference-hardware results or a target pass/fail. Original raw JSON/checksums/path names are preserved at `benchmarks/results/2026-10-20-reference/`; the historical name does not confer reference status. Changed 0.10.0 code requires its own evidence.

**Historical PRF-04 is verified narrowly for same-build authoritative continuation in 0.6.0.** All 131 workspace tests passed: 56 simulation, 27 world/rendering, 28 input-system, five persistence, nine store, four main-module, and two capture tests. They include Windows failed-primary replacement rollback, bounded reads, rejection without mutation, full continuation/cache/ID checks, input precedence/gating, and same-update mesh refresh after loading. Formatting, strict Clippy, release build, and twelve inspected GPU captures passed; all controls/status/version fit at 1024 × 680.

The historical 0.6.0 45,597,184-byte executable has SHA-256 `eb2b21b97e6053dd256034ade6cd17d5cac0c7ca7770db8873f17aab791f04fd`. Its 20 payload hashes and twelve guides matched after extraction into a fresh temporary path with spaces and `é`. Separate extracted processes launched from `C:\Windows` passed 3,600-tick continuation plus commands, explicit backup, and paused load; the loaded-small PNG matched the original byte-for-byte. Native testing was attempted with an isolated extracted instance, but another app obscured it; no keyboard/mouse input was issued and only that test process was closed. **Native Save/Load/Backup remains unverified.** Implementation `d7a16f6` is pushed in draft PR #4 and passed Windows CI run 39422425034; these results do not verify changed 0.10.0 code.

**PRF-03 is verified for the 0.5.0 automatic circuit and fixed-update behavior.** All 99 tests passed: 43 simulation, 27 world/rendering, 23 input-system, four main-module, and two capture tests. Formatting, strict Clippy, and release compilation passed. Sixteen GPU captures completed with exit code 0 and were inspected, including all rotations, lift/descent/braking, completed lap and parked train, minimum-window layouts, terrain/views, and crowd/overlap regressions. The train report records the executable hash and observed counters. A Bevy scheduling regression produced equal whole-Park state over 120 simulated seconds at 30/60/144 Hz update schedules; this is not measured hardware performance.

Normal launch at 1280 × 800 showed lap 1 on the lift at 01:04. Space paused at 01:15 with lap 1, Running at 0.94 tiles/s, twelve guests, and 116 destinations reached. Human desktop activity then obscured the game, so **native O/F and Open/Close/Find buttons remain unverified**; the F attempt had no unambiguous observed result. Carry these retests and delivery checks forward. The narrow PRF-03 acceptance does not accept all controls or the entire foundation.

**The historical 0.4.0 crowd proof is verified (PRF-02).** All 72 tests passed: 30 simulation, 20 world/rendering, 18 input-system, and four main-module tests. Formatting, release Clippy with warnings denied, and release compilation passed. Eleven engine captures completed with exit code 0 and were visually inspected, including all four crowd rotations, isolated/recovered guests, minimum-window layouts, Flat terrain, and Underground track. The [crowd report](docs/CROWD_REPORT.md) records its executable SHA-256, successful hosted CI, and package relocation with 18 matching payload hashes.

Native input through `@oai/sky` verified normal path removal/restoration at 1280 × 800. Space paused at 00:21 with 29 destinations reached; wheel zoom changed 100% → 162%. Removing occupied Path #37 with the button cleared selection, left twelve guests and the paused time intact, and showed one orange lost guest. R restored the same coordinate as fresh Path #47. Resume recovered the guest; pausing later at 00:42 showed 59 arrivals. A second remove/restore-button cycle, help open/close without click-through, and native close also passed. Two helper-injected Delete attempts had no effect: the headless shortcut test passes, but **physical Delete remains a retest**. Use the verified Remove path button in the meantime.

The **0.3.0 overlap proof is verified (PRF-01)**. Source commit `0220c7b` passed [Windows CI run 39406042854](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39406042854): 51 tests, formatting, release Clippy with warnings denied, and Windows packaging. All 14 engine captures were inspected: Surface and Underground in four rotations, Flat in both views, and minimum-window path, track, hidden-selection, and help scenes. Every process exited with code 0. The relocated package ran from `C:\Windows` after extraction into a path with spaces and `é`; all 17 payload hashes matched. See the overlap report for the exact executable hash and detailed evidence.

Historical 0.3.0 native input at 1280 × 800 selected Path #7 (display tile 15,17, height 8), revealed track with U, then selected Track #18 (14,18, height 0). The visibility button hid the track while retaining its ID and burial hint. Space paused at 02:14; Flat exposed the same track, all four rotations preserved identity/time, wheel zoom changed 100% → 113%, and Fit restored 100%. Hills hid the track again; clicking its old projection selected foreground terrain (12,19, height 6). Help/close preserved selection/time, and the native close button exited the process.

Historical **0.2.0** evidence includes 34 passing tests, formatting, Clippy with warnings denied, seven inspected engine captures, and a release Windows ZIP. Its 16 payload hashes matched after extraction into a separate path containing spaces and `é`; the extracted executable rendered selected raised terrain from an unrelated working directory and exited with code 0. See the terrain report. Clean-machine testing remains open; these results do not verify changed 0.10.0 code.

The historical **0.1.0** foundation report records 16 passing tests, release compilation, formatting, Clippy, three inspected images, and a relocated ZIP launch with matching payload hashes. Keep these results attached to their named checkpoint.

Normal desktop input was checked in 0.2.0 at 1280 × 800. Roof and visible cliff selection updated the inspector; T and the preset button preserved the selected tile while changing its elevation; rotation preserved selection; wheel zoom, Fit, and grid toggle worked. The timer advanced at 4x, Space paused it at 02:36, opening/closing help preserved that time and selection, Resume restored 4x, and the native close button ended the process. These observations plus authoritative clock tests verify FND-03.

Keyboard panning was not clearly established in the 0.3.0 native session, and right/middle dragging was unavailable through the desktop helper. Physical display scaling and multiple monitors remain untested, so FND-02 stays in progress. Clean-machine testing without development tools is also pending. These gaps do not invalidate the demonstrated PRF-01 render/pick-layer behavior; do not treat its completion as overall foundation or distribution acceptance.

### Checklist state at this handoff

| IDs | State | Meaning |
| --- | --- | --- |
| DEF-01, DEF-02 | Planned | Exact content/expansion audit and final product/platform decisions remain open. |
| FND-01 | Verified | Reverified in 0.10.0: the final refresh matched all 25 package payload hashes and sixteen current guides; relocated continuation/backup/paused-load, byte-identical 256-map capture, and sibling-wrapper CPU/GPU checks passed from C:\Windows. Historical native evidence remains separate; clean-machine compatibility remains open. |
| FND-02 | In progress | Native selection, four rotations, wheel zoom, and Fit passed in 0.3.0. Visible panning, right/middle dragging, and physical DPI/display checks remain open. |
| FND-03 | Verified | Earlier speed/inspector/visibility evidence is retained; 0.4.0 reverified native pause/resume, path-edit buttons, counters, and help, with complete-step speed/pause tests. |
| FND-04 | Verified | Reverified in 0.10.0 through 193 release tests, formatting, strict release Clippy, release compilation, and successful Windows CI run 39440918757 for source ff2857e. The static visibility report records exact evidence; later heads need their own CI status. |
| REL-01 | In progress | Packaging is implemented and a ZIP was checked; do not silently change its checklist state without reviewing its acceptance evidence. |
| PRF-01 | Verified | Named 0.3.0 tests, captures, native identity/visibility/preset checks, and package relocation establish the overlap baseline. New changes require regression checks. |
| PRF-02 | Verified | 0.4.0 automated invariants, eleven inspected captures, and native path deletion/restoration/recovery establish the proof. Physical Delete shortcut retesting remains explicit; Remove path is verified. |
| PRF-03 | Verified | 0.5.0 automatic circuit/fixed-update behavior passed 99 tests, sixteen captures, and normal launch showing lap 1; Space paused. Native O/F/buttons and physical hardware frame-rate performance remain open. |
| PRF-04 | Verified | 0.6.0 technical continuation: 131 tests, production/extracted fresh-process replay/backup, twelve inspected captures, and relocated package hashes/guides passed. Native save controls remain unverified. |
| PRF-05 | In progress | No reference PC is selected and this development PC is excluded as a reference. The bounded 0.8.0 guest-rendering/connectivity optimization is complete and verified. Version 0.9.0 verifies isolated large-map rendering and actual GPU diagnostics through source tests, captures, bounded profiles, continuation, and CI; relocated package checks passed. The final refresh verified all fifteen current guides and all 24 payload hashes. Version 0.10.0 static terrain chunk visibility is complete: 193 tests, 74 wrapper checks, eight exact image pairs outside version text, six GPU profiles, bidirectional save continuation, CI, and package relocation passed. Full-target hardware acceptance and the unsupported 50-ride workload remain open. |
| Remaining gameplay/release entries | Planned | Consult the checklist for individual acceptance requirements. |

Stage 1 is not fully accepted until its remaining panning/dragging, display, and fresh-machine requirements pass. Stage 2 has completed the stated PRF-01 through PRF-04 technical criteria and continues with PRF-05 measurement; the entire technical-proof stage and unverified controls remain unfinished.

## 3. Set up the new PC

Use a Windows 11 x64 development PC with a working graphics driver. A desktop session and compatible GPU are required to run/render the game; simulation tests do not need a window. Initial development used an Intel UHD 630 and approximately 8 GB RAM. Those are recorded observations, not minimum requirements or a performance guarantee.

Install these prerequisites from their official distributions:

| Prerequisite | Required setup |
| --- | --- |
| Git for Windows | Git available in the terminal; authenticate to the repository if needed. |
| PowerShell 7 | Use `pwsh`, particularly for packaging scripts. Do not rely on legacy Windows PowerShell 5 behavior. |
| Visual Studio 2022 or Build Tools 2022 | Select **Desktop development with C++**, including x64/x86 MSVC tools, a Windows SDK, and the MSVC redistributable files. The packaging helper currently explicitly locates Visual Studio 2022, not an arbitrary newer version. |
| Rust through rustup | Install toolchain `1.93.1-x86_64-pc-windows-msvc`, with `rustfmt`, `clippy`, and `rust-docs`. |

The repository's `rust-toolchain.toml` pins the tested toolchain and components. The `rust-version = "1.88"` field in the workspace manifest is a declared lower bound, not the toolchain used for the verified foundation. Use 1.93.1 for this baseline. Bevy is pinned exactly to **0.17.3**; retain `Cargo.lock` and use `--locked`.

On the second PC used for 0.3.0, pinned Rust and requested components were installed under `%USERPROFILE%\.cargo\bin`; persistent PATH was left unchanged. If a fresh PowerShell cannot find Cargo on that PC, enable it for the current process with:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
```

That second PC lacked MSVC build tools and the Windows SDK because their installer could not obtain UAC elevation; its verified 0.3.0 runtime/package came from CI. The current 0.10.0 session uses the original OneDrive development PC, which built the crowd and train checkpoints locally. Inspect the actual toolchain on whichever PC is active rather than carrying the other machine's installation state forward. Install missing prerequisites before local builds, or use the hosted Windows workflow.

Open a new PowerShell 7 terminal after installing the prerequisites. These commands inspect the tools and install the pinned Rust toolchain without changing your global default toolchain:

```powershell
git --version
pwsh --version
rustup --version
rustup toolchain install 1.93.1-x86_64-pc-windows-msvc --profile minimal --component rustfmt --component clippy --component rust-docs
```

For a fresh-clone alternative to the shared OneDrive workspace, choose a development directory you own. The example directory below is only a convenient convention:

```powershell
New-Item -ItemType Directory -Path "$env:USERPROFILE\source" -Force
Set-Location "$env:USERPROFILE\source"
git clone https://github.com/strijbosmh-afk/RailRust-Tycoon.git
Set-Location RailRust-Tycoon
git fetch origin
git switch --track origin/codex/static-terrain-visibility
git status --short --branch
git remote -v
rustup show active-toolchain
rustc -vV
cargo --version
```

The branch-switch example assumes the 0.10.0 feature branch has been published and has not been merged elsewhere. Inspect `git branch -a` and the current PR before substituting another branch; a plain clone initially checks out the default branch, which may be older. If the local feature branch already exists, use `git switch codex/static-terrain-visibility` instead of `--track`.

Confirm `rustc -vV` reports version 1.93.1 and host `x86_64-pc-windows-msvc`. Do not switch this project to the GNU Windows toolchain: the runtime inspection and packaging scripts assume MSVC.

If Git requests authentication, use the normal Git credential manager or GitHub authentication flow. Do not put access tokens in repository URLs, files, shell scripts, or this document. Each PC may need its own authentication and Git commit identity.

Before making commits on the new PC, set a repository-local identity using your actual name and a verified GitHub email or the no-reply email shown in your GitHub email settings. Replace the example values before running:

```powershell
git config --local user.name "YOUR_NAME"
git config --local user.email "YOUR_VERIFIED_OR_GITHUB_NOREPLY_EMAIL"
```

## 4. Rebuild and establish the baseline

Run these from the repository root in PowerShell 7:

```powershell
cargo fetch --locked
cargo fmt --all --check
cargo test --release --locked --workspace -j 2
cargo clippy --release --locked --workspace --all-targets -j 2 -- -D warnings
cargo build --release --locked --package park_game --bin parkbound -j 2
cargo run --release --locked --package park_game --bin parkbound -j 2
```

The first Bevy build compiles a substantial graphics dependency tree. On the initial 8 GB machine, the first release build took about 26 minutes with two compiler jobs. Subsequent changes reuse the cache. Reserve sufficient disk space for dependencies and `target`, and allow the first build to finish rather than repeatedly clearing its cache. Use `-j 1` if two jobs still exhaust memory. Release-profile checks are intentional: the foundation was validated in that profile, and using one profile avoids compiling separate large dependency sets unnecessarily.

For a fast simulation-only iteration:

```powershell
cargo test --release --locked --package park_sim -j 2
```

This does not replace workspace checks for a change that also affects rendering, UI, dependencies, or packaging.

The development executable is `target\release\parkbound.exe`; the packaged executable is named `Parkbound.exe`. Both refer to the same application. Close a running copy before overwriting that executable during a new build or package check.

### Controls to verify manually

| Input | Expected result |
| --- | --- |
| Left click on map | Select the nearest visible terrain/path/track surface; inspector identity and highlight agree. |
| Right or middle drag | Pan without a jump when dragging begins. |
| WASD / arrow keys | Pan with the keyboard. |
| Wheel / + / - | Zoom while preserving usable selection behavior. |
| Q / E | Rotate through four views; selected simulation surface remains the same. |
| Home / Fit | Reset zoom and pan; preserve selection. |
| G | Toggle grid. |
| T / terrain preset button | Switch Flat ↔ Terraced while preserving time, materials, object IDs/heights, and selected identity. |
| U / visibility button | Switch Surface ↔ Underground; hidden terrain and the decorative reference floor cannot be selected underground. |
| Delete / Remove path | Remove the selected path, clear stale selection/navigation references, and preserve unrelated objects. Non-path selections must be rejected with a status message. |
| R / Restore paths | Atomically restore missing demonstration paths with new IDs; preserve time, guest state/IDs, and surviving object IDs. |
| O / Open or Close | Toggle train operation; closing in motion finishes the lap, closing at station parks it. This does not pause the park. |
| F / Find | Center the circuit at 150% zoom while preserving authoritative state and selected identity. |
| F5 / Save | Save the current authoritative simulation; a valid existing primary becomes the backup. |
| F9 / Load | Load the primary into a validated candidate, then replace the park paused and reset presentation. |
| Shift+F9 / Backup | Explicitly load the previous valid backup paused; do not silently fall back on primary failure. |
| Space | Pause; resume the previously chosen speed. |
| 1 / 2 / 3 | Choose 1x / 2x / 4x simulation speed. |
| H / F1 | Toggle help. |
| Escape | Close help, otherwise clear selection. |
| Window close / Alt+F4 | Exit. |

Also test toolbar buttons, help-close click consumption, selection beside UI borders, dragging after focus changes, minimum window size, and display scaling. In both terrain presets and visibility modes, rotate through all four views and inspect the shared crossing. Buried track must not be selected through terrain in Surface view; the nearest visible path/track surface must win underground. Select a track in Underground, return to Surface, and verify its hidden outline does not show through terrain. Switch to Flat and confirm the same ID becomes exposed without moving. Check that the underground floor and absent terrain are never selected. Pause and confirm the timer stops; help itself does not automatically pause normal play. The inspector displays one-based coordinates, while `TilePos` uses zero-based coordinates.

FND-03 retains recorded 0.2.0 evidence and passed the relevant 0.3.0 control regressions described above. Record remaining panning/dragging and display results, Windows/GPU details, and limitations before marking FND-02 verified. Repeat affected checks when controls or layout change.

For the crowd proof, watch guests and arrival counts advance, then pause before deleting a path. A deletion must reconcile guest references even while paused. Resume to observe rerouting or a lost state; isolate a remaining path to exercise disconnection. Restore paths and confirm local recovery without resetting time or guests. Test removal of occupied, next, and destination nodes separately. Guests must not jump to distant paths or retain removed IDs. The 0.4.0 native session verified Remove path, R, and Restore; physical Delete needs a retest because the helper's two injections had no effect despite the passing headless key test.

For the train proof, use Find, observe all phases and a completed lap, then Close during motion and wait for station parking. Open again and confirm the full 90-tick dwell before dispatch. Repeat Close at the station and while globally paused: the command can change operation, but only resumed ticks can advance a moving train. Guests must keep moving while the train is closed. Verify O/F and both buttons, all rotations, sampled track picking, the top-right ride panel's map-input exclusion, and the minimum window. Record current regression evidence in the static visibility report.

For persistence, use a separate absolute save directory and record time, train phase/lap/open request, and guest counters. Save, change the park and advance, then Load: the prior state must return paused with reset view and no click-through or stale pending edit. Resume and compare continuation. Save twice and explicitly load Backup to recover the first state. Test missing/corrupt/unsupported saves without losing the current park, and confirm a fresh launch does not overwrite either file. Check all three shortcuts/buttons and help/input gating. File-level and headless tests supplement normal UI observations.

## 5. Capture a rendered diagnostic frame

The app supports:

```text
parkbound.exe --capture-frame <absolute-output.png>
  [--capture-scene overview|selection|path|track|crowd|lost|recovered|help|
                   train|train-lift|train-drop|train-brake|train-return|train-closed]
  [--capture-map flat|terraced]
  [--capture-view surface|underground]
  [--capture-rotation 0|1|2|3]
  [--capture-ticks 0..36000]
  [--capture-small-window]
```

It installs diagnostic systems only when `--capture-frame` is present. The preset advances the requested number of complete simulation ticks (default 600), then pauses and suppresses keyboard, map-pointer, and UI-button actions and hover tinting. `selection` chooses terrain; `path` and `track` choose fixture objects. `crowd` frames the network at 160% zoom. `lost` isolates the first guest's current node by removing its neighbors, then advances 60 ticks; `recovered` additionally restores paths and advances 600 ticks. These are controlled demonstrations, not synthetic clicks on the player controls.

Train scenes focus the circuit at 150% and select its current piece. `train` captures the requested tick state; named phases seek interior section behavior with a limit of 36000 additional complete ticks. Lift requires speed 24 and lead height 12..16 (excluding 16); drop requires speed at least 60 and height above 8 through 14; brake requires speed 20 and a lead car at least one third tile into the current piece. These speeds are integer distance units per tick. Return requires a stopped station wait after at least one lap; Closed requests Close and waits for stopped parking. A fault or target state not reached within the bound fails the capture. Guests also advance during these complete ticks. Keep `--capture-ticks` separate from the final total when recording evidence.

Specify map, visibility, rotation, and ticks explicitly when comparing views. A selected buried track can intentionally remain invisible in Surface view. `--capture-small-window` requests 1024 × 680. A successful capture waits at least 90 frames and two seconds, checks the PNG, and exits with code 0; timeout is 120 seconds. This requires a rendering environment and is not ordinary-input evidence.

Use a fresh output filename for each run. The implementation saves through a temporary file and renames it; an existing destination may cause failure on Windows. The following PowerShell example handles paths containing spaces and waits for the GUI process to exit:

```powershell
$repoDirectory = (Get-Location).Path
$captureDirectory = Join-Path $repoDirectory 'artifacts'
New-Item -ItemType Directory -Path $captureDirectory -Force
$capturePath = Join-Path $captureDirectory ("track-underground-{0}.png" -f (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
$gameExecutable = Join-Path $repoDirectory 'target\release\parkbound.exe'
$captureArguments = '--capture-frame "{0}" --capture-scene track --capture-map terraced --capture-view underground --capture-rotation 1' -f $capturePath
$captureProcess = Start-Process -FilePath $gameExecutable -ArgumentList $captureArguments -WindowStyle Hidden -Wait -PassThru
if ($captureProcess.ExitCode -ne 0) { throw "Capture failed: exit code $($captureProcess.ExitCode)" }
Get-Item -LiteralPath $capturePath
```

Repeat with train phases and the retained crowd/surface/help scenes; compare Flat/Terraced and Surface/Underground in all four rotations, including minimum-window captures. Inspect cars on the rails, slopes/curves, station fit, foreground occlusion, track picking, guest recovery, and legible controls/counters. Compare phase/lap/tick states to establish movement, not merely the existence of markers. A valid PNG and successful exit do not prove visual correctness. Do not report another application's screenshot as game evidence. `artifacts` is ignored by Git; describe exact current evidence in the static visibility report.

## 6. Package a portable Windows build

For a current source build, the simplest command is:

```powershell
pwsh -NoProfile -File scripts/package-windows.ps1
```

On a memory-constrained PC, build explicitly with two jobs first, then package that exact binary:

```powershell
cargo build --release --locked --package park_game --bin parkbound -j 2
if ($LASTEXITCODE -ne 0) { throw 'Release build failed' }
pwsh -NoProfile -File scripts/package-windows.ps1 -SkipBuild
if ($LASTEXITCODE -ne 0) { throw 'Packaging failed' }
```

Only use `-SkipBuild` immediately after a successful build of the current source. It does not prove an existing executable is up to date. The script assumes the normal host target output `release/parkbound.exe`; do not add a different `--target` layout or change its binary assumptions without updating and testing packaging.

The version 0.10.0 package layout is shown below; verification and the final guide-refresh procedure are recorded in the static visibility report:

```text
dist/
  Parkbound-0.10.0-windows-x64/
    Parkbound.exe
    measure-performance.ps1
    ...runtime, notices, documentation, manifests...
  Parkbound-0.10.0-windows-x64.zip
  Parkbound-0.10.0-windows-x64.zip.sha256
```

Packaging derives the version from Cargo metadata. It collects the locked Windows dependency notices, the embedded font license, Rust standard-library notices, and required MSVC redistributable DLLs. It writes `DEPENDENCIES.json`, `NATIVE_DEPENDENCIES.json`, `THIRD_PARTY_NOTICES.txt`, and `SHA256SUMS.txt`. Existing extracted package directories are preserved with a `.previous-...` suffix; the current ZIP is replaced. Check `dist` occasionally for accumulated previous packages.

The native runtime helper uses Visual Studio 2022's `dumpbin` to inspect imported DLLs recursively and copies the required runtime files from its redistributable directory. The foundation package needed `VCRUNTIME140.dll`. Other Windows system/UCRT libraries come from the operating system. Do not copy arbitrary DLLs from a machine's system folder or assume import inspection proves clean-machine compatibility.

If packaging reports missing Rust standard-library notices, install `rust-docs` for the pinned toolchain. If it reports a missing Visual Studio redistributable directory or `dumpbin`, repair the required C++ installation. Supplementary license source/hash failures require review; do not bypass them.

Check the ZIP hash against its adjacent `.sha256` file:

```powershell
Get-FileHash -LiteralPath 'dist\Parkbound-0.10.0-windows-x64.zip' -Algorithm SHA256
Get-Content -LiteralPath 'dist\Parkbound-0.10.0-windows-x64.zip.sha256'
```

Extract **the entire ZIP**, then run `Parkbound.exe` from the extracted package. Test with an unrelated working directory, standard-user permissions, and a path containing spaces and non-ASCII characters. Run the same package on a clean Windows PC without Rust or Visual Studio and without networking before claiming portable release compatibility. Keep its required DLLs and notices beside the executable.

The script does not sign or publish anything. `dist` is ignored by Git. A source push is distinct from publishing a downloadable game release.

## 7. Repository map and implementation details

| File or directory | Responsibility |
| --- | --- |
| `Cargo.toml` | Two-crate workspace, shared version/edition/MSRV, and build profiles. |
| `Cargo.lock` | Exact dependency graph; committed and retained for reproducible dependency resolution. |
| `rust-toolchain.toml` | Tested Rust toolchain and component pin for new PCs. |
| `.gitattributes` | Normalizes source text to LF while preserving exact bytes under `scripts/notices/`. |
| `crates/park_sim/src/lib.rs` | Authoritative terrain/map storage, validated heights, Flat/Terraced profiles, clock, speed enum, and simulation tests. |
| `crates/park_sim/src/surfaces.rs` | Stable terrain/object identity, placement/removal, shared clearance, deterministic overlap fixture, and invariant tests. |
| `crates/park_sim/src/crowds.rs` | Same-height path connectivity, deterministic BFS and SplitMix64 state, guest identities/progress, lost-state recovery, crowd fixture/restoration, and simulation regressions. |
| `crates/park_sim/src/trains.rs` | Track ports/spec validation, integer sampled circuit/arc distances, stable envelopes, train motion/operation/dwell/laps, fault/repair, poses, and simulation regressions. |
| `crates/park_sim/src/saves.rs` | Strict versioned JSON DTOs, bounds/reference/state validation, authoritative continuation encoding, and derived circuit reconstruction. |
| `crates/park_sim/src/diagnostics.rs` | Opt-in deterministic workload fixtures and a read-only adapter around the production path graph/BFS. |
| `crates/park_game/src/benchmark.rs` | Bounded CPU simulation, graph, route, static/dynamic mesh, codec, and filesystem measurements with actual-work JSON. |
| `crates/park_game/src/measurement.rs` | Shared timing summaries, percentile definitions, and new-output validation. |
| `crates/park_game/src/frame_probe.rs` | Main-loop profiles, CPU hooks, actual fixture/camera/window metadata, mesh work counts, and optional GPU receipt collection. |
| `crates/park_game/src/render_scene.rs` | Strict isolated fixture CLI, camera presets, edit/persistence isolation, and actual scene metadata. |
| `crates/park_game/src/gpu_probe.rs` | Bounded asynchronous Bevy GPU/CPU render-span collection, enabled capability status, raw receipt samples, and attribution limits. |
| `crates/park_game/src/save_store.rs` | LocalAppData path selection, bounded reads, staged/synced replacement, previous-valid backup, writer lock, and recovery errors. |
| `crates/park_game/src/persistence.rs` | Save/Load/Backup requests, 32 × 32 app guard, atomic live-state replacement, paused view reset, and status. |
| `crates/park_game/src/save_check.rs` | Opt-in fresh-process continuation/backup diagnostic using production save commands and a new explicit directory. |
| `crates/park_game/src/main.rs` | Bevy app setup, resources, fixed update, camera layout, keyboard/pointer input, scene refresh, and input-state tests. |
| `crates/park_game/src/static_landscape.rs` | Retained static entity/mesh slots, guarded full-window eligibility, resource replacement, and final geometry-selection counters. |
| `crates/park_game/src/world/static_chunks.rs` | Atomic emission-group partitioning, per-layer spatial/overflow buckets, exact vertex bounds, and full-scene equivalence tests. |
| `crates/park_game/src/world/guest_render.rs` | Persistent guest templates, projected roof/object lookup, stack shadow clipping, reusable CPU mesh vectors, work counters, and equivalence tests. |
| `crates/park_game/src/world.rs` | Terrain/object projection, connected paths, sampled circuit faces, separate guest/train meshes, shared picking/selection, visibility, circuit focus, and geometry tests. |
| `crates/park_game/src/ui.rs` | UI layout, actions, labels, help modal, button state, and refresh. |
| `crates/park_game/src/capture.rs` | Opt-in scene/map/visibility/rotation/window capture presets, screenshot receipt, PNG validation, and diagnostic exit. |
| `scripts/package-windows.ps1` | Release build/staging, metadata, package documentation, notices, archive, and hashes. |
| `scripts/measure-performance.ps1` | Sequential bounded CPU/frame runs, allowlisted hardware identity, binary/source metadata, sampled process memory, and JSON summaries. Also included in the ZIP. |
| `scripts/write-third-party-notices.ps1` | Locked dependency traversal and license text collection. |
| `scripts/copy-windows-runtime.ps1` | Native import inspection, MSVC runtime copying, and native manifest. |
| `scripts/notices/` | Reviewed supplementary license texts, original license documents, and exact source/hash records. Keep these in Git. |
| `.github/workflows/windows.yml` | Windows checks, packaging, and temporary review artifact upload. |
| `docs/` | Product decisions, acceptance checklist, build evidence, and asset provenance. |
| `benchmarks/results/` | Reviewed local diagnostic JSON measurements and manifests, separate from packaged guide files; no reference PC is selected. |

### Simulation boundary

`park_sim` has no Bevy dependency. `Park` owns dimensions, a row-major `Vec<Tile>`, a `u64` tick count, `Vec<SurfaceObject>`, guests/seed/RNG, optional circuit/train state, and monotonically increasing object/guest-ID allocators. Dimensions are validated in `1..=256`; invalid map construction panics. Tile access is bounds checked. A `Tile` has a `Terrain` enum and `height: u8` in abstract steps; `MAX_TERRAIN_HEIGHT` is 32.

`Park::new` creates flat terrain without objects, guests, or a train. Normal startup applies Terraced, installs the crowd/overlap fixture, then installs the train fixture. `try_apply_terrain_profile` validates every proposed height before mutation; it preserves authoritative operation state, materials, time, IDs, and object heights. `apply_terrain_profile` remains a convenience wrapper that panics on a rejected profile without changing state; use the fallible API for player actions. `set_terrain_height` validates bounds, height, and object clearance atomically.

`SurfaceId` distinguishes `Terrain(TilePos)` from `Object(ObjectId)`. A `SurfaceObject` contains ID, tile, `Path`/`Track` kind, X/Y axis, height, and clearance. `place_object(pos, kind, axis, height, clearance)` returns a stable ID or `PlacementError`; `remove_object(id)` returns the removed object or `None`, and never recycles an ID. Occupied intervals are half-open, with positive clearance and total height at most 32. Shared-tile overlaps are rejected. Paths stay on/above terrain; track can also be wholly buried. Terrain cannot bisect an object.

Read-only APIs are `objects()`, `object(id)`, `surface_position(id)`, `surface_height(id)`, `is_underground(id)`, and `max_surface_height()`. Invalid/removed references resolve to `None`, or `false` for the burial query. Maximum surface height excludes the empty clearance above it. `install_overlap_fixture()` is atomic, requires a 32 × 32 map, and preserves already matching objects on repeat installation.

`crowds.rs` adds `GuestId`, `GuestState::{Walking, Waiting, Lost}`, `Guest`, `GuestPosition`, and `CrowdStats`. Read-only APIs are `guests()`, `crowd_stats()`, `guest_position(id)`, and `path_neighbors(id)`. A guest stores current/next/destination path IDs, a retained tile/height anchor, integer progress, state, arrival count, and private route/wait data. Missing guest IDs return no presentation position.

Paths connect only through cardinal adjacency at equal height; diagonal neighbors, vertically stacked surfaces, and track objects do not connect. Axis affects decoration, not path connectivity. Deterministic BFS uses stable object-ID order. Route choice uses stored SplitMix64 integer state. Walking takes 30 complete ticks per edge and destination dwell takes 30 ticks. Initial guests have small deterministic start delays. Version 0.8.0 retains a derived immutable Arc path graph between unchanged crowd ticks. Successful path placement/removal invalidates it; the next crowd tick builds from a spatial (x,y,height) index with four cardinal lookups and sorted neighbor IDs. Cold path_neighbors queries derive an immediate answer without mutating cache counters. Clones share only the immutable graph, and loading starts cold. Cache warmth and diagnostic counters do not affect saves or authoritative Park equality; keep explicit equality aligned with all future authoritative fields.

`install_crowd_fixture(seed)` atomically installs the 37-node network, retained overlap track, and twelve guests. Repeating it preserves existing guest identities, time, and RNG state, even if a different seed is supplied. `restore_crowd_paths()` restores missing demonstration paths only; it preserves current guests and existing objects and rejects a collision atomically. New paths get new IDs.

`remove_object(id)` now reconciles guest references immediately. If removal affects a guest's current/next/destination/route, that route is canceled, movement returns to its retained starting anchor, and state becomes Lost. A vanished current path clears the current ID. On a later tick, a guest may replan in its component; recovery without a current path searches only the retained tile and cardinal same-height neighbors, ordered by distance then stable ID. It never chooses an arbitrary nearest path elsewhere in the park. Isolated nodes remain Lost until reconnected. These rules also apply when removal happens while paused.

`trains.rs` adds `TrackDirection`, `TrackPort`, `TrackPieceSpec`, `TrackKind`, `TrackPoint`, `TrackPiece`, `TrainCircuit`, `TrainPhase`, `Train`, `TrainPose`, and `TrainError`. `circuit()`/`train()` return optional read-only state; `train_cars()` supplies three center/front/rear poses. `install_train_fixture()` is atomic and a no-op when healthy. `install_train_circuit(&[TrackPieceSpec])` supports explicit headless construction/validation, not a player editor. Exactly one station, valid tile-edge connections including last→first, legal grades, positive lengths, and shared clearance are required. Curves are level quarter turns; lifts rise one step per tile; drops descend one or two; other pieces are level straights.

Each piece owns a stable Track object envelope and sampled centerline; the circuit stores ordered pieces/lengths, specifications, and station stop. `TRACK_UNITS = 1024` applies to each tile and height step, with vertical arc distance weighted to one quarter tile. Train state includes open request, phase, integer speed/distance, laps, dwell, piece index, and private lap-progress/departed fields. `TrainPhase` is Station/Lift/Drop/Brake/Cruising/Closed/Fault. The save codec preserves all authoritative fields; poses and render meshes remain derived.

The train initially opens with a 90-tick dwell. `set_train_open(false)` parks a dwelling train immediately or requests completion of a moving train's current lap. Reopening a parked train begins a full dwell. Global pause permits command changes but advances no distance/dwell. Rail envelopes reserve minimum height through maximum height plus two; camera fit includes actual sampled maxima. Decorative support posts do not reserve collision columns. Removing any circuit track faults/stops the train and clears circuit references/geometry. Reinstall repairs missing objects with fresh IDs, preserves completed laps and unrelated park state, and parks the train **closed**; it does not restart automatically.

The app schedules `FixedUpdate` at 30 Hz and calls `Park::advance(speed)`. This executes zero, one, two, or four **complete authoritative steps**, in order: clock → crowd → train. Reaching `u64::MAX` stops all simulation. New systems must join that complete step explicitly; never multiply only the clock for accelerated speed.

There is no general gameplay command queue, sloped/queue path connectivity, admission, guest needs/spending, collision avoidance, guest ride routing/boarding, multi-train operations, or money yet. Serialization covers the current authoritative Park, with future systems added to the versioned schema as implemented. Placement and fixture repair are not complete ownership/cost construction commands.

The intended extension remains: validated atomic commands → fixed-step simulation → state/events → presentation. Store future money in integer minor units; use stable simulation/content IDs; preserve deterministic iteration and seeded randomness. Do not store Bevy entities or rendering caches as the long-term save representation.

### Presentation boundary

`Simulation(Park)` and `ViewState` are Bevy resources. `ViewState` owns presentation state and current speed: rotation, zoom, pan, grid, Surface/Underground visibility, `SurfaceId` hover/selection, help state, drag state, and flags. `park_sim` remains authoritative for park data. A separate `park_content` crate is planned only when authored content definitions justify it; it does not exist yet.

The Update systems are chained deliberately:

```text
UI button actions → keyboard input → apply pending terrain preset → apply path actions
→ apply train actions → apply save/load action → pointer input → camera update → mesh update → static chunk visibility → UI refresh
```

Path requests apply after terrain presets, followed by train-operation/focus and save/load actions, before picking and scene refresh. A successful load replaces the candidate and resets pending view requests, selection, and caches before those later systems run. Button actions clear and then set `ui_pointer_consumed` for that frame; loading also consumes the pointer so its button cannot select the new park underneath. Pending edits apply before picking, so a frame does not inspect stale surfaces. Preserve this ordering and its regression tests.

`world.rs` uses 64 × 32 world-unit diamonds. `HEIGHT_STEP = 16` converts one simulation height step to presentation units. Geometry, depth, visibility, picking, and selection must agree across terrain and object layers. A cliff selects its terrain column; a path or track surface selects its own object ID. Ordinary picking excludes buried objects. Underground mode removes terrain from rendering/picking; its reference floor is non-selectable. Selection must obey scene occlusion rather than being forced above it. The legacy terrain-only picking wrapper remains for baseline tests. Version 0.9 adds controlled square 256-map previews with conservative scene bounds and focused cameras. Rectangular maps remain unsupported by presentation; ordinary editable play/load stays 32 × 32.

The camera and pointer transform use logical window pixels consistently. Mouse dragging uses successive logical cursor positions; replacing that with raw mouse-motion counts can reintroduce DPI-dependent behavior. UI exclusion bounds and camera offsets depend on `SIDEBAR`, `HEADER`, `FOOTER`, and the ride-panel dimensions/margin; change the layout and these boundaries together. The 236 × 152 ride panel is at the top right and blocks map input.

Terrain, path, and track surfaces use static procedural geometry rebuilt when dirty. Version 0.10 keeps one legacy mesh on maps at most 32 × 32 and partitions larger scenes into complete-triangle chunks under StaticLandscape. Camera changes select existing chunks without rebuilding geometry; static changes reuse slots and remove excess resources. Preset, visibility, removal, and restoration changes must invalidate scene and selection geometry even if selected identity stays unchanged. Circuit rendering/picking/selection share actual sampled faces, never the roofs of clearance envelopes. `world::GuestRenderer` now updates the separate guest mesh in place, while `world::train_mesh` retains the separate train mesh. Motion alone does not rebuild terrain. The old `world::guest_mesh` remains an independent geometry oracle. GuestRenderer prepares 192 body templates, caches full-object/dimension/rotation roof snapshots and an ID lookup, clips shadows in stack buffers, and reuses five CPU mesh vectors. This is not GPU instancing; actual GPU upload/presentation costs remain separate. Cars use authoritative center/front/rear poses; guests use endpoint/progress data. Both use shared depth and are not independent pick targets. Decorative supports are omitted where they intersect another object's occupied vertical range. `NoFrustumCulling` prevents stale bounds on changing meshes; retain it or update bounds correctly. Large-fixture CPU mesh costs were measured in 0.7/0.8. Historical controlled rendering and instrumented GPU-span evidence belongs to the 0.9 report; current chunk visibility evidence belongs to the static visibility report; physical uploads and display presentation remain unmeasured.

Zoom is bounded to 0.6–3.5; pan is bounded; Fit resets zoom/pan while retaining selection and accounts for terrain/object height. Keep the same camera layout calculation in rendering and pointer conversion. Rendering uses opaque white materials with vertex colors. Release builds use the Windows GUI subsystem, so opening the executable does not supply a normal console log window. Use capture exit status and the relevant debugger/log strategy when diagnosing a silent launch failure.

## 8. Resume work in this sequence

### A. Reproduce and close the foundation gaps

Build from the existing or newly cloned repository using the pinned toolchain; record the commit, Windows version, graphics driver, and check results. Complete ordinary input checks and clean-machine package checks where the new PC makes that possible. Update individual IDs only after their acceptance criteria pass. Keep the scope/catalogue audit visible rather than treating it as finished because coding has started.

### B. Preserve the accepted PRF-01 overlap baseline

The 0.3.0 overlap fixture, stable identity, visibility, and clearance proof is accepted. Retain it as a regression scene while improving performance. Its historical evidence is recorded in the overlap report; record current regression results in the static visibility report.

1. Run simulation, geometry, input, formatting, lint, and release checks on the exact current source; inspect hosted CI for that commit.
2. Demonstrate the shared crossing in all four rotations and both visibility modes, including Flat/Terraced comparison, zoom, pan, UI exclusion, and boundaries.
3. Verify hidden track and hidden selection do not show through terrain; check the underground reference floor cannot be selected.
4. Verify IDs and object heights survive view/preset changes; rejected clearance and removed references remain safe.
5. Record normal mouse/keyboard behavior and outstanding physical display or hardware limitations separately from synthetic tests; FND-02 remains open.
6. Package and inspect each new checkpoint, recording hashes and relocation results. Prior-build evidence does not automatically verify changed code.

Shared clearance and surface rules should become the foundation for paths, track, scenery, and construction. Reject illegal placement atomically once commands are introduced.

### C. Preserve the accepted PRF-02 crowd baseline

Version 0.4.0 verifies this small proof through 72 passing tests, eleven inspected captures, native removal/restoration/recovery controls, package relocation, and hosted CI. Read the crowd report and preserve same-height/cardinal connectivity, stable IDs, complete ticks, and local lost recovery as invariants. Do not start a second crowd implementation. Retest the physical Delete shortcut; the crowd checkpoint made no display-rate or large-park performance claim.

Preserve the train and save baselines below while measuring PRF-05. Queue ownership/access rules, slopes, actual guest entrance/exit, attraction routing, needs, and collision avoidance remain later work; the current graph treats all existing Path objects as public walkable nodes.

### D. Preserve the accepted PRF-03 automatic train baseline

Version 0.5.0 verifies the representative station → lift → drop → brake → station circuit's automatic movement and fixed-update behavior. Preserve sampled track geometry, stable identities, integer distance/speed, exact station/lap accounting, closure/reopening semantics, crowd progress, and shared rendering/picking. Repeat affected regressions using the recorded test method; distinguish simulated schedules from measured hardware performance. This is a technical circuit, not a coaster editor or complete ride family.

Preserve PRF-04 below, then proceed to PRF-05. Retest O/F and Open/Close/Find through native input when the game can be observed continuously; the interrupted 0.5.0 native session verified Space pause and an already completed lap only. Keep physical Delete, pan/drag, physical display, and clean-machine checks visible as separate work.

### E. Preserve accepted save/reload continuation: PRF-04

Version 0.6.0 verifies technical continuation through tests, fresh processes, inspected rendering, and package relocation; see [SAVE_REPORT.md](docs/SAVE_REPORT.md). `Park::to_save_bytes()`/`from_save_bytes()` use strict `parkbound-save` JSON with schema version 1 and content version 1. Preserve terrain/materials, vector order, object IDs/envelopes/allocators, ticks, seed/RNG, guest routes/waits/motion/arrivals, ordered circuit specifications/object IDs, and all train motion/open/dwell/lap fields. Reconstruct samples/lengths/station cache without fixture installation or newly allocated IDs. Lost guests without a current path and faulted trains without a circuit are legitimate states.

The 8 MiB format bounds collection sizes before graph validation: 4,096 objects, 1,024 paths, 256 guests, 2,048 circuit pieces, and 65,536 remaining route IDs total. These limits are for the proof, not the proposed benchmark. Validate references, adjacency/heights, IDs/allocators, clearance, circuit closure/grades, and train state before replacing the live park. Unknown fields/versions are rejected; migration support remains future work.

The game layer owns filesystem storage and controls. `SaveStore::user_directory()` chooses `%LOCALAPPDATA%\Parkbound\saves`; `--save-dir` requires an absolute override. The primary is `park.park.json`, backup `park.park.json.bak`, writer lock `park.park.json.lock`. A valid former primary becomes the backup; a corrupt primary cannot poison it. Same-directory staged/synced files replace each name, with backup rollback on final-primary failure. These independent renames are not a single transaction or power-loss guarantee. A rollback error retains a recovery file and must be investigated, not erased by sweeping temporary files.

Startup neither loads nor saves. F5/Save writes one slot; F9/Load and Shift+F9/Backup explicitly choose a file. Loading into a candidate preserves the live park on errors, rejects non-32 × 32 presentation, then resets selection/camera/view and dirty caches and starts paused at a 1x resume speed. Train Open/Close intent survives. Custom valid terrain heights survive and display Custom. UI/camera/pause preferences are not saved.

Preserve whole-Park continuation, corruption, version/bounds, disk failure/recovery, and render/cache checks when changing code. Include moving train, station dwell, pending Close, lost guests, and path edits. The opt-in `--save-check write` then separate `--save-check verify` process diagnostic requires a fresh absolute `--save-dir` whose parent exists; write creates that directory and refuses an existing one. Verify checks 3,600 continued ticks/commands plus explicit backup and creates a new `verification.txt` receipt. It uses production save actions without a renderer; Windows CI is configured to run both after packaging. Render scenes `saved|loaded|backup|load-error` also require an explicit isolated `--save-dir`; the saved scene writes that slot and the error scene asserts no live-Park mutation. Use only QA directories, never player slots. Complete native Save/Load/Backup retests separately while moving to PRF-05.

### F. Optimize with local diagnostics: PRF-05

The proposed benchmark is 256 × 256 tiles, 5,000 guests, and 50 operating rides, with 60 FPS at 1080p to be evaluated only after agreeing a representative PC. No reference PC is selected; the user explicitly rejected this poorly performing development PC as a reference. **These are design targets, not achieved capabilities.** Current diagnostics contain exactly one circuit/train; normal player rendering stays 32 × 32 and the 5,000-guest fixture cannot be saved under the unchanged 256-guest cap. Keep PRF-05 in progress for the full target while measuring the supported envelope.

`park_sim::diagnostics::build_fixture` constructs `baseline` (normal 32 × 32/12 guests/37 paths), `save-scale` (256 × 256/256 guests/1,024 paths), and `stress` (256 × 256/5,000 guests/1,024 paths). The larger path grid spans x/y 80..111 and has 1,984 undirected edges. Every fixture has one 46-piece circuit/train and deterministic seed/state. `DiagnosticPathGraph` wraps the actual production graph and BFS without changing simulation state. Initial guest waits span 0..30; completed 60-tick warmup excludes the first routing burst from measured simulation ticks. Incomplete warmup remains explicit.

Build Release, then run `pwsh -NoProfile -File scripts/measure-performance.ps1 -OutputDirectory <absolute-new-directory>`. Its parent must exist. Default CPU runs are three repetitions per case, 60 warmup ticks and 300 measured ticks; runtime soft phase caps retain incomplete outcomes. The wrapper's CPU timeout is 180 seconds per process. `-Cases`, `-Repetitions` 1..5, `-BenchmarkTicks` 30..1800, and `-SkipFrames` support focused batches. It does not build or mutate player saves.

The two baseline frame runs request physical 1920 × 1080 with `AutoVsync` then `AutoNoVsync`, default 600 measured samples after at least 120 warmup frames and two seconds. `-ProfileSamples` accepts 120..3600. Keep the window visible/unobscured and inspect actual resolution, scale, present mode, adapter, focus/visibility/occlusion, and any timeout. Frame intervals are main-loop pacing, not GPU execution or presented-frame timing. Internal frame timeout is 120 seconds and wrapper timeout 150.

Each sequential child produces a new JSON file. `summary.json` contains relative run names, executable hash/size/runtime version, source checkout identity, whitelisted OS/CPU/GPU-driver/RAM, elapsed/exit status, and 200 ms sampled working/private bytes. It omits usernames, computer names, serials, and absolute paths. Sampled maxima can miss short peaks; the OS peak-working-set counter is separate. `-Executable` can target an extracted package, but checkout identity is not binary-build attestation. Record fresh measurements per PC.

The ZIP includes `measure-performance.ps1` beside `Parkbound.exe`. PowerShell 7 is required only for this optional wrapper. From that folder run `.\measure-performance.ps1 -Executable .\Parkbound.exe -OutputDirectory <absolute-new-directory>`. The packaged wrapper defaults to its sibling executable and leaves source metadata null when no source manifest exists.

Read [OPTIMIZATION_REPORT.md](docs/OPTIMIZATION_REPORT.md) for retained 0.8.0 contracts/results and the corrected [PERFORMANCE_REPORT.md](docs/PERFORMANCE_REPORT.md) for historical local observations. Validate completed work/unsupported outcomes and compare like-for-like workloads. Graph rebuilds, routing bursts, mesh generation, serialization, and one-train/large-map limitations are distinct questions. Preserve complete ticks, deterministic IDs/RNG, removal/recovery, sampled track rendering/picking, and save continuation in any optimization. Do not start a large content catalogue before architecture/target decisions are recorded.

The retained historical local diagnostic data is in `benchmarks/results/2026-10-20-reference/` with its manifest and extended subdirectory. Fourteen production processes completed: three 300-tick CPU repetitions for each case, two 600-sample baseline frame runs, and three 1,800-tick stress repetitions. The current PC is an i5-9400T with Intel UHD 630 driver 31.0.101.2115 and approximately 7.87 GiB physical RAM. These are measurements of that desktop session, with power/temperature/background load unrecorded and frame occlusion unknown. They are not hardware requirements.

Version 0.8.0 completes reusable guest geometry and cached path connectivity with explicit invalidation. Its six raw JSON files and manifest are retained at `benchmarks/results/2026-10-20-optimization-local/`. One 300-tick run per case and one 1,800-tick stress run all completed, using clean source checkpoint `3f1ea88` and the unchanged runtime binary. Each animated pair completed sixty samples with equal final state/vertex totals; every measured tick reused the graph, and templates/decks/buffer capacities stayed unchanged through retained updates. The optimization report contains the timing table and noise limits. These are single local observations, not representative-hardware speedup claims.

Retain the verified optimization and its regression oracles. Version 0.9 below verifies controlled large-map/GPU diagnostics, and 0.10 completes the bounded static visibility change. The next recommended gameplay slice is interactive path placement/demolition in the existing park. The earlier 0.7.0 frame result cannot identify a GPU cause, and 0.8 made no frame-performance comparison. Agree a representative PC separately before hardware acceptance; full ride-count/player-map/save-cap scaling remains open.

### G. Retain the verified 0.9.0 controlled rendering/GPU baseline

Read [LARGE_MAP_GPU_REPORT.md](docs/LARGE_MAP_GPU_REPORT.md) before running diagnostic commands. `--render-case baseline|save-scale|stress` creates a separate preview; `--render-camera overview|crowd|ride`, `--render-rotation 0..3`, and `--render-underground` require that mode. It permits camera/inspection/pause/speed/grid/view/Find but blocks terrain, path edits/restoration, train operation, and saves. Normal startup, schema/content versions, and 32 × 32 load restrictions remain unchanged.

`--profile-gpu` requires `--profile-frames <new-absolute.json>`. Existing profiles remain baseline/no-GPU without opt-ins. Optional shared render flags select a controlled 256 × 256 case. The wrapper supports `-SkipCpu -RenderCase stress -RenderCamera crowd -GpuDiagnostics -ProfileSamples 120`, plus an absolute new output directory, for two bounded frame modes without repeating CPU fixtures. It rejects no-run combinations and frame settings with SkipFrames. Historical reports remain accepted only without incompatible opt-ins.

The receipt-window GPU API exposes actual per-span timestamp durations, render-encoding CPU durations, and optional pipeline counters. Enabled device features govern availability; missing support is explicit, never estimated from CPU time. Bevy delivers the latest diagnostics asynchronously: receipt frame/time are not source GPU frame identity; late warmup/final missing batches are possible. Never sum nested spans or call these presentation time. Logical mesh payload bytes/vertices/indices are generated CPU work, not physical GPU upload/allocation or visibility counts.

Verify overview and focused visible crowd/ride rendering, all rotations and views, bounds/picking/depth, unchanged ordinary play/saves, and isolated controls. Retain partial/unavailable results. All 178 release tests, strict source checks, six GPU profiles, inspected captures, bidirectional 0.8/0.9 continuation, and implementation CI passed. Relocated package checks passed; the final refresh verified all fifteen current guides and all 24 payload hashes in the report. Raw results are at `benchmarks/results/2026-10-20-large-map-gpu-local/`: six profiles plus three summaries and a manifest. All windows were visible/focused at actual 1920 × 1080; occlusion was unknown. That historical build submitted all 2,251,764 landscape vertices in overview and focused views. Version 0.10 below now selects only intersecting chunks while retaining the full-scene oracle and camera/edge/depth/picking regressions. This is a reduction in unnecessary submitted work, not a bottleneck diagnosis or an FPS promise. No reference PC is selected and PRF-05 stays open. Preserve all historical diagnostic datasets byte-for-byte. Do not run long performance sweeps on this excluded host to make hardware decisions.

### H. Retain the completed 0.10.0 static visibility implementation

Read [STATIC_VISIBILITY_REPORT.md](docs/STATIC_VISIBILITY_REPORT.md) for the current contract and verification. StaticLandscape retains the original single mesh for maps at most 32 × 32. Larger maps keep each terrain tile/details, scenery object, and surface object as an atomic emission group, including track supports/base/rails. Groups use 1024 × 512 world-unit buckets by actual group bounds center, with per-layer overflow for oversized groups. Layer keys order returned chunks Shadow → Terrain → Scenery → Object. Static assets precede dynamic meshes during initial allocation; neither rule guarantees global GPU draw order after slot shrink/regrowth changes mesh asset IDs. Original triangle order is preserved within each group; groups and triangles are not split between buckets. Each chunk uses its actual emitted-vertex bounds. The full `build_scene` mesh is a transient partition input and retained correctness oracle, not an additional persistent GPU mesh.

The camera test uses inclusive full-window world-XY bounds with a two-logical-pixel guard. It covers world rendering behind UI and retains intersecting triangles without clipping/splitting them. Pan, zoom, and resize only toggle chunk Visibility. Static content/grid/rotation/view changes still rebuild all static geometry; entity/mesh slots are reused and excess slots removed. Same-tick load invalidation and large→small→large replacements must not leave stale geometry. Dynamic guests/train remain unchanged. Startup avoids a duplicate geometry build; unchanged viewport bounds skip selection evaluation. If window/camera bounds are unavailable, all chunks remain selected.

Verify triangle/output equivalence, conservative bounds/edge cases, all rotations and Surface/Underground, picking/depth, no camera-only geometry rebuild or resource churn, and reduced camera-selected geometry in focused large views. `render_work.static_visibility` is a final snapshot with method `conservative_world_xy_aabb`, chunking `single_mesh|spatial_groups`, bucket size `[1024,512]`, selection state `camera_bounds|unavailable_all_selected`, guarded world viewport or null, and total/selected/hidden chunk counts. `generated` and `selected` each report vertices, indices, and logical CPU payload bytes; generated totals also remain at `render_work.meshes.landscape`. `geometry_builds`, `visibility_evaluations`, and individual chunk `visibility_changes` are cumulative since initialization, including startup/warmup. These are CPU eligibility counts, not measured GPU draw calls or visible pixels. Upload/presentation and whole-GPU-frame time remain unmeasured. Preserve the existing frame/GPU receipt semantics and source/EXE identity. Thirty new wrapper checks and 44 historical wrapper checks passed. The corrected group/layer implementation passed 193 release tests (128 game/65 simulation), formatting, strict release Clippy, release build, and CI. Eight matched image pairs, bidirectional 0.9/0.10 continuation, six GPU profiles, and package relocation passed. All profiles recorded one geometry build/evaluation; the focused stress camera selected 13/212 chunks and 149,556/2,251,764 generated static vertices. All generated meshes remain retained; partitioning temporarily holds the full source mesh. Initial construction/peak memory/full rebuild work are not reduced by this step. Overview selects every chunk and can add batching overhead; no overview FPS benefit is claimed.

This bounded step is complete. Preserve its regression oracles, schema, same-tick replacement rules, and raw evidence at `benchmarks/results/2026-10-20-static-visibility-local/`. PRF-05 remains in progress and this PC is not reference hardware. Preserve all historical datasets/reports/notices and the normal save/player limits.

### I. Recommended next slice: interactive paths

Build interactive level-path placement and demolition in the existing 32 × 32 park. Add a clear placement preview, explicit valid/rejected feedback, and deliberate placement/removal actions. Use the existing clearance checks, stable IDs, same-height cardinal connectivity, and path-cache invalidation. Verify joins/branches, guest rerouting, disconnection/lost recovery, blocked placement without mutation, and removed IDs. Include same-step save/load and deterministic continuation for newly built and removed paths, alongside headless input and safe native-control checks when possible.

This begins BLD-01/BLD-02/BLD-04; it does not finish their eventual costs/refunds, ride queues, or complete construction criteria. The current Remove/Restore fixture buttons are proof controls, not a path editor. Do not widen maps/save caps, fabricate fifty rides, add economy implicitly, or turn this slice into more FPS tuning on the excluded PC. Carry existing native/physical/clean-machine gaps separately.

### J. Continue the full milestones

After technical proofs, Stage 3 integrates the first complete park loop: construction, paths, a flat ride, a shop, guests, queues, pricing, money, demolition, inspection, and saves. Stages 4–7 expand coasters, operations, ride families, original content, scenarios, and editors. Stage 8 covers performance, stability, balance, accessibility, and polish; Stage 9 closes public distribution. Keep each feature's persistence, UI, acceptance demonstration, and relevant tests in the same milestone.

Do not start all of these systems in one change. Select a small observable behavior, implement it end to end, verify it, update the checklist/report, and retain a working build.

## 9. Work safely across two PCs

Use GitHub as the shared source history. This 0.10.0 session uses the original OneDrive folder recorded above. The separate second-PC clone remains `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`, with no verified OneDrive sync. For separate clones, push completed work to GitHub and fetch it on the receiving PC. For the shared folder, finish the editing/build/Git session on the first PC, let OneDrive finish syncing on both PCs, then check Git status and the expected commit on the receiving PC. Use only one active editing/build/Git session for that shared checkout. Rebuild generated output as needed rather than relying on another PC's `target` cache.

Before a work session:

```powershell
git status --short --branch
git fetch origin
git remote show origin
```

Check the default branch and outstanding pull requests. This checkpoint is on `codex/static-terrain-visibility`, based on completed diagnostic checkpoint `96518d9`. Inspect PR #8 and the large-map/optimization/measurement/save/train/crowd/overlap stack before deciding the current review base. Preserve that ancestry and review actual merge status before changing a PR base. Only switch/pull with a clean working tree or after deliberately preserving local changes.

To resume the published feature branch on a PC without a local copy:

```powershell
git switch --track origin/codex/static-terrain-visibility
```

If the local branch already exists:

```powershell
git switch codex/static-terrain-visibility
git pull --ff-only origin codex/static-terrain-visibility
```

After the full review stack merges, resume from the verified merge on `main` instead. Until then, retain the optimization branch ancestry and keep the review stacked deliberately. When a fast-forward fails, inspect histories and resolve intentionally; do not force-push or reset away the other PC's changes.

Before leaving a PC, run relevant checks, inspect the diff, explicitly stage the intended source/docs, commit, and push the branch:

```powershell
git diff --check
git diff --stat
git status --short
# Stage the specific files changed for this task, then inspect what will be saved.
git diff --cached --stat
git diff --cached
git commit -m "Reuse guest rendering and cache path connectivity"
git push -u origin codex/static-terrain-visibility
```

The staging step is intentionally task-specific; do not accidentally include credentials, unrelated or unrequested private machine paths, diagnostics from other apps, executables, caches, or unrelated files. Ensure `Cargo.lock`, relevant notices, and documentation changes are included when they belong to the change. Do not commit generated `target`, `dist`, or `artifacts` directories. Preserve `.gitattributes`: supplementary notice files have recorded SHA-256 hashes, so changing their line endings through Windows Git settings would break verification. They are deliberately excluded from text normalization.

Use a pull request to review substantial changes and preserve the playable default branch. Update the milestone report with the commit, commands/check results, exact next acceptance ID, known blockers, and unresolved decisions. Push work before moving PCs; an unpushed local commit or stash is not backed up on GitHub and may not yet have reached the other PC through OneDrive. On the receiving PC, verify the expected commit with `git log -1 --oneline` before starting.

Player save files are separate from this source workflow. LocalAppData is local to each Windows account; the shared source folder and GitHub do not transfer it. Close Parkbound on both PCs, preserve any receiving-PC save, and separately copy `park.park.json` and optionally `park.park.json.bak` to the receiving save directory. Do not copy the lock or an in-progress temp file. Load using the same supported build and verify the restored paused state before resuming. An explicit absolute `--save-dir` can select a user-chosen QA/portable directory; do not run two writers against a shared or syncing save directory.

If a crash leaves a lock, close every Parkbound instance before removing only `park.park.json.lock` in the selected save directory. Keep the primary, backup, and any reported recovery file. See the save report for the storage contract and its limits.

If status says **Recovery: see save folder**, preserve `.parkbound-rollback-*.tmp` there after closing all instances. It contains the exact former backup bytes from a failed rollback. Copy it separately and validate before restoring a slot; a preserved backup may itself be corrupt. Do not sweep temporary files or erase the existing primary/backup while diagnosing the failure.

## 10. CI, licensing, and release decisions

The Windows workflow runs on pushes, pull requests, and manual dispatch. It uses a Windows 2022 runner, Rust 1.93.1 with formatting/lint/docs components, formatting, release-profile Clippy with warnings denied, release-profile tests, packaging, and artifact upload. The review artifact retention is 14 days. Inspect the real run for the intended commit; configuration alone is not a passing CI result. The workflow has read-only repository contents permission and does not publish a GitHub Release.

The following hosted runs were verified successful on 20 October 2026:

| Commit | Hosted Windows run |
| --- | --- |
| `e4a8d2f` | [Successful run 39265273354](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39265273354) |
| `2ec6bf9` | [Successful run 39379555949](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39379555949) |
| `dc832fa` (0.2.0 starting baseline) | [Successful run 39381924927](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39381924927) |
| `0220c7b` (0.3.0 overlap runtime source) | [Successful run 39406042854](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39406042854): 51 tests, formatting, release Clippy, and Windows package |
| `20cbc76` (0.4.0 crowd runtime source) | [Successful run 39412913892](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39412913892): tests, formatting, release Clippy, Windows package, and artifact upload |
| `bd227f1` (0.5.0 train runtime source) | [Successful PR run 39417086959](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39417086959): source checks and Windows packaging for draft PR #3 |
| `d7a16f6` (0.6.0 save runtime source) | [Successful PR run 39422425034](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39422425034): formatting, lints, tests, packaging, fresh-process continuation, and artifact upload for draft PR #4 |
| `e93eb99` (0.7.0 measurement runtime source) | [Successful PR run 39425219126](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39425219126) for draft PR #5. The performance report distinguishes historical local diagnostic measurements from hosted checks. |
| `3f1ea88` (0.8.0 wrapper checkpoint, runtime `9460a9`) | [Successful PR run 39429282120](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39429282120) for draft PR #6, including build, package, continuation, and CPU-wrapper checks. The optimization report retains that historical evidence. |
| `5d49b7e` (0.9.0 render/GPU runtime source) | [Successful PR run 39435077589](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39435077589) for draft PR #7: formatting, lint, tests, build/package, production continuation, CPU-report smoke checks, and artifact upload. |
| `ff2857e` (0.10.0 static visibility runtime source) | [Successful PR run 39440918757](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39440918757) for draft PR #8. Runtime/source/package evidence is in the static visibility report. |

These results belong to their exact implementation commits. The earlier executables were also exercised locally through captures, native input, and relocation. Historical 0.5.0 results remain in the train report. Historical 0.6.0 tests/source checks, twelve inspected captures, fresh-process continuation, verified package relocation, and successful hosted CI remain in the save report. Historical 0.7.0 local source checks and measurements are recorded in the performance report; implementation CI run 39425219126 and package relocation passed, with 22 matching payload hashes, thirteen matching guides, fresh-process continuation, and the bundled wrapper exercised from C:\Windows. Documentation-only evidence updates do not replace runtime-source identity, and a later head has its own CI status; the recorded success does not claim its later run has passed. Archive hashes change when guides are repackaged; use the adjacent `.zip.sha256` and verify every payload hash and bundled guide against the final snapshot; do not reuse an earlier package count as current evidence.

There is no selected project source-code license yet. Free gameplay distribution and permission to reuse source are separate decisions. Saving source in this GitHub repository does not by itself select MIT, Apache, GPL, or another project license. Decide source licensing before making reuse promises or accepting outside contributions. Keep all third-party licenses and asset provenance intact.

No original RCT code, artwork, music, scenarios, or other original game files are bundled or required. Do not import those files or copy code from reference projects without reviewing applicable rights and obligations. The font has its own license; it is not project-authored art. Update provenance and dependency notices for every adopted external asset or dependency.

Public naming, final hardware/Windows requirements, signing, installer choice, source license, catalogue/expansion scope, and public release channel still need resolution. The proposed download path is ZIPs first, then installer/public distribution. Existing local builds are unsigned. Publishing a source checkpoint is not acceptance of the full game's release criteria.

## 11. Copy-paste prompt for the next coding assistant

```text
Continue https://github.com/strijbosmh-afk/RailRust-Tycoon, temporarily named Parkbound.
Read RESUME_DEVELOPMENT.md, docs/STATIC_VISIBILITY_REPORT.md, the historical LARGE_MAP_GPU/OPTIMIZATION/
PERFORMANCE/SAVE/TRAIN/CROWD/OVERLAP/TERRAIN/FOUNDATION reports, PRODUCT_DECISIONS,
GAMEPLAY_CHECKLIST, and DEVELOPMENT_PLAN. Inspect actual source/Git state first.

NO REFERENCE PC IS SELECTED. The user explicitly rejected this poorly performing
PC as a reference. All timings on it are local diagnostics only. Hardware requirements
and proposed 60 FPS at 1080p acceptance wait for an agreed representative PC.
Preserve historical raw JSON/hashes/path names in benchmarks/results; the old
2026-10-20-reference directory name is not hardware approval.

Completed bounded milestone is 0.10.0 static terrain chunk visibility, branch
codex/static-terrain-visibility from verified 0.9.0 checkpoint 96518d9. Check current
source/review/CI/package evidence in STATIC_VISIBILITY_REPORT before deciding what remains.
Current source ff2857e passed 193 tests, strict source checks, eight image comparisons,
bidirectional 0.9/0.10 continuation, six GPU profiles, CI 39440918757, and package relocation.
Final refresh verified all 25 payload hashes and sixteen current guides.
Historical 0.9.0 source 5d49b7e passed 178 tests,
profiles/captures/continuation, CI 39435077589, and final 24-payload/15-guide packaging.
Do not infer current verification from historical 159 tests or CI 39429282120.
The bounded 0.8.0 cache/rendering optimization is complete; do not reimplement it.
Inspect PR #8 and all stacked reviews/merge state before choosing the next base.
Preserve unrelated changes and remote history. Current source is the original shared
OneDrive path in this guide; the second-PC path is a separate clone. Use one active
editing/build/Git session in the shared folder.

Normal startup remains the 32x32 demonstration with explicit Save/Load/Backup.
Render previews use --render-case baseline|save-scale|stress, camera overview|crowd|ride,
rotation 0..3 and optional Underground. Camera/inspection/speed/grid/view/Find remain
available; terrain/path edits/restoration, train Open/Close, and persistence are disabled.
Capture/profile modes suppress ordinary input too. Larger square rendering is a
controlled diagnostic, not general large-map player saves or a fifty-ride simulation.

--profile-gpu requires --profile-frames and adds optional real GPU timestamp spans.
Preserve default baseline/no-GPU profiles and exact fixture/camera/window metadata.
Keep main-loop pacing, main-world CPU hooks, render-pass CPU encoding, GPU timestamps,
pipeline counters, and generated CPU mesh bytes distinct. GPU readback is asynchronous
and latest-only; receipt time/frame cannot identify source frame, dropped batches,
or exact warmup membership. Never sum nested spans or invent unsupported GPU values.
Physical upload bytes/time and display presentation remain unmeasured. Reports are
bounded, new-path-only, retain partial/unavailable outcomes, and cannot touch player saves.
Wrapper supports frame-only SkipCpu with explicit render options/GpuDiagnostics;
SkipCpu+SkipFrames and frame options with SkipFrames fail before output/process creation.

Preserve park_sim complete 30 Hz integer ticks, IDs/RNG/BFS ordering, authoritative save
bytes/equality, and derived Arc graph caching with exact successful-edit invalidation.
Preserve GuestRenderer templates, cached roofs, five reused vectors, same-ID load
invalidation, shadows/depth/occlusion, and the legacy geometry oracle. Normal 32-map output
and save behavior need regressions alongside all-rotation 256-map bounds/picking/render proof.

Retain schema 1/content 1, strict 8 MiB/collection limits, backup/lock/recovery behavior,
and full authoritative continuation. Default saves are %LOCALAPPDATA%\Parkbound\saves,
outside app/OneDrive; copy saves separately between PCs. Startup never writes; successful
load pauses/resets presentation at 1x while preserving train intent. Production --save-check
uses a new absolute QA directory and checks 3,600 ticks/commands, backup, and paused load.

PRF-05 stays open: no representative PC, only one circuit/train, normal editable 32 × 32 map,
and unchanged 256-guest save cap. The static terrain visibility work is complete on
maps larger than 32: atomic terrain/scenery/surface groups containing complete triangles,
deterministic returned chunk order and initial static-before-dynamic asset allocation,
without a global GPU draw-order guarantee after slot regrowth; actual bounds and full-window viewport
with two-logical-pixel guard, camera-only Visibility updates, and reused entity/mesh slots.
Preserve the small-map legacy mesh, full-scene oracle, same-tick load invalidation,
depth/picking/shadows, and unchanged dynamic guests/train. Preserve verified geometry/output
equality and selected-geometry regressions; selected means CPU eligibility, not GPU draws/pixels.
render_work.static_visibility is a final snapshot; counters include startup/warmup.
Generated totals stay in meshes.landscape. Unknown viewport selects all conservatively.
All generated meshes remain resident, and static changes still rebuild the whole scene.
No lower startup/peak memory/rebuild cost or overview FPS benefit is claimed.
Next recommended gameplay slice: interactive level-path placement/demolition on 32×32,
with preview/rejection feedback, clearance/stable IDs/cache invalidation, guest routing,
and same-step save/load/continuation tests. This begins BLD-01/02/04 but does not complete
costs/refunds/queues/full construction. Existing Remove/Restore is not a path editor.
Choose work from gameplay needs and verified evidence,
not this PC's FPS. Native Save/Load/Backup, O/F/buttons, physical Delete, pan/drag/display,
and clean PC offline/standard-user checks remain open unless directly reverified.
No full construction, boarding/queues, money, scenarios, autosave/browser/migrations,
or audio exists. Preserve original assets and exact notice bytes.

Use Rust 1.93.1 MSVC / Bevy 0.17.3/Cargo.lock. Complete authorized work and relevant checks,
update all current guides/checklist, verify final package files, and push source/handoff
before changing PCs. Do not select a public name, license, hardware reference, clean PC
support, or public release implicitly.
```
