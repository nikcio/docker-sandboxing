# dotnet (v3)

.NET SDK baked into the mixin layer at image build (official install
script, independent of the base's distro channel), with NuGet egress. v3
port of [`../dotnet`](../dotnet) — read that for the full behavior story;
see [`../README.md`](../README.md) for what the v3 format changes.

## Build / use

```bash
docker buildx build -f mixins-v3/dotnet/dotnet.yaml -t my/dotnet-mixin:dev mixins-v3/dotnet/
sbx run <agent> . --kit ./mixins-v3/dotnet   # or the published image reference
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `dotnetVersion` | *(empty = latest)* | .NET SDK version (`MAJOR.MINOR.PATCH`) |

## Runtime egress

`nuget.org` + wildcard, `*.microsoft.com`, `*.dotnet.microsoft.com`,
`dot.net` + wildcard, `aka.ms`, `*.azureedge.net`. No install-phase grant.

Telemetry opt-out (`DOTNET_CLI_TELEMETRY_OPTOUT`, `DOTNET_NOLOGO`) ships
as image environment.
