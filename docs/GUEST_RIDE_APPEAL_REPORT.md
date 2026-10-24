# Guest ride appeal — 0.34.0

This bounded GST-01/GST-04/UI-02 milestone starts from the 0.33 documentation checkpoint `b16d77babe9ee018c542470d11f456836e7e4ca1` on `codex/guest-ride-appeal`. The preceding runtime source `b4523bb8eab5365947759a6643241a3da83400a4` / PR #31 passed all 674 local tests and all 34 exact-source Windows CI steps. [RIDE_TESTING_DISPATCH_REPORT.md](RIDE_TESTING_DISPATCH_REPORT.md) preserves its executable, package and acceptance scope. The 0.34 implementation is in progress; no new verification or delivery is claimed yet.

## Fresh guest choices

Guests considering a new ride now weigh the installed ride's challenge and cumulative reviews alongside the existing distance, queue, ticket, ride-family and repeat-visit factors. Challenge is a soft preference: exceeding a guest's preference makes that ride less attractive but never creates an admission ban. Existing access, open-state, wallet/value and capacity checks keep their authority. Stable ride identity still breaks ties.

A guest's preferred challenge is derived deterministically from the existing crowd seed and guest identity: 25, 45, 65 or 85 out of 100. Coaster challenge is a bounded static layout heuristic based on the longest continuous drop and curve density; carousel challenge is 20. It uses installed specifications only. Equipment/loading settings, unapplied drafts and running a mechanics test do not change this profile. Deriving it never advances time, samples a lap, changes random state or mutates the park.

Choice reputation starts with five neutral three-star prior reviews, then incorporates all actual completed-trip review totals. It uses cumulative history, without recency decay. This keeps an unrated ride neutral and limits the influence of a single opinion. The actual average and actual review count remain separately available and displayed; the prior does not invent guest reviews or alter saved totals.

Only a fresh choice by an idle guest uses these factors. Existing ride or service visitors retain their target, queue place, reservations and locked fare. A profile shown later describes the current selected ride and guest preference; it does not reconstruct why an already-active visit was chosen or predict the guest's next selection. Test-tab excitement/intensity/nausea estimates remain separate mechanics/layout output and are not directly used as demand scores. This milestone adds no forces, safety claims or population demand percentage.

## Player presentation

The existing Rides details keep the actual review average/count and add **Current ride challenge** and **Choice reputation**, with a neutral-start explanation. Guests shows **Preferred challenge (soft preference)** and the selected ride's current challenge, within/above-preference fit and reputation. Affordability/value checks stay separate. Unavailable or removed rides clear current-fit data instead of keeping another ride's values. No raw ranking penalties are exposed.

Rides remains 820 × 540; Guests remains 760 × 520, within the 1024 × 680 minimum window. The Test dashboard remains separate. The loading hint is clarified to **0 means no minimum**. These panels read authoritative profiles and never commit park edits.

## Save and compatibility contract

New saves use strict schema/content **18/18** because future guest-choice rules have changed. The serialized park body is identical to version 17: no appeal profiles, derived challenge preferences or cached test results are stored. The exact version-17 reader preserves every original field, including active visits, fares, dispatch policies and elapsed extra waits. Existing version-1–16 readers remain supported. Loading alone never rewrites a source slot.

Version 0.33 and earlier reject schema 18; retain legacy copies before saving with 0.34. Migration verifies preservation and deterministic continuation under current rules, rather than equality with an older build's future choices. Map/save caps and design/catalog format version 1 remain unchanged. No new dependency or external asset is added.

## Verification

| Check | Current result |
| --- | --- |
| Formatting, strict all-target release Clippy, release tests and build | Pending |
| Fresh-process appeal continuation and retained proofs | Pending |
| Authentic version-17 migration and unchanged-slot refusal by 0.33 | Pending |
| Minimum-window Rides/Guests capture inspection | Pending |
| Exact-source Windows CI | Pending |
| Package payload/current-guide integrity and fresh relocated execution | Pending |
| Physical native gestures and clean-PC/offline/standard-user acceptance | Not verified |
| Representative hardware performance | Open; this development PC is not a reference |

Controlled fixtures must use actual reviews and real selected guests, preserve source slots, and distinguish a current explanatory profile from a controlled comparison of fresh choice outcomes. Record exact source/executable identities and scoped receipts before marking any check passed. The existing 0.33 archive remains unchanged; its later CI receipt supplements its historical bundled guides.

## Following work

Wider coaster construction, configurable train composition, additional ride families, staff, maintenance, research, scenarios, audio and deeper park management remain unfinished. Full test/open modes, realistic forces and broader guest needs/behavior remain separate work. Whole gameplay families remain in progress, and no reference PC is selected.
