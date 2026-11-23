# RadialMenu.Settings (WinUI 3 / C#)

設定画面。常駐本体 `RadialMenu.exe`(Rust) とは別の exe で、同じ `config.json` を読み書きします。
HTML(WebView2) を使わないので、右クリックの「印刷」メニューは出ません。

## できること
- 設定を変えるとすぐ自動保存し、本体へ再読み込みを通知します(再読み込み操作は不要)。
- 段の追加・**削除**・並べ替え、段ごとの**切り替えキー**(自動 / なし / 0-9 / A-Z / F1-F12)。
- 項目の追加・削除・並べ替え、動作(キー送信 / プログラム / URL)、アイコン。
- 全般(有効、長押し時間、サイズ、テーマ、アクセント色、自動起動)、更新確認と更新。

## ビルド (Windows, .NET 8 SDK)
```
cd settings-winui
dotnet publish -c Release
```
出力: `bin\Release\net8.0-windows10.0.19041.0\win-x64\publish\`
(Windows App SDK ランタイム同梱。フォルダーごと配ります)

## 配置
`RadialMenu.exe`(本体) と publish の中身を**同じフォルダー**に置きます。
`RadialMenu.Settings.exe` が本体と同じフォルダー(または `settings\` サブフォルダー)にあれば、
トレイの「設定」・本体の二重起動で開きます。

## 本体(Rust)のビルド
```
cargo build --release
```
UIAccess 版は従来どおり `RADIAL_UIACCESS=1` で作り、署名して Program Files に置きます。
起動に失敗したときは `%LOCALAPPDATA%\RadialMenu\error.log` と、ダイアログに原因が出ます。
設定アプリの例外は `%LOCALAPPDATA%\RadialMenu\settings-error.log` に出ます。

## 更新
更新元 URL の下の `latest.json` を見ます。`url` は Inno Setup のインストーラー(.exe)にしてください。
```
{ "version": "0.3.0", "url": "RadialMenu-Setup.exe", "sha256": "<インストーラーの sha256>" }
```
例: URL = `https://github.com/OWNER/REPO/releases/latest/download`
更新ボタンはインストーラーをダウンロードし、sha256 を確認して `/SILENT /CLOSEAPPLICATIONS /RESTARTAPPLICATIONS` で起動します。
