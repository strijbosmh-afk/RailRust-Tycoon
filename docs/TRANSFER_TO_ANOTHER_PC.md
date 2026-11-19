# Transfer Parkbound to another Windows PC

The transfer folder contains the complete committed source, a verified Windows
installer, the portable game ZIP, project documentation, an offline Git bundle
and a separate copy of your current player data. Copy the entire folder, or
copy its companion ZIP and extract it on the other PC.

## What to open first

| Item | Purpose |
| --- | --- |
| `START_HERE.md` | Short instructions for the specific transfer |
| `Installer/` | Install Parkbound for your Windows account |
| `Portable/` | Extract and play without installing |
| `Source/` | Complete source snapshot, assets, Cargo lockfile and all tracked Markdown files |
| `History/Parkbound.git.bundle` | Restore the same branch and its Git history without downloading the repository |
| `PlayerData/Parkbound/` | Separate backup of the original PC's player-data folder |
| `Verification/` | CI and packaging records |
| `TRANSFER_MANIFEST.json` and `SHA256SUMS.txt` | Source identity, inventory and file-integrity records |

The source snapshot and Git bundle include the latest handoff documentation.
The installer and portable ZIP retain the tested 0.99.3 executable; their
packaging commit is recorded separately from the source commit. Nothing needs
to be compiled to play. Build caches and the active `.git` directory are not
copied; the bundle is the portable Git-history copy.

## Play on the other PC

1. Copy or extract the whole transfer folder to a local, writable directory.
2. Open `Installer` and run `Parkbound-0.99.3-setup-x64.exe`. Alternatively,
   extract the ZIP in `Portable` and run the extracted `Parkbound.exe`.
3. The default installed location is `%LOCALAPPDATA%\Programs\Parkbound`.
4. Start a new park or choose the built-in Sunhaven Gardens demonstration.
5. Press H or F1, then the document icon, to open the English player manual.

Player data normally lives in `%LOCALAPPDATA%\Parkbound`. To continue the parks
from this transfer, close the game on both PCs. Back up any existing player
folder on the destination first, then copy `PlayerData/Parkbound` to that
location. Keep its complete `saves` directory together: campaign progress,
park slots and the remaining Tips allowance belong to the saved state. Do not
combine individual save files from different versions of the same park.

## Resume development, including offline Git restoration

Install Git, Rust 1.93.1 (MSVC), Visual Studio C++ Build Tools with a Windows SDK,
and PowerShell 7. Python 3.10+ is needed to regenerate/check the player manual;
Inno Setup 6.7.4 is needed only to rebuild the installer. See `INSTALLATIE.md`.
The first Rust build needs internet access to download its locked dependencies.

The `Source` folder can be used as a complete source snapshot. To retain Git
history, clone the included bundle into a **new** development folder instead:

```powershell
# Run from the extracted transfer folder. Choose an unused destination.
git clone --branch codex/right-click-settings ./History/Parkbound.git.bundle C:/Dev/Parkbound
git -C C:/Dev/Parkbound remote set-url origin https://github.com/strijbosmh-afk/RailRust-Tycoon.git
git -C C:/Dev/Parkbound rev-parse HEAD
```

Compare the printed commit with `sourceCommit` in `TRANSFER_MANIFEST.json`.
The remote replacement above applies only to this newly restored bundle clone;
preserve the remote of any existing project checkout.

Open that Git root in Codex or your editor. Read `AGENTS.md`, `PROJECT.json`,
`STATUS.md` and `RESUME_DEVELOPMENT.md` before making changes. Then run:

```powershell
Set-Location C:/Dev/Parkbound
cargo fmt --all -- --check
python scripts/build-player-manual.py --check
cargo clippy --release --workspace --all-targets --locked -- -D warnings
cargo test --release --workspace --locked
cargo run --release --locked --package park_game --bin parkbound
```

The original shared Git root is
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
The destination PC may use a different path. Its optional Codex project-helper
installation and registration are separate from the game; do not copy login
credentials, machine configuration or hook trust. Google Drive synchronization
is disabled in this project's `PROJECT.json` and must stay disabled unless the
owner explicitly re-enables it. This transfer does not install tools on the
other PC and does not prove that the game has run on that hardware.

## Verify the transfer

The producer verifies every source file against its Git blob, clones the Git
bundle into a separate temporary checkout, and checks the installer/portable
ZIP against their recorded SHA256 hashes. `SHA256SUMS.txt` covers all transfer
payload files, including the separate player-data backup.

Run `Verify-Transfer.ps1` from PowerShell 7 after copying or extracting the
folder. It checks all hashes and rejects missing or additional files without
changing your source, installation or saves. Keep the original transfer intact;
make your development clone and unpack the playable ZIP into other folders.

The runtime source's GitHub CI run `44536241835` passed all six jobs: Windows
checks/package, installer lifecycle, and four endurance scenarios. These are
correctness results; PC1830 is not a product performance reference. The transfer
manifest distinguishes that verified commit from later documentation commits.
