## .NET local tools

- Pinned local tools (.config/dotnet-tools.json manifests in the
  workspace) are restored at sandbox start, before opencode runs
- Add tools with `dotnet tool install <name>` (writes the manifest) —
  new sandboxes restore them automatically
