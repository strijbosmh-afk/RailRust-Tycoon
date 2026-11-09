# Meadow Challenge stage — 0.68.0

## Scope

This phase introduces one original, optional authored objective. A saved scenario kind and start baselines distinguish challenge progress from the park's lifetime arrival and ride counters. The win condition checks 20 new admissions, 5 completed visitor ride trips and at least $750.00 cash on the same simulation tick. Completion is recorded once and persists. It uses existing park entry, ride, wallet, finance and advertising mechanics. Sandbox remains the default.

Starting the challenge atomically sets Free entry and locks charging policy. Radio campaigns are prohibited; an active radio campaign prevents the challenge from starting. Flyers stay available. The UI shows live progress, completion and clear refusal notices. The challenge has no deadline, loss condition, award or campaign progression yet. The cash goal accepts loan proceeds; this stage does not claim a profitability objective.

Strict saves advance from **37/37 to 38/38**. Older schemas migrate to sandbox, and current saves require the scenario field. Loaded challenge state checks start tick, baseline counters, completed tick and locked policy. The independent `write-challenge` / `verify-challenge` production proof starts through the player command, rejects forbidden actions, saves progress before the arrival goal, loads paused in another process, replays exactly 3,600 ticks to completion across flyer expiry, and checks that the original primary save remains byte-identical. Windows CI runs the proof.

## Verification and limits

All **877 release workspace tests (492 game / 385 simulation)** passed after correcting a synthetic version-11 save fixture to omit the new field. Formatting, strict release all-target Clippy and the Windows release build passed. Executable SHA-256: `084932df054826a8ffc23d81c7198c0f804e2cfea5004a7850afaaeeb0a355fa`. A fresh Unicode/space ZIP extraction checked **120 payload hashes and 108 current guides**. Eight extracted-game processes passed the challenge, radio, flyers and wellness writer/verifier pairs. The challenge proof saved before the arrival goal, reloaded paused, replayed exactly 3,600 ticks to completion through flyer expiry, and left the primary save unchanged.

The 1024×680 `visitor-challenge`, `visitor-challenge-completed` and `flat-start` captures passed and were directly inspected. The panel shows the goal and controls without clipping, the completed marker is legible, and the fresh starter station and low track remain at ground level. Capture SHA-256 values: `bb3f930821d19b2b767d72bd43ecc1c5f58614bafeb8a749c2676ee731988f0c`, `741cfebf75772eb138ac1fa31d696fd811525f26203fddfa225f13ae6424c55d`, and `29944e04d546fd44453c55e04e4f6cd422c76fb996d74b0f1051cc73b7b3c186`. The unsigned ZIP is `dist/Parkbound-0.68.0-windows-x64.zip` with an adjacent `.zip.sha256`; refresh and recheck after guide edits. This development PC is not a performance reference. Native controls and a clean target Windows PC remain separate acceptance.

Next add a scenario selection/start screen and multiple original scenarios, then deadline/failure rules, a recorded campaign progression path, and eventually an editor. The current rule and goal definitions are one built-in preset, not an external scenario format.
