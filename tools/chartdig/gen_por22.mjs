// Por-22 (p250.json): Density vs Sonic crossplot, customary (former CP-7).
// X = DT us/ft (40..130, gray minor line every 2 us/ft, 44 lines starting at 42);
// Y = rhob g/cc (1.8..3.0, minor 0.02, bottom gray line = 2.98).
// BLUE 3,70,145 = time-average (straight 2-pt lines), RED 255,64,58 = field
// observation (17-pt chains). Graduations every 5 p.u., labels every 10.
// y of a graduation is analytic: rhob = t/100*rhof + (1-t/100)*rma, rhof = 1.0.
// Tick dashes: the on-curve endpoint is the data tip; tick side flips per curve
// (calcite ticks point up-left, quartz/dolomite down-right) and the red curves
// cross near the top, so tips are picked per-dash by (analytic 5-multiple
// residual + distance-to-chain) with page-wide greedy competition.
// node gen_por22.mjs  ->  out_por22.json
import { writeFileSync } from "fs";
import {
  loadPage, gridLines, gridFit, anchorAxis, curveChains, chainExtreme,
  coloredPolys, polyLen, minDistToChain,
} from "./lib.mjs";

const RHOF = 1.0;
const { strokes, texts } = loadPage("p250.json");
const { hs, vs } = gridLines(strokes);
const gy = gridFit(hs), gx = gridFit(vs);
console.log(`grid: ${vs.length}v (${gx.slope.toFixed(3)}pt, rms ${gx.rms.toFixed(3)}) x ${hs.length}h (${gy.slope.toFixed(3)}pt, rms ${gy.rms.toFixed(3)})`);

const xa = anchorAxis(texts, gx, { labelRe: /^(4|5|6|7|8|9|10|11|12|13)0$/, step: 2, side: "x", edge: hs[0] });
const ya = anchorAxis(texts, gy, { labelRe: /^[123]\.\d$/, step: -0.02, side: "y", edge: vs[0] });
console.log(`anchors: x base ${xa.base} votes ${JSON.stringify(xa.votes)} | y base ${ya.base} votes ${JSON.stringify(ya.votes)}`);
const toDT = px => xa.base + 2 * (px - gx.inter) / gx.slope;
const toRhob = py => ya.base - 0.02 * (py - gy.inter) / gy.slope;
const pyOf = rho => gy.inter + gy.slope * (ya.base - rho) / 0.02;

const gates = [];
const gate = (name, ok, detail) => { console.log(`GATE ${name}: ${ok ? "PASS" : "FAIL"} — ${detail}`); gates.push({ name, ok }); };

// frame rectangle (44,46,53) edges must read DT 40/130 and rhob 3.0/1.8
const frameRect = coloredPolys(strokes, "44,46,53").find(p => {
  const xs = p.map(q => q[0]), ys = p.map(q => q[1]);
  return Math.max(...xs) - Math.min(...xs) > 300 && Math.max(...ys) - Math.min(...ys) > 400;
});
{
  const xs = frameRect.map(q => q[0]), ys = frameRect.map(q => q[1]);
  const dtL = toDT(Math.min(...xs)), dtR = toDT(Math.max(...xs));
  const rB = toRhob(Math.min(...ys)), rT = toRhob(Math.max(...ys));
  gate("frame-x", Math.abs(dtL - 40) < 0.2 && Math.abs(dtR - 130) < 0.2, `left ${dtL.toFixed(2)} (exp 40), right ${dtR.toFixed(2)} (exp 130)`);
  gate("frame-y", Math.abs(rB - 3.0) < 0.01 && Math.abs(rT - 1.8) < 0.01, `bottom ${rB.toFixed(4)} (exp 3.0), top ${rT.toFixed(4)} (exp 1.8)`);
}

const frame = { x0: vs[0], x1: vs[vs.length - 1], y0: hs[0], y1: hs[hs.length - 1] };
const MATS = [
  { name: "Quartz sandstone", rmaCands: [2.65] },
  { name: "Calcite (limestone)", rmaCands: [2.71] },
  { name: "Dolomite", rmaCands: [2.87, 2.85] },
];
const FAMS = [
  { key: "ta", id: "por22_ta", color: "3,70,145", label: "Por-22 Density-Sonic crossplot (customary), time average" },
  { key: "fo", id: "por22_fo", color: "255,64,58", label: "Por-22 Density-Sonic crossplot (customary), field observation" },
];

const chainXatY = (ch, y) => {
  for (let i = 1; i < ch.length; i++) {
    const [x1, y1] = ch[i - 1], [x2, y2] = ch[i];
    if ((y1 - y) * (y2 - y) <= 0 && y1 !== y2) return x1 + (x2 - x1) * (y - y1) / (y2 - y1);
  }
  return null;
};

// ---- extraction per family (mats carry a fixed rma each) --------------------
function extractFamily(fam, rmaByMat) {
  const chains = curveChains(strokes, fam.color, frame).filter(c => c.len > 100).map(c => c.ch);
  if (chains.length !== 3) throw new Error(`${fam.key}: expected 3 curve chains, got ${chains.length}`);
  const mats = MATS.map(m => {
    const rma = rmaByMat[m.name];
    const chain = chains.find(ch => Math.abs(toRhob(chainExtreme(ch, "minY")[1]) - rma) <= 0.05);
    return { name: m.name, rma, chain };
  });
  if (mats.some(m => !m.chain) || new Set(mats.map(m => m.chain)).size !== 3)
    throw new Error(`${fam.key}: chain<->matrix assignment not distinct/complete`);

  // candidates per dash: (mat, endpoint) with endpoint near that mat's chain
  const dashes = coloredPolys(strokes, fam.color).filter(d => {
    const l = polyLen(d);
    return l > 1.2 && l < 16 && d.length <= 8;
  });
  const cands = []; // {di, mat, e, dist, est, t, resid, score, len}
  dashes.forEach((d, di) => {
    const len = polyLen(d);
    for (const m of mats) for (const e of [d[0], d[d.length - 1]]) {
      const dist = minDistToChain(e, m.chain);
      if (dist > 2.0) continue;
      const est = (m.rma - toRhob(e[1])) / (m.rma - RHOF) * 100;
      const t = 5 * Math.round(est / 5);
      if (t < 0) continue;
      const resid = Math.abs(est - t);
      cands.push({ di, mat: m, e, dist, est, t, resid, score: resid + 0.3 * dist, len });
    }
  });
  // page-wide greedy: best score first; one assignment per dash, one per (mat,t)
  cands.sort((a, b) => a.score - b.score);
  const usedDash = new Set(), usedSlot = new Set(), asg = [];
  for (const c of cands) {
    const slot = `${c.mat.name}|${c.t}`;
    if (usedDash.has(c.di) || usedSlot.has(slot)) continue;
    usedDash.add(c.di); usedSlot.add(slot); asg.push(c);
  }
  const unassigned = dashes.length - usedDash.size;
  const lens = asg.map(a => a.len).sort((a, b) => a - b);
  const medLen = lens[Math.floor(lens.length / 2)];
  return { mats, asg, unassigned, medLen, nDashes: dashes.length };
}

// dolomite rma candidate scoring (2.87 vs 2.85): rms of graduation residuals
const rmaByMat = { "Quartz sandstone": 2.65, "Calcite (limestone)": 2.71 };
{
  const scores = MATS[2].rmaCands.map(rma => {
    let rms = 0, n = 0, un = 0;
    for (const fam of FAMS) {
      const r = extractFamily(fam, { ...rmaByMat, "Dolomite": rma });
      const dol = r.asg.filter(a => a.mat.name === "Dolomite");
      for (const a of dol) { rms += a.resid * a.resid; n++; }
      un += r.unassigned;
    }
    return { rma, rms: Math.sqrt(rms / n), n, un };
  });
  console.log(`dolomite rma scoring: ${scores.map(s => `${s.rma}: rms ${s.rms.toFixed(3)} over ${s.n} grads, ${s.un} unassigned`).join(" | ")}`);
  scores.sort((a, b) => (a.un - b.un) || (a.rms - b.rms));
  rmaByMat["Dolomite"] = scores[0].rma;
  gate("dol-rma", scores[0].rms < 0.5, `chose rma ${scores[0].rma} (rms ${scores[0].rms.toFixed(3)})`);
}

// ---- build output -----------------------------------------------------------
const out = [];
const curveGeom = []; // for worked-example check: {famKey, name, grads}
for (const fam of FAMS) {
  const { mats, asg, unassigned, medLen, nDashes } = extractFamily(fam, rmaByMat);
  gate(`${fam.key}-dashes`, unassigned === 0, `${nDashes} dashes, ${unassigned} unassigned (median len ${medLen.toFixed(1)})`);
  const curves = [];
  for (const m of mats) {
    const rows = asg.filter(a => a.mat === m)
      .map(a => ({ t: a.t, x: toDT(a.e[0]), est: a.est, long: a.len >= 1.3 * medLen, dist: a.dist }))
      .sort((a, b) => a.t - b.t);
    // fill interior gaps from the drawn chain at the analytic y
    const filled = [];
    const have = new Set(rows.map(r => r.t));
    for (let t = rows[0].t; t <= rows[rows.length - 1].t; t += 5) if (!have.has(t)) {
      const px = chainXatY(m.chain, pyOf(t / 100 * RHOF + (1 - t / 100) * m.rma));
      if (px !== null) { rows.push({ t, x: toDT(px), est: t, long: false, filledFromChain: true }); filled.push(t); }
    }
    rows.sort((a, b) => a.t - b.t);
    const gaps = rows.slice(1).map((r, i) => r.t - rows[i].t).filter(g => g !== 5);
    const longs = rows.filter(r => r.long).map(r => r.t);
    const badLongs = longs.filter(t => t % 5 !== 0);
    const estRms = Math.sqrt(rows.filter(r => !r.filledFromChain).reduce((s, r) => s + (r.est - r.t) ** 2, 0) / rows.filter(r => !r.filledFromChain).length);
    console.log(`${m.name} [${fam.key}]: rma ${m.rma}, t ${rows[0].t}..${rows[rows.length - 1].t} step5, ${rows.length} grads, estRms ${estRms.toFixed(3)}, maxTipDist ${Math.max(...rows.filter(r => !r.filledFromChain).map(r => r.dist)).toFixed(2)}${filled.length ? `, filled-from-chain [${filled.join(",")}]` : ""}`);
    gate(`${fam.key}-${m.name}-seq`, gaps.length === 0, `gaps ${gaps.join(",") || "none"}`);
    gate(`${fam.key}-${m.name}-longs`, badLongs.length === 0, `longs [${longs.join(",") || "none"}] all on 5-multiples`);
    gate(`${fam.key}-${m.name}-est`, estRms < 0.35, `graduation estimate rms ${estRms.toFixed(3)} p.u.`);
    if (fam.key === "ta") {
      // time average is exactly linear: DT(t) = dtma + t/100*(dtf - dtma); dtf must be 189
      const n = rows.length, st = rows.reduce((s, r) => s + r.t, 0), sx = rows.reduce((s, r) => s + r.x, 0);
      const stt = rows.reduce((s, r) => s + r.t * r.t, 0), stx = rows.reduce((s, r) => s + r.t * r.x, 0);
      const b = (n * stx - st * sx) / (n * stt - st * st), a = (sx - b * st) / n;
      const dtf = a + 100 * b;
      const rmsLin = Math.sqrt(rows.reduce((s, r) => s + (a + b * r.t - r.x) ** 2, 0) / n);
      gate(`ta-${m.name}-dtf`, Math.abs(dtf - 189) <= 1.5, `dtma ${a.toFixed(1)}, implied dtf ${dtf.toFixed(1)} (chart states 189), linear rms ${rmsLin.toFixed(2)} us/ft`);
    }
    const rhobOf = t => t / 100 * RHOF + (1 - t / 100) * m.rma;
    const grads = rows.map(r => [r.t, Math.round(r.x * 1e4) / 1e4, Math.round(rhobOf(r.t) * 1e4) / 1e4]);
    curves.push({ name: m.name, rmaChart: m.rma, labelEvery: 10, grads });
    curveGeom.push({ famKey: fam.key, name: m.name, grads });
  }
  out.push({ id: fam.id, label: fam.label, xAxis: "dt", yAxis: "density", curves, isoConnect: true });
}

// ---- mineral points (blue circles + text labels) ----------------------------
const MINERALS = ["Sylvite", "Salt", "Trona", "Gypsum", "Sulfur", "Polyhalite", "Anhydrite"];
const circles = coloredPolys(strokes, "3,70,145").filter(p => p.length >= 20).map(p => {
  const xs = p.map(q => q[0]), ys = p.map(q => q[1]);
  return { cx: (Math.min(...xs) + Math.max(...xs)) / 2, cy: (Math.min(...ys) + Math.max(...ys)) / 2, w: Math.max(...xs) - Math.min(...xs) };
}).filter(c => c.w > 2 && c.w < 8);
const labelTexts = texts.filter(t => MINERALS.includes(t.s.trim()));
gate("minerals-found", circles.length === 7 && labelTexts.length === 7, `${circles.length} circles, ${labelTexts.length} labels (expect 7 each)`);
const pairs = [];
for (const lt of labelTexts) for (const c of circles)
  pairs.push({ lt, c, d: Math.hypot(lt.x + (lt.w || 0) / 2 - c.cx, lt.y + 3.5 - c.cy) });
pairs.sort((a, b) => a.d - b.d);
const usedL = new Set(), usedC = new Set(), points = [];
for (const p of pairs) {
  if (usedL.has(p.lt) || usedC.has(p.c)) continue;
  usedL.add(p.lt); usedC.add(p.c);
  points.push({ x: Math.round(toDT(p.c.cx) * 1e4) / 1e4, y: Math.round(toRhob(p.c.cy) * 1e4) / 1e4, label: p.lt.s.trim(), _d: p.d });
}
points.sort((a, b) => a.x - b.x);
for (const p of points) console.log(`mineral point: ${p.label.padEnd(11)} DT ${p.x.toFixed(2)} us/ft, rhob ${p.y.toFixed(3)} g/cc (label-circle dist ${p._d.toFixed(1)}pt)`);
const salt = points.find(p => p.label === "Salt"), sylv = points.find(p => p.label === "Sylvite");
gate("minerals-sanity", Math.abs(salt.x - 67) < 1.5 && Math.abs(salt.y - 2.04) < 0.02 && Math.abs(sylv.x - 74) < 1.5 && Math.abs(sylv.y - 1.86) < 0.02,
  `Salt (${salt.x.toFixed(1)}, ${salt.y.toFixed(3)}) exp ~(67, 2.04); Sylvite (${sylv.x.toFixed(1)}, ${sylv.y.toFixed(3)}) exp ~(74, 1.86)`);
for (const p of points) delete p._d;
out[0].points = points;

// ---- worked example: (DT 82, rhob 2.3) on TA calcite at t = 24 +/- 1 --------
// note: the red field-observation quartz curve genuinely crosses the blue
// calcite time-average curve at this exact spot on the chart, so "nearest
// overall" can flip on sub-line-width noise; the gate requires TA calcite to
// read 24 +/- 1 AND to be nearest overall or tied within one line width.
{
  const norm = (dt, rho) => [(dt - 40) / 90, (rho - 1.8) / 1.2];
  const P = norm(82, 2.3);
  const nearestOf = list => {
    let best = null;
    for (const cg of list) for (let i = 1; i < cg.grads.length; i++) {
      const [t1, x1, r1] = cg.grads[i - 1], [t2, x2, r2] = cg.grads[i];
      const A = norm(x1, r1), B = norm(x2, r2);
      const vx = B[0] - A[0], vy = B[1] - A[1], L2 = vx * vx + vy * vy;
      let f = L2 ? ((P[0] - A[0]) * vx + (P[1] - A[1]) * vy) / L2 : 0;
      f = Math.max(0, Math.min(1, f));
      const d = Math.hypot(P[0] - (A[0] + f * vx), P[1] - (A[1] + f * vy));
      if (!best || d < best.d) best = { d, cg, t: t1 + f * (t2 - t1) };
    }
    return best;
  };
  const bestTA = nearestOf(curveGeom.filter(c => c.famKey === "ta"));
  const bestAll = nearestOf(curveGeom);
  const LINE_W = 0.002; // ~0.18 us/ft in normalized units, about one stroke width
  const taOk = bestTA.cg.name === "Calcite (limestone)" && Math.abs(bestTA.t - 24) <= 1;
  const tie = bestAll.cg === bestTA.cg || bestAll.d >= bestTA.d - LINE_W;
  gate("worked-example", taOk && tie,
    `(DT 82, rhob 2.3): nearest TA curve = ${bestTA.cg.name} at t ${bestTA.t.toFixed(2)}, dist ${bestTA.d.toFixed(5)} (expect calcite, 24 +/- 1); ` +
    `nearest overall = ${bestAll.cg.name} [${bestAll.cg.famKey}] at t ${bestAll.t.toFixed(2)}, dist ${bestAll.d.toFixed(5)}` +
    (bestAll.cg !== bestTA.cg ? ` (curve-crossing tie: margin ${(bestTA.d - bestAll.d).toFixed(5)} < line width ${LINE_W})` : ""));
}

writeFileSync("out_por22.json", JSON.stringify(out, null, 1));
const fails = gates.filter(g => !g.ok);
console.log(`\nwrote out_por22.json: ${out.length} defs, ${out.map(d => d.curves.length).join("+")} curves, ${out[0].points.length} points`);
console.log(fails.length ? `GATES FAILED: ${fails.map(f => f.name).join(", ")}` : "ALL GATES PASSED");
if (fails.length) process.exitCode = 1;
