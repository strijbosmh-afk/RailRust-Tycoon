#requires -Version 7.0
[CmdletBinding()]
param([Parameter(Mandatory)][string]$InstallDirectory)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT' -or -not [IO.Path]::IsPathFullyQualified($InstallDirectory)) {
    throw 'Choose an absolute Windows directory for the build tool.'
}
if (Test-Path -LiteralPath $InstallDirectory) { throw 'Build-tool directory must be new.' }
$parent = Split-Path -Parent $InstallDirectory
[void](New-Item -ItemType Directory -Path $parent -Force)
$download = Join-Path $parent 'innosetup-6.7.4.exe'
$expected = 'a549929319f9120716677c685535ca4671e4b937b5fdc87e2585f32d8eb30e8c'
if (-not (Test-Path -LiteralPath $download)) {
    Invoke-WebRequest -Uri 'https://github.com/jrsoftware/issrc/releases/download/is-6_7_4/innosetup-6.7.4.exe' -OutFile $download
}
if ((Get-FileHash -LiteralPath $download -Algorithm SHA256).Hash -ne $expected) {
    throw 'Inno Setup download differs from the pinned official release.'
}
$arguments = @('/CURRENTUSER', '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-', '/NOICONS', ('/DIR="{0}"' -f $InstallDirectory))
$process = Start-Process -FilePath $download -ArgumentList $arguments -PassThru -Wait -WindowStyle Hidden
if ($process.ExitCode -ne 0) { throw "Build-tool installer returned $($process.ExitCode)." }
$compiler = Join-Path $InstallDirectory 'ISCC.exe'
if (-not (Test-Path -LiteralPath $compiler)) { throw 'Inno Setup compiler was not installed.' }
Write-Output $compiler
