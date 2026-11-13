# Phase 7 — visitors, staff, weather and park life

Version 0.86.0 adds park-life simulation to newly created parks. Existing saves preserve their original simulation until **Explore → Manage → Staff and park life → Enable park life** is used. Saves use schema/content 49/49.

## Visitors

New arrivals form three-person parties with family, thrill-seeking or relaxed preferences. Admission and wallets remain individual. Parties prefer suitable ride categories and regroup on public paths between visits. Members can choose separate affordable activities. A departing member causes the group to finish its current activities and leave together.

Queue tolerance uses the visitor's preferences. Persistent low satisfaction prompts departure after at least one minute in the park. Satisfied departing parties schedule a return attempt after 200 simulation seconds. A return uses newly allocated visitor identities and fresh visitor wallets, retaining the party's preferences. Previous spending remains in the retired-visitor accounts.

Nearby scenery improves satisfaction up to 80. Entertainers can raise satisfaction to 85. Heat increases thirst. Rain and storms reduce satisfaction outdoors, while covered benches protect resting visitors and help restore energy. The guest inspector shows group membership, return status and shelter status.

## Staff teams

Open **Explore → Manage → Staff and park life**. Hire any combination of up to 20 team members. All five roles cost $1 every 30 simulation seconds. Staff walk along connected public paths from the visitor entrance and spend four seconds completing a task. Their task, completed-work count, patrol size and unpaid status are visible in the roster.

| Role | Work |
| --- | --- |
| Mechanic | Inspects and services due rides once they are closed, stopped and empty |
| Handyman | Cleans litter and vomit and empties bins |
| Security | Resolves vandalism and guest-distress incidents |
| Entertainer | Improves the satisfaction of nearby guests |
| Service attendant | Helps tired or nauseous guests and assists service waiting/use |

Select a staff row and use **Patrol tile** to add or remove a public-path tile. **Whole park** clears the restriction. Patrols restrict task destinations, with travel allowed through the connected public network. **Find selected** centers the park on the worker. **Dismiss selected** removes that worker and retains all paid wages in Finance.

Insufficient funds suspend the affected worker. The outstanding wage is retried when money becomes available. Unpaid workers remain visible and resume after payment. Removing paths clears invalid movement references and assignments. Workers wait at the entrance if their route or workplace is unavailable. Building a connection lets them recover.

The earlier single-mechanic and single-handyman controls remain under **Grounds controls** for existing parks. They retain their original contracts and accounting. The new roster is separate and defaults to empty.

## Weather and shelters

Automatic weather cycles through sunny, hot, rainy and thunderstorm conditions, changing every 60 simulation seconds. **Weather policy** can also select fixed weather. Changing policy preserves the simulation clock.

Storms close exposed rides while dispatched passengers complete their journeys. Covered carousels and teacups can keep operating. Rides closed by the weather are checked for safe reopening when it clears. A player's explicit close request is respected. A ride with damaged access, unpaid costs or outstanding maintenance remains closed until its requirements are satisfied.

Use **Cover a path**, then click a public path. This adds a striped roof to a bench on that tile. A new bench costs $1 through the existing construction ledger. Covering an existing bench is free. Toggling the roof off keeps the bench. Removing the bench or path removes its cover, and normal bench demolition provides the existing refund. The roof fits within the public path's reserved clearance.

## Incidents

Severely tired or sick visitors can generate a distress report. Unhappy visitors near benches or bins can cause vandalism. A vandalized shelter stops protecting visitors until security repairs it. Incidents affect local satisfaction, appear as orange map markers and remain in the management report until resolved or their path is removed.

Use **Find first incident** to center the affected tile, then check the security team's route or patrol. Service attendants relieve the underlying physical needs. Incidents, resolved counts, weather closures, staff routes, wages, group membership and pending return visits are saved.

## Verification and continuation

The runtime includes a mixed-park writer/verifier using all 14 rides, all eight facilities, all five staff roles and covered paths. It advances 18,000 ticks before saving, then compares 9,000 additional ticks in a fresh process. The original save must remain byte-identical.

```powershell
.\target\release\parkbound.exe --save-check write-park-life --save-dir 'C:\Parkbound-Proofs\park-life'
.\target\release\parkbound.exe --save-check verify-park-life --save-dir 'C:\Parkbound-Proofs\park-life'
```

Use a new absolute directory for the writer. Capture scenes are `park-life`, `park-life-rain` and `park-life-staff`. The current verification record, source identity and shared OneDrive path are recorded in `RESUME_DEVELOPMENT.md`.

The development PC supplies functional diagnostics. Its speed is excluded from performance targets.
