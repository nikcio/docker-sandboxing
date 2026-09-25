# syntax=docker/dockerfile:1

# The stock template image (ENTRYPOINT tini -- / CMD opencode) plus the
# sandbox entrypoint shim.
#
# The shim (kit/dockerrun/sandbox-entrypoint.sh) runs the composed mixins'
# gating checks before the agent starts — v3 lifecycle startup hooks race
# the agent, so the .env guard has to sit in the launch path instead. It
# execs the original launch command at the end, so workloads without the
# mixin scripts behave exactly like the base image.
FROM docker/sandbox-templates:opencode-docker
USER root
COPY dockerrun/*.sh /opt/sandbox/bin/
RUN chmod 755 /opt/sandbox/bin/*.sh \
    && chown root:root /opt/sandbox/bin/*.sh
USER agent
ENTRYPOINT ["/opt/sandbox/bin/sandbox-entrypoint.sh", "tini", "--", "opencode"]
CMD []
