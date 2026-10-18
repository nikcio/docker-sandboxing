# Sandy

A desktop app for [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/) with guided local setup and a live view of your sandboxes, powered by the `sbx` CLI.

![Sandy](../sandy/src-tauri/icons/icon.png)

## What it does

### Browsing sandboxes

- **Home** combines a live local sandbox list, action cards for creation, templates, images, and help, and an inspector for the selected sandbox.
- **Sandboxes** shows the local sandboxes returned by `sbx ls`, including their reported state, agent, and workspace. Search by name, agent, or workspace; keep CLI order or sort by name. Auto-refresh polls every 5 seconds and can be toggled in the inspector; manual refresh is also available.
- **Templates** is a coming-soon page linking to environment examples.
- Creation (including from an image), open, start, stop, restart, duplicate/create-from-sandbox, more-actions, and account controls are placeholders that show a coming-soon notice. Use the CLI for sandbox management; these controls do not perform those operations.

### Guided setup in Settings

When setup is incomplete, Sandy automatically opens **Settings** at the first missing step, skipping already configured steps on entry. Each step shows what it will run and performs its setup action only when requested:

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
- To create a sandbox, start with the [`sbxenv.yaml` examples](../README.md#use-it-in-your-project) and run `sbx env run` in your project.
