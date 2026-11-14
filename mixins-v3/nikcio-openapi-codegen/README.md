# nikcio-openapi-codegen (v3)

The [openapi-code-generator](https://openapi.nikcio.com) .NET global tool
`Nikcio.OpenApiCodeGen` (the `openapi-codegen` CLI, for generating C#
models from OpenAPI 3.x documents), on the .NET SDK the mixin also
carries, baked into the mixin layer at image build.

## Build / use

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin
`&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/nikcio-openapi-codegen   # openapi-codegen (C#)
```

For local development, point `--kit` at the directory instead —
`sbx run <agent> . --kit ./mixins-v3/nikcio-openapi-codegen` — or build
the image yourself:

```bash
docker buildx build -f mixins-v3/nikcio-openapi-codegen/nikcio-openapi-codegen.yaml -t my/codegen-mixin:dev mixins-v3/nikcio-openapi-codegen/
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `codegenVersion` | *(empty = latest)* | Nikcio.OpenApiCodeGen version |
| `dotnetVersion` | *(empty = latest LTS)* | .NET SDK version |

## How it works

- The mixin carries its own .NET SDK, installed the same way the `dotnet`
  mixin does: official install script into `/opt/dotnet`, `dotnet`
  symlinked onto `/usr/local/bin`, and the same telemetry/NuGet image
  environment (`DOTNET_CLI_TELEMETRY_OPTOUT`, `DOTNET_NOLOGO`,
  `DOTNET_GENERATE_ASPNET_CERTIFICATE=false`, `NUGET_XMLDOC_MODE=skip`).
- `dotnet tool install --global Nikcio.OpenApiCodeGen` (pinned with
  `codegenVersion` if set) installs the tool; the global-tools directory
  ships in the layer and is added to `PATH` by a `/etc/profile.d/`
  script.
- The versions resolve "latest" at build, so the image is a snapshot:
  rebuild (or pin via the build args) to update it.

## Runtime egress

`openapi.nikcio.com` (the generator's docs site), `nuget.org` + wildcard
(`dotnet tool install`/`update` upgrades). No install-phase grant.
