# Resume development on another Windows PC

This is the handoff guide for **RailRust-Tycoon**, the repository for an original Rust theme park simulation inspired by classic RollerCoaster Tycoon 1 and 2. The application currently displays **Parkbound**, a temporary working title; the repository name does not rename the application. Current milestone: path and crowd proof **0.4.0**, 20 October 2026, following foundation 0.1.0, terrain 0.2.0, and overlap 0.3.0. Read [the crowd report](docs/CROWD_REPORT.md) for actual verification; implementation and acceptance are tracked separately.

Canonical repository: <https://github.com/strijbosmh-afk/RailRust-Tycoon>.

The repository contains the source, locked dependencies, build scripts, notices, and development records needed to rebuild on another PC. Generated executables, ZIPs, screenshots, and Cargo build caches are deliberately ignored by Git. Continue from the synced workspace below or clone the repository, then rebuild as needed; copying a previous PC's `target` folder is unnecessary.

### Shared OneDrive working folder

The current 0.4.0 session is back in the original PC's shared OneDrive workspace. Its full local path, recorded on 20 October 2026, is:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
```

The path within that OneDrive root is `Documenten\ChatGPT\Railroad Tycoon`. On another PC, the Windows username or OneDrive root may differ; locate the synced folder and open it as the project workspace. The folder name remains `Railroad Tycoon`, while the GitHub repository is `RailRust-Tycoon` and the application's temporary title is `Parkbound`.

On the original PC, open the workspace in PowerShell with:

```powershell
Set-Location -LiteralPath 'C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon'
```

You can continue from that synced workspace or use a separate GitHub clone. The 0.3.0 checkout on the second PC was `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`; it is a separate local path, not evidence that OneDrive is syncing it. Both paths are retained intentionally. Verify Git state and the expected commit on whichever checkout you use. For a shared workspace, let OneDrive finish syncing and follow the two-PC workflow in section 9. The clone commands below are unnecessary when opening an existing checkout.

The current feature branch is [`codex/path-crowds`](https://github.com/strijbosmh-afk/RailRust-Tycoon/tree/codex/path-crowds), based on `codex/overlapping-surfaces` at `2508398eaa4e731018ae49213f8b57fd291c4a14`. The overlap work was found on GitHub while `main` still held 0.2.0. Fetch the current feature branch when resuming; do not assume cloning `main` retrieves the newest checkpoint. The overlap pull request remains separate until merged. Check actual branch/PR state before choosing a base for further work.

The crowd implementation is commit `20cbc76936d698bdb1f7a6dc2e22789d987b3d93`, followed by documentation-only handoff updates. [Draft PR #2](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/2) targets the overlap branch and depends on [PR #1](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/1); neither was merged in this session. [Windows CI for the crowd implementation](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39412913892) passed tests, formatting, linting, packaging, and artifact upload. The local ZIP passed relocation and all 18 payload hashes; see the crowd report for the executable identity and remaining checks.

## 1. Read this first

The user confirmed:

- Rust implementation, classic RCT1/RCT2 isometric construction and park management.
- Windows only, with free distribution and all functionality included.
- Development in playable stages, eventually covering the agreed complete classic gameplay inventory.

Current defaults are offline single-player, an independent simulation library, original assets, four isometric orientations, and versioned local saves when persistence is implemented. Windows 11 x64 is the first build/test target, not a proven minimum requirement. Multiplayer, mobile/console ports, and scripting are outside the current plan. Expansion content and original-file import compatibility remain undecided.

**The first executable is a foundation, not a finished management game.** Do not infer completion from the visible park or the existence of a package.

Read the project records in this order:

1. [Crowd report](docs/CROWD_REPORT.md): current 0.4.0 implementation, verification, and remaining checks.
2. [Overlap report](docs/OVERLAP_REPORT.md): historical 0.3.0 acceptance evidence.
3. [Terrain report](docs/TERRAIN_REPORT.md): historical 0.2.0 evidence.
4. [Foundation report](docs/FOUNDATION_REPORT.md): historical 0.1.0 evidence.
5. [Product decisions](docs/PRODUCT_DECISIONS.md): confirmed direction, defaults, and open decisions.
6. [Gameplay checklist](docs/GAMEPLAY_CHECKLIST.md): stable acceptance IDs and their states.
7. [Development plan](DEVELOPMENT_PLAN.md): stages, architecture, and completion gates.
8. [Asset provenance](docs/ASSET_PROVENANCE.md): asset ownership and attribution.

This guide is a continuation aid. Update the checklist and milestone reports as evidence changes rather than letting this dated baseline override later results.

## 2. What exists today

| Area | Implementation for 0.4.0 |
| --- | --- |
| Park | Deterministic 32 × 32 site with grass, water, and sand. Starts in Terraced mode at heights 0/2/4/6; Flat mode remains available for comparison. |
| Network and overlap | 37 path nodes at height 8 form a loop, junctions, and branches around the original walkway. Nine track tiles remain at x13/y12..20, height 0. Each occupies two height steps. |
| Guests | Twelve seeded guests route over same-height cardinal path connections, walk between nodes, wait at destinations, and enter a defined lost state when disconnected. |
| Presentation | Original terrain, cliffs, connected path decks, rails, and a separate guest mesh. Surface mode hides buried track. Underground removes terrain and uses a non-selectable reference floor. No external game art is required. |
| Camera | Four rotations, keyboard and right/middle-drag panning, zoom, surface-height-aware fit, and grid toggle. |
| Inspection | Hover and selected-surface highlights with stable terrain/object identity; inspector distinguishes kind, object ID, tile, and height. Hidden surfaces cannot be picked through ordinary terrain. |
| UI | Pause/resume and 1x/2x/4x speeds, terrain T and underground U controls, Delete/Remove path, R/Restore paths, help, and total/walking/waiting/lost/arrival statistics. |
| Simulation | Separate dependency-free Rust library with bounded terrain, stable identities, clearance, atomic fixture restoration, deterministic routing/RNG/guest progress, and complete fixed ticks. |
| Diagnostics | Existing surface captures plus crowd/lost/recovered presets and a bounded simulation-tick argument; map, visibility, rotation, minimum-window options, PNG verification, and exit status. |
| Delivery | Windows packaging, native-runtime inspection/copying, dependency/font notices, checksums, and a GitHub Actions workflow. |

**Not implemented:** general terrain/path/track editors, slopes, complete player construction, ride queues/boarding, guest needs/spending, ride operations, coaster design, staff, economy, scenarios, research, audio, persistent settings, park saves, or loading. Delete/Restore paths are proof controls, not a full construction system. Guests are not individually selectable. Preset switching preserves time, terrain materials, object identities, and object heights; Flat exposes the same track that Terraced buries. Each normal launch creates a fresh seeded demonstration. Trees, entrance arch, and underground reference floor are decorative.

**The 0.4.0 crowd proof is verified (PRF-02).** All 72 tests passed: 30 simulation, 20 world/rendering, 18 input-system, and four main-module tests. Formatting, release Clippy with warnings denied, and release compilation passed. Eleven engine captures completed with exit code 0 and were visually inspected, including all four crowd rotations, isolated/recovered guests, minimum-window layouts, Flat terrain, and Underground track. The [crowd report](docs/CROWD_REPORT.md) records the exact executable SHA-256 and evidence; current package and hosted-CI verification are recorded separately there.

Native input through `@oai/sky` verified normal path removal/restoration at 1280 × 800. Space paused at 00:21 with 29 destinations reached; wheel zoom changed 100% → 162%. Removing occupied Path #37 with the button cleared selection, left twelve guests and the paused time intact, and showed one orange lost guest. R restored the same coordinate as fresh Path #47. Resume recovered the guest; pausing later at 00:42 showed 59 arrivals. A second remove/restore-button cycle, help open/close without click-through, and native close also passed. Two helper-injected Delete attempts had no effect: the headless shortcut test passes, but **physical Delete remains a retest**. Use the verified Remove path button in the meantime.

The **0.3.0 overlap proof is verified (PRF-01)**. Source commit `0220c7b` passed [Windows CI run 39406042854](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39406042854): 51 tests, formatting, release Clippy with warnings denied, and Windows packaging. All 14 engine captures were inspected: Surface and Underground in four rotations, Flat in both views, and minimum-window path, track, hidden-selection, and help scenes. Every process exited with code 0. The relocated package ran from `C:\Windows` after extraction into a path with spaces and `é`; all 17 payload hashes matched. See the overlap report for the exact executable hash and detailed evidence.

Historical 0.3.0 native input at 1280 × 800 selected Path #7 (display tile 15,17, height 8), revealed track with U, then selected Track #18 (14,18, height 0). The visibility button hid the track while retaining its ID and burial hint. Space paused at 02:14; Flat exposed the same track, all four rotations preserved identity/time, wheel zoom changed 100% → 113%, and Fit restored 100%. Hills hid the track again; clicking its old projection selected foreground terrain (12,19, height 6). Help/close preserved selection/time, and the native close button exited the process.

Historical **0.2.0** evidence includes 34 passing tests, formatting, Clippy with warnings denied, seven inspected engine captures, and a release Windows ZIP. Its 16 payload hashes matched after extraction into a separate path containing spaces and `é`; the extracted executable rendered selected raised terrain from an unrelated working directory and exited with code 0. See the terrain report. Clean-machine testing remains open; these results do not verify changed 0.4.0 code.

The historical **0.1.0** foundation report records 16 passing tests, release compilation, formatting, Clippy, three inspected images, and a relocated ZIP launch with matching payload hashes. Keep these results attached to their named checkpoint.

Normal desktop input was checked in 0.2.0 at 1280 × 800. Roof and visible cliff selection updated the inspector; T and the preset button preserved the selected tile while changing its elevation; rotation preserved selection; wheel zoom, Fit, and grid toggle worked. The timer advanced at 4x, Space paused it at 02:36, opening/closing help preserved that time and selection, Resume restored 4x, and the native close button ended the process. These observations plus authoritative clock tests verify FND-03.

Keyboard panning was not clearly established in the 0.3.0 native session, and right/middle dragging was unavailable through the desktop helper. Physical display scaling and multiple monitors remain untested, so FND-02 stays in progress. Clean-machine testing without development tools is also pending. These gaps do not invalidate the demonstrated PRF-01 render/pick-layer behavior; do not treat its completion as overall foundation or distribution acceptance.

### Checklist state at this handoff

| IDs | State | Meaning |
| --- | --- | --- |
| DEF-01, DEF-02 | Planned | Exact content/expansion audit and final product/platform decisions remain open. |
| FND-01 | Verified | Reverified in 0.3.0 through native launch/close, 14 inspected diagnostic captures, and relocated ZIP launch with all 17 payload hashes matching. |
| FND-02 | In progress | Native selection, four rotations, wheel zoom, and Fit passed in 0.3.0. Visible panning, right/middle dragging, and physical DPI/display checks remain open. |
| FND-03 | Verified | Earlier speed/inspector/visibility evidence is retained; 0.4.0 reverified native pause/resume, path-edit buttons, counters, and help, with complete-step speed/pause tests. |
| FND-04 | Verified | 0.4.0 passed all 72 local tests, formatting, release Clippy with warnings denied, and release compilation. Check the crowd report for the current package/hosted-CI status. |
| REL-01 | In progress | Packaging is implemented and a ZIP was checked; do not silently change its checklist state without reviewing its acceptance evidence. |
| PRF-01 | Verified | Named 0.3.0 tests, captures, native identity/visibility/preset checks, and package relocation establish the overlap baseline. New changes require regression checks. |
| PRF-02 | Verified | 0.4.0 automated invariants, eleven inspected captures, and native path deletion/restoration/recovery establish the proof. Physical Delete shortcut retesting remains explicit; Remove path is verified. |
| PRF-03 through PRF-05 | Planned | Train operation follows accepted crowd recovery, then save continuation and benchmark proofs. |
| Remaining gameplay/release entries | Planned | Consult the checklist for individual acceptance requirements. |

Stage 1 is not fully accepted until its remaining panning/dragging, display, and fresh-machine requirements pass. Stage 2 has completed PRF-01 and PRF-02 and continues with PRF-03; the complete technical-proof stage is not finished.

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

That second PC lacked MSVC build tools and the Windows SDK because their installer could not obtain UAC elevation; its verified 0.3.0 runtime/package came from CI. The current 0.4.0 session is back on the original OneDrive development PC, which previously built 0.2.0 locally. Inspect the actual toolchain on whichever PC is active rather than carrying the other machine's installation state forward. Install missing prerequisites before local builds, or use the hosted Windows workflow.

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
git switch --track origin/codex/path-crowds
git status --short --branch
git remote -v
rustup show active-toolchain
rustc -vV
cargo --version
```

The branch-switch example assumes the published 0.4.0 branch still exists and has not been merged elsewhere. Inspect `git branch -a` and the current PR before substituting another branch; a plain clone initially checks out the default branch, which may be older. If the local feature branch already exists, use `git switch codex/path-crowds` instead of `--track`.

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
| Space | Pause; resume the previously chosen speed. |
| 1 / 2 / 3 | Choose 1x / 2x / 4x simulation speed. |
| H / F1 | Toggle help. |
| Escape | Close help, otherwise clear selection. |
| Window close / Alt+F4 | Exit. |

Also test toolbar buttons, help-close click consumption, selection beside UI borders, dragging after focus changes, minimum window size, and display scaling. In both terrain presets and visibility modes, rotate through all four views and inspect the shared crossing. Buried track must not be selected through terrain in Surface view; the nearest visible path/track surface must win underground. Select a track in Underground, return to Surface, and verify its hidden outline does not show through terrain. Switch to Flat and confirm the same ID becomes exposed without moving. Check that the underground floor and absent terrain are never selected. Pause and confirm the timer stops; help itself does not automatically pause normal play. The inspector displays one-based coordinates, while `TilePos` uses zero-based coordinates.

FND-03 retains recorded 0.2.0 evidence and passed the relevant 0.3.0 control regressions described above. Record remaining panning/dragging and display results, Windows/GPU details, and limitations before marking FND-02 verified. Repeat affected checks when controls or layout change.

For the crowd proof, watch guests and arrival counts advance, then pause before deleting a path. A deletion must reconcile guest references even while paused. Resume to observe rerouting or a lost state; isolate a remaining path to exercise disconnection. Restore paths and confirm local recovery without resetting time or guests. Test removal of occupied, next, and destination nodes separately. Guests must not jump to distant paths or retain removed IDs. The 0.4.0 native session verified Remove path, R, and Restore; physical Delete needs a retest because the helper's two injections had no effect despite the passing headless key test.

## 5. Capture a rendered diagnostic frame

The app supports:

```text
parkbound.exe --capture-frame <absolute-output.png>
  [--capture-scene overview|selection|path|track|crowd|lost|recovered|help]
  [--capture-map flat|terraced]
  [--capture-view surface|underground]
  [--capture-rotation 0|1|2|3]
  [--capture-ticks 0..36000]
  [--capture-small-window]
```

It installs diagnostic systems only when `--capture-frame` is present. The preset advances the requested number of complete simulation ticks (default 600), then pauses and suppresses keyboard, map-pointer, and UI-button actions and hover tinting. `selection` chooses terrain; `path` and `track` choose fixture objects. `crowd` frames the network at 160% zoom. `lost` isolates the first guest's current node by removing its neighbors, then advances 60 ticks; `recovered` additionally restores paths and advances 600 ticks. These are controlled demonstrations, not synthetic clicks on the player controls.

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

Repeat with `crowd`, `lost`, `recovered`, `selection`, `path`, and `help`; compare Flat/Terraced and Surface/Underground in all four rotations, including minimum-window captures. Inspect guest positions, missing path gaps, lost markers, recovery, crossing occlusion, selection outlines, and legible controls/counters. Compare multiple simulation tick counts to establish movement, not merely the existence of markers. A valid PNG and successful exit do not prove visual correctness. Do not report another application's screenshot as game evidence. `artifacts` is ignored by Git; describe exact evidence in the crowd report.

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

For version 0.4.0, expected output after successful packaging is:

```text
dist/
  Parkbound-0.4.0-windows-x64/
    Parkbound.exe
    ...runtime, notices, documentation, manifests...
  Parkbound-0.4.0-windows-x64.zip
  Parkbound-0.4.0-windows-x64.zip.sha256
```

Packaging derives the version from Cargo metadata. It collects the locked Windows dependency notices, the embedded font license, Rust standard-library notices, and required MSVC redistributable DLLs. It writes `DEPENDENCIES.json`, `NATIVE_DEPENDENCIES.json`, `THIRD_PARTY_NOTICES.txt`, and `SHA256SUMS.txt`. Existing extracted package directories are preserved with a `.previous-...` suffix; the current ZIP is replaced. Check `dist` occasionally for accumulated previous packages.

The native runtime helper uses Visual Studio 2022's `dumpbin` to inspect imported DLLs recursively and copies the required runtime files from its redistributable directory. The foundation package needed `VCRUNTIME140.dll`. Other Windows system/UCRT libraries come from the operating system. Do not copy arbitrary DLLs from a machine's system folder or assume import inspection proves clean-machine compatibility.

If packaging reports missing Rust standard-library notices, install `rust-docs` for the pinned toolchain. If it reports a missing Visual Studio redistributable directory or `dumpbin`, repair the required C++ installation. Supplementary license source/hash failures require review; do not bypass them.

Check the ZIP hash against its adjacent `.sha256` file:

```powershell
Get-FileHash -LiteralPath 'dist\Parkbound-0.4.0-windows-x64.zip' -Algorithm SHA256
Get-Content -LiteralPath 'dist\Parkbound-0.4.0-windows-x64.zip.sha256'
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
| `crates/park_game/src/main.rs` | Bevy app setup, resources, fixed update, camera layout, keyboard/pointer input, scene refresh, and input-state tests. |
| `crates/park_game/src/world.rs` | Terrain/object projection, connected path geometry, separate guest mesh, shared surface picking/selection, visibility, underground reference floor, and geometry tests. |
| `crates/park_game/src/ui.rs` | UI layout, actions, labels, help modal, button state, and refresh. |
| `crates/park_game/src/capture.rs` | Opt-in scene/map/visibility/rotation/window capture presets, screenshot receipt, PNG validation, and diagnostic exit. |
| `scripts/package-windows.ps1` | Release build/staging, metadata, package documentation, notices, archive, and hashes. |
| `scripts/write-third-party-notices.ps1` | Locked dependency traversal and license text collection. |
| `scripts/copy-windows-runtime.ps1` | Native import inspection, MSVC runtime copying, and native manifest. |
| `scripts/notices/` | Reviewed supplementary license texts, original license documents, and exact source/hash records. Keep these in Git. |
| `.github/workflows/windows.yml` | Windows checks, packaging, and temporary review artifact upload. |
| `docs/` | Product decisions, acceptance checklist, build evidence, and asset provenance. |

### Simulation boundary

`park_sim` has no Bevy dependency. `Park` owns dimensions, a row-major `Vec<Tile>`, a `u64` tick count, `Vec<SurfaceObject>`, guest state, the stored crowd seed/RNG state, and monotonically increasing object/guest-ID allocators. Dimensions are validated in `1..=256`; invalid map construction panics. Tile access is bounds checked. A `Tile` has a `Terrain` enum and `height: u8` in abstract steps; `MAX_TERRAIN_HEIGHT` is 32.

`Park::new` creates flat terrain without objects or guests. Normal startup applies Terraced and installs the crowd fixture, which includes the original overlap objects. `try_apply_terrain_profile` validates every proposed height before mutation; it preserves materials, time, IDs, and object heights. `apply_terrain_profile` remains a convenience wrapper that panics on a rejected profile without changing state; use the fallible API for player actions. `set_terrain_height` validates bounds, height, and object clearance atomically.

`SurfaceId` distinguishes `Terrain(TilePos)` from `Object(ObjectId)`. A `SurfaceObject` contains ID, tile, `Path`/`Track` kind, X/Y axis, height, and clearance. `place_object(pos, kind, axis, height, clearance)` returns a stable ID or `PlacementError`; `remove_object(id)` returns the removed object or `None`, and never recycles an ID. Occupied intervals are half-open, with positive clearance and total height at most 32. Shared-tile overlaps are rejected. Paths stay on/above terrain; track can also be wholly buried. Terrain cannot bisect an object.

Read-only APIs are `objects()`, `object(id)`, `surface_position(id)`, `surface_height(id)`, `is_underground(id)`, and `max_surface_height()`. Invalid/removed references resolve to `None`, or `false` for the burial query. Maximum surface height excludes the empty clearance above it. `install_overlap_fixture()` is atomic, requires a 32 × 32 map, and preserves already matching objects on repeat installation.

`crowds.rs` adds `GuestId`, `GuestState::{Walking, Waiting, Lost}`, `Guest`, `GuestPosition`, and `CrowdStats`. Read-only APIs are `guests()`, `crowd_stats()`, `guest_position(id)`, and `path_neighbors(id)`. A guest stores current/next/destination path IDs, a retained tile/height anchor, integer progress, state, arrival count, and private route/wait data. Missing guest IDs return no presentation position.

Paths connect only through cardinal adjacency at equal height; diagonal neighbors, vertically stacked surfaces, and track objects do not connect. Axis affects decoration, not path connectivity. Deterministic BFS uses stable object-ID order. Route choice uses stored SplitMix64 integer state. Walking takes 30 complete ticks per edge and destination dwell takes 30 ticks. Initial guests have small deterministic start delays. A deliberately small graph is rebuilt from authoritative objects each tick, so edits cannot leave a stale navigation cache. Measure before adopting a larger graph cache.

`install_crowd_fixture(seed)` atomically installs the 37-node network, retained overlap track, and twelve guests. Repeating it preserves existing guest identities, time, and RNG state, even if a different seed is supplied. `restore_crowd_paths()` restores missing demonstration paths only; it preserves current guests and existing objects and rejects a collision atomically. New paths get new IDs.

`remove_object(id)` now reconciles guest references immediately. If removal affects a guest's current/next/destination/route, that route is canceled, movement returns to its retained starting anchor, and state becomes Lost. A vanished current path clears the current ID. On a later tick, a guest may replan in its component; recovery without a current path searches only the retained tile and cardinal same-height neighbors, ordered by distance then stable ID. It never chooses an arbitrary nearest path elsewhere in the park. Isolated nodes remain Lost until reconnected. These rules also apply when removal happens while paused.

The app schedules `FixedUpdate` at 30 Hz and calls `Park::advance(speed)`. This now executes zero, one, two, or four **complete authoritative steps**. Each step advances the clock then the crowd; reaching `u64::MAX` stops both. New simulation systems must join that complete step in explicit order. Do not return to merely increasing the clock for accelerated speed.

There is no general gameplay command queue, sloped/queue connectivity, admission, guest needs/spending, collision avoidance, ride routing, train simulation, money, or serialization yet. Placement and proof restoration are not complete ownership/cost construction commands.

The intended extension remains: validated atomic commands → fixed-step simulation → state/events → presentation. Store future money in integer minor units; use stable simulation/content IDs; preserve deterministic iteration and seeded randomness. Do not store Bevy entities or rendering caches as the long-term save representation.

### Presentation boundary

`Simulation(Park)` and `ViewState` are Bevy resources. `ViewState` owns presentation state and current speed: rotation, zoom, pan, grid, Surface/Underground visibility, `SurfaceId` hover/selection, help state, drag state, and flags. `park_sim` remains authoritative for park data. A separate `park_content` crate is planned only when authored content definitions justify it; it does not exist yet.

The Update systems are chained deliberately:

```text
UI button actions → keyboard input → apply pending terrain preset → apply path actions
→ pointer input → camera update → mesh update → UI refresh
```

Path-removal/restoration requests apply after terrain presets and before picking and scene refresh. Button actions clear and then set `ui_pointer_consumed` for that frame. Pointer handling checks it so a button click cannot also select a tile behind it. Pending terrain presets and path edits apply before picking, so a frame does not inspect stale surfaces. Preserve this ordering and its regression tests when refactoring UI.

`world.rs` uses 64 × 32 world-unit diamonds. `HEIGHT_STEP = 16` converts one simulation height step to presentation units. Geometry, depth, visibility, picking, and selection must agree across terrain and object layers. A cliff selects its terrain column; a path or track surface selects its own object ID. Ordinary picking excludes buried objects. Underground mode removes terrain from rendering/picking; its reference floor is non-selectable. Selection must obey scene occlusion rather than being forced above it. The legacy terrain-only picking wrapper remains for baseline tests. General map sizes require coordinated changes to square-map presentation assumptions, not only a larger `Park`.

The camera and pointer transform use logical window pixels consistently. Mouse dragging uses successive logical cursor positions; replacing that with raw mouse-motion counts can reintroduce DPI-dependent behavior. UI exclusion bounds and camera offsets depend on `SIDEBAR`, `HEADER`, and `FOOTER`; change the layout and these boundaries together.

Terrain, path, and track surfaces use a static procedural mesh rebuilt when the view marks it dirty. Preset, visibility, removal, and restoration changes must invalidate scene and selection geometry even if selected identity stays unchanged. Selection is a separate mesh. Guests use their own dynamic `world::guest_mesh` so walking alone does not rebuild terrain. Guest rendering interpolates authoritative endpoint/progress data and applies shared depth; markers are not independent pick targets. `NoFrustumCulling` avoids stale cached bounds on meshes whose vertices change. Retain that treatment or refresh bounds correctly. The proof is not a measured large-park rendering architecture.

Zoom is bounded to 0.6–3.5; pan is bounded; Fit resets zoom/pan while retaining selection and accounts for terrain/object height. Keep the same camera layout calculation in rendering and pointer conversion. Rendering uses opaque white materials with vertex colors. Release builds use the Windows GUI subsystem, so opening the executable does not supply a normal console log window. Use capture exit status and the relevant debugger/log strategy when diagnosing a silent launch failure.

## 8. Resume work in this sequence

### A. Reproduce and close the foundation gaps

Build from the existing or newly cloned repository using the pinned toolchain; record the commit, Windows version, graphics driver, and check results. Complete ordinary input checks and clean-machine package checks where the new PC makes that possible. Update individual IDs only after their acceptance criteria pass. Keep the scope/catalogue audit visible rather than treating it as finished because coding has started.

### B. Preserve the accepted PRF-01 overlap baseline

The 0.3.0 overlap fixture, stable identity, visibility, and clearance proof is accepted. Retain it as a regression scene while developing navigation. Its historical evidence is recorded in the overlap report; record current regression results in the crowd report.

1. Run simulation, geometry, input, formatting, lint, and release checks on the exact current source; inspect hosted CI for that commit.
2. Demonstrate the shared crossing in all four rotations and both visibility modes, including Flat/Terraced comparison, zoom, pan, UI exclusion, and boundaries.
3. Verify hidden track and hidden selection do not show through terrain; check the underground reference floor cannot be selected.
4. Verify IDs and object heights survive view/preset changes; rejected clearance and removed references remain safe.
5. Record normal mouse/keyboard behavior and outstanding physical display or hardware limitations separately from synthetic tests; FND-02 remains open.
6. Package and inspect each new checkpoint, recording hashes and relocation results. Prior-build evidence does not automatically verify changed code.

Shared clearance and surface rules should become the foundation for paths, track, scenery, and construction. Reject illegal placement atomically once commands are introduced.

### C. Preserve the accepted PRF-02 crowd baseline

Version 0.4.0 verifies this small proof through 72 passing tests, eleven inspected captures, and native removal/restoration/recovery controls. Read the crowd report and preserve same-height/cardinal connectivity, stable IDs, complete ticks, and local lost recovery as invariants. Do not start a second crowd implementation. Retest the physical Delete shortcut and complete any remaining package/hosted-CI checks; no display-rate or large-park performance measurement is claimed by this proof.

Proceed to PRF-03 below. Queue ownership/access rules, slopes, actual guest entrance/exit, attraction routing, needs, and collision avoidance remain later work; the current graph treats all existing Path objects as public walkable nodes.

### D. Prove one moving train: PRF-03

Create a representative station → lift → drop → brake → station circuit. Keep track geometry, train movement, operating state, and rendering separate. Use track-constrained motion rather than adopting general rigid-body simulation by default. Prove stable movement across display frame rates, clear endpoint/connection validation, and reproducible simulation stepping. This is a technical circuit, not completion of the custom coaster editor or every ride family.

### E. Prove save/reload continuation: PRF-04

Define a versioned authoritative save schema with IDs, world state, time, content versions, and random-generator state. Validate before accepting data. Write safely via a temporary file and replacement, keep backups, and rebuild render/navigation caches after loading. Test that uninterrupted execution and save → reload → continuation produce matching authoritative state for the same build and commands. Include malformed/truncated/unsupported input cases without overwriting a good save.

### F. Measure before scaling: PRF-05

The proposed benchmark is 256 × 256 tiles, 5,000 guests, and 50 operating rides, aiming at 60 FPS at 1080p on a recorded reference PC. **These are design targets, not achieved capabilities.** Measure frame time, simulation time, routing time, memory, save size, and loading. Use results to approve or revise architecture before producing a large content catalogue.

### G. Continue the full milestones

After technical proofs, Stage 3 integrates the first complete park loop: construction, paths, a flat ride, a shop, guests, queues, pricing, money, demolition, inspection, and saves. Stages 4–7 expand coasters, operations, ride families, original content, scenarios, and editors. Stage 8 covers performance, stability, balance, accessibility, and polish; Stage 9 closes public distribution. Keep each feature's persistence, UI, acceptance demonstration, and relevant tests in the same milestone.

Do not start all of these systems in one change. Select a small observable behavior, implement it end to end, verify it, update the checklist/report, and retain a working build.

## 9. Work safely across two PCs

Use GitHub as the shared source history. This 0.4.0 session uses the original OneDrive folder recorded above. The separate second-PC clone remains `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`, with no verified OneDrive sync. For separate clones, push completed work to GitHub and fetch it on the receiving PC. For the shared folder, finish the editing/build/Git session on the first PC, let OneDrive finish syncing on both PCs, then check Git status and the expected commit on the receiving PC. Use only one active editing/build/Git session for that shared checkout. Rebuild generated output as needed rather than relying on another PC's `target` cache.

Before a work session:

```powershell
git status --short --branch
git fetch origin
git remote show origin
```

Check the default branch and outstanding pull requests. This checkpoint is on `codex/path-crowds`, stacked on the unmerged `codex/overlapping-surfaces` work when the session began. Preserve that ancestry and review the actual merge status before changing a PR base. Only switch/pull with a clean working tree or after deliberately preserving local changes.

To resume the published feature branch on a PC without a local copy:

```powershell
git switch --track origin/codex/path-crowds
```

If the local branch already exists:

```powershell
git switch codex/path-crowds
git pull --ff-only origin codex/path-crowds
```

After the crowd work merges, resume from the verified merge on `main` instead. If opening the subsequent train proof before that merge, branch from the current crowd checkpoint and keep the review stacked deliberately. When a fast-forward fails, inspect the histories and resolve intentionally; do not force-push or reset away the other PC's changes.

Before leaving a PC, run relevant checks, inspect the diff, explicitly stage the intended source/docs, commit, and push the branch:

```powershell
git diff --check
git diff --stat
git status --short
# Stage the specific files changed for this task, then inspect what will be saved.
git diff --cached --stat
git diff --cached
git commit -m "Add deterministic path navigation and crowd recovery"
git push -u origin codex/path-crowds
```

The staging step is intentionally task-specific; do not accidentally include credentials, unrelated or unrequested private machine paths, diagnostics from other apps, executables, caches, or unrelated files. Ensure `Cargo.lock`, relevant notices, and documentation changes are included when they belong to the change. Do not commit generated `target`, `dist`, or `artifacts` directories. Preserve `.gitattributes`: supplementary notice files have recorded SHA-256 hashes, so changing their line endings through Windows Git settings would break verification. They are deliberately excluded from text normalization.

Use a pull request to review substantial changes and preserve the playable default branch. Update the milestone report with the commit, commands/check results, exact next acceptance ID, known blockers, and unresolved decisions. Push work before moving PCs; an unpushed local commit or stash is not backed up on GitHub and may not yet have reached the other PC through OneDrive. On the receiving PC, verify the expected commit with `git log -1 --oneline` before starting.

## 10. CI, licensing, and release decisions

The Windows workflow runs on pushes, pull requests, and manual dispatch. It uses a Windows 2022 runner, Rust 1.93.1 with formatting/lint/docs components, formatting, release-profile Clippy with warnings denied, release-profile tests, packaging, and artifact upload. The review artifact retention is 14 days. Inspect the real run for the intended commit; configuration alone is not a passing CI result. The workflow has read-only repository contents permission and does not publish a GitHub Release.

The following hosted runs were verified successful on 20 October 2026:

| Commit | Hosted Windows run |
| --- | --- |
| `e4a8d2f` | [Successful run 39265273354](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39265273354) |
| `2ec6bf9` | [Successful run 39379555949](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39379555949) |
| `dc832fa` (0.2.0 starting baseline) | [Successful run 39381924927](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39381924927) |
| `0220c7b` (0.3.0 overlap runtime source) | [Successful run 39406042854](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39406042854): 51 tests, formatting, release Clippy, and Windows package |

These results belong to their exact commits. The 0.3.0 executable from `0220c7b` was also exercised locally through render captures, native input, and relocation. Later source changes need their own checks; documentation-only evidence updates do not replace the recorded runtime-source identity.

There is no selected project source-code license yet. Free gameplay distribution and permission to reuse source are separate decisions. Saving source in this GitHub repository does not by itself select MIT, Apache, GPL, or another project license. Decide source licensing before making reuse promises or accepting outside contributions. Keep all third-party licenses and asset provenance intact.

No original RCT code, artwork, music, scenarios, or other original game files are bundled or required. Do not import those files or copy code from reference projects without reviewing applicable rights and obligations. The font has its own license; it is not project-authored art. Update provenance and dependency notices for every adopted external asset or dependency.

Public naming, final hardware/Windows requirements, signing, installer choice, source license, catalogue/expansion scope, and public release channel still need resolution. The proposed download path is ZIPs first, then installer/public distribution. Existing local builds are unsigned. Publishing a source checkpoint is not acceptance of the full game's release criteria.

## 11. Copy-paste prompt for the next coding assistant

```text
Continue this existing Rust project from https://github.com/strijbosmh-afk/RailRust-Tycoon.
The repository is RailRust-Tycoon; Parkbound is only the temporary application title.
Read RESUME_DEVELOPMENT.md, docs/CROWD_REPORT.md, docs/OVERLAP_REPORT.md,
docs/TERRAIN_REPORT.md, docs/FOUNDATION_REPORT.md, docs/PRODUCT_DECISIONS.md,
docs/GAMEPLAY_CHECKLIST.md, and DEVELOPMENT_PLAN.md,
then inspect the actual source and current Git state. Preserve unrelated local
changes and remote history. Current feature branch is codex/path-crowds, based on
codex/overlapping-surfaces; inspect their PR/merge state before choosing a base.
Main may still be older. The current session returned to the original shared
OneDrive path; the separate second-PC path and single-active-session workflow are
also recorded in the guide.

The confirmed product is a free, fully featured, Windows-only classic RCT1/RCT2-style
isometric park simulation in Rust, developed in stages with original assets. We have
a version 0.4.0 path/crowd proof, not a finished game. It retains the accepted 0.3.0
overlap baseline and expands to 37 path nodes at height 8, nine track nodes at
height 0, and 12 seeded guests. Routing uses deterministic BFS over cardinal
same-height paths.
Every accelerated tick runs the full simulation. Delete removes a selected path;
R restores missing fixture paths with fresh IDs, preserving time, guests/RNG and
surviving objects. Guests lose invalid references immediately, retain an anchor,
and recover locally rather than teleporting to distant paths. Guests use a separate
dynamic mesh and are not selectable. U/T and stable surface selection remain.
No full construction, slopes, ride queues/boarding, needs/spending, operating rides,
finances, saves, or audio exist. PRF-02 is verified through 72 passing tests,
eleven inspected engine captures, and native Remove/Restore buttons plus R.
Native path #37 became fresh #47 on restoration, and guests recovered on resume.
Read the crowd report for the exact executable hash and package/hosted-CI status.
Helper-injected Delete had no effect, so retest the physical shortcut despite its
passing headless test. FND-02 pan/drag/physical-display and clean-machine checks
remain open. Do not reuse historical 0.3.0 results as current evidence.

Use the repository-pinned Rust 1.93.1 MSVC toolchain and Bevy 0.17.3 with Cargo.lock.
On an 8 GB machine use -j 2. First reproduce the baseline checks and inspect CI for
the current commit. Keep park_sim independent of Bevy. Read the input-ordering,
logical-pixel, mesh-culling, and accelerated-tick notes before refactoring them.

Implement the next small PRF-03 slice: a station/lift/drop/brake/station train
circuit with deterministic movement. Carry the explicit Delete shortcut retest
and remaining delivery checks forward; do not reopen the completed crowd proof.
Preserve crowd recovery, speed equivalence, complete ticks, shared clearance,
surface identity, all four rotations, both terrain presets, and both visibility
modes. Keep save/reload and benchmarks as later independent proofs. Continue the
content/expansion audit and available foundation checks separately.

Run relevant tests, formatting, Clippy, a release build, render inspection, and normal
input checks where available. Document actual evidence and limitations, update the
checklist and milestone report, and create a reproducible Windows package. Do not
claim manual input, clean-machine compatibility, hosted CI success, or public release
without evidence. Do not choose a final name or project source license implicitly.
Keep the result reviewable and playable before starting another major subsystem.
```
