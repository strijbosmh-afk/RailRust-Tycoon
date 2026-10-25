# Visitor arrivals and departures — 0.43.0

Visitor flow is opt-in; loading an older park keeps its existing crowd behavior until you enable it.

1. Close any inspection window and select an ordinary public path in the park. Choose a path connected to at least one other public path; queues and ride gates cannot serve as park entry.
2. Open **Guests**, choose **Visitor flow**, then **Use selected path**.
3. Choose **Toggle arrivals**. The view shows whether the entry is connected and whether arrivals are on.

One visitor arrives every 30 simulation seconds while entry is connected, up to 64 active guests. Existing larger crowds are preserved. New visitors bring their own $20 wallets; that money becomes park income only when they actually buy a ride or service. Pausing with Space freezes arrivals, walking and visits. Turning arrivals off stops new visitors; tracked guests still finish their stays and leave.

Enabling arrivals starts a ten simulation minute stay for the existing crowd. New arrivals each get ten minutes. Afterward they finish any current ride, queue or service visit and walk along public paths to the entry. They leave only on reaching it. Closing admission can clear queues; paid passengers complete their journeys normally.

To request an early departure, choose a guest in **Guest list**, return to **Visitor flow** and choose **Send selected home**. The guest stays selected while active. The display distinguishes visitors leaving from those still finishing visits.

Keep a public return route to the entry. Disconnected guests remain in the park; restore paths to let them leave. If you remove the entry, arrivals pause and departing visitors wait. Select a new ordinary public path and use **Use selected path** to resume. No guest teleports through missing paths.

Departed visitors' spending, ride completions, ratings and service history remain in lifetime accounting. Their unspent wallets leave with them. Finance retains actual ride/service operating profit, construction/refunds, loan funding, interest and book value separately. Guest list and satisfaction summaries describe current guests.

Saves use schema/content **22/22**. Versions 1–21 migrate explicitly with arrivals disabled and no invented departures; old 0.42 and earlier readers refuse new saves. Keep a separate copy before moving saves between versions. Existing 256 active-guest save cap and 8 MiB file cap remain; this first flow is capped at 64 new-flow active guests and one million lifetime guests.

This milestone provides deterministic entry/stay/departure management. Demand, admission tickets, decorative entry buildings, guest budgets, marketing, scenarios and full-game completion remain future work.
