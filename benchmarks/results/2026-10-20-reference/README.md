# Parkbound 0.7.0 measurements

These are the original JSON outputs from the release executable built from
`e93eb99ddb4ac84a8ea1bc6f8a324424e5d27559`, measured on 20 October 2026.
The executable SHA-256 is
`06c10a106dd3afe8d51c604c8238b10b5f5266f594a660b3e7e7b919260f742b`.
All 16 JSON files were copied byte-for-byte after checking for private paths
and personal identifiers. `SHA256SUMS.txt` covers every JSON file.

The root batch contains three repetitions of each 300-tick CPU case, two
600-sample frame profiles, and its wrapper summary. `extended/` contains three
1,800-tick stress runs and a second summary. The first batch recorded a clean
checkout. The extended batch recorded documentation changes in progress; the
runtime sources and measured executable were unchanged. The wrappers recorded
the executable hash again after each batch.

Read [the performance report](../../../docs/PERFORMANCE_REPORT.md) for hardware,
method, limitations, ranges across runs, and the next implementation decisions.
These are initial observations on an active Windows workstation, not a controlled
idle-machine or GPU benchmark. Process completion does not establish the proposed
50-ride/60-FPS target. Frame data covers the normal 32-by-32 demo, and the larger
CPU fixtures are synthetic grids with one circuit.
