# Ticket prices, guest wallets and ride revenue — 0.17.0

Updated 22 October 2026. This stage adds the first money flow to the existing complete passenger journey: an editable ticket price, guest wallets, a locked boarding fare, payment at departure, earned ride revenue and park cash. Work starts at `20491b13d5e49a3b8b707f53fec789ddf97694cd` on `codex/ride-economy`, based on unmerged [PR #14](https://github.com/strijbosmh-afk/RailRust-Tycoon/pull/14). The implementation checkpoint, new draft PR and verification evidence are pending. Historical journey results remain in [RIDE_JOURNEY_REPORT.md](RIDE_JOURNEY_REPORT.md).

## Player behavior

New parks give each guest **$20.00** and the park **$1,000.00**. Rides begin with a free ticket, including migrated parks. Use the Ticket −/+ buttons or **F6/F7** to change the fare by $0.50 within $0.00–$100.00. The header shows park cash, ride revenue and guest wallets; ride feedback explains affordability. Dollars are the game's currency, not a real-money feature.

Guests must afford the current ticket to seek/join the ride, and the queue front is checked again before boarding. A price rise can therefore prevent a guest from boarding. An unaffordable applicant pays nothing and uses the normal retry cooldown. If the front of the single-file queue becomes unable to pay, the queue returns safely using the existing cancellation rules; guests do not pass through each other.

At the start of boarding, the guest reserves a seat and **locks the current fare**. Their wallet still owns that money. The first actual train departure transfers the locked amount once from their wallet into their spending total, ride revenue and park cash. A later price change applies to later boarders, including guests already waiting in the queue, but never reprices a locked passenger.

Closing before departure costs the guest nothing. There is no debit to refund. Closing during the lap lets the train finish and keeps the departed fare earned. A missing exit can hold passengers for repair without changing their payment. Free trips earn zero. Loading a reserved boarding transfer or a paid moving train preserves the exact payment state and must not charge twice. Completed-trip counts still require a completed lap followed by physical return to a public path; revenue and completed-trip counts measure different events.

Normal play retains the 32 × 32 demonstration, twelve guests, one fixed three-car circuit, six seats, and player-built queue/entrance/exit connections. Startup has no queue or gates. The connected fixture and world coordinates are in [RESUME_DEVELOPMENT.md](../RESUME_DEVELOPMENT.md#5-controls-and-reproducible-ride-construction). Pricing does not build those connections. The train can still run empty, and without an exit the historical queue/patience behavior remains available.

This stage does not implement construction costs, refunds, operating expenses, park admission, shops, loans, staff wages, a finance calendar, guest arrivals/departures or replenishing wallets. It does not complete the whole management game. The development PC is explicitly excluded as a performance reference; no local performance batch is planned.

## Authoritative model

All money uses integer cents. There are no floating-point prices or wall-clock payment events.

| State/API | Contract |
| --- | --- |
| `Guest.wallet_cents: u32` | Remaining guest-owned funds, initially 2,000. Uncharged locked fares remain included. |
| `Guest.spent_cents: u64` | Lifetime fares paid, initially zero. |
| `Ride.ticket_price_cents: u32` | Current fare, initially zero, maximum 10,000. |
| `Ride.revenue_cents: u64` | Actual departure payments, initially zero. No expenses are deducted. |
| `Ride.unaffordable_rejections: u64` | Rejected attempts, not a count of distinct people or money. |
| `PassengerJourney.fare_cents: u32` | Fare locked when boarding starts. |
| `PassengerJourney.fare_paid: bool` | Exactly-once settlement flag, matching the passenger's departed flag. |
| `Park::cash_cents() -> u64` | Initial 100,000 plus earned ride revenue. |
| `set_ride_ticket_price(RideId, u32)` | Immediate validated command; a rejected unknown ride or excessive price leaves the park unchanged. Other live state is preserved. |
| `ride_economy_stats(RideId)` | Current price/revenue/rejection count, guest and affordable counts, and reserved fare total. Stats are derived without mutating the park. |

`INITIAL_GUEST_WALLET_CENTS`, `INITIAL_PARK_CASH_CENTS`, `MAX_TICKET_PRICE_CENTS` and `PRICE_STEP_CENTS` expose the defaults/bounds. A wallet can cover only one journey because guests cannot hold multiple visits. `affordable_guests` counts all current wallets that cover the current price, including existing passengers; it is not a promise that those guests can board now. `reserved_fares_cents` is still guest-owned money and must never be added to revenue or park cash.

The current single-ride model enforces:

```text
guest.wallet_cents + guest.spent_cents = 2,000
sum(guest.spent_cents) = ride.revenue_cents
park.cash_cents = 100,000 + ride.revenue_cents
unpaid locked fare <= guest.wallet_cents
passenger.fare_paid = passenger.departed
```

Repair/restoration preserves existing guest and ride identities and their money. Diagnostic fixtures begin with a fresh Park. Future guest lifecycle, multiple rides, shops or expense systems must explicitly extend the accounting invariants before changing them. Do not saturate, silently discard, or mint currency to hide an accounting error.

## Inputs, persistence and compatibility

Price commands work while paused without moving guests or train state. Help, capture/diagnostic input gates and pending Load/Backup suppress conflicting edits. A successful or failed Load/Backup wins over a same-update price edit; a same-update Save includes an accepted price edit. A successful load resets transient price input/feedback along with other tool state and pauses simulation. The compact ride/build/help layout must remain usable at the 1024 × 680 minimum.

Version 0.17 writes strict **schema 6/content 6**. Guest wallets/spending, ride prices/revenue/rejections, park cash and passenger locked fare/paid flags are required fields. Validation checks both existing geometry/journey invariants and currency conservation before replacing a live park. Negative/fractional values, unknown fields, inconsistent balances, mismatched departure/payment flags, excessive prices and hybrid version shapes are invalid. Before a guest's first completed public return, their spending must equal the active paid fare, or zero without an active paid passenger. This rejects fabricated historical spending even when aggregate balances agree; completed legacy free trips retain zero spending.

Strict versions **1/1 through 5/5** migrate explicitly. Each legacy guest receives $20.00 and zero spending; cash becomes $1,000.00; the ride price/revenue/rejections become zero. Existing v5 passengers receive a zero locked fare and a paid flag matching departure. Their old journey/completion history remains intact, without retroactive revenue or payments. Versions 1–4 keep their previously defined geometry/admission migrations before adding the economy defaults. No migration draws new RNG values or adds gates/passengers.

Loading does not rewrite either file. Explicit Save writes 6/6, which 0.16 intentionally rejects. Preserve closed-game copies when rolling back. Source save limits and the app's 32 × 32 map limit remain unchanged. Default saves remain outside OneDrive at `%LOCALAPPDATA%\Parkbound\saves`; transfer both primary and backup separately when switching PCs.

## Reproduction and evidence

The new production proof modes are `write-economy` and `verify-economy`. Use a fresh absolute directory and separate processes; never use the player's save directory. The writer refuses an existing target. It uses the same real queue/entrance/exit fixture as the journey proof, with no injected passenger or money state. It saves a naturally reserved **$3.00 boarding transfer** as backup, changes the current price to $5.00 without repricing that reservation, and saves an actually paid moving train as primary. The original passenger must have paid the locked $3.00, not the later price. The primary also has Custom terrain so loading must restore paused/Custom/Inspect presentation.

Continuation runs exactly **10,800 complete ticks**, with bounded natural-state stages:

| Stage | Required observation |
| --- | --- |
| Existing paid trip | Change price to $7.00 without changing any guest/locked fare, then close during motion and require completed public returns. |
| Predeparture cancellation | Reopen at $4.00 and reach an actual unpaid boarding transfer. Change the displayed price to $6.00, close before departure, and require physical public return with identical wallets/spending/revenue/cash and no new completed trip. |
| Insufficient funds | Open at $100.00, require a real unaffordable rejection, and verify no occupied seat or money transfer. |
| Affordable recovery | Reduce price to $2.00 and require another paid departure and completed public return. |

Every tick independently compares each guest's before/after payment state. A debit and spending increase must equal the previously locked fare only when the same ride/ticket remains Riding, its unpaid passenger first becomes departed/paid, and the train changes from zero to positive speed; otherwise both deltas must be zero. Existing tickets cannot change their locked fare or revert from paid to unpaid. Free fares still require the actual departure transition, with zero money transferred. Cash and revenue deltas must equal those individual payments. This catches repeated matching debits/credits that conservation alone would miss. All stages also check full balances, passenger roles/seats, guest/ride identities and save round-trips. The verifier loads the exact primary and reproduces the complete final Park, then restores the exact boarding backup paused/Inspect and separately cancels it without a charge. Verification leaves both slot files unchanged. The Windows workflow invokes the new two-process proof alongside all retained continuation modes; actual results are pending.

```powershell
$exe = (Resolve-Path '.\target\release\parkbound.exe').Path
$proof = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound economy proof é ' + [guid]::NewGuid())
foreach ($mode in @('write-economy', 'verify-economy')) {
    $arguments = '--save-check {0} --save-dir "{1}"' -f $mode, $proof
    $process = Start-Process -FilePath $exe -ArgumentList $arguments -WindowStyle Hidden -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw "Economy proof failed: $mode" }
}
Get-Content -LiteralPath (Join-Path $proof 'verification-economy.txt')
```

The six production capture scenes are `economy-reserved`, `economy-price-change`, `economy-earned`, `economy-cancelled`, `economy-unaffordable` and `economy-free`. Each seek is bounded at 18,000 extra ticks. They use actual simulation states rather than assigning fictitious cash or passenger state: a $3.00 boarding transfer, that same transfer with a new $7.00 price, paid passengers moving on the lift, uncharged cancellation, $100.00 rejection, and a completed free trip respectively. Their tick helper checks the same independent payment deltas and conservation. Presentation changes across rotations/views must preserve both full Park and serialized bytes. All retained camera/terrain/view/minimum-window options remain. Still images establish rendered feedback, not native gestures or performance.

```powershell
$exe = (Resolve-Path '.\target\release\parkbound.exe').Path
$images = Join-Path ([IO.Path]::GetTempPath()) ('Parkbound economy images ' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $images | Out-Null
& $exe --capture-frame (Join-Path $images 'reserved.png') --capture-scene economy-reserved --capture-rotation 0 --capture-map terraced --capture-view surface
& $exe --capture-frame (Join-Path $images 'earned-small.png') --capture-scene economy-earned --capture-rotation 2 --capture-view underground --capture-small-window
```

| Gate | Current evidence |
| --- | --- |
| Exact source, draft PR and Windows CI | Pending. |
| Release tests, formatting, strict lint and production build | Pending. |
| Authentic legacy migration, reverse rejection and production continuation | Pending. |
| Inspected captures and actual native price/save/journey controls | Pending. |
| Executable identity, notices, package manifests/guides and relocation | Pending. |

Raw receipts will be retained under `artifacts/ride-economy-0.17.0/`. This ignored folder is local/shared-OneDrive evidence, not part of Git. The archive is `dist\Parkbound-0.17.0-windows-x64.zip`; its adjacent `.zip.sha256` is authoritative after guide refreshes. The resume guide records both PC paths, source branch and setup. Exact runtime evidence must stay distinct from later documentation commits.

Shared source checkout: `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. Separate second-PC checkout: `C:\Users\michiel strijbos\Documents\ChatGPT\RailRust Tycoon`. Do not edit the shared checkout simultaneously from both PCs. Player saves in LocalAppData require separate transfer; they are not included in Git or the ZIP.

Unverified native construction/demolition, terrain shortcuts, pan/drag/DPI, clean-PC offline/standard-user behavior, signing/licensing/title decisions and representative hardware remain explicit release work. Same-PC relocation does not prove clean-PC compatibility. Do not infer a pass from a historical binary, a successful build, or the presence of a screenshot file.

The next gameplay stage should give guests preferences and satisfaction so pricing becomes a demand decision: willingness to pay, queue/ride experience, guest thoughts and ride ratings. Keep actual wallet affordability separate from perceived value. Needs and services, multiple rides and a general track builder follow in further substantial stages.
