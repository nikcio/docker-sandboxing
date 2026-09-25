# syntax=docker/dockerfile:1

# Stack toolchain comes from the published template image (built from
# ../template-dotnet/Dockerfile by the publish-image workflow), plus the
# sandbox entrypoint shim.
#
# The shim (../kit/dockerrun/sandbox-entrypoint.sh) runs the composed
# mixins' gating checks before the agent starts — v3 lifecycle startup
# hooks race the agent, so the .env guard has to sit in the launch path
# instead. It execs the original launch command at the end, so workloads
# without the mixin scripts behave exactly like the base image.
# x-release-please-start-version
FROM docker.io/nikcio/opencode-dotnet:v2.2.0
# x-release-please-end-version
USER root
COPY dockerrun/*.sh /opt/sandbox/bin/
RUN chmod 755 /opt/sandbox/bin/*.sh \
    && chown root:root /opt/sandbox/bin/*.sh
USER agent
ENTRYPOINT ["/opt/sandbox/bin/sandbox-entrypoint.sh", "tini", "--", "opencode"]
CMD []
