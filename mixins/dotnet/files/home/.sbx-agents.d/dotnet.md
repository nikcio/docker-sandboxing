## .NET / NuGet

- The .NET SDK is baked into the image (`dotnet` resolves to
  /usr/share/dotnet; `DOTNET_ROOT` is set, SDK tools are on PATH)
- Manage packages with NuGet: `dotnet add package`, `dotnet restore`,
  `dotnet tool install`
- CLI telemetry is off (DOTNET_CLI_TELEMETRY_OPTOUT) and NuGet skips XML
  doc downloads (NUGET_XMLDOC_MODE=skip)
