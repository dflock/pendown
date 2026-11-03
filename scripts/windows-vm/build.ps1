# Clones the shared repo into C:\src\pendown, or pulls if it's there, then
# builds Pendown in release, copies the .exe files to Z:\windows-build (on
# Linux, shared/windows-build in the folder PENDOWN_VM_DIR names) and starts
# it. It builds main as committed; pass a branch to build another:
#   powershell -ExecutionPolicy Bypass -File Z:\pendown\scripts\windows-vm\build.ps1 [branch]
param([string]$Branch = "main")
$ErrorActionPreference = "Stop"

$repo = "C:\src\pendown"
if (Test-Path $repo) {
    # The repo used to be Z: itself.
    git -C $repo remote set-url origin Z:\pendown
    git -C $repo fetch origin
} else {
    git clone Z:\pendown $repo
}
git -C $repo checkout $Branch
git -C $repo merge --ff-only "origin/$Branch"

Set-Location $repo
cargo build --release -p pendown -p pendown-cli
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$out = "Z:\windows-build"
New-Item -ItemType Directory -Force $out | Out-Null
Copy-Item target\release\pendown.exe, target\release\pendown-cli.exe $out
Write-Host "Built pendown.exe and pendown-cli.exe, and copied them to $out"
Start-Process "$repo\target\release\pendown.exe"
