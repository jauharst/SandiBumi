"""Generate docs/manual_test_plan.xlsx from docs/manual_test_plan.md.

The markdown stays the source of truth — this is a projection of it into a form that is
actually pleasant to work through: one row per test, a Result dropdown instead of three
checkboxes, and a Summary sheet whose counts update live as you mark.

The dropdown is a real improvement over the markdown, not just a convenience: a cell holds
exactly one value, so the "more than one box ticked" case that testplan-tally.ps1 has to
detect and quarantine cannot occur at all.

Usage:  py -3 tools/testplan-to-xlsx.py [in.md] [out.xlsx]
"""

import re
import sys
from pathlib import Path

from openpyxl import Workbook
from openpyxl.formatting.rule import CellIsRule
from openpyxl.styles import Alignment, Border, Font, PatternFill, Side
from openpyxl.utils import get_column_letter
from openpyxl.worksheet.datavalidation import DataValidation

FONT = "Arial"

# Earth tones lifted from the app's own light theme so the workbook and SandiBumi look
# related rather than accidentally different.
INK = "332A1F"
DIM = "7C6B52"
RULE = "DDD0AF"
HEAD_BG = "E9DFC8"
INPUT_BG = "FFF7D6"  # the cells you fill
PASS_BG, FAIL_BG, BLOCK_BG = "D8EAD3", "F6D5D2", "FCE8C8"
PASS_FG, FAIL_FG, BLOCK_FG = "1E5E22", "9C2620", "8A5200"

SECTION_RE = re.compile(r"^# Section (\S+)\s*[—-]\s*(.*)$")
TEST_RE = re.compile(r"^### (T-[A-Za-z]+-\d+)\s*[—-]\s*(.*)$")


def clean(text: str) -> str:
    """Strip markdown bold markers; keep everything else readable as plain text."""
    text = re.sub(r"\*\*(.+?)\*\*", r"\1", text, flags=re.S)
    return re.sub(r"\n{3,}", "\n\n", text).strip()


def field(block: str, label: str) -> str:
    """Pull `**Label:** value` up to the next bold label or end of block."""
    m = re.search(
        rf"\*\*{label}:?\*\*:?\s*(.*?)(?=\n\s*\*\*[A-Z]|\Z)", block, flags=re.S
    )
    return clean(m.group(1)) if m else ""


def parse(md_path: Path):
    lines = md_path.read_text(encoding="utf-8").splitlines()
    sections, tests = [], []
    sec_code = sec_name = ""
    cur = None
    buf: list[str] = []

    def flush():
        if cur is None:
            return
        block = "\n".join(buf)
        # Steps run from **Steps:** to the **Result — T-…:** anchor, and deliberately keep
        # the inline Expected/Known-issue lines: that is the order you read them in.
        m = re.search(r"\*\*Steps:\*\*\s*(.*?)(?=\*\*Result\s*[—-])", block, flags=re.S)
        cur["steps"] = clean(m.group(1)) if m else ""
        cur["tool"] = field(block, "Tool/panel")
        cur["pre"] = field(block, "Preconditions")
        cur["known"] = "Yes" if re.search(r"\*\*Known issue", block) else ""
        tests.append(cur)

    for line in lines:
        ms = SECTION_RE.match(line)
        if ms:
            flush()
            cur, buf = None, []
            sec_code, sec_name = ms.group(1), ms.group(2).strip()
            sections.append((sec_code, sec_name))
            continue
        mt = TEST_RE.match(line)
        if mt:
            flush()
            buf = []
            cur = {
                "sec": sec_code,
                "id": mt.group(1),
                "title": mt.group(2).strip(),
            }
            continue
        if cur is not None:
            buf.append(line)
    flush()
    return sections, tests


def style_header(ws, ncols):
    thin = Side(style="thin", color=RULE)
    for c in range(1, ncols + 1):
        cell = ws.cell(row=1, column=c)
        cell.font = Font(name=FONT, bold=True, size=10, color=INK)
        cell.fill = PatternFill("solid", fgColor=HEAD_BG)
        cell.alignment = Alignment(vertical="center", wrap_text=True)
        cell.border = Border(bottom=thin)
    ws.row_dimensions[1].height = 30


def build(md_path: Path, out_path: Path):
    sections, tests = parse(md_path)
    if len(tests) != 250:
        print(f"WARNING: parsed {len(tests)} tests, expected 250")

    wb = Workbook()

    # ---------------------------------------------------------------- Tests
    ws = wb.active
    ws.title = "Tests"
    headers = [
        "Section", "Test ID", "Test", "Result", "Notes",
        "Known issue", "Tool / panel", "Preconditions", "Steps & expected",
    ]
    ws.append(headers)
    style_header(ws, len(headers))

    body = Font(name=FONT, size=10, color=INK)
    top = Alignment(vertical="top", wrap_text=True)
    for t in tests:
        ws.append([
            t["sec"], t["id"], t["title"], "", "",
            t["known"], t["tool"], t["pre"], t["steps"],
        ])

    last = ws.max_row
    for row in ws.iter_rows(min_row=2, max_row=last, max_col=len(headers)):
        for cell in row:
            cell.font = body
            cell.alignment = top
        row[3].fill = PatternFill("solid", fgColor=INPUT_BG)   # Result
        row[4].fill = PatternFill("solid", fgColor=INPUT_BG)   # Notes
        row[3].alignment = Alignment(vertical="center", horizontal="center")
        row[5].alignment = Alignment(vertical="center", horizontal="center")

    widths = [9, 14, 40, 11, 40, 9, 26, 34, 96]
    for i, w in enumerate(widths, start=1):
        ws.column_dimensions[get_column_letter(i)].width = w

    # showDropDown is inverted in the OOXML spec — the attribute means "suppress the in-cell
    # dropdown", so False is what puts the arrow in the cell. showErrorMessage must be set
    # explicitly: openpyxl leaves it off, and without it Excel accepts any typed value and the
    # list is merely advisory (verified via Excel COM: ShowError came back False).
    # showInputMessage stays off on purpose — a tooltip firing on every one of 250 cells during
    # a long click-through is an irritation, and the arrow is self-explanatory.
    dv = DataValidation(
        type="list",
        formula1='"Pass,Fail,Blocked"',
        allow_blank=True,
        showDropDown=False,
        showErrorMessage=True,
    )
    dv.errorStyle = "stop"
    dv.errorTitle = "Not a valid result"
    dv.error = "Pick Pass, Fail or Blocked from the dropdown, or leave the cell blank."
    ws.add_data_validation(dv)
    dv.add(f"D2:D{last}")

    rng = f"D2:D{last}"
    ws.conditional_formatting.add(rng, CellIsRule(
        operator="equal", formula=['"Pass"'],
        fill=PatternFill("solid", fgColor=PASS_BG), font=Font(name=FONT, bold=True, color=PASS_FG)))
    ws.conditional_formatting.add(rng, CellIsRule(
        operator="equal", formula=['"Fail"'],
        fill=PatternFill("solid", fgColor=FAIL_BG), font=Font(name=FONT, bold=True, color=FAIL_FG)))
    ws.conditional_formatting.add(rng, CellIsRule(
        operator="equal", formula=['"Blocked"'],
        fill=PatternFill("solid", fgColor=BLOCK_BG), font=Font(name=FONT, bold=True, color=BLOCK_FG)))

    ws.auto_filter.ref = f"A1:{get_column_letter(len(headers))}{last}"
    ws.freeze_panes = "D2"   # Section / ID / Test stay put while you scroll right

    # -------------------------------------------------------------- Summary
    sm = wb.create_sheet("Summary")
    sm.append(["Code", "Section", "Tests", "Pass", "Fail", "Blocked", "Untested", "Done"])
    style_header(sm, 8)

    n = len(tests)
    for i, (code, name) in enumerate(sections, start=2):
        sm.append([
            code, name,
            f'=COUNTIF(Tests!$A$2:$A${n + 1},$A{i})',
            f'=COUNTIFS(Tests!$A$2:$A${n + 1},$A{i},Tests!$D$2:$D${n + 1},"Pass")',
            f'=COUNTIFS(Tests!$A$2:$A${n + 1},$A{i},Tests!$D$2:$D${n + 1},"Fail")',
            f'=COUNTIFS(Tests!$A$2:$A${n + 1},$A{i},Tests!$D$2:$D${n + 1},"Blocked")',
            f'=C{i}-D{i}-E{i}-F{i}',
            f'=IFERROR((D{i}+E{i}+F{i})/C{i},0)',
        ])
    tot = len(sections) + 2
    sm.append([
        "", "TOTAL",
        *[f"=SUM({get_column_letter(c)}2:{get_column_letter(c)}{tot - 1})" for c in range(3, 8)],
        f'=IFERROR((D{tot}+E{tot}+F{tot})/C{tot},0)',
    ])

    for row in sm.iter_rows(min_row=2, max_row=tot, max_col=8):
        for cell in row:
            cell.font = Font(name=FONT, size=10, color=INK)
        row[7].number_format = "0%"
    for c in range(1, 9):
        for cell in (sm.cell(row=tot, column=c),):
            cell.font = Font(name=FONT, size=10, bold=True, color=INK)
    for col, w in zip("ABCDEFGH", [9, 46, 8, 8, 8, 10, 10, 8]):
        sm.column_dimensions[col].width = w
    sm.freeze_panes = "A2"

    sm.conditional_formatting.add(f"E2:E{tot}", CellIsRule(
        operator="greaterThan", formula=["0"],
        fill=PatternFill("solid", fgColor=FAIL_BG), font=Font(name=FONT, bold=True, color=FAIL_FG)))
    sm.conditional_formatting.add(f"F2:F{tot}", CellIsRule(
        operator="greaterThan", formula=["0"],
        fill=PatternFill("solid", fgColor=BLOCK_BG), font=Font(name=FONT, bold=True, color=BLOCK_FG)))

    # ------------------------------------------------------------ How to use
    hw = wb.create_sheet("How to use", 0)
    rows = [
        ("SandiBumi — manual test plan", ""),
        ("", ""),
        ("Generated from docs/manual_test_plan.md, which stays the source of truth.", ""),
        ("Regenerate any time with:  py -3 tools/testplan-to-xlsx.py", ""),
        ("", ""),
        ("WHICH CELLS YOU FILL IN", ""),
        ("The two cream-shaded columns on the Tests sheet, and nothing else:", ""),
        ("   Result (column D)", "pick Pass / Fail / Blocked from the dropdown"),
        ("   Notes  (column E)", "free text - what went wrong, or a timing for the PERF tests"),
        ("Everything else is reference text. Summary is all formulas - do not type in it.", ""),
        ("", ""),
        ("EXAMPLE OF A FILLED ROW", ""),
        ("Section", "SHELL"),
        ("Test ID", "T-SHELL-02"),
        ("Test", "Ribbon tab walk + overflow chevrons"),
        ("Result", "Fail"),
        ("Notes", "chevron did not appear until the window went below 900px"),
        ("", ""),
        ("HOW TO WORK THROUGH IT", ""),
        ("1. Sort or filter the Tests sheet by Section and work one section at a time.", ""),
        ("2. Mark exactly one result per test. Leave it blank until you have actually run it -", ""),
        ("   blank means untested, and the Summary counts it that way.", ""),
        ("3. A test whose precondition failed is Blocked, not Fail.", ""),
        ("4. Known issue = Yes means it is EXPECTED to fail in one specific way, already", ""),
        ("   confirmed in AUDIT-2026-07-21-full-qc.md. Mark it Fail and note 'known'. If it", ""),
        ("   fails a DIFFERENT way, that is a new finding - say so in Notes.", ""),
        ("5. Watch Summary as you go; it updates live.", ""),
        ("6. When done, filter Result to Fail and Blocked and send me that list.", ""),
        ("", ""),
        ("WHY THERE IS NO 'CONTRADICTORY' COLUMN", ""),
        ("The markdown version has three checkboxes per test, so it can be ticked", ""),
        ("inconsistently and the tally script has to quarantine those. A dropdown cell holds", ""),
        ("exactly one value, so that failure mode does not exist here.", ""),
    ]
    for a, b in rows:
        hw.append([a, b])
    hw["A1"].font = Font(name=FONT, size=15, bold=True, color=INK)
    for r in range(2, len(rows) + 1):
        a, b = hw.cell(row=r, column=1), hw.cell(row=r, column=2)
        heading = a.value and a.value.isupper() and len(a.value) < 60
        a.font = Font(name=FONT, size=10, bold=bool(heading), color=INK if heading else DIM)
        b.font = Font(name=FONT, size=10, color=INK)
    for r in (8, 9, 13, 14, 15, 16, 17):
        hw.cell(row=r, column=1).font = Font(name=FONT, size=10, color=INK)
    for r in (16, 17):
        hw.cell(row=r, column=2).fill = PatternFill("solid", fgColor=INPUT_BG)
    hw.column_dimensions["A"].width = 78
    hw.column_dimensions["B"].width = 58
    hw.sheet_view.showGridLines = False

    wb.save(out_path)
    print(f"wrote {out_path}  ({len(tests)} tests, {len(sections)} sections)")


if __name__ == "__main__":
    root = Path(__file__).resolve().parent.parent
    src = Path(sys.argv[1]) if len(sys.argv) > 1 else root / "docs" / "manual_test_plan.md"
    dst = Path(sys.argv[2]) if len(sys.argv) > 2 else root / "docs" / "manual_test_plan.xlsx"
    build(src, dst)
