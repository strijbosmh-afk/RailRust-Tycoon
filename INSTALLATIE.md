# Installation and development

## Windows installer in 0.96.0

Run `Parkbound-0.96.0-setup-x64.exe` to install for your Windows account, or extract the portable ZIP. The default program folder is `%LOCALAPPDATA%\Programs\Parkbound`. Player data uses `%LOCALAPPDATA%\Parkbound\saves`. Close the game before updating. Setup and uninstall preserve player data. The package runs with the included runtime DLLs and assets.

To build the installer, use PowerShell 7 and Inno Setup 6.7.4 alongside the existing Rust 1.93.1 and Visual Studio C++ tools. Run `scripts/package-windows.ps1`, then `scripts/package-installer.ps1 -PackageDirectory ABSOLUTE_PACKAGE_DIRECTORY -IsccPath ABSOLUTE_ISCC_PATH`. The compiler bootstrap helper is `scripts/install-inno-setup.ps1 -InstallDirectory ABSOLUTE_NEW_DIRECTORY`. It verifies the pinned upstream download before installing the build tool for the current user.

The installer lifecycle and previous-version upgrade commands are documented in `docs/PHASE_12_WINDOWS_DISTRIBUTION.md`. The owner has deferred the original source-code license decision for this Windows test release.

Version 0.94.0 uses the same Rust toolchain, dependencies and embedded assets as 0.93.0. Extract the verified `Parkbound-0.94.0-windows-x64.zip` and run `Parkbound.exe`. Existing parks automatically use the updated landscape materials.

Version 0.93.0 adds the ride graphics refinement. It uses the same embedded assets, dependencies and installation requirements as 0.92.0. Extract `dist/Parkbound-0.93.0-windows-x64.zip` when its verification is complete, then run `Parkbound.exe`.


## Play on Windows

Version 0.92.0 embeds its title artwork, oak texture and interface fonts. The package includes the source art and font licenses. It supports offline launch from an extracted writable folder.

Extract the latest verified `dist/Parkbound-*-windows-x64.zip` to a writable directory and run `Parkbound.exe`. The package includes assets, required runtime files and player documentation. See `README.md` for controls and save locations.

Version 0.91.0 includes **Play Sunhaven Gardens · 7 rides** on the title screen. The playable demo is built into the executable and needs no separate download or installation. Save your visit from the park menu to continue it later.

## Resume development on another PC

Clone the repository specified in `PROJECT.json`, then check out the branch and commit recorded in `STATUS.md` and `RESUME_DEVELOPMENT.md`. Install Git, Rust 1.93.1 and the Microsoft Visual C++ build tools with Windows SDK. Use the committed lockfile.

The first source build downloads the locked dependencies from crates.io. The extracted Windows distribution supports offline play.

Run `cargo fmt --all -- --check`, `cargo clippy --release --workspace --all-targets --locked -- -D warnings` and `cargo test --release --workspace --locked`. Build the Windows package with `scripts/package-windows.ps1`.

For project registration on that PC, read the current Softwareprojecten standard prompt and local `ONBOARDING.md`, verify the existing GitHub and private Drive destinations, then use `node onboarding.mjs --project PATH --apply`. The helper installation lives in `%USERPROFILE%\.codex\softwareprojecten`. Review native hook trust on that PC. Automatic export is established only after its native Stop event, local export and accepted Drive readback receipt.

The current shared OneDrive path is recorded in `RESUME_DEVELOPMENT.md`. Register the actual Git root on each PC rather than assuming an identical drive path.

## Phase 10 preferences

Version 0.89.0 adds `accessibility-settings.json` and sound-settings version 2 in the selected save directory. Older sound preferences migrate when read and are rewritten when the user changes a control. The dependency/toolchain requirements are unchanged. Run `Parkbound.exe --accessibility-check ABSOLUTE_NEW_DIRECTORY` and then `--verify-accessibility SAME_DIRECTORY` for independent-process preference verification.

## Phase 11 beta tools

Version 0.90.0 includes `verify-beta.ps1` beside the packaged executable. PowerShell 7 runs the supplied script without Rust. A source checkout uses `scripts/verify-beta.ps1` with the release build. Choose a new absolute output folder. See `docs/PHASE_11_BETA_GUIDE.md` for cases, run lengths and separate rendering measurements. The Rust toolchain and dependency versions are unchanged.
