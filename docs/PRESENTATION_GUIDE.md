# Visuals and sound — 0.44.0

The park has richer grass and sand details, small meadow flowers, fuller tree canopies and moving water glints. Decorative trees clear away from nearby constructed surfaces. These details change presentation only; they do not change construction costs, terrain collision, guest choices or save data. Surface and Underground keep their existing picking and depth rules. Water glints animate on normal maps; larger diagnostic maps retain static water to keep the overlay bounded. Pausing freezes the glints.

Open **Help → Sound [F10]**, or press **F10** directly, to adjust sound. Five independent controls cover Master, Music, Ambience, Rides, and Alerts & interface. Use -10/+10, Mute/Unmute, Test chime or Reset defaults. Escape/F10 returns to the park. Defaults are 40%, 30%, 55%, 65% and 70%; saved mute and levels return next launch.

Music is an original short meadow theme. Ambience combines soft wind, spaced bird chirps and crowd texture. Coasters have wheel/chain textures; operating carousels have a light melody. Interface clicks, construction, removal, actual customer sales and closures have gentle cues. Multiple cues can mix; sound does not change the simulation or park RNG.

Music and ambience continue while paused. Ride effects pause. Sound fades when the game loses focus and resumes when focus returns. If no output device can be opened, the status explains the problem and the game stays playable. It uses the Windows default output; restart the game after changing output devices if sound stops.

Your levels are stored in `sound-settings.json` beside the park save directory, independently of the park's primary/backup slots. A settings write failure keeps the chosen levels for the current session and displays a notice. Reset defaults restores only sound levels and mute. Park saves remain schema/content 22/22 and are compatible with 0.43.0; saved track formats stay version 1.

All sounds are synthesized from original Rust code; no recordings, downloaded music or external sound pack is required. Controlled capture and diagnostic runs disable live output and settings writes. The developer sound proof exports five WAV previews and verifies exact PCM replay separately from native device completion and human listening.
