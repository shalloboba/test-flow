# Radial Menu インストーラーを一括ビルドする (Windows / PowerShell)
# 必要: Rust (cargo), .NET 8 SDK, Inno Setup 6 (iscc.exe)
# 使い方:
#   .\build-installer.ps1                 # 通常版
#   .\build-installer.ps1 -UiAccess       # UIAccess 版 (署名が必要。下の注意を参照)
#   .\build-installer.ps1 -UpdateUrl https://github.com/OWNER/REPO/releases/latest/download
#   .\build-installer.ps1 -CertThumbprint <拇印>   # signtool で署名(UIAccess 版は必須)
param(
    [switch]$UiAccess,
    [string]$UpdateUrl = "",
    [string]$CertThumbprint = ""
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

function Find-Iscc {
    $c = Get-Command iscc.exe -ErrorAction SilentlyContinue
    if ($c) { return $c.Source }
    foreach ($p in @("$env:ProgramFiles(x86)\Inno Setup 6\ISCC.exe", "$env:ProgramFiles\Inno Setup 6\ISCC.exe",
                     "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe")) {
        if (Test-Path $p) { return $p }
    }
    throw "Inno Setup 6 (ISCC.exe) が見つかりません。https://jrsoftware.org/isdl.php から入れてください。"
}
function Sign($file) {
    if ($CertThumbprint) { signtool sign /sha1 $CertThumbprint /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $file }
}

# 1) 本体 (Rust)
if ($UiAccess) { $env:RADIAL_UIACCESS = "1"; if (-not $CertThumbprint) { Write-Warning "UIAccess 版は署名しないと起動しません (-CertThumbprint)。" } }
else { Remove-Item Env:RADIAL_UIACCESS -ErrorAction SilentlyContinue }
if ($UpdateUrl) { $env:RADIAL_UPDATE_URL = $UpdateUrl }
cargo build --release
Sign "$root\target\release\RadialMenu.exe"

# 2) 設定アプリ (WinUI 3)
Push-Location settings-winui
dotnet publish -c Release
Pop-Location
Sign "$root\settings-winui\bin\Release\net8.0-windows10.0.19041.0\win-x64\publish\RadialMenu.Settings.exe"

# 3) インストーラー
& (Find-Iscc) "$PSScriptRoot\RadialMenu.iss"
$setup = "$PSScriptRoot\output\RadialMenu-Setup.exe"
Sign $setup

# 4) latest.json (更新元に置く)
$ver = (Select-String -Path "$PSScriptRoot\RadialMenu.iss" -Pattern '#define MyAppVersion\s+"([^"]+)"').Matches[0].Groups[1].Value
$sha = (Get-FileHash $setup -Algorithm SHA256).Hash.ToLower()
$json = [ordered]@{ version = $ver; url = "RadialMenu-Setup.exe"; sha256 = $sha } | ConvertTo-Json
[IO.File]::WriteAllText("$PSScriptRoot\output\latest.json", $json, (New-Object Text.UTF8Encoding($false)))

Write-Host ""
Write-Host "完成: $setup"
Write-Host "      $PSScriptRoot\output\latest.json  (sha256 $sha)"
