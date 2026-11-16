# Phase 10 verification

Version 0.89.0 on `codex/phase-10-sound-accessibility`, based on Phase 9 commit `7c2fa3c70bbbdd21917444068360e6b577bd1fa4`.

## Scope

- Seven-step guide observes actual path construction, a new Carousel, ready ride access, completed visits and a successful save. It opens the existing paid tools and can be skipped or replayed.
- Keyboard focus, forward/reverse traversal, foreground modal filtering, focus scrolling, and Enter activation operate through existing button handlers.
- Eighteen bounded unique shortcuts are stored independently. Text entry and modifier chords preserve their existing input path. Fixed Help, navigation, sound, window recovery and Escape remain available.
- High contrast, captions and reduced decoration motion are independent presentation settings. Actual moving vehicles and guest positions remain visible.
- Original second theme, rain, guest/staff responses, independent reaction level, v1 audio preference migration and retryable Windows PCM output.

## Verification record

The full release suite passed 1,021 tests (562 game and 459 simulation) at source `775854c`, including focus-scroll, pointer-boundary and contrast-outline refinements. Final targeted tests and Windows CI cover the final source. Strict release Clippy passed after these refinements. Final source, build and package identities are recorded below when verified. Raw local evidence is under `artifacts/phase10/`; source checks also use `artifacts/phase10-tests.log` and `artifacts/phase10-clippy.log`.

Native checks observed Enter → New Park → Create Park → guide, Help opening, Tab traversal with an explicit focus outline and scroll, Enter → Controls & accessibility, saved contrast and a changed Paths shortcut with the toolbar label updated to F3. A later foreground change interrupted the remaining native sequence. Controlled captures supplement these checks.

The old phase remains the comparison baseline. This PC is excluded from performance acceptance. Subjective listening, extended disconnect/reconnect testing and a new-player session on independent hardware remain Phase 11 acceptance work.

## Source and local delivery

Runtime source commit `dae33c85753003a526315b000e883e63ee8c6856` is published in PR 81, stacked on Phase 9. The preliminary executable SHA256 was `6a793eec752be41db06901c4863fc0730628f193e279ae5bd479943d85607e28`; the delivery identity below includes the successful-save revision fix.

- Full release regression passed 1,021 tests. Five final accessibility tests, both final guide tests and the repeated-save regression pass.
- Formatting and strict all-target release Clippy pass. The final Windows release build passes.
- The final executable passes separate writer/verifier processes for accessibility preferences and six deterministic sound clips. The Windows PCM device proof also completes.
- Controlled captures were inspected at 1024×680, 1280×800, 2560×1440, 3440×1440 and 3840×2160, including 100%, 150% and 200% interface scale and high contrast. The final minimum-size image includes explicit button outlines and reduced motion.
- Native input verification was stopped with the physical Escape key. Further native gestures remain unverified in this session.
- The extracted package validates 146 payload hashes and 133 current guides. Its park-library and portable-scenario writer/verifier processes pass from a fresh Unicode/space path under `C:\Windows`. Extracted accessibility preferences also pass both fresh processes. Sound and accessibility captures were inspected. Final packaging is refreshed after this report.

The archive is `dist/Parkbound-0.89.0-windows-x64.zip`. Its adjacent SHA256 file records the current ZIP identity after documentation refresh. Raw native/capture/settings/audio results are under `artifacts/phase10`.

A final review found that saving twice could repeat the same displayed message. The guide and save caption now observe a separate session-only successful-save counter, so a successful repeated save always completes the step. The counter is excluded from park files. A regression covers two identical save messages with separate successful revisions.

Delivery executable SHA256 `4840818ee166a989b9a9cbeea4e327539b5f542537a3a9a4dc215fafcc89fa35` corresponds to runtime source `dae33c85753003a526315b000e883e63ee8c6856`. The final guide and repeated-save tests pass. The final suite passes 1,022 tests (563 game and 459 simulation). Exact-source Windows CI run 43829013933 passed all 68 steps, including packaging, accessibility restart, audio restart and retained gameplay checks.

The six original PCM clips have zero clipped samples. The forty-second mix has a 0.0819 full-scale peak and 0.0120 RMS. The evening/reaction clip has a 0.0448 peak. These are signal measurements, with listening acceptance recorded separately.

## Published verification

[Windows CI run 43829013933](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/43829013933) verifies exact runtime source `dae33c85753003a526315b000e883e63ee8c6856`. Job 133169532500 passes all 68 steps. The Windows artifact ID is `14208687182`, wrapper size 19,112,961 bytes, wrapper SHA256 `1dbfc3ac69cdd117722cd56dc092ed95492fba1bf9d42385625bf4f820bb87dd`. Its contained CI ZIP hash is `723f6d0d91ed4f5684bb405e7ff75572da7adbf90672b0685cac983443f3b587`. Local package hashes are recorded separately.

The refreshed local package retains the delivery executable SHA256 above and the updated guides. Raw source, PCM, preference and extraction evidence remains in ignored `artifacts/phase10/`. Phase 10 implementation is complete. Phase 11 retains independent new-player usability, extended audio-device behavior, mixed-DPI/native display checks, balance and representative-hardware acceptance.
