# Park startup

- Project `railrust-tycoon`
- Session `park-startup-2026-11-19` on `MICHIEL-BUILD`, 17:37 UTC
- Branch `codex/right-click-settings`, starting commit `9843b075d972e4a671c62920a406d0dc818efb6e`

The demo ran 9,000 opening ticks after every click. The intro now starts the
same preparation on the background compute pool. Subsequent plays clone the
prepared park. Both demo and new-park actions display a startup screen before
the transition. The existing Rust toolchain and Windows build requirements
apply. Park saves remain schema/content 52/52.

Formatting, strict workspace Clippy, all six front-end tests and the release
build passed. Native Windows checks opened Sunhaven Gardens and a new 32×32
sandbox with the first-park guide, using a temporary save directory. The
existing local manual-path edit was present in that build.

Measure click-to-playable timing on representative hardware if a release
performance figure is needed. This session produced a local source build.
