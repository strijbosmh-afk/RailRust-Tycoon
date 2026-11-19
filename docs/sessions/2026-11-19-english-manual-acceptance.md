# English manual and native interaction acceptance

Project: `railrust-tycoon`. Session: `01a15441-13d3-7142-b742-e896afad208c`.
PC: `PC1830`. Started 2026-11-19 12:41 UTC. Branch:
`codex/right-click-settings`; starting commit
`0cf1718696e7a1eb09edc0e8a324b19888872535`, draft PR #93.

The owner requested completion of native right-click/manual-opening checks,
GitHub CI inspection, an updated resume guide and any fixes pushed. They also
required the manual to be English, matching the game.

The 25-chapter manual is translated to English, including all 80 headings,
14 tables and the search/navigation interface. `PLAYER_MANUAL.md` is canonical;
its generated HTML is used by Help. The earlier HTML address is an English
compatibility copy and the earlier Markdown file is an English pointer.
Runtime notices and the Help control are translated too. Version 0.99.2 keeps
save schema/content 52/52 and adds no runtime dependency.

## Native evidence

The installed 0.99.1 executable was tested in the new isolated profile
`artifacts/english-native-0991`, using the production Sunhaven Gardens park.
The Windows Computer Use connection recovered after two initial timeouts and
a JavaScript-kernel reset. Only returned app/window identifiers were used.

- Right-clicking Golden Wheel's owned queue opened Golden Wheel #6.
- Right-clicking the carousel canopy opened Garden Carousel #4.
- Right-clicking the drinks building opened Services with Drinks #2 selected.
- The actual price buttons changed that facility from $1.00 to $1.50 and back.
  The paused clock (05:36), cash ($9,048.90), sales and served count stayed fixed.
- F5 displayed Park saved. Player data outside the isolated profile was not used.

The helper later lost a usable foreground/window binding during close; no input
was sent to the unrelated content captured in that failure. Final-build manual
launch and final packaging/CI outcomes are recorded below when completed.

## Storage and environment

The first 0.99.2 package also passed native right-clicks on Garden Carousel #4
and Drinks #2 after loading the prior isolated save. Help's section, tooltip
and status text were English. Its manual button exposed a real defect: the
Explorer command opened Documents, and no manual browser window appeared.
The opener was replaced with `ShellExecuteW`, a literal UTF-16 file path with
native separators, null command arguments and checked Windows error codes.
Retesting the corrected build is required before claiming manual acceptance.

The real registered root is
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
The existing registration, project identity and Git remote are preserved. The
local standard prompt is 2.1.0; Drive remains explicitly disabled. Existing
exports stay local. No timing from this PC is product performance evidence.


## Corrected opener and final interruption

The corrected runtime is source `42c7f09fb3bb57ce888d2b7db72c3c1ad0b7396b`,
SHA256 `d6f68d4fcc2df1e9c922cf4cff34c16a2e8687bbf43ec2c101df66f60eb9c67b`.
Strict workspace/all-target Clippy and the release build passed. The prepared
package passed 203 payload checks and 174 current guides. The English manual
contains 9,277 words; all 120 local links, 80 headings and 14 tables were checked.
The resolver test and seven pointer tests passed on 0.99.2 before the final
Windows-opener replacement. The replacement was compiled/linted and requires
the remaining native observation.

The first 0.99.2 package reloaded the isolated saved park correctly and passed
right-click checks for Garden Carousel #4 and Drinks #2. Help displayed English
controls and instructions. The corrected package was then launched to repeat
manual opening. Computer Use reported a physical Escape stop before an input
could be sent. No further Computer Use calls followed. The corrected manual
launcher and search/navigation are therefore NOT recorded as native passes.
The isolated process was left alone; installation requires the game to close.

The previous exact-head CI run `44525877778` passed its test/build and numerous
continuation steps and was still finishing when checked. The latest remote
state is recorded in `artifacts/english-ci.json`; the PR checks are authoritative.
Do not infer a completed workflow from local test results. Remaining work is
native manual-opening/search verification and any still-running remote jobs.
