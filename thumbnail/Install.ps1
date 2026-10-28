param()
$ErrorActionPreference = 'Stop'
$clsid = '{DA7638F6-36B8-459F-90BC-CEF9A6CFC923}'
$handler = '{E357FCCD-A995-4576-B01F-234630154E96}'
$association = "Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Classes\.vectorcraft\shellex\$handler"
$approved = 'Registry::HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Shell Extensions\Approved'
$root = Join-Path $env:ProgramFiles 'Leo Design\Thumbnail Preview'
$dll = Join-Path $root 'LeoDesign.ThumbnailPreview.dll'
$regasm = Join-Path $env:WINDIR 'Microsoft.NET\Framework64\v4.0.30319\RegAsm.exe'

$admin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
    [Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $admin) {
    $args = '-NoProfile -ExecutionPolicy Bypass -File "' + $PSCommandPath + '"'
    Start-Process -FilePath 'powershell.exe' -ArgumentList $args -Verb RunAs
    exit
}
if (-not [Environment]::Is64BitProcess) { throw 'Please use 64-bit Windows PowerShell.' }
if (-not (Test-Path $regasm)) { throw 'Windows .NET Framework 4.x RegAsm.exe not found.' }
$source = Join-Path $PSScriptRoot 'LeoDesign.ThumbnailPreview.dll'
if (-not (Test-Path $source)) { throw 'Keep the DLL and install scripts together in the extracted folder.' }

Write-Host 'Installing Leo Design Thumbnail Preview...' -ForegroundColor Cyan
New-Item -ItemType Directory -Path $root -Force | Out-Null
# Back up an existing .vectorcraft handler on the first install. Never silently discard it.
$backup = Join-Path $root 'previous-handler.txt'
if (-not (Test-Path $backup)) {
    $current = if (Test-Path $association) { (Get-Item -Path $association).GetValue('') } else { $null }
    if ($current) { [IO.File]::WriteAllText($backup, [string]$current) }
    else { [IO.File]::WriteAllText($backup, '__none__') }
}
Copy-Item -Path $source -Destination $dll -Force
& $regasm $dll /codebase /nologo
if ($LASTEXITCODE -ne 0) { throw 'RegAsm failed; Windows thumbnail extension was not installed.' }
New-Item -Path $association -Force | Out-Null
Set-Item -Path $association -Value $clsid
New-Item -Path $approved -Force | Out-Null
New-ItemProperty -Path $approved -Name $clsid -PropertyType String -Value 'Leo Design Thumbnail Preview' -Force | Out-Null
Add-Type -Namespace LeoShell -Name Notifications -MemberDefinition @'
[System.Runtime.InteropServices.DllImport("shell32.dll")]
public static extern void SHChangeNotify(uint id, uint flags, System.IntPtr a, System.IntPtr b);
'@
[LeoShell.Notifications]::SHChangeNotify(0x08000000, 0, [IntPtr]::Zero, [IntPtr]::Zero)
Write-Host 'Installation registered. If existing thumbnails do not refresh, restart File Explorer.' -ForegroundColor Green
Write-Host 'NOTE: A .vectorcraft file needs an embedded PNG Preview (Save As > Preview) to show its artwork.'
Read-Host 'Press Enter to close'
