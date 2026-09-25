# syntax=docker/dockerfile:1
FROM scratch
COPY --chown=1000:1000 files/home/.sandbox-agents.md /home/agent/.sandbox-agents.md
COPY --chown=1000:1000 files/home/.sandbox-agents/ /home/agent/.sandbox-agents/
