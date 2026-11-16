# Phase 10 verification

Version 0.89.0 on `codex/phase-10-sound-accessibility`, based on Phase 9 commit `7c2fa3c70bbbdd21917444068360e6b577bd1fa4`.

## Scope

- Seven-step guide observes actual path construction, a new Carousel, ready ride access, completed visits and a successful save. It opens the existing paid tools and can be skipped or replayed.
- Keyboard focus, forward/reverse traversal, foreground modal filtering, focus scrolling, and Enter activation operate through existing button handlers.
- Eighteen bounded unique shortcuts are stored independently. Text entry and modifier chords preserve their existing input path. Fixed Help, navigation, sound, window recovery and Escape remain available.
- High contrast, captions and reduced decoration motion are independent presentation settings. Actual moving vehicles and guest positions remain visible.
- Original second theme, rain, guest/staff responses, independent reaction level, v1 audio preference migration and retryable Windows PCM output.

## Verification record

The full release suite passed 1,021 tests (562 game and 459 simulation) before the final focus-scroll, pointer-boundary and contrast-outline refinements. Final targeted tests and Windows CI cover the final source. Strict release Clippy passed after these refinements. Final source, build and package identities are recorded below when verified. Raw local evidence is under `artifacts/phase10/`; source checks also use `artifacts/phase10-tests.log` and `artifacts/phase10-clippy.log`.

Native checks observed Enter → New Park → Create Park → guide, Help opening, Tab traversal with an explicit focus outline and scroll, Enter → Controls & accessibility, saved contrast and a changed Paths shortcut with the toolbar label updated to F3. A later foreground change interrupted the remaining native sequence. Controlled captures supplement these checks.

The old phase remains the comparison baseline. This PC is excluded from performance acceptance. Subjective listening, extended disconnect/reconnect testing and a new-player session on independent hardware remain Phase 11 acceptance work.
