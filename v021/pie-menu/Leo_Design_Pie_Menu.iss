#ifndef BaseExe
  #error "BaseExe is required"
#endif
#ifndef PieExe
  #error "PieExe is required"
#endif
#define BaseHash GetSHA256OfFile(BaseExe)
#define PieHash GetSHA256OfFile(PieExe)
#define PieVersion "0.7.4"

[Setup]
AppId=LeoDesignOptionalPieMenuForCoreBridgeV021
AppName=Leo Design Pie Menu v{#PieVersion}
AppVersion={#PieVersion}
AppPublisher=Leo Design
AppPublisherURL=https://github.com/leodesign8522/leo-vectorcraft-build
DefaultDirName={autopf}\ArtCraft\VectorCraft
UsePreviousAppDir=yes
DisableDirPage=no
DirExistsWarning=no
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
OutputBaseFilename=Leo_Design_Pie_Menu_v{#PieVersion}_Optional_for_Core_Bridge_v0.10.6
Compression=lzma2
SolidCompression=yes
CloseApplications=yes
RestartApplications=no
Uninstallable=yes
UninstallDisplayName=Leo Design Pie Menu v{#PieVersion} (Core Bridge v0.10.6)
UninstallFilesDir={app}\LeoDesignPieMenu
SetupLogging=yes

[Files]
; The primary EXE belongs to Core Bridge and must NEVER be deleted by uninstall.
Source: "{#PieExe}"; DestDir: "{app}"; DestName: "vectorcraft.exe"; Flags: ignoreversion uninsneveruninstall
Source: "{#BaseExe}"; DestDir: "{app}\LeoDesignPieMenu"; DestName: "Core_Bridge_v0.10.6_No_Pie.exe"; Flags: ignoreversion
Source: "OPTIONAL_INSTALL.md"; DestDir: "{app}\LeoDesignPieMenu"; Flags: ignoreversion

[Code]
function HasHash(const Path, Expected: String): Boolean;
begin
  Result := False;
  if not FileExists(Path) then Exit;
  try
    Result := CompareText(GetSHA256OfFile(Path), Expected) = 0;
  except
    Result := False;
  end;
end;

function TargetExe(): String;
begin
  Result := ExpandConstant('{app}\vectorcraft.exe');
end;

function CompatibleTarget(): Boolean;
begin
  Result := HasHash(TargetExe(), '{#BaseHash}') or HasHash(TargetExe(), '{#PieHash}');
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  Result := '';
  if not CompatibleTarget() then
    Result := '请先在此目录安装配套的 Leo Design Core Bridge v0.10.6（不含饼菜单）。' + #13#10 +
      '本可选包不会覆盖其他版本。请关闭 VectorCraft 后重试。';
end;

function InitializeUninstall(): Boolean;
var
  Target, Base, Stage, Pending: String;
begin
  Result := False;
  Target := TargetExe();
  Base := ExpandConstant('{app}\LeoDesignPieMenu\Core_Bridge_v0.10.6_No_Pie.exe');
  Stage := ExpandConstant('{app}\LeoDesignPieMenu\restore.pending.exe');
  Pending := ExpandConstant('{app}\LeoDesignPieMenu\pie.pending.exe');

  // A later Core update must never be overwritten with this older restore file.
  if not HasHash(Target, '{#PieHash}') then
  begin
    Log('Primary EXE differs from this optional edition; keep it unchanged.');
    Result := True;
    Exit;
  end;
  if not HasHash(Base, '{#BaseHash}') then
  begin
    SuppressibleMsgBox('配套 Core 恢复文件缺失或已改变，卸载已停止。请重新安装 Core Bridge v0.10.6。', mbError, MB_OK, IDOK);
    Exit;
  end;
  if FileExists(Stage) or FileExists(Pending) then
  begin
    SuppressibleMsgBox('发现上次未完成的恢复文件，卸载已停止。请保留这些文件并重新安装 Core Bridge。', mbError, MB_OK, IDOK);
    Exit;
  end;
  if not FileCopy(Base, Stage, True) then
  begin
    SuppressibleMsgBox('无法准备 Core 恢复文件，请关闭 VectorCraft 后重试。', mbError, MB_OK, IDOK);
    Exit;
  end;
  if not HasHash(Stage, '{#BaseHash}') then
  begin
    DeleteFile(Stage);
    SuppressibleMsgBox('Core 恢复文件校验失败，主程序未改变。', mbError, MB_OK, IDOK);
    Exit;
  end;
  if not RenameFile(Target, Pending) then
  begin
    DeleteFile(Stage);
    SuppressibleMsgBox('无法恢复主程序，请关闭 VectorCraft 后重试。', mbError, MB_OK, IDOK);
    Exit;
  end;
  // Recheck after acquiring the old target, in case another updater raced us.
  if not HasHash(Pending, '{#PieHash}') then
  begin
    RenameFile(Pending, Target);
    DeleteFile(Stage);
    SuppressibleMsgBox('主程序已被其他更新改变，卸载已停止。', mbError, MB_OK, IDOK);
    Exit;
  end;
  if not RenameFile(Stage, Target) then
  begin
    if not RenameFile(Pending, Target) then
      SuppressibleMsgBox('恢复未完成，请保留 LeoDesignPieMenu 目录中的恢复文件并重新安装 Core Bridge。', mbError, MB_OK, IDOK)
    else
      DeleteFile(Stage);
    Exit;
  end;
  DeleteFile(Pending);
  Result := True;
end;

