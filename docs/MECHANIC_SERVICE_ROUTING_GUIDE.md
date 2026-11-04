# Mechanic travel and service access — 0.57.0

Hire the park mechanic in **Rides [Y] → Operations**. The mechanic still costs **$1.00 every 900 simulation ticks** ($2.00 per simulation minute). You can see the teal uniform and orange tool case on a public path once a park entry path is set in **Guests [L]**.

For automatic service, connect a continuous **public path** from the chosen park entry to the ride's **exit**. Queue paths do not count. Once an inspection-due ride is closed, stopped and empty, the mechanic walks along that route, services it from the exit landing and returns toward the entry. Each path step takes 30 simulation ticks. The Operations notice shows whether the mechanic is waiting for an entry, available or travelling to a ride. Reopen the ride after service using the ordinary access and funds checks.

Removing a path can interrupt the assignment. Reconnect the public route and the mechanic tries again. If the path beneath the mechanic is removed, the worker returns to the valid entry on the next simulation tick. The existing **Service ride** button remains a direct player action and does not wait for the worker. Saving preserves the mechanic's current path, next path, partial step and target ride, as well as wages and ride maintenance.

This stage has one mechanic. Guests and the mechanic can share paths; there are no patrol zones, collision reservations, timed repair animations, handymen or breakdown types yet.
