# Parkbound 0.95.0 Windows test release

This release adds a per-user Windows installer, Start menu entries, an optional
desktop shortcut, program removal, package identity metadata and distribution
checks. Setup supports English and Dutch. The portable ZIP remains available.

Close running Parkbound instances before updating or uninstalling. Setup detects
the running game and asks you to close it. Choose the same program folder to
update. Player data is stored separately and survives an update, repair,
reinstallation or uninstall.

The seven-ride Sunhaven Gardens demo, original campaign, sandbox and editors are
included. The preceding graphics updates add detailed ride bodies, stone paths,
timber bridges, terrain materials and varied planting.

Park saves retain schema/content 51/51. Accessibility preferences retain version
1 and sound settings retain version 2. Copy your complete player-data folder with
the game closed when moving computers.

## Test coverage and release status

The installer targets Windows 10/11 x64. The exact tested installation and
upgrade results are in `docs/PHASE_12_DISTRIBUTION_VERIFICATION.md`. Independent
clean-PC and representative-hardware acceptance remain to be completed. This
development PC supplies correctness checks, not performance targets.

The Windows binaries are unsigned. The owner has deferred the open-source license
decision. This is the Windows test channel, version 0.95.0. See `RIGHTS.md` and
`PRIVACY.md` for distribution and data-handling information.
