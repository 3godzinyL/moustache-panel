param(
  [string]$BuildRoot = "src-tauri\target\release",
  [string]$Publisher = "CN=REPLACE_WITH_YOUR_CERTIFICATE_SUBJECT",
  [string]$CertificatePath = "",
  [string]$CertificatePassword = ""
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$Layout = Join-Path $Root "packaging\msix\layout"
$ManifestTemplate = Join-Path $Root "packaging\msix\AppxManifest.xml"
$ExeCandidates = @(
  (Join-Path $Root "$BuildRoot\moustache-control-center.exe"),
  (Join-Path $Root "$BuildRoot\Moustache Control Center.exe")
)
$Exe = $ExeCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
$PackagedExe = "Moustache Control Center.exe"
$Dll = Join-Path $Root "src-tauri\resources\moustache_native.dll"
$Out = Join-Path $Root "src-tauri\target\release\bundle\msix\MoustacheControlCenter.msix"

if (!$Exe) {
  throw "Release executable not found. Checked: $($ExeCandidates -join ', '). Run npm run build first."
}
if (!(Test-Path $Dll)) {
  throw "Native DLL not found: $Dll"
}

Remove-Item $Layout -Recurse -Force -ErrorAction SilentlyContinue
New-Item $Layout -ItemType Directory | Out-Null
New-Item (Join-Path $Layout "Assets") -ItemType Directory | Out-Null

Copy-Item $Exe (Join-Path $Layout $PackagedExe)
Copy-Item $Dll $Layout

$Manifest = (Get-Content $ManifestTemplate -Raw).Replace(
  "CN=REPLACE_WITH_YOUR_CERTIFICATE_SUBJECT",
  $Publisher
)
Set-Content (Join-Path $Layout "AppxManifest.xml") $Manifest -Encoding UTF8

$AssetSource = Join-Path $Root "packaging\msix\Assets\*"
Copy-Item $AssetSource (Join-Path $Layout "Assets") -Recurse -Force

New-Item (Split-Path $Out) -ItemType Directory -Force | Out-Null
& makeappx.exe pack /d $Layout /p $Out /o

if ($CertificatePath) {
  & signtool.exe sign /fd SHA256 /f $CertificatePath /p $CertificatePassword $Out
}

Write-Host "MSIX created: $Out"
