# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG CODEGEN_VERSION_ARG
ARG DOTNET_VERSION_ARG

# Same staged wrapper as the dotnet mixin (a Dockerfile RUN is line-based,
# so a multi-line sh -c body can't ride inline).
COPY <<'EOF' /tmp/sbx-dotnet-install-wrapper.sh
set -e
curl -fsSL https://dot.net/v1/dotnet-install.sh -o /tmp/sbx-dotnet-install.sh
# bash 5.3 (Ubuntu 26.04) + set -u: the installer appends empty values
# unquoted, which silently skips array elements, and its verbose
# diagnostics then read missing array indexes. Quote the appends and
# default the index reads so the script works on every bash.
sed -i -e "s/effective_versions+=(\$effective_version)/effective_versions+=(\"\${effective_version-}\")/" \
       -e "s/specific_versions+=(\$specific_version)/specific_versions+=(\"\${specific_version-}\")/" \
       -e "s/\${link_types\[\$link_index\]}/\${link_types[\$link_index]:-}/g" \
       -e "s/\${effective_versions\[\$link_index\]}/\${effective_versions[\$link_index]:-}/g" \
       -e "s/\${specific_versions\[\$link_index\]}/\${specific_versions[\$link_index]:-}/g" \
       /tmp/sbx-dotnet-install.sh
chmod +x /tmp/sbx-dotnet-install.sh
/tmp/sbx-dotnet-install.sh --version "${DOTNET_VERSION}" --install-dir "$DOTNET_ROOT"
rm -f /tmp/sbx-dotnet-install.sh
ln -sf "$DOTNET_ROOT/dotnet" /usr/local/bin/dotnet
mkdir -p "$DOTNET_ROOT/tools"
EOF

RUN set -e; install -m 0755 -d /var/lib/apt/sbx-install-lists; \
    export DEBIAN_FRONTEND=noninteractive; \
    { apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists update -qq || true; }; \
    apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists install -y -qq --no-install-recommends ca-certificates curl xz-utils; \
    DOTNET_VERSION="${DOTNET_VERSION_ARG}"; \
    if [ -z "${DOTNET_VERSION}" ]; then \
      DOTNET_CHANNEL="$(curl -fsSL 'https://builds.dotnet.microsoft.com/dotnet/release-metadata/releases-index.json' | grep -o '"channel-version": *"[0-9.]*"\|"support-phase": *"[a-z-]*"' | paste - - | grep -v '"go-live"\|"eol"\|"preview"' | head -n 1 | grep -o '[0-9]*\.[0-9]*')"; \
      DOTNET_VERSION="$(curl -fsSL "https://builds.dotnet.microsoft.com/dotnet/release-metadata/${DOTNET_CHANNEL}/releases.json" | grep -o '"latest-sdk": *"[^"]*"' | head -n 1 | grep -o '[0-9][0-9.]*')"; \
    else \
      DOTNET_CHANNEL="$(echo "${DOTNET_VERSION}" | cut -d. -f1-2)"; \
    fi; \
    export DOTNET_VERSION DOTNET_ROOT=/opt/dotnet \
           DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_NOLOGO=1 \
           DOTNET_GENERATE_ASPNET_CERTIFICATE=false NUGET_XMLDOC_MODE=skip; \
    sh /tmp/sbx-dotnet-install-wrapper.sh; \
    rm -f /tmp/sbx-dotnet-install-wrapper.sh; \
    export PATH=$PATH:/opt/dotnet/tools; \
    hash -r; \
    # The slim build stage has no ICU; the sandbox base does, so runtime
    # keeps full globalization and only these build steps run invariant.
    export DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1; \
    dotnet --version; \
    CG_SPEC="${CODEGEN_VERSION_ARG}"; \
    if [ -n "${CG_SPEC}" ]; then CG_SPEC="--version ${CG_SPEC}"; fi; \
    dotnet tool install --global Nikcio.OpenApiCodeGen ${CG_SPEC}

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
