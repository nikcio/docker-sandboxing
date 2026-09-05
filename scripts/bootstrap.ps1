#requires -Version 5.1
<#
.SYNOPSIS
    Local development setup for the template, kit, and mixins.

.DESCRIPTION
    - Builds the template image and loads it into the sandbox runtime
      (or pushes it when -PushRegistry is given).
    - Registers the Zeldoc.ai API key (proxy-managed — the sandbox never
      sees it) and pre-creates the credential binding. Already-stored
      secrets are skipped; an env var always (re)registers.
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

.EXAMPLE
    $env:ZELDOC_API_KEY = "zd-..."
    ./scripts/bootstrap.ps1
#>
[CmdletBinding()]
param(
    [string]$TemplateTag = "opencode-node-dotnet:v1",
    [string]$PushRegistry = "",
    [switch]$SkipBuild
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

# GitHub tokens are provisioned PER SANDBOX with sbx's own prompt
# (`sbx secret set github --sandbox <name>`) — never globally here.
# See docs/github-pat.md.

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
Write-Host "Done. Develop the kits/mixins in a sandbox:" -ForegroundColor Green
Write-Host "  sbx env run                    # from this repo root (uses .\.sbxenv.yaml, local kits)"
Write-Host "  .\scripts\new-sandbox.ps1      # wizard launcher for any workspace"
Write-Host ""
Write-Host "Tip: kit changes only apply to NEW sandboxes. Recreate with:"
Write-Host "  sbx rm <sandbox-name> && sbx env run"
Write-Host ""
Write-Host "GitHub PAT for a sandbox (lets the agent push and open PRs): store it with"
Write-Host "sbx's own prompt, scoped to the sandbox (see docs/github-pat.md):"
Write-Host "  sbx secret set github --sandbox <sandbox-name>"
