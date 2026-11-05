# Park entry and charging rules — 0.65.0

Open **Guests [L] → Visitor flow**. Select a connected public path as the entry and enable **Toggle arrivals**. The park must also be open. One visitor considers entering every 900 simulation ticks (30 seconds at normal speed).

Use **Entry −$0.50** and **Entry +$0.50** to set a price from free to $20.00. The price is charged when a visitor actually enters. It leaves that visitor's wallet, appears as **Park entry** income in Finance, and contributes to the park's cash. An unaffordable or unattractive price can cause an arriving visitor to decline; **Turned away** counts these decisions. The willingness calculation is deterministic so a saved park continues the same way on another PC. It is deliberately a first bounded demand model, not a full scenario or marketing simulation.

**Next charging rule** cycles through:

| Rule | Entry | Ride tickets |
| --- | --- | --- |
| Flexible | May be priced | May be priced |
| Free entry | Always free | May be priced |
| Free rides | May be priced | Always free |

Changing to **Free entry** clears an existing entry price. Changing to **Free rides** clears all current ride prices and blocks paid ride prices while that rule is active. An already boarded visitor keeps the fare agreed when boarding. Switching back to Flexible does not restore old prices; set them again. The selected rule, prices, money, refused arrivals and current visitors are saved with the park. Existing version-34 parks load as Flexible with free entry; older executables cannot read version-35 saves.

New games continue to start on flat ground. For cross-PC source and save transfer, use [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md).
