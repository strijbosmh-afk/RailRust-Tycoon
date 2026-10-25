# Ride cashflows

Open **Finance [N]**, then choose **Ride cashflows** at the top. The view button cycles through **Park summary**, then **Transactions**, then Ride cashflows again. Use **Previous / Next** or **Page Up / Page Down** to browse ride history. Four rides fit on each report page, ordered by stable ride ID. Changing views returns to page one.

Each ride shows its family, current installed/open state, lifetime ticket income, recorded construction spending, refunds, upkeep and net cash:

**Net cash = ticket income + refunds − recorded construction − ride upkeep.**

Coaster construction includes paid track purchases and replacements; refunds include paid track removals. Carousel spending and demolition refunds belong to that specific ride ID. A rebuilt carousel receives a new ID and its own row. The removed ride's history remains visible as **Not installed**.

Older receipts stay included after the bounded recent transaction list rolls over. Facility costs and sales stay separate from ride figures, even when a facility and a ride happen to have the same numeric ID. The park totals above the report still include both rides and facilities.

Legacy free construction has zero recorded cost. These figures describe recorded cash rather than current ride value, forecast profit or payback time. Ride upkeep follows the [operating costs guide](RIDE_OPERATING_COSTS_GUIDE.md); facility upkeep and supplies retain their existing rules. The [park summary](PARK_FINANCE_SUMMARY_GUIDE.md) separates operating profit from construction/refund cash movements. Wages, interest and depreciation are not modelled yet.

The report is read-only. Opening it, switching views and paging do not change money, guests, ride operation, track or saves. Simulation keeps advancing unless paused; **Space** pauses it from Finance. Normal save/load preserves the underlying histories. Loading resets the report to the Transactions view.
