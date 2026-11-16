# Phase 11 verification

Version 0.90.0 on `codex/phase-11-beta-stability`, based on Phase 10 commit `3cecab22fe24aa6063ff4cb9d8b0dc53e3bfd07d`.

## Implemented work

- Actionable Operations page uses production access, maintenance, stock and staff state. Inspection opens the affected management panel. Queries preserve simulation state.
- Four current-content beta workloads cover the starter, full catalogue, 256×256 map and active-visitor capacity. Original diagnostic cases retain their content.
- Checkpoint writer compares continuously advanced and repeatedly reloaded parks, verifies backup rotation and reconciles finance against independent ledger buckets.
- A fresh verifier process replays from the initial checkpoint and compares the complete authoritative park at every interval. It checks that primary and backup files remain unchanged.
- Machine-readable reports retain performance, throughput, finance, satisfaction, guest thoughts, research and staff observations. A shipped PowerShell runner adds executable identity, bounded process timeouts and memory sampling.
- Windows CI exercises all four cases across fresh processes and retains the resulting evidence.

## Evidence

Runtime checkpoint `3f3bf6da1ff57b861ad4d6bb1358a144dfd4e97a` is published in PR 82. The final implementation includes one breadth-first distance traversal per ride/staff decision. An independent replay against pre-optimization checkpoint saves preserves the complete authoritative park after 900 ticks. The route-distance regression compares every pair in the starter graph against production route reconstruction.

[Windows CI run 43838769874](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/43838769874) passes the complete 70-step build, test, packaging and gameplay-proof job and all four one-hour endurance jobs. Earlier checkpoints 3902a16 and 88350a2 also passed their 70-step Windows job. The final optimized regression passes 1,029 tests (566 game and 463 simulation), both locally and in Windows CI. Strict all-target release Clippy and the release build pass.

The optimized Windows executable SHA256 is `b6ec57e229f9b8f414e97768da7122aa0cba9bc5c1c78ccab79aacef67a499e6`. All four optimized 900-tick beta cases passed separate writers and verifiers, including the new 48/126/256 visitor populations. Minimum-window and 2560×1440 Operations captures were inspected for this executable. The optimized 150-payload package passed extraction, 136 current-guide checks, startup/library proofs, the complete campaign/editor proof and the relocated starter beta runner. The final package is refreshed after the evidence update.

The 108,000-tick mixed-park run passes all 13 checkpoints, backup rotations, exact reload comparisons and account checks. Its final state records 278 completed rides, 1,583 completed service visits and $7,349.50 cash, with 52 guests present and 1,413 historical departures. The $120 facility-research category is complete. Both local and CI separate-process replay pass for this state. CI also passes the 256-initial-guest case with 234 ride completions, 1,058 service uses and $7,022.50 cash after one hour. The medium and crowd artifacts were downloaded and their SHA256 values checked against GitHub metadata.

The large mixed case also passes 108,000 ticks and all 13 replay checkpoints, ending with 244 completed rides, 1,139 service uses and $7,086.50 cash. Across all four CI cases, 432,000 unique simulated ticks and 52 saved checkpoints were replayed in separate processes. Each writer additionally compares uninterrupted and repeatedly restored simulations. Local and CI medium checkpoint files match byte-for-byte at all 13 checkpoints.

Verified endurance artifact hashes are `3f7c504fb477c11f7a2b2cb1501d1e4382c266c76ffe9ccfd516ebe6e5e4a065` for medium artifact `14214483562`, `99bdaea0d7255f9f20fa09b1f3497efe6d3a512723dce7681a3502dc8f5ab52e` for capacity artifact `14214104008`, and `0889650c2666d1150db5d619063621de5d57dcf9ebd9eb698b75980bc6abafbb` for large artifact `14216586875`.

The optimized CPU measurement completes with the intended 48-guest, 14-ride fixture. The 256×256 frame/GPU probe completes all 120 samples and receives hardware timestamp spans. Both remain local diagnostics. The CI package artifact `14213536763` has verified wrapper SHA256 `8fccedcc74653f7c7a69dc719b5c9f5402da1626f39ef6ae3b25bb8e62054e93` and executable SHA256 `e6f0e56a410428345cc9a484b2440df74d60b9710013b2347412956f421b9afe`. CI beta artifact `14213111187` has verified SHA256 `598224ca77c63380b126151f5b71881b2c52817285a349ad1db761287847ab9b` and all four complete writer/replay records.

The initial native capture process ended without an image. A second capture using an explicit isolated profile completed at both resolutions. New physical keyboard/mouse gestures were not exercised in this session.

Raw local evidence is under `artifacts/phase11/` and `artifacts/phase11-*.log`. One local test link attempt encountered an occupied executable. The retry uses the same source. Initial empty mixed-fixture smoke reports and an interrupted endurance run are retained separately from the final populated cases.

## Remaining acceptance

Phase 11's full exit gate remains open until representative hardware/display checks and independent player/audio/balance sessions pass. No target frame rate, minimum specification or playtest outcome has been inferred from this development PC. Existing save schema/content 51/51 and independent settings formats remain in place.
