#requires -Version 5.1
<#
.SYNOPSIS
    One-time host setup for the opencode-node-dotnet Docker Sandboxes kit.

.DESCRIPTION
    - Enables clipboard image paste for sandboxes (sbx settings).
    - Allows the GitHub kit source (kit.allowedSources) so kits/mixins are
      fetched from github.com/nikcio/docker-sandboxing.
    - Builds the template image and loads it into the sandbox runtime
      (or pushes it when -PushRegistry is given).
    - Registers the Zeldoc.ai API key as a proxy-managed service secret
      (the real key never enters the sandbox) and pre-creates the
      credential binding. Secrets already stored with `sbx secret` are
      skipped (an env var always (re)registers).
    - Registers the configurable `sbx-new` and `sbx-env` shell functions.
      `sbx-env` pre-flights the environment's GitHub PAT through sbx's own
      prompt (stored only in sbx's secret store) before `sbx env run`.
    - Validates the kit and mixins.

.PARAMETER TemplateTag
    Tag for the template image. Defaults to opencode-node-dotnet:v1
    (must match sandbox.image in kit/spec.yaml).

.PARAMETER PushRegistry
    Optional registry prefix, e.g. docker.io/myorg. When set, the template is
    pushed there instead of loaded locally. Remember to update sandbox.image
    in kit/spec.yaml to "<PushRegistry>/opencode-node-dotnet:v1".

.PARAMETER SkipBuild
    Skip the template build (e.g. template already loaded).

.PARAMETER SkipAlias
    Skip registering the sbx-new shell function.

.EXAMPLE
    $env:ZELDOC_API_KEY = "zd-..."
    ./scripts/bootstrap.ps1
#>
[CmdletBinding()]
param(
    [string]$TemplateTag = "opencode-node-dotnet:v1",
    [string]$PushRegistry = "",
    [switch]$SkipBuild,
    [switch]$SkipAlias
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot

function Invoke-Step {
    param([string]$Message, [scriptblock]$Action)
    Write-Host "==> $Message" -ForegroundColor Cyan
    & $Action
}

# Returns $true when sbx already holds a secret for the service (matched on
# the TYPE/NAME columns of `sbx secret ls`).
function Test-SecretStored {
    param([string]$Service)
    $list = @()
    try { $list = @(sbx secret ls 2>$null) } catch { $list = @() }
    return [bool]($list | Select-String -Pattern "(^|\s)service\s+$Service(\s|$)" -Quiet)
}

Invoke-Step "sbx settings: allow clipboard image paste" {
    sbx settings set clipboard.imagePaste true
}

# Kits are fetched from GitHub, so the source must be in the allowlist.
# The setting replaces the whole list — merge, don't overwrite.
$kitSource = "github.com/nikcio/"
Invoke-Step "sbx settings: allow kit source $kitSource" {
    $current = sbx settings get kit.allowedSources
    $entries = @()
    try { $entries = @($current | ConvertFrom-Json) } catch { $entries = @() }
    if ($entries -notcontains $kitSource) {
        if ($entries -notcontains "docker.io/") { $entries = @("docker.io/") + $entries }
        $entries = @($entries | Where-Object { $_ }) + $kitSource
        $json = '["' + ($entries -join '","') + '"]'
        sbx settings set kit.allowedSources $json
    }
}

# Register the configurable `sbx-new` and `sbx-env` launchers as shell
# functions. Skip with -SkipAlias. Idempotent (marker comments).
if (-not $SkipAlias) {
    $marker = "sbx-new (docker-sandboxing)"
    $launcher = Join-Path $repoRoot "scripts\new-sandbox.ps1"
    if (-not (Test-Path $PROFILE)) {
        New-Item -ItemType File -Force -Path $PROFILE | Out-Null
    }
    if (-not (Select-String -Path $PROFILE -SimpleMatch $marker -Quiet)) {
        Invoke-Step "Registering sbx-new function in PowerShell profile" {
            Add-Content -Encoding utf8 $PROFILE @"

# $marker — configurable sandbox launcher (see scripts/new-sandbox.ps1)
function sbx-new { & "$launcher" @args }
"@
        }
    }
    $marker = "sbx-env (docker-sandboxing)"
    $launcher = Join-Path $repoRoot "scripts\sbx-env.ps1"
    if (-not (Select-String -Path $PROFILE -SimpleMatch $marker -Quiet)) {
        Invoke-Step "Registering sbx-env function in PowerShell profile" {
            Add-Content -Encoding utf8 $PROFILE @"

# $marker — GitHub-PAT-aware `sbx env run` launcher (see scripts/sbx-env.ps1)
function sbx-env { & "$launcher" @args }
"@
        }
    }
}

if (-not $SkipBuild) {
    Invoke-Step "Building template image $TemplateTag" {
        docker build -t $TemplateTag (Join-Path $repoRoot "template")
    }

    if ($PushRegistry) {
        $remote = "$PushRegistry/$TemplateTag"
        Invoke-Step "Pushing template to $remote" {
            docker tag $TemplateTag $remote
            docker push $remote
        }
        Write-Host "    Update sandbox.image in kit/spec.yaml to: $remote"
    }
    else {
        New-Item -ItemType Directory -Force -Path (Join-Path $repoRoot "dist") | Out-Null
        $tarPath = Join-Path $repoRoot ("dist\" + ($TemplateTag -replace '[:/]', '-') + ".tar")
        Invoke-Step "Saving image to $tarPath" {
            docker image save $TemplateTag -o $tarPath
        }
        Invoke-Step "Loading template into the sandbox runtime" {
            sbx template load $tarPath
        }
    }
}

Invoke-Step "Registering Zeldoc API key (proxy-managed; never enters the sandbox)" {
    $key = $env:ZELDOC_API_KEY
    if (-not $key -and (Test-SecretStored zeldoc)) {
        Write-Host "    Already registered - skipping (set `$env:ZELDOC_API_KEY to update it)."
        return
    }
    if (-not $key) {
        $secure = Read-Host "ZELDOC_API_KEY" -AsSecureString
        $key = [Runtime.InteropServices.Marshal]::PtrToStringAuto(
            [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure))
    }
    if (-not $key) { throw "No Zeldoc API key provided (set `$env:ZELDOC_API_KEY or re-run)." }
    $key | sbx secret set zeldoc
    Clear-Variable key
}

# GitHub tokens are provisioned PER ENVIRONMENT (a `secrets.github` entry in
# the project's .sbxenv.yaml, stored at the sandbox scope) — never globally
# here. See README, "GitHub CLI + a scoped personal access token".

# Third-party v2 kits need a credential binding approval. The first
# interactive `sbx run` prompts for it; pre-create it for unattended use.
$bindingsPath = Join-Path $env:APPDATA "sbx\credentials.yaml"
if (-not (Test-Path $bindingsPath)) {
    Invoke-Step "Pre-creating credential bindings for zeldoc ($bindingsPath)" {
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $bindingsPath) | Out-Null
        @'
bindings:
  zeldoc:
    apiKey:
      domains:
        - api.zeldoc.ai
'@ | Set-Content -Encoding utf8 $bindingsPath
    }
}
else {
    if (-not (Select-String -Path $bindingsPath -Pattern "zeldoc" -Quiet)) {
        Write-Host "    Hint: add a 'zeldoc' apiKey binding (domains: api.zeldoc.ai) to $bindingsPath,"
        Write-Host "    or approve it interactively on the first 'sbx run'."
    }
}

Invoke-Step "Validating kit and mixins" {
    Get-ChildItem -Directory (Join-Path $repoRoot "mixins") | ForEach-Object {
        sbx kit validate $_.FullName
    }
    sbx kit validate (Join-Path $repoRoot "kit")
}

Write-Host ""
Write-Host "Done. Open a NEW shell so 'sbx-new' and 'sbx-env' are loaded, then launch with:" -ForegroundColor Green
Write-Host "  sbx-new                                   # guided wizard"
Write-Host "  sbx-new <path-to-project>                 # scripted: full stack from GitHub"
Write-Host "  sbx-new -Profile node <path-to-project>   # node-only mixin set"
Write-Host "  sbx-new -ListProfiles                     # all profiles"
Write-Host ""
Write-Host "Tip: kit changes only apply to NEW sandboxes. Recreate with:"
Write-Host "  sbx rm <sandbox-name> && sbx-new <path-to-project>"
Write-Host ""
Write-Host "GitHub PAT for a sandbox environment: launch with 'sbx-env' - it prompts"
Write-Host "via sbx at first run (stored only in sbx's secret store). Rotate with:"
Write-Host "  sbx secret set github --sandbox <environment-name>"
