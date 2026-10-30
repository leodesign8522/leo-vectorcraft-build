#ifndef SourceExe
  #define SourceExe "dist\vectorcraft.exe"
#endif

#define BridgeVersion "0.6.0"
#define UpstreamVersion "0.18.0"

[Setup]
AppId=LeoDesignCoreBridgeOverlayForVectorCraftV018
AppName=Leo Design Core Bridge v{#BridgeVersion} (VectorCraft v{#UpstreamVersion})
AppVersion={#BridgeVersion}
AppPublisher=Leo Design
AppPublisherURL=https://github.com/leodesign8522/leo-vectorcraft-build
DefaultDirName={autopf}\VectorCraft
UsePreviousAppDir=no
DisableDirPage=no
DirExistsWarning=no
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
OutputBaseFilename=Leo_Design_Core_Bridge_v{#BridgeVersion}_Overlay_for_VectorCraft_v{#UpstreamVersion}
Compression=lzma2
SolidCompression=yes
CloseApplications=yes
RestartApplications=no
Uninstallable=no
CreateUninstallRegKey=no
SetupLogging=yes
SetupIconFile=
; This is a safe overlay installer, NOT a separately loadable VectorCraft plugin.
; It must be rebuilt against each supported upstream release.

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "vectorcraft.exe"; Flags: ignoreversion; BeforeInstall: BackupOriginalExecutable
Source: "Restore_Official_VectorCraft.cmd"; DestDir: "{app}\LeoDesign"; Flags: ignoreversion

[Code]
function TargetExe(): String;
begin
  Result := AddBackslash(WizardDirValue()) + 'vectorcraft.exe';
end;

function CheckTarget(const ShowMessage: Boolean): Boolean;
var
  S, ExePath: String;
begin
  Result := False;
  ExePath := TargetExe();
  if not FileExists(ExePath) then
  begin
    if ShowMessage then
      MsgBox('没有找到官方 VectorCraft 主程序：' + #13#10 + ExePath + #13#10 +
        '请选择已安装 VectorCraft 的文件夹，不能选空目录。', mbError, MB_OK);
    Exit;
  end;

  // Where the Windows file version is present, block incompatible releases.
  if GetVersionNumbersString(ExePath, S) then
  begin
    if (Pos('0.18.', S) <> 1) and (S <> '0.18.0') then
    begin
      if ShowMessage then
        MsgBox('目标程序版本为 ' + S + #13#10 +
          '此安装包只适用于 VectorCraft v0.18.x。请勿覆盖其他版本。', mbError, MB_OK);
      Exit;
    end;
  end;
  Result := True;
end;

function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;
  if CurPageID = wpSelectDir then
    Result := CheckTarget(True);
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  Result := '';
  if not CheckTarget(False) then
    Result := '找不到兼容的 VectorCraft v0.18 主程序。请检查安装路径和程序版本。';
end;

procedure BackupOriginalExecutable();
var
  DestDir, OriginalExe, BackupExe: String;
begin
  OriginalExe := ExpandConstant('{app}\vectorcraft.exe');
  DestDir := ExpandConstant('{app}\LeoDesign\Backup');
  BackupExe := DestDir + '\VectorCraft_original_v0.18.exe';
  if not FileExists(OriginalExe) then
    RaiseException('无法找到待备份的 VectorCraft 主程序。');
  if not DirExists(DestDir) and not ForceDirectories(DestDir) then
    RaiseException('无法创建原版程序备份目录：' + DestDir);
  // Never overwrite the initial backup with an already enhanced executable.
  if not FileExists(BackupExe) then
  begin
    if not FileCopy(OriginalExe, BackupExe, False) then
      RaiseException('无法备份官方程序。覆盖安装已中止：' + BackupExe);
  end;
end;
