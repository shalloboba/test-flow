; Radial Menu インストーラー (Inno Setup 6)
; ビルド: build-installer.ps1 を実行(または、先に本体と設定アプリをビルドしてから iscc RadialMenu.iss)

#define MyAppName      "Radial Menu"
#define MyAppVersion   "0.4.0"
#define MyAppPublisher "Radial Menu"
#define MyAppExe       "RadialMenu.exe"
#define SettingsExe    "RadialMenu.Settings.exe"

; ビルド成果物の場所(このファイルからの相対パス)
#define HostBin        "..\target\release\RadialMenu.exe"
#define SettingsDir    "..\settings-winui\bin\Release\net8.0-windows10.0.22621.0\win-x64\publish"

[Setup]
; 固定の AppId(更新時に同じアプリとして上書きするため、変えないこと)
AppId={{6B1F3C5E-2D8A-4E57-9A41-7C3D0E5B8F21}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
VersionInfoVersion={#MyAppVersion}
; UIAccess 版を使う場合は Program Files 配置が必須なので、管理者権限でインストールする
PrivilegesRequired=admin
DefaultDirName={autopf}\RadialMenu
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
DisableDirPage=auto
UninstallDisplayIcon={app}\{#MyAppExe}
UninstallDisplayName={#MyAppName}
SetupIconFile=..\host\app.ico
OutputDir=output
OutputBaseFilename=RadialMenu-Setup
Compression=lzma2/ultra
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
; 更新(/SILENT /CLOSEAPPLICATIONS /RESTARTAPPLICATIONS)でも常駐本体と設定アプリを止めてから上書きする
CloseApplications=yes
RestartApplications=no
MinVersion=10.0.17763

[Languages]
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "english";  MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "autostart"; Description: "Windows 起動時に自動で開始する"; GroupDescription: "オプション:"
Name: "desktopicon"; Description: "デスクトップにショートカットを作成する"; GroupDescription: "オプション:"; Flags: unchecked

[Files]
; 常駐本体
Source: "{#HostBin}"; DestDir: "{app}"; Flags: ignoreversion
; 設定アプリ(WinUI 3 / ランタイム同梱)。本体は settings\ サブフォルダーも探す
Source: "{#SettingsDir}\*"; DestDir: "{app}\settings"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExe}"
Name: "{group}\{#MyAppName} の設定"; Filename: "{app}\settings\{#SettingsExe}"; WorkingDir: "{app}\settings"
Name: "{group}\{#MyAppName} のアンインストール"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExe}"; Tasks: desktopicon

[Registry]
; 自動起動(設定アプリ/本体と同じ HKCU の Run キー・同じ登録名)。
; 新規インストール時のみ登録し、更新では利用者が設定アプリで切り替えた状態を壊さない。
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "RadialMenu"; ValueData: """{app}\{#MyAppExe}"""; Flags: uninsdeletevalue; Tasks: autostart; Check: IsFreshInstall

[Run]
; 完了後に起動(サイレント更新でも起動する。多重起動は本体側のミューテックスで防がれるが、
; 事前に本体を終了しているので設定画面が開くことはない)
Filename: "{app}\{#MyAppExe}"; WorkingDir: "{app}"; Description: "{#MyAppName} を起動する"; Flags: nowait postinstall runasoriginaluser

[UninstallDelete]
; 設定(%APPDATA%\RadialMenu\config.json)は利用者のデータなので消さない。ログのみ削除。
Type: files; Name: "{localappdata}\RadialMenu\error.log"
Type: files; Name: "{localappdata}\RadialMenu\settings-error.log"

[Code]
const
  HostClass = 'RadialMenuHostWindow';
  WM_CLOSE = $0010;

function FindWindowW(lpClassName, lpWindowName: String): LongWord;
  external 'FindWindowW@user32.dll stdcall';
function PostMessageW(hWnd: LongWord; Msg: LongWord; wParam: LongWord; lParam: LongWord): Boolean;
  external 'PostMessageW@user32.dll stdcall';

function IsFreshInstall: Boolean;
var
  Dummy: String;
begin
  { AppId の {{ は [Setup] 用のエスケープ。レジストリ上のキー名は { 1個 }
  Result := not RegQueryStringValue(HKLM,
    'Software\Microsoft\Windows\CurrentVersion\Uninstall\{6B1F3C5E-2D8A-4E57-9A41-7C3D0E5B8F21}_is1',
    'UninstallString', Dummy);
end;

{ 常駐本体にウィンドウを閉じるメッセージを送って終了させ、最大 5 秒待つ }
procedure StopHost;
var
  H: LongWord;
  I: Integer;
begin
  H := FindWindowW(HostClass, '');
  if H = 0 then Exit;
  PostMessageW(H, WM_CLOSE, 0, 0);
  for I := 1 to 50 do
  begin
    Sleep(100);
    if FindWindowW(HostClass, '') = 0 then Exit;
  end;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  StopHost;
  Result := '';
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usUninstall then StopHost;
end;
