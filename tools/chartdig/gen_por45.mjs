// Por-4 (p228, APS epithermal) + Por-5 (p229, CNL thermal): neutron porosity
// equivalence — apparent limestone porosity (x, pu) -> true porosity for the
// indicated matrix (y, pu). Calcite (limestone) is the identity diagonal (black);
// quartz sandstone reads above it, dolomite below.
//   Por-5 curves: blue solid = NPHI (ratio method), red solid = TNPH 0 ppm,
//                 red dashed = TNPH 250,000 ppm  (SS + DOL each)
//   Por-4 curves: red solid = APLC, red dashed = FPLC, blue solid = SNP
// These are plain continuous curves (no graduation dashes — the axes carry the
// values), so we sample the drawn polyline directly, unlike the D-N family.
// Dashed-vs-solid is a setDash stroke property recorded by extract.mjs.
// Emits ../../src-tauri/src/neutron_charts.rs (v/v tables) + out_por45.json.
// node gen_por45.mjs
import { writeFileSync } from "fs";
import { loadPage, gridLines, gridFit, anchorAxis, polyLen, mergeChains } from "./lib.mjs";

const OUT_RS = "../../src-tauri/src/neutron_charts.rs";
const STEP = 0.5; // pu sampling step along x

function coloredPolysDash(strokes, color, dashed) {
  const out = [];
  for (const s of strokes) {
    if (s.color.map(c => Math.round(c)).join(",") !== color) continue;
    if (s.kind !== "stroke") continue;
    if (!!s.dashed !== dashed) continue;
    for (const p of s.polys) out.push(p);
  }
  return out;
}

/** Long open chains of one color+dash inside the frame, sorted by length desc. */
function chains(strokes, color, dashed, frame, minLen = 100) {
  const polys = coloredPolysDash(strokes, color, dashed).filter(p =>
    p.every(q => q[0] > frame.x0 - 40 && q[0] < frame.x1 + 10 && q[1] > frame.y0 - 10 && q[1] < frame.y1 + 10));
  return mergeChains(polys).filter(ch => polyLen(ch) > minLen).sort((a, b) => polyLen(b) - polyLen(a));
}

/** Interpolate chain device-y at device-x (median of all segment crossings). */
function chainYatX(ch, x) {
  const hits = [];
  for (let i = 1; i < ch.length; i++) {
    const [ax, ay] = ch[i - 1], [bx, by] = ch[i];
    if ((ax <= x && x <= bx) || (bx <= x && x <= ax)) {
      const t = bx === ax ? 0 : (x - ax) / (bx - ax);
      hits.push(ay + t * (by - ay));
    }
  }
  if (!hits.length) return null;
  hits.sort((a, b) => a - b);
  return hits[Math.floor(hits.length / 2)];
}

function digitizeChart(pageFile, chartName, families) {
  const { strokes, texts } = loadPage(pageFile);
  const { hs, vs } = gridLines(strokes);
  const gy = gridFit(hs), gx = gridFit(vs);
  // GATE grid fit: rms in data units. Por-4's x-grid has genuine artwork placement
  // wobble (~0.021 pu, benign — the curves follow the fitted uniform frame), so the
  // bound is 0.03 pu, not the 0.0004 pu the clean grids achieve.
  const gxPu = gx.rms / Math.abs(gx.slope), gyPu = gy.rms / Math.abs(gy.slope);
  console.log(`${chartName}: grid ${vs.length}v (${gx.slope.toFixed(3)}pt, rms ${gxPu.toFixed(4)} pu) x ${hs.length}h (${gy.slope.toFixed(3)}pt, rms ${gyPu.toFixed(4)} pu)`);
  if (gxPu > 0.03 || gyPu > 0.03) throw new Error(`${chartName}: grid-fit rms out of bounds (${gxPu.toFixed(4)}/${gyPu.toFixed(4)} pu, gate 0.03)`);

  // Anchor with the axis labels just outside the grid (the Purpose/Example prose
  // further down the page also contains bare numbers — keep it out of the vote).
  const xTexts = texts.filter(t => t.y < hs[0] - 3 && t.y > hs[0] - 30);
  const yTexts = texts.filter(t => t.x < vs[0] - 5 && t.x > vs[0] - 40);
  const xa = anchorAxis(xTexts, gx, { labelRe: /^-?(0|10|20|30|40)$/, step: 1, side: "x", edge: hs[0] });
  const ya = anchorAxis(yTexts, gy, { labelRe: /^(0|10|20|30|40)$/, step: 1, side: "y", edge: vs[0] });
  console.log(`  anchors: x base ${xa.base} votes ${JSON.stringify(xa.votes)} | y base ${ya.base} votes ${JSON.stringify(ya.votes)}`);
  if (xa.nLabels < 3 || ya.nLabels < 3) throw new Error("too few axis labels for anchoring");

  const toX = px => xa.base + (px - gx.inter) / gx.slope;
  const toY = py => ya.base + (py - gy.inter) / gy.slope;
  const devX = pu => gx.inter + (pu - xa.base) * gx.slope;
  const frame = { x0: vs[0], x1: vs[vs.length - 1], y0: hs[0], y1: hs[hs.length - 1] };

  // --- GATE identity: the black diagonal (calcite/limestone) must be y = x ---
  const blackDiag = chains(strokes, "44,46,53", false, frame, 100)
    .filter(ch => {
      const xs = ch.map(q => q[0]), ys = ch.map(q => q[1]);
      const open = Math.hypot(ch[0][0] - ch[ch.length - 1][0], ch[0][1] - ch[ch.length - 1][1]) > 10;
      return open && Math.max(...xs) - Math.min(...xs) > 100 && Math.max(...ys) - Math.min(...ys) > 100;
    });
  if (blackDiag.length !== 1) throw new Error(`${chartName}: expected 1 black identity diagonal, got ${blackDiag.length}`);
  // Bias and scatter gated separately: the residual is almost entirely a smooth
  // constant-sign frame offset (~0.13-0.15 pu) shared by every curve on the page —
  // it cancels in conversion deltas — while scatter is the true digitization noise.
  // A single raw-rms bound would sit at ~2% margin on Por-4 and flip on any rerun.
  let idN = 0, idSum = 0, idSum2 = 0;
  for (let pu = 2; pu <= 38; pu += 1) {
    const y = chainYatX(blackDiag[0], devX(pu));
    if (y === null) continue;
    const r = toY(y) - pu;
    idN++; idSum += r; idSum2 += r * r;
  }
  const idBias = idSum / idN;
  const idScatter = Math.sqrt(Math.max(0, idSum2 / idN - idBias * idBias));
  const idOk = Math.abs(idBias) < 0.25 && idScatter < 0.08;
  console.log(`  GATE identity: calcite diagonal bias ${idBias.toFixed(3)} pu (|.|<0.25), scatter ${idScatter.toFixed(3)} pu (<0.08) over ${idN} samples ${idOk ? "OK" : "FAIL"}`);
  if (!idOk) throw new Error("identity gate failed");

  // --- curve families ---
  const curves = [];
  for (const fam of families) {
    const chs = chains(strokes, fam.color, fam.dashed, frame, 100);
    if (chs.length !== 2) throw new Error(`${chartName} ${fam.tool}: expected 2 chains (SS+DOL), got ${chs.length}`);
    // sandstone reads ABOVE the identity at mid-chart, dolomite below
    const midX = devX(20);
    const pair = chs.map(ch => ({ ch, y20: toY(chainYatX(ch, midX)) })).sort((a, b) => b.y20 - a.y20);
    if (!(pair[0].y20 > 20 && pair[1].y20 < 20))
      throw new Error(`${chartName} ${fam.tool}: chains don't straddle the identity at x=20 (${pair.map(p => p.y20.toFixed(1)).join(", ")})`);
    for (const [mat, { ch }] of [["SS", pair[0]], ["DOL", pair[1]]]) {
      const dxs = ch.map(q => q[0]);
      const x0 = Math.ceil(toX(Math.min(...dxs)) / STEP) * STEP;
      const x1 = Math.floor(toX(Math.max(...dxs)) / STEP) * STEP;
      const pts = [];
      for (let pu = x0; pu <= x1 + 1e-9; pu += STEP) {
        const dy = chainYatX(ch, devX(pu));
        if (dy === null) continue;
        pts.push([Math.round(pu * 100) / 100, Math.round(toY(dy) * 100) / 100]);
      }
      // GATE monotone: strictly increasing on the ROUNDED values actually emitted —
      // chart_lerp in Rust inverts these tables and divides by y-differences, so an
      // equal pair would be a division by zero (the Rust test suite re-checks this).
      for (let i = 1; i < pts.length; i++)
        if (pts[i][1] <= pts[i - 1][1])
          throw new Error(`${chartName} ${fam.tool} ${mat}: non-monotone at x=${pts[i][0]} (${pts[i - 1][1]} -> ${pts[i][1]})`);
      // GATE side: SS above / DOL below identity across the mid-range. The DOL
      // bound is loose near zero: Por-4's epithermal dolomite curves hug the
      // identity at low porosity (matrix effect far smaller than thermal CNL).
      for (const [x, y] of pts.filter(p => p[0] >= 10 && p[0] <= 35)) {
        const d = y - x;
        const ok = mat === "SS" ? d > 0.8 && d < 9 : d < -0.15 && d > -12;
        if (!ok) throw new Error(`${chartName} ${fam.tool} ${mat}: offset from identity ${d.toFixed(2)} pu at x=${x} out of band`);
      }
      console.log(`  ${fam.tool} ${mat}: x ${pts[0][0]}..${pts[pts.length - 1][0]} pu, ${pts.length} pts, y(20)=${(pts.find(p => p[0] === 20) ?? ["-", NaN])[1]}`);
      curves.push({ tool: fam.tool, mat, pts });
    }
  }
  return { curves, devX, toY, chartName };
}

// ---- Por-5: CNL thermal (worked example on the page: TNPH 18 pu @ 250 kppm -> sandstone 24 pu) ----
const por5 = digitizeChart("p229.json", "Por-5", [
  { color: "3,70,145", dashed: false, tool: "NPHI" },
  { color: "255,64,58", dashed: false, tool: "TNPH_FRESH" },
  { color: "255,64,58", dashed: true, tool: "TNPH_SALT" },
]);
{
  const ssSalt = por5.curves.find(c => c.tool === "TNPH_SALT" && c.mat === "SS").pts;
  const p18 = ssSalt.find(p => p[0] === 18);
  const ok = p18 && Math.abs(p18[1] - 24) <= 0.6;
  console.log(`  GATE Por-5 worked example: TNPH salt SS at 18 pu -> ${p18?.[1]} pu (book: 24) ${ok ? "OK" : "FAIL"}`);
  if (!ok) throw new Error("Por-5 worked example gate failed");
  // info: salinity ordering for SS (salt curve above fresh at 18 pu)
  const ssFresh = por5.curves.find(c => c.tool === "TNPH_FRESH" && c.mat === "SS").pts.find(p => p[0] === 18);
  console.log(`  info: SS at 18 pu — fresh ${ssFresh?.[1]}, salt ${p18[1]}`);
}

// ---- Por-4: epithermal — APS APLC/FPLC + legacy sidewall SNP (no printed worked
// example, so the solid/dashed family identity gets its own physics gate below) ----
const por4 = digitizeChart("p228.json", "Por-4", [
  { color: "255,64,58", dashed: false, tool: "APLC" },
  { color: "255,64,58", dashed: true, tool: "FPLC" },
  { color: "3,70,145", dashed: false, tool: "SNP" },
]);
{
  // GATE Por-4 curve identity: the legend transcription (APLC = solid red,
  // FPLC = dashed red) is the only thing naming these chains, and every
  // structural gate would pass with the two swapped. Physics disambiguates:
  // the array measurement (APLC) hugs the calcite identity in dolomite while
  // the near-to-far ratio (FPLC) carries the big matrix effect.
  const dolOff = tool => {
    const p = por4.curves.find(c => c.tool === tool && c.mat === "DOL").pts.find(q => q[0] === 20);
    return p[1] - 20;
  };
  const aplc = dolOff("APLC"), fplc = dolOff("FPLC");
  const ok = Math.abs(fplc) > Math.abs(aplc) + 2;
  console.log(`  GATE Por-4 curve identity: dolomite offset at 20 pu — APLC ${aplc.toFixed(2)}, FPLC ${fplc.toFixed(2)} (|FPLC| must exceed |APLC| by 2 pu) ${ok ? "OK" : "FAIL"}`);
  if (!ok) throw new Error("Por-4 APLC/FPLC identity gate failed — legend reading suspect");
}

// ---- emit ----
const all = [
  ...por5.curves.map(c => ({ chart: "Por-5", ...c })),
  ...por4.curves.map(c => ({ chart: "Por-4", ...c })),
];
writeFileSync("out_por45.json", JSON.stringify(all, null, 1));

// SNP is the legacy sidewall neutron pad tool — it shares Por-4 with the APS
// curves because both are epithermal, but it is not an APS output, so it gets
// its own prefix instead of the (wrong) APS_ one.
const rustName = c =>
  c.chart === "Por-5" ? `CNL_${c.tool}_${c.mat}` : c.tool === "SNP" ? `SNP_${c.mat}` : `APS_${c.tool}_${c.mat}`;
let rs = `//! GENERATED by tools/chartdig/gen_por45.mjs — DO NOT EDIT BY HAND.
//!
//! Neutron porosity equivalence curves digitized at vector precision from the
//! Schlumberger Log Interpretation Charts 2013 edition, charts Por-5 (CNL
//! thermal: NPHI ratio method + TNPH env-corrected, 0 / 250,000 ppm) and Por-4
//! (epithermal: APS APLC/FPLC + the legacy sidewall SNP). Each table maps
//! apparent limestone neutron porosity (x) to true porosity for the indicated
//! matrix (y), both in v/v. Calcite (limestone) is the identity and has no
//! table. Tables are strictly increasing in both coordinates, so they are
//! invertible; outside the tabulated span, extend with the end-segment slope.
//! Note the sandstone curves leave the chart's 40-pu frame top early, so their
//! tables end at x ~ 0.32-0.36 apparent limestone; dolomite runs to ~ 0.40.
//!
//! Validation (in the generator, each a hard gate): grid-index calibration rms
//! < 0.03 pu; the black calcite diagonal reads back as the identity within a
//! |bias| < 0.25 pu / scatter < 0.08 pu split (the small constant-sign frame
//! offset is common to every curve on a page and cancels in conversions);
//! every emitted table strictly monotone and on the correct side of the
//! identity; the Por-4 solid/dashed family identity pinned by the dolomite
//! matrix-effect contrast (|FPLC| > |APLC| + 2 pu at 20 pu); and Por-5's
//! printed worked example (TNPH 18 pu at 250 kppm -> quartz sandstone 24 pu)
//! reproduced to 0.6 pu.

`;
for (const c of all) {
  rs += `/// ${c.chart}: ${c.tool.replace("_", " ")} — ${c.mat === "SS" ? "quartz sandstone" : "dolomite"}\n`;
  rs += `pub const ${rustName(c)}: &[(f32, f32)] = &[\n`;
  for (let i = 0; i < c.pts.length; i += 6) {
    rs += "    " + c.pts.slice(i, i + 6).map(([x, y]) => `(${(x / 100).toFixed(4)}, ${(y / 100).toFixed(4)})`).join(", ") + ",\n";
  }
  rs += "];\n\n";
}
writeFileSync(OUT_RS, rs);
console.log(`\nwrote out_por45.json + ${OUT_RS}: ${all.map(c => `${rustName(c)}:${c.pts.length}`).join(" ")}`);
