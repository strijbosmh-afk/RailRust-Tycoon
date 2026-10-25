# Ride cashflow reporting — 0.38.0

Implementation is in verification on `codex/ride-cashflow-report`, based on verified 0.37.0 checkpoint `7ddea6acde170146a5c9b6241b530683eac6f51d` / PR #36. The checked-opening and starter-design packages remain retained.

This bounded MGT-07 step adds **Finance → Ride cashflows** alongside recent transactions. Four stable-ID rows per page show coaster/carousel type, installed/open state, ticket income, recorded construction, refunds and net cash. Demolished carousel histories remain visible; rebuilding gets its own row. View changes reset paging. The original ledger view and park-wide totals remain available.

`Park::ride_cashflow` and `ride_cashflows` derive figures from authoritative lifetime ride/track records, including earlier receipts and demolition history. Signed net cash uses integer `i128`, avoiding unsigned subtraction and intermediate overflow. Rendering explains tickets + refunds − recorded construction; facility transactions remain separate. No park mutation, new save fields, dependencies or assets are introduced. Schema/content remains **19/19** and design/catalog formats **1**.

Source tests compare every component with opening, archived and recent owner-attributed ledger entries, including an operating facility sharing a numeric ID with a ride. They verify paid/removal history, legacy free track, strict save roundtrips, actual button switching/paging, simultaneous view/page priority and immutable park state. Production `write-cashflow` / `verify-cashflow` construct a $270 original coaster, a removed $300 carousel with its $150 refund and a separate live $300 carousel. Actual UI refresh labels reconcile with cash and lifetime records, through occupied paid primary, closed backup and exact restart continuation.

[RIDE_CASHFLOW_GUIDE.md](RIDE_CASHFLOW_GUIDE.md) explains the report and its scope. Recorded cash is not current valuation or forecast profit; recurring ride costs, wages, interest and depreciation remain absent. MGT-07, park/company valuation, broader finance and the full game remain unfinished.

Validation receipts will be retained in `artifacts/ride-cashflow-0.38.0/`. Physical native input, clean-PC/offline/standard-user and representative performance acceptance remain separate. No reference PC is selected; this development PC is excluded. Preserve the normal map/save caps, prior packages and all three continuation paths.
