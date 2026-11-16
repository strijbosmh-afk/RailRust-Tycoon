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

Verification is in progress. Results and exact source/package identities will be recorded after checks finish. Raw local evidence is under `artifacts/phase11/` and `artifacts/phase11-*.log`.

## Remaining acceptance

Phase 11's full exit gate remains open until representative hardware/display checks and independent player/audio/balance sessions pass. No target frame rate, minimum specification or playtest outcome has been inferred from this development PC. Existing save schema/content 51/51 and independent settings formats remain in place.
