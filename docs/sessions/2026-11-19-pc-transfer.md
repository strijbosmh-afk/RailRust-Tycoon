# Complete PC transfer package

Project `railrust-tycoon`; session `01a15441-13d3-7142-b742-e896afad208c`;
PC `PC1830`; 2026-11-19 15:15 UTC; branch `codex/right-click-settings`.
Baseline: `e2b86b6965fbcf43da9346ea5e79b67c67f3dd33`.
The containing commit records the transfer instructions. The exact source
commit used to create the handoff is in `TRANSFER_MANIFEST.json`.

The owner requested one folder containing all code, the installer and Markdown
files for another PC. The handoff destination is
`dist/Parkbound-Transfer-2026-11-19`, with a companion ZIP. It contains:

- Complete committed source, original assets, Cargo.lock, pinned toolchain,
  build scripts and all tracked technical/player Markdown documentation.
- The verified 0.99.3 Windows installer and portable ZIP, with their original
  checksums/build receipts. No gameplay rebuild is required.
- An offline Git bundle for the current branch and its reachable history,
  without copying the active .git directory or authentication.
- A separate copy of the complete current Parkbound player-data folder, with
  restoration instructions that preserve the destination's existing data.
- A start guide, transfer manifest, file hashes and a read-only verification
  script. A copied/extracted transfer should be verified before use.

Source blob hashes, bundle restore/commit identity, distributable hashes and
copied player files are checked during creation. The ZIP is read back and its
entries verified. Exact results and the final source identity are retained in
the handoff's Verification directory and local `artifacts/pc-transfer-*`.

GitHub was rechecked during this session. Baseline run `44536241835` is now
successful: windows, windows_installer and all four beta_endurance jobs passed.
The previous pending-CI note is resolved. These are correctness results, not a
performance endorsement of PC1830. Runtime remains 0.99.3, schema/content 52/52.

The installation guide now distinguishes optional per-PC Codex helper setup
from playing/building the game and explicitly keeps Drive disabled. No
credentials, whole-machine helper configuration, build caches, active Git
metadata or unrelated project files are included. No manual archive hook was
run. This handoff does not claim installation or native testing on another PC.

Shared root:
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
Local standard prompt 2.1.0 was used; the project registration and origin were
preserved. Commit/push only these documentation changes to the existing branch
and verify its remote head. Next: copy the transfer folder or ZIP, run
Verify-Transfer.ps1, then follow START_HERE.md on the new Windows PC.
