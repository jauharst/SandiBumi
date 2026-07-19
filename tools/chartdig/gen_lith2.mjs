// Lith-2 (p206.json): NGS Th (ppm, y 0..25) vs K (%, x 0..5) clay-mineral chart.
// Content: 6 blue Th/K ratio lines from the origin (emitted analytic from their
// labeled ratios), 2 red dashed lines (clay line, feldspar/evaporite line,
// least-squares fitted), and mineral-field text labels as label-only points.
// node gen_lith2.mjs  ->  out_lith2.json
import { writeFileSync } from "fs";
import { loadPage, gridLines, gridFit, anchorAxis, coloredPolys, mergeChains } from "./lib.mjs";

const { strokes, texts } = loadPage("p206.json");
const { hs, vs } = gridLines(strokes);
const gy = gridFit(hs), gx = gridFit(vs);
console.log(`grid: ${vs.length}v (${gx.slope.toFixed(3)}pt/line, rms ${gx.rms.toFixed(3)}) x ${hs.length}h (${gy.slope.toFixed(3)}pt/line, rms ${gy.rms.toFixed(3)})`);

// sparse major-only grid: v lines are K=1..4 (frame carries 0 and 5), h lines are
// Th=5,10,15,20 in 5-ppm steps — anchor both with the axis labels.
const xa = anchorAxis(texts, gx, { labelRe: /^[0-5]$/, step: 1, side: "x", edge: hs[0] });
const ya = anchorAxis(texts, gy, { labelRe: /^(0|5|10|15|20|25)$/, step: 5, side: "y", edge: vs[0] });
console.log(`anchors: x base ${xa.base} votes ${JSON.stringify(xa.votes)} | y base ${ya.base} votes ${JSON.stringify(ya.votes)}`);
if (Math.max(...Object.values(xa.votes)) < 5 || Math.max(...Object.values(ya.votes)) < 5)
  throw new Error("axis anchor votes too weak");
const toK = px => xa.base + (px - gx.inter) / gx.slope;
const toTh = py => ya.base + 5 * (py - gy.inter) / gy.slope;
const r4 = v => Math.round(v * 1e4) / 1e4;

// least-squares y = a + b*x through data-space points
const fitLine = pts => {
  const n = pts.length;
  let sx = 0, sy = 0, sxx = 0, sxy = 0;
  for (const [x, y] of pts) { sx += x; sy += y; sxx += x * x; sxy += x * y; }
  const b = (n * sxy - sx * sy) / (n * sxx - sx * sx);
  return { a: (sy - b * sx) / n, b };
};

// ---------- (1) blue Th/K ratio lines ----------
const blue = coloredPolys(strokes, "3,70,145").map(p => {
  const d = p.map(q => [toK(q[0]), toTh(q[1])]);
  const ks = d.map(q => q[0]);
  return { d, ...fitLine(d), k0: Math.min(...ks), k1: Math.max(...ks) };
});
// ratio lines fan out from the origin: near-zero Th-intercept, substantial K span
const fan = blue.filter(l => Math.abs(l.a) < 1.5 && l.k1 - l.k0 > 0.5);
const extras = blue.filter(l => !fan.includes(l));
console.log(`blue polys ${blue.length}: ${fan.length} origin-fan candidates, ${extras.length} extra`);
for (const l of extras) console.log(`  extra blue line (not emitted): slope ${l.b.toFixed(3)} icpt ${l.a.toFixed(3)} K ${l.k0.toFixed(2)}..${l.k1.toFixed(2)}`);

const ratioLabels = texts.filter(t => /^Th\/K = /.test(t.s)).map(t => ({
  s: t.s, r: parseFloat(t.s.replace("Th/K = ", "")),
  K: toK(t.x + (t.w || 0) / 2), Th: toTh(t.y),
})).sort((a, b) => b.r - a.r);
if (ratioLabels.length !== 6) throw new Error(`expected 6 Th/K labels, got ${ratioLabels.length}`);

let ratioFails = 0;
const ratioLines = [];
const used = new Set();
for (const L of ratioLabels) {
  // nearest fan line by vertical distance at the label's K (labels sit on their lines)
  const scored = fan.map(l => ({ l, d: Math.abs(l.a + l.b * L.K - L.Th) })).sort((a, b) => a.d - b.d);
  const best = scored[0], second = scored[1];
  if (used.has(best.l)) throw new Error(`label ${L.s} collides with an already-matched line`);
  used.add(best.l);
  const diff = (best.l.b - L.r) / L.r * 100;
  const ok = Math.abs(diff) <= 5;
  if (!ok) ratioFails++;
  console.log(`GATE ratio ${L.s}: drawn slope ${best.l.b.toFixed(4)} vs labeled ${L.r} -> ${diff >= 0 ? "+" : ""}${diff.toFixed(1)}% ${ok ? "PASS" : "FAIL"} | label-on-line dTh ${best.d.toFixed(2)} ppm (2nd-nearest ${second.d.toFixed(2)})`);
  const kEnd = Math.min(5, 25 / L.r); // analytic line clipped at x=5 / y=25
  ratioLines.push({ pts: [[0, 0], [r4(kEnd), r4(L.r * kEnd)]], label: L.s, dash: false });
}

// ---------- (2) red dashed lines ----------
const redChains = mergeChains(coloredPolys(strokes, "255,64,58"))
  .map(ch => ch.map(q => [toK(q[0]), toTh(q[1])]));
const meanTh = ch => ch.reduce((s, q) => s + q[1], 0) / ch.length;
const clayPts = [], feldPts = [];
for (const ch of redChains) (meanTh(ch) > 8 ? clayPts : feldPts).push(...ch);
if (!clayPts.length || !feldPts.length) throw new Error("red line grouping failed");
const fc = fitLine(clayPts), ff = fitLine(feldPts);
const clayTh0 = fc.a, clayTh5 = fc.a + 5 * fc.b;
const clayOk = [clayTh0, clayTh5].every(v => v >= 15 && v <= 22);
if (!clayOk) ratioFails += 100; // count as hard failure
console.log(`GATE clay line: Th(K=0) ${clayTh0.toFixed(2)}, Th(K=5) ${clayTh5.toFixed(2)} (range 15..22) ${clayOk ? "PASS" : "FAIL"}`);
console.log(`CHECK 100% illite point: clay line at K=5 -> (5, ${clayTh5.toFixed(2)}) vs chart's ~(5, 20)`);
console.log(`feldspar line fit: Th(K=0) ${ff.a.toFixed(3)}, Th(K=5) ${(ff.a + 5 * ff.b).toFixed(3)} (slope ${ff.b.toFixed(3)})`);
const redLines = [
  { pts: [[0, r4(fc.a)], [5, r4(fc.a + 5 * fc.b)]], label: "Clay line", dash: true },
  { pts: [[0, r4(ff.a)], [5, r4(ff.a + 5 * ff.b)]], label: "Feldspar line", dash: true },
];

// ---------- (3) mineral-field labels as label-only points ----------
const wanted = ["Kaolinite", "Montmorillonite", "Chlorite", "Mixed-layer clay", "Illite",
  "Micas", "Glauconite", "Feldspar", "~70% illite", "~30% glauconite", "100% illite point"];
const points = [];
for (const w of wanted) {
  const t = texts.find(t => t.s === w);
  if (!t) throw new Error(`label text not found: ${w}`);
  points.push({ x: r4(toK(t.x + (t.w || 0) / 2)), y: r4(toTh(t.y)), label: w });
}
{ // "~40% mica" is typeset as two stacked text items — merge their centers
  const t1 = texts.find(t => t.s === "~40%"), t2 = texts.find(t => t.s === "mica");
  if (!t1 || !t2) throw new Error("~40% mica label parts not found");
  points.push({
    x: r4((toK(t1.x + t1.w / 2) + toK(t2.x + t2.w / 2)) / 2),
    y: r4((toTh(t1.y) + toTh(t2.y)) / 2),
    label: "~40% mica",
  });
}
let outside = points.filter(p => p.x < 0 || p.x > 5 || p.y < 0 || p.y > 25);
console.log(`GATE labels in frame: ${points.length - outside.length}/${points.length} inside ${outside.length ? "FAIL " + JSON.stringify(outside) : "PASS"}`);
for (const p of points) console.log(`  label "${p.label}" at K ${p.x} Th ${p.y}`);

const def = {
  id: "lith2_thk",
  label: "Lith-2 NGS clay-mineral identification (Th vs K)",
  xAxis: "k",
  yAxis: "th",
  lines: [...ratioLines, ...redLines],
  points,
};
writeFileSync("out_lith2.json", JSON.stringify([def], null, 1));
console.log(`\nwrote out_lith2.json: 1 def, ${def.lines.length} lines, ${def.points.length} points`);
console.log(`SUMMARY: ratio-slope gate ${6 - (ratioFails % 100)}/6 within 5%; clay gate ${clayOk ? "PASS" : "FAIL"}; labels gate ${outside.length ? "FAIL" : "PASS"}`);
