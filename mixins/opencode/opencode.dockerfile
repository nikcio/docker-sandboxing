# syntax=docker/dockerfile:1
FROM scratch
COPY --chown=1000:1000 files/home/.config/opencode/opencode.jsonc /usr/share/sandbox/opencode/opencode.jsonc
