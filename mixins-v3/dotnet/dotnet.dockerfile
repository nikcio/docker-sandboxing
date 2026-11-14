# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG DOTNET_VERSION_ARG

# The SDK root lives under /opt (not /usr/share/dotnet, the v2 layout):
# a mixin's layers are an overlay on an unknown base, and a distro-SDK
# base may already own the canonical path.
RUN set -e; \
    apt-get update; \
    apt-get install -y --no-install-recommends ca-certificates curl xz-utils; \
    curl -fsSL https://dot.net/v1/dotnet-install.sh -o /tmp/dotnet-install.sh; \
    chmod +x /tmp/dotnet-install.sh; \
    if [ -n "${DOTNET_VERSION_ARG}" ]; then \
      /tmp/dotnet-install.sh --version "${DOTNET_VERSION_ARG}" --install-dir /opt/dotnet; \
    else \
      # Pin LTS explicitly so a future default-channel change upstream
      # cannot silently change what "no version" installs.
      /tmp/dotnet-install.sh --channel LTS --install-dir /opt/dotnet; \
    fi; \
    rm -f /tmp/dotnet-install.sh; \
    ln -sf /opt/dotnet/dotnet /usr/local/bin/dotnet; \
    mkdir -p /opt/dotnet/tools; \
    # The slim build stage has no ICU; the sandbox base does, so runtime
    # keeps full globalization and only this check runs invariant.
    DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1 dotnet --version

FROM scratch

COPY --from=build /opt/dotnet /opt/dotnet
COPY --from=build /usr/local/bin/dotnet /usr/local/bin/dotnet

COPY <<'EOF' /etc/profile.d/sbx-v3-dotnet.sh
export DOTNET_ROOT=/opt/dotnet
export PATH=$PATH:/opt/dotnet/tools
export DOTNET_CLI_TELEMETRY_OPTOUT=1
export DOTNET_NOLOGO=1
# Skips the first-run ASP.NET developer-certificate generation and NuGet
# extracting XML doc files during restore.
export DOTNET_GENERATE_ASPNET_CERTIFICATE=false
export NUGET_XMLDOC_MODE=skip
EOF

ENV DOTNET_CLI_TELEMETRY_OPTOUT=1 \
    DOTNET_NOLOGO=1 \
    DOTNET_GENERATE_ASPNET_CERTIFICATE=false \
    NUGET_XMLDOC_MODE=skip
