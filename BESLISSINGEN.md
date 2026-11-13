# Project decisions

## 2026-11-13 — Phase 9

- Use six campaign scenarios. Retain Meadow and Boardwalk and add Cedar Valley, Research Ridge, Lakeside and Grand Festival, each with distinct goals.
- Provide ten objective families. Operating-profit goals use the current lifetime operating result, including template history. Sales goals count new receipts from scenario start.
- Extend sandbox setup with budget, research availability, arrivals, loans and charging rules.
- Store portable scenarios as versioned `.parkscenario.json` documents containing validated park state, objectives and available content. Keep authoring drafts separate from test play state.
- Preserve the existing public GitHub repository and existing Drive project IDs. The registration rules for creating a new private repository apply when a new project is created.
- Record correctness evidence from this PC and GitHub checks. Exclude this PC from performance recommendations.
- The user requested the full build and codebase in Codex Cloud and explicitly approved a new cloud chat. Preserve this handoff request while completing Phase 9. Cloud provisioning requires recovery after the initial launch stalled and the command-line service reported an unavailable environment.
