<#
.SYNOPSIS
  Counts the ticked checkboxes in docs/manual_test_plan.md and prints the tally.

.DESCRIPTION
  The plan ends every test in a `- [ ] Pass / Fail / Blocked` task list. This reads those
  marks back: per-section counts for the doc's Tally table, then the Fail/Blocked list with
  each test's Notes — which is exactly what step 6 of "How to use this plan" asks you to hand
  back for fixing.

  A test with MORE THAN ONE box ticked is reported separately and counted in none of the
  columns: a contradictory mark must never be silently scored as a pass.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File tools\testplan-tally.ps1
#>
param([string]$Path)

if (-not $Path) { $Path = Join-Path (Split-Path $PSScriptRoot -Parent) "docs\manual_test_plan.md" }
if (-not (Test-Path -LiteralPath $Path)) { Write-Error "test plan not found: $Path"; exit 1 }

$lines = Get-Content -LiteralPath $Path -Encoding UTF8
$sectionOrder = New-Object System.Collections.ArrayList
$tests = New-Object System.Collections.ArrayList
$currentSection = "(none)"
$cur = $null

foreach ($line in $lines) {
    if ($line -match '^# Section (\S+)') {
        $currentSection = $Matches[1]
        if (-not $sectionOrder.Contains($currentSection)) { [void]$sectionOrder.Add($currentSection) }
        continue
    }
    if ($line -match '^### (T-[A-Za-z]+-\d+)') {
        $cur = [pscustomobject]@{
            Id      = $Matches[1]
            Section = $currentSection
            Marks   = New-Object System.Collections.ArrayList
            Notes   = ""
        }
        [void]$tests.Add($cur)
        continue
    }
    if ($null -ne $cur) {
        if ($line -match '^- \[[xX]\]\s*(Pass|Fail|Blocked)') { [void]$cur.Marks.Add($Matches[1]) }
        elseif ($line -match '^\*\*Notes:\*\*\s*(.*)$') { $cur.Notes = $Matches[1].Trim() }
    }
}

$rows = New-Object System.Collections.ArrayList
$totals = @{ Tests = 0; Pass = 0; Fail = 0; Blocked = 0; Untested = 0; Bad = 0 }

foreach ($s in $sectionOrder) {
    $inSection = @($tests | Where-Object { $_.Section -eq $s })
    if ($inSection.Count -eq 0) { continue }
    $row = @{ Pass = 0; Fail = 0; Blocked = 0; Untested = 0; Bad = 0 }
    foreach ($t in $inSection) {
        if ($t.Marks.Count -eq 0) { $row.Untested++ }
        elseif ($t.Marks.Count -gt 1) { $row.Bad++ }
        else { $row[$t.Marks[0]]++ }
    }
    [void]$rows.Add([pscustomobject]@{
        Section = $s; Tests = $inSection.Count; Pass = $row.Pass; Fail = $row.Fail
        Blocked = $row.Blocked; Untested = $row.Untested; Contradictory = $row.Bad
    })
    $totals.Tests += $inSection.Count
    foreach ($k in @("Pass", "Fail", "Blocked", "Untested", "Bad")) { $totals[$k] += $row[$k] }
}

[void]$rows.Add([pscustomobject]@{
    Section = "TOTAL"; Tests = $totals.Tests; Pass = $totals.Pass; Fail = $totals.Fail
    Blocked = $totals.Blocked; Untested = $totals.Untested; Contradictory = $totals.Bad
})

Write-Output ""
$rows | Format-Table -AutoSize

$bad = @($tests | Where-Object { $_.Marks.Count -gt 1 })
if ($bad.Count -gt 0) {
    Write-Output "CONTRADICTORY MARKS (more than one box ticked - counted in no column, fix these first):"
    foreach ($t in $bad) { Write-Output ("  {0}  ticked: {1}" -f $t.Id, ($t.Marks -join ", ")) }
    Write-Output ""
}

$attention = @($tests | Where-Object { $_.Marks.Count -eq 1 -and $_.Marks[0] -ne "Pass" })
if ($attention.Count -gt 0) {
    Write-Output "FAIL / BLOCKED - hand this list back for fixing:"
    foreach ($t in $attention) {
        $note = $t.Notes
        if (($note -eq "") -or ($note -match '^_+$')) { $note = "(no notes)" }
        Write-Output ("  [{0}] {1} - {2}" -f $t.Marks[0].ToUpper(), $t.Id, $note)
    }
    Write-Output ""
}

if ($totals.Untested -eq $totals.Tests) {
    Write-Output "Nothing ticked yet - tick a Pass/Fail/Blocked box under any test, then re-run."
}
