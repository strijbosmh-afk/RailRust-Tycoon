# Visitor trends and guest triage

Open **Guests [L]**. The **Guest list** shows eight visitors per page, ordered by their stable guest number. Click the **Filter** button repeatedly to cycle through **All**, **Urgent**, **Unhappy**, and **Lost**. The line above the list shows how many visitors match out of all active guests. Empty results leave the detail area unselected. Page buttons and **PgUp/PgDn** browse the current filtered set; selecting a visitor and using **Find** still locates that visitor in the park. Filtering only changes what the panel shows.

**Urgent** means at least one of Hunger, Thirst or Toilet is 70 or higher. **Unhappy** means satisfaction is below 40. **Lost** follows the visitor's live lost state. The categories may overlap and update as the park runs. Select **All** to return to the complete list.

Choose **Park stats** for the park-wide condition. The green bars in **Visitor trend** show average satisfaction out of 100. Red bars show the percentage of active visitors with at least one urgent need. A pair of bars is sampled immediately and then every 30 seconds of simulated park time. At most 24 samples appear, with the newest at the right. **Space** pauses sampling with the park. Loading another park starts a fresh chart; the history is deliberately a session view and is not stored in a park save. The snapshot and list always show current values even between chart samples.

The park save format remains **24/24**. For the shared OneDrive checkout and other-PC setup, see [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md). Implementation and evidence are recorded in [VISITOR_TRENDS_TRIAGE_REPORT.md](VISITOR_TRENDS_TRIAGE_REPORT.md).
