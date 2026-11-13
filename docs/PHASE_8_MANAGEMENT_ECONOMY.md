# Phase 8 — management and economy

Version 0.87.0 introduces research, facility inventories, themed promotions, payroll limits and historic operating reports. Open **Explore → Manage → Management and economy**. New parks enable these systems automatically. An older park can opt in with **Enable management**. Saves use schema/content 50/50.

## Research

Choose a category and cycle funding through paused, $1, $2.50 and $5 per 30 simulation seconds. Each project requires $40 of paid research. The ledger records actual spending. Insufficient cash limits the next payment to affordable $0.50 steps. Changing categories preserves each category's investment. Completed categories stop charging.

| Category | Initially available | Research order |
| --- | --- | --- |
| Gentle | Festival Carousel, Meadow Wheel, Whirling Cups | Skyline Tower |
| Thrill | — | Skyward Swing, Beacon Drop, Corsair Ship |
| Water | — | Timber Falls, River Rapids |
| Transport | Park Railway | Vista Monorail |
| Coasters | Classic Coaster | Steel Launch Coaster, Inverted Coaster |
| Facilities | Food, Drink, Toilet, Rest, Information | Ice cream, Souvenir, First aid |

The catalogue displays availability. Construction quotes and placement enforce research requirements. Installed rides continue to operate when an older park enables management. Research begins paused to let the player choose how to spend starting cash.

## Supplies and merchandise

Every facility has an inventory of up to 100 consumable units. Supplies includes the items needed to provide toilets, rest, information and first aid. Stock starts empty when management is enabled. Automatic ordering is initially on and buys 20 units whenever an open facility has fewer than five. Each order requires enough cash for its entire batch. An unsuccessful order retries while automatic ordering remains enabled.

Use the **Supplies** tab to select a facility, order 20 units, change its price or open/close it. The panel shows stock, purchases, customers, sales, upkeep and operating result. Each unit is paid on delivery at the existing facility supply cost. A customer consumes one unit and pays the quoted price after service finishes. Stock shortages prevent sales. Price changes preserve a customer's locked quote.

The general ledger retains individual supply receipts and service attribution after its recent-history list rolls over. Operating results expense supplies at purchase. Demolition preserves those expenses and unused stock in the retired facility's history. The normal construction refund applies to the building. The park's book value covers installed construction assets.

Souvenirs use the same inventory and payment model. Their guest need, item price, cost and satisfaction effects remain connected to the Phase 6/7 visitor simulation.

## Marketing and pricing

The **Marketing** tab offers existing $25 flyers and $40 radio campaigns, plus $50 Family week and Coaster enthusiasts promotions. A themed promotion lasts five simulation minutes and increases the arrival cadence to one party every ten seconds. Family week requires an open gentle ride. Coaster enthusiasts requires an open coaster. One themed promotion can run at a time.

Themed promotion totals show candidates, admitted visitors, attributed entry receipts and campaign cost. Attribution covers the active promotion window and accumulates across themed campaigns. Flyer and radio statistics remain in Visitor flow. Guests still apply the normal entry-price willingness check and individual wallet limits.

The **Policies** tab offers explicit $2, $3 and $5 price presets for all installed rides. The operation checks every ride before committing. Reserved fares retain their original quote. **Entry pricing** opens existing park admission controls, including free-entry and free-ride policies. Scenario rules apply to these controls.

## Staff budgets

Set the five-role staff team's payroll cap from $0 to $20 per 30-second window in $1 steps. A worker whose payment exceeds available cash or the cap suspends work and retries. Raising the cap allows payments and work to resume. The staff panel identifies workers waiting for wages. Legacy mechanic and handyman controls retain their existing separate wages.

The cap limits actual team wage payments. It is independent of research, supply ordering, ride upkeep and legacy staff wages. Closing a facility stops its automatic orders and upkeep. Use staffing controls to dismiss workers or adjust patrols when reducing costs.

## Management reports

**Overview** shows cash, sales, operating expenses, operating result, installed park value, debt, research and promotion spending. It also surfaces low cash, loss-making periods, stock shortages, unpaid staff and disconnected visitor entry, with the relevant management action.

The displayed indicators use explicit definitions.

- Guest satisfaction is the mean satisfaction of active guests, with 50 shown when the park is empty.
- Popularity is lifetime admissions divided by admission candidates, with 50 shown before any candidate arrives.
- Reliability is the share of installed ride records below the maintenance shutdown threshold, with 100 shown when there are no rides.

**History** retains 60 account snapshots, recorded once per simulation minute. Each displayed period is the difference between two snapshots. Operating expenses include upkeep, delivered supplies, staff wages, research and marketing. Construction, demolition refunds, borrowing, repayments and paid interest appear separately in Finance accounts. The ledger and reports reconcile to cash using integer cents.

**Finance accounts** retains per-ride and facility attribution, assets, loan limits, principal, accrued/paid interest and recent receipts. Research and themed promotions have their own receipt types and totals. Old receipts remain represented in archived totals and owner accounts.

## Original scenarios

Meadow Challenge keeps free entry, its existing flyer policy, a maximum research rate of $2.50 per 30 seconds and no themed promotions. Starting Meadow from a managed sandbox reduces a higher research rate to the permitted level. Active radio or themed promotions must finish before starting Meadow. Boardwalk Sprint keeps its free-ride rule and deadline. Both scenarios retain their existing objective and save behavior.

## Save continuation and verification

Schema/content 50 adds an optional management account. Readers retain support for schemas 1–49. Older parks preserve their prior research and per-service supply behavior until management is enabled. New management state is rejected under older schema headers.

Validation checks research investment against its ledger, promotion spending and attribution bounds, stock against purchases and completed service, budget ranges, bounded ordered history and cash reconciliation. Reports, research progress, inventory, marketing timers and payroll windows resume from their saved state.

The native `write-management` / `verify-management` proof uses a mixed park and checks a fresh-process continuation against expected future bytes. The proof also checks the independently reconstructed finance buckets. Unit tests cover normal construction locks, paused research, empty stock, paid consumption, automatic ordering, declined orders, demolition, old headers, corrupted stock, scenario policies and suspended staff.

Development-PC timings are diagnostic information. Representative hardware targets and wider play-balance evaluation belong to Phase 11.

## Next phase

Phase 9 extends the campaign and sandbox, adds original scenario authoring/import/export, and improves the scenario and design library workflow. Its scope and acceptance gate are in [the full-game roadmap](FULL_GAME_ROADMAP.md).
