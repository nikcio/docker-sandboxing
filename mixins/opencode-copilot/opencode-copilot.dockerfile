# syntax=docker/dockerfile:1
FROM scratch
COPY --chown=1000:1000 files/home/.config/opencode/mixins.d/10-copilot.json /home/agent/.config/opencode/mixins.d/10-copilot.json
