## Python / uv

- Python is uv-managed (`python`/`python3` resolve to the interpreter baked
  into the image)
- Prefer uv for packages and venvs: `uv add`, `uv pip`, `uv venv`, `uv run`
- uv updates: rerun the installer —
  `curl -LsSf https://astral.sh/uv/install.sh | sh` (installs user-level
  to `~/.local/bin`, shadowing the baked uv; binaries come from GitHub
  releases, so compose the `git` mixin)
- Other interpreters: `uv python install <version>` — downloads from GitHub
  releases, so compose the `git` mixin (github.com + *.githubusercontent.com)
