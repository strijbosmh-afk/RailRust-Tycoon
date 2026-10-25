# Ride cashflow reporting — 0.38.0

Version **0.38.0 is locally verified and packaged**, runtime `f5d6297c5e0a8ee73944cea62bde83dce9836e4a` on `codex/ride-cashflow-report` in [draft PR #37](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/37), based on verified 0.37.0 checkpoint `7ddea6acde170146a5c9b6241b530683eac6f51d` / PR #36. The checked-opening and starter-design packages remain retained. No milestone PRs have been merged.

This bounded MGT-07 step adds **Finance → Ride cashflows** alongside recent transactions. Four stable-ID rows per page show coaster/carousel type, installed/open state, ticket income, recorded construction, refunds and net cash. Demolished carousel histories remain visible; rebuilding gets its own row. View changes reset paging. The original ledger view and park-wide totals remain available.

`Park::ride_cashflow` and `ride_cashflows` derive figures from authoritative lifetime ride/track records, including earlier receipts and demolition history. Signed net cash uses integer `i128`, avoiding unsigned subtraction and intermediate overflow. Rendering explains tickets + refunds − recorded construction; facility transactions remain separate. No park mutation, new save fields, dependencies or assets are introduced. Schema/content remains **19/19** and design/catalog formats **1**.

Source tests compare every component with opening, archived and recent owner-attributed ledger entries, including an operating facility sharing a numeric ID with a ride. They verify paid/removal history, legacy free track, strict save roundtrips, actual button switching/paging, simultaneous view/page priority and immutable park state. Production `write-cashflow` / `verify-cashflow` construct a $270 original coaster, a removed $300 carousel with its $150 refund and a separate live $300 carousel. Actual UI refresh labels reconcile with cash and lifetime records, through occupied paid primary, closed backup and exact restart continuation.

[RIDE_CASHFLOW_GUIDE.md](RIDE_CASHFLOW_GUIDE.md) explains the report and its scope. Recorded cash is not current valuation or forecast profit; recurring ride costs, wages, interest and depreciation remain absent. MGT-07, park/company valuation, broader finance and the full game remain unfinished.

## Verification

Formatting, strict all-target release Clippy and all **723 release tests (427 game / 296 simulation)** passed. The final production build completed in **39.19 seconds**. The independently verified local executable is **49,974,272 bytes**, SHA256 **`fddf3dcc945c4e36d0ccc5ae024b14e0fff9d085e4dd75155e1e5222e0a9a847`**. Runtime source remained unchanged after this build.

Fresh extraction outside the checkout to `C:\Users\strij\AppData\Local\Temp\Parkbound 0.38 cashflow delivery é`, launched from `C:\Windows`, passed **60 payload hashes / 48 current guides**, exact starter-design checks and all **24 production proof processes**: writer/verifier pairs for cashflow, opening, finance, starters, trains, operations, dispatch, appeal, design transfer, track construction, access and coaster testing. A separate **four-process 0.37 ↔ 0.38 continuation proof** passed with authentic old/new executables and unchanged source slots, using paid occupied opening profiles and exact replay.

All **seven final 1024 × 680 captures** passed direct inspection: empty report, paid construction with removed history, real paid ride income, second report page, the retained archived transaction page, exit-path opening refusal and Pocket Oval preview. Labels, money, page controls, four-row spacing and footer explanations fit without clipping.

Focused native checks used this exact extracted executable and an isolated copy of the closed cashflow backup. Actual **F9**, the **Finance button**, **Ride cashflows / Transactions buttons**, **Escape** and **F5** passed in the 1280 × 800 client window. Cash remained $280 at tick 0. Independent SHA256 checks prove both saved slots stayed byte-identical to the original closed park after UI navigation and saving. A Page Down attempt at page 1/1 left the display unchanged; key recognition and physical page navigation remain unverified. A later native opening attempt was stopped after a user-input guard and a minimized test window; native opening/O and normal Alt+F4 closure remain unverified. Only the two known isolated test processes were cleaned up, with executable/argument identity checked; default player profiles were unused.

All **291 dependency entries** and five runtime/notice files match 0.37 byte for byte. Prior 0.37 and 0.36 ZIPs remain unchanged. Receipts, native scope and visual review are retained in `artifacts/ride-cashflow-0.38.0/`. The download is `dist/Parkbound-0.38.0-windows-x64.zip`; the adjacent `.zip.sha256` identifies the archive after final guide-only refresh and fresh integrity verification with the same executable.

Exact-source Windows [CI 40183614031](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40183614031), job 121139654448, passed all **39 steps**, including the cashflow proof. The 0.37 source run 40182744749 passed all **38 steps**. No CI artifacts were downloaded or executed.

## Following work

Next extend the management loop with recurring ride operating costs, explicit funds closure and owner-attributed reports, preserving paid passenger journeys and all historical receipts through a versioned save migration. Broader guest arrival/departure, park finances, valuations and scenario progression also remain unfinished.

Remaining physical native input, file-picker, clean-PC/offline/standard-user and representative performance acceptance stay separate. No reference PC is selected; this development PC is excluded. Preserve the normal map/save caps, prior packages and all three continuation paths.
