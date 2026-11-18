#ifndef PackageDir
  #error PackageDir must name a verified Windows package.
#endif
#ifndef ReleaseVersion
  #error ReleaseVersion is required.
#endif
#ifndef ReleaseOutput
  #error ReleaseOutput is required.
#endif

[Setup]
AppId=Parkbound.Free.Windows.x64
AppName=Parkbound
AppVersion={#ReleaseVersion}
AppPublisher=Parkbound project
AppPublisherURL=https://github.com/strijbosmh-afk/RailRust-Tycoon
AppSupportURL=https://github.com/strijbosmh-afk/RailRust-Tycoon/issues
DefaultDirName={localappdata}\Programs\Parkbound
DefaultGroupName=Parkbound
PrivilegesRequired=lowest
ArchitecturesAllowed=x64os
ArchitecturesInstallIn64BitMode=x64os
MinVersion=10.0
DisableProgramGroupPage=auto
AllowNoIcons=yes
WizardStyle=modern
WizardSizePercent=110
InfoBeforeFile={#SourcePath}\INSTALLATION.txt
OutputDir={#ReleaseOutput}
OutputBaseFilename=Parkbound-{#ReleaseVersion}-setup-x64
Compression=lzma2
SolidCompression=yes
SetupMutex=Local\Parkbound.Setup.34245805-94B2-4E62-9306-1B56003A3F55
AppMutex=Local\Parkbound.Game.34245805-94B2-4E62-9306-1B56003A3F55
CloseApplications=no
RestartApplications=no
UninstallDisplayIcon={app}\Parkbound.exe
UninstallDisplayName=Parkbound
VersionInfoVersion={#ReleaseVersion}
VersionInfoDescription=Parkbound Windows setup
VersionInfoProductName=Parkbound

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "dutch"; MessagesFile: "compiler:Languages\Dutch.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#PackageDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\Parkbound"; Filename: "{app}\Parkbound.exe"; WorkingDir: "{app}"
#if FileExists(PackageDir + "\START_HERE.txt")
Name: "{group}\Parkbound guide"; Filename: "{app}\START_HERE.txt"
#endif
Name: "{group}\{cm:UninstallProgram,Parkbound}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Parkbound"; Filename: "{app}\Parkbound.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\Parkbound.exe"; Description: "{cm:LaunchProgram,Parkbound}"; Flags: nowait postinstall skipifsilent

[Code]
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  InstallPath, PlayerPath: String;
begin
  InstallPath := AddBackslash(Lowercase(ExpandFileName(WizardDirValue)));
  PlayerPath := AddBackslash(Lowercase(ExpandConstant('{localappdata}\Parkbound')));
  if Pos(PlayerPath, InstallPath) = 1 then
    Result := 'Choose a program folder outside your Parkbound player-data folder.'
  else
    Result := '';
end;

// Inno Setup records the files it installs and removes those files on uninstall.
// Player data is outside {app}. No recursive deletion of player folders is used.
