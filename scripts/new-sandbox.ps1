<#
.SYNOPSIS
    Wizard-style launcher for creating/attaching an opencode-node-dotnet
    Docker Sandboxes sandbox.

.DESCRIPTION
    Run with no arguments for a guided wizard: it asks for the workspace,
    mixin profile, kit source, sandbox name, and launch mode, shows a
    summary, and creates/attaches the sandbox.

    Any argument/flag skips the wizard (scripted mode) — unset values fall
    back to environment variables, then defaults:

        SBX_SANDBOX_PROFILE   full (default) | node | dotnet | node-docker | browser | none
        SBX_SANDBOX_MIXINS    comma-separated mixin override, e.g. zeldoc,git,node
        SBX_SANDBOX_SOURCE    git (default; fetch from GitHub) | local (clone)
        SBX_SANDBOX_REPO      GitHub repo (default: nikcio/docker-sandboxing)
        SBX_SANDBOX_REF       pin git kits to a branch/tag/commit (optional)
        SBX_SANDBOX_REPO_DIR  local repo root for -Source local (default: repo root)

    If a sandbox with the resolved name already exists, it re-attaches
    without kit flags (kits only apply at creation).

.EXAMPLE
    .\scripts\new-sandbox.ps1                          # guided wizard

.EXAMPLE
    .\scripts\new-sandbox.ps1 D:\code\my-project       # scripted: everything else defaults

.EXAMPLE
    .\scripts\new-sandbox.ps1 -Profile node D:\code\web

.EXAMPLE
    .\scripts\new-sandbox.ps1 -Mixins zeldoc,git,node -Source local .
#>
[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$Workspace,
    [string]$Name,
    [string]$Profile = $env:SBX_SANDBOX_PROFILE,
    [string[]]$Mixins,
    [string]$Source = $env:SBX_SANDBOX_SOURCE,
    [string]$Repo = $env:SBX_SANDBOX_REPO,
    [string]$Ref = $env:SBX_SANDBOX_REF,
    [string]$RepoDir = $env:SBX_SANDBOX_REPO_DIR,
    [switch]$Detach,
    [switch]$Yes,
    [switch]$ListProfiles
)

$ErrorActionPreference = "Stop"

$allMixins = [ordered]@{
    "base"                    = "entrypoint runtime (hooks, opencode autostart, login shell) + agent guidance + MCP gateway (required)"
    "global-opencode-config"  = "permissive OpenCode config + provider config merge (required)"
    "env-guard"               = "workspace .env guard: refuses/removes .env files (required)"
    "banner"                  = "startup banner (cosmetic)"
    "opencode-runtime"        = "OpenCode runtime egress: updates, models.dev, Zen, plugins"
    "zeldoc"                  = "Zeldoc.ai (zdev) model provider: proxy-managed key + network"
    "copilot"                 = "GitHub Copilot model provider: device-flow sign-in + network"
    "git"                     = "git hosting (GitHub/GitLab) + gh auth + worktree workflow"
    "node"                    = "Node.js/NVM/PNPM: nodejs.org + npm registry"
    "dotnet"                  = ".NET/NuGet + Microsoft hosts, telemetry off"
    "docker"                  = "container registries for the in-sandbox Docker engine"
    "apt"                     = "Ubuntu/Microsoft package mirrors for apt (+ background cache warm)"
    "browser"                 = "Google Chrome browser software (no network rules)"
    "playwright"              = "Playwright + Chromium headless shell (lightest)"
    "playwright-chromium"     = "Playwright + full Chromium"
    "playwright-all"          = "Playwright + Chromium, Firefox, WebKit"
    "sbx"                     = "sbx CLI inside the sandbox: kit authoring (validate/inspect/pack)"
}

$profiles = [ordered]@{
    full          = @("base", "global-opencode-config", "env-guard", "banner", "opencode-runtime", "zeldoc", "git", "node", "dotnet", "docker", "apt", "browser", "playwright")
    node          = @("base", "global-opencode-config", "env-guard", "banner", "opencode-runtime", "zeldoc", "git", "node")
    dotnet        = @("base", "global-opencode-config", "env-guard", "banner", "opencode-runtime", "zeldoc", "git", "dotnet")
    "node-docker" = @("base", "global-opencode-config", "env-guard", "banner", "opencode-runtime", "zeldoc", "git", "node", "docker")
    browser       = @("base", "global-opencode-config", "env-guard", "banner", "opencode-runtime", "zeldoc", "git", "node", "apt", "browser", "playwright")
    none          = @()
}

if ($ListProfiles) {
    $profiles.GetEnumerator() | ForEach-Object {
        "{0,-12} {1}" -f $_.Key, ($_.Value -join ", ")
    }
    return
}

function Read-Default([string]$Prompt, [string]$Default) {
    $suffix = if ($Default) { " [$Default]" } else { "" }
    $value = Read-Host "$Prompt$suffix"
    if ([string]::IsNullOrWhiteSpace($value)) { $Default } else { $value.Trim() }
}

function Read-Confirm([string]$Prompt, [bool]$Default) {
    $hint = if ($Default) { "[Y/n]" } else { "[y/N]" }
    while ($true) {
        $value = Read-Host "$Prompt $hint"
        if ([string]::IsNullOrWhiteSpace($value)) { return $Default }
        switch ($value.Trim().ToLower()) {
            "y" { return $true } "yes" { return $true }
            "n" { return $false } "no" { return $false }
            default { Write-Host "    Please answer y or n." }
        }
    }
}

function Read-Option([string]$Prompt, [System.Collections.IDictionary]$Options, [string]$DefaultKey) {
    Write-Host $Prompt
    $keys = @()
    $i = 0
    foreach ($k in $Options.Keys) {
        $i++
        $keys += $k
        $desc = $Options[$k]
        if ($desc) { Write-Host ("  {0}) {1,-14} {2}" -f $i, $k, $desc) }
        else { Write-Host ("  {0}) {1}" -f $i, $k) }
    }
    while ($true) {
        $value = Read-Host "Choose$(if ($DefaultKey) { " [$DefaultKey]" })"
        if ([string]::IsNullOrWhiteSpace($value)) { $value = $DefaultKey }
        if ([string]::IsNullOrWhiteSpace($value)) { Write-Host "    Invalid choice."; continue }
        $value = $value.Trim()
        if ($value -match '^\d+$' -and [int]$value -ge 1 -and [int]$value -le $keys.Count) {
            return $keys[[int]$value - 1]
        }
        if ($keys -contains $value) { return $value }
        Write-Host "    Invalid choice."
    }
}

# Normalize fallbacks (env vars already applied via param defaults).
if (-not $Repo) { $Repo = "nikcio/docker-sandboxing" }
if (-not $Source) { $Source = "git" }

$scriptRepoRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
$wizard = -not $Yes -and ($PSBoundParameters.Count -eq 0) -and -not [Console]::IsInputRedirected

if ($wizard) {
    Write-Host "==> Create a new opencode-node-dotnet sandbox"
    Write-Host ""

    # 1. Workspace
    if (-not $Workspace) {
        while ($true) {
            $Workspace = Read-Default "Workspace directory" (Get-Location).Path
            if (Test-Path $Workspace) { $Workspace = (Resolve-Path $Workspace).Path; break }
            if (Read-Confirm "Workspace '$Workspace' does not exist. Create it?" $true) {
                New-Item -ItemType Directory -Force -Path $Workspace | Out-Null
                $Workspace = (Resolve-Path $Workspace).Path
                break
            }
        }
    }
    else {
        $Workspace = (Resolve-Path $Workspace).Path
    }

    # 2. Mixin profile
    if (-not $Mixins) {
        $options = [ordered]@{}
        foreach ($k in $profiles.Keys) { $options[$k] = ($profiles[$k] -join ", ") }
        $options["custom"] = "pick mixins yourself"
        $defaultKey = if ($Profile -and $profiles.Contains($Profile)) { $Profile } else { "full" }
        $choice = Read-Option "Mixin profile" $options $defaultKey
        if ($choice -eq "custom") {
            while ($true) {
                Write-Host "    Available mixins:"
                foreach ($k in $allMixins.Keys) { Write-Host ("      {0,-18} {1}" -f $k, $allMixins[$k]) }
                $raw = Read-Default "Mixins (comma-separated, 'all', or 'none')" "all"
                if ($raw -eq "all") { $Mixins = $profiles["full"]; break }
                if ($raw -eq "none") { $Mixins = @(); break }
                $tokens = @($raw -split "," | ForEach-Object { $_.Trim() } | Where-Object { $_ })
                $bad = @($tokens | Where-Object { -not $allMixins.Contains($_) })
                if ($bad.Count -eq 0) { $Mixins = $tokens; break }
                Write-Host "    Unknown mixin(s): $($bad -join ', ')"
            }
        }
        else {
            $Mixins = $profiles[$choice]
        }
        $Profile = $choice
    }
    else {
        if (-not $profiles.Contains($Profile)) { $Profile = "custom" }
    }

    # 3. Kit source
    $Source = Read-Option "Kit source" ([ordered]@{
        git   = "fetch kits from github.com/$Repo (recommended)"
        local = "use the local clone of this repository"
    }) $Source
    if ($Source -eq "git") {
        $Ref = Read-Default "Pin kits to a branch/tag/commit (blank = default branch)" $Ref
    }
    else {
        if (-not $RepoDir) { $RepoDir = $scriptRepoRoot }
        $RepoDir = (Resolve-Path (Read-Default "Local repository directory" $RepoDir)).Path
    }

    # 4. Sandbox name (with attach handling for existing names)
    $defaultName = "opencode-node-dotnet-$(Split-Path -Leaf $Workspace)"
    $Name = Read-Default "Sandbox name" $(if ($Name) { $Name } else { $defaultName })
    $existing = @(sbx ls -q 2>$null)
    while ($existing -contains $Name) {
        $choice = Read-Option "Sandbox '$Name' already exists" ([ordered]@{
            attach = "re-attach to it (kits are ignored)"
            rename = "pick a different name"
            cancel = "exit without doing anything"
        }) "attach"
        if ($choice -eq "attach") {
            sbx run --name $Name
            return
        }
        if ($choice -eq "cancel") { return }
        $Name = Read-Default "Sandbox name" $defaultName
    }

    # 5. Launch mode
    $mode = Read-Option "Launch mode" ([ordered]@{
        attach = "create and attach (interactive)"
        detach = "create only (no attach)"
    }) "attach"
    $Detach = ($mode -eq "detach")

    # 6. Summary + confirm
    $kitLabel = if ($Source -eq "git") {
        "git: github.com/$Repo$(if ($Ref) { " @ $Ref" })"
    } else {
        "local: $RepoDir"
    }
    Write-Host ""
    Write-Host "==> Sandbox configuration"
    Write-Host "    workspace   $Workspace"
    Write-Host "    name        $Name"
    Write-Host "    source      $kitLabel"
    Write-Host "    mixins      $(if ($Mixins) { $Mixins -join ', ' } else { '<none>' })"
    Write-Host "    launch      $(if ($Detach) { 'create only' } else { 'create and attach' })"
    if (-not (Read-Confirm "Proceed" $true)) { return }
    Write-Host ""
}
else {
    # Scripted mode: fill unset values from defaults.
    if (-not $Profile) { $Profile = "full" }
    if (-not $Workspace) { $Workspace = "." }
    $Workspace = (Resolve-Path $Workspace).Path
    if (-not $Mixins -and $env:SBX_SANDBOX_MIXINS) {
        $Mixins = @($env:SBX_SANDBOX_MIXINS -split "," | ForEach-Object { $_.Trim() } | Where-Object { $_ })
    }
    if (-not $Mixins) {
        if (-not $profiles.Contains($Profile)) {
                throw "Unknown profile '$Profile'. Known: $(($profiles.Keys -join ', ')) (or pass -Mixins)."
        }
        $Mixins = $profiles[$Profile]
    }
    if (-not $Name) {
        $Name = "opencode-node-dotnet-$(Split-Path -Leaf $Workspace)"
    }
    # Re-attach when the sandbox already exists (kits only apply at creation).
    $existing = @(sbx ls -q 2>$null)
    if ($existing -contains $Name) {
        Write-Host "==> Attaching to existing sandbox '$Name'"
        sbx run --name $Name
        return
    }
}

# Build the kit references.
$kits = @()
if ($Source -eq "git") {
    $gitBase = "git+https://github.com/$Repo.git"
    $kits += if ($Ref) { "$gitBase#ref=$Ref&dir=kit-published-node-dotnet" } else { "$gitBase#dir=kit-published-node-dotnet" }
    foreach ($m in $Mixins) {
        $kits += if ($Ref) { "$gitBase#ref=$Ref&dir=mixins/$m" } else { "$gitBase#dir=mixins/$m" }
    }
}
else {
    $RepoDir = (Resolve-Path $RepoDir).Path
    $kits += Join-Path $RepoDir "kit-node-dotnet"
    foreach ($m in $Mixins) { $kits += Join-Path $RepoDir "mixins/$m" }
}

$kitArgs = $kits | ForEach-Object { "--kit"; $_ }

if (-not $wizard) {
    Write-Host "==> Creating sandbox '$Name' (profile: $Profile, source: $Source)"
}
Write-Host "    kits: $($kits -join '  ')"

if ($Detach) {
    sbx create -q --name $Name @kitArgs opencode-node-dotnet $Workspace
}
else {
    sbx run --name $Name @kitArgs opencode-node-dotnet $Workspace
}
