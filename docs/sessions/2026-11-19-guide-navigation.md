# First-park guide navigation fix

Project `railrust-tycoon`; session `01a15441-13d3-7142-b742-e896afad208c`;
PC `PC1830`; started 2026-11-19 13:15 UTC. Branch
`codex/right-click-settings`, baseline `f918250fde2c9ead94c34dd5d03bc07007c8119c`.

The owner reported that Next stopped working at 2/7. The code deliberately
ignored that control until the public-path count exceeded the starting count.
Later ride, access, trip and save tasks had similar silent gates.

Next now navigates all seven instructions without requiring practice first.
The guide separately reports what remains to practise, uses current tool and
navigation captions, and can target an existing ride for access in an existing
park. New connected path IDs replace net-count detection, including a build
plus demolition without a net increase. No simulation/save-format change is
needed; the park remains 52/52 and the guide's state is transient.

Regressions cover complete seven-step navigation in starter and demo parks,
unchanged saved park bytes, connected-path detection after removal, live
practice with actual rides/guests/saving, existing-ride access and modal guards.
Verification results and distribution identity will be recorded after checks.

Shared Git root:
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
The registration and remote are preserved. Local standard prompt 2.1.0 is used;
Drive remains disabled and existing exports are retained. This PC supplies
correctness checks only. Earlier native manual-opening acceptance remains
unverified after the user's Computer Use stop and is not part of this proof.


Formatting and strict release Clippy passed. All six guide regressions passed:
optional practice with real construction, paid trips and a successful save;
replay and real tool selection; all seven steps in starter/demo parks without
mutating saved park bytes; connected new path IDs after demolition; existing
ride access and explanatory labels; and foreground-modal navigation guards.
Windows release compilation and distribution checks are in progress.


## Completed validation and Windows checkpoint

Recorded 2026-11-19 13:27 UTC. Runtime commit:
`16292a10ce2bb7f1bb2467c2398078c93eb2a0db`.
All six focused guide tests passed. Formatting, strict workspace/all-target
release Clippy and the release build passed. The extracted package passed 204
payload hashes, 175 current guides and both fresh park-library processes.
No native mouse check or performance acceptance is claimed.

The installer upgraded PC1830 to 0.99.3. Its 204 payloads matched, all four
existing player files were preserved byte-for-byte, and the two shortcuts and
Installed apps icon were verified. Runtime SHA256:
`f3c847f4927fec5f5485f212db4f571eaa341a5866a16a0ae11453fb2006f2f7`.
The final guide refresh uses this same executable. Logs and final package
checksums remain under `artifacts/guide-*` and `dist` respectively.

No code work remains for this Next-button fix. The previous manual-opening
native retest remains independent and unverified after the prior user stop.
Git publication uses the existing branch/PR; verify its latest remote head.
