# Builds release binaries with HEIC support and produces both Windows downloads in converter/dist:
#   Converter-<ver>-x64-portable.zip   (exe + CLI + DLLs + licences, runs from anywhere)
#   Converter_<ver>_x64-setup.exe      (NSIS, per-user install, via cargo-packager)
# Needs: VCPKG_ROOT pointing at a vcpkg with `libheif[core]:x64-windows` installed, and cargo-packager.
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
if (-not $env:VCPKG_ROOT) { throw 'Set VCPKG_ROOT to your vcpkg folder (with libheif[core]:x64-windows installed).' }
$env:VCPKGRS_DYNAMIC = '1'

cargo build --release -p converter-gui --features heic
if ($LASTEXITCODE) { throw 'GUI build failed' }
cargo build --release -p converter-cli --features converter-core/heic
if ($LASTEXITCODE) { throw 'CLI build failed' }

$version = [regex]::Match((cargo pkgid -p converter-gui), '[#@]([0-9][^#@]*)$').Groups[1].Value
if (-not $version) { throw 'Could not read the package version' }
$stage = Join-Path $root 'dist/stage'
Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $stage | Out-Null
$dlls = Join-Path $env:VCPKG_ROOT 'installed/x64-windows/bin'
Copy-Item (Join-Path $dlls 'heif.dll'), (Join-Path $dlls 'libde265.dll') $stage
Copy-Item target/release/converter-cli.exe, LICENSE-MIT, LICENSE-APACHE $stage
@"
Converter $version

Converter is licensed under MIT OR Apache-2.0 (see LICENSE-MIT, LICENSE-APACHE).

heif.dll (libheif) and libde265.dll (libde265) are licensed under the GNU LGPL v3
and are shipped as separate libraries: you may replace them with your own builds.
Source: https://github.com/strukturag/libheif and https://github.com/strukturag/libde265
"@ | Set-Content -Encoding utf8 (Join-Path $stage 'THIRD-PARTY.txt')

# Portable zip.
$zipDir = Join-Path $root "dist/Converter-$version-x64-portable"
Remove-Item -Recurse -Force $zipDir, "$zipDir.zip" -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $zipDir | Out-Null
Copy-Item target/release/Converter.exe, "$stage/*" $zipDir
Compress-Archive -Path "$zipDir/*" -DestinationPath "$zipDir.zip"
Remove-Item -Recurse -Force $zipDir

# Installer.
Push-Location crates/converter-gui
cargo packager --release -f nsis
if ($LASTEXITCODE) { Pop-Location; throw 'cargo packager failed' }
Pop-Location
Get-ChildItem dist -File | Format-Table Name, Length
