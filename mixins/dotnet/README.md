# dotnet

.NET SDK + NuGet egress for the sandbox: the toolchain install (when the base image lacks it), telemetry opt-out, and every domain NuGet and the SDK need.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/dotnet   # .NET / NuGet
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/dotnet <agent> .`

The install runs at sandbox creation only and needs egress. It uses the domains in the table below plus the Ubuntu apt mirrors (`archive.ubuntu.com`/`security.ubuntu.com`) for its `apt-get` calls. On base images that already ship the SDK the install is a no-op.

## How it works

- **Check-and-install.** When `dotnet` is missing, tries the apt feed (`dotnet-sdk-<channel>`) first and falls back to the official `dot.net/v1/dotnet-install.sh` script into `/usr/share/dotnet`, the same layout as the base images.
- **Env.** Sets `DOTNET_CLI_TELEMETRY_OPTOUT` and `DOTNET_NOLOGO` and skips the first-run ASP.NET certificate generation and NuGet XML docs.
- `dotnet` and its tools path are exported via `/etc/sandbox-persistent.sh` so login shells pick them up.

## Version arguments

| Arg | Default | Description |
| --- | ------- | ----------- |
| `dotnetVersion` | *(empty = latest SDK)* | .NET SDK version to install in `MAJOR.MINOR.PATCH` form. The first two segments select the apt channel |

Pass values with `--kit-arg dotnet.dotnetVersion=<version>` (or an `args:` entry on the mixin's `kits:` line in `sbxenv.yaml`); see [Version arguments](../../docs/mixins.md#version-arguments).

## Network domains

| Domain | Why |
| ------ | --- |
| `nuget.org`, `*.nuget.org` | NuGet restore, push, search |
| `*.microsoft.com` | SDK/workload downloads, MS Build hosts, packages.microsoft.com |
| `*.dotnet.microsoft.com` | SDK download CDN (builds.dotnet.microsoft.com) |
| `dot.net`, `*.dot.net` | dotnet-install script + feeds |
| `aka.ms` | Short links the install script resolves download URLs through |
| `*.azureedge.net` | Legacy download CDNs |
