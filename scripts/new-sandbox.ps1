<#
.SYNOPSIS
    Configurable launcher (alias target) for creating/attaching an
    opencode-node-dotnet Docker Sandboxes sandbox.

.DESCRIPTION
    Composes the sandbox kit with the mixins for a chosen profile and runs
    `sbx run`. If a sandbox with the resolved name already exists, it
    re-attaches without kit flags (kits only apply at creation).

    Configurable via parameters or environment variables (flags win):

        SBX_SANDBOX_PROFILE   full (default) | node | dotnet | node-docker | none
        SBX_SANDBOX_MIXINS    comma-separated mixin override, e.g. zeldoc,git,node
        SBX_SANDBOX_SOURCE    git (default; fetch from GitHub) | local (clone)
        SBX_SANDBOX_REPO      GitHub repo (default: nikcio/docker-sandboxing)
        SBX_SANDBOX_REF       pin git kits to a branch/tag/commit (optional)
        SBX_SANDBOX_REPO_DIR  local repo root for -Source local (default: repo root)

.PARAMETER Workspace
    Project directory to mount as the workspace. Default: current directory.

.PARAMETER Name
    Sandbox name. Default: opencode-node-dotnet-<workspace basename>.

.PARAMETER Detach
    Create without attaching (sbx create -q instead of sbx run).

.EXAMPLE
    sbx-new D:\code\my-project
    sbx-new -Profile node D:\code\web
    sbx-new -Mixins zeldoc,git,node -Source local .
#>
[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$Workspace = ".",
    [string]$Name,
    [string]$Profile = $(if ($env:SBX_SANDBOX_PROFILE) { $env:SBX_SANDBOX_PROFILE } else { "full" }),
    [string[]]$Mixins,
    [ValidateSet("git", "local")]
    [string]$Source = $(if ($env:SBX_SANDBOX_SOURCE) { $env:SBX_SANDBOX_SOURCE } else { "git" }),
    [string]$Repo = $(if ($env:SBX_SANDBOX_REPO) { $env:SBX_SANDBOX_REPO } else { "nikcio/docker-sandboxing" }),
    [string]$Ref = $env:SBX_SANDBOX_REF,
    [string]$RepoDir = $env:SBX_SANDBOX_REPO_DIR,
    [switch]$Detach,
    [switch]$ListProfiles
)

$ErrorActionPreference = "Stop"

$profiles = [ordered]@{
    full          = @("opencode-runtime", "zeldoc", "git", "node", "dotnet", "docker", "apt")
    node          = @("opencode-runtime", "zeldoc", "git", "node")
    dotnet        = @("opencode-runtime", "zeldoc", "git", "dotnet")
    "node-docker" = @("opencode-runtime", "zeldoc", "git", "node", "docker")
    none          = @()
}

if ($ListProfiles) {
    $profiles.GetEnumerator() | ForEach-Object {
        "{0,-12} {1}" -f $_.Key, ($_.Value -join ", ")
    }
    return
}

if (-not $profiles.Contains($Profile)) {
    throw "Unknown profile '$Profile'. Known: $(($profiles.Keys -join ', ')) (or pass -Mixins)."
}

if (-not $Mixins -and $env:SBX_SANDBOX_MIXINS) {
    $Mixins = $env:SBX_SANDBOX_MIXINS -split "," | ForEach-Object { $_.Trim() } | Where-Object { $_ }
}
if (-not $Mixins) { $Mixins = $profiles[$Profile] }

# Resolve sandbox name (sbx default naming: <agent>-<workspace basename>).
if (-not $Name) {
    $base = Split-Path -Leaf ((Resolve-Path $Workspace).Path)
    $Name = "opencode-node-dotnet-$base"
}

# Re-attach when the sandbox already exists (kits only apply at creation).
$existing = @(sbx ls -q 2>$null)
if ($existing -contains $Name) {
    Write-Host "==> Attaching to existing sandbox '$Name'"
    sbx run --name $Name
    return
}

# Build the kit references.
$kits = @()
if ($Source -eq "git") {
    $gitBase = "git+https://github.com/$Repo.git"
    $kits += if ($Ref) { "$gitBase#ref=$Ref&dir=kit" } else { "$gitBase#dir=kit" }
    foreach ($m in $Mixins) {
        $kits += if ($Ref) { "$gitBase#ref=$Ref&dir=mixins/$m" } else { "$gitBase#dir=mixins/$m" }
    }
}
else {
    if (-not $RepoDir) { $RepoDir = Split-Path -Parent (Split-Path -Parent $PSCommandPath) }
    $RepoDir = (Resolve-Path $RepoDir).Path
    $kits += Join-Path $RepoDir "kit"
    foreach ($m in $Mixins) { $kits += Join-Path $RepoDir "mixins/$m" }
}

$kitArgs = $kits | ForEach-Object { "--kit"; $_ }

Write-Host "==> Creating sandbox '$Name' (profile: $Profile, source: $Source)"
Write-Host "    kits: $($kits -join '  ')"

if ($Detach) {
    sbx create -q --name $Name @kitArgs opencode-node-dotnet $Workspace
}
else {
    sbx run --name $Name @kitArgs opencode-node-dotnet $Workspace
}
