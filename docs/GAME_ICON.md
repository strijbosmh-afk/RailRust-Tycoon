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

The implementation is under verification. Final local build, test, package,
installation and Git publication results are recorded in the icon session
report and the current status/resume checkpoint. The 32- and 256-pixel frames
were visually inspected and transparent corner pixels were confirmed.

This presentation change keeps park saves at schema/content 51/51. The current
PC supplies correctness evidence. Representative-hardware performance remains
part of the separate release acceptance work.
