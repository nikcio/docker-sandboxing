#requires -Version 5.1
<#
.SYNOPSIS
    Launcher for `sbx env run` that provisions the environment's GitHub PAT
    first, using sbx's own prompt.

.DESCRIPTION
    sbx resolves `secrets.*.command` entries non-interactively (a "host
    shell command whose standard output becomes the secret", per the
    environment-files docs), so an interactive prompt inside a secret
    command can never work: sbx captures stdout as the secret and only
    surfaces stderr on failures. The token therefore lives in sbx's secret
    store (the OS keychain), scoped per sandbox, and this launcher makes
    sure it exists BEFORE `sbx env run` provisions secrets:

    1. Reads `name:` from the environment file (first non-flag argument,
       else .\.sbxenv.yaml / .\.sbxenv.yml).
    2. If `sbx secret ls` has no github entry scoped to that sandbox, runs
       `sbx secret set github --sandbox <name>` - sbx prompts ("Enter
       secret:") and stores the value in the keychain. Fine-grained PATs
       only - never the broad-scope host `gh auth token`. Skipping (empty
       input) is fine: public repos and git over SSH keep working, and the
       in-sandbox [git-auth] banner will point at this same command.
    3. Runs `sbx env run` with all arguments.

    Rotation: re-run `sbx secret set github --sandbox <name>`, or remove
    and recreate the environment.

.EXAMPLE
    sbx-env                          # environment in the current directory

.EXAMPLE
    sbx-env D:\envs\my-project       # explicit environment directory/file
#>

$ErrorActionPreference = "Stop"

# Locate the environment file: first non-flag argument (file or directory),
# else .\.sbxenv.yaml / .\.sbxenv.yml.
$envFile = $null
foreach ($arg in $args) {
    if ($arg -like '-*') { continue }
    if (Test-Path $arg -PathType Leaf) { $envFile = $arg; break }
    foreach ($fileName in @(".sbxenv.yaml", ".sbxenv.yml")) {
        $candidate = Join-Path $arg $fileName
        if (Test-Path $candidate -PathType Leaf) { $envFile = $candidate; break }
    }
    if ($envFile) { break }
}
if (-not $envFile) {
    foreach ($candidate in @(".sbxenv.yaml", ".sbxenv.yml")) {
        if (Test-Path $candidate -PathType Leaf) { $envFile = $candidate; break }
    }
}

# Derive the sandbox name from the file's `name:` (the scope sbx env uses).
$sandboxName = $null
if ($envFile) {
    $line = Select-String -Path $envFile -Pattern '^name:' | Select-Object -First 1
    if ($line) {
        $sandboxName = (($line.Line -split ':', 2)[1].Trim().Trim('"', "'").Trim())
        if (-not $sandboxName) { $sandboxName = $null }
    }
}

if ($sandboxName) {
    # Provision the per-sandbox GitHub token through sbx's own prompt when
    # it is not stored yet. sbx captures stdout as the secret, so this must
    # happen outside secret resolution - hence this launcher.
    $stored = [bool](@((sbx secret ls 2>$null)) | Select-String -Pattern "(^|\s)$([regex]::Escape($sandboxName))\s+service\s+github(\s|$)" -Quiet)
    if (-not $stored) {
        Write-Host "No GitHub token stored for sandbox '$sandboxName' yet." -ForegroundColor Cyan
        Write-Host "sbx will prompt for a fine-grained PAT (stored only in sbx's secret store)." -ForegroundColor Cyan
        Write-Host "Leave it empty to skip - public repos and git over SSH keep working.`n"
        & sbx secret set github --sandbox $sandboxName
        if ($LASTEXITCODE -ne 0) {
            Write-Host "No token stored - continuing. Public repos and git over SSH still work;" -ForegroundColor Yellow
            Write-Host "store one later with: sbx secret set github --sandbox $sandboxName" -ForegroundColor Yellow
        }
    }
}
else {
    Write-Host "Could not read a 'name:' from the environment file - skipping the GitHub token check." -ForegroundColor Yellow
}

& sbx env run @args
exit $LASTEXITCODE
