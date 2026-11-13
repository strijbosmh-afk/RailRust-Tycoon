# Codex Cloud project handoff

## Source and identity

Project `railrust-tycoon` is Parkbound, an original Rust isometric park game for Windows x64. The repository is `https://github.com/strijbosmh-afk/RailRust-Tycoon.git`. Continue from the existing branch `codex/phase-9-campaign-editors` and verify the remote commit before editing. Preserve the Git history, project ID and Drive IDs in `PROJECT.json`.

The original local chat is `01a15441-13d3-7142-b742-e896afad208c`. Its shared checkout is `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon` on PC1830. Register the actual cloud path separately if supported by the cloud environment.

Read `AGENTS.md`, `STATUS.md`, `INSTALLATIE.md`, `BESLISSINGEN.md`, the latest section of `RESUME_DEVELOPMENT.md` and `docs/PHASE_9_CAMPAIGN_AND_EDITORS.md` before continuing. Earlier resume sections describe historical builds.

## Phase 9 scope

The campaign contains Meadow, Boardwalk, Cedar Valley, Research Ridge, Lakeside Services and Grand Festival. Sandbox starts select funds, research availability, terrain, visitors, loans and charging rules. The scenario editor authors map content, available rides/facilities, rules and objectives, with independent test play and versioned `.parkscenario.json` exchange. Park saves use schema/content 51/51, and campaign profiles use version 2 with version-1 migration.

Phase 9 runtime source is `9f5f4e39ce96373d25ac1ffc5023c2d33ba3121e`. Windows CI run `43452409591` passes all 67 steps and 1,012 tests. Its build artifact is `14052059495`, with wrapper SHA256 `79427c8c43db9cfe03a0d7a0ac0923da856e5232154aaea2ca5e1d0610b52a3d`. The Windows executable SHA256 is `87bd947ed46d16fa978bd87f7e32e48083a4b499416097b4db9e314b96dc800b`.

The full local package is `dist/Parkbound-0.88.0-windows-x64.zip`, with an adjacent `.sha256` file. The registered private Drive Momentopnamen folder also holds manual source and Windows-build handoff archives after upload. Find the versioned files under that verified parent and compare their downloaded bytes with the recorded receipt in `artifacts/phase9-handoff-receipts.json`. GitHub artifacts expire after 14 days. Preserve the complete Windows ZIP in the cloud workspace and verify its bytes before reporting the transfer complete.

## Cloud launch status

The user explicitly approved creating a new Codex Cloud chat on 2026-11-13. The desktop cloud-creation request stalled without returning a task ID and was stopped. The authenticated `codex cloud list --json` returned no tasks, and `codex cloud exec` reported `no cloud environments are available for this workspace`. The app's environment listing advertised a default configuration, but that launch did not complete. The user has been asked to connect a cloud environment for this repository.

After the environment is available, list the current advertised environments and use the matching runnable configuration. The requested transfer includes the complete tracked source, original assets, technical guides and full Windows distribution. Verify both the checked-out commit and the downloaded package. Keep the current local project and GitHub branch available throughout the handoff.

## Verification and future work

Use the pinned Rust version in `rust-toolchain.toml` and the committed lockfile. Standard checks are `cargo fmt --all -- --check`, `cargo clippy --release --workspace --all-targets --locked -- -D warnings` and `cargo test --release --workspace --locked`. Windows packaging runs through `scripts/package-windows.ps1`. The Windows GitHub workflow exercises the production executable and continuation proofs.

Keep platform-independent simulation checks available in cloud environments. The distributed executable is a Windows program and native presentation acceptance uses Windows. Record the actual environment and checks performed. Timings from PC1830 are excluded from performance references.

Phase 10 covers sound, onboarding, controls and accessibility. Start it only when the user requests further development. During this transfer, preserve the published source and verify the downloaded build.

## Storage

GitHub holds the complete tracked source, assets and technical guides. Windows package files are build artifacts. The private Softwareprojecten Drive registration belongs to `strijbosmh@gmail.com`; verify that account, ownership and destination parents before any Drive uploads. A new cloud environment has its own installation and export capabilities. Preserve the distinction between repository transfer, verified package transfer and native chat-export evidence.
