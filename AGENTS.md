# RailRust Tycoon / Parkbound

## Google Drive synchronization stopped

The owner disabled Google Drive synchronization for this project on 2026-11-18.
Respect `PROJECT.json` → `drive.syncEnabled = false`. Use the local standard
prompt cache and preserve existing project IDs and queued exports. Work remains
in the shared OneDrive checkout and GitHub. Do not resume Drive transfers until
the owner explicitly requests them. The local archive helper keeps local
snapshots and suppresses the Drive upload continuation for this project.

- Open the actual Git root registered as `railrust-tycoon`. Preserve project identity, remotes, branches and unrelated changes.
- Read `PROJECT.json`, `STATUS.md`, `RESUME_DEVELOPMENT.md` and applicable parent instructions before code work.
- Use the current central Softwareprojecten prompt identified in `PROJECT.json`. The local helper is `%USERPROFILE%\.codex\softwareprojecten\archive.mjs`. Consult `ONBOARDING.md` in that directory for an unregistered PC.
- Preserve the existing central project registration. Verify GitHub ownership and the Drive account, owner and parent chain before registration or uploads. Only `strijbosmh@gmail.com` is authorized for project Drive storage.
- Follow the documented `onboarding.mjs --project PATH --apply` procedure for local registration after verifying destinations. Never replace an existing remote automatically.
- Git stores source and technical documentation. Stage task-related paths explicitly, verify changes, commit and push to the agreed branch, then verify the remote commit.
- Update `STATUS.md` and the resume guide. Record decisions in `BESLISSINGEN.md`, installation requirements in `INSTALLATIE.md`, and each session in `docs/sessions/` with project/session IDs, PC, UTC time, branch, commit, remaining work, checks and next step.
- The trusted native Stop hook creates exports and requests a follow-up for Drive upload. Do not run manual snapshots or hooks as proof of automation, and do not change hook trust automatically.
- Upload through the normal Drive connector. Verify owner, parent, size and remote checksum or SHA256 of downloaded bytes before acknowledging receipts. Keep failed or unverified exports queued.
- Chat exports include available user and assistant text with secret redaction. Record coverage limits. Exclude hidden reasoning, tool contents, media, authentication files, browser profiles, active `.git`, `target` and `node_modules`.
- Report local storage, Git publication, verified Drive storage and outstanding work separately. Installation on one PC does not prove automatic storage on another.
- Preserve deterministic simulation and compatibility with supported save versions. Run formatting, strict lints and relevant tests, including fresh-process continuation when save state changes.
- The current development PC is a correctness-checking environment. Do not use its timing as a product performance reference.
