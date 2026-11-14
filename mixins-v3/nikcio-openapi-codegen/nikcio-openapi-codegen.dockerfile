# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG CODEGEN_VERSION_ARG
ARG DOTNET_VERSION_ARG

ENV DOTNET_ROOT=/opt/dotnet \
    DOTNET_CLI_TELEMETRY_OPTOUT=1 \
    DOTNET_NOLOGO=1 \
    DOTNET_GENERATE_ASPNET_CERTIFICATE=false \
    NUGET_XMLDOC_MODE=skip \
    # The slim build stage has no ICU; the sandbox base does, so runtime
    # keeps full globalization and only these build steps run invariant.
    DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl

# Pin LTS explicitly so a future default-channel change upstream cannot
# silently change what "no version" installs, and so the SDK's runtime
# matches the bundled tool (targets net10.0). The script runs fine
# unpatched on this stage's bash 5.2: the bash-5.3/set -u quoting fixes
# the dotnet mixin carries are not needed here.
RUN curl -fsSL https://dot.net/v1/dotnet-install.sh -o /tmp/dotnet-install.sh \
 && chmod +x /tmp/dotnet-install.sh \
 && if [ -n "${DOTNET_VERSION_ARG}" ]; then \
      /tmp/dotnet-install.sh --version "${DOTNET_VERSION_ARG}" --install-dir "${DOTNET_ROOT}"; \
    else \
      /tmp/dotnet-install.sh --channel LTS --install-dir "${DOTNET_ROOT}"; \
    fi \
 && ln -sf "${DOTNET_ROOT}/dotnet" /usr/local/bin/dotnet

RUN dotnet --version \
 && if [ -n "${CODEGEN_VERSION_ARG}" ]; then \
      dotnet tool install --global Nikcio.OpenApiCodeGen --version "${CODEGEN_VERSION_ARG}"; \
    else \
      dotnet tool install --global Nikcio.OpenApiCodeGen; \
    fi

FROM scratch

COPY --from=build /opt/dotnet /opt/dotnet
COPY --from=build /root/.dotnet /opt/agent-dotnet-tools
COPY --from=build /usr/local/bin/dotnet /usr/local/bin/dotnet

COPY <<'EOF' /etc/profile.d/sbx-v3-openapi-codegen.sh
export DOTNET_ROOT=/opt/dotnet
export PATH=$PATH:/opt/dotnet/tools:/opt/agent-dotnet-tools/tools
export DOTNET_CLI_TELEMETRY_OPTOUT=1
export DOTNET_NOLOGO=1
export DOTNET_GENERATE_ASPNET_CERTIFICATE=false
export NUGET_XMLDOC_MODE=skip
EOF

ENV DOTNET_CLI_TELEMETRY_OPTOUT=1 \
    DOTNET_NOLOGO=1 \
    DOTNET_GENERATE_ASPNET_CERTIFICATE=false \
    NUGET_XMLDOC_MODE=skip
