# A Windows VM for building Pendown

This builds and runs Pendown on Windows 11 from a Linux machine, in a VM inside Docker, using [dockur/windows](https://github.com/dockur/windows). It needs Docker with Compose and KVM (`/dev/kvm`).

| File | What it does |
| --- | --- |
| `docker-compose.yml` | Defines the VM, and what it shares with Linux |
| `oem/setup.ps1` | Installs Git, the Visual Studio C++ build tools with the Windows SDK, and Rust |
| `build.ps1` | Builds Pendown in the VM, copies the `.exe` files out and starts it |

## Choose where the VM lives

Windows' disk grows to tens of gigabytes (the limit is 128 GB), so it's kept outside the repo. Choose a folder and name it in `.env` here, which Git ignores. Use a full path:

```sh
cd scripts/windows-vm
echo "PENDOWN_VM_DIR=$HOME/vms/pendown-windows" > .env
```

Compose won't start without it. The folder ends up holding:

| Path | What it is |
| --- | --- |
| `storage/` | Windows' disk. Delete it to start again with a fresh Windows |
| `shared/windows-build/` | The built `pendown.exe` and `pendown-cli.exe` |
| `shared/pendown/` | An empty folder, where the repo is mounted. Docker creates it and root owns it |

## Set up Windows

Run these from `scripts/windows-vm`.

1. Start the VM. The first start downloads and installs Windows, which takes a while:

   ```sh
   sudo docker compose up -d
   ```

2. Watch it at <http://127.0.0.1:8006>, or connect with Remote Desktop to `127.0.0.1:3389`. The user and password are both `pendown`. The ports only listen on this machine, so nothing else on the network can reach the VM.

3. When Windows is at its desktop, install the build tools by running this once, in PowerShell. The Visual Studio part is slow:

   ```powershell
   powershell -ExecutionPolicy Bypass -File C:\OEM\setup.ps1
   ```

## Build

In a new PowerShell in the VM:

```powershell
powershell -ExecutionPolicy Bypass -File Z:\pendown\scripts\windows-vm\build.ps1
```

This builds `main`. To build another branch, add its name to the end of that command.

The script clones the repo into `C:\src\pendown` from `Z:\pendown`, a read-only view of your repo on Linux, or fetches if it's already there. It then builds in release, copies the `.exe` files to `Z:\windows-build`, and starts Pendown.

It builds what's committed on the branch, not your working tree, so commit your changes before building them.

## Stop

```sh
sudo docker compose stop
```

`sudo docker compose up -d` starts it again, with Windows and its build tools as you left them.
