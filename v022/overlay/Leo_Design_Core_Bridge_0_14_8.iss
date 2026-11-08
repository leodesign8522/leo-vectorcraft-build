#ifndef SourceExe
  #define SourceExe "dist\vectorcraft.exe"
#endif
#define SourceDir ExtractFileDir(SourceExe)
[Setup]
AppId=LeoDesignCoreBridgeV022_0148
AppName=Leo Design Core Bridge 0.14.8 for VectorCraft 0.22.0
AppVersion=0.14.8
AppPublisher=Leo Design
AppPublisherURL=https://github.com/leodesign8522/leo-vectorcraft-build
DefaultDirName={localappdata}\Programs\Leo Design\VectorCraft-0.22-Bridge-0.14.8
DefaultGroupName=Leo Design VectorCraft 0.22 Bridge 0.14.8
UsePreviousAppDir=no
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
OutputBaseFilename=Leo_Design_Core_Bridge_v0.14.8_Installer_for_VectorCraft_v0.22.0
Compression=lzma2
SolidCompression=yes
CloseApplications=yes
RestartApplications=no
SetupLogging=yes

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "vectorcraft.exe"; Flags: ignoreversion
Source: "{#SourceDir}\README_FIRST.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\BUILD_VALIDATION.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\UPSTREAM_COMMIT.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\BUILD_COMMIT.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\VERSION.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\portable.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\NOTICE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\ASSETS.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\licenses\*"; DestDir: "{app}\licenses"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\Leo Design VectorCraft 0.22 Bridge 0.14.8"; Filename: "{app}\vectorcraft.exe"; Parameters: "--new-instance"
Name: "{group}\Uninstall"; Filename: "{uninstallexe}"
