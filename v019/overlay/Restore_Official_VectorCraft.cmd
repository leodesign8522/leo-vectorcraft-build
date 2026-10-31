@echo off
setlocal EnableExtensions
title Leo Design - Restore Original VectorCraft
set "APPDIR=%~dp0.."
set "BACKUP=%~dp0Backup\VectorCraft_original_v0.18.exe"
set "TARGET=%APPDIR%\vectorcraft.exe"
echo.
echo Leo Design Core Bridge - Restore official VectorCraft v0.18
echo.
if not exist "%BACKUP%" (
  echo ERROR: Official program backup was not found:
  echo "%BACKUP%"
  pause
  exit /b 1
)
tasklist /FI "IMAGENAME eq vectorcraft.exe" 2>nul | findstr /I /C:"vectorcraft.exe" >nul
if not errorlevel 1 (
  echo Please close VectorCraft completely before restoring.
  pause
  exit /b 2
)
echo This will restore the ORIGINAL v0.18 executable from the installer's backup.
echo It will not delete or edit your VectorCraftData settings and documents.
echo If official VectorCraft has updated to a newer version, do NOT use this.
choice /M "Restore official VectorCraft v0.18 now"
if errorlevel 2 exit /b 0
copy /Y "%BACKUP%" "%TARGET%" >nul
if errorlevel 1 (
  echo ERROR: Restore failed. Right-click this file and choose Run as administrator.
  pause
  exit /b 3
)
echo.
echo Restored official VectorCraft executable.
echo The Leo Design backup has been retained.
pause
exit /b 0
