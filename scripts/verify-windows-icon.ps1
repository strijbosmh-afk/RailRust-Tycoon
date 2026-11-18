#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Executable,
    [string]$Icon = (Join-Path $PSScriptRoot '../assets/art/parkbound.ico'),
    [switch]$CheckWindow,
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '../artifacts/icon-verification')
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$Executable = (Resolve-Path -LiteralPath $Executable).Path
$Icon = (Resolve-Path -LiteralPath $Icon).Path
[void](New-Item -ItemType Directory -Path $OutputDirectory -Force)
$OutputDirectory = (Resolve-Path -LiteralPath $OutputDirectory).Path

if (-not ('ParkboundIconProbe' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
public static class ParkboundIconProbe {
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr LoadLibraryExW(string name, IntPtr file, uint flags);
    [DllImport("kernel32.dll")] public static extern bool FreeLibrary(IntPtr module);
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern IntPtr FindResourceW(IntPtr module, IntPtr name, IntPtr type);
    [DllImport("kernel32.dll")] static extern uint SizeofResource(IntPtr module, IntPtr resource);
    [DllImport("kernel32.dll")] static extern IntPtr LoadResource(IntPtr module, IntPtr resource);
    [DllImport("kernel32.dll")] static extern IntPtr LockResource(IntPtr resource);
    public static byte[] ReadResource(IntPtr module, int name, int type) {
        IntPtr resource = FindResourceW(module, (IntPtr)name, (IntPtr)type);
        if (resource == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
        byte[] bytes = new byte[SizeofResource(module, resource)];
        IntPtr data = LockResource(LoadResource(module, resource));
        if (data == IntPtr.Zero) throw new InvalidOperationException("Cannot read icon resource.");
        Marshal.Copy(data, bytes, 0, bytes.Length);
        return bytes;
    }
    delegate bool WindowCallback(IntPtr window, IntPtr state);
    [DllImport("user32.dll")] static extern bool EnumWindows(WindowCallback callback, IntPtr state);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
    public static IntPtr FindWindow(int process) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((window, state) => {
            GetWindowThreadProcessId(window, out uint owner);
            if (owner != process || !HasIcon(window, 1) || !HasIcon(window, 0)) return true;
            found = window;
            return false;
        }, IntPtr.Zero);
        return found;
    }
    [DllImport("user32.dll", SetLastError = true)]
    static extern IntPtr SendMessageTimeoutW(IntPtr window, uint message, UIntPtr wparam,
        IntPtr lparam, uint flags, uint timeout, out UIntPtr result);
    public static bool HasIcon(IntPtr window, uint kind) {
        return SendMessageTimeoutW(window, 0x7f, (UIntPtr)kind, IntPtr.Zero, 2, 1000,
            out UIntPtr result) != IntPtr.Zero && result != UIntPtr.Zero;
    }
    [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr window, uint message, UIntPtr wparam, IntPtr lparam);
}
'@
}

$source = [IO.File]::ReadAllBytes($Icon)
$count = [BitConverter]::ToUInt16($source, 4)
if ([BitConverter]::ToUInt16($source, 0) -ne 0 -or [BitConverter]::ToUInt16($source, 2) -ne 1 -or $count -ne 7) {
    throw 'Expected the seven-size Parkbound ICO.'
}
$module = [ParkboundIconProbe]::LoadLibraryExW($Executable, [IntPtr]::Zero, 0x22)
if ($module -eq [IntPtr]::Zero) { throw 'Cannot open executable resources.' }
$verified = @()
try {
    $group = [ParkboundIconProbe]::ReadResource($module, 1, 14)
    if ([BitConverter]::ToUInt16($group, 4) -ne $count) { throw 'Embedded icon count differs.' }
    foreach ($index in 0..($count - 1)) {
        $entry = 6 + 16 * $index
        $resourceEntry = 6 + 14 * $index
        $size = @(16, 24, 32, 48, 64, 128, 256)[$index]
        $encodedSize = $size % 256
        if ($source[$entry] -ne $encodedSize -or $source[$entry + 1] -ne $encodedSize) {
            throw "Unexpected ICO dimensions for $size."
        }
        foreach ($byte in 0..11) {
            if ($source[$entry + $byte] -ne $group[$resourceEntry + $byte]) {
                throw "Embedded icon header differs at $size pixels."
            }
        }
        $length = [BitConverter]::ToUInt32($source, $entry + 8)
        $offset = [BitConverter]::ToUInt32($source, $entry + 12)
        $payload = [byte[]]::new($length)
        [Array]::Copy($source, $offset, $payload, 0, $length)
        $resourceId = [BitConverter]::ToUInt16($group, $resourceEntry + 12)
        $embedded = [ParkboundIconProbe]::ReadResource($module, $resourceId, 3)
        $sha256 = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($payload))
        if ($embedded.Length -ne $length -or [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($embedded)) -cne $sha256) {
            throw "Embedded pixels differ at $size pixels."
        }
        $verified += [pscustomobject]@{ size = $size; bytes = $length; sha256 = $sha256.ToLowerInvariant() }
    }
} finally { [void][ParkboundIconProbe]::FreeLibrary($module) }
$version = (Get-Item -LiteralPath $Executable).VersionInfo
if ($version.ProductName -ne 'Parkbound' -or $version.FileDescription -ne 'Parkbound') {
    throw 'Missing Parkbound executable identity.'
}

$windowVerified = $false
if ($CheckWindow) {
    $profile = Join-Path $OutputDirectory ('profile-' + [guid]::NewGuid().ToString('N'))
    $process = Start-Process -FilePath $Executable -ArgumentList ('--save-dir "{0}"' -f $profile) -WorkingDirectory $env:WINDIR -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $OutputDirectory 'window.stdout.log') -RedirectStandardError (Join-Path $OutputDirectory 'window.stderr.log')
    $window = [IntPtr]::Zero
    try {
        $timer = [Diagnostics.Stopwatch]::StartNew()
        while ($timer.Elapsed.TotalSeconds -lt 90 -and -not $process.HasExited) {
            $window = [ParkboundIconProbe]::FindWindow($process.Id)
            if ($window -ne [IntPtr]::Zero -and [ParkboundIconProbe]::HasIcon($window, 1) -and [ParkboundIconProbe]::HasIcon($window, 0)) {
                $windowVerified = $true
                break
            }
            Start-Sleep -Milliseconds 250
        }
        if (-not $windowVerified) { throw 'The running game did not expose both native window icons. See retained logs.' }
    } finally {
        if (-not $process.HasExited) {
            if ($window -ne [IntPtr]::Zero) { [void][ParkboundIconProbe]::PostMessageW($window, 0x10, [UIntPtr]::Zero, [IntPtr]::Zero) }
            if (-not $process.WaitForExit(10000)) { Stop-Process -Id $process.Id -Force }
        }
        $process.Dispose()
    }
}
[pscustomobject]@{
    executable = $Executable
    executableSha256 = (Get-FileHash -LiteralPath $Executable).Hash.ToLowerInvariant()
    version = $version.FileVersion
    icons = $verified
    nativeWindowIconsVerified = $windowVerified
} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'verification.json') -Encoding utf8
Write-Output "Parkbound $($version.FileVersion): all seven embedded icon sizes match the source ICO."
if ($CheckWindow) { Write-Output 'Running game exposes both large and small native window icons.' }
