// Lith-1 (p205.json): clay-mineral identification boxes, two stacked panels.
//   Top:    Pe (0..10 linear, y) vs K %      (0..10 linear, x)
//   Bottom: Pe (0..10 linear, y) vs Th/K     (LOG 0.1..100, x)
// Boxes = dark-blue (3,70,145) 5-pt closed axis-aligned rects; circles = 66-pt
// closed loops of the same color; labels = text items. Grid is sparse majors,
// some lines split by text -> cluster split pieces by coordinate, then reuse
// gridFit + anchorAxis from lib.  node gen_lith1.mjs
import { writeFileSync } from "fs";
import { loadPage, gridFit, anchorAxis, colorKey } from "./lib.mjs";

const { strokes, texts } = loadPage("p205.json");
const GRAY = "155,156,159", BLUE = "3,70,145";
const bbox = p => {
  const xs = p.map(q => q[0]), ys = p.map(q => q[1]);
  return { x0: Math.min(...xs), y0: Math.min(...ys), x1: Math.max(...xs), y1: Math.max(...ys) };
};
const fail = msg => { console.log("GATE FAIL: " + msg); throw new Error(msg); };

// ---------- panel split: cluster gray segments by coordinate within a y-band ----------
function graySegs() {
  const h = [], v = [];
  for (const s of strokes.filter(s => colorKey(s) === GRAY)) for (const p of s.polys) {
    const b = bbox(p);
    if (b.y1 - b.y0 < 0.5 && b.x1 - b.x0 > 5) h.push({ c: (b.y0 + b.y1) / 2, a: b.x0, b: b.x1 });
    else if (b.x1 - b.x0 < 0.5 && b.y1 - b.y0 > 5) v.push({ c: (b.x0 + b.x1) / 2, a: b.y0, b: b.y1 });
  }
  return { h, v };
}
function cluster(segs) { // merge split pieces sharing a coordinate (tol 1pt); return centers + total span
  const out = [];
  for (const s of segs.sort((p, q) => p.c - q.c)) {
    const last = out[out.length - 1];
    if (last && Math.abs(s.c - last.cs[0]) < 1) { last.cs.push(s.c); last.span += s.b - s.a; last.lo = Math.min(last.lo, s.a); last.hi = Math.max(last.hi, s.b); }
    else out.push({ cs: [s.c], span: s.b - s.a, lo: s.a, hi: s.b });
  }
  return out.map(o => ({ c: o.cs.reduce((a, b) => a + b) / o.cs.length, span: o.span, lo: o.lo, hi: o.hi }));
}
const segs = graySegs();
const PANELS = {
  top: { yBand: [395, 660], xLabelBandY: [388, 404], name: "top (Pe vs K)" },
  bot: { yBand: [88, 345], xLabelBandY: [78, 90], name: "bottom (Pe vs Th/K)" },
};
for (const key of ["top", "bot"]) {
  const P = PANELS[key];
  const hs = cluster(segs.h.filter(s => s.c > P.yBand[0] && s.c < P.yBand[1]));
  const vs = cluster(segs.v.filter(s => (s.a + s.b) / 2 > P.yBand[0] && (s.a + s.b) / 2 < P.yBand[1]));
  const wMax = Math.max(...hs.map(o => o.span)), hMax = Math.max(...vs.map(o => o.span));
  P.hs = hs.filter(o => o.span > 0.8 * wMax); P.vs = vs.filter(o => o.span > 0.8 * hMax);
  // frame = extent of the gray grid itself
  P.frame = { x0: Math.min(...P.hs.map(o => o.lo)), x1: Math.max(...P.hs.map(o => o.hi)),
              y0: Math.min(...P.vs.map(o => o.lo)), y1: Math.max(...P.vs.map(o => o.hi)) };
  console.log(`${P.name}: ${P.vs.length} v-lines [${P.vs.map(o => o.c.toFixed(1)).join(", ")}], ${P.hs.length} h-lines [${P.hs.map(o => o.c.toFixed(1)).join(", ")}], frame x ${P.frame.x0.toFixed(1)}..${P.frame.x1.toFixed(1)} y ${P.frame.y0.toFixed(1)}..${P.frame.y1.toFixed(1)}`);
}

// ---------- y axes (both panels): Pe major lines every 2, labels 0..10 ----------
for (const key of ["top", "bot"]) {
  const P = PANELS[key];
  const gy = gridFit(P.hs.map(o => o.c));
  const yTexts = texts.filter(t => t.x < P.frame.x0 - 2 && t.y > P.yBand[0] && t.y < P.yBand[1]);
  const ya = anchorAxis(yTexts, gy, { labelRe: /^(0|2|4|6|8|10)$/, step: 2, side: "y", edge: P.vs[0].c });
  P.toPe = py => ya.base + 2 * (py - gy.inter) / gy.slope;
  console.log(`GATE y-axis ${key}: gridFit slope ${gy.slope.toFixed(3)} rms ${gy.rms.toFixed(3)} | anchor base ${ya.base} votes ${JSON.stringify(ya.votes)} (${ya.nLabels} labels)`);
  if (gy.rms > 0.5) fail(`${key} y gridFit rms ${gy.rms}`);
  if (Math.max(...Object.values(ya.votes)) < ya.nLabels - 1 || ya.nLabels < 5) fail(`${key} y anchor votes not unanimous`);
  const pe0 = P.toPe(P.frame.y0), pe10 = P.toPe(P.frame.y1);
  console.log(`GATE y-edges ${key}: Pe(frame bottom)=${pe0.toFixed(3)} (want 0), Pe(frame top)=${pe10.toFixed(3)} (want 10)`);
  if (Math.abs(pe0) > 0.1 || Math.abs(pe10 - 10) > 0.1) fail(`${key} Pe frame edges off`);
}

// ---------- top x axis: K % linear, major lines every 2, labels 0,2,...,10 ----------
{
  const P = PANELS.top;
  const gx = gridFit(P.vs.map(o => o.c));
  const xTexts = texts.filter(t => t.y > P.xLabelBandY[0] && t.y < P.xLabelBandY[1]);
  const xa = anchorAxis(xTexts, gx, { labelRe: /^(0|2|4|6|8|10)$/, step: 2, side: "x", edge: P.frame.y0 });
  P.toX = px => xa.base + 2 * (px - gx.inter) / gx.slope;
  console.log(`GATE x-axis top: gridFit slope ${gx.slope.toFixed(3)} rms ${gx.rms.toFixed(3)} | anchor base ${xa.base} votes ${JSON.stringify(xa.votes)} (${xa.nLabels} labels)`);
  if (gx.rms > 0.5) fail("top x gridFit rms");
  if (Math.max(...Object.values(xa.votes)) < xa.nLabels - 1 || xa.nLabels < 5) fail("top x anchor votes not unanimous");
  const k0 = P.toX(P.frame.x0), k10 = P.toX(P.frame.x1);
  console.log(`GATE x-edges top: K(frame left)=${k0.toFixed(3)} (want 0), K(frame right)=${k10.toFixed(3)} (want 10)`);
  if (Math.abs(k0) > 0.1 || Math.abs(k10 - 10) > 0.1) fail("top K frame edges off");
}

// ---------- bottom x axis: log10(Th/K) calibrated on decade lines 0.1, 1, 10, 100 ----------
{
  const P = PANELS.bot;
  const LABVALS = [0.2, 0.3, 0.6, 1, 2, 3, 6, 10, 20, 30, 60]; // interior gray lines, left to right
  if (P.vs.length !== LABVALS.length) fail(`bottom: expected ${LABVALS.length} interior v-lines, got ${P.vs.length}`);
  // labels vote: each interior line must sit at the width-center of its label
  const xTexts = texts.filter(t => t.y > P.xLabelBandY[0] && t.y < P.xLabelBandY[1]);
  P.vs.forEach((o, i) => {
    const lab = xTexts.find(t => Math.abs(parseFloat(t.s) - LABVALS[i]) < 1e-9);
    if (!lab) fail(`bottom: no label ${LABVALS[i]}`);
    const d = o.c - (lab.x + lab.w / 2);
    if (Math.abs(d) > 2) fail(`bottom: line ${LABVALS[i]} is ${d.toFixed(2)}pt from its label center`);
  });
  console.log(`GATE bottom x labels: all ${LABVALS.length} interior lines within 2pt of their label centers`);
  // decade anchors: frame edges (0.1, 100) + the 1 and 10 lines; LSQ device = A + B*log10(v)
  const dec = [[Math.log10(0.1), P.frame.x0], [0, P.vs[3].c], [1, P.vs[7].c], [2, P.frame.x1]];
  const n = dec.length, sl = dec.reduce((a, d) => a + d[0], 0), sx = dec.reduce((a, d) => a + d[1], 0);
  const sll = dec.reduce((a, d) => a + d[0] * d[0], 0), slx = dec.reduce((a, d) => a + d[0] * d[1], 0);
  const B = (n * slx - sl * sx) / (n * sll - sl * sl), A = (sx - B * sl) / n;
  P.toX = px => Math.pow(10, (px - A) / B);
  const decResid = dec.map(d => (A + B * d[0] - d[1]));
  console.log(`GATE bottom decades: fit ${A.toFixed(2)} + ${B.toFixed(2)}*log10(v); residuals [${decResid.map(r => r.toFixed(2)).join(", ")}]pt`);
  if (decResid.some(r => Math.abs(r) > 1.0)) fail("bottom decade residual > 1pt");
  // sanity: interior lines vs the decade fit (artwork wobble, informational gate <= 3pt)
  const resid = P.vs.map((o, i) => ({ v: LABVALS[i], r: o.c - (A + B * Math.log10(LABVALS[i])) }));
  console.log(`bottom interior-line residuals vs log fit: ${resid.map(o => `${o.v}:${o.r.toFixed(2)}`).join(" ")}`);
  if (resid.some(o => Math.abs(o.r) > 3)) fail("bottom interior line > 3pt off log fit");
}

// ---------- boxes + circles (dark blue) ----------
const boxes = [], circles = [];
for (const s of strokes.filter(s => colorKey(s) === BLUE)) for (const p of s.polys) {
  const closed = Math.hypot(p[0][0] - p[p.length - 1][0], p[0][1] - p[p.length - 1][1]) < 0.5;
  const b = bbox(p);
  if (closed && p.length === 5) {
    // must be axis-aligned rectangle: every edge parallel to an axis
    const axisAligned = p.slice(1).every((q, i) => Math.abs(q[0] - p[i][0]) < 0.05 || Math.abs(q[1] - p[i][1]) < 0.05);
    if (!axisAligned) fail(`non-axis-aligned 5-pt blue poly at ${JSON.stringify(b)}`);
    boxes.push(b);
  } else if (closed && p.length > 20 && b.x1 - b.x0 < 8 && b.y1 - b.y0 < 8) {
    circles.push({ cx: (b.x0 + b.x1) / 2, cy: (b.y0 + b.y1) / 2 });
  } else fail(`unclassified blue poly pts=${p.length} at ${JSON.stringify(b)}`);
}
console.log(`blue geometry: ${boxes.length} boxes, ${circles.length} circles`);
if (boxes.length !== 15 || circles.length !== 13) fail("expected 15 boxes + 13 circles");

// ---------- label -> box assignment per panel (exact min-cost bijection) ----------
const EXPECT = {
  top: ["Chlorite", "Glauconite", "Biotite", "Illite", "Montmorillonite", "Muscovite", "Kaolinite"],
  bot: ["Chlorite", "Glauconite", "Biotite", "Illite", "Montmorillonite", "Muscovite", "Kaolinite", "Mixed layer"],
};
const rectGap = (lo, hi, a, b) => (hi < a ? a - hi : lo > b ? lo - b : 0);
function assign(panelKey) {
  const P = PANELS[panelKey];
  const inPanel = y => y > P.yBand[0] && y < P.yBand[1];
  const pboxes = boxes.filter(b => inPanel((b.y0 + b.y1) / 2));
  const labs = EXPECT[panelKey].map(name => {
    const t = texts.find(t => t.s === name && inPanel(t.y));
    if (!t) fail(`${panelKey}: label "${name}" not found`);
    return { name, x0: t.x, x1: t.x + t.w, y0: t.y, y1: t.y + 7 };
  });
  if (pboxes.length !== labs.length) fail(`${panelKey}: ${pboxes.length} boxes vs ${labs.length} labels`);
  const cost = (l, b) => 2 * rectGap(l.y0, l.y1, b.y0, b.y1) + rectGap(l.x0, l.x1, b.x0, b.x1);
  let best = null; // brute-force min-cost permutation (n <= 8)
  const perm = [], used = new Array(pboxes.length).fill(false);
  (function rec(i, tot) {
    if (best && tot >= best.tot) return;
    if (i === labs.length) { best = { tot, perm: perm.slice() }; return; }
    for (let j = 0; j < pboxes.length; j++) if (!used[j]) {
      used[j] = true; perm.push(j); rec(i + 1, tot + cost(labs[i], pboxes[j])); perm.pop(); used[j] = false;
    }
  })(0, 0);
  const out = labs.map((l, i) => ({ name: l.name, box: pboxes[best.perm[i]], cost: cost(l, pboxes[best.perm[i]]) }));
  for (const o of out) {
    console.log(`  ${panelKey} "${o.name}" -> device box [${o.box.x0.toFixed(1)},${o.box.y0.toFixed(1)},${o.box.x1.toFixed(1)},${o.box.y1.toFixed(1)}] (label-gap cost ${o.cost.toFixed(1)}pt)`);
    if (o.cost > 30) fail(`${panelKey} "${o.name}" label-gap cost ${o.cost.toFixed(1)} > 30pt`);
  }
  console.log(`GATE assignment ${panelKey}: ${out.length} labels matched 1:1, total cost ${best.tot.toFixed(1)}pt`);
  return out;
}
console.log("label -> box assignment:");
const topAsg = assign("top"), botAsg = assign("bot");

// ---------- circles -> boxes (smallest containing box) ----------
function attachCircles(asg, panelKey) {
  const P = PANELS[panelKey];
  const pc = circles.filter(c => c.cy > P.yBand[0] && c.cy < P.yBand[1]);
  for (const c of pc) {
    const owners = asg.filter(o => c.cx >= o.box.x0 - 0.3 && c.cx <= o.box.x1 + 0.3 && c.cy >= o.box.y0 - 0.3 && c.cy <= o.box.y1 + 0.3)
      .sort((a, b) => (a.box.x1 - a.box.x0) * (a.box.y1 - a.box.y0) - (b.box.x1 - b.box.x0) * (b.box.y1 - b.box.y0));
    if (!owners.length) fail(`${panelKey}: circle at (${c.cx.toFixed(1)},${c.cy.toFixed(1)}) inside no box`);
    const o = owners[0];
    if (o.circle) fail(`${panelKey}: box "${o.name}" has two circles`);
    o.circle = c;
  }
  const without = asg.filter(o => !o.circle).map(o => o.name);
  console.log(`GATE circles ${panelKey}: ${pc.length} circles each inside its own box; boxes without circle: [${without.join(", ") || "none"}]`);
  return without;
}
const w1 = attachCircles(topAsg, "top"), w2 = attachCircles(botAsg, "bot");
if (w1.join() !== "Chlorite" || w2.join() !== "Mixed layer") fail("unexpected circle-less boxes");

// ---------- convert to data coords + sanity/physics gates ----------
const r3 = v => Math.round(v * 1000) / 1000;
function dataBox(o, P) {
  const xs = [P.toX(o.box.x0), P.toX(o.box.x1)].sort((a, b) => a - b);
  const ys = [P.toPe(o.box.y0), P.toPe(o.box.y1)].sort((a, b) => a - b);
  return { xLo: xs[0], xHi: xs[1], peLo: ys[0], peHi: ys[1] };
}
console.log("box bounds in data coords:");
for (const [asg, P, xName, xSane] of [[topAsg, PANELS.top, "K", [-0.05, 10.05]], [botAsg, PANELS.bot, "ThK", [0.095, 101]]]) {
  for (const o of asg) {
    o.d = dataBox(o, P);
    if (o.circle) o.pt = { x: P.toX(o.circle.cx), y: P.toPe(o.circle.cy) };
    console.log(`  ${xName} panel ${o.name}: ${xName} ${r3(o.d.xLo)}..${r3(o.d.xHi)}, Pe ${r3(o.d.peLo)}..${r3(o.d.peHi)}${o.pt ? `, circle (${r3(o.pt.x)}, ${r3(o.pt.y)})` : ""}`);
    if (o.d.xLo < xSane[0] || o.d.xHi > xSane[1]) fail(`${o.name} ${xName} out of range`);
    if (o.d.peLo < -0.05 || o.d.peHi > 10.05) fail(`${o.name} Pe out of range`);
  }
}
// cross-panel consistency: each shared mineral's Pe range must agree between panels
console.log("GATE cross-panel Pe consistency (shared minerals, tol 0.15):");
for (const name of EXPECT.top) {
  const a = topAsg.find(o => o.name === name).d, b = botAsg.find(o => o.name === name).d;
  const dLo = Math.abs(a.peLo - b.peLo), dHi = Math.abs(a.peHi - b.peHi);
  console.log(`  ${name}: top Pe ${r3(a.peLo)}..${r3(a.peHi)} vs bottom ${r3(b.peLo)}..${r3(b.peHi)} (d ${r3(dLo)}/${r3(dHi)})`);
  if (dLo > 0.15 || dHi > 0.15) fail(`${name} Pe mismatch between panels`);
}
// physics spot checks (chartbook/textbook mineral values act as the worked example)
const PHYS = { Kaolinite: 1.8, Montmorillonite: 2.1, Muscovite: 2.4, Illite: 3.5, Glauconite: 6.4, Biotite: 6.3, Chlorite: 6.3 };
console.log("GATE physics: published mineral Pe must fall in (box range +- 0.6):");
for (const [name, pe] of Object.entries(PHYS)) {
  const d = topAsg.find(o => o.name === name).d;
  const ok = pe > d.peLo - 0.6 && pe < d.peHi + 0.6;
  console.log(`  ${name}: published Pe ${pe} vs box ${r3(d.peLo)}..${r3(d.peHi)} ${ok ? "OK" : "FAIL"}`);
  if (!ok) fail(`${name} physics Pe check`);
}
const musK = topAsg.find(o => o.name === "Muscovite").d, bioK = topAsg.find(o => o.name === "Biotite").d,
      chlK = topAsg.find(o => o.name === "Chlorite").d, kaoT = botAsg.find(o => o.name === "Kaolinite").d,
      glaT = botAsg.find(o => o.name === "Glauconite").d;
console.log(`GATE physics x: muscovite K ${r3(musK.xLo)}..${r3(musK.xHi)} (want hi-K >6), biotite K ${r3(bioK.xLo)}..${r3(bioK.xHi)} (>6), chlorite K ${r3(chlK.xLo)}..${r3(chlK.xHi)} (<0.5), kaolinite ThK ${r3(kaoT.xLo)}..${r3(kaoT.xHi)} (>=10), glauconite ThK ${r3(glaT.xLo)}..${r3(glaT.xHi)} (low, <2)`);
if (!(musK.xLo > 6 && bioK.xLo > 6 && chlK.xHi < 0.5 && kaoT.xLo >= 9.5 && glaT.xLo < 2)) fail("physics x-axis check");

// ---------- emit ----------
const mkDef = (id, label, xAxis, asg) => ({
  id, label, xAxis, yAxis: "pe",
  regions: asg.map(o => ({
    poly: [[r3(o.d.xLo), r3(o.d.peLo)], [r3(o.d.xHi), r3(o.d.peLo)], [r3(o.d.xHi), r3(o.d.peHi)], [r3(o.d.xLo), r3(o.d.peHi)]],
    label: o.name,
  })),
  points: asg.filter(o => o.pt).map(o => ({ x: r3(o.pt.x), y: r3(o.pt.y), label: o.name })),
});
const out = [
  mkDef("lith1_pek", "Lith-1 mineral ID: Pe vs K (%)", "k", topAsg),
  mkDef("lith1_pethk", "Lith-1 mineral ID: Pe vs Th/K ratio (log x axis)", "thk", botAsg),
];
writeFileSync("out_lith1.json", JSON.stringify(out, null, 1));
console.log(`\nwrote out_lith1.json: ${out.length} defs, ${out[0].regions.length}+${out[1].regions.length} regions, ${out[0].points.length}+${out[1].points.length} points`);
