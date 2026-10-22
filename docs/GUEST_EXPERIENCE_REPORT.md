# Guest preferences, satisfaction and ride ratings — 0.18.0

Updated 22 October 2026. This milestone gives each guest a different ticket-price limit and queue comfort, explains their decisions, and records satisfaction and ride reviews after completed public returns. Work starts from `f62f209a6848b68ed0ef76ba011f74cf6887dc92` on `codex/guest-experience`, stacked on `codex/ride-economy` / [draft PR #15](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/15). Runtime `efda12f22c292cc43dc21ba8c3246e4f77882b62` is pushed in [draft PR #16](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/16). The bounded implementation is verified through source tests, production migration/continuation, inspected rendering, exact-source CI and relocated packaging. New native guest-panel gestures remain unverified after interrupted input. Historical money-flow evidence remains in [ECONOMY_REPORT.md](ECONOMY_REPORT.md).

## Player behavior

Each guest has a fixed willingness to pay of **$4, $6, $8 or $10**, and queue comfort of **15, 25, 35 or 45 seconds**. These traits are derived from the saved crowd seed and stable guest ID without consuming the crowd's random stream. Price comfort is separate from money: a guest can have enough cash but consider the ticket poor value. The ride panel's interested count requires both money and willingness; it does not promise a free seat or connected route.

Guests check their wallet first, then perceived value, when seeking/admitting and again at the queue front. A front guest who refuses a changed price triggers the existing safe tail-first queue return. Nobody overtakes or teleports. A fare already locked for a boarding passenger remains unchanged. Refusals produce distinct thoughts and counters, use the existing retry cooldown, and do not repeatedly drain satisfaction.

Guests begin at **70/100 satisfaction**. Crossing their queue comfort once costs five points and produces a long-wait thought. A joined-queue or predeparture passenger cancellation costs eight points once and gives no review. The existing whole-line 60-second patience limit remains; personal queue comfort does not replace it. Thoughts describe meaningful events, so their age measures time since the event rather than a label refreshed every tick.

A review is created only when a guest finishes a real lap and physically returns through the exit to the public path. Boarding, departure, arrival at the station, missing-exit holds, cancelled trips and loading a save do not themselves create reviews. A returning guest may rate each completed trip; the ride average covers all reviews, not only each guest's latest one.

The integer review formula starts at five stars. Subtract one if the locked fare exceeds half the guest's price limit, or two if it exceeds the whole limit (possible with a preserved legacy reservation). Subtract `min(2, waited_ticks / comfortable_queue_ticks)` for queue experience, then clamp to 1–5. Completed-return satisfaction becomes `(3 * previous_satisfaction + stars * 20) / 4`, using integer division. Preferences stay fixed in this stage; satisfaction does not secretly change the stated price limit.

Use **L or Guests** to open the guest list. The panel shows eight stable-ID-sorted guests per page, with wallet/spending, price limit, queue comfort, satisfaction, activity, latest thought and its age, completed trips, and latest review/count. Click a guest or use Up/Down; Page Up/Down changes page. L or Escape closes the panel. Help and Guests are mutually exclusive. The modal suppresses construction, price/ride edits and save/load controls behind it, including its opening/closing update; Space can still pause while Guests is open. The compact ride title shows an average or Unrated. Guests are inspected through the list; world meshes remain unpickable.

Normal play retains twelve guests, the 32 × 32 park, one fixed circuit and six seats. Queue/entrance/exit construction remains necessary. This milestone adds no ride intensity/physics preferences, guest needs, shops, guest arrivals/departures, multiple rides, construction expenses or track builder. This PC is excluded from performance acceptance, and no local performance batch is required.

## Authoritative data and invariants

| State | Contract |
| --- | --- |
| `Guest.preferences` | Required price-limit and queue-comfort values from the supported discrete sets. |
| `Guest.satisfaction` | Integer 0–100. |
| `Guest.thought`, `thought_tick` | Saved event and simulation timestamp, never in the future. |
| `Guest.ratings_given`, `rating_total`, `last_rating` | Lifetime count/sum and nullable latest 1–5-star review. Historical unrated completions are permitted. |
| `Ride.value_rejections` | Refused attempts due to perceived value, distinct from insufficient funds. |
| `Ride.rating_count`, `rating_total` | Sum of guest review counts and sums for the current single-ride model. |
| `Park::ride_experience_stats` | Read-only willing/interested/total counts, value rejections, review count/sum and nullable average in tenths. |
| `Park::guest_experience_summary` | Read-only average satisfaction, count below 40 and total guests. An empty crowd has no average. |

Every guest review count is at most that guest's completed trips. A zero count has sum zero and no latest review; nonzero counts require a latest review and a valid sum, including the possible remaining history after subtracting the latest score. Guest counts/sums reconcile with the ride. Counts are capped at `u64::MAX / 5`, with one future-review slot reserved per active passenger; an exhausted count prevents new boarding, while existing reserved trips can finish safely. Displayed averages use widened arithmetic. Money invariants from the economy milestone remain intact; a review transfers no currency.

## Strict save version 7 and migration

Version 0.18 writes **schema 7/content 7**. All experience fields are required, including explicit nullable `last_rating`; unknown fields, invalid discrete preferences, out-of-range satisfaction/stars, future thought timestamps and inconsistent review totals are rejected before replacing the park. The existing size/map caps remain.

Versions **1/1 through 6/6** migrate explicitly. Existing fields, IDs, allocators, RNG state, currency and active journeys remain authoritative. New preferences derive from seed/ID, satisfaction begins at 70, and review counts/sums begin at zero with no latest review. Historical completed trips stay unrated. An existing locked legacy fare is preserved; that active trip can receive its first prospective review on a future public return.

New preferences intentionally affect future choices after migration. Compatibility means preserving the loaded historical state and deterministic continuation under the new rules, not reproducing every future decision of the old executable. Loading never rewrites either slot. Explicit Save writes 7/7, which 0.17 intentionally rejects; retain closed-game copies for rollback. Player saves remain in `%LOCALAPPDATA%\Parkbound\saves`, outside Git and the source folder.

## Reproduction

Run production proofs in fresh absolute temporary directories, never in player saves. `write-feedback` / `verify-feedback` use natural queue/passenger/review states and **18,000 complete continuation ticks**. The proof retains the independent per-guest payment checks and adds review-event checks: ratings may change only on a completed physical public return, preferences remain fixed, and aggregate reviews reconcile. It exercises price-value rejection, long waiting, satisfaction, cancellation without a review and recovery with another completed review. It loads primary/backup through the production controller paused with Inspect reset and preserves slot bytes.

```powershell
$exe = (Resolve-Path '.\target\release\parkbound.exe').Path
$proof = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound feedback proof é ' + [guid]::NewGuid())
foreach ($mode in @('write-feedback', 'verify-feedback')) {
    $arguments = '--save-check {0} --save-dir "{1}"' -f $mode, $proof
    $process = Start-Process -FilePath $exe -ArgumentList $arguments -WindowStyle Hidden -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw "Feedback proof failed: $mode" }
}
Get-Content -LiteralPath (Join-Path $proof 'verification-feedback.txt')
```

For authentic older saves, first use a retained older executable's writer in another fresh directory, then run current `--save-check verify-legacy --save-dir <absolute-directory>`. This checks every original raw park field against the migrated park, validates only the added experience defaults, and compares two current-version command continuations with save round-trips. It does not compare against an old executable's expected future or disable new preferences. The primary and backup remain unchanged. The receipt is `verification-legacy.txt`.

Six diagnostic scenes use real guest states: `feedback-value`, `feedback-queue`, `feedback-satisfied`, `feedback-cancelled`, `feedback-recovered`, and `feedback-guests`. Existing rotation, terrain, surface/underground and minimum-window options remain. All six open the actual Guests panel with the relevant guest selected; `feedback-guests` exercises the second list page. Example:

```powershell
& $exe --capture-frame (Join-Path $env:TEMP 'parkbound-feedback-guests.png') --capture-scene feedback-guests --capture-small-window
```

Captures establish presentation, tests establish input ordering, and native interaction establishes real keyboard/mouse behavior. They are separate claims. CI retains prior continuation modes and adds feedback continuation; it does not establish representative hardware performance.

## Verification record

Runtime source **`efda12f22c292cc43dc21ba8c3246e4f77882b62`** is pushed on `codex/guest-experience` in [draft PR #16](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/16), stacked on unmerged `codex/ride-economy` / PR #15. All **367 release tests (230 game / 137 simulation)** passed. The last-review regression was strengthened with a valid matching-aggregate control, followed by another passing 137-test simulation run; production source did not change. Formatting, final strict all-target release Clippy and production build passed. The executable is **47,141,888 bytes**, SHA-256 **`b8c05e05dbab244ef0e1ca1d147e660fd1314c7bc04c70de4b59f6807e17a98c`**. [Exact-source push CI 39753833991](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39753833991) passed all **22 steps**, including tests, packaging, retained/new continuation proofs, reporting smoke and uploads. [PR CI 39753861050](https://github.com/strijbosmh-afk/RailRust-Tycoon/actions/runs/39753861050) also passed for that exact source. Later documentation commits retain this runtime identity and need their own CI status.

All **47 production process checks passed**. Authentic schemas 1–6 were written by the retained 0.12–0.17 binaries; additional authentic occupied v5 journey and paid/reserved v6 economy saves were included. The new reader preserves every original primary/backup park field, initializes experience without historical reviews, and runs 3,600 current-version ticks per slot with commands and repeated round-trips. Both original files remain unchanged. Version 0.17 rejects v7 without changing either slot. All retained construction/admission/journey/economy proofs pass, and the new feedback proof reproduces **18,000 complete ticks**, checking actual public-return review deltas, frozen fare/wait scoring, satisfaction, stable traits, cancellation without review, value refusal and recovery. Its backup restores real queue discomfort paused; its primary restores a reviewed park paused/Custom/Inspect with the guest panel reset.

All **twelve production PNGs were visually inspected**: eight feedback variants plus occupied journey, entrance preview, help and overview. Four rotations, Flat/Terraced, Surface/Underground and normal/minimum windows are represented. The Guests panel shows readable traits, money, activity, event age, satisfaction and reviews on both pages; blank final-page rows cannot select a guest. Queue discomfort shows 65/100 without a review; joined cancellation shows 62/100 without a completion/review; completed return shows 77/100 and 5/5; recovery shows six reviews and a 4.8/5 ride average. Help and construction controls still fit at 1024 × 680. Pictures establish presentation, not native gestures or continuous motion.

The portable **0.18 package verifies 33 payload hashes and 24 current Markdown guides**. All 291 dependency entries and the five dependency/native/notices/runtime files remain byte-identical to 0.17. A fresh extraction under a spaces/é path, launched from `C:\Windows`, passed the 18,000-tick feedback proof and produced a byte-identical minimum-window Guests capture. Guide/manifest/ZIP checks are refreshed after the final documentation update; the adjacent `.zip.sha256` is the final archive identity. Same-PC relocation does not establish clean-machine compatibility.

**New native controls remain unverified.** The extracted 0.18 QA instance opened with isolated temporary copied slots. The first F9 attempt reported detected desktop input; the subsequent observation did not show the game. All automated input stopped, and no load or new control outcome was inferred. The QA window was left open. L/Guests, row selection, paging, closing and native modal save blocking therefore have no current native pass; source input regressions and captures passed independently. Prior native results remain historical, including the unfinished 0.17 price gestures. Default player saves were never accessed.

Raw receipts are in `artifacts/guest-experience-0.18.0/`: `tests.txt`, `sim-tests-final.txt`, `clippy-final.txt`, `build-final.txt`, `runtime-verification.txt`, `capture-inspection.txt`, `capture-hashes.txt`, `captures/`, `native-instance.json`, `native-verification.txt`, `notices-comparison.txt`, `relocated-runtime-verification.txt`, `ci-jobs.json`, and package receipts. This ignored directory requires OneDrive or explicit transfer to another PC; it is not in Git. `final-handoff.txt` records exact runtime identity, final documentation HEAD, package checks and remote/local equality after the final push. No local performance batch was run; this PC remains excluded. Native, clean-PC/offline/standard-user, signing/licensing/title and representative-hardware release gates remain open.

Raw local receipts belong in `artifacts/guest-experience-0.18.0/`, ignored by Git. The Windows package is `dist\Parkbound-0.18.0-windows-x64.zip`; its adjacent `.zip.sha256` identifies the final archive after guides are refreshed. Preserve all historical reports and their original evidence. Same-PC relocation does not establish clean-PC, offline or standard-user compatibility.

Shared source checkout: `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. Separate second-PC checkout: `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`. Follow [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md) for one-editor-at-a-time OneDrive handoff and separate save transfer.

The next cohesive milestone is guest needs and reachable services, with understandable thoughts, deterministic recovery and explicit extensions to the currency/save invariants. Multiple rides, track construction and broader park management follow. The full feature families and distribution gates remain open.
