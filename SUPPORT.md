# Parkbound support

Use the in-game Help and First park guide for construction, access and coaster
controls. `START_HERE.txt` explains installation and player-data locations.

Report reproducible problems at
https://github.com/strijbosmh-afk/RailRust-Tycoon/issues.

Include the game version shown in Credits, Windows version, the action that
caused the problem and the result you expected. A screenshot can help explain
graphics and layout problems. If a particular park is needed to reproduce the
problem, share a copy after reviewing its names and content.

For a startup or installer problem, keep the downloaded package and its adjacent
`.sha256` file. The ZIP includes `release.json` with its version, source checkout
identity and executable hash. Windows PowerShell can check a download with
`Get-FileHash -Algorithm SHA256 -LiteralPath 'C:\path\to\download'`.

An installer log can be recorded with
`Parkbound-0.95.0-setup-x64.exe /LOG="C:\path\to\setup.log"`.
Review logs before sharing because paths may identify your Windows account.

Copy `%LOCALAPPDATA%\Parkbound\saves` while the game is closed before experimenting
with a failing save. Keep the original file and backup available for recovery.
