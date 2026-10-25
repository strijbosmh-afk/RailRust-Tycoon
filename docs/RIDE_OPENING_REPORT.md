# Checked ride opening — 0.37.0

Implementation is in verification on `codex/ride-opening-checks`, based on the verified 0.36.0 checkpoint `b861b75`. Previous exact-source Windows CI 40141146431 passed all 37 steps. The three original starter designs and configurable car-count packages remain retained.

This bounded RID-05 step checks the installed ride, entrance/queue and public exit return path before player opening. Operations, the main Open button and O use the same simulation `set_ride_open_checked` boundary. Refusals are atomic and display a reason; the Operations status describes readiness before a click. Every open request derives its answer from current geometry and access. No cached certificate can survive a path edit.

`ride_opening_status` checks known ride identity, installed availability, current coaster occupied envelopes and train availability, then the existing entrance/queue and exit-network rules. Construction and strict saves already validate installed geometry/clearance. The low-level `set_ride_open` switch remains for fixtures and historical continuation; normal player controls use the checked command. Closing is permitted with damaged access so existing trips can finish and unload after repair.

Mechanics tests remain isolated from access readiness. They can run on a closed, stopped, empty installed coaster before gates are built. No test certificate, new live test mode, force simulation, automatic path repair or auto-closing on access damage is added. Existing saves keep their open state and paid journeys. Saves remain schema/content **19/19** and design/catalog formats **1**.

[RIDE_OPENING_GUIDE.md](RIDE_OPENING_GUIDE.md) explains the player flow. Source tests cover both ride families, absent and damaged access, queue and exit-path repairs, atomic refusal, closing after damage, real Operations/main-button/O input paths, paid journeys and strict save continuation. `write-opening` / `verify-opening` exercise production controls and fresh-process continuation with an occupied paid primary and a closed ready backup for both rides.

The retained empty-dispatch fixture now builds valid access for both rides and uses the maximum permitted fare to keep its existing guests out. It still proves positive empty loading waits and timeout departure without charging a fare, through the checked player opening command. The paid fixture retains ordinary affordable fares. Existing shortcut tests now expect a refusal when reopening an unconnected demonstration coaster.

Verification receipts will be retained in `artifacts/ride-opening-0.37.0/`. Native physical input/file-picker, clean-PC and representative hardware acceptance remain separate. No reference PC is selected; this development PC is excluded. The full game, broader RID-05 test modes, multiple coasters, staff, maintenance, research, scenarios, audio and complete management remain unfinished. Preserve the normal map/save caps and all three continuation paths.
