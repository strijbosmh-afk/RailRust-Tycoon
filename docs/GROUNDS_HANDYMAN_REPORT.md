# Grounds and handyman stage — 0.59.0

## Delivered

- Opt-in grounds activity creates deterministic litter or spills on occupied public paths at a bounded rate. The cap is 64 distinct dirty paths.
- Dirty paths slowly lower the satisfaction of visitors standing on them. Nausea determines spill versus litter. Grounds activity consumes no random draws and does not alter park cash.
- One assigned handyman follows connected public paths from the configured park entry, targets a reachable mess, cleans on arrival and returns toward the entry. Position interpolates across flat and sloped paths.
- Dedicated dynamic mesh draws roof-clipped debris and a uniformed worker with a broom. The mesh refreshes on simulation and view changes without rebuilding static terrain. Guests → Park stats exposes counts and the assignment control.
- Strict save schema/content advances from 28/28 to 29/29. Older versions migrate to inactive grounds. Current saves require grounds state; old-version field smuggling, invalid path references, malformed routes, duplicate IDs and oversized mess lists are rejected. Demolition reconciles dirt and worker route.
- New `write-grounds` / `verify-grounds` modes use the production save/load path in separate processes. They compare an exact 3,600-tick continuation and confirm the original primary slot was not rewritten.

## Scope and limits

The first handyman assignment is free and has no payroll entry. A future phase can add a paid staffing model. Grounds can accumulate after dismissal; parks that never assign a handyman preserve the prior simulation behavior. The model has one mess per public path and does not yet include bins, sweeping zones, guests visually discarding items or water contamination. A connected park entry is required for a worker to move. This PC is not a performance reference.

## Verification

Run `cargo fmt --all -- --check`, `cargo test --release --workspace --locked`, `cargo clippy --release --workspace --all-targets --locked -- -D warnings`, and `cargo build --release --locked -p park_game --bin parkbound`. Package with `scripts/package-windows.ps1`, extract to a new Unicode/space folder, run `scripts/verify-windows-package.ps1`, then run the two grounds proof modes and capture `grounds-map` and `grounds-stats` at 1024×680. Inspect the captured roof/depth alignment and the Park stats control. Exact results and commit/PR identities are in [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md).

The final local source checks passed: **835 release tests (479 game / 356 simulation)**, formatting, strict release all-target Clippy and the Windows release build. Executable SHA-256 is **`c3907681dc9a62649d706059e05287f10d675e5c9bbb996bf5607a521b1d710b`**. Fresh extraction in `artifacts/grounds-handyman-0.59.0/final proof é/` checked **102 payload hashes and 90 current guides**. Independent grounds, prior mechanic-route and wellness two-process proofs passed. The new proof saved a real worker mid-edge, replayed exactly 3,600 ticks, cleaned a path and left the original primary byte-for-byte unchanged. Both packaged 1024×680 `grounds-map` and `grounds-stats` captures passed and were visually inspected. Their SHA-256 values are `21f15cb11e1f8797f0116b0b19ce30654f98ca14437ac2369d4e286f1b46cc56` and `88666fc88b8450c77a69a80915d08563033907c3f4e4918a5860bb438dcc4bb1`. This development PC is not a performance reference; physical input and clean-PC acceptance remain separate.
