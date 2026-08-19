// SB-PLT-024 route 2 (DEC-078, 2026-08-19): re-derive chart-overlay curves from PUBLISHED
// equations and the chart pages' own STATED parameters, replacing the digitized coordinates
// as each derivation is executed and verified. This generator is the single source of truth
// for every derived curve; `src/ui/chartOverlaysDerived.gen.ts` is its output and is
// freshness-checked by the green gate (same pattern as gen-third-party-licenses.mjs).
//
// Executed so far: `por22_ta` (Por-22 density-sonic, time-average family) and
// `lith6_mid` (Lith-6 MID carbonate ternary: vertices, percent grid and the anhydrite
// point — see the MID block below for what deliberately stays digitized).
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
// NOT yet derived, recorded honestly: `por22_fo` (field observation) — a first-pass check
// against the Raymer-Hunt-Gardner closed forms left systematic residuals beyond
// digitization tolerance, so the exact published formulation the chart used must be
// established before replacement; if none reproduces it, FO is adjudicated
// vendor-empirical and joins the Gate 5 class (src/ui/chartOverlayPolicy.ts). The
// def's mineral POINTS also remain digitized pending their own constant-based derivation.

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

const round = (value, dp) => Number(value.toFixed(dp));

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

export function derivedOverlays() {
  return [{ id: "por22_ta", curves: por22TaCurves() }, lith6MidDerived()];
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
    if (actual !== expected) {
      console.error("chartOverlaysDerived.gen.ts is stale - run: node tools/gen-derived-overlays.mjs");
      process.exit(1);
    }
    console.log("chartOverlaysDerived.gen.ts is current");
  } else {
    fs.writeFileSync(OUT_PATH, expected);
    console.log(`wrote ${OUT_PATH}`);
  }
}
