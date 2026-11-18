# Phase 12 distribution verification

The Windows test distribution is implemented on
`codex/phase-12-windows-distribution`, based on
`b84dd85ba2296715fc7d8cfcf3108be0d288d7f0`. Runtime implementation
`c55c0b7894771e58813277f8c2453ecb7b94fef5` is published in PR 87.

## Completed checks on PC1830

- Formatting and strict release Clippy passed across the workspace and all targets.
- The Windows release build and the running-instance mutex regression passed.
- The normal title screen was captured and inspected at 1024x680. It displays
  version 0.95.0 and the playable demo, campaign, sandbox and editor entry points.
- The ZIP passed 179 payload hashes and 157 bundled guide comparisons. The game
  version query agreed with the package version.
- The installer lifecycle passed 14 separate processes. These covered a prior
  0.94.0 installation, production park/campaign/settings writers, refusal while
  the game mutex exists, upgrade to 0.95.0, production continuation verifiers,
  repair, protected uninstall, uninstall, reinstall and final uninstall.
- All 21 park-library, campaign and preference files retained their exact bytes
  during upgrade, repair, uninstall and reinstall. A player-created file in the
  program directory also survived removal.
- Installed payload hashes, the current-user Installed apps version and the
  Start menu target/working directory matched the package.
- The test process was unelevated. Runtime processes launched from the Windows
  directory with only Windows system directories in PATH.

The previous-version installer fixture was built from the retained 0.94.0
Windows package using the current installer recipe. The test therefore upgraded
the real previous executable and data, while also testing the new install recipe.

The native information and destination pages were inspected. The subsequent
native interaction reported user input, so automated wizard navigation ended.
The visible installer continued installing the 0.95.0 test version in the
default per-user program directory. The automated lifecycle above used a separate
installation and retained its evidence independently.

## Artifact identities

Runtime executable SHA256 is
`8e49c101d14799d53433d160ae4d15d7dcf3675de139c80ad4a497d4a0fbe70c`.
The first fully exercised setup SHA256 is
`96b051da0553d3c2b0a3d7c6434f9c684ba8b35ea0057f0fd0815e83b4a2207a`.
The previous-version fixture setup SHA256 is
`7a92b5a427f5c5cd45ee193344cae91e933eb6de98ed200573dfac2af660cebb`.
The first verified ZIP SHA256 is
`b6967d75ddd23f45583536143738e644580b348431467d929612443bba5e399f`.

The final documentation refresh retains the same executable. Its ZIP and setup
each have an adjacent `.sha256` file, and the setup has a `.build.json` record
containing its compiler and payload identities. Local logs, title capture and
saved-data hashes are retained under `artifacts/phase12/`.

The Windows job in CI run 44294574454 passed all 1,038 tests and its complete
build/continuation checks. Its dependent installer job reached the final removal
after a successful reinstall, then exposed a test-script assumption. Inno Setup
can allocate `unins001.exe` while the previous uninstaller finishes cleaning up.
The verifier now reads the current registered uninstaller before each removal.
Corrected source `10f5215aeb39ddf460936c2b3e80dddc0d1da9a2` passed the full
Windows build/continuation job and installer lifecycle in run `44298610393`,
including final removal after reinstall. The starter endurance case passed;
three longer cases were still running at the closing checkpoint. See the
[GitHub run](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/44298610393).
Downloaded installer logs and build identities are retained under
`artifacts/phase12/ci-corrected-installer/`.

The final ZIP also passed production demo write/verify in separate processes.
All seven rides completed paid visits, facilities served guests and 18,000 ticks
continued exactly. Its title capture was inspected. The final installer rejected
a player-data destination while preserving the existing personal installation.

Independent clean-PC, representative hardware and player acceptance remain open.
The owner deferred the original source-code license decision for this Windows
test release.
