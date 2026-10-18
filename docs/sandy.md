# Sandy

A desktop app for [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/) with guided setup, reusable environment templates, and local sandbox management, powered by the `sbx` CLI.

![Sandy](../sandy/src-tauri/icons/icon.png)

## What it does

### Browsing sandboxes

- **Home** and **Sandboxes** show local sandbox state, agent, workspace, and an inspector. Search by name, agent, or workspace; keep CLI order or sort by name.
- Start, stop, or restart a selected sandbox. Starting resumes its existing agent, kits, files, and workspace; restarting stops it first, then resumes it.
- The list always refreshes every 5 seconds while the dashboard is open, when Sandy regains focus, and after actions. **Refresh** is also available; refresh errors keep the last successful list visible.

### Create and save an environment

1. Choose **Create a sandbox**. Enable kit providers: GitHub repositories, Docker Hub kit search, or specific Docker Hub repositories. Fetch versions or enter a reference, save provider changes, then choose **Discover kits**.
2. Select a **v2 agent** (including built-in agents) or a **v3 workload**. Sandy offers compatible mixins from enabled providers, matching the kit generation and any required agent. V2 and v3 kits cannot be combined.
3. Complete kit inputs, choose an existing workspace folder, and name the sandbox. Review the selected references and YAML preview.
4. Choose **Save template** or **Save & launch**. Templates are real `sbxenv.yaml` files in Sandy's per-user application-data directory by default, or at a file location you choose **outside the workspace**. Both kit generations use environment schema v1. The saved file path and YAML are shown after saving.

Open **Templates** to review saved YAML and launch it with a different name or workspace. Each new launch keeps the saved template intact and creates a separate launch file outside the workspace. Reusing an existing sandbox name resumes that sandbox's saved configuration; use a new name to create a sandbox with different tools or a different workspace.

### Connect your terminal or editor

Select a running sandbox and choose **Open**:

- **Terminal** opens your computer's terminal with `sbx exec -it NAME /bin/bash`. Sandy uses Terminal on macOS, Windows Terminal or PowerShell on Windows, and a supported terminal emulator on Linux. SSH setup is not required.
- **OpenCode Desktop** shows the steps for **Settings → Server → Add SSH Server** and provides the sandbox host and SSH user.
- **VS Code** shows how to install **Remote – SSH**, choose **Connect to Host…**, and open the project folder inside the sandbox.

The editor guides include **Set up SSH**, setup checks, and copyable connection details. Hosts are `<name>.sbx`, with SSH user `_default_user_`; leave the port unset so the CLI-managed SSH configuration handles the connection over the local daemon. See Docker's [editor and app integrations](https://docs.docker.com/ai/sandboxes/integrations/).

### Guided setup in Settings

When a setup check reports an incomplete step, Sandy automatically opens **Settings** at the first missing step, skipping already configured steps on entry. Unavailable checks show their errors and retain the last known state until refreshed. Each step shows what it will run and performs its setup action only when requested:

1. **Installs the `sbx` CLI** — the tool that creates and manages sandboxes — using your platform's package manager:
   - macOS: Homebrew (`brew install docker/tap/sbx`)
   - Windows: WinGet (`winget install -h Docker.sbx`)
   - Ubuntu: Docker's apt repository + the `docker-sbx` package, and adds your user to the `kvm` group
2. **Signs you in to Docker** — runs `sbx login`, which opens a browser.
3. **Enables image pasting** — runs `sbx settings set clipboard.imagePaste true` so you can paste screenshots into agent chats with Ctrl+V.
4. **Sets up SSH** — runs `sbx setup ssh` so every sandbox is reachable as `<name>.sbx` from your terminal or editor.

Use **Skip for now** to skip a step or **Set up later** to defer setup and browse Home. Errors show details and a **Try again** action. The finish summary shows what is ready and what remains for later. Return to **Settings** to continue incomplete setup or **Review setup**; completed steps can be selected and explicitly run again.

## Installing Sandy

Download the installer for your platform from the [releases page](https://github.com/nikcio/docker-sandboxing/releases) — Sandy is built and attached automatically whenever a release is published:

| Platform | File |
| -------- | ---- |
| macOS (Apple silicon) | `.dmg` |
| Windows | `.msi` (or the NSIS `.exe` setup) |
| Linux | `.AppImage` or `.deb` |

## Building from source

Requirements: Node.js 22+, [pnpm](https://pnpm.io) (`corepack enable`), Rust (stable), and the [Tauri system dependencies](https://tauri.app/start/prerequisites/) for your platform. On Ubuntu:

```bash
sudo apt install libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev libxdo-dev file pkg-config libdbus-1-dev
```

```bash
cd sandy
pnpm install
pnpm tauri dev    # run in development
pnpm tauri build  # produce installers
```

Packages are pinned exactly and installs reject anything published in the last 24 hours (`pnpm-workspace.yaml`) as supply-chain protection.

### Windows: `link.exe` errors

If `pnpm tauri dev` on Windows fails with `link: extra operand '…'`, the wrong `link.exe` was found on your PATH — Git Bash ships a Unix `link` tool with the same name as the MSVC linker. Build from a developer shell instead so the MSVC toolchain comes first:

- **PowerShell**: `Import-Module "C:\Program Files\Microsoft Visual Studio\18\BuildTools\Common7\Tools\Microsoft.VisualStudio.DevShell.dll"; Enter-VsDevShell -VsInstallPath "C:\Program Files\Microsoft Visual Studio\18\BuildTools" -SkipAutomaticLocation` (adjust `18\BuildTools` to your edition — find the folder under `C:\Program Files\Microsoft Visual Studio\`), or
- launch the **x64 Native Tools Command Prompt for VS** from the Start menu and run the commands there.

The VS Build Tools installer must include the **Desktop development with C++** workload (`winget install Microsoft.VisualStudio.BuildTools`, currently major version 18 / "Visual Studio 2026").

## Notes

- Sandbox names, states, agents, and workspaces come from the local `sbx` CLI.
- Sandy checks PATH and standard Homebrew/Windows installer locations for `sbx`. Custom installations should put the native CLI on PATH.
- Local sandbox launches require your platform's virtualization support; see Docker's [installation prerequisites](https://docs.docker.com/ai/sandboxes/install/).
- For a CLI-first workflow, use the [`sbxenv.yaml` examples](../README.md#use-it-in-your-project) and run `sbx env run` in your project.
