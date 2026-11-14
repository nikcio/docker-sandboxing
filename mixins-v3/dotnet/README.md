# dotnet (v3)

.NET SDK baked into the mixin layer at image build (official install
script, independent of the base's distro channel), with NuGet egress.

## Build / use

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin
`&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/dotnet   # .NET / NuGet
```

For local development, point `--kit` at the directory instead —
`sbx run <agent> . --kit ./mixins-v3/dotnet` — or build the image yourself:

```bash
docker buildx build -f mixins-v3/dotnet/dotnet.yaml -t my/dotnet-mixin:dev mixins-v3/dotnet/
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `dotnetVersion` | *(empty = latest)* | .NET SDK version (`MAJOR.MINOR.PATCH`) |

## How it works

- The SDK is installed by the official `dot.net/v1/dotnet-install.sh`
  script into `/opt/dotnet` and symlinked onto `/usr/local/bin`;
  `/opt/dotnet/tools` is added to `PATH` so global tools resolve. With no
  `dotnetVersion`, the newest SDK on a supported channel is installed. The
  versions resolve at build, so the image is a snapshot: rebuild to update.
- Image environment opts out of telemetry (`DOTNET_CLI_TELEMETRY_OPTOUT`,
  `DOTNET_NOLOGO`), skips the first-run ASP.NET developer-certificate
  generation (`DOTNET_GENERATE_ASPNET_CERTIFICATE=false`), and stops NuGet
  extracting XML doc files during restore (`NUGET_XMLDOC_MODE=skip`). A
  `/etc/profile.d/` script additionally exports `DOTNET_ROOT` for login
  shells.
- The `aka.ms` egress rule exists because the install script resolves its
  primary SDK download URLs by following `aka.ms` short links (it silently
  falls back to the CDNs when they are blocked) — relevant if the script
  is re-run in-sandbox.

## Runtime egress

`nuget.org` + wildcard, `*.microsoft.com`, `*.dotnet.microsoft.com`
(SDK download CDN), `dot.net` + wildcard (install script + feeds), `aka.ms`,
`*.azureedge.net` (legacy download CDNs). No install-phase grant: the SDK
is baked at build.
