# Phase 12 Windows test distribution

Version 0.95.0 adds the installer and distribution checks. The owner chose to
defer the open-source license decision while this Windows test version is
completed. The public 1.0 acceptance still includes independent clean-PC,
hardware and player checks from the roadmap.

## Install and play

Run `Parkbound-0.95.0-setup-x64.exe`. Setup installs for the current user in
`%LOCALAPPDATA%\Programs\Parkbound`, creates a Start menu entry and offers an
optional desktop shortcut. It includes the game, runtime DLLs, fonts, art and
notices. Installation and play work offline from the downloaded files.

Start Parkbound and choose **Play Sunhaven Gardens** for the seven-ride demo,
or **New Park** for sandbox and campaign play. The first-park guide and Help
cover construction, access and coaster controls. The version is visible in
Credits and is available from `Parkbound.exe --version`.

The portable `Parkbound-0.95.0-windows-x64.zip` is also supplied. Extract it in a
writable folder and launch `Parkbound.exe`.

## Update, remove and recover

Close all Parkbound instances before setup or uninstall. A named Windows mutex
keeps setup from replacing a running game's files. Setup asks the player to
close the game rather than terminating its process. Run the newer setup in the
same program folder to update or repair it.

Parks, campaign progress, designs, scenarios and settings use the existing
`%LOCALAPPDATA%\Parkbound\saves` directory. Copy that entire folder with the game
closed to back up or move your data. Setup rejects this player-data directory as
a program destination. The uninstaller removes its recorded program files and
shortcuts, while player data and additional player-created files are retained.

Park saves remain schema/content 51/51. Accessibility settings remain version 1
and sound settings version 2. Existing loading, backups and legacy import flows
continue through the game.

## Build and check

Use Rust 1.93.1, Visual Studio C++ Build Tools with the Windows SDK, PowerShell 7
and Inno Setup 6.7.4. `scripts/install-inno-setup.ps1 -InstallDirectory ABSOLUTE_NEW_DIRECTORY`
downloads the pinned upstream compiler installer, verifies its published SHA256
and installs it for the current user. Alternatively install Inno Setup yourself
and pass its `ISCC.exe` path to the packaging script.

1. Run `scripts/package-windows.ps1` to create the versioned ZIP and manifest.
2. Run `scripts/package-installer.ps1 -PackageDirectory ABSOLUTE_PACKAGE_DIRECTORY -IsccPath ABSOLUTE_ISCC_PATH`.
3. Run `scripts/verify-installer.ps1 -Installer ABSOLUTE_SETUP_PATH -PackageDirectory ABSOLUTE_PACKAGE_DIRECTORY -OutputDirectory ABSOLUTE_NEW_EVIDENCE_DIRECTORY` in an account without an installed Parkbound.
4. Supply `-PreviousInstaller ABSOLUTE_PREVIOUS_SETUP_PATH` to exercise a real older-version upgrade. Without it, the initial installation uses the current setup and checks repair/reinstall behavior.

The installer check creates isolated program and player-data folders with spaces
and Unicode names. It checks installed bytes, Start menu targets, current-user
registration, running-game refusal, park-library and campaign continuation,
settings, repair, uninstall, preservation of extra player files and reinstallation.
It retains logs and saved-data hashes. Existing installed Parkbound registrations
are protected by running this check in a clean account.

`release.json` records package version, executable SHA256, save format and the
source checkout at packaging time. Adjacent `.sha256` files identify each ZIP and
installer. The installer build record also records compiler and package hashes.
Windows CI builds the installer from the tested ZIP and exercises its lifecycle.

## Current source and evidence

Branch `codex/phase-12-windows-distribution` starts at
`b84dd85ba2296715fc7d8cfcf3108be0d288d7f0`. The shared source checkout is
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
See `PHASE_12_DISTRIBUTION_VERIFICATION.md` for completed checks and remaining
acceptance. This development PC supplies correctness evidence.

Inno Setup's per-user mode, installer application mutex and stable AppId behavior
follow its upstream documentation. [Per-user mode](https://jrsoftware.org/is6help/topic_setup_privilegesrequired.htm),
[application mutex](https://jrsoftware.org/is6help/topic_setup_appmutex.htm),
[upgrade identity](https://jrsoftware.org/is6help/topic_setup_appid.htm).
