param([string]$Installer, [string]$BaseExe, [string]$PieExe)
$ErrorActionPreference = 'Stop'
$testDir = Join-Path $env:RUNNER_TEMP "leo-pie-install-test-$env:GITHUB_RUN_ID"
New-Item -ItemType Directory -Force -Path $testDir | Out-Null
$target = Join-Path $testDir 'vectorcraft.exe'
$prefsDir = Join-Path $testDir 'VectorCraftData'
New-Item -ItemType Directory -Force -Path $prefsDir | Out-Null
$prefs = Join-Path $prefsDir 'test-user-preferences.json'
'{"keepOtherFeatures":true,"leo_pies":[{"name":"saved menu"}]}' | Set-Content $prefs -Encoding utf8
$prefsHash = (Get-FileHash $prefs).Hash
$baseHash = (Get-FileHash $BaseExe).Hash
$pieHash = (Get-FileHash $PieExe).Hash

$script:installAttempt = 0
function Install-Pie {
  $script:installAttempt += 1
  $installLog = Join-Path $testDir "install-$script:installAttempt.log"
  $installerArgs = @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', "/LOG=`"$installLog`"", "/DIR=`"$testDir`"")
  $p = Start-Process -FilePath $Installer -ArgumentList $installerArgs -WindowStyle Hidden -Wait -PassThru
  if ($p.ExitCode -ne 0) {
    Write-Host "Installer exit code: $($p.ExitCode)"
    if (Test-Path $installLog) { Get-Content $installLog | Select-Object -Last 35 | ForEach-Object { Write-Host $_ } }
  }
  return $p.ExitCode
}
function Uninstall-Pie {
  $uninstaller = (Get-ChildItem (Join-Path $testDir 'LeoDesignPieMenu') -Filter 'unins*.exe' | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
  $p = Start-Process -FilePath $uninstaller -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') -WindowStyle Hidden -Wait -PassThru
  if ($p.ExitCode -ne 0) { throw "Uninstall failed: $($p.ExitCode)" }
}
function Assert-Prefs {
  if ((Get-FileHash $prefs).Hash -ne $prefsHash) { throw 'User preferences must remain byte-identical' }
}

# Wrong/external/newer program: refuse before writing anything.
Copy-Item $BaseExe $target
[System.IO.File]::AppendAllText($target, 'different-build')
$wrongHash = (Get-FileHash $target).Hash
if ((Install-Pie) -eq 0) { throw 'Installer accepted an unmatched Core EXE' }
if ((Get-FileHash $target).Hash -ne $wrongHash) { throw 'Unmatched EXE was overwritten' }
Assert-Prefs

# Matching Core -> optional Pie -> matching Core, without removing other features.
Copy-Item $BaseExe $target -Force
if ((Install-Pie) -ne 0) { throw 'Optional install failed' }
if ((Get-FileHash $target).Hash -ne $pieHash) { throw 'Optional EXE not installed' }
Assert-Prefs
Uninstall-Pie
if ((Get-FileHash $target).Hash -ne $baseHash) { throw 'Uninstall must restore exact Core-only EXE' }
Assert-Prefs

# A later primary-program update must survive uninstalling an older add-on.
if ((Install-Pie) -ne 0) { throw 'Second optional install failed' }
Copy-Item $BaseExe $target -Force
[System.IO.File]::AppendAllText($target, 'simulated-later-core')
$laterHash = (Get-FileHash $target).Hash
Uninstall-Pie
if ((Get-FileHash $target).Hash -ne $laterHash) { throw 'Uninstall rolled back a later Core version' }
Assert-Prefs
Write-Host 'PASS: wrong-version rejection, optional install/remove, later-update retention, preferences preservation'
