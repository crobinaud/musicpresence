; Inno Setup Script for MusicPresence
; Generates a modern Windows Installer and Uninstaller

#ifndef AppVersion
#define AppVersion "1.0.0"
#endif

[Setup]
AppId={{D68F23B8-438B-4C67-9DF2-87EB2A21F5CA}
AppName=MusicPresence
AppVersion={#AppVersion}
AppVerName=MusicPresence {#AppVersion}
AppPublisher=Cyprien ROBINAUD
AppPublisherURL=https://github.com/crobinaud/musicpresence
AppSupportURL=https://github.com/crobinaud/musicpresence/issues
AppUpdatesURL=https://github.com/crobinaud/musicpresence/releases
DefaultDirName={autopf}\MusicPresence
DefaultGroupName=MusicPresence
DisableProgramGroupPage=yes
SourceDir=..
OutputDir=.
LicenseFile=LICENSE
SetupIconFile=app.ico
UninstallDisplayIcon={app}\MusicPresence.exe
UninstallDisplayName=MusicPresence
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog commandline
OutputBaseFilename=MusicPresence-Setup
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startup"; Description: "Launch MusicPresence automatically on Windows startup"; GroupDescription: "Windows Integration:"

[Files]
Source: "target\release\MusicPresence.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "config.toml"; DestDir: "{app}"; Flags: ignoreversion onlyifdoesntexist
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "app.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\MusicPresence"; Filename: "{app}\MusicPresence.exe"; IconFilename: "{app}\MusicPresence.exe"
Name: "{autodesktop}\MusicPresence"; Filename: "{app}\MusicPresence.exe"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "MusicPresence"; ValueData: """{app}\MusicPresence.exe"""; Flags: uninsdeletevalue; Tasks: startup

[Run]
Filename: "{app}\MusicPresence.exe"; Description: "{cm:LaunchProgram,MusicPresence}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: files; Name: "{app}\MusicPresence.exe.old"
Type: files; Name: "{app}\MusicPresence.exe.new"
