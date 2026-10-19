# Supplemental distribution notices

Some crates omit their workspace license files when published to crates.io.
These copies fill those omissions, and include the SIL Open Font License for
Bevy's embedded Fira Mono font. `sources.json` records the exact crate version,
its `.cargo_vcs_info.json` source commit, the download URL, and a SHA-256 hash for
every copied text. The packaging helper checks the source commit and file hashes
before including these notices. Packaging does not download license texts.

Most files come from the dependency's exact upstream commit. Two exceptions are
explicitly recorded in the manifest: `constgebra` declares `MIT OR Apache-2.0`
without supplying a license file, so its Apache-2.0 option is used with the
standard Apache license text; `hexf-parse` declares `CC0-1.0` without supplying a
license file, so the standard Creative Commons CC0 legal text is included.

When updating dependencies, run `scripts/package-windows.ps1`. If it reports
missing notices, inspect the new crate's source and original upstream license
files, add the required texts here, and record their exact versions, source
commits, URLs, and hashes in `sources.json`. Do not disable the missing-notice or
font-license checks. Original package license and copyright texts are preserved
in `THIRD_PARTY_NOTICES.txt`; `DEPENDENCIES.json` records the matching inventory.

`msvc/` contains the original Microsoft Visual C++ 2015-2022 Runtime license
document from Microsoft's license page and a text extraction for convenience.
These accompany any imported runtime DLLs that the package script copies,
unmodified, from the installed Visual Studio 2022 x64 redistributable directory.
The DLL list, versions, source locations, and hashes go in
`NATIVE_DEPENDENCIES.json`. Windows system DLLs are not copied.

- Runtime license: https://visualstudio.microsoft.com/license-terms/vs2022-cruntime/
- Original document: https://visualstudio.microsoft.com/wp-content/uploads/2021/09/Visual-C-Runtime-2015-2022-License-1.docx
- Distributable files: https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution

The package also includes the installed Rust toolchain's complete
`COPYRIGHT-library.html` under `RUST_STANDARD_LIBRARY_NOTICES.html` for its
statically linked standard library and supporting components.
