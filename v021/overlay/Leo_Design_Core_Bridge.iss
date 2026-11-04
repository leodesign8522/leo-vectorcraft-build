#ifndef SourceExe
  #define SourceExe "dist\vectorcraft.exe"
#endif

#define BridgeVersion "0.10.6"
#define UpstreamVersion "0.21.0"

[Setup]
AppId=LeoDesignCoreBridgeOverlayForVectorCraftV021
AppName=Leo Design Core Bridge (No Pie Menu) v{#BridgeVersion} (VectorCraft v{#UpstreamVersion})
AppVersion={#BridgeVersion}
AppPublisher=Leo Design
AppPublisherURL=https://github.com/leodesign8522/leo-vectorcraft-build
DefaultDirName={autopf}\ArtCraft\VectorCraft
UsePreviousAppDir=no
DisableDirPage=no
DirExistsWarning=no
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
OutputBaseFilename=Leo_Design_Core_Bridge_v{#BridgeVersion}_No_Pie_Overlay_for_VectorCraft_v{#UpstreamVersion}
Compression=lzma2
SolidCompression=yes
CloseApplications=yes
RestartApplications=no
Uninstallable=no
CreateUninstallRegKey=no
SetupLogging=yes
; This is a safe overlay installer, NOT a separately loadable VectorCraft plugin.
; It must be rebuilt against each supported upstream release.

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "vectorcraft.exe"; Flags: ignoreversion

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
      MsgBox('û���ҵ��ٷ� VectorCraft ������' + #13#10 + ExePath + #13#10 +
        '��ѡ���Ѱ�װ VectorCraft ���ļ��У�����ѡ��Ŀ¼��', mbError, MB_OK);
    Exit;
  end;

  // Leo Design: only overwrite the matching official version.
  // Users can reinstall the official program; no program backup is created.
  // Where the Windows file version is present, block incompatible releases.
  if GetVersionNumbersString(ExePath, S) then
  begin
    if (S <> '0.21.0.0') and (S <> '0.21.0') then
    begin
      if ShowMessage then
        MsgBox('Ŀ�����汾Ϊ ' + S + #13#10 +
          '�˰�װ��ֻ������ VectorCraft v0.21.0�����𸲸������汾��', mbError, MB_OK);
      Exit;
    end;
  end
  else
  begin
    if ShowMessage then
      MsgBox('�޷���ȡĿ�����汾����װ��ֹͣ����ѡ��ٷ� VectorCraft v0.21.0��', mbError, MB_OK);
    Exit;
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
    Result := '�Ҳ������ݵ� VectorCraft v0.21 ���������鰲װ·���ͳ���汾��';
end;



