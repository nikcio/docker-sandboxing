#requires -Version 5.1
<#
.SYNOPSIS
    Launcher for `sbx env run` that provisions the sandbox's GitHub PAT
    first, using sbx's own prompt.

.DESCRIPTION
    sbx can't prompt while resolving a `secrets.*.command` entry (stdout is
    captured as the secret), so this launcher checks `sbx secret ls` for a
    github entry scoped to the environment file's `name:` and, when missing,
    runs `sbx secret set github --sandbox <name>` before `sbx env run`.
    Empty input skips the token — public repos and git over SSH keep
    working. Rotation is the same command.

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
