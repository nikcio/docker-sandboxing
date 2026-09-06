# OpenAPI Code Generator (C#)

- `openapi-codegen` is installed as a .NET global tool
  (Nikcio.OpenApiCodeGen) — it generates records, enums, and type aliases
  from OpenAPI 3.x specs (JSON or YAML)
- Install/update it with `dotnet tool install --global Nikcio.OpenApiCodeGen`
  (or `dotnet tool update --global Nikcio.OpenApiCodeGen`)
- Usage: `openapi-codegen <spec-file-or-url> -o <output.cs> -n <Namespace>`
  (`-h` for all options; it can fetch specs from URLs)
- Docs: https://openapi.nikcio.com — reachable from the sandbox
