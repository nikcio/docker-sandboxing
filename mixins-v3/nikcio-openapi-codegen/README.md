# nikcio-openapi-codegen (v3)

The openapi-code-generator .NET global tool `Nikcio.OpenApiCodeGen` (on the
.NET SDK it also carries) baked into the mixin layer at image build. v3
port of [`../nikcio-openapi-codegen`](../nikcio-openapi-codegen) — read
that for the full behavior story; see [`../README.md`](../README.md) for
what the v3 format changes.

## Build / use

```bash
docker buildx build -f mixins-v3/nikcio-openapi-codegen/nikcio-openapi-codegen.yaml -t my/codegen-mixin:dev mixins-v3/nikcio-openapi-codegen/
sbx run <agent> . --kit ./mixins-v3/nikcio-openapi-codegen   # or the published image reference
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `codegenVersion` | *(empty = latest)* | Nikcio.OpenApiCodeGen version |
| `dotnetVersion` | *(empty = latest)* | .NET SDK version |

## Runtime egress

`openapi.nikcio.com` (docs), `nuget.org` + wildcard (`dotnet tool
install`/`update`). No install-phase grant.
