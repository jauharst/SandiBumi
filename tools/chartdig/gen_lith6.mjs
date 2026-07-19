// Lith-6 (former CP-21): Umaa-Rhomaa matrix-identification (MID) ternary
// triangle, p212.json.  X = Umaa (b/cc), minor 0.2/line; Y = rhomaa g/cc,
// INVERTED (2.2 top), minor 0.01/line.  This page draws vertical minors in
// light gray "155,156,159" and horizontal minors in "112,113,118" - merge.
// Blue "3,70,145": 3 sides + 12 internal ternary lines (2-pt segments),
// 7 mineral circles (66-pt closed polys), 5 arrows (shaft + 5-pt head):
// Salt / Barite / Heavy minerals (off-scale direction) and 2x Gas direction.
// Vertices = intersections of adjacent side lines; internal 20/40/60/80
// subdivisions are re-emitted ANALYTICALLY from the vertices and validated
// against the drawn ones.        node gen_lith6.mjs  ->  out_lith6.json
import { writeFileSync } from "fs";
import { loadPage, colorKey, gridLines, gridFit, anchorAxis, polyLen } from "./lib.mjs";

const BLUE = "3,70,145";
let fails = 0;
const gate = (ok, msg) => { console.log(`${ok ? "PASS" : "FAIL"} ${msg}`); if (!ok) fails++; };

const { strokes, texts } = loadPage("p212.json");

// ---- calibration: merge the two gray tones, fit grid, anchor with labels ----
const merged = strokes.map(s => (colorKey(s) === "112,113,118" ? { ...s, color: [155, 156, 159] } : s));
const { hs, vs } = gridLines(merged);
const gy = gridFit(hs), gx = gridFit(vs);
console.log(`grid: ${vs.length}v (${gx.slope.toFixed(3)}pt/line, rms ${gx.rms.toFixed(3)}) x ${hs.length}h (${gy.slope.toFixed(3)}pt/line, rms ${gy.rms.toFixed(3)})`);
const X_STEP = 0.2, Y_STEP = 0.01; // Umaa per vertical line, g/cc per horizontal line
const xa = anchorAxis(texts, gx, { labelRe: /^(2|4|6|8|10|12|14|16)$/, step: X_STEP, side: "x", edge: hs[0] });
const ya = anchorAxis(texts, gy, { labelRe: /^[23]\.\d$/, step: -Y_STEP, side: "y", edge: vs[0] });
console.log(`anchors: x base ${xa.base} votes ${JSON.stringify(xa.votes)} | y base ${ya.base} votes ${JSON.stringify(ya.votes)}`);
const toU = px => xa.base + ((px - gx.inter) / gx.slope) * X_STEP;
const toRho = py => ya.base - Y_STEP * ((py - gy.inter) / gy.slope);
gate(Math.max(...Object.values(xa.votes)) === xa.nLabels && xa.nLabels === 8, `x anchor unanimous (${xa.nLabels} labels 2..16 -> base ${xa.base})`);
gate(Math.max(...Object.values(ya.votes)) === ya.nLabels && ya.nLabels === 10, `y anchor unanimous (${ya.nLabels} labels 2.2..3.1 -> base ${ya.base})`);

// ---- classify the blue artwork ----
const circles = [], segments = [], heads = [];
for (const s of strokes.filter(s => colorKey(s) === BLUE)) for (const p of s.polys) {
  const xs = p.map(q => q[0]), ys = p.map(q => q[1]);
  const w = Math.max(...xs) - Math.min(...xs), h = Math.max(...ys) - Math.min(...ys);
  const closed = Math.hypot(p[0][0] - p[p.length - 1][0], p[0][1] - p[p.length - 1][1]) < 0.5;
  if (p.length >= 20 && closed && w > 3 && w < 9 && h > 3 && h < 9)
    circles.push({ c: [(Math.min(...xs) + Math.max(...xs)) / 2, (Math.min(...ys) + Math.max(...ys)) / 2] });
  else if (p.length === 2 && polyLen(p) > 25) segments.push({ a: p[0], b: p[1] });
  else if (p.length <= 8 && closed && polyLen(p) >= 10 && polyLen(p) <= 20) heads.push(p);
  else console.log(`  [unclassified blue poly: ${p.length} pts, len ${polyLen(p).toFixed(1)}]`);
}
console.log(`blue art: ${circles.length} circles, ${segments.length} segments, ${heads.length} arrowheads`);
gate(circles.length === 7 && heads.length === 5, `blue inventory: 7 circles + 5 arrowheads found`);

// ---- triangle sides: segments whose BOTH endpoints sit near circle centers ----
const near = (p, q, tol) => Math.hypot(p[0] - q[0], p[1] - q[1]) <= tol;
const sideSegs = [], rest = [];
for (const s of segments) {
  const ia = circles.findIndex(c => near(s.a, c.c, 6)), ib = circles.findIndex(c => near(s.b, c.c, 6));
  if (ia >= 0 && ib >= 0 && ia !== ib) sideSegs.push({ ...s, ia, ib });
  else rest.push(s);
}
gate(sideSegs.length === 3, `exactly 3 side segments touch two circles each (got ${sideSegs.length})`);
const vertexCircleIdx = [...new Set(sideSegs.flatMap(s => [s.ia, s.ib]))];
gate(vertexCircleIdx.length === 3, `side segments define 3 vertex circles`);

// infinite-line intersection of the two sides adjacent to each vertex circle
const lineInt = (s1, s2) => {
  const [x1, y1] = s1.a, [x2, y2] = s1.b, [x3, y3] = s2.a, [x4, y4] = s2.b;
  const den = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
  const t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / den;
  return [x1 + t * (x2 - x1), y1 + t * (y2 - y1)];
};
const vertices = vertexCircleIdx.map(ci => {
  const [s1, s2] = sideSegs.filter(s => s.ia === ci || s.ib === ci);
  const dev = lineInt(s1, s2);
  const circ = circles[ci].c;
  return { ci, dev, circ, U: toU(dev[0]), rho: toRho(dev[1]), Uc: toU(circ[0]), rhoc: toRho(circ[1]) };
});
// roles: Quartz = min Umaa, Calcite = max Umaa, Dolomite = max rhomaa
const Q = vertices.reduce((a, v) => (v.U < a.U ? v : a));
const C = vertices.reduce((a, v) => (v.U > a.U ? v : a));
const D = vertices.find(v => v !== Q && v !== C);
for (const [n, v] of [["Quartz", Q], ["Calcite", C], ["Dolomite", D]]) {
  console.log(`${n}: intersection (${v.U.toFixed(3)}, ${v.rho.toFixed(4)})  circle-center (${v.Uc.toFixed(3)}, ${v.rhoc.toFixed(4)})  dU ${(v.U - v.Uc).toFixed(3)} drho ${(v.rho - v.rhoc).toFixed(4)}`);
  gate(Math.abs(v.U - v.Uc) <= 0.3 && Math.abs(v.rho - v.rhoc) <= 0.02, `${n} vertex matches drawn corner circle within 0.3 Umaa / 0.02 rhomaa`);
}
gate(Math.abs(Q.U - 4.8) <= 0.3 && Math.abs(Q.rho - 2.65) <= 0.02, `Quartz (${Q.U.toFixed(2)}, ${Q.rho.toFixed(3)}) near textbook (4.8, 2.65)`);
gate(Math.abs(C.U - 13.8) <= 0.3 && Math.abs(C.rho - 2.71) <= 0.02, `Calcite (${C.U.toFixed(2)}, ${C.rho.toFixed(3)}) near textbook (13.8, 2.71)`);
gate(Math.abs(D.U - 9.0) <= 0.3 && D.rho >= 2.85 && D.rho <= 2.89, `Dolomite (${D.U.toFixed(2)}, ${D.rho.toFixed(3)}) near textbook (9.0, 2.87ish)`);

// ---- split remaining segments: internal ternary lines vs arrow shafts ----
const bary = (p, A, B, Cc) => { // barycentric of p in triangle A,B,C (any 2D coords)
  const v0 = [B[0] - A[0], B[1] - A[1]], v1 = [Cc[0] - A[0], Cc[1] - A[1]], v2 = [p[0] - A[0], p[1] - A[1]];
  const den = v0[0] * v1[1] - v0[1] * v1[0];
  const wB = (v2[0] * v1[1] - v2[1] * v1[0]) / den, wC = (v0[0] * v2[1] - v0[1] * v2[0]) / den;
  return [1 - wB - wC, wB, wC]; // weights of A, B, C
};
const inTri = p => bary(p, Q.dev, C.dev, D.dev).every(w => w > -0.05 && w < 1.05);
const internal = rest.filter(s => inTri(s.a) && inTri(s.b));
const shafts = rest.filter(s => !(inTri(s.a) && inTri(s.b)));
gate(internal.length === 12, `12 internal ternary lines inside triangle (got ${internal.length})`);
gate(shafts.length === 5, `5 arrow shafts outside triangle (got ${shafts.length})`);

// ---- analytic 20/40/60/80 subdivisions from vertices (data coords) ----
const lerp = (A, B, f) => [A[0] + f * (B[0] - A[0]), A[1] + f * (B[1] - A[1])];
const Qd = [Q.U, Q.rho], Cd = [C.U, C.rho], Dd = [D.U, D.rho];
const analytic = [];
for (const f of [0.2, 0.4, 0.6, 0.8]) {
  analytic.push({ fam: "dolomite", f, p1: lerp(Qd, Dd, f), p2: lerp(Cd, Dd, f) }); // const dolomite
  analytic.push({ fam: "quartz", f, p1: lerp(Dd, Qd, f), p2: lerp(Cd, Qd, f) }); // const quartz
  analytic.push({ fam: "calcite", f, p1: lerp(Qd, Cd, f), p2: lerp(Dd, Cd, f) }); // const calcite
}
// validate drawn internal lines against analytic ones (0.01 rho scaled = 0.2 U,
// i.e. both minor cells weigh equally: metric = hypot(dU, drho*20))
const dist2 = (p, q) => Math.hypot(p[0] - q[0], (p[1] - q[1]) * 20);
let worst = 0, matched = 0;
for (const s of internal) {
  const a = [toU(s.a[0]), toRho(s.a[1])], b = [toU(s.b[0]), toRho(s.b[1])];
  let best = null;
  for (const al of analytic) {
    const d = Math.min(Math.max(dist2(a, al.p1), dist2(b, al.p2)), Math.max(dist2(a, al.p2), dist2(b, al.p1)));
    if (!best || d < best.d) best = { d, al };
  }
  worst = Math.max(worst, best.d);
  matched++;
  console.log(`  drawn line (${a[0].toFixed(2)},${a[1].toFixed(3)})-(${b[0].toFixed(2)},${b[1].toFixed(3)}) = ${best.al.f * 100}% ${best.al.fam} line, max endpoint dev ${best.d.toFixed(3)}`);
}
gate(matched === 12 && worst <= 0.15, `all 12 drawn internal lines match analytic subdivisions, worst endpoint dev ${worst.toFixed(3)} <= 0.15 units`);

// ---- worked example: (Umaa 13, rhomaa 2.74) -> ~20% dolomite / 80% calcite ----
const [fq, fc, fd] = bary([13, 2.74], Qd, Cd, Dd);
console.log(`worked example (13, 2.74): quartz ${fq.toFixed(3)}, calcite ${fc.toFixed(3)}, dolomite ${fd.toFixed(3)}`);
gate(Math.abs(fd - 0.2) <= 0.07 && Math.abs(fq) <= 0.05, `example reads dolomite ${(fd * 100).toFixed(1)}% (expect 20 +/- 7) with quartz ~0 (${(fq * 100).toFixed(1)}%)`);

// ---- mineral circles (non-vertex) + labels; arrows + labels ----
const tCenter = t => [t.x + (t.w || 0) / 2, t.y + 3];
const nearestText = (p, names) => {
  let best = null;
  for (const t of texts.filter(t => names.includes(t.s.trim()))) {
    const d = Math.hypot(p[0] - tCenter(t)[0], p[1] - tCenter(t)[1]);
    if (!best || d < best.d) best = { d, s: t.s.trim() };
  }
  return best;
};
const mineralNames = ["K-feldspar", "Kaolinite", "Illite", "Anhydrite"];
const minerals = [];
for (let i = 0; i < circles.length; i++) {
  if (vertexCircleIdx.includes(i)) continue;
  const lbl = nearestText(circles[i].c, mineralNames);
  minerals.push({ x: toU(circles[i].c[0]), y: toRho(circles[i].c[1]), label: lbl.s, dTxt: lbl.d });
  console.log(`mineral ${lbl.s}: (${toU(circles[i].c[0]).toFixed(3)}, ${toRho(circles[i].c[1]).toFixed(4)}) [label ${lbl.d.toFixed(1)}pt away]`);
}
gate(minerals.length === 4 && new Set(minerals.map(m => m.label)).size === 4, `4 non-vertex mineral circles uniquely labeled: ${minerals.map(m => m.label).join(", ")}`);

const arrowNames = ["Salt", "Barite", "Heavy minerals", "Gas direction"];
const arrows = [];
for (const s of shafts) {
  const mid = [(s.a[0] + s.b[0]) / 2, (s.a[1] + s.b[1]) / 2];
  const lbl = nearestText(mid, arrowNames);
  // tip end = endpoint nearest an arrowhead; extend to the head's apex
  let tipEnd = null, head = null, bd = Infinity;
  for (const h of heads) for (const e of [s.a, s.b]) {
    const d = Math.min(...h.map(hp => Math.hypot(hp[0] - e[0], hp[1] - e[1])));
    if (d < bd) { bd = d; tipEnd = e; head = h; }
  }
  const base = tipEnd === s.a ? s.b : s.a;
  const apex = head.reduce((a, p) => (Math.hypot(p[0] - base[0], p[1] - base[1]) > Math.hypot(a[0] - base[0], a[1] - base[1]) ? p : a));
  arrows.push({ label: lbl.s, pts: [[toU(base[0]), toRho(base[1])], [toU(apex[0]), toRho(apex[1])]] });
  console.log(`arrow ${lbl.s}: (${toU(base[0]).toFixed(2)}, ${toRho(base[1]).toFixed(3)}) -> tip (${toU(apex[0]).toFixed(2)}, ${toRho(apex[1]).toFixed(3)}) [head ${bd.toFixed(1)}pt from shaft end]`);
}
const arrowCounts = arrows.reduce((a, x) => ((a[x.label] = (a[x.label] || 0) + 1), a), {});
gate(arrowCounts["Salt"] === 1 && arrowCounts["Barite"] === 1 && arrowCounts["Heavy minerals"] === 1 && arrowCounts["Gas direction"] === 2,
  `arrows labeled: ${JSON.stringify(arrowCounts)}`);

// ---- assemble overlay def ----
const r3 = v => Math.round(v * 1e3) / 1e3, r4 = v => Math.round(v * 1e4) / 1e4;
const pt = ([u, r]) => [r3(u), r4(r)];
// side labels as printed on the chart: QC "% calcite", DQ "% quartz", CD "% dolomite"
const lines = [
  { pts: [pt(Qd), pt(Cd)], label: "% calcite", dash: false },
  { pts: [pt(Dd), pt(Qd)], label: "% quartz", dash: false },
  { pts: [pt(Cd), pt(Dd)], label: "% dolomite", dash: false },
  ...analytic.map(al => ({ pts: [pt(al.p1), pt(al.p2)], dash: false })),
  ...arrows.map(a => ({ pts: a.pts.map(pt), label: a.label, dash: false })),
];
const points = [
  { x: r3(Q.U), y: r4(Q.rho), label: "Quartz" },
  { x: r3(C.U), y: r4(C.rho), label: "Calcite" },
  { x: r3(D.U), y: r4(D.rho), label: "Dolomite" },
  ...minerals.map(m => ({ x: r3(m.x), y: r4(m.y), label: m.label })),
];
const out = [{
  id: "lith6_mid",
  label: "Lith-6 Umaa-Rhomaa MID plot (former CP-21)",
  xAxis: "umaa",
  yAxis: "rhomaa",
  points,
  lines,
  isoConnect: false,
}];
writeFileSync("out_lith6.json", JSON.stringify(out, null, 1));
console.log(`\nwrote out_lith6.json: ${out.length} def, ${points.length} points, ${lines.length} lines`);
if (fails) { console.log(`\n*** ${fails} GATE(S) FAILED ***`); process.exit(1); }
console.log("ALL GATES PASSED");
