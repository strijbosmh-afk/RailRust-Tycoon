# Resume development on another Windows PC

This is the handoff guide for **RailRust-Tycoon**, the repository for an original Rust theme park simulation inspired by classic RollerCoaster Tycoon 1 and 2. The application currently displays **Parkbound**, a temporary working title; the repository name does not rename the application. Current milestone: overlap proof **0.3.0**, 20 October 2026, following foundation 0.1.0 and terrain 0.2.0. Read [the overlap report](docs/OVERLAP_REPORT.md) for actual verification; implementation and acceptance are tracked separately.

Canonical repository: <https://github.com/strijbosmh-afk/RailRust-Tycoon>.

The repository contains the source, locked dependencies, build scripts, notices, and development records needed to rebuild on another PC. Generated executables, ZIPs, screenshots, and Cargo build caches are deliberately ignored by Git. Continue from the synced workspace below or clone the repository, then rebuild as needed; copying a previous PC's `target` folder is unnecessary.

### Shared OneDrive working folder

The original PC's workspace was in the user's shared OneDrive folder. Its full local path, recorded on 20 October 2026, was:

```text
C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon
```

The path within that OneDrive root is `Documenten\ChatGPT\Railroad Tycoon`. On another PC, the Windows username or OneDrive root may differ; locate the synced folder and open it as the project workspace. The folder name remains `Railroad Tycoon`, while the GitHub repository is `RailRust-Tycoon` and the application's temporary title is `Parkbound`.

On the original PC, open the workspace in PowerShell with:

```powershell
Set-Location -LiteralPath 'C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon'
```

You can continue from that synced workspace or use a separate GitHub clone. The resumed checkout on the second PC is `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`; it is a separate local path, not evidence that OneDrive is syncing it. Verify Git state and the expected commit on whichever checkout you use. For a shared workspace, let OneDrive finish syncing and follow the two-PC workflow in section 9. The clone commands below are unnecessary when opening an existing checkout.

## 1. Read this first

The user confirmed:

- Rust implementation, classic RCT1/RCT2 isometric construction and park management.
- Windows only, with free distribution and all functionality included.
- Development in playable stages, eventually covering the agreed complete classic gameplay inventory.

Current defaults are offline single-player, an independent simulation library, original assets, four isometric orientations, and versioned local saves when persistence is implemented. Windows 11 x64 is the first build/test target, not a proven minimum requirement. Multiplayer, mobile/console ports, and scripting are outside the current plan. Expansion content and original-file import compatibility remain undecided.

**The first executable is a foundation, not a finished management game.** Do not infer completion from the visible park or the existence of a package.

Read the project records in this order:

1. [Overlap report](docs/OVERLAP_REPORT.md): current 0.3.0 implementation, verification, and remaining checks.
2. [Terrain report](docs/TERRAIN_REPORT.md): historical 0.2.0 evidence.
3. [Foundation report](docs/FOUNDATION_REPORT.md): historical 0.1.0 evidence.
4. [Product decisions](docs/PRODUCT_DECISIONS.md): confirmed direction, defaults, and open decisions.
5. [Gameplay checklist](docs/GAMEPLAY_CHECKLIST.md): stable acceptance IDs and their states.
6. [Development plan](DEVELOPMENT_PLAN.md): stages, architecture, and completion gates.
7. [Asset provenance](docs/ASSET_PROVENANCE.md): asset ownership and attribution.

This guide is a continuation aid. Update the checklist and milestone reports as evidence changes rather than letting this dated baseline override later results.

## 2. What exists today

| Area | Implementation for 0.3.0 |
| --- | --- |
| Park | Deterministic 32 × 32 site with grass, water, and sand. Starts in Terraced mode at heights 0/2/4/6; Flat mode remains available for comparison. |
| Overlap fixture | 21 stable objects: 12 X-axis path tiles at zero-based x8..19/y16, height 8; nine Y-axis track tiles at x13/y12..20, height 0. Each occupies two height steps. |
| Presentation | Original terrain, cliffs, path decks, and rails with shared depth and selection geometry. Surface mode hides buried track. Underground removes terrain and uses a non-selectable reference floor. No external game art is required. |
| Camera | Four rotations, keyboard and right/middle-drag panning, zoom, surface-height-aware fit, and grid toggle. |
| Inspection | Hover and selected-surface highlights with stable terrain/object identity; inspector distinguishes kind, object ID, tile, and height. Hidden surfaces cannot be picked through ordinary terrain. |
| UI | Header, sidebar, footer, pause/resume, 1x/2x/4x speeds, terrain preset button/T shortcut, Surface/Underground button/U shortcut, and help modal. |
| Simulation | Separate dependency-free Rust library with bounded terrain, stable non-reused object IDs, half-open clearance intervals, atomic placement/terrain validation, deterministic fixtures/presets, and clock. |
| Diagnostics | Overview, terrain selection, path, track, and help captures; selectable map profile, visibility, and rotation; minimum-window preset; PNG verification and exit status. |
| Delivery | Windows packaging, native-runtime inspection/copying, dependency/font notices, checksums, and a GitHub Actions workflow. |

**Not implemented:** terrain/path/track editors, slopes, player construction, path connectivity, guests, queues, ride operations, coaster design, staff, economy, scenarios, research, audio, persistent settings, park saves, or loading. The fixture's path and track are independently selectable proof objects, not usable attractions or guest routes. Preset switching preserves time, terrain materials, object identities, and object heights; Flat exposes the same track that Terraced buries. Each normal launch creates a fresh park. Trees, entrance arch, and underground reference floor are decorative.

The **0.3.0 overlap proof is verified (PRF-01)**. Source commit `0220c7b` passed [Windows CI run 39406042854](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39406042854): 51 tests, formatting, release Clippy with warnings denied, and Windows packaging. All 14 engine captures were inspected: Surface and Underground in four rotations, Flat in both views, and minimum-window path, track, hidden-selection, and help scenes. Every process exited with code 0. The relocated package ran from `C:\Windows` after extraction into a path with spaces and `é`; all 17 payload hashes matched. See the overlap report for the exact executable hash and detailed evidence.

Native input at 1280 × 800 selected Path #7 (display tile 15,17, height 8), revealed track with U, then selected Track #18 (14,18, height 0). The visibility button hid the track while retaining its ID and burial hint. Space paused at 02:14; Flat exposed the same track, all four rotations preserved identity/time, wheel zoom changed 100% → 113%, and Fit restored 100%. Hills hid the track again; clicking its old projection selected foreground terrain (12,19, height 6). Help/close preserved selection/time, and the native close button exited the process.

Historical **0.2.0** evidence includes 34 passing tests, formatting, Clippy with warnings denied, seven inspected engine captures, and a release Windows ZIP. Its 16 payload hashes matched after extraction into a separate path containing spaces and `é`; the extracted executable rendered selected raised terrain from an unrelated working directory and exited with code 0. See the terrain report. Clean-machine testing remains open; these results do not verify 0.3.0.

The historical **0.1.0** foundation report records 16 passing tests, release compilation, formatting, Clippy, three inspected images, and a relocated ZIP launch with matching payload hashes. Keep these results attached to their named checkpoint.

Normal desktop input was checked in 0.2.0 at 1280 × 800. Roof and visible cliff selection updated the inspector; T and the preset button preserved the selected tile while changing its elevation; rotation preserved selection; wheel zoom, Fit, and grid toggle worked. The timer advanced at 4x, Space paused it at 02:36, opening/closing help preserved that time and selection, Resume restored 4x, and the native close button ended the process. These observations plus authoritative clock tests verify FND-03.

Keyboard panning was not clearly established in the 0.3.0 native session, and right/middle dragging was unavailable through the desktop helper. Physical display scaling and multiple monitors remain untested, so FND-02 stays in progress. Clean-machine testing without development tools is also pending. These gaps do not invalidate the demonstrated PRF-01 render/pick-layer behavior; do not treat its completion as overall foundation or distribution acceptance.

### Checklist state at this handoff

| IDs | State | Meaning |
| --- | --- | --- |
| DEF-01, DEF-02 | Planned | Exact content/expansion audit and final product/platform decisions remain open. |
| FND-01 | Verified | Reverified in 0.3.0 through native launch/close, 14 inspected diagnostic captures, and relocated ZIP launch with all 17 payload hashes matching. |
| FND-02 | In progress | Native selection, four rotations, wheel zoom, and Fit passed in 0.3.0. Visible panning, right/middle dragging, and physical DPI/display checks remain open. |
| FND-03 | Verified | 0.2.0 speed/pause/resume evidence is retained; 0.3.0 inspector, visibility/preset buttons, pause, rotation, and help behavior passed with current simulation tests. |
| FND-04 | Verified | 0.3.0 source `0220c7b` passed 51 tests, formatting, release Clippy, and Windows packaging in CI 39406042854. |
| REL-01 | In progress | Packaging is implemented and a ZIP was checked; do not silently change its checklist state without reviewing its acceptance evidence. |
| PRF-01 | Verified | Current tests, all-rotation captures, native identity/visibility/preset checks, and package relocation establish correct terrain/path/underground rendering and picking. |
| PRF-02 through PRF-05 | Planned | Path connectivity/crowd recovery is next, followed by train operation, save continuation, and benchmark proofs. |
| Remaining gameplay/release entries | Planned | Consult the checklist for individual acceptance requirements. |

Stage 1 is not fully accepted until its remaining panning/dragging, display, and fresh-machine requirements pass. Stage 2 has completed PRF-01 and continues with PRF-02; the complete technical-proof stage is not finished.

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

On the current resumed host, pinned Rust and all requested components are installed under `%USERPROFILE%\.cargo\bin`; persistent PATH was deliberately left unchanged. Enable them for the current PowerShell process with:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
```

MSVC build tools and the Windows SDK are still absent on this host because the installer could not obtain UAC elevation. Install those prerequisites before local source builds, or use the hosted Windows workflow. The verified 0.3.0 runtime/package came from CI; local simulation type-checking and Clippy did not require the missing linker.

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
| Left click on map | Select the nearest visible terrain/path/track surface; inspector identity and highlight agree. |
| Right or middle drag | Pan without a jump when dragging begins. |
| WASD / arrow keys | Pan with the keyboard. |
| Wheel / + / - | Zoom while preserving usable selection behavior. |
| Q / E | Rotate through four views; selected simulation surface remains the same. |
| Home / Fit | Reset zoom and pan; preserve selection. |
| G | Toggle grid. |
| T / terrain preset button | Switch Flat ↔ Terraced while preserving time, materials, object IDs/heights, and selected identity. |
| U / visibility button | Switch Surface ↔ Underground; hidden terrain and the decorative reference floor cannot be selected underground. |
| Space | Pause; resume the previously chosen speed. |
| 1 / 2 / 3 | Choose 1x / 2x / 4x simulation speed. |
| H / F1 | Toggle help. |
| Escape | Close help, otherwise clear selection. |
| Window close / Alt+F4 | Exit. |

Also test toolbar buttons, help-close click consumption, selection beside UI borders, dragging after focus changes, minimum window size, and display scaling. In both terrain presets and visibility modes, rotate through all four views and inspect the shared crossing. Buried track must not be selected through terrain in Surface view; the nearest visible path/track surface must win underground. Select a track in Underground, return to Surface, and verify its hidden outline does not show through terrain. Switch to Flat and confirm the same ID becomes exposed without moving. Check that the underground floor and absent terrain are never selected. Pause and confirm the timer stops; help itself does not automatically pause normal play. The inspector displays one-based coordinates, while `TilePos` uses zero-based coordinates.

FND-03 retains recorded 0.2.0 evidence and passed the relevant 0.3.0 control regressions described above. Record remaining panning/dragging and display results, Windows/GPU details, and limitations before marking FND-02 verified. Repeat affected checks when controls or layout change.

## 5. Capture a rendered diagnostic frame

The app supports:

```text
parkbound.exe --capture-frame <absolute-output.png>
  [--capture-scene overview|selection|path|track|help]
  [--capture-map flat|terraced]
  [--capture-view surface|underground]
  [--capture-rotation 0|1|2|3]
  [--capture-small-window]
```

It installs diagnostic systems only when `--capture-frame` is present. The capture preset pauses simulation and suppresses keyboard, map-pointer, and UI-button actions and hover tinting so normal input cannot alter the view. `selection` chooses terrain; `path` and `track` choose fixture objects with stable IDs. Specify map, visibility, and rotation explicitly when comparing views. A selected buried track can intentionally remain invisible in Surface view. `--capture-small-window` requests 1024 × 680. A successful capture waits at least 90 frames and two seconds, checks the PNG, and exits with code 0; timeout is 120 seconds. This requires a rendering environment and is not ordinary-input evidence.

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

Repeat with `overview`, `selection`, `path`, and `help`; compare Flat/Terraced and Surface/Underground at rotations 0, 1, 2, and 3, including minimum-window captures. Inspect crossing occlusion, terrain/cliff alignment, exposed versus buried track, selection outlines, scenery, and controls. A valid PNG and successful exit do not prove visual correctness. Do not report another application's screenshot as game evidence. `artifacts` is ignored by Git; describe the exact evidence in the overlap report.

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

For version 0.3.0, expected output after successful packaging is:

```text
dist/
  Parkbound-0.3.0-windows-x64/
    Parkbound.exe
    ...runtime, notices, documentation, manifests...
  Parkbound-0.3.0-windows-x64.zip
  Parkbound-0.3.0-windows-x64.zip.sha256
```

Packaging derives the version from Cargo metadata. It collects the locked Windows dependency notices, the embedded font license, Rust standard-library notices, and required MSVC redistributable DLLs. It writes `DEPENDENCIES.json`, `NATIVE_DEPENDENCIES.json`, `THIRD_PARTY_NOTICES.txt`, and `SHA256SUMS.txt`. Existing extracted package directories are preserved with a `.previous-...` suffix; the current ZIP is replaced. Check `dist` occasionally for accumulated previous packages.

The native runtime helper uses Visual Studio 2022's `dumpbin` to inspect imported DLLs recursively and copies the required runtime files from its redistributable directory. The foundation package needed `VCRUNTIME140.dll`. Other Windows system/UCRT libraries come from the operating system. Do not copy arbitrary DLLs from a machine's system folder or assume import inspection proves clean-machine compatibility.

If packaging reports missing Rust standard-library notices, install `rust-docs` for the pinned toolchain. If it reports a missing Visual Studio redistributable directory or `dumpbin`, repair the required C++ installation. Supplementary license source/hash failures require review; do not bypass them.

Check the ZIP hash against its adjacent `.sha256` file:

```powershell
Get-FileHash -LiteralPath 'dist\Parkbound-0.3.0-windows-x64.zip' -Algorithm SHA256
Get-Content -LiteralPath 'dist\Parkbound-0.3.0-windows-x64.zip.sha256'
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
| `crates/park_game/src/main.rs` | Bevy app setup, resources, fixed update, camera layout, keyboard/pointer input, scene refresh, and input-state tests. |
| `crates/park_game/src/world.rs` | Terrain/object projection, shared render/pick/selection geometry, visibility, underground reference floor, and geometry tests. |
| `crates/park_game/src/ui.rs` | UI layout, actions, labels, help modal, button state, and refresh. |
| `crates/park_game/src/capture.rs` | Opt-in scene/map/visibility/rotation/window capture presets, screenshot receipt, PNG validation, and diagnostic exit. |
| `scripts/package-windows.ps1` | Release build/staging, metadata, package documentation, notices, archive, and hashes. |
| `scripts/write-third-party-notices.ps1` | Locked dependency traversal and license text collection. |
| `scripts/copy-windows-runtime.ps1` | Native import inspection, MSVC runtime copying, and native manifest. |
| `scripts/notices/` | Reviewed supplementary license texts, original license documents, and exact source/hash records. Keep these in Git. |
| `.github/workflows/windows.yml` | Windows checks, packaging, and temporary review artifact upload. |
| `docs/` | Product decisions, acceptance checklist, build evidence, and asset provenance. |

### Simulation boundary

`park_sim` has no Bevy dependency. `Park` owns dimensions, a row-major `Vec<Tile>`, a `u64` tick count, `Vec<SurfaceObject>`, and a monotonically increasing object-ID allocator. Dimensions are validated in `1..=256`; invalid map construction panics. Tile access is bounds checked. A `Tile` has a `Terrain` enum and `height: u8` in abstract steps; `MAX_TERRAIN_HEIGHT` is 32.

`Park::new` creates flat terrain without objects. Normal startup applies Terraced and installs the overlap fixture. `try_apply_terrain_profile` validates every proposed height before mutation; it preserves materials, time, IDs, and object heights. `apply_terrain_profile` remains a convenience wrapper that panics on a rejected profile without changing state; use the fallible API for player actions. `set_terrain_height` validates bounds, height, and object clearance atomically.

`SurfaceId` distinguishes `Terrain(TilePos)` from `Object(ObjectId)`. A `SurfaceObject` contains ID, tile, `Path`/`Track` kind, X/Y axis, height, and clearance. `place_object(pos, kind, axis, height, clearance)` returns a stable ID or `PlacementError`; `remove_object(id)` returns the removed object or `None`, and never recycles an ID. Occupied intervals are half-open, with positive clearance and total height at most 32. Shared-tile overlaps are rejected. Paths stay on/above terrain; track can also be wholly buried. Terrain cannot bisect an object.

Read-only APIs are `objects()`, `object(id)`, `surface_position(id)`, `surface_height(id)`, `is_underground(id)`, and `max_surface_height()`. Invalid/removed references resolve to `None`, or `false` for the burial query. Maximum surface height excludes the empty clearance above it. `install_overlap_fixture()` is atomic, requires a 32 × 32 map, and preserves already matching objects on repeat installation.

Terrain and fixtures use deterministic integer data. There is no seeded RNG state, gameplay command queue, navigation graph, guest system, train simulation, money, or serialization yet. The placement API is a clearance foundation, not a complete ownership/cost/connectivity construction command.

The app schedules `FixedUpdate` at 30 Hz and calls `Park::advance(speed)`. That currently increments time by 0, 1, 2, or 4 ticks with saturating arithmetic. **When real systems are added, each accelerated tick must execute the complete authoritative step in explicit order.** Merely increasing the clock counter would make the simulated behavior disagree with displayed speed.

The intended extension remains: validated atomic commands → fixed-step simulation → state/events → presentation. Store future money in integer minor units; use stable simulation/content IDs; preserve deterministic iteration and seeded randomness. Do not store Bevy entities or rendering caches as the long-term save representation.

### Presentation boundary

`Simulation(Park)` and `ViewState` are Bevy resources. `ViewState` owns presentation state and current speed: rotation, zoom, pan, grid, Surface/Underground visibility, `SurfaceId` hover/selection, help state, drag state, and flags. `park_sim` remains authoritative for park data. A separate `park_content` crate is planned only when authored content definitions justify it; it does not exist yet.

The Update systems are chained deliberately:

```text
UI button actions → keyboard input → apply pending terrain preset → pointer input
→ camera update → mesh update → UI refresh
```

Button actions clear and then set `ui_pointer_consumed` for that frame. Pointer handling checks it so a click that closes help cannot also select a tile behind the modal. Pending terrain presets apply before picking, so a frame does not inspect stale elevations. Preserve this ordering and its regression tests when refactoring UI.

`world.rs` uses 64 × 32 world-unit diamonds. `HEIGHT_STEP = 16` converts one simulation height step to presentation units. Geometry, depth, visibility, picking, and selection must agree across terrain and object layers. A cliff selects its terrain column; a path or track surface selects its own object ID. Ordinary picking excludes buried objects. Underground mode removes terrain from rendering/picking; its reference floor is non-selectable. Selection must obey scene occlusion rather than being forced above it. The legacy terrain-only picking wrapper remains for baseline tests. General map sizes require coordinated changes to square-map presentation assumptions, not only a larger `Park`.

The camera and pointer transform use logical window pixels consistently. Mouse dragging uses successive logical cursor positions; replacing that with raw mouse-motion counts can reintroduce DPI-dependent behavior. UI exclusion bounds and camera offsets depend on `SIDEBAR`, `HEADER`, and `FOOTER`; change the layout and these boundaries together.

The scene is one procedural mesh, rebuilt when the view marks it dirty. Preset and visibility changes must invalidate scene and selection geometry even if selected identity stays unchanged. Selection is a separate mesh. Both currently use `NoFrustumCulling`: replacing vertices without updating a cached bounding box can hide geometry incorrectly. Retain that treatment or correctly refresh bounds during refactors. Dynamic entities and large parks still need a measured update strategy; the monolithic mesh remains a small proof implementation.

Zoom is bounded to 0.6–3.5; pan is bounded; Fit resets zoom/pan while retaining selection and accounts for terrain/object height. Keep the same camera layout calculation in rendering and pointer conversion. Rendering uses opaque white materials with vertex colors. Release builds use the Windows GUI subsystem, so opening the executable does not supply a normal console log window. Use capture exit status and the relevant debugger/log strategy when diagnosing a silent launch failure.

## 8. Resume work in this sequence

### A. Reproduce and close the foundation gaps

Build from the existing or newly cloned repository using the pinned toolchain; record the commit, Windows version, graphics driver, and check results. Complete ordinary input checks and clean-machine package checks where the new PC makes that possible. Update individual IDs only after their acceptance criteria pass. Keep the scope/catalogue audit visible rather than treating it as finished because coding has started.

### B. Preserve the accepted PRF-01 overlap baseline

The 0.3.0 overlap fixture, stable identity, visibility, and clearance proof is accepted. Retain it as a regression scene while developing navigation. Its current evidence is recorded in the overlap report; rerun the affected checks when changing these systems.

1. Run simulation, geometry, input, formatting, lint, and release checks on the exact current source; inspect hosted CI for that commit.
2. Demonstrate the shared crossing in all four rotations and both visibility modes, including Flat/Terraced comparison, zoom, pan, UI exclusion, and boundaries.
3. Verify hidden track and hidden selection do not show through terrain; check the underground reference floor cannot be selected.
4. Verify IDs and object heights survive view/preset changes; rejected clearance and removed references remain safe.
5. Record normal mouse/keyboard behavior and outstanding physical display or hardware limitations separately from synthetic tests; FND-02 remains open.
6. Package and inspect each new checkpoint, recording hashes and relocation results. Prior-build evidence does not automatically verify changed code.

Shared clearance and surface rules should become the foundation for paths, track, scenery, and construction. Reject illegal placement atomically once commands are introduced.

### C. Prove paths and crowd recovery: PRF-02

This is the next implementation slice. Reuse PRF-01's stable IDs and clearance to introduce height-aware path connectivity, destination selection, and a small seeded guest simulation in `park_sim`. First make accelerated speed execute every fixed simulation step. Start with a tiny deterministic test park; test junctions, disconnected regions, and deletion of occupied/destination paths. Guests must enter a defined lost/recovery state rather than retain invalid references. Render and compare display frame rates before optimizing crowd size. Queue rules belong in connectivity semantics even before detailed boarding.

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
git switch -c codex/overlapping-surfaces
```

If continuing an existing branch on the second PC, use that branch instead of creating a parallel branch with the same intent:

```powershell
git switch --track origin/codex/overlapping-surfaces
```

If it already exists locally, use `git switch codex/overlapping-surfaces`, then `git pull --ff-only`. When a fast-forward fails, inspect the histories and resolve intentionally; do not force-push or reset away the other PC's changes.

Before leaving a PC, run relevant checks, inspect the diff, explicitly stage the intended source/docs, commit, and push the branch:

```powershell
git diff --check
git diff --stat
git status --short
# Stage the specific files changed for this task, then inspect what will be saved.
git diff --cached --stat
git diff --cached
git commit -m "Add overlapping path and underground surface proof"
git push -u origin codex/overlapping-surfaces
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
Read RESUME_DEVELOPMENT.md, docs/OVERLAP_REPORT.md, docs/TERRAIN_REPORT.md, docs/FOUNDATION_REPORT.md,
docs/PRODUCT_DECISIONS.md, docs/GAMEPLAY_CHECKLIST.md, and DEVELOPMENT_PLAN.md,
then inspect the actual source and current Git state. Preserve unrelated local
changes and remote history. The shared OneDrive path and single-active-session
workflow are recorded in the guide; continue there or use a separate clone.

The confirmed product is a free, fully featured, Windows-only classic RCT1/RCT2-style
isometric park simulation in Rust, developed in stages with original assets. We have
a verified version 0.3.0 overlap proof, not a finished game. It retains Flat/Terraced
terrain and adds 12 elevated path objects at height 8 and nine track objects at
height 0, all clearance 2. Stable IDs, shared clearance, and U-controlled visibility
connect simulation, rendering, picking, and inspection. Flat exposes the same track
without moving it. Underground removes terrain and uses a non-selectable floor.
No path/terrain editor, guests, operating rides, finances, saves, or audio exist.
PRF-01 is verified: runtime source 0220c7b passed Windows CI 39406042854 with 51 tests,
render inspection, native overlap controls, and package relocation. Read the report
for exact evidence. FND-02 panning/dragging/physical-display checks and clean-machine
compatibility remain open unless later reports explicitly close them.

Use the repository-pinned Rust 1.93.1 MSVC toolchain and Bevy 0.17.3 with Cargo.lock.
On an 8 GB machine use -j 2. First reproduce the baseline checks and inspect CI for
the current commit. Keep park_sim independent of Bevy. Read the input-ordering,
logical-pixel, mesh-culling, and accelerated-tick notes before refactoring them.

The next implementation is PRF-02: height-aware paths and a tiny seeded crowd that
recovers safely from deletions. Preserve the PRF-01 scene and its regressions across
all four rotations, both presets, and both visibility modes. Before moving entities,
make accelerated time run each complete authoritative tick. Keep train operation,
saves, and benchmarks as later independent proofs. Continue the exact content/
expansion audit in parallel and complete available foundation checks separately.

Run relevant tests, formatting, Clippy, a release build, render inspection, and normal
input checks where available. Document actual evidence and limitations, update the
checklist and milestone report, and create a reproducible Windows package. Do not
claim manual input, clean-machine compatibility, hosted CI success, or public release
without evidence. Do not choose a final name or project source license implicitly.
Keep the result reviewable and playable before starting another major subsystem.
```
