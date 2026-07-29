<#
.SYNOPSIS
  Renames the project folder from "XX. Arshilla" to "XX. SandiBumi" and fixes everything
  the rename would otherwise break.

.DESCRIPTION
  MUST BE RUN WITH CLAUDE CODE CLOSED (and Cursor/VS Code/SandiBumi closed too).
  A process's current directory is an open handle on that directory, and Claude Code's
  working directory IS the project folder - so the rename cannot be done from inside a
  session, by Claude or by anything else. Windows refuses it with "used by another process".

  Everything else is already done: every path reference inside the repo was updated to
  D:\XX. SandiBumi in the same commit that added this script. This does the filesystem side:

    1. renames the folder
    2. repairs git worktree metadata (absolute paths are baked into .git/worktrees/*)
    3. repoints the app's recent-projects list (%APPDATA%\SandiBumi\projects.json), which
       otherwise shows every project as "(missing)"
    4. carries over the Claude project memory folder, since Claude keys it on the project path

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File "D:\XX. SandiBumi\tools\rename-to-sandibumi.ps1"
  # (run it from its NEW location after the rename, or copy it out first - see note below)

.NOTES
  Chicken-and-egg: this script lives inside the folder it renames. That is fine - PowerShell
  reads the whole file before executing, so the script keeps running after its own file moves.
  Just do not run it with the working directory set inside the folder.
#>
param(
    [string]$OldPath = "D:\XX. Arshilla",
    [string]$NewName = "XX. SandiBumi"
)

$ErrorActionPreference = "Stop"
$NewPath = Join-Path (Split-Path $OldPath -Parent) $NewName

# Never operate from inside the folder being renamed - that handle is the whole problem.
Set-Location (Split-Path $OldPath -Parent)

if (-not (Test-Path -LiteralPath $OldPath)) {
    if (Test-Path -LiteralPath $NewPath) { Write-Host "Already renamed: $NewPath"; exit 0 }
    Write-Error "Source folder not found: $OldPath"; exit 1
}
if (Test-Path -LiteralPath $NewPath) { Write-Error "Target already exists: $NewPath"; exit 1 }

$item = Get-Item -LiteralPath $OldPath -Force
if ($item.LinkType) { Write-Error "$OldPath is a $($item.LinkType), not a real folder. Remove the link first."; exit 1 }

Write-Host "[1/4] renaming $OldPath -> $NewPath"
try {
    Rename-Item -LiteralPath $OldPath -NewName $NewName -ErrorAction Stop
} catch {
    Write-Host ""
    Write-Host "BLOCKED - a running process still holds the folder open." -ForegroundColor Yellow
    Write-Host "Close Claude Code, Cursor/VS Code, SandiBumi and any terminal sitting in that"
    Write-Host "folder, then run this again. (Claude Code is the usual culprit: the project"
    Write-Host "folder is its working directory, so it holds a handle for its whole lifetime.)"
    Write-Host ""
    Write-Error $_.Exception.Message
    exit 1
}
Write-Host "      renamed."

Write-Host "[2/4] repairing git worktree metadata"
# `git worktree repair` writes its progress report to stderr. Do NOT redirect it with 2>&1:
# in PS 5.1 that wraps each line in an ErrorRecord and trips $ErrorActionPreference = Stop,
# which made an earlier version of this script report a SUCCESSFUL repair as "skipped".
$prevEap = $ErrorActionPreference
$ErrorActionPreference = "Continue"
try {
    # A bare `worktree repair` fixes only the MAIN worktree; linked worktrees keep pointing at
    # the old path and go "prunable" (proven by the fixture test). Git needs their NEW paths
    # passed explicitly, so enumerate them under the renamed root and hand them over.
    $linked = @()
    $wtRoot = Join-Path $NewPath ".claude\worktrees"
    if (Test-Path -LiteralPath $wtRoot) {
        $linked = @(Get-ChildItem -LiteralPath $wtRoot -Directory | ForEach-Object { $_.FullName })
    }
    if ($linked.Count -gt 0) { & git -C $NewPath worktree repair @linked | Out-Null }
    else { & git -C $NewPath worktree repair | Out-Null }

    # Verify rather than assume: a stale linked worktree shows up as "prunable".
    $list = @(& git -C $NewPath worktree list)
    $list | ForEach-Object { Write-Host "      $_" }
    $stale = @($list | Select-String -Pattern "prunable").Count
    if ($stale -gt 0) { Write-Host "      WARNING: $stale worktree(s) still stale" -ForegroundColor Yellow }
    else { Write-Host "      worktrees ok" }
} catch { Write-Host "      (worktree repair reported: $($_.Exception.Message))" }
$ErrorActionPreference = $prevEap

Write-Host "[3/4] repointing the app's recent-projects list"
$recents = Join-Path $env:APPDATA "SandiBumi\projects.json"
if (Test-Path -LiteralPath $recents) {
    try {
        $raw = Get-Content -LiteralPath $recents -Raw -Encoding UTF8
        if ($raw -like "*$OldPath*") {
            Copy-Item -LiteralPath $recents -Destination "$recents.pre-rename-backup" -Force
            ($raw -replace [regex]::Escape($OldPath), $NewPath) |
                Set-Content -LiteralPath $recents -Encoding UTF8
            Write-Host "      updated (backup: projects.json.pre-rename-backup)"
        } else { Write-Host "      no stale paths - left alone" }
    } catch { Write-Host "      (skipped: $($_.Exception.Message))" }
} else { Write-Host "      none found - skipped" }

Write-Host "[4/4] carrying over the Claude project memory folder"
$claudeProjects = Join-Path $env:USERPROFILE ".claude\projects"
$oldKey = Join-Path $claudeProjects (($OldPath -replace '[:\\]', '-'))
$newKey = Join-Path $claudeProjects (($NewPath -replace '[:\\]', '-'))
$oldMem = Join-Path $oldKey "memory"
if ((Test-Path -LiteralPath $oldMem) -and (@(Get-ChildItem -LiteralPath $oldMem -Recurse -File -ErrorAction SilentlyContinue).Count -gt 0)) {
    New-Item -ItemType Directory -Force -Path $newKey | Out-Null
    Copy-Item -LiteralPath $oldMem -Destination $newKey -Recurse -Force
    Write-Host "      copied memory -> $newKey\memory (original left in place)"
} else { Write-Host "      memory folder empty or absent - nothing to carry over" }

Write-Host ""
Write-Host "DONE. The project now lives at: $NewPath" -ForegroundColor Green
Write-Host ""
Write-Host "Next:"
Write-Host "  1. Reopen Claude Code / your editor on $NewPath"
Write-Host "  2. Clear the stale Rust build cache BEFORE building - Tauri's build script bakes"
Write-Host "     the old absolute path into target\, so the first build FAILS with"
Write-Host "     'failed to read plugin permissions ... $OldPath\...' until you run:"
Write-Host "         cd src-tauri; cargo clean -p sandibumi -p tauri"
Write-Host "     (package-scoped on purpose - a full 'cargo clean' rebuilds DuckDB from source)"
Write-Host "  3. Verify with: powershell -ExecutionPolicy Bypass -File tools\check.ps1"
Write-Host ""
Write-Host "Known leftovers, both harmless:"
Write-Host "  - .vs\XX. Arshilla.slnx\  : Visual Studio cache, recreated under the new name"
Write-Host "  - docs\sandibumi_dev_playbook.md still has old paths (your file - left untouched)"
