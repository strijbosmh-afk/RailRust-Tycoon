# Installation and development

## Play on Windows

Extract the latest verified `dist/Parkbound-*-windows-x64.zip` to a writable directory and run `Parkbound.exe`. The package includes assets, required runtime files and player documentation. See `README.md` for controls and save locations.

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
