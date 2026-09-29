<#
.SYNOPSIS
    sbx sandbox setup example (Windows PowerShell).

.DESCRIPTION
    A guided, copy-and-tweak script that:
      1. Installs the `sbx` CLI if it isn't already installed
      2. Signs you in with `sbx login`
      3. Registers your secrets:
           - zeldoc  (global  - the model provider key)
           - github  (sandbox - the GitHub token so the agent can push)
      4. Prints how to run the sandbox

    Run it with:

        powershell -ExecutionPolicy Bypass -File examples\setup-sandbox.ps1

    or from an existing PowerShell session:

        .\examples\setup-sandbox.ps1

.NOTES
    Requires Windows 11 (64-bit Intel/AMD, Windows Hypervisor Platform).
    See https://docs.docker.com/ai/sandboxes/install/ for requirements.
#>

# --------------------------------------------------------------------------
# CONFIGURATION - tweak these to fit your project
# --------------------------------------------------------------------------

# Sandbox name: must match the `name:` in your sbxenv.yaml. Secrets set with
# `-Sandbox <name>` are scoped to this sandbox only.
$SandboxName = if ($env:SANDBOX_NAME) { $env:SANDBOX_NAME } else { "my-project" }

# Example file to copy into your project (see examples/ in this repo).
# Set SandboxExample to match your stack: dotnet, go, node, python, rust.
$SandboxExample = if ($env:SANDBOX_EXAMPLE) { $env:SANDBOX_EXAMPLE } else { "go" }

# Install command for sbx. Override if you prefer another method
# (see https://docs.docker.com/ai/sandboxes/install/).
$SbxInstallCommand = "winget install -h Docker.sbx"

# --------------------------------------------------------------------------
# SECRET HELPERS - extend this section for your own secrets
#
#   Set-SecretValue <secret-name> <scope> <docs-url>
#     scope: "global" (available to every sandbox) or "sandbox" (scoped to
#     $SandboxName only).
#
#   Example - add a private npm token scoped to the sandbox:
#     Set-SecretValue npm_token sandbox "https://docs.npmjs.com/creating-and-viewing-access-tokens"
#   ...then reference it from sbxenv.yaml bindings.
# --------------------------------------------------------------------------

function Set-SecretValue {
    param(
        [string]$Name,
        [ValidateSet("global", "sandbox")]
        [string]$Scope,
        [string]$Docs
    )

    Write-Host ""
    Write-Host "--- Secret: $Name ($Scope) ----------------------------------"
    Write-Host "Docs: $Docs"

    $scopeArgs = @()
    if ($Scope -eq "sandbox") {
        $scopeArgs = @("-Sandbox", $SandboxName)
    }

    sbx secret get $Name @scopeArgs *> $null
    if ($LASTEXITCODE -eq 0) {
        $replace = Read-Host "A $Name secret already exists. Replace it? [y/N]"
        if ($replace -notmatch "^[Yy]$") {
            Write-Host "Keeping existing $Name secret."
            return
        }
    }
    Write-Host "You will be prompted for the value; it is stored in your OS keychain,"
    Write-Host "never in the sandbox."
    sbx secret set $Name @scopeArgs
}

# --------------------------------------------------------------------------
# Helpers
# --------------------------------------------------------------------------

function Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "=== $Message ==="
}

function Info {
    param([string]$Message)
    Write-Host "  $Message"
}

# --------------------------------------------------------------------------
# 1. Install sbx if missing
# --------------------------------------------------------------------------

Step "Step 1/4: sbx CLI"

if (Get-Command sbx -ErrorAction SilentlyContinue) {
    Info "sbx already installed."
} else {
    Info "sbx not found. Installing with: $SbxInstallCommand"
    Invoke-Expression $SbxInstallCommand
    # winget adds the install to PATH for future sessions; refresh for this one.
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" +
                [System.Environment]::GetEnvironmentVariable("Path", "User")
    if (-not (Get-Command sbx -ErrorAction SilentlyContinue)) {
        Write-Error "sbx still not found after install; open a new PowerShell window or install manually (https://docs.docker.com/ai/sandboxes/install/)."
        exit 1
    }
    Info "sbx installed."
}

# --------------------------------------------------------------------------
# 2. Sign in
# --------------------------------------------------------------------------

Step "Step 2/4: sign in"

sbx whoami *> $null
if ($LASTEXITCODE -eq 0) {
    Info "Already signed in."
} else {
    Info "Opening a browser for Docker OAuth..."
    sbx login
}

# --------------------------------------------------------------------------
# 3. Set up secrets
# --------------------------------------------------------------------------

Step "Step 3/4: secrets"

# zeldoc: GLOBAL - model provider key, available to all your sandboxes.
# Get/register/approve your key: https://docs.zeldoc.ai/connect-opencode
# Repo docs: docs/zeldoc-api-key.md
Set-SecretValue -Name zeldoc -Scope global -Docs "https://docs.zeldoc.ai/connect-opencode"

# github: SANDBOX-SCOPED - fine-grained PAT so the agent can push over HTTPS.
# Create one at GitHub: Settings -> Developer settings -> Personal access
# tokens -> Fine-grained tokens (repo guide: docs/github-pat.md).
# Skip this if you only clone public repos / use git over SSH.
$wantGithub = Read-Host "Set a GitHub token for sandbox '$SandboxName'? [Y/n]"
if ($wantGithub -notmatch "^[Nn]$") {
    Set-SecretValue -Name github -Scope sandbox -Docs "https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-fine-grained-personal-access-tokens"
} else {
    Info "Skipped github secret (pushing over HTTPS won't work without it)."
}

# >>> Add more secrets here, e.g.:
# Set-SecretValue -Name npm_token -Scope sandbox -Docs "https://docs.npmjs.com/creating-and-viewing-access-tokens"

# --------------------------------------------------------------------------
# 4. Host settings (one-time)
# --------------------------------------------------------------------------

Step "Step 4/4: host settings"

# sbx only fetches kits from allowed sources. The set call REPLACES the whole
# list, so merge with existing entries first.
Info "Allowing the kit sources for this repo's kits..."
$existing = sbx settings get kit.allowedSources 2>$null
if (-not $existing) { $existing = "[]" }
Info "Current kit.allowedSources: $existing"
# Escape the embedded quotes on Windows PowerShell 5.1 / PowerShell < 7.3:
# those versions strip double quotes when passing arguments to native
# executables, which corrupts JSON (`[\"..\"..]` keeps them intact).
# PowerShell 7.3+ passes arguments correctly and must NOT get the escapes.
if ($PSVersionTable.PSVersion -ge [version]"7.3") {
    $allowedSources = '["docker.io/","github.com/nikcio/"]'
} else {
    $allowedSources = '[\"docker.io/\",\"github.com/nikcio/\"]'
}
sbx settings set kit.allowedSources $allowedSourcesInfo "Done. If you had other entries in the list, re-add them (rerun this step after checking the output above)."

# Optional: let the sandboxed agent read images you paste
# sbx settings set clipboard.imagePaste true

# --------------------------------------------------------------------------
# Done - how to run
# --------------------------------------------------------------------------

Step "All set"

Write-Host @"

Next steps:

  1. Copy the example env file for your stack into your project root
     and rename it sbxenv.yaml (commit it so teammates get the same sandbox):

       Copy-Item examples\opencode-$SandboxExample.sbxenv.yaml C:\path\to\your\project\sbxenv.yaml

     In the copy, set 'name: $SandboxName' (or update SandboxName in
     this script) and drop the 'kits:' lines you don't need.

  2. Run the sandbox from your project root:

       sbx env run

     The sandbox starts and launches OpenCode automatically. Quitting
     OpenCode exits the sandbox; rerun the command to start it again.

  3. First run only: sbx asks you to approve credential bindings
     (which hosts may receive which secret). Approve the prompts.

Useful commands:

  sbx env rm      # remove the sandbox (and its scoped secrets)
  sbx secret set github -Sandbox $SandboxName   # rotate a secret

Full guide: docs/getting-started.md in this repo, or
https://docs.docker.com/ai/sandboxes/
"@
