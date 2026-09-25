# syntax=docker/dockerfile:1
FROM scratch
COPY --chown=0:0 env-guard.sh /opt/sandbox/env-guard/env-guard.sh
