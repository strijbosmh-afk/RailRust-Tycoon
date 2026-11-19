# Right-click individual settings and Dutch player manual

Project `railrust-tycoon`, session `01a15441-13d3-7142-b742-e896afad208c`,
PC `PC1830`, 2026-11-19 UTC. Branch `codex/right-click-settings`, based on
`3d08981d99f8f3373941a2ffe72deaeaf2e66bd6`.

The owner requested access to individual ride and service settings through a
right-click. A short click opens the existing window for the visible object's
owner. Dragging keeps panning. Track, gates and owned queues resolve to their
ride. Modal, focus, cursor, camera and park-generation changes cancel pending
clicks. The gesture never changes park data or Tips usage.

The source root is `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. Git identity and the configured GitHub
repository were verified. Local standard prompt 2.1.0 is in use, Google Drive
remains disabled and existing queued exports are retained. No new dependencies
or save-version changes are introduced; saves remain 52/52.

## Verification

All 599 application tests and strict workspace Clippy passed. The seven new
regressions cover owner identity, rotations, DPI/UI scaling, no-op ground clicks,
right/middle panning, jitter, cancellation and unchanged save bytes/Tips.
The Windows release build and embedded-icon verification passed. The extracted
package passed 200 payload hashes and 172 current guides. All six fresh-process
operations, Tips and park-library writer/verifier runs passed. This PC supplies
correctness evidence only.

## Dutch player manual

The owner also requested a very detailed, stepwise and navigable manual.
`docs/HANDLEIDING_NL.md` contains 25 chapters and about 9,500 words covering
first play, menus, camera, construction, every catalogue model and service,
individual management, needs, finance, staff, research, advertising, all six
campaign scenarios, editors, saves, Tips, shortcuts and troubleshooting.
The generated offline HTML has 80 heading targets, 120 local links, 14 tables,
search, print layout and no external assets. Help exposes a document icon with
the full hover/focus label. Opening it leaves the simulation at its chosen speed.

The Python standard-library generator and CI freshness check validate all
chapter links. The package verifier checks the HTML against the source copy.
HTML parsing and JavaScript syntax checks passed. The file-location regression
checks paths with spaces, the packaged-file preference and source fallback.

Native interaction testing displayed the actual 0.99.1 title screen in an
isolated muted profile and selected the demo. The user then stopped Computer
Use with physical Escape. No further Computer Use actions were sent. Native
right-click and browser-opening acceptance are therefore not claimed.


## Final runtime and local installation checkpoint

Recorded 2026-11-19 12:37 UTC. Runtime/manual commit:
`38179d701e229ab38fe4d844c026c7c06b04611e` on
`codex/right-click-settings`. The final documentation commit follows it.
Executable SHA256:
`2bff66774ed77e6b542762fbbdcc2455ddddc6ef1e9d8c6e14d36e5b2f275ffd`.

The 0.99.1 installer updated PC1830 at
`C:\Users\Michiel Strijbos\AppData\Local\Programs\Parkbound`.
All 200 installed payload hashes matched, all four existing player files were
unchanged, and both shortcuts and the Installed apps icon were checked.
The final guide-only refresh uses the same executable. Its ZIP/installer
checksums are recorded alongside the distribution files, avoiding recursive
checksums inside their own documentation. The package verification scripts
compare every bundled guide with the checkout.

Remaining acceptance: native right-click on a ride and a service, plus opening
Help's manual in the default browser. Computer Use was stopped by the user;
these observations are not claimed. Automated regressions cover the gesture,
owner identity and unchanged save state. There is no remaining code work in
this request. Google Drive upload stays disabled; local exports are retained.
