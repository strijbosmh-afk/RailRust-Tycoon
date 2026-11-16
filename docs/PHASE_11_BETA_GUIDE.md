# Phase 11 beta stability and balance

Version 0.90.0 adds an Operations page and reproducible beta runs. Park saves remain schema/content 51/51. Existing parks keep their simulation rules, prices and guest limits.

## Find operational problems

Open **Manage → Management → Operations**. The page shows current issues and visitor counts. Use **Previous issue**, **Next issue** and **Inspect issue** to open the affected ride, facility, visitor or staff panel. It checks disconnected access, unpaid operating costs, maintenance shutdowns, empty stock, unpaid staff and stranded guests. Issues disappear as their causes are resolved. Intentionally closed, healthy attractions are valid operating choices.

## Reproduce a beta run

The Windows package includes `verify-beta.ps1`. PowerShell 7 runs the test suite against its adjacent executable. The destination must be a new absolute directory. Every case uses isolated files.

```powershell
./verify-beta.ps1 -OutputDirectory 'C:\Parkbound beta\run-01'
```

From the source checkout, use `./scripts/verify-beta.ps1` after a release build. Supply `-Executable` to select a particular packaged build. Default runs simulate 108,000 ticks, equal to one hour of game time, per case. Each writer advances an uninterrupted park and a repeatedly reloaded park. A separate process then replays every checkpoint from the original starting state. The writer also checks backup rotation and independent finance totals.

| Case | Map and workload |
| --- | --- |
| `starter` | Flat 32×32 starter, existing guests and coaster, management and park life |
| `medium-mixed` | 64×64 district with all 14 catalogue rides, eight facilities, five staff roles, research, weather and arrivals |
| `large-mixed` | 256×256 version of that district, with a connected avenue extending across the map |
| `crowd-capacity` | Large mixed park beginning with 256 guests admitted through production arrival rules |

The mixed fixtures retain the existing catalogue showcase's recorded historical admission income. Their balance is useful for comparing changes under a fixed workload. Campaign difficulty is assessed separately with actual campaign starts and play sessions.

```powershell
./scripts/verify-beta.ps1 -OutputDirectory 'C:\Parkbound beta\quick-01' -Ticks 900 -CheckpointTicks 900
./scripts/verify-beta.ps1 -OutputDirectory 'C:\Parkbound beta\four-hours-01' -Cases large-mixed -Ticks 432000 -CheckpointTicks 18000
```

`beta-suite.json` identifies the executable by SHA256 and records each process exit, duration and sampled private memory peak. Each case contains checkpoint saves, `beta-report.json` and `beta-verification.json`. Reports include ride throughput, ticket prices, refusals, ratings, service income and costs, guest thoughts, satisfaction, research, staff work and cash. Failed runs retain their evidence and return a failing exit status. Up to 128 intervals and 12 simulated hours are supported.

The report's simulation timings cover the uninterrupted production ticks. Save and load durations are separate. Memory is sampled every 200 ms. These values describe the machine that ran the test.

## Measure rendering separately

The same case names work in the CPU and rendering tools. Existing `baseline`, `save-scale` and synthetic `stress` cases remain available for historical comparisons. The synthetic 5,000-guest case exceeds the 512-guest save limit and the 256-active-arrival limit.

```powershell
./scripts/measure-performance.ps1 -OutputDirectory 'C:\Parkbound beta\cpu-01' -Cases medium-mixed,large-mixed,crowd-capacity -SkipFrames
./scripts/measure-performance.ps1 -OutputDirectory 'C:\Parkbound beta\gpu-01' -SkipCpu -RenderCase large-mixed -RenderCamera ride -GpuDiagnostics
```

Repeat the rendering run at all four rotations and at the display sizes in `FULL_GAME_ROADMAP.md`. Save the graphics driver, resolution, scaling and power settings with the results. GPU timestamps depend on device support. Frame-loop time, GPU spans and simulation time have separate meanings.

## Acceptance still requiring a separate PC

The current development PC is excluded from performance acceptance. A representative Windows machine, display coverage, independent first-player session, subjective sound assessment and campaign balance sessions remain required by the roadmap. Record CPU, GPU, RAM, Windows version, resolution and scaling before setting minimum specifications or frame-time targets.

Use `PHASE_11_VERIFICATION.md` for observed results. Record failures with the build version, executable hash, case/seed, exact steps and the last successful save. Save copies and diagnostic files should be reviewed before sharing because player-created names may appear in them.
