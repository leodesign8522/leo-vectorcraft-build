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
if (Test-Path $association) {
    $current = (Get-Item -Path $association).GetValue('')
    # Do not remove another app's handler if the association has changed.
    if ($current -eq $clsid) {
        $backup = Join-Path $root 'previous-handler.txt'
        $previous = if (Test-Path $backup) { [IO.File]::ReadAllText($backup).Trim() } else { '__none__' }
        if ($previous -eq '__none__') { Remove-Item -Path $association -Recurse -Force }
        else { Set-Item -Path $association -Value $previous }
    }
}
Remove-ItemProperty -Path $approved -Name $clsid -ErrorAction SilentlyContinue
if (Test-Path $dll) { & $regasm $dll /unregister /nologo | Out-Host }
Add-Type -Namespace LeoShell -Name Notifications -MemberDefinition @'
[System.Runtime.InteropServices.DllImport("shell32.dll")]
public static extern void SHChangeNotify(uint id, uint flags, System.IntPtr a, System.IntPtr b);
'@
[LeoShell.Notifications]::SHChangeNotify(0x08000000, 0, [IntPtr]::Zero, [IntPtr]::Zero)
Write-Host 'Leo Design Thumbnail Preview unregistered.' -ForegroundColor Green
Write-Host 'You may delete the installed DLL folder after restarting File Explorer.'
Read-Host 'Press Enter to close'
