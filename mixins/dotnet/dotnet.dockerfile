# syntax=docker/dockerfile:1
FROM scratch
ENV DOTNET_CLI_TELEMETRY_OPTOUT=1 \
    DOTNET_NOLOGO=1
