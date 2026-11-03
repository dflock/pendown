# Installs what building Pendown needs: Git, the Visual Studio C++ build
# tools with the Windows SDK, and Rust. Run once, in PowerShell:
#   powershell -ExecutionPolicy Bypass -File C:\OEM\setup.ps1
$ErrorActionPreference = "Stop"

winget install --id Git.Git --exact --accept-source-agreements --accept-package-agreements
# The C++ tools and, with the recommended parts, the Windows SDK, which has
# fxc.exe for gpui's shaders. This one takes a while.
winget install --id Microsoft.VisualStudio.2022.BuildTools --exact `
    --accept-source-agreements --accept-package-agreements `
    --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
winget install --id Rustlang.Rustup --exact --accept-source-agreements --accept-package-agreements

$git = "C:\Program Files\Git\cmd\git.exe"
# Keep the repo's line endings: the tests compare files byte for byte.
& $git config --global core.autocrlf false
# The shared repo belongs to another user, as Windows sees it.
& $git config --global --add safe.directory "*"

& "$env:USERPROFILE\.cargo\bin\rustup.exe" default stable

Write-Host "Done. Open a new PowerShell, then run Z:\pendown\scripts\windows-vm\build.ps1"
