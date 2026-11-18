# Soundtrack and spatial audio session

Project `railrust-tycoon`, session `01a15441-13d3-7142-b742-e896afad208c`,
PC `PC1830`, checkpoint 2026-11-18 15:35 UTC. Branch `codex/soundtrack-and-spatial-audio`
starts at `7c41a3ef5b9e7c55cf3a35d4a857ce0717e28911`.

The user requested stopping Google Drive synchronization, varied entertaining
music and attraction-specific sound with enthusiastic visitors while zooming.
The existing Git root, status/resume documents and GitHub repository were
checked. The cached standard prompt is version 2.1.0. Its cloud copy was not
read after synchronization was disabled.

`PROJECT.json` and this PC's existing project registration now disable Drive
synchronization. The local archive helper honors both flags, keeps local
exports and suppresses upload continuations. Existing registrations, remote
IDs and queued exports are preserved. A local backup of the helper and config
was made before the change. Node syntax and the pure policy checks passed.
No manual snapshot or hook was run, and no Drive calls were made.

Three original arranged compositions, four-source camera-aware ride sound,
actual lift and water effects, and CC0 recorded visitor cheering are
implemented. The WAV is embedded and bundled with its original and license.
Sound settings remain version 2. Saves remain 51/51.

All 581 game and 464 simulation tests passed on the final mix, together with
formatting, strict Clippy and the release build. Sixteen focused audio tests
cover zoom, rotation, stereo gain, source fade, pause/mute, compatibility and
sample-block independence. The test park is advanced through its real station
wait before asserting moving-train sound. An earlier run was superseded while
the mix was balanced. The final complete run is `artifacts/audio-final-tests.log`.

Eleven soundtrack/effect WAVs were reproduced exactly across fresh processes.
PCM format, nonzero RMS and headroom checks passed. Windows accepted and
completed the native stereo output block. The extracted package passed 191
payload hashes, 164 guide checks, demo/front continuation and an additional
sound verification launched from `C:\Windows`. Title and credits captures
were inspected. The package verifier now includes the new audio provenance.

The real 0.96.1 installation was upgraded to 0.97.0. All 191 installed payloads
matched and both existing player files were preserved. Both shortcuts retained
the Parkbound icon. Runtime `44c25003d7bb7172175ec85f385c2c2394c9e56c` is
published in PR 90. Executable SHA256 is
`e86078439abe0730613dfc7cc7682043fcac8075ecaba6b11cdd5ccddabd75f4`.
ZIP and installer checksums accompany the distributions in `dist/`.

GitHub run `44333819884` had passed tests and packaging and was continuing
integration checks at this checkpoint. Superseded runs on this branch were
cancelled. Read `docs/AUDIO_OVERHAUL.md` for commands and
`assets/audio/PROVENANCE.md` for attribution and conversion details. Remaining
acceptance is physical listening on speakers/headphones and the broader
independent hardware/player checks. The local helper change was verified by
pure policy checks, with evidence in `artifacts/drive-sync-disabled.json`.
Installation of that helper change on other PCs is unverified.

The shared local path is
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
This PC is excluded from performance acceptance.
