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
function Find-MSBuild {
    $c = Get-Command msbuild.exe -ErrorAction SilentlyContinue
    if ($c) { return $c.Source }
    $vw = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vw) {
        $p = & $vw -latest -requires Microsoft.Component.MSBuild -find 'MSBuild\**\Bin\MSBuild.exe' | Select-Object -First 1
        if ($p) { return $p }
    }
    return $null
}
function Sign($file) {
    if ($CertThumbprint) { signtool sign /sha1 $CertThumbprint /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $file }
}

# 1) 本体 (Rust)
if ($UiAccess) { $env:RADIAL_UIACCESS = "1"; if (-not $CertThumbprint) { Write-Warning "UIAccess 版は署名しないと起動しません (-CertThumbprint)。" } }
else { Remove-Item Env:RADIAL_UIACCESS -ErrorAction SilentlyContinue }
if ($UpdateUrl) { $env:RADIAL_UPDATE_URL = $UpdateUrl }
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "本体 (cargo build) が失敗しました。" }
Sign "$root\target\release\RadialMenu.exe"

# 2) 設定アプリ (WinUI 3)
Push-Location settings-winui
# WinUI 3 は dotnet CLI だと XAML コンパイラーのエラー内容が隠れるため、Visual Studio の MSBuild を優先する
$msb = Find-MSBuild
if ($msb) {
    Write-Host "MSBuild: $msb"
    & $msb RadialMenu.Settings.csproj /restore /t:Publish /p:Configuration=Release /p:RuntimeIdentifier=win-x64 /v:minimal
} else {
    dotnet publish -c Release
}
$dotnetExit = $LASTEXITCODE
if ($dotnetExit -ne 0) {
    # XAML コンパイラーの本当のエラーは output.json に書かれる
    Get-ChildItem -Path . -Recurse -Filter output.json -ErrorAction SilentlyContinue | ForEach-Object {
        Write-Host "---- $($_.FullName) ----"
        Get-Content $_.FullName -Raw | Write-Host
    }
}
Pop-Location
if ($dotnetExit -ne 0) { throw "設定アプリ (dotnet publish) が失敗しました。上の output.json のエラーを確認してください。" }
Sign "$root\settings-winui\bin\Release\net8.0-windows10.0.22621.0\win-x64\publish\RadialMenu.Settings.exe"

# 3) インストーラー
& (Find-Iscc) "$PSScriptRoot\RadialMenu.iss"
if ($LASTEXITCODE -ne 0) { throw "インストーラー作成 (iscc) が失敗しました。" }
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
