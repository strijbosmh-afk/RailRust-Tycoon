# Resume development on another Windows PC

This is the handoff guide for **RailRust-Tycoon**, the repository for an original Rust theme park simulation inspired by classic RollerCoaster Tycoon 1 and 2. The application currently displays **Parkbound**, a temporary working title; the repository name does not rename the application. Baseline: foundation version **0.1.0**, 19 October 2026.

Canonical repository: <https://github.com/strijbosmh-afk/RailRust-Tycoon>.

The repository contains the source, locked dependencies, build scripts, notices, and development records needed to rebuild on another PC. Generated executables, ZIPs, screenshots, and Cargo build caches are deliberately ignored by Git. Continue from the synced workspace below or clone the repository, then rebuild as needed; copying a previous PC's `target` folder is unnecessary.

### Shared OneDrive working folder

The existing workspace is in the user's shared OneDrive folder. Its full local path on the original PC, recorded on 20 October 2026, is:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
```

The path within that OneDrive root is `Documenten\ChatGPT\Railroad Tycoon`. On another PC, the Windows username or OneDrive root may differ; locate the synced folder and open it as the project workspace. The folder name remains `Railroad Tycoon`, while the GitHub repository is `RailRust-Tycoon` and the application's temporary title is `Parkbound`.

On the original PC, open the workspace in PowerShell with:

```powershell
Set-Location -LiteralPath 'C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon'
```

You can continue from this synced workspace or use a separate GitHub clone. For the shared workspace, let OneDrive finish syncing before continuing and follow the two-PC workflow in section 9. The clone commands below are for setting up a separate checkout; they are unnecessary if you are opening the existing synced workspace.

## 1. Read this first

The user confirmed:

- Rust implementation, classic RCT1/RCT2 isometric construction and park management.
- Windows only, with free distribution and all functionality included.
- Development in playable stages, eventually covering the agreed complete classic gameplay inventory.

Current defaults are offline single-player, an independent simulation library, original assets, four isometric orientations, and versioned local saves when persistence is implemented. Windows 11 x64 is the first build/test target, not a proven minimum requirement. Multiplayer, mobile/console ports, and scripting are outside the current plan. Expansion content and original-file import compatibility remain undecided.

**The first executable is a foundation, not a finished management game.** Do not infer completion from the visible park or the existence of a package.

Read the project records in this order:

1. [Foundation report](docs/FOUNDATION_REPORT.md): what actually ran and what remains unverified.
2. [Product decisions](docs/PRODUCT_DECISIONS.md): confirmed direction, defaults, and open decisions.
3. [Gameplay checklist](docs/GAMEPLAY_CHECKLIST.md): stable acceptance IDs and their states.
4. [Development plan](DEVELOPMENT_PLAN.md): stages, architecture, and completion gates.
5. [Asset provenance](docs/ASSET_PROVENANCE.md): asset ownership and attribution.

This guide is a continuation aid. Update the checklist and milestone reports as evidence changes rather than letting this dated baseline override later results.

## 2. What exists today

| Area | Implemented in 0.1.0 |
| --- | --- |
| Park | Deterministic 32 × 32 flat tile site with grass, water, and sand. |
| Presentation | Original procedural geometry, lake/shore, decorative trees, soil sides, shadows, and an entrance arch. No external game art is required. |
| Camera | Four rotations, keyboard and right/middle-drag panning, zoom, fit, and grid toggle. |
| Inspection | Hover and selected-tile highlights; terrain and coordinate inspector. |
| UI | Header, sidebar, footer, pause/resume, 1x/2x/4x speeds, and help modal. |
| Simulation | Separate dependency-free Rust library with bounded map storage and a fixed-step clock. |
| Diagnostics | Optional overview, selection, and help render captures with PNG verification and exit status. |
| Delivery | Windows packaging, native-runtime inspection/copying, dependency/font notices, checksums, and a GitHub Actions workflow. |

**Not implemented:** terrain editing or actual elevation, paths, guests, queues, construction, ride operations, coaster design, staff, economy, scenarios, research, audio, persistent settings, park saves, or loading. Each normal launch creates a fresh park. The trees and entrance arch are decoration, not simulation objects or usable facilities.

The foundation has 16 passing tests recorded in its report: seven simulation tests, five projection/geometry tests, and four camera/UI-input-state tests. Release compilation, formatting, and Clippy passed on the recorded development host. Three engine-rendered images were inspected. The extracted ZIP ran outside the source folder, from a path containing spaces and an accented character; its payload hashes matched.

Native mouse/keyboard interaction was **not** manually verified in the initial session: the desktop capture helper returned an unrelated application image, so rendered captures and logic tests were used and recorded separately. Clean-machine testing without development tools is still pending. Neither a local package nor this guide proves that a GitHub Actions run succeeded; inspect the repository's Actions results for the actual commit.

### Checklist state at this handoff

| IDs | State | Meaning |
| --- | --- | --- |
| DEF-01, DEF-02 | Planned | Exact content/expansion audit and final product/platform decisions remain open. |
| FND-01 | Verified | Native startup, rendering, diagnostic clean exit, and relocated ZIP launch were demonstrated. |
| FND-02, FND-03 | In progress | Camera/picking/UI/clock code exists; normal-input acceptance remains open. |
| FND-04 | Verified | Recorded automated checks and release compilation passed. |
| REL-01 | In progress | Packaging is implemented and a ZIP was checked; do not silently change its checklist state without reviewing its acceptance evidence. |
| PRF-01 through PRF-05 | Planned | Stage 2 height/picking, crowd, train, save-continuation, and benchmark proofs are next. |
| Remaining gameplay/release entries | Planned | Consult the checklist for individual acceptance requirements. |

Stage 1 is not fully accepted until its normal-input and fresh-machine requirements pass. Stage 2 can proceed in small slices while those checks are completed.

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
git status --short --branch
git remote -v
rustup show active-toolchain
rustc -vV
cargo --version
```

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
| Left click on map | Select the tile under the pointer; inspector and highlight agree. |
| Right or middle drag | Pan without a jump when dragging begins. |
| WASD / arrow keys | Pan with the keyboard. |
| Wheel / + / - | Zoom while preserving usable selection behavior. |
| Q / E | Rotate through four views; selected world tile remains the same. |
| Home / Fit | Reset zoom and pan; preserve selection. |
| G | Toggle grid. |
| Space | Pause; resume the previously chosen speed. |
| 1 / 2 / 3 | Choose 1x / 2x / 4x simulation speed. |
| H / F1 | Toggle help. |
| Escape | Close help, otherwise clear selection. |
| Window close / Alt+F4 | Exit. |

Also test toolbar buttons, help-close click consumption, selection beside UI borders, dragging after focus changes, minimum window size, and display scaling. Pause and confirm the simulation timer stops; help itself does not automatically pause normal play. The inspector displays one-based coordinates, while `TilePos` uses zero-based coordinates. The timer represents elapsed simulation time, not a park calendar.

Record actual results, Windows/GPU details, and limitations in a milestone report before marking FND-02 or FND-03 verified.

## 5. Capture a rendered diagnostic frame

The app supports:

```text
parkbound.exe --capture-frame <absolute-output.png> [--capture-scene overview|selection|help]
```

It installs diagnostic systems only when `--capture-frame` is present. The capture preset pauses simulation; `selection` rotates the map, zooms to 135%, and selects its center tile. A successful capture waits at least 90 frames and two seconds, checks the PNG, and exits with code 0. The timeout is 120 seconds. This requires a real rendering environment and is not a substitute for ordinary input testing.

Use a fresh output filename for each run. The implementation saves through a temporary file and renames it; an existing destination may cause failure on Windows. The following PowerShell example handles paths containing spaces and waits for the GUI process to exit:

```powershell
$repoDirectory = (Get-Location).Path
$captureDirectory = Join-Path $repoDirectory 'artifacts'
New-Item -ItemType Directory -Path $captureDirectory -Force
$capturePath = Join-Path $captureDirectory ("selection-{0}.png" -f (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
$gameExecutable = Join-Path $repoDirectory 'target\release\parkbound.exe'
$captureArguments = '--capture-frame "{0}" --capture-scene selection' -f $capturePath
$captureProcess = Start-Process -FilePath $gameExecutable -ArgumentList $captureArguments -WindowStyle Hidden -Wait -PassThru
if ($captureProcess.ExitCode -ne 0) { throw "Capture failed: exit code $($captureProcess.ExitCode)" }
Get-Item -LiteralPath $capturePath
```

Repeat with `overview` and `help`, then inspect the actual images. A valid PNG and successful exit do not prove the scene is visually correct. Do not report a screenshot from another application as game evidence. `artifacts` is ignored by Git; share selected evidence intentionally or describe it in the milestone report.

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

For version 0.1.0, output is:

```text
dist/
  Parkbound-0.1.0-windows-x64/
    Parkbound.exe
    ...runtime, notices, documentation, manifests...
  Parkbound-0.1.0-windows-x64.zip
  Parkbound-0.1.0-windows-x64.zip.sha256
```

Packaging derives the version from Cargo metadata. It collects the locked Windows dependency notices, the embedded font license, Rust standard-library notices, and required MSVC redistributable DLLs. It writes `DEPENDENCIES.json`, `NATIVE_DEPENDENCIES.json`, `THIRD_PARTY_NOTICES.txt`, and `SHA256SUMS.txt`. Existing extracted package directories are preserved with a `.previous-...` suffix; the current ZIP is replaced. Check `dist` occasionally for accumulated previous packages.

The native runtime helper uses Visual Studio 2022's `dumpbin` to inspect imported DLLs recursively and copies the required runtime files from its redistributable directory. The foundation package needed `VCRUNTIME140.dll`. Other Windows system/UCRT libraries come from the operating system. Do not copy arbitrary DLLs from a machine's system folder or assume import inspection proves clean-machine compatibility.

If packaging reports missing Rust standard-library notices, install `rust-docs` for the pinned toolchain. If it reports a missing Visual Studio redistributable directory or `dumpbin`, repair the required C++ installation. Supplementary license source/hash failures require review; do not bypass them.

Check the ZIP hash against its adjacent `.sha256` file:

```powershell
Get-FileHash -LiteralPath 'dist\Parkbound-0.1.0-windows-x64.zip' -Algorithm SHA256
Get-Content -LiteralPath 'dist\Parkbound-0.1.0-windows-x64.zip.sha256'
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
| `crates/park_sim/src/lib.rs` | Authoritative terrain/map storage, clock, speed enum, and simulation tests. |
| `crates/park_game/src/main.rs` | Bevy app setup, resources, fixed update, camera layout, keyboard/pointer input, scene refresh, and input-state tests. |
| `crates/park_game/src/world.rs` | Isometric projection and inverse picking, procedural meshes, selection mesh, and geometry tests. |
| `crates/park_game/src/ui.rs` | UI layout, actions, labels, help modal, button state, and refresh. |
| `crates/park_game/src/capture.rs` | Opt-in render capture presets, screenshot receipt, PNG validation, and diagnostic exit. |
| `scripts/package-windows.ps1` | Release build/staging, metadata, package documentation, notices, archive, and hashes. |
| `scripts/write-third-party-notices.ps1` | Locked dependency traversal and license text collection. |
| `scripts/copy-windows-runtime.ps1` | Native import inspection, MSVC runtime copying, and native manifest. |
| `scripts/notices/` | Reviewed supplementary license texts, original license documents, and exact source/hash records. Keep these in Git. |
| `.github/workflows/windows.yml` | Windows checks, packaging, and temporary review artifact upload. |
| `docs/` | Product decisions, acceptance checklist, build evidence, and asset provenance. |

### Simulation boundary

`park_sim` has no Bevy dependency. `Park` owns dimensions, a row-major `Vec<Tile>`, and a `u64` tick count. Dimensions are validated in `1..=256`; invalid construction currently panics. Tile access is bounds checked. A `Tile` has a `Terrain` enum and `height: u8`, but every generated tile has height zero, and rendering/picking are flat. The height field is not an elevation system.

Initial terrain uses deterministic integer formulas. There is no seeded random-generator state, command queue, object-ID allocation, navigation graph, guest system, train simulation, or serialization yet. These are planned contracts, not implemented capabilities.

The app schedules `FixedUpdate` at 30 Hz and calls `Park::advance(speed)`. That currently increments time by 0, 1, 2, or 4 ticks with saturating arithmetic. **When real systems are added, each accelerated tick must execute the complete authoritative step in explicit order.** Merely increasing the clock counter would make the simulated behavior disagree with displayed speed.

The intended extension remains: validated atomic commands → fixed-step simulation → state/events → presentation. Store future money in integer minor units; use stable simulation/content IDs; preserve deterministic iteration and seeded randomness. Do not store Bevy entities or rendering caches as the long-term save representation.

### Presentation boundary

`Simulation(Park)` and `ViewState` are Bevy resources. `ViewState` owns presentation state and current speed: rotation, zoom, pan, grid, hover/selection, help state, drag state, and flags. `park_sim` remains authoritative for park data. A separate `park_content` crate is planned only when real authored content definitions justify it; it does not exist yet.

The Update systems are chained deliberately:

```text
UI button actions → keyboard input → pointer input → camera update → mesh update → UI refresh
```

Button actions clear and then set `ui_pointer_consumed` for that frame. Pointer handling checks it so a click that closes help cannot also select a tile behind the modal. Preserve this ordering and its regression test when refactoring UI.

`world.rs` uses 64 × 32 world-unit diamonds, with rotation and inverse picking in the same module. Shared edges have deterministic ownership. The renderer currently requires a square park even though the simulation can store rectangular ones. `main.rs` additionally uses the fixed `MAP_SIZE = 32` for camera/picking/selection. General map sizes require coordinated changes, not only a larger `Park`.

The camera and pointer transform use logical window pixels consistently. Mouse dragging uses successive logical cursor positions; replacing that with raw mouse-motion counts can reintroduce DPI-dependent behavior. UI exclusion bounds and camera offsets depend on `SIDEBAR`, `HEADER`, and `FOOTER`; change the layout and these boundaries together.

The landscape is one procedural mesh, rebuilt when the view marks it dirty. Selection is a separate mesh, rebuilt when hover/selection/rotation changes. Selection has `NoFrustumCulling` because replacing moving vertices without updating a cached bounding box previously risked hiding the highlight. Heights, dynamic entities, and large parks need a deliberate depth/visibility and update strategy; the current monolithic mesh is a foundation convenience, not a proven large-park architecture.

Zoom is bounded to 0.6–3.5; pan is bounded; Fit resets zoom/pan while retaining selection. Rendering uses opaque white materials with vertex colors. Release builds use the Windows GUI subsystem, so opening the executable does not supply a normal console log window. Use capture exit status and the relevant debugger/log strategy when diagnosing a silent launch failure.

## 8. Resume work in this sequence

### A. Reproduce and close the foundation gaps

Build from the existing or newly cloned repository using the pinned toolchain; record the commit, Windows version, graphics driver, and check results. Complete ordinary input checks and clean-machine package checks where the new PC makes that possible. Update individual IDs only after their acceptance criteria pass. Keep the scope/catalogue audit visible rather than treating it as finished because coding has started.

### B. Implement the first part of PRF-01: height-aware terrain and picking

Make this the next small implementation slice. Preserve the current flat-map behavior and tests as the regression baseline.

1. Define height units, legal ranges, and simulation ownership. Decide how an elevated surface differs from terrain and how selectable surfaces are identified.
2. Add a small representative stepped-height scene using original geometry. Introduce the minimum authoritative data needed; do not build the full terrain editor first.
3. Implement projection, depth/occlusion, and height-aware picking together. The flat inverse diamond formula alone cannot identify stacked surfaces.
4. Demonstrate visible and hidden surfaces at different heights in all four rotations, with zoom, pan, UI exclusion, edge/boundary behavior, and stable selected world identity.
5. Add elevated-path and underground-track overlap fixtures to finish the full PRF-01 acceptance. A raised terrain demo alone does not close that ID.
6. Record a rendered demonstration and normal-input results; package the checkpoint before expanding the system.

Shared clearance and surface rules should become the foundation for paths, track, scenery, and construction. Reject illegal placement atomically once commands are introduced.

### C. Prove paths and crowd recovery: PRF-02

Introduce height-aware path connectivity, stable IDs, destination selection, and a small seeded guest simulation in `park_sim`. Start with a tiny deterministic test park. Test junctions, disconnected regions, and deletion of occupied/destination paths. Guests must enter a defined lost/recovery state rather than retaining invalid references. Render the result and verify behavior at different display frame rates before optimizing crowd size. Queue rules belong in connectivity semantics, even if detailed ride boarding comes later.

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

Use GitHub as the shared source history. The existing working directory is the shared OneDrive folder recorded above; a separate clone on each PC is also an option. When continuing in the shared folder, finish the editing/build/Git session on the first PC, let OneDrive finish syncing on both PCs, then check Git status and the expected commit on the receiving PC. Use only one active editing/build/Git session for that shared checkout. Rebuild generated output as needed rather than relying on another PC's `target` cache.

Before a work session:

```powershell
git status --short --branch
git fetch origin
git remote show origin
```

Check the remote's default branch name. The following examples use `main`; substitute the actual branch if the repository uses a different default. Only switch/pull with a clean working tree or after deliberately preserving your local changes:

```powershell
git switch main
git pull --ff-only origin main
git switch -c codex/height-aware-terrain
```

If continuing an existing branch on the second PC, use that branch instead of creating a parallel branch with the same intent:

```powershell
git switch --track origin/codex/height-aware-terrain
```

If it already exists locally, use `git switch codex/height-aware-terrain`, then `git pull --ff-only`. When a fast-forward fails, inspect the histories and resolve intentionally; do not force-push or reset away the other PC's changes.

Before leaving a PC, run relevant checks, inspect the diff, explicitly stage the intended source/docs, commit, and push the branch:

```powershell
git diff --check
git diff --stat
git status --short
# Stage the specific files changed for this task, then inspect what will be saved.
git diff --cached --stat
git diff --cached
git commit -m "Implement height-aware terrain picking"
git push -u origin codex/height-aware-terrain
```

The staging step is intentionally task-specific; do not accidentally include credentials, unrelated or unrequested private machine paths, diagnostics from other apps, executables, caches, or unrelated files. Ensure `Cargo.lock`, relevant notices, and documentation changes are included when they belong to the change. Do not commit generated `target`, `dist`, or `artifacts` directories. Preserve `.gitattributes`: supplementary notice files have recorded SHA-256 hashes, so changing their line endings through Windows Git settings would break verification. They are deliberately excluded from text normalization.

Use a pull request to review substantial changes and preserve the playable default branch. Update the milestone report with the commit, commands/check results, exact next acceptance ID, known blockers, and unresolved decisions. Push work before moving PCs; an unpushed local commit or stash is not backed up on GitHub and may not yet have reached the other PC through OneDrive. On the receiving PC, verify the expected commit with `git log -1 --oneline` before starting.

## 10. CI, licensing, and release decisions

The Windows workflow runs on pushes, pull requests, and manual dispatch. It uses a Windows 2022 runner, Rust 1.93.1 with formatting/lint/docs components, formatting, release-profile Clippy with warnings denied, release-profile tests, packaging, and artifact upload. The review artifact retention is 14 days. Inspect the real run for the intended commit; configuration alone is not a passing CI result. The workflow has read-only repository contents permission and does not publish a GitHub Release.

There is no selected project source-code license yet. Free gameplay distribution and permission to reuse source are separate decisions. Saving source in this GitHub repository does not by itself select MIT, Apache, GPL, or another project license. Decide source licensing before making reuse promises or accepting outside contributions. Keep all third-party licenses and asset provenance intact.

No original RCT code, artwork, music, scenarios, or other original game files are bundled or required. Do not import those files or copy code from reference projects without reviewing applicable rights and obligations. The font has its own license; it is not project-authored art. Update provenance and dependency notices for every adopted external asset or dependency.

Public naming, final hardware/Windows requirements, signing, installer choice, source license, catalogue/expansion scope, and public release channel still need resolution. The proposed download path is ZIPs first, then installer/public distribution. Existing local builds are unsigned. Publishing a source checkpoint is not acceptance of the full game's release criteria.

## 11. Copy-paste prompt for the next coding assistant

```text
Continue this existing Rust project from https://github.com/strijbosmh-afk/RailRust-Tycoon.
The repository is RailRust-Tycoon; Parkbound is only the temporary application title.
Read RESUME_DEVELOPMENT.md, docs/FOUNDATION_REPORT.md, docs/PRODUCT_DECISIONS.md,
docs/GAMEPLAY_CHECKLIST.md, and DEVELOPMENT_PLAN.md, then inspect the actual source
and current Git state. Preserve unrelated local changes and remote history.

The confirmed product is a free, fully featured, Windows-only classic RCT1/RCT2-style
isometric park simulation in Rust, developed in stages with original assets. We have
a version 0.1.0 foundation, not a finished game. Terrain is currently flat; no paths,
guests, rides, construction, finances, saves, or audio exist yet. Do not mark planned
contracts as implemented. FND-02/FND-03 normal-input and clean-machine checks remain
open unless later reports provide evidence.

Use the repository-pinned Rust 1.93.1 MSVC toolchain and Bevy 0.17.3 with Cargo.lock.
On an 8 GB machine use -j 2. First reproduce the baseline checks and inspect CI for
the current commit. Keep park_sim independent of Bevy. Read the input-ordering,
logical-pixel, mesh-culling, and accelerated-tick notes before refactoring them.

Implement the next small PRF-01 slice: height-aware terrain rendering and picking,
preserving flat-map behavior and all four rotations. Define height/surface identity,
prove occlusion and correct selection, and add appropriate tests plus a visible
demonstration. Full PRF-01 also needs elevated-path and underground-track overlap
proofs; a raised terrain scene alone does not complete it. Keep crowd navigation,
train circuit, save/reload determinism, and benchmark work as subsequent slices.

Run relevant tests, formatting, Clippy, a release build, render inspection, and normal
input checks where available. Document actual evidence and limitations, update the
checklist and milestone report, and create a reproducible Windows package. Do not
claim manual input, clean-machine compatibility, hosted CI success, or public release
without evidence. Do not choose a final name or project source license implicitly.
Keep the result reviewable and playable before starting another major subsystem.
```
