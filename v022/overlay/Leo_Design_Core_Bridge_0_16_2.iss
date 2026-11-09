#ifndef SourceExe
  #define SourceExe "dist\vectorcraft.exe"
#endif
#define SourceDir ExtractFileDir(SourceExe)
[Setup]
AppId=LeoDesignCoreBridgeV022_0162
AppName=Leo Design Core Bridge 0.16.2 for VectorCraft 0.22.0
AppVersion=0.16.2
AppPublisher=Leo Design
DefaultDirName={localappdata}\Programs\Leo Design\VectorCraft-0.22-Bridge-0.16.2
DefaultGroupName=Leo Design VectorCraft 0.22 Bridge 0.16.2
UsePreviousAppDir=no
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
OutputBaseFilename=Leo_Design_Core_Bridge_v0.16.2_Installer_for_VectorCraft_v0.22.0
Compression=lzma2
SolidCompression=yes
CloseApplications=yes
RestartApplications=no
SetupLogging=yes
[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
[Icons]
Name: "{group}\Leo Design VectorCraft 0.22 Bridge 0.16.2"; Filename: "{app}\vectorcraft.exe"; Parameters: "--new-instance"
Name: "{localappdata}\Leo Design\AI Bridge\LaunchVectorCraft"; Filename: "{app}\vectorcraft.exe"; Parameters: "--new-instance"
Name: "{group}\Uninstall"; Filename: "{uninstallexe}"
[Code]
var SenderPaths: String;
function BridgeRoot: String;
begin Result:=ExpandConstant('{localappdata}\Leo Design\AI Bridge'); end;
procedure WriteUtf8(const FileName, Content: String);
var Lines: TArrayOfString;
begin
  SetArrayLength(Lines,1);
  Lines[0]:=Content;
  if not SaveStringsToUTF8File(FileName,Lines,False) then RaiseException('Unable to write bridge configuration: '+FileName);
end;
procedure InstallAt(const AppRoot: String);
var Locales, Dirs: TFindRec; Presets, Candidate, Folder: String;
begin
  Presets:=AddBackslash(AppRoot)+'Presets';
  if not DirExists(Presets) then exit;
  if FindFirst(AddBackslash(Presets)+'*',Locales) then begin
    try
      repeat
        if (Locales.Attributes and FILE_ATTRIBUTE_DIRECTORY<>0) and
          (Locales.Name<>'.') and (Locales.Name<>'..') then begin
          Candidate:=AddBackslash(Presets)+Locales.Name;
          if FindFirst(AddBackslash(Candidate)+'*',Dirs) then begin
            try
              repeat
                if (Dirs.Attributes and FILE_ATTRIBUTE_DIRECTORY<>0) and
                  ((Dirs.Name='Scripts') or (Dirs.Name='脚本') or (Dirs.Name='指令碼') or
                   (Dirs.Name='スクリプト') or (Dirs.Name='Skripten')) then begin
                  Folder:=AddBackslash(Candidate)+Dirs.Name;
                  if FileCopy(ExpandConstant('{app}\AI Bridge\Send_to_VectorCraft.jsx'),
                    AddBackslash(Folder)+'Leo Design - Send to VectorCraft.jsx',False) then begin
                    SenderPaths:=SenderPaths+AddBackslash(Folder)+'Leo Design - Send to VectorCraft.jsx'+#13#10;
                    Log('Installed Illustrator sender: '+Folder);
                  end else Log('Unable to install Illustrator sender: '+Folder);
                end;
              until not FindNext(Dirs);
            finally FindClose(Dirs); end;
          end;
        end;
      until not FindNext(Locales);
    finally FindClose(Locales); end;
  end;
end;
procedure ScanAdobe(const AdobeRoot: String);
var Apps: TFindRec;
begin
  if FindFirst(AddBackslash(AdobeRoot)+'Adobe Illustrator *',Apps) then begin
    try
      repeat
        if Apps.Attributes and FILE_ATTRIBUTE_DIRECTORY<>0 then InstallAt(AddBackslash(AdobeRoot)+Apps.Name);
      until not FindNext(Apps);
    finally FindClose(Apps); end;
  end;
end;
procedure CurStepChanged(CurStep: TSetupStep);
var Registered, Candidate: String; Attempts: Integer;
begin
  if CurStep=ssPostInstall then begin
    ForceDirectories(BridgeRoot);
    ForceDirectories(BridgeRoot+'\inbox');
    WriteUtf8(BridgeRoot+'\receiver.txt',ExpandConstant('{app}\vectorcraft.exe'));
    SenderPaths:='';
    ScanAdobe(ExpandConstant('{pf64}\Adobe'));
    ScanAdobe(ExpandConstant('{pf32}\Adobe'));
    if RegQueryStringValue(HKLM64,'SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\Illustrator.exe','',Registered) then begin
      Candidate:=ExtractFileDir(RemoveQuotes(Registered));
      for Attempts:=0 to 4 do begin InstallAt(Candidate);Candidate:=ExtractFileDir(Candidate);end;
    end;
    // Keep the list for diagnostics; never remove user scripts or script directories.
    WriteUtf8(BridgeRoot+'\installed-senders.txt',SenderPaths);
    if SenderPaths='' then begin
      Log('Illustrator script directory was not found. Sender remains available in the installed AI Bridge folder.');
      if not WizardSilent then MsgBox('未找到 Illustrator 脚本目录。VectorCraft 已安装；发送脚本位于安装目录的 AI Bridge 文件夹。自定义安装的 Illustrator 可将该脚本放入其 Presets/语言/脚本目录。',mbInformation,MB_OK);
    end else if not WizardSilent then
      MsgBox('双向发送入口已配置。请重启 Illustrator，在 文件 → 脚本 中找到 Leo Design - Send to VectorCraft。VectorCraft 中使用 文件 → 发送选中对象到 Illustrator。',mbInformation,MB_OK);
  end;
end;
