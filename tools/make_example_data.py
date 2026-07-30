#!/usr/bin/env python3
"""Generates the example import datasets in `dataset for test/examples/`.

Stdlib-only and fully deterministic (seeded LCG noise) — running it twice produces
byte-identical files, so regeneration never dirties the git diff unless the recipe
changed. One synthetic field, three wells (SANDI-01/02/03), one shared geology:

    SHALE-1  (cap)          GR ~110, RHOB ~2.45, NPHI ~0.38, ILD ~2.5
    SAND-A   (gas-bearing)  GR ~45,  RHOB ~2.22, NPHI ~0.14, ILD ~120   <- N/D gas crossover
    SAND-B   (water-bearing)GR ~52,  RHOB ~2.34, NPHI ~0.27, ILD ~3.2
    SHALE-2  (base seal)

The same RHOB profile that goes into the LAS also drives the core-plug porosities, so
density-porosity vs core cross-checks agree the way a real consistent delivery would.
Zone tops shift per well (SANDI-02 is 10 m deeper, SANDI-03 5 m shallower) so tops
import + multi-well plots have real structure to show.

Every file matches what `src-tauri/src/parsers.rs` accepts TODAY; the cargo test
`example_data_test.rs` parses each one, so a parser change that breaks an example
fails the gate loudly.

Usage:  py -3 tools/make_example_data.py
"""

from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "dataset for test" / "examples"
NULL = -999.25
STEP = 0.1524  # half-foot metric grid, the app's standard

# Per-well structural shift (m) applied to every zone top and the LAS window.
WELLS = {"SANDI-01": 0.0, "SANDI-02": +10.0, "SANDI-03": -5.0}
# Zone tops for SANDI-01 (shift applies per well).
TOPS = {"TOP_SAND_A": 1520.0, "TOP_SAND_B": 1535.0, "TOP_SHALE_2": 1550.0}
LAS_TOP, LAS_BASE = 1500.0, 1560.0

# Zone endpoint values per curve: (shale1, sand_a_gas, sand_b_water, shale2)
ZONES = {
    "GR":   (110.0, 45.0, 52.0, 105.0),
    "CALI": (10.5, 8.6, 8.6, 10.0),
    "SP":   (-20.0, -75.0, -70.0, -25.0),
    "ILD":  (2.5, 120.0, 3.2, 2.2),
    "NPHI": (0.38, 0.14, 0.27, 0.36),
    "RHOB": (2.45, 2.22, 2.34, 2.48),
    "DT":   (118.0, 88.0, 95.0, 115.0),
    "PEF":  (3.2, 1.9, 2.0, 3.1),
}
NOISE = {"GR": 4.0, "CALI": 0.15, "SP": 2.0, "ILD": 0.06, "NPHI": 0.012,
         "RHOB": 0.015, "DT": 2.0, "PEF": 0.08}  # ILD noise is multiplicative (log-domain)


class Lcg:
    """Tiny deterministic PRNG (numerical recipes LCG) — no `random` module, so the
    output can never drift with a Python upgrade."""

    def __init__(self, seed: int):
        self.state = seed & 0xFFFFFFFF

    def next(self) -> float:  # uniform [0, 1)
        self.state = (1664525 * self.state + 1013904223) & 0xFFFFFFFF
        return self.state / 2**32

    def gauss(self) -> float:  # ~N(0,1) via sum of 6 uniforms (plenty for log noise)
        return (sum(self.next() for _ in range(6)) - 3.0) * (2.0 / 6.0) ** -0.5 / 3.0


def smooth(x: float) -> float:
    x = max(0.0, min(1.0, x))
    return x * x * (3.0 - 2.0 * x)


def zone_blend(depth: float, shift: float, vals: tuple) -> float:
    """Piecewise zone value with 1.5 m smooth transitions at each shifted top."""
    bounds = [TOPS["TOP_SAND_A"] + shift, TOPS["TOP_SAND_B"] + shift, TOPS["TOP_SHALE_2"] + shift]
    v = vals[0]
    for b, nxt in zip(bounds, vals[1:]):
        v = v + (nxt - v) * smooth((depth - b) / 1.5 + 0.5)
    return v


def curve_value(name: str, depth: float, shift: float, rng: Lcg) -> float:
    base = zone_blend(depth, shift, ZONES[name])
    if name == "ILD":  # resistivity noise belongs in log domain
        return base * 10 ** (NOISE[name] * rng.gauss() * 0.15)
    return base + NOISE[name] * rng.gauss()


def make_las(well: str, shift: float, seed: int) -> str:
    top, base = LAS_TOP + shift, LAS_BASE + shift
    n = int(round((base - top) / STEP)) + 1
    stop = top + (n - 1) * STEP
    rng = Lcg(seed)
    hdr = f"""~Version Information
 VERS.                 2.0 : CWLS Log ASCII Standard - Version 2.0
 WRAP.                  NO : One line per depth step
~Well Information
#MNEM.UNIT       Data                    Description
#---- -----      ----------------       -----------------------------
 STRT.M           {top:.4f}             : Start depth
 STOP.M           {stop:.4f}             : Stop depth
 STEP.M           {STEP:.4f}                : Step
 NULL.            {NULL}               : Null value
 WELL.            {well}              : Well name
 FLD .            SANDI                 : Field
 LOC .            Mahakam Delta (synthetic example) : Location
 SRVC.            SandiBumi             : Service company
 DATE.            2026-07-30            : Log date
~Curve Information
#MNEM.UNIT       API Code               Description
#---- -----      ----------------       -----------------------------
 DEPT.M                                 : Measured depth
 GR  .GAPI                              : Gamma ray
 CALI.IN                                : Caliper
 SP  .MV                                : Spontaneous potential
 ILD .OHMM                              : Deep induction resistivity
 NPHI.V/V                               : Neutron porosity (limestone)
 RHOB.G/C3                              : Bulk density
 DT  .US/F                              : Compressional slowness
 PEF .B/E                               : Photoelectric factor
~ASCII
"""
    lines = [hdr]
    # A deliberate 1-m NPHI/PEF null gap mid-SAND-A in every well: exercises the app's
    # NULL handling and gives Bad-Hole QC something real to flag (CALI washes out there).
    gap_top = TOPS["TOP_SAND_A"] + shift + 6.0
    gap_base = gap_top + 1.0
    for i in range(n):
        d = top + i * STEP
        in_gap = gap_top <= d <= gap_base
        vals = {name: curve_value(name, d, shift, rng) for name in ZONES}
        if in_gap:
            vals["NPHI"] = NULL
            vals["PEF"] = NULL
            vals["CALI"] += 2.5  # washout
        nphi = f"{NULL:8.2f}" if in_gap else f"{vals['NPHI']:8.4f}"
        pef = f"{NULL:8.2f}" if in_gap else f"{vals['PEF']:8.3f}"
        lines.append(
            f"{d:10.4f} {vals['GR']:8.2f} {vals['CALI']:7.2f} {vals['SP']:8.2f}"
            f" {vals['ILD']:9.3f} {nphi} {vals['RHOB']:8.4f} {vals['DT']:8.2f} {pef}\n"
        )
    return "".join(lines)


def make_bad_las(well: str, mode: str) -> str:
    """Deliberately malformed LAS exemplars for the manual test plan's failure-path
    tests (T-IMP-03 / T-IMP-04): `dup` repeats a block of depths (imports with a
    dropped-rows warning); `null` has an all-NULL depth column (clean error, no well)."""
    rng = Lcg(9001)
    top, n = 1500.0, 40
    hdr = f"""~Version Information
 VERS.                 2.0 : CWLS Log ASCII Standard - Version 2.0
 WRAP.                  NO : One line per depth step
~Well Information
 STRT.M           {top:.4f}             : Start depth
 STOP.M           {top + (n - 1) * STEP:.4f}             : Stop depth
 STEP.M           {STEP:.4f}                : Step
 NULL.            {NULL}               : Null value
 WELL.            {well}              : Well name
 LOC .            Deliberately malformed exemplar : Location
~Curve Information
 DEPT.M                                 : Measured depth
 GR  .GAPI                              : Gamma ray
 RHOB.G/C3                              : Bulk density
~ASCII
"""
    lines = [hdr]
    for i in range(n):
        if mode == "null":
            d = NULL  # every depth is the null sentinel -> no importable rows
        elif mode == "dup" and 10 <= i < 15:
            d = top + 9 * STEP  # rows 10..14 repeat row 9's depth -> 5 dropped, rest import
        else:
            d = top + i * STEP
        gr = 95.0 + 6.0 * rng.gauss()
        rhob = 2.42 + 0.02 * rng.gauss()
        lines.append(f"{d:10.4f} {gr:8.2f} {rhob:8.4f}\n")
    return "".join(lines)


def make_core_csv() -> str:
    """RCAL plugs for SANDI-01, off the 0.1524 grid on purpose (core is stored at native
    depths). Porosity/Sw in PERCENT — the importer's percent→fraction heuristic converts."""
    rng = Lcg(7001)
    rows = ["DEPTH,CPOR,CPERM,CGD,CSW\n"]
    d = TOPS["TOP_SAND_A"] + 0.55
    while d < TOPS["TOP_SHALE_2"] - 1.0:
        rhob = zone_blend(d, 0.0, ZONES["RHOB"])
        poro = (2.65 - rhob) / 1.65 + 0.02 * rng.gauss()          # density-consistent
        perm = 10 ** (18.0 * poro - 2.2 + 0.25 * rng.gauss())      # poro-perm transform
        gd = 2.65 + 0.012 * rng.gauss()
        in_gas = d < TOPS["TOP_SAND_B"]
        sw = (32.0 if in_gas else 84.0) + 4.0 * rng.gauss()
        rows.append(f"{d:.2f},{poro*100:.1f},{perm:.1f},{gd:.3f},{sw:.1f}\n")
        d += 1.7 + 0.6 * rng.next()  # irregular plug spacing, like a real core
    return "".join(rows)


def make_core_multiwell_csv() -> str:
    """ONE core file for the whole field, in the BLSO/PHR delivery shape: WN well-name
    column, a units row under the headers, suffixed mnemonics, porosity/Sw in percent.
    The import wizard (T-IMP-07) detects all of it and routes rows per well — no well
    selection needed.

    It is deliberately WIDER than core_data's four measurements: SO_1 (numeric oil
    saturation), LITH (free text) and SAMPLE_ID (mixed alphanumeric) exercise the wizard's
    extra-column path, which stores them as point data typed per cell."""
    rng = Lcg(7005)
    rows = ["TAPE_NAME,TOOL_STRING,WN,DEPTH,CPERM_1,CPOR_2,CSW_1,GDEN_1,SO_1,LITH,SAMPLE_ID\n",
            '"","","",M,MD,V/V,V/V,G/C3,V/V,,\n']
    for well, shift in WELLS.items():
        d = TOPS["TOP_SAND_A"] + shift + 0.55
        plug = 1
        while d < TOPS["TOP_SHALE_2"] + shift - 4.0:
            rhob = zone_blend(d, shift, ZONES["RHOB"])
            poro = (2.65 - rhob) / 1.65 + 0.02 * rng.gauss()
            perm = 10 ** (18.0 * poro - 2.2 + 0.25 * rng.gauss())
            gd = 2.65 + 0.012 * rng.gauss()
            in_gas = d < TOPS["TOP_SAND_B"] + shift
            sw = (32.0 if in_gas else 84.0) + 4.0 * rng.gauss()
            so = max(0.0, 100.0 - sw - (18.0 if in_gas else 2.0))
            lith = "SANDSTONE" if poro > 0.20 else ("SHALY SAND" if poro > 0.13 else "SILTY SHALE")
            rows.append(
                f'"","",{well},{d:.2f},{perm:.1f},{poro*100:.1f},{sw:.1f},{gd:.3f},'
                f'{so:.1f},{lith},{well[-2:]}-P{plug:03d}\n'
            )
            plug += 1
            d += 2.4 + 0.8 * rng.next()
    return "".join(rows)


def make_xrd_multiwell_txt() -> str:
    """Tab-delimited TXT with a WELL column: exercises both the delimiter sniffing and
    the aux importer's per-well routing (T-IMP-11) in one exemplar."""
    lines = ["WELL\tDEPTH\tQUARTZ\tCALCITE\tILLITE\tKAOLINITE\n"]
    data = [
        ("SANDI-01", 1521.1, 72.5, 2.8, 6.8, 8.1),
        ("SANDI-01", 1537.8, 61.2, 3.1, 12.4, 11.5),
        ("SANDI-02", 1531.4, 70.9, 2.5, 7.4, 8.8),
        ("SANDI-02", 1547.2, 59.8, 3.4, 13.1, 12.0),
        ("SANDI-03", 1516.6, 71.7, 2.9, 7.0, 8.4),
    ]
    for well, d, q, c, i, k in data:
        lines.append(f"{well}\t{d}\t{q}\t{c}\t{i}\t{k}\n")
    return "".join(lines)


def make_scal_long() -> str:
    """Flat lab export: plug context only on each plug's FIRST row (merged-cell style —
    the parser forward-fills). Sw in %PV."""
    rng = Lcg(7002)
    rows = ["SAMPLE,DEPTH,PERM,PORO,PC,SW\n"]
    plugs = [(1, 1522.35, 325.0, 26.1), (2, 1528.80, 88.0, 22.4), (3, 1539.15, 12.5, 18.9)]
    pcs = [1, 2, 4, 8, 15, 35, 75, 150]
    for no, depth, perm, poro in plugs:
        swirr = min(92.0, 18.0 + 900.0 / perm)  # tighter plug -> higher Swirr (capped < 100 %PV)
        for j, pc in enumerate(pcs):
            sw = swirr + (100.0 - swirr) / (1.0 + (pc / 3.0) ** 0.9) + 0.8 * rng.gauss()
            sw = max(swirr, min(100.0, sw))
            ctx = f"{no},{depth:.2f},{perm:.1f},{poro:.1f}" if j == 0 else ",,,"
            rows.append(f"{ctx},{pc},{sw:.1f}\n")
    return "".join(rows)


def make_scal_wide() -> str:
    """Corelab-style porous-plate report: free-form preamble, then a header row whose
    pressure columns ARE the psi values, one row per plug, cells = brine sat %PV."""
    rng = Lcg(7003)
    pcs = [1, 2, 4, 8, 15, 35, 75, 150]
    out = [
        "SANDIBUMI EXAMPLE LABORATORY,,,,,,,,,,,\n",
        "POROUS PLATE CAPILLARY PRESSURE,,,,,,,,,,,\n",
        "Well: SANDI-01   Overburden stress: 2000 psi,,,,,,,,,,,\n",
        ",,,,,,,,,,,\n",
        "SAMPLE,DEPTH,PERM,PORO," + ",".join(str(p) for p in pcs) + "\n",
    ]
    plugs = [(1, 1522.35, 325.0, 26.1), (2, 1528.80, 88.0, 22.4), (3, 1539.15, 12.5, 18.9)]
    for no, depth, perm, poro in plugs:
        swirr = min(92.0, 18.0 + 900.0 / perm)
        sats = []
        for pc in pcs:
            sw = swirr + (100.0 - swirr) / (1.0 + (pc / 3.0) ** 0.9) + 0.8 * rng.gauss()
            sats.append(f"{max(swirr, min(100.0, sw)):.1f}")
        out.append(f"{no},{depth:.2f},{perm:.1f},{poro:.1f}," + ",".join(sats) + "\n")
    return "".join(out)


def make_scal_centrifuge() -> str:
    """Per-plug key-value blocks + a SPEED/PC/SW table each — the digitized-workbook
    shape. The table header appears only above the FIRST block on purpose: the parser
    must carry it over (a real hand-merged file often drops the repeats)."""
    rng = Lcg(7004)
    out = []
    plugs = [(4, 1524.10, 210.0, 25.0), (5, 1541.60, 6.8, 17.5)]
    first = True
    for no, depth, perm, poro in plugs:
        out.append(f"SAMPLE,{no}\n")
        out.append(f"DEPTH,{depth:.2f}\n")
        out.append(f"PERM,{perm:.1f}\n")
        out.append(f"PORO,{poro:.1f}\n")
        if first:
            out.append("SPEED,PC,SW\n")
            first = False
        swirr = min(92.0, 20.0 + 700.0 / perm)
        for rpm, pc in [(500, 2.1), (900, 6.8), (1400, 16.4), (2100, 37.0), (3200, 86.0)]:
            sw = swirr + (100.0 - swirr) / (1.0 + (pc / 3.0) ** 0.9) + 0.8 * rng.gauss()
            out.append(f"{rpm},{pc},{max(swirr, min(100.0, sw)):.1f}\n")
        out.append(",,\n")
    return "".join(out)


def make_tops() -> str:
    rows = ["WELL,TOP,MD\n"]
    for well, shift in WELLS.items():
        for top, d in TOPS.items():
            rows.append(f"{well},{top},{d + shift:.1f}\n")
    return "".join(rows)


def make_deviation() -> str:
    """SANDI-02: vertical to 300 m, build to 25 deg by 800 m, hold. AZI 135."""
    rows = ["MD,INC,AZI\n"]
    md = 0.0
    while md <= 1650.0:
        if md <= 300.0:
            inc = 0.0
        elif md <= 800.0:
            inc = 25.0 * smooth((md - 300.0) / 500.0)
        else:
            inc = 25.0
        rows.append(f"{md:.1f},{inc:.2f},{135.0 if inc > 0 else 0.0:.1f}\n")
        md += 50.0
    return "".join(rows)


def make_locations() -> str:
    return (
        "WELL,EASTING,NORTHING,UTM_ZONE\n"
        "SANDI-01,506250.0,9935420.0,UTM 50S\n"
        "SANDI-02,507890.0,9936115.0,UTM 50S\n"
        "SANDI-03,505130.0,9934310.0,UTM 50S\n"
    )


def make_petrography() -> str:
    return (
        "TOP,BASE,LITHOLOGY,GRAIN_SIZE,SORTING,CEMENT,VISIBLE_POROSITY\n"
        "1520.5,1523.0,Sandstone,Fine,Well,Quartz overgrowth,Intergranular good\n"
        "1523.0,1527.5,Sandstone,Fine to medium,Well,Minor calcite,Intergranular good\n"
        "1527.5,1531.0,Sandstone,Fine,Moderate,Calcite patchy,Intergranular reduced\n"
        "1535.2,1539.0,Sandstone,Very fine to fine,Moderate,Quartz + clay rims,Intergranular fair\n"
        "1539.0,1544.5,Sandstone,Very fine,Poor,Clay matrix,Micro-porosity dominant\n"
        "1550.3,1552.0,Claystone,,,,None visible\n"
    )


def make_xrd() -> str:
    return (
        "DEPTH,QUARTZ,K_FELDSPAR,PLAGIOCLASE,CALCITE,DOLOMITE,SIDERITE,ILLITE,KAOLINITE,CHLORITE\n"
        "1521.1,72.5,4.2,3.1,2.8,0.5,0.4,6.8,8.1,1.6\n"
        "1526.4,68.9,5.0,3.5,4.6,0.8,0.3,7.2,8.0,1.7\n"
        "1537.8,61.2,4.8,3.0,3.1,0.6,1.2,12.4,11.5,2.2\n"
        "1542.3,55.4,4.1,2.7,2.5,0.4,1.8,16.9,13.6,2.6\n"
        "1551.0,38.2,2.9,2.1,1.9,0.3,2.4,28.7,19.2,4.3\n"
    )


def make_perforations() -> str:
    return (
        "TOP,BASE,STATUS,SHOT_DENSITY\n"
        "1521.0,1526.5,Open,12 spf\n"
        "1528.0,1531.5,Open,12 spf\n"
        "1536.0,1538.0,Squeezed,8 spf\n"
    )


def make_bad_las(well: str, kind: str) -> str:
    """The two DELIBERATELY BROKEN exemplars behind manual test plan T-IMP-03/-04, so
    Jauhar never has to doctor a file by hand to exercise a failure path.

    `kind="dup"`  — 40 rows, of which rows 10-14 repeat row 9's depth (the shape a bad
                    tape splice produces). Must IMPORT: 35 rows kept (first occurrence
                    of each depth wins) plus a dropped-rows warning.
    `kind="null"` — every depth cell is the NULL sentinel. Must FAIL CLEANLY with
                    "no importable rows" and commit no orphan well row.

    Deliberately short (40 rows) so the expected counts in the README and in
    `example_data_test.rs` can be stated exactly rather than approximately.
    """
    n = 40
    top = 1500.0
    rng = Lcg(4000 + len(kind))
    hdr = f"""~Version Information
 VERS.                 2.0 : CWLS Log ASCII Standard - Version 2.0
 WRAP.                  NO : One line per depth step
~Well Information
 STRT.M           {top:.4f}             : Start depth
 STOP.M           {top + (n - 1) * STEP:.4f}             : Stop depth
 STEP.M           {STEP:.4f}                : Step
 NULL.            {NULL}               : Null value
 WELL.            {well}          : Well name
 FLD .            SANDI                 : Field
~Curve Information
 DEPT.M                                 : Measured depth
 GR  .GAPI                              : Gamma ray
 RHOB.G/C3                              : Bulk density
~ASCII
"""
    lines = [hdr]
    for i in range(n):
        if kind == "null":
            depth = NULL
        elif 9 <= i <= 13:
            depth = top + 8 * STEP  # rows 10-14 (1-indexed) repeat row 9's depth
        else:
            depth = top + i * STEP
        gr = curve_value("GR", top + i * STEP, 0.0, rng)
        rhob = curve_value("RHOB", top + i * STEP, 0.0, rng)
        lines.append(f"{depth:10.4f} {gr:8.2f} {rhob:8.4f}\n")
    return "".join(lines)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    files = {}
    for i, (well, shift) in enumerate(WELLS.items()):
        files[f"{well}.las"] = make_las(well, shift, seed=1000 + i)
    files["bad_dup_depth.las"] = make_bad_las("SANDI-BAD-DUP", "dup")
    files["bad_null_depth.las"] = make_bad_las("SANDI-BAD-NULL", "null")
    files["core_rcal_SANDI-01.csv"] = make_core_csv()
    files["core_rcal_multiwell.csv"] = make_core_multiwell_csv()
    files["xrd_multiwell.txt"] = make_xrd_multiwell_txt()
    files["scal_pc_long_SANDI-01.csv"] = make_scal_long()
    files["scal_porous_plate_wide_SANDI-01.csv"] = make_scal_wide()
    files["scal_centrifuge_SANDI-01.csv"] = make_scal_centrifuge()
    files["tops_multiwell.csv"] = make_tops()
    files["deviation_SANDI-02.csv"] = make_deviation()
    files["well_locations.csv"] = make_locations()
    files["petrography_SANDI-01.csv"] = make_petrography()
    files["xrd_SANDI-01.csv"] = make_xrd()
    files["perforations_SANDI-01.csv"] = make_perforations()
    for name, body in files.items():
        (OUT / name).write_text(body, encoding="ascii", newline="\n")
        print(f"wrote {name}  ({len(body.splitlines())} lines)")


if __name__ == "__main__":
    main()
