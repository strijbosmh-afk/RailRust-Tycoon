# Windows distribution session

Project `railrust-tycoon`, session `01a15441-13d3-7142-b742-e896afad208c`, PC
`PC1830`, UTC date 2026-11-18. Branch `codex/phase-12-windows-distribution` begins
at `b84dd85ba2296715fc7d8cfcf3108be0d288d7f0`.

Read the applicable project instructions, registration, status, resume guide and
online standard prompt 2.1.0. GitHub ownership and the existing remote were
verified. The baseline Windows workflow 43893074765 passed. The user requested
the next phase and chose to defer the original source-code license decision.

Implemented the per-user offline installer, running-game guard, version query,
release metadata, user guides and installer lifecycle checks. Formatting, strict
Clippy, the release build and mutex regression passed. The title capture was
inspected. The ZIP passed 179 payload hashes and 157 guide comparisons. All 14
installer lifecycle processes passed with unelevated privileges and system-only
PATH. They preserve 21 park/campaign/settings files and an extra player file
through upgrade, repair, uninstall and reinstall. Runtime commit
`c55c0b7894771e58813277f8c2453ecb7b94fef5` is pushed in PR 87. Player format remains
51/51. The latest documentation refresh and final package identities are retained
in the adjacent checksums and artifact logs.

Native information and destination pages were inspected. Further wizard input
was detected, so automated navigation stopped. The visible setup continued to
install 0.95.0 at the default per-user program path. The isolated automated test
installation was removed successfully.

Shared checkout is
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
The earlier 0.94.0 DropSync transfer has eight verified downloads. Native Drive
export `q-d4882dccb560c1de91b9a01c5d681d98` has an accepted receipt for all 164 files.
Three older exports remain queued. This session uses the normal native Stop
handoff for its next archive. No manual snapshot was created.

The development PC is excluded from product performance acceptance. The full
release retains independent clean-PC, display/hardware and player acceptance.

The Windows job of CI run 44294574454 passed all 1,038 tests and its production
continuation checks. Its installer job exposed a rapid-reinstall test assumption
when Inno Setup chose `unins001.exe`. The verifier now resolves the registered
uninstaller before each removal. Corrected CI confirmation is pending. The final
local ZIP additionally passed production demo write/verify across processes,
18,000-tick continuation and an inspected title capture. The final installer also
rejected the player-data directory without changing the personal installation.
