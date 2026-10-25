# Ride operating costs and funds closure

Open **Rides [Y] → Operations** to see the selected ride's running rate and accumulated upkeep. Finance shows separate ride and facility upkeep; **Ride cashflows** deducts upkeep from each ride's net cash and keeps expenses after demolition.

| Ride | Bill every 30 simulation seconds | Rate per simulation minute |
| --- | ---: | ---: |
| Coaster | $1.00 | $2.00 |
| Carousel | $0.50 | $1.00 |

These are fixed costs per open ride, including empty trains and queues with no customers. Car count, lift speed and loading policy do not change the rate. **Space** pauses both activity and costs. Faster simulation advances both together.

Closing stops accrual and preserves the partial billing interval. Reopening continues that interval; closing and reopening cannot erase a pending bill. A new ride starts with a full interval before its first charge. An imported older park starts the new cost clock when it next advances, without retrospective charges.

If cash cannot cover a bill, the ride closes admission and shows **Funds closure**. The failed payment takes no money and creates no debt. Already-paid trips finish normally and guests use the existing exit. Unpaid reservations follow the normal closing/cancellation rules. A broken exit can still hold guests until repaired.

The overdue interval stays pending while closed. Recover cash through normal park income or a paid demolition refund, repair access if needed, then choose **Open ride** or **O**. Adding cash alone does not reopen the ride. The opening command refuses an overdue ride when funds are still insufficient. After a successful manual reopening, the overdue bill is paid on the next simulation tick, before departures. While paused, reopening changes the operating state but does not advance time or charge the bill.

On the same tick, ride bills are handled in stable ride-ID order, followed by facility upkeep, completed facility transactions and ride departure fares. Money earned later that tick cannot fund an earlier due bill. Leave operating float for the time guests need to walk, queue and board.

Saves use schema/content **20/20**. Older saves through version 19 migrate explicitly, preserving cash, passengers, fares, car settings, construction/refund history and existing ledger entries. Earlier game versions reject new saves. Keep your earlier builds and backup saves when moving between versions.

These costs do not add wages, loans, electricity simulation, wear, inspections or maintenance breakdowns. The broader management roadmap remains in progress.
