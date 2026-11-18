# Parkbound application icon

Version 0.96.1 uses the user-approved original red coaster, golden rails and
forest-green badge. The master is `assets/art/parkbound-icon.png`, a transparent
1254×1254 RGBA image. The generation mode, exact prompt and provenance are in
`assets/art/PROVENANCE.md`. Both asset hashes are in `assets/manifest.json`.

## Windows integration

`scripts/make-icon.ps1` builds `assets/art/parkbound.ico` with 16, 24, 32, 48,
64, 128 and 256 pixel frames. Each frame retains 32-bit alpha. The conversion
uses Windows System.Drawing and preserves the PNG master.

`crates/park_game/build.rs` embeds that ICO as resource 1 and adds the package
version and Parkbound product name to the executable. Its locked build
dependency is `embed-resource` 3.0.12. The existing Windows SDK supplies the
resource compiler. Runtime code in `app_icon.rs` applies large and small icons
on the native window's main thread, selecting sizes using its initial DPI.

Setup and uninstall use the same artwork. Start menu and optional desktop
shortcuts refer to the installed `assets/art/parkbound.ico`. Both icon files
are included by the existing recursive artwork packaging step. The Inno Setup
directive embeds the setup icon into the uninstaller as well. [Inno Setup icon documentation](https://jrsoftware.org/inno-setup/documentation/topic_setup_setupiconfile.htm)

## Rebuild and verify

Run from the registered repository in Windows PowerShell 7.

```powershell
./scripts/make-icon.ps1
./scripts/validate-assets.ps1
cargo build --release --locked --package park_game --bin parkbound
./scripts/verify-windows-icon.ps1 -Executable ./target/release/parkbound.exe -CheckWindow
./scripts/package-windows.ps1 -SkipBuild
./scripts/package-installer.ps1 -PackageDirectory ./dist/Parkbound-0.96.1-windows-x64
```

If artwork is deliberately changed, update its manifest hashes after
regenerating the ICO. The icon verifier reads all seven RT_ICON payloads from
the actual executable and compares their bytes with the source ICO. It also
checks executable identity. With `-CheckWindow`, it starts an isolated game
profile, reads both native window icon handles and closes its test process.
Results are written under `artifacts/icon-verification` by default.

Windows CI runs the embedded-resource check after packaging. The existing
installer lifecycle check also verifies the Start menu's icon location when
the package contains the ICO.

## Verification checkpoint

Runtime source `c18174696b34460eb48db76d4f015c5876a1c483` is published in
PR 89. All 574 game tests and 464 simulation tests passed, with formatting,
strict release Clippy and the release build. The ICO regenerates byte-for-byte
on PC1830. Asset validation covers 19 families. The 32- and 256-pixel frames
were visually inspected, with transparent corner pixels confirmed.

All seven embedded resource payloads match the ICO. Both large and small
native window icons were verified on the release build and installed game.
The extracted ZIP passed 185 payload hashes and 161 bundled-guide checks,
independent demo/front save continuation and an inspected title capture.

The existing 0.96.0 installation was upgraded to 0.96.1. Its 185 payload files
match the package, and both existing player files retain their hashes.
Start menu and desktop shortcuts and Installed apps reference the new ICO.
Icons extracted from the game, setup and uninstaller are pixel-identical.
Evidence is under `artifacts/icon-*`. The executable SHA256 is
`4cb731db5641fb677be3fb6781863d577fb4d87ad306d61ee70df60645c9052c`.
Current ZIP and installer hashes accompany those files in `dist/`.

At 2026-11-18 14:48 UTC, the remote workflow had passed workspace tests,
packaging and the embedded-icon check and was continuing integration checks.
The first lint run caught the integer-resource pointer spelling. The corrected
implementation passed strict Clippy. [Windows verification run](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/44325579555)

This presentation change keeps park saves at schema/content 51/51. The current
PC supplies correctness evidence. Representative-hardware performance remains
part of the separate release acceptance work.
