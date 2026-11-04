# Mechanic and wages — 0.56.0

Open Rides [Y] → Operations and click **Hire mechanic**. One park-wide mechanic can be hired. The button changes to **Dismiss mechanic** while employed. Hiring and dismissal are free; the mechanic earns **$1.00 every 30 simulation seconds** ($2.00 per minute) while employed. Wage charges appear in Finance [N] and reduce cash and operating profit. If the park cannot pay a wage when it is due, the mechanic is dismissed automatically and no debt is created.

When a ride is due for inspection, close it and let passengers finish and exit. A hired mechanic services an eligible stopped ride on the next simulation tick. This resets its completed-trip count since service; reopen the ride after the usual checks. The manual **Service ride** button remains available. A maintenance shutdown still blocks reopening until service is complete.

The mechanic is a park-wide management role in this stage. There is no visible walking staff member, patrol area, service-access route or travel delay yet. Save the park to preserve hiring, wage progress, wage receipts and ride service state. These are planned steps toward physical staff operations.
