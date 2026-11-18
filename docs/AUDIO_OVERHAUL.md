# Parkbound 0.97.0 audio

## Soundtrack

Three original instrumental arrangements play as a continuous playlist.

- **Sunhaven Parade** uses a bright melody, chord accompaniment and a steady 112 BPM rhythm.
- **Lakeside Swing** uses a swung melody and lighter accompaniment at 104 BPM.
- **Starlight Carnival** uses a quicker 124 BPM rhythm and answering melody.

Each composition has 32 bars with quieter passages, fuller choruses and fills.
Three-second overlaps join the tracks. The playlist lasts about 3 minutes
15 seconds, with an altered bridge on subsequent passes. Stereo accompaniment
and a short cross-channel delay give the music space.

## Zoom and attraction sound

The listener uses the same camera layout, scale, rotation and UI offset as the
rendered park. Moving and zooming toward an attraction increases its presence.
Sources outside that view fade with distance. At most four nearby sound
sources are mixed, with smooth changes between them.

- Coasters have rolling wheels, rail joints, wind on descents and a metallic chain on actual lift sections.
- Log flumes and rapids have flowing water and stronger splashes on descents. Open water rides remain audible while a boat waits at the station.
- Railway and monorail have distinct movement sounds.
- Carousels, wheels, cups, swing rides, ships and towers have their own sound patterns.
- Nearby occupied descents and completed rides can produce enthusiastic visitor reactions. Crowd presence follows actual nearby visitors.

The music becomes quieter near prominent rides. Ride motion fades out when
the simulation is paused. The title screen plays the soundtrack with the
park sound sources quiet. Window focus, master mute and the existing separate
music, ambience, rides, interface and reactions controls remain available in
**F10**.

The crowd uses AuraVoice's CC0 recording. The retained original, converted PCM
and terms are in `assets/audio/`. The package includes those files and their
provenance. The remaining music and effects are original procedural work.
[Audio provenance](../assets/audio/PROVENANCE.md)

## Reproduction

The Rust toolchain and runtime dependencies are unchanged. The WAV is embedded
in the executable. Player machines need no audio download or decoder package.
Sound settings remain version 2, and park saves remain schema/content 51/51.

```powershell
cargo test --release --package park_game sound:: --locked
cargo build --release --locked --package park_game --bin parkbound
./target/release/parkbound.exe --sound-check C:\Temp\Parkbound-audio-new
./target/release/parkbound.exe --verify-sound C:\Temp\Parkbound-audio-new
./target/release/parkbound.exe --sound-device-check C:\Temp\Parkbound-device-new
```

Use new output directories. The sound proof exercises the real volume controls
and independently reproduces eleven WAV excerpts. These include all three
themes, water, chains and cheering. The device check confirms Windows accepts
a stereo PCM block. Listening quality is assessed separately from those
technical checks. Current results are recorded in the session report.

## Google Drive synchronization

The owner disabled synchronization on 2026-11-18. `PROJECT.json` records
`drive.syncEnabled = false`. This PC's existing registration also records
`driveSyncEnabled = false`. The local archive helper reads either flag and
keeps local exports while suppressing Drive upload continuations and cloud
prompt requests. Existing project IDs, folders and queued exports are retained.

On another PC, respect the project flag before processing any queued upload.
Use the cached standard prompt and the instructions in `AGENTS.md`. Keep the
helper's per-project synchronization guard when updating its installation.
The owner's explicit request is required to enable Drive transfers again.

The shared checkout remains
`C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`.
GitHub remains the source transfer route. This PC supplies correctness checks
and is excluded from performance acceptance.
