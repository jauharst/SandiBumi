// SB-PLT-024 route 2 (DEC-078, 2026-08-19): re-derive chart-overlay curves from PUBLISHED
// equations and the chart pages' own STATED parameters, replacing the digitized coordinates
// as each derivation is executed and verified. This generator is the single source of truth
// for every derived curve; `src/ui/chartOverlaysDerived.gen.ts` is its output and is
// freshness-checked by the green gate (same pattern as gen-third-party-licenses.mjs).
//
// Executed so far: `por22_ta` (Por-22 density-sonic, time-average family), `lith6_mid`
// (Lith-6 MID carbonate ternary: vertices, percent grid and the anhydrite point — see
// the MID block below for what deliberately stays digitized), `por22_fo`, `lith3`,
// `lith4` and `lith2_thk` (each documented in its own note below).
//
// Constants, each cited:
//  - tf = 189 us/ft and rho_f = 1.0 g/cm3 — STATED in Por-22's chart header
//    (Schlumberger Log Interpretation Charts, 2013 ed., p. 238).
//  - Time-average equation dt = phi*tf + (1-phi)*dtma — Wyllie, Gregory & Gardner,
//    "Elastic wave velocities in heterogeneous and porous media", Geophysics 21(1), 1956;
//    the chart legend itself names the family "Time average".
//  - Matrix slownesses = 1e6/vma at the vma values chart Por-1 (p. 212) prints on its
//    matrix-velocity fan: 18,000 ft/s (quartz sandstone), 21,000 ft/s (calcite/limestone),
//    23,000 ft/s (dolomite).
//  - Matrix densities 2.65 / 2.71 / 2.87 g/cm3 with the density graduation
//    rho_b = rho_ma - phi*(rho_ma - rho_f): the classical matrix densities Por-22's own
//    printed graduations use — every digitized graduation satisfies this arithmetic at the
//    page's stated rho_f exactly.
//
// tools/chart-derivation.test.mjs proves the derived curves reproduce the digitized chart
// within digitization tolerance (<= 0.4 us/ft, <= 0.005 g/cm3) and that the generated
// module on disk is current.
//
// `por22_fo` (field observation) — DERIVED under DEC-079 (2026-08-19) as the RHG 1980
// ALGORITHM, not as a replica of the printed curve. The chart's red set traces the
// paper's hand-drawn empirical transform (its Fig. 9; the chartbook's Por-1 page cites
// it as "Reference 20"), which the paper's own segmented algorithms only "reasonably
// duplicate" — good to ~1 us/ft below 30 p.u., diverging to ~13 us/ft at 40 p.u.
// Jauhar's ruling: ship the published algorithm with its stated constants (it is what
// IP / Geolog / Techlog compute), and accept the visible high-porosity departure from
// the printed chart as the paper's own algorithm-vs-curve gap. Sources, all stated in
// Raymer, Hunt & Gardner, "An improved sonic transit time-to-porosity transform",
// SPWLA 21st Annual Logging Symposium, 1980 (paper P; copy in Jauhar's library):
//  - segments: phi < 37%: V = (1-phi)^2*Vma + phi*Vf ("can be used regardless of the
//    nature of the saturating fluid"; also the form the reference suite and IP implement);
//    phi > 47%: the fluid-suspension form 1/(rho*V^2) = phi/(rho_f*Vf^2) +
//    (1-phi)/(rho_ma*Vma^2); 37..47%: linear interpolation in dt between the two
//    branch values at 37% and 47% (the endpoint reading the reference suite's
//    dt_from_models source pins).
//  - matrix velocities (paper, Summary item 3 — deliberately NOT the TA leg's Por-1
//    fan values): sandstone 17,850 ft/s (56 us/ft), limestone 20,500 ft/s (49 us/ft),
//    dolomite 22,750 ft/s (44 us/ft).
//  - Vf = 5,300 ft/s: stated in the paper's figures AND on the chartbook Por-1 page.
//  - rho_f 1.0 and the matrix densities: the Por-22 density graduation (as for TA).
// The def's mineral POINTS remain digitized pending their own constant-based derivation.
//
// `lith2_thk` (executed increment 6) — the six Th/K boundary lines are GEOMETRY from the
// chart page's own printed values: Lith-2 (p. 194, former CP-19) labels every boundary
// line with its ratio ("Th/K = 25 / 12 / 3.5 / 2.0 / 0.6 / 0.3"), and a constant-ratio
// line is the segment from the origin to where that ratio exits the printed frame
// (K 0..5 %, Th 0..25 ppm). The classification itself is published: Quirein, Gardner &
// Watson, SPE 11143 (1982), Fig. 2 prints the same spectral classification (the 12 and
// 3.5 boundaries verbatim), building on Hassan, Hossin & Combaz, SPWLA 17th (1976),
// Fig. 7 / Hassan & Hossin 1975 (both papers in Jauhar's library). NOT derived, kept
// digitized: the dashed clay and feldspar lines and the region label points - their
// positions are measured-variability graphics with no printed numeric source in either
// paper (the numeric per-mineral table is Edmundson & Raymer 1979, not yet obtained).

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const OUT_PATH = path.join(repo, "src", "ui", "chartOverlaysDerived.gen.ts");

export const CONSTANTS = {
  TF_US_FT: 189.0, // stated, Por-22 p. 238
  RHO_F: 1.0, // stated, Por-22 p. 238
  LITHS: [
    // vMa from Por-1 p. 212's printed fan; rhoMa per the graduation arithmetic note above.
    // maxPhi mirrors the printed chart's own graduation extent per lithology.
    { name: "Quartz sandstone", rhoMa: 2.65, vMaFtS: 18000, maxPhi: 45 },
    { name: "Calcite (limestone)", rhoMa: 2.71, vMaFtS: 21000, maxPhi: 45 },
    { name: "Dolomite", rhoMa: 2.87, vMaFtS: 23000, maxPhi: 45 },
  ],
};

// RHG 1980 constants (DEC-079) — see the header note for the citations. maxPhi mirrors
// the printed FO curves' own graduation extent per lithology (sandstone stops at 35).
export const RHG = {
  VF_FT_S: 5300, // paper figures + Por-1 p. 211: "water with a velocity of 5,300 ft/s"
  LITHS: [
    { name: "Quartz sandstone", rhoMa: 2.65, vMaFtS: 17850, maxPhi: 35 },
    { name: "Calcite (limestone)", rhoMa: 2.71, vMaFtS: 20500, maxPhi: 40 },
    { name: "Dolomite", rhoMa: 2.87, vMaFtS: 22750, maxPhi: 40 },
  ],
  LOW_END: 0.37, // the paper's segment boundaries
  HIGH_START: 0.47,
};

const round = (value, dp) => Number(value.toFixed(dp));

// --- Lith-3 / Lith-4 Platform Express TLD PEF charts (executed increment 4, DEC-079) --
// The Pe legs are DERIVED from the published litho-density physics of Gardner &
// Dumanoir, "Litho-Density log interpretation", SPWLA 21st Annual Logging Symposium,
// 1980 (paper N; copy in Jauhar's library), which states the full convention:
//   Pe = (Z/10)^3.6 (eq 3), rho_b = 1.0704*rho_e - 0.1883 (eq 5, so
//   rho_e = (rho_b + 0.1883)/1.0704), U = Pe*rho_e (eq 6), and U and rho_b both mix
//   VOLUMETRICALLY (eqs 7-8). Hence for porosity f:
//   Pe(f) = [f*Uf + (1-f)*Uma] / [f*rho_e_fluid + (1-f)*rho_e_ma].
// Constants, cited:
//  - mineral U and rho_b(log): the paper's printed table (quartz 4.78/2.64,
//    calcite 13.8/2.71, dolomite 9.00/2.88), sourced there to Edmundson et al. 1979.
//  - fluids: the chartbook Lith-5 legend states both pairs verbatim - "Fresh water
//    (0 ppm), rho_f = 1.0 g/cm3, Uf = 0.398" and "Salt water (200,000 ppm),
//    rho_f = 1.11 g/cm3, Uf = 1.36" (p. 198). The DENSITY graduation uses the
//    Lith-3/Lith-4 chart headers' own rho_f (1.0 fresh / 1.1 salt) with the classical
//    matrix densities - proven exact against every digitized graduation.
// Fidelity, ruled by DEC-079: the printed charts are TLD TOOL charts and carry a small
// systematic tool-window slope the pure physics does not (residuals <= 0.07 Pe, ~1% of
// the axis, against digitization jitter of ~0.02); Jauhar ruled the physics-derived
// legs replace the digitized ones regardless.
export const GD = {
  RHOE_SCALE: 1.0704, // eq (5)
  RHOE_OFFSET: 0.1883, // eq (5)
  MATS: [
    { name: "Quartz sandstone", U: 4.78, rbLog: 2.64, rhoMaGrad: 2.65 },
    { name: "Calcite (limestone)", U: 13.8, rbLog: 2.71, rhoMaGrad: 2.71 },
    { name: "Dolomite", U: 9.0, rbLog: 2.88, rhoMaGrad: 2.87 },
  ],
  FLUIDS: {
    fresh: { Uf: 0.398, rbLog: 1.0, rhoFGrad: 1.0 },
    salt: { Uf: 1.36, rbLog: 1.11, rhoFGrad: 1.1 },
  },
  MAX_PHI: 45, // the printed charts graduate 0..45 p.u. every 1 p.u.
};

export function gdRhoE(rb) {
  return (rb + GD.RHOE_OFFSET) / GD.RHOE_SCALE;
}

export function lithPefCurves(fluidKey) {
  const fl = GD.FLUIDS[fluidKey];
  return GD.MATS.map((mat) => {
    const grads = [];
    for (let phi = 0; phi <= GD.MAX_PHI; phi += 1) {
      const f = phi / 100;
      const u = f * fl.Uf + (1 - f) * mat.U;
      const rhoE = f * gdRhoE(fl.rbLog) + (1 - f) * gdRhoE(mat.rbLog);
      const rho = mat.rhoMaGrad - f * (mat.rhoMaGrad - fl.rhoFGrad);
      grads.push([phi, round(u / rhoE, 4), round(rho, 4)]);
    }
    return { name: mat.name, labelEvery: 10, grads };
  });
}

// --- Lith-6 Umaa-rhomaa MID plot (executed increment 2) ------------------------------
// The carbonate ternary is GEOMETRY once its three vertices are fixed: the labeled
// percent edges connect the vertices, and the twelve unlabeled interior lines are the
// constant-fraction grid at 20/40/60/80% (verified: every digitized interior line's
// endpoints sit on the vertex-to-vertex parameterization at those exact fractions).
// Constants, cited:
//  - Umaa values 4.8 (quartz), 13.8 (calcite), 9.0 (dolomite), 15 (anhydrite) and the
//    calcite/anhydrite densities 2.71 / 2.98 g/cm3: printed in Appendix B pp. 279-280
//    (U and rho_log columns).
//  - Quartz 2.65 and dolomite 2.87 g/cm3: the classical matrix densities the chart's
//    plotted positions use - the same values Por-22's own graduations use - each verified
//    against the digitized point within tolerance by the test.
// NOT derived, kept digitized: the K-feldspar, kaolinite and illite points (they plot at
// APPARENT positions through the chart's porosity transform, not at raw constants) and
// the annotation arrows (Heavy minerals / Gas direction / Salt / Barite).
export const MID = {
  QUARTZ: { u: 4.8, rho: 2.65 },
  CALCITE: { u: 13.8, rho: 2.71 },
  DOLOMITE: { u: 9.0, rho: 2.87 },
  ANHYDRITE: { u: 15.0, rho: 2.98 },
  GRID_LEVELS: [0.2, 0.4, 0.6, 0.8],
};

export function lith6MidDerived() {
  const { QUARTZ: q, CALCITE: c, DOLOMITE: d } = MID;
  const at = (from, to, t) => [
    round(from.u + t * (to.u - from.u), 4),
    round(from.rho + t * (to.rho - from.rho), 4),
  ];
  const points = [
    { x: MID.QUARTZ.u, y: MID.QUARTZ.rho, label: "Quartz" },
    { x: MID.CALCITE.u, y: MID.CALCITE.rho, label: "Calcite" },
    { x: MID.DOLOMITE.u, y: MID.DOLOMITE.rho, label: "Dolomite" },
    { x: MID.ANHYDRITE.u, y: MID.ANHYDRITE.rho, label: "Anhydrite" },
  ];
  const lines = [
    { pts: [at(q, c, 0), at(q, c, 1)], label: "% calcite" },
    { pts: [at(d, q, 0), at(d, q, 1)], label: "% quartz" },
    { pts: [at(c, d, 0), at(c, d, 1)], label: "% dolomite" },
  ];
  // Constant-fraction interior grid: for vertex V at content level c, the line connects
  // V + (1-c)(A-V) and V + (1-c)(B-V), where A and B are the other two vertices.
  for (const [v, a, b] of [
    [q, c, d],
    [c, q, d],
    [d, q, c],
  ]) {
    for (const level of MID.GRID_LEVELS) {
      lines.push({ pts: [at(v, a, 1 - level), at(v, b, 1 - level)] });
    }
  }
  return {
    id: "lith6_mid",
    points,
    lines,
    keepDigitizedLineLabels: ["Heavy minerals", "Gas direction", "Salt", "Barite"],
  };
}

// --- Lith-2 NGS Th-K clay-mineral identification (executed increment 6) --------------
// The printed boundary values and the chart frame, all from Lith-2 p. 194 (see the
// header note). Labels are kept verbatim as printed - "2.0" prints with its zero.
export const LITH2 = {
  RATIOS: [
    { r: 25, label: "Th/K = 25" },
    { r: 12, label: "Th/K = 12" },
    { r: 3.5, label: "Th/K = 3.5" },
    { r: 2.0, label: "Th/K = 2.0" },
    { r: 0.6, label: "Th/K = 0.6" },
    { r: 0.3, label: "Th/K = 0.3" },
  ],
  K_MAX: 5, // the printed frame: K 0..5 %
  TH_MAX: 25, // Th 0..25 ppm
};

/** Constant-ratio boundary lines: origin to where the ratio exits the printed frame. */
export function lith2ThkLines() {
  return LITH2.RATIOS.map(({ r, label }) => {
    const exitsTop = LITH2.K_MAX * r >= LITH2.TH_MAX;
    const end = exitsTop
      ? [round(LITH2.TH_MAX / r, 4), LITH2.TH_MAX]
      : [LITH2.K_MAX, round(LITH2.K_MAX * r, 4)];
    return { pts: [[0, 0], end], label };
  });
}

/** Wyllie time-average curves for Por-22, graduated every 5 p.u. like the printed chart. */
export function por22TaCurves() {
  return CONSTANTS.LITHS.map((lith) => {
    const dtMa = 1e6 / lith.vMaFtS;
    const grads = [];
    for (let phi = 0; phi <= lith.maxPhi; phi += 5) {
      const f = phi / 100;
      const dt = f * CONSTANTS.TF_US_FT + (1 - f) * dtMa;
      const rho = lith.rhoMa - f * (lith.rhoMa - CONSTANTS.RHO_F);
      grads.push([phi, round(dt, 4), round(rho, 4)]);
    }
    return { name: lith.name, labelEvery: 10, grads };
  });
}

/** RHG 1980 segmented transform: dt (us/ft) at fractional porosity f. */
export function rhgDt(lith, f) {
  const dtf = 1e6 / RHG.VF_FT_S;
  const dtLow = (p) => 1e6 / ((1 - p) ** 2 * lith.vMaFtS + p * RHG.VF_FT_S);
  const dtSusp = (p) => {
    const rho = lith.rhoMa - p * (lith.rhoMa - CONSTANTS.RHO_F);
    const dtMa = 1e6 / lith.vMaFtS;
    return Math.sqrt(
      (rho * p * dtf * dtf) / CONSTANTS.RHO_F + (rho * (1 - p) * dtMa * dtMa) / lith.rhoMa,
    );
  };
  if (f <= RHG.LOW_END) return dtLow(f);
  if (f >= RHG.HIGH_START) return dtSusp(f);
  // 37..47%: linear interpolation in dt between the branch values at the segment ends
  // (the endpoint reading — the reference suite's dt_from_models source pins it).
  const a = (RHG.HIGH_START - f) / (RHG.HIGH_START - RHG.LOW_END);
  return a * dtLow(RHG.LOW_END) + (1 - a) * dtSusp(RHG.HIGH_START);
}

/** RHG field-observation curves for Por-22, graduated every 5 p.u. like the printed chart. */
export function por22FoCurves() {
  return RHG.LITHS.map((lith) => {
    const grads = [];
    for (let phi = 0; phi <= lith.maxPhi; phi += 5) {
      const f = phi / 100;
      const rho = lith.rhoMa - f * (lith.rhoMa - CONSTANTS.RHO_F);
      grads.push([phi, round(rhgDt(lith, f), 4), round(rho, 4)]);
    }
    return { name: lith.name, labelEvery: 10, grads };
  });
}

export function derivedOverlays() {
  return [
    { id: "por22_ta", curves: por22TaCurves() },
    { id: "por22_fo", curves: por22FoCurves() },
    { id: "lith3", curves: lithPefCurves("fresh") },
    { id: "lith4", curves: lithPefCurves("salt") },
    lith6MidDerived(),
    {
      id: "lith2_thk",
      lines: lith2ThkLines(),
      // The dashed apparent-position lines stay digitized (no printed numeric source).
      keepDigitizedLineLabels: ["Clay line", "Feldspar line"],
    },
  ];
}

export function renderModule() {
  const lines = [];
  lines.push("// GENERATED by tools/gen-derived-overlays.mjs - DO NOT HAND-EDIT; re-run the generator.");
  lines.push("// SB-PLT-024 route 2 (DEC-078): equation-derived overlay data replacing digitized");
  lines.push("// coordinates. The equations, cited constants and the digitized-agreement proof live");
  lines.push("// in the generator and tools/chart-derivation.test.mjs.");
  lines.push('import type { OverlayCurve, OverlayLine, OverlayPoint } from "./chartOverlays";');
  lines.push("");
  lines.push("export interface DerivedOverlay {");
  lines.push("  id: string;");
  lines.push("  curves?: OverlayCurve[];");
  lines.push("  /** Replace same-label digitized points only; unmatched digitized points stay. */");
  lines.push("  points?: OverlayPoint[];");
  lines.push("  /** Replace the digitized lines wholesale, except labels listed below. */");
  lines.push("  lines?: OverlayLine[];");
  lines.push("  keepDigitizedLineLabels?: string[];");
  lines.push("}");
  lines.push("");
  lines.push("export const DERIVED_CHART_OVERLAYS: DerivedOverlay[] = [");
  for (const overlay of derivedOverlays()) {
    lines.push(`  {`);
    lines.push(`    id: ${JSON.stringify(overlay.id)},`);
    if (overlay.curves) {
      lines.push(`    curves: [`);
      for (const curve of overlay.curves) {
        const grads = curve.grads.map((g) => `[${g[0]},${g[1]},${g[2]}]`).join(",");
        lines.push(
          `      { name: ${JSON.stringify(curve.name)}, labelEvery: ${curve.labelEvery}, grads: [${grads}] },`,
        );
      }
      lines.push(`    ],`);
    }
    if (overlay.points) {
      const points = overlay.points
        .map((p) => `{ x: ${p.x}, y: ${p.y}, label: ${JSON.stringify(p.label)} }`)
        .join(", ");
      lines.push(`    points: [${points}],`);
    }
    if (overlay.lines) {
      lines.push(`    lines: [`);
      for (const line of overlay.lines) {
        const pts = `[[${line.pts[0][0]},${line.pts[0][1]}],[${line.pts[1][0]},${line.pts[1][1]}]]`;
        const label = line.label ? `, label: ${JSON.stringify(line.label)}` : "";
        lines.push(`      { pts: ${pts}${label} },`);
      }
      lines.push(`    ],`);
    }
    if (overlay.keepDigitizedLineLabels) {
      lines.push(
        `    keepDigitizedLineLabels: [${overlay.keepDigitizedLineLabels.map((l) => JSON.stringify(l)).join(", ")}],`,
      );
    }
    lines.push(`  },`);
  }
  lines.push("];");
  lines.push("");
  return lines.join("\n");
}

const invokedDirectly =
  process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (invokedDirectly) {
  const expected = renderModule();
  if (process.argv.includes("--check")) {
    const actual = fs.existsSync(OUT_PATH) ? fs.readFileSync(OUT_PATH, "utf8") : "";
    // Compare CONTENT, not line endings: a fresh checkout materializes the file with
    // CRLF under git autocrlf, and that must not read as staleness.
    const norm = (t) => t.replace(/\r\n/g, "\n");
    if (norm(actual) !== norm(expected)) {
      console.error("chartOverlaysDerived.gen.ts is stale - run: node tools/gen-derived-overlays.mjs");
      process.exit(1);
    }
    console.log("chartOverlaysDerived.gen.ts is current");
  } else {
    fs.writeFileSync(OUT_PATH, expected);
    console.log(`wrote ${OUT_PATH}`);
  }
}
