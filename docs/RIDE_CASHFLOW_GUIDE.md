# Ride cashflows

Open **Finance [N]**, then choose **Ride cashflows** at the top. The **Transactions** button returns to the recent ledger. Use **Previous / Next** or **Page Up / Page Down** to browse. Four rides fit on each report page, ordered by stable ride ID. Changing views returns to page one.

Each ride shows its family, current installed/open state, lifetime ticket income, recorded construction spending, refunds and net cash:

**Net cash = ticket income + refunds − recorded construction.**

Coaster construction includes paid track purchases and replacements; refunds include paid track removals. Carousel spending and demolition refunds belong to that specific ride ID. A rebuilt carousel receives a new ID and its own row. The removed ride's history remains visible as **Not installed**.

Older receipts stay included after the bounded recent transaction list rolls over. Facility costs and sales stay separate from ride figures, even when a facility and a ride happen to have the same numeric ID. The park totals above the report still include both rides and facilities.

Legacy free construction has zero recorded cost. These figures describe recorded cash rather than current ride value, forecast profit or payback time. Recurring ride running costs, wages, interest and depreciation are not modelled yet. Facility upkeep and supplies retain their existing rules and appear in park finances.

The report is read-only. Opening it, switching views and paging do not change money, guests, ride operation, track or saves. Simulation keeps advancing unless paused; **Space** pauses it from Finance. Normal save/load preserves the underlying histories. Loading resets the report to the Transactions view.
