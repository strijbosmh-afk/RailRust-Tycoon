# Guest ride appeal — 0.34.0

This bounded GST-01/GST-04/UI-02 milestone is **locally verified** at runtime source **`3e51f8511721f465e15c7467bb2c52d68503da82`** in [draft PR #32](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/32), based on PR #31. All 690 release tests, strict source/build checks, production continuation, authentic migration, inspected captures and fresh package checks passed; exact-source CI 40111680983 passed all 35 steps and 690 tests. Work started from 0.33 documentation checkpoint `b16d77babe9ee018c542470d11f456836e7e4ca1` on `codex/guest-ride-appeal`. [RIDE_TESTING_DISPATCH_REPORT.md](RIDE_TESTING_DISPATCH_REPORT.md) preserves the verified 0.33 runtime, unchanged archive and later CI receipt.

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
| Formatting, strict all-target release Clippy, release tests and build | Passed — **690 tests (410 game / 280 simulation)**; production build **50.96 seconds** |
| Fresh-process appeal continuation and retained proofs | Passed — **14 separate extracted-executable processes** |
| Authentic version-17 migration and unchanged-slot refusal by 0.33 | Passed — **eight processes / eight legacy slots**, 3,600 current-rule ticks per slot; four new slots unchanged after refusal |
| Minimum-window Rides/Guests capture inspection | Passed — all **13 final 1024 × 680 PNGs** directly inspected, including five independently reviewed |
| Exact-source Windows CI | Passed — run 40111680983, job 120924544421, all **35 steps / 690 tests** for runtime source `3e51f8511721f465e15c7467bb2c52d68503da82` |
| Package payload/current-guide integrity and fresh relocated execution | Passed — **49 payload hashes / 40 current guides**, fourteen proof processes and thirteen captures |
| Mechanics reports versus 0.33 | Passed — baseline/tuned reports byte-identical |
| Dependency/runtime/notices inventory versus 0.33 | Passed — **291 dependencies / five unchanged files** |
| Physical native gestures and clean-PC/offline/standard-user acceptance | Not verified |
| Representative hardware performance | Open; this development PC is not a reference |

The final local executable is **49,750,528 bytes**, SHA-256 **`58e771bd49491fe7211995ec1e82538b501cbe85cc4e09a5a899a7abcaf6f055`**. Receipts are under `artifacts/ride-appeal-0.34.0/`, including `tests-final.log`, `package-verification.log`, `source-identity.txt`, `visual-review.txt`, `notices.txt` and `mechanics-comparison.txt`. The earlier 689-test pass and five pre-wording captures are preliminary evidence and are superseded by this source and final delivery checks.

Fresh extraction to `C:\Users\strij\AppData\Local\Temp\Parkbound 0.34 appeal delivery é`, launched outside the checkout, verified every payload/current guide and all fourteen runtime proof processes. Actual reviewed/selected states drive the captures. All thirteen final PNGs were directly inspected: eight by the main reviewer and five independently. They have no clipping or overlapping text; an existing thumbnail caption wraps within a word but remains fully visible. Controlled images establish rendering, not physical input acceptance.

`migration.log` and `migration/migration.txt` record eight separate processes covering authentic schema-17 dispatch main/paid and track main/editor primary/backups: **eight legacy slots** preserve every original field and source byte through **3,600 current-rule ticks each**. The 0.33 reader refuses schema 18, with all **four new slots unchanged**. Both executable hashes remain unchanged. `mechanics-comparison.txt` separately confirms that baseline and tuned mechanics reports are byte-identical to 0.33; guest-choice changes do not alter isolated test results.

The portable archive is `dist/Parkbound-0.34.0-windows-x64.zip`. Preserve this executable while refreshing final guides, manifest and ZIP, then recheck a fresh extraction; the adjacent `.zip.sha256` identifies the mutable archive. Exact-source push [Windows CI 40111680983](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/40111680983), job **120924544421**, passed all **35 steps** and **690 tests** for head **`3e51f8511721f465e15c7467bb2c52d68503da82`**. Complete run/jobs/artifact API metadata, decoded job logs and an exact-head summary are retained in `artifacts/ride-appeal-0.34.0/ci-push-40111680983-*`. Artifact **12566403412** was identified only: no payload was downloaded or executed, and no byte/pixel equivalence with that unexamined CI artifact is claimed. The delivered executable remains the verified local binary above.

The existing 0.33 archive remains unchanged; its later CI receipt supplements its historical bundled guides. Native new controls and clean-PC/offline/standard-user acceptance remain unverified. No reference PC is selected, and local checks do not establish representative hardware performance.

## Following work

The next bounded milestone is **single-coaster car count 1–3**, with **two seats per car**, default three cars, using the existing station. The carousel retains six seats. Edits must require a closed, stationary, empty coaster and reject downsizing below its configured minimum load. Preserve passenger/fare ownership and historical six-seat receipts through an explicit migration; do not infer multiple trains, changed mass or force simulation from configurable capacity. Implementation has not started.

Wider coaster construction, multiple trains/coasters, additional ride families, staff, maintenance, research, scenarios, audio and deeper management remain unfinished. Full test/open modes, realistic forces and broader guest behavior remain separate work. Whole gameplay families remain in progress, and no reference PC is selected.
