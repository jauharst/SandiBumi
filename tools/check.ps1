# SandiBumi green gate — ONE command that proves the tree is healthy.
# (docs/V1_SCOPE.md Q3 / docs/RELEASE.md §5 step 0.)
#
#   powershell -ExecutionPolicy Bypass -File tools\check.ps1
#
# Runs, in order, exiting non-zero at the FIRST failure:
#   1. takeover ledger — PRD source rows, tracker rows and dashboard agree;
#      the generated distributed-dependency licence inventory is current
#   2. verification matrix — generated output agrees with REVIEW.md + capability map
#   3. frontend gate — `npm run build` (= tsc && vite build; tsc runs inside it,
#      so a separate `tsc --noEmit` pass would only duplicate work)
#   4. backend gate  — `cargo test` in src-tauri, through vcvars pinned to 14.29
#      when that toolset exists (the reference machine's 14.50 is broken — missing
#      clui.dll, see CLAUDE.md); plain `cargo test` on a healthy machine.
#
# Flags for the inner loop (the FULL gate is what "green" means — flags are for
# iterating, not for release):
#   -SkipRust       frontend-only (skips cargo test)
#   -SkipFrontend   backend-only  (skips npm run build)
#   -VcVarsVer      MSVC toolset pin (default 14.29)
#   -Ignored        ALSO run the #[ignore]d shelf (optional-package + field-fixture
#                   tests) after the normal suite. Not part of the green gate - the
#                   gate must stay green on a fresh clone - but the shelf rots when
#                   nothing runs it (five stale tests proved that on 2026-08-18), so
#                   run this after any large stretch on a machine with the packages
#                   and SANDIBUMI_FIELD_FIXTURES set.
#
# Windows PowerShell 5.1 compatible: no `&&`, no ternary.

param(
    [switch]$SkipRust,
    [switch]$SkipFrontend,
    [switch]$Ignored,
    [string]$VcVarsVer = "14.29"
)

$ErrorActionPreference = "Continue"
$repo = Split-Path -Parent $PSScriptRoot   # tools\ -> repo root

# Machine note (CLAUDE.md): fresh shells sometimes miss installer PATH updates —
# prepend the known homes so the gate works from any shell.
$env:PATH = "C:\Program Files\nodejs;$env:USERPROFILE\.cargo\bin;$env:PATH"

function Fail([string]$stage, [int]$code) {
    Write-Host ""
    Write-Host ("GATE FAILED at {0} (exit {1})" -f $stage, $code) -ForegroundColor Red
    exit 1
}

$total = [System.Diagnostics.Stopwatch]::StartNew()

# --- Stage 1: takeover ledger ------------------------------------------------
Write-Host "[1/4] takeover ledger: source and tracker agree..." -ForegroundColor Cyan
$sw = [System.Diagnostics.Stopwatch]::StartNew()
Push-Location $repo
& npm run test:takeover-ledger
$code = $LASTEXITCODE
if ($code -eq 0) {
    & npm run check:takeover-ledger
    $code = $LASTEXITCODE
}
if ($code -eq 0) {
    & npm run test:gate2-program
    $code = $LASTEXITCODE
}
if ($code -eq 0) {
    & npm run check:gate2-program
    $code = $LASTEXITCODE
}
if ($code -eq 0) {
    & npm run test:gate2-hygiene
    $code = $LASTEXITCODE
}
if ($code -eq 0) {
    & npm run check:gate2-hygiene
    $code = $LASTEXITCODE
}
if ($code -eq 0) {
    & npm run test:unit-registry
    $code = $LASTEXITCODE
}
if ($code -eq 0) {
    & npm run check:unit-registry
    $code = $LASTEXITCODE
}
if ($code -eq 0) {
    & npm run test:chart-derivation
    $code = $LASTEXITCODE
}
if ($code -eq 0) {
    & npm run check:chart-derivation
    $code = $LASTEXITCODE
}
if ($code -eq 0) {
    & npm run test:release-inventory
    $code = $LASTEXITCODE
}
if ($code -eq 0) {
    & node "tools/gen-third-party-licenses.mjs" --check
    $code = $LASTEXITCODE
}
Pop-Location
if ($code -ne 0) { Fail "takeover ledger" $code }
Write-Host ("[1/4] takeover ledger green in {0:n0}s" -f $sw.Elapsed.TotalSeconds) -ForegroundColor Green

# --- Stage 2: capability verification matrix --------------------------------
Write-Host "[2/4] verification matrix: generated file is current..." -ForegroundColor Cyan
$sw = [System.Diagnostics.Stopwatch]::StartNew()
Push-Location $repo
& node "tools/generate-verification-matrix.mjs" --check
$code = $LASTEXITCODE
Pop-Location
if ($code -ne 0) { Fail "verification matrix" $code }
Write-Host ("[2/4] verification matrix green in {0:n0}s" -f $sw.Elapsed.TotalSeconds) -ForegroundColor Green

# --- Stage 3: frontend (tsc + vite build) -----------------------------------
if (-not $SkipFrontend) {
    Write-Host "[3/4] frontend gate: npm run build (tsc + vite)..." -ForegroundColor Cyan
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    Push-Location $repo
    & npm run test:frontend
    $code = $LASTEXITCODE
    if ($code -ne 0) {
        Pop-Location
        Fail "frontend acceptance tests" $code
    }
    & npm run build
    $code = $LASTEXITCODE
    Pop-Location
    if ($code -ne 0) { Fail "frontend (tsc + vite build)" $code }
    Write-Host ("[3/4] frontend green in {0:n0}s" -f $sw.Elapsed.TotalSeconds) -ForegroundColor Green
} else {
    Write-Host "[3/4] frontend gate SKIPPED (-SkipFrontend)" -ForegroundColor Yellow
}

# --- Stage 4: backend (cargo test, pinned toolchain when present) -----------
if (-not $SkipRust) {
    Write-Host "[4/4] backend gate: cargo test..." -ForegroundColor Cyan
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $vcvars = "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat"
    if (Test-Path $vcvars) {
        # cmd's own && chains inside the string; $LASTEXITCODE propagates from cmd /c.
        cmd /c "call `"$vcvars`" -vcvars_ver=$VcVarsVer && cd /d `"$repo\src-tauri`" && cargo test"
        $code = $LASTEXITCODE
    } else {
        Push-Location (Join-Path $repo "src-tauri")
        & cargo test
        $code = $LASTEXITCODE
        Pop-Location
    }
    if ($code -ne 0) { Fail "backend (cargo test)" $code }
    Write-Host ("[4/4] backend green in {0:n0}s" -f $sw.Elapsed.TotalSeconds) -ForegroundColor Green
    if ($Ignored) {
        Write-Host "[4b] ignored shelf: cargo test -- --ignored..." -ForegroundColor Cyan
        $sw2 = [System.Diagnostics.Stopwatch]::StartNew()
        if (Test-Path $vcvars) {
            cmd /c "call `"$vcvars`" -vcvars_ver=$VcVarsVer && cd /d `"$repo\src-tauri`" && cargo test --lib -- --ignored"
            $code = $LASTEXITCODE
        } else {
            Push-Location (Join-Path $repo "src-tauri")
            & cargo test --lib -- --ignored
            $code = $LASTEXITCODE
            Pop-Location
        }
        if ($code -ne 0) { Fail "ignored shelf (cargo test -- --ignored)" $code }
        Write-Host ("[4b] ignored shelf green in {0:n0}s" -f $sw2.Elapsed.TotalSeconds) -ForegroundColor Green
    }
} else {
    Write-Host "[4/4] backend gate SKIPPED (-SkipRust)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host ("GATE GREEN in {0:n0}s" -f $total.Elapsed.TotalSeconds) -ForegroundColor Green
exit 0
