# dotnet

.NET SDK + NuGet egress for the sandbox: the toolchain install (when the
template image lacks it), telemetry opt-out, and every domain NuGet and
the SDK need.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - docker.io/nikcio/sbx-mixin-dotnet:vX.Y.Z   # .NET / NuGet
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/dotnet <workload> .`

The install runs at **sandbox creation only** and needs egress — the
domains in the table below plus the Ubuntu apt mirrors
(`archive.ubuntu.com`/`security.ubuntu.com`) for its `apt-get` calls. On
template images that already ship the SDK the install is a cheap no-op.

## How it works

- **Check-and-install**: when `dotnet` is missing, tries the apt feed
  (`dotnet-sdk-10.0`) first and falls back to the official
  `dot.net/v1/dotnet-install.sh` script into `/usr/share/dotnet` — the
  same layout as the template images.
- **Env**: sets `DOTNET_CLI_TELEMETRY_OPTOUT` and `DOTNET_NOLOGO` and
  skips the first-run ASP.NET certificate generation and NuGet XML docs.
- `dotnet` and its tools path are exported via `/etc/sandbox-persistent.sh`
  so login shells pick them up.

## Network domains

| Domain | Why |
| ------ | --- |
| `nuget.org`, `*.nuget.org` | NuGet restore, push, search |
| `*.microsoft.com` | SDK/workload downloads, MS Build hosts, packages.microsoft.com |
| `*.dotnet.microsoft.com` | SDK download CDN (builds.dotnet.microsoft.com) |
| `dot.net`, `*.dot.net` | dotnet-install script + feeds |
| `aka.ms` | Short links the install script resolves download URLs through |
| `*.azureedge.net` | Legacy download CDNs |
