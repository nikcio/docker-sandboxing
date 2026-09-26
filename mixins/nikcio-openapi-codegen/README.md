# nikcio-openapi-codegen

Installs the [openapi-code-generator](https://openapi.nikcio.com) .NET global tool (`Nikcio.OpenApiCodeGen` — the `openapi-codegen` CLI) for generating C# models from OpenAPI 3.x documents, plus the docs-site egress.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/nikcio-openapi-codegen&ref=vX.Y.Z   # openapi-codegen (C#)
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/nikcio-openapi-codegen <workload> .`

Needs the `dotnet` mixin — the install ensures the SDK first when it is missing, then installs the tool as the `agent` user. It runs at **sandbox creation only** and needs egress for the .NET SDK + tool download (covered by the `dotnet` mixin's domains) and the Ubuntu apt mirrors (`archive.ubuntu.com`/`security.ubuntu.com`) for its `apt-get` calls.

## How it works

- **SDK**: when `dotnet` is missing, installs it the same way the `dotnet` mixin does (apt feed, falling back to the dot.net install script).
- **Tool**: `dotnet tool install --global Nikcio.OpenApiCodeGen` as the `agent` user; `~/.dotnet/tools` is added to `PATH` via `/etc/sandbox-persistent.sh`. Skipped when `openapi-codegen` is already on `PATH`.
- The agent note (shipped via the mixin's `agent-context` capability) documents the CLI for the agent.

## Network domains

| Domain | Why |
| ------ | --- |
| `openapi.nikcio.com:443` | The generator's docs site |
