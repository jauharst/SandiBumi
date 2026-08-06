# Run Claude Code on the Kimi Code subscription (Kimi K3).
#
#   powershell -ExecutionPolicy Bypass -File tools\kimi.ps1 -Verify
#   powershell -ExecutionPolicy Bypass -File tools\kimi.ps1
#   powershell -ExecutionPolicy Bypass -File tools\kimi.ps1 -Task "…scoped brief…"
#   powershell -ExecutionPolicy Bypass -File tools\kimi.ps1 -Task "…" -Worktree kimi-panes
#
# See docs/delegation_kimi.md for WHAT to hand it and what never to. In short: mechanical,
# compiler-gated, convention-light work. Never the numeric modules, never a physics default,
# never the DuckDB write discipline — those fail silently and no gate catches them.
#
# THE KEY. Two different Kimi endpoints exist and picking the wrong one is the usual failure:
#   subscription  -> key from kimi.com/code/console   -> https://api.kimi.com/coding/
#   pay-per-token -> key from platform.kimi.ai        -> https://api.moonshot.ai/anthropic
# A platform.kimi.ai key bills per token and ignores the subscription entirely. This script
# defaults to the SUBSCRIPTION endpoint; -PayPerToken switches it deliberately.
#
# Supply the key as $env:KIMI_CODE_KEY, or put it alone in a file `.kimi-key` at the repo root
# (gitignored). Never pass it on the command line — it lands in shell history.
#
# Windows PowerShell 5.1 compatible: no `&&`, no ternary.

param(
    [string]$Task,
    [string]$Worktree,
    [string]$Model = "k3[1m]",
    [switch]$Verify,
    [switch]$PayPerToken
)

$ErrorActionPreference = "Continue"
$repo = Split-Path -Parent $PSScriptRoot   # tools\ -> repo root

# Machine note (CLAUDE.md): fresh shells sometimes miss installer PATH updates.
$env:PATH = "C:\Program Files\nodejs;$env:USERPROFILE\.cargo\bin;$env:PATH"

function Die([string]$msg) {
    Write-Host ""
    Write-Host $msg -ForegroundColor Red
    exit 1
}

# ---- key -------------------------------------------------------------------------------
$key = $env:KIMI_CODE_KEY
if ([string]::IsNullOrWhiteSpace($key)) {
    $keyFile = Join-Path $repo ".kimi-key"
    if (Test-Path $keyFile) {
        $key = (Get-Content $keyFile -Raw).Trim()
    }
}
if ([string]::IsNullOrWhiteSpace($key)) {
    Die "No Kimi key. Set `$env:KIMI_CODE_KEY, or write it into .kimi-key at the repo root.`nSubscription keys are created at kimi.com/code/console (up to 5, each shown only once)."
}

# ---- endpoint --------------------------------------------------------------------------
$baseUrl = "https://api.kimi.com/coding/"
$flavour = "subscription (kimi.com/code/console)"
if ($PayPerToken) {
    $baseUrl = "https://api.moonshot.ai/anthropic"
    $flavour = "PAY-PER-TOKEN (platform.kimi.ai) - this bills per token"
}

# ---- context window --------------------------------------------------------------------
# The k3[1m] bracket form is a Claude Code env-var convention only, and needs Allegretto or
# above. Plain k3 is 256K.
$ctx = 262144
if ($Model -like "*[[]1m[]]*") { $ctx = 1048576 }

# ---- environment -----------------------------------------------------------------------
# EVERY model-tier variable must be set. A missing one does not error — it makes subagents and
# background tasks fail silently on an unknown model.
$env:ANTHROPIC_BASE_URL             = $baseUrl
$env:ANTHROPIC_AUTH_TOKEN           = $key
$env:ANTHROPIC_MODEL                = $Model
$env:ANTHROPIC_DEFAULT_OPUS_MODEL   = $Model
$env:ANTHROPIC_DEFAULT_SONNET_MODEL = $Model
$env:ANTHROPIC_DEFAULT_HAIKU_MODEL  = $Model
$env:ANTHROPIC_DEFAULT_FABLE_MODEL  = $Model
$env:CLAUDE_CODE_SUBAGENT_MODEL     = $Model
$env:CLAUDE_CODE_MAX_CONTEXT_TOKENS = $ctx
$env:CLAUDE_CODE_AUTO_COMPACT_WINDOW = $ctx

# A leftover Anthropic key silently conflicts with ANTHROPIC_AUTH_TOKEN and fails in a way that
# reads like a network fault. Clear it for this process only.
Remove-Item Env:\ANTHROPIC_API_KEY -ErrorAction SilentlyContinue

# ---- worktree --------------------------------------------------------------------------
# Hand-off from a live Claude session: isolate the tree so two agents are not editing one
# checkout. Branch name follows the repo convention (feat/ fix/ chore/ docs/ + slug).
$workDir = $repo
if (-not [string]::IsNullOrWhiteSpace($Worktree)) {
    $workDir = Join-Path (Split-Path -Parent $repo) ("SandiBumi-" + $Worktree)
    if (-not (Test-Path $workDir)) {
        Write-Host ("Creating worktree {0} on branch feat/{1}" -f $workDir, $Worktree) -ForegroundColor Cyan
        git -C $repo worktree add -b ("feat/" + $Worktree) $workDir
        if ($LASTEXITCODE -ne 0) { Die "git worktree add failed." }
    }
}

Write-Host ""
Write-Host ("Kimi Code  model={0}  ctx={1}  {2}" -f $Model, $ctx, $flavour) -ForegroundColor Cyan
Write-Host ("Working in {0}" -f $workDir) -ForegroundColor DarkGray
Write-Host "Verify inside the session with /status — /model never lists Kimi." -ForegroundColor DarkGray
Write-Host ""

Push-Location $workDir
try {
    if ($Verify) {
        claude -p "Reply with exactly: KIMI-OK"
        Write-Host ""
        Write-Host "Expect KIMI-OK above. Anything else means the key or base URL is wrong." -ForegroundColor DarkGray
    }
    elseif (-not [string]::IsNullOrWhiteSpace($Task)) {
        # Bounds are restated here on purpose: K3 fills ambiguity by deciding, so the rules that
        # bind the task must be in the prompt rather than left to CLAUDE.md alone.
        $brief = @"
$Task

Bounds for this task, which override any instinct to fill in gaps:
- Change ONLY the files named above. If the work seems to need another file, stop and say so.
- Do NOT invent or adjust any physics default, published coefficient, or threshold. If one is
  missing, stop and say so — every such value needs a cited source.
- Do NOT touch the numeric modules (equations, multimin, multimin2, ssc, lrlc, satheight,
  thomeer, hfu, montecarlo, petrography, distribution) or any DuckDB write path.
- No client, operator, field, block or well identifier anywhere in code, tests or comments.
- Finish by running `npx tsc --noEmit` and, if Rust changed, `cd src-tauri; cargo check`.
  Report the actual output. Do not claim green without it.
- Commit to the current branch. Never push to master.
"@
        claude -p $brief
    }
    else {
        claude
    }
}
finally {
    Pop-Location
}
