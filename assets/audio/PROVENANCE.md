# Audio sources

The three compositions, arrangements, synthesized instruments, ride effects,
wind, rain and birds are original Parkbound work in `sound/score.rs` and
`sound/synth.rs`.

## Visitor cheer

AuraVoice released the group cheering recording as **CC0 1.0** on OpenGameArt.
The author permits unrestricted use and requests no credit. The retained
original is `cheers-original.ogg`, downloaded on 2026-11-18. Its SHA256 is
`10d9f614307127a8c2b2b129e9a291b6071c8fc91f1fed4916f78ad47c956b25`.
[Original publication](https://opengameart.org/content/cheers-0)

`guest-cheer.wav` is a technical conversion to mono 44,100 Hz signed 16-bit
PCM, linearly resampled from the original 48,000 Hz mono recording. Peak
amplitude is normalized to 0.75, with 10 ms edge fades. It contains 89,485
frames and retains the full recording. `scripts/convert-cheer.py` performs the
conversion. Conversion used Python with soundfile
0.14.0 and numpy 2.5.5 in the ignored local tooling directory. These tools are
only needed when regenerating the asset. The committed WAV is embedded in the
game. Its SHA256 is
`f3823e9b2a3a3aeebcb5c8d10810a5f2cebb620793fbef52ddd5ad9215ba6634`.

The full CC0 legal text is retained in `CC0-1.0.txt` and appended to the
packaged third-party notices. The mixer layers this recording at slightly
different pitches and stereo positions to build a small enthusiastic crowd.
[CC0 terms](https://creativecommons.org/publicdomain/zero/1.0/)
