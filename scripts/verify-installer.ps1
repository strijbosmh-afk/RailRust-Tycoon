#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Installer,
    [Parameter(Mandatory)][string]$PackageDirectory,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [string]$PreviousInstaller,
    [ValidateRange(30, 1800)][int]$ProcessTimeoutSeconds = 300
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$Installer = (Resolve-Path -LiteralPath $Installer).Path
$PackageDirectory = (Resolve-Path -LiteralPath $PackageDirectory).Path
if ((Split-Path -Leaf $PackageDirectory) -notmatch '^Parkbound-(\d+\.\d+\.\d+)-windows-x64$') {
    throw 'Select the versioned Windows package being tested.'
}
$expectedVersion = $Matches[1]
if (-not [IO.Path]::IsPathFullyQualified($OutputDirectory) -or (Test-Path -LiteralPath $OutputDirectory)) {
    throw 'Choose a new absolute evidence directory.'
}
$registryPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Parkbound.Free.Windows.x64_is1'
if (Test-Path -LiteralPath $registryPath) {
    throw 'Parkbound is already installed for this account. Run this acceptance check in a clean Windows account.'
}
$mutexName = 'Local\Parkbound.Game.34245805-94B2-4E62-9306-1B56003A3F55'
$existing = $null
if ([Threading.Mutex]::TryOpenExisting($mutexName, [ref]$existing)) {
    $existing.Dispose()
    throw 'Close running Parkbound instances before the installer acceptance check.'
}
[void](New-Item -ItemType Directory -Path $OutputDirectory)
$OutputDirectory = (Resolve-Path -LiteralPath $OutputDirectory).Path
$installed = Join-Path $OutputDirectory 'Installed Parkbound é'
$profiles = Join-Path $OutputDirectory 'Profiles'
[void](New-Item -ItemType Directory -Path $profiles)
$groupName = 'Parkbound acceptance ' + [guid]::NewGuid().ToString('N')
$groupDirectory = Join-Path ([Environment]::GetFolderPath('StartMenu')) ('Programs\' + $groupName)
$executable = Join-Path $installed 'Parkbound.exe'
$log = Join-Path $OutputDirectory 'verification.txt'

function Record([string]$Message) {
    Add-Content -LiteralPath $log -Value $Message -Encoding utf8
    Write-Output $Message
}

function Run-Process([string]$File, [string]$Arguments, [string]$Name, [switch]$ExpectFailure) {
    $previousPath = $env:PATH
    try {
        $env:PATH = "$env:WINDIR\System32;$env:WINDIR"
        $process = Start-Process -FilePath $File -ArgumentList $Arguments -WorkingDirectory $env:WINDIR -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $OutputDirectory "$Name.stdout.txt") -RedirectStandardError (Join-Path $OutputDirectory "$Name.stderr.txt")
    } finally { $env:PATH = $previousPath }
    $timer = [Diagnostics.Stopwatch]::StartNew()
    while (-not $process.WaitForExit(10000)) {
        if ($timer.Elapsed.TotalSeconds -ge $ProcessTimeoutSeconds) {
            Stop-Process -Id $process.Id -Force
            throw "$Name exceeded its time limit. Its logs are retained."
        }
        Write-Output "$Name running"
    }
    $process.WaitForExit()
    if ($ExpectFailure) {
        if ($process.ExitCode -eq 0) { throw "$Name unexpectedly succeeded." }
    } elseif ($process.ExitCode -ne 0) {
        throw "$Name returned $($process.ExitCode)."
    }
    Record "$Name PASS (exit $($process.ExitCode))"
}

function Run-Setup([string]$File, [string]$Name, [switch]$ExpectFailure) {
    $arguments = '/VERYSILENT /SUPPRESSMSGBOXES /SP- /NORESTART /LANG=english /DIR="{0}" /GROUP="{1}" /LOG="{2}"' -f $installed, $groupName, (Join-Path $OutputDirectory "$Name.setup.log")
    Run-Process $File $arguments $Name -ExpectFailure:$ExpectFailure
}

function Profile-Hashes {
    $lines = @(Get-ChildItem -LiteralPath $profiles -File -Recurse | Sort-Object FullName | ForEach-Object {
        '{0}  {1}' -f (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash, [IO.Path]::GetRelativePath($profiles, $_.FullName)
    })
    return $lines -join "`n"
}

function Assert-Profiles([string]$Expected) {
    if ((Profile-Hashes) -cne $Expected) { throw 'Player data changed during installation or removal.' }
}

function Assert-Installed {
    foreach ($line in Get-Content -LiteralPath (Join-Path $PackageDirectory 'SHA256SUMS.txt')) {
        if ($line -notmatch '^([a-fA-F0-9]{64})  (.+)$') { throw 'Invalid package manifest.' }
        $path = Join-Path $installed $Matches[2]
        if (-not (Test-Path -LiteralPath $path) -or (Get-FileHash -LiteralPath $path).Hash -ne $Matches[1]) {
            throw "Installed payload mismatch for $($Matches[2])"
        }
    }
    $entry = Get-ItemProperty -LiteralPath $registryPath
    if ($entry.InstallLocation.TrimEnd('\') -ne $installed) { throw 'Unexpected installed location.' }
    if ($entry.DisplayVersion -ne $expectedVersion) { throw 'Unexpected version in Installed apps.' }
    $shortcut = Join-Path $groupDirectory 'Parkbound.lnk'
    if (-not (Test-Path -LiteralPath $shortcut)) { throw 'Missing Start menu shortcut.' }
    $shell = New-Object -ComObject WScript.Shell
    try {
        $link = $shell.CreateShortcut($shortcut)
        if ($link.TargetPath -ne $executable -or $link.WorkingDirectory -ne $installed) {
            throw 'Start menu shortcut points outside the installed game.'
        }
    } finally { [void][Runtime.InteropServices.Marshal]::ReleaseComObject($shell) }
    Record 'Installed files, current-user registration and Start menu target PASS'
}

function Run-Game([string]$Mode, [string]$Folder, [string]$Name) {
    Run-Process $executable ('--save-check {0} --save-dir "{1}"' -f $Mode, $Folder) $Name
}

$initialInstaller = $Installer
if ($PreviousInstaller) { $initialInstaller = (Resolve-Path -LiteralPath $PreviousInstaller).Path }
Record "Installer SHA256 $((Get-FileHash -LiteralPath $Installer).Hash.ToLowerInvariant())"
Record "Initial installer SHA256 $((Get-FileHash -LiteralPath $initialInstaller).Hash.ToLowerInvariant())"
Record "Prior-version installer supplied: $([bool]$PreviousInstaller)"
$principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
Record "Elevated test process: $($principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator))"
Record 'Runtime processes use Windows system folders in PATH and launch outside the source checkout.'
Run-Setup $initialInstaller 'initial-install'
$frontProfile = Join-Path $profiles 'Park library é'
$campaignProfile = Join-Path $profiles 'Campaign é'
Run-Game 'write-front' $frontProfile 'write-front'
Run-Game 'write-campaign' $campaignProfile 'write-campaign'
$preferences = Join-Path $profiles 'Preferences é'
Run-Process $executable ('--accessibility-check "{0}"' -f $preferences) 'write-preferences'
$baseline = Profile-Hashes
[IO.File]::WriteAllText((Join-Path $OutputDirectory 'player-data-before-upgrade.sha256'), $baseline)

$mutex = [Threading.Mutex]::new($false, $mutexName)
try { Run-Setup $Installer 'running-game-upgrade-refused' -ExpectFailure }
finally { $mutex.Dispose() }
Assert-Profiles $baseline
Run-Setup $Installer 'upgrade'
Assert-Installed
Assert-Profiles $baseline
Run-Game 'verify-front' $frontProfile 'verify-front'
Run-Game 'verify-campaign' $campaignProfile 'verify-campaign'
Run-Process $executable ('--verify-accessibility "{0}"' -f $preferences) 'verify-preferences'
$baseline = Profile-Hashes
Run-Setup $Installer 'repair-install'
Assert-Installed
Assert-Profiles $baseline

$userFile = Join-Path $installed 'player-note.txt'
[IO.File]::WriteAllText($userFile, 'A player-created file must survive uninstall.')
$uninstaller = Join-Path $installed 'unins000.exe'
$mutex = [Threading.Mutex]::new($false, $mutexName)
try { Run-Process $uninstaller '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART' 'running-game-uninstall-refused' -ExpectFailure }
finally { $mutex.Dispose() }
Assert-Profiles $baseline
Run-Process $uninstaller '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART' 'uninstall'
if (Test-Path -LiteralPath $executable) { throw 'Uninstall retained the installed executable.' }
if (Test-Path -LiteralPath $registryPath) { throw 'Uninstall retained its registration.' }
if (Test-Path -LiteralPath (Join-Path $groupDirectory 'Parkbound.lnk')) { throw 'Uninstall retained its shortcut.' }
if ((Get-Content -LiteralPath $userFile -Raw) -ne 'A player-created file must survive uninstall.') { throw 'Uninstall changed a player-created file.' }
Assert-Profiles $baseline
Record 'Uninstall preserved all park, campaign, settings and player-created file bytes PASS'

Run-Setup $Installer 'reinstall'
Assert-Installed
Assert-Profiles $baseline
Run-Process $uninstaller '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART' 'final-uninstall'
Assert-Profiles $baseline
if (Test-Path -LiteralPath $registryPath) { throw 'Final uninstall retained its registration.' }
Record 'Install, upgrade, repair, uninstall and reinstall acceptance PASS'
