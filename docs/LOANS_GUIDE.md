# Loans, repayments and interest

Open **Finance [N]** and cycle **Ride cashflows → Park summary → Loans**. Use **Borrow $100**, **Repay $100** or **Pay interest**. After Loans, choose **Park value** for [explained assets and liabilities](PARK_VALUE_GUIDE.md), then Transactions. The loan limit is **$2,000 outstanding principal**; a new park starts with no loan. Actions work while paused and never advance time. Closing Finance or pressing conflicting loan buttons cannot also make a loan transaction.

Borrowing adds cash and matching debt. Principal repayment removes cash and debt. Neither changes ride/service operating profit or customer sales. The park summary shows gross borrowing, repayments and paid interest separately:

**Net cash = operating profit + refunds − construction + borrowing − repayments − paid interest.**

Starting cash plus net cash equals current cash. Unpaid interest is a liability, not money already spent; only paid interest reduces cash. These are lifetime recorded totals, including older archived transactions and removed rides/facilities.

Interest is **0.5% of outstanding principal per simulation minute**, or $0.50 per $100 borrowed per minute. It is simple interest, not an annual rate. Each tick accrues a prorated amount. Sub-cent fractions carry forward through repayment, pausing and save/load, including after principal reaches zero. Borrowing or repaying shortly before a collection cannot erase accrued interest.

Collections happen at the park's fixed 60-second simulation boundaries, before ride upkeep, facility costs and new customer income. A first collection can occur less than a minute after borrowing, for only the prorated amount. **Space** pauses accrual and activity; fast-forward accelerates both.

If cash is insufficient, a collection uses available cash and retains the unpaid balance as **Still due**. Interest does not compound and the game does not automatically borrow. **Pay interest** also pays as much as current cash allows. **Repay $100** requires cash for the full $100 principal payment **and all accrued whole-cent interest**; otherwise it refuses the entire transaction. Keep cash for running costs. Borrowing alone does not reopen rides or facilities closed for funds.

Schema/content **21/21** explicitly migrates saves through version 20 with zero loan balance and no invented financing or interest. Cash, earlier receipts, ongoing journeys and cost clocks remain preserved. Older builds reject new saves, so retain earlier builds/backups when upgrading. Scenario-specific credit limits, bankruptcy, collateral, wages, staff and maintenance remain later milestones.
