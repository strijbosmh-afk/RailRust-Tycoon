# Paid handyman and path bins — 0.60.0

This stage continues the grounds model on `codex/paid-handyman-bins`, stacked on the 0.59.0 `codex/grounds-handyman` branch and [draft PR #59](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/59). Source repository: <https://github.com/strijbosmh-afk/RailRust-Tycoon>.

## Delivered

- A hired handyman earns $0.50 per 900 employed simulation ticks. Finance records each ownerless wage, includes it in cash conservation and total wages, and tracks mechanic and handyman totals separately through ledger archiving. Unaffordable wages dismiss the worker without debt.
- Up to 64 free bins can be attached to selected public paths. Each holds four litter events; full bins no longer absorb litter, and spills are never absorbed. The existing path network determines whether the worker can reach and empty a used bin. Bin geometry follows the path roof and the dynamic grounds render.
- Build offers a reversible **Bin on selected path** action. Guests → Park stats reports bin and wage data alongside visible mess and worker state. Demolition removes a bin and invalidates a route to it.
- Strict save schema/content advances to **30/30**. Schema 29 migrates to an empty bin list, zero wage progress and zero archived handyman wages. Older versions cannot smuggle new fields or wage receipts; current saves require them. Validation checks path ownership, bounded/sorted bins, fill, route targets and wage progress.
- `write-handyman-bins` / `verify-handyman-bins` run in separate game processes through production save/load. The proof requires naturally used bin, mid-edge worker route, bin emptying, paid wages and an exact 3,600-tick future replay while the original primary slot remains byte-identical.

## Scope and acceptance

Bins are free because this slice has no bin construction ledger. A larger grounds economy could add priced bins and refunds with explicit finance records. There is one worker, no patrol zones, no terrain-wide litter and no staff roster. Real native mouse input and clean target Windows PC acceptance are separate from automated captures and same-PC package relocation. This development PC is not a performance reference.

Run `cargo fmt --all -- --check`, `cargo test --release --workspace --locked`, `cargo clippy --release --workspace --all-targets --locked -- -D warnings`, and `cargo build --release --locked -p park_game --bin parkbound` with the pinned toolchain. Build the Windows ZIP with `scripts/package-windows.ps1`, extract to a fresh Unicode/space folder, verify manifest and sidecar with `scripts/verify-windows-package.ps1`, execute the new two-process proof and prior grounds/mechanic-route/wellness proofs, then inspect `bins-map` and `bins-stats` at 1024×680. Exact final counts, hashes and source identity belong in [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md).

Final local release verification passed **838 tests (479 game / 359 simulation)**, formatting, strict release all-target Clippy and the Windows release build. A fresh extraction verified **104 payload hashes and 92 current guides** and passed the new handyman-bins, prior grounds, mechanic-route and wellness two-process proofs plus both 1024×680 captures. The final `bins-map` and `bins-stats` images were directly inspected; the map bin was also inspected at a cropped 4× view. Packaged executable SHA-256: **`459bdbe5b48bb800877c6ec70e974b39504ca484077ba5400100a254e3e3fe06`**. The development PC is not a performance reference.
