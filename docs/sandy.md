# Sandy

A friendly desktop app for [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/): it installs and configures everything you need, then shows your running sandboxes in one place. Built for people who don't want to touch a terminal.

![Sandy](../sandy/src-tauri/icons/icon.png)

## What it does

**First-run setup wizard** (one time, re-runnable anytime from the Setup button):

1. **Installs the `sbx` CLI** — the tool that creates and manages sandboxes — using your platform's package manager:
   - macOS: Homebrew (`brew install docker/tap/sbx`)
   - Windows: WinGet (`winget install -h Docker.sbx`)
   - Ubuntu: Docker's apt repository + the `docker-sbx` package, and adds your user to the `kvm` group
2. **Signs you in to Docker** — runs `sbx login`, which opens a browser.
3. **Enables image pasting** — runs `sbx settings set clipboard.imagePaste true` so you can paste screenshots into agent chats with Ctrl+V.
4. **Sets up SSH** — runs `sbx setup ssh` so every sandbox is reachable as `<name>.sbx` from your terminal or editor.

**Command center** (the main window): lists your running local sandboxes (`sbx ls`) with name, state, agent, and workspace, refreshing automatically every 5 seconds.

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

Packages are pinned exactly and installs reject anything published in the last 24 hours (`.npmrc`) as supply-chain protection.

## Notes

- Sandy is a thin layer on top of the `sbx` CLI: it runs `sbx` commands and shows the output. Nothing is configured behind your back — each wizard step tells you exactly what will run.
- The wizard can be re-run at any time from the command center (Setup button), for example to re-do SSH setup after changing machines.
- Creating, connecting to, and recreating sandboxes from inside Sandy is planned for a future version; use [`sbxenv.yaml` examples](../README.md#use-it-in-your-project) with `sbx env run` for now.
