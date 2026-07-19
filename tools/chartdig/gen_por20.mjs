// Por-20 (p247): Sonic transit time vs CNL thermal neutron porosity (customary).
// x: apparent limestone neutron porosity (pu, minor grid 1 pu, labels 0..40)
// y: DT us/ft, minor grid 1 us/ft, INCREASING with device y (base = bottom line)
// BLUE 3,70,145 solid = time-average family; RED 255,64,58 dashed = field
// observation (dash pattern is a stroke property -> chains are continuous paths,
// so the short red strokes are pure graduation ticks). y is NOT analytic:
// sequential t along each curve anchored at the bottom graduation 0.
// node gen_por20.mjs
import { writeFileSync } from "fs";
import {
  loadPage, gridLines, gridFit, anchorAxis, curveChains, dashTipsMulti, chainExtreme,
  validateRows, colorKey,
} from "./lib.mjs";

const { strokes, texts } = loadPage("p247.json");
const { hs, vs } = gridLines(strokes);
const gy = gridFit(hs), gx = gridFit(vs);
console.log(`grid: ${vs.length}v (${gx.slope.toFixed(3)}pt, rms ${gx.rms.toFixed(3)}) x ${hs.length}h (${gy.slope.toFixed(3)}pt, rms ${gy.rms.toFixed(3)})`);

const xa = anchorAxis(texts, gx, { labelRe: /^(0|5|10|15|20|25|30|35|40|45)$/, step: 1, side: "x", edge: hs[0] });
const ya = anchorAxis(texts, gy, { labelRe: /^(40|50|60|70|80|90|100|110)$/, step: 1, side: "y", edge: vs[0] });
console.log(`anchors: x base ${xa.base} votes ${JSON.stringify(xa.votes)} | y base ${ya.base} votes ${JSON.stringify(ya.votes)}`);

const toPu = px => xa.base + (px - gx.inter) / gx.slope;
const toDt = py => ya.base + (py - gy.inter) / gy.slope;
const devX = pu => gx.inter + (pu - xa.base) * gx.slope;
const devY = dt => gy.inter + (dt - ya.base) * gy.slope;
const frame = { x0: vs[0], x1: vs[vs.length - 1], y0: hs[0], y1: hs[hs.length - 1] };

// ---- chain selection: 3 longest per color, minerals by bottom DT descending ----
const FAMS = [
  { color: "3,70,145", suffix: "_ta", tag: "time average", expect: { "Quartz sandstone": 55.5, "Calcite (limestone)": 47.5, "Dolomite": 43.5 } },
  { color: "255,64,58", suffix: "_fo", tag: "field observation", expect: null },
];
const MINERALS = ["Quartz sandstone", "Calcite (limestone)", "Dolomite"]; // bottom DT high -> low
const targets = [];
const famJobs = [];
for (const fam of FAMS) {
  const chains = curveChains(strokes, fam.color, frame, 13).filter(c => c.len > 100);
  if (chains.length !== 3) throw new Error(`${fam.tag}: expected 3 chains, got ${chains.length}`);
  const withBot = chains.map(({ ch }) => ({ ch, bot: chainExtreme(ch, "minY") }))
    .sort((a, b) => b.bot[1] - a.bot[1]); // highest bottom DT first = quartz
  const mats = withBot.map((c, i) => ({
    name: MINERALS[i], chain: c.ch, key: `${fam.suffix}|${MINERALS[i]}`, color: fam.color,
    botDt: toDt(c.bot[1]), botPu: toPu(c.bot[0]),
  }));
  for (const m of mats) {
    const exp = fam.expect?.[m.name];
    const ok = exp == null || Math.abs(m.botDt - exp) <= 1.0;
    console.log(`GATE chain-id ${fam.tag} ${m.name}: bottom DT ${m.botDt.toFixed(2)} pu ${m.botPu.toFixed(2)}${exp != null ? ` (expect ~${exp}) ${ok ? "OK" : "FAIL"}` : ""}`);
    if (!ok) throw new Error(`chain-id gate failed for ${m.name}`);
    targets.push({ color: m.color, chain: m.chain, key: m.key });
  }
  famJobs.push({ ...fam, mats });
}

const tipMap = dashTipsMulti(strokes, targets, { tipSide: "maxX", lenMax: 16, maxDist: 6 });

// ---- per-curve graduation tables (sequential t from bottom = 0) ----
const allGrads = []; // {name, fam, t, dev:[x,y], pu, dt} for label cross-check
const overlays = [];
for (const fam of famJobs) {
  const curves = [];
  for (const m of fam.mats) {
    const tips = tipMap.get(m.key) ?? [];
    const lens = tips.map(t => t.len).sort((a, b) => a - b);
    const medLen = lens[Math.floor(lens.length / 2)] || 3;
    const bottom = chainExtreme(m.chain, "minY");
    const entries = [{ pu: toPu(bottom[0]), dt: toDt(bottom[1]), dev: bottom, len: 0 }];
    for (const t of tips) {
      const pu = toPu(t.tip[0]), dt = toDt(t.tip[1]);
      if (Math.abs(dt - entries[0].dt) > 0.35 || Math.abs(pu - entries[0].pu) > 0.6) entries.push({ pu, dt, dev: t.tip, len: t.len });
    }
    entries.sort((a, b) => a.dt - b.dt);
    // pairwise dedupe (a stray double-dash would shift the whole sequence)
    for (let i = 1; i < entries.length; i++) {
      if (entries[i].dt - entries[i - 1].dt < 0.35 && Math.abs(entries[i].pu - entries[i - 1].pu) < 0.6) {
        console.log(`  [dedupe ${m.name}${fam.suffix}: dropped near-duplicate at DT ${entries[i].dt.toFixed(2)}]`);
        entries.splice(i, 1); i--;
      }
    }
    // sequential spacing sanity: each device-DT gap vs neighbor average
    const gapsDt = entries.slice(1).map((e, i) => e.dt - entries[i].dt);
    let badGaps = 0;
    for (let i = 0; i < gapsDt.length; i++) {
      const nb = [];
      if (i > 0) nb.push(gapsDt[i - 1]);
      if (i + 1 < gapsDt.length) nb.push(gapsDt[i + 1]);
      const ref = nb.reduce((a, b) => a + b, 0) / nb.length;
      if (gapsDt[i] > 1.8 * ref || gapsDt[i] < 0.55 * ref) { badGaps++; console.log(`  [gap warn ${m.name}${fam.suffix}: gap ${gapsDt[i].toFixed(2)} vs local ${ref.toFixed(2)} after t=${i}]`); }
    }
    const rows = entries.map((e, i) => ({ t: i, pu: e.pu, dt: e.dt, dev: e.dev, long: e.len >= medLen * 1.3 }));
    const v = validateRows(rows.map(r => ({ t: r.t, long: r.long })), { name: `${m.name}${fam.suffix}`, strict: false });
    const longs = rows.filter(r => r.long).length;
    console.log(`GATE seq ${m.name}${fam.suffix}: ${rows.length} grads t 0..${rows.length - 1}, gaps ${v.gaps.length}, badLongs ${v.badLongs.length} (${longs} longs), badSpacing ${badGaps} ${v.gaps.length === 0 && v.badLongs.length === 0 && badGaps === 0 ? "OK" : "FAIL"}`);
    if (v.gaps.length || v.badLongs.length || badGaps) throw new Error(`sequential gate failed for ${m.name}${fam.suffix}`);
    // straight-line fit DT vs t
    const n = rows.length;
    const st = rows.reduce((a, r) => a + r.t, 0), sd = rows.reduce((a, r) => a + r.dt, 0);
    const stt = rows.reduce((a, r) => a + r.t * r.t, 0), std = rows.reduce((a, r) => a + r.t * r.dt, 0);
    const b = (n * std - st * sd) / (n * stt - st * st), a = (sd - b * st) / n;
    const ssRes = rows.reduce((s, r) => s + (r.dt - (a + b * r.t)) ** 2, 0);
    const mean = sd / n, ssTot = rows.reduce((s, r) => s + (r.dt - mean) ** 2, 0);
    const r2 = 1 - ssRes / ssTot;
    if (fam.suffix === "_ta") {
      const exp = fam.expect[m.name];
      const ok = r2 > 0.995 && Math.abs(a - exp) <= 1.0;
      console.log(`GATE straight ${m.name}${fam.suffix}: DT = ${a.toFixed(2)} + ${b.toFixed(4)}*t, R^2 ${r2.toFixed(5)} (>0.995), dt(0) ${a.toFixed(2)} expect ~${exp} ${ok ? "OK" : "FAIL"}`);
      if (!ok) throw new Error(`straightness gate failed for ${m.name}`);
    } else {
      console.log(`info fit ${m.name}${fam.suffix}: DT = ${a.toFixed(2)} + ${b.toFixed(4)}*t, R^2 ${r2.toFixed(5)} (field obs, no gate)`);
    }
    for (const r of rows) allGrads.push({ name: m.name, fam: fam.suffix, t: r.t, dev: r.dev, pu: r.pu, dt: r.dt });
    curves.push({
      name: m.name, labelEvery: 5,
      grads: rows.map(r => [r.t, Math.round(r.pu * 1e2) / 1e4, Math.round(r.dt * 1e3) / 1e3]),
    });
  }
  overlays.push({ fam, curves });
}

// ---- graduation label cross-check (labels drawn along the curves) ----
const inPlot = texts.filter(t => /^(0|5|10|15|20|25|30|35|40)$/.test(t.s.trim()) && t.x > vs[0] + 2 && t.x < vs[vs.length - 1] && t.y > hs[0] + 2 && t.y < hs[hs.length - 1]);
let match = 0, miss = [];
for (const t of inPlot) {
  const c = [t.x + (t.w || 0) / 2, t.y + 3.2];
  let best = null, bd = Infinity;
  for (const g of allGrads) {
    const d = Math.hypot(g.dev[0] - c[0], g.dev[1] - c[1]);
    if (d < bd) { bd = d; best = g; }
  }
  if (best && best.t === parseFloat(t.s)) match++;
  else miss.push(`"${t.s}"@(${t.x.toFixed(0)},${t.y.toFixed(0)}) -> ${best.name}${best.fam} t=${best.t} d=${bd.toFixed(1)}`);
}
console.log(`GATE labels: ${match}/${inPlot.length} in-plot graduation labels match nearest grad t ${match / inPlot.length >= 0.8 ? "OK" : "FAIL"}`);
if (miss.length) console.log("  mismatches: " + miss.join(" | "));
if (match / inPlot.length < 0.8) throw new Error("label gate failed");

// ---- worked example: (nphi 20 pu, DT 89) nearest quartz TA graduation = 24.5 +/- 1 ----
const ex = [devX(20), devY(89)];
let exBest = null, exD = Infinity;
for (const g of allGrads.filter(g => g.fam === "_ta" && g.name === "Quartz sandstone")) {
  const d = Math.hypot(g.dev[0] - ex[0], g.dev[1] - ex[1]);
  if (d < exD) { exD = d; exBest = g; }
}
const exOk = Math.abs(exBest.t - 24.5) <= 1;
console.log(`GATE example: (20 pu, DT 89) nearest quartz time-average grad t=${exBest.t} (dev dist ${exD.toFixed(1)}pt, grad at ${exBest.pu.toFixed(2)} pu / DT ${exBest.dt.toFixed(2)}) expect 24.5+/-1 ${exOk ? "OK" : "FAIL"}`);
if (!exOk) throw new Error("worked example gate failed");

// ---- Salt / Anhydrite reference circles (blue 66-pt small polys) ----
const circles = [];
for (const s of strokes.filter(s => colorKey(s) === "3,70,145")) for (const p of s.polys) {
  const xs = p.map(q => q[0]), ys = p.map(q => q[1]);
  const w = Math.max(...xs) - Math.min(...xs), h = Math.max(...ys) - Math.min(...ys);
  if (p.length >= 20 && w < 8 && h < 8) circles.push([(Math.max(...xs) + Math.min(...xs)) / 2, (Math.max(...ys) + Math.min(...ys)) / 2]);
}
if (circles.length !== 2) throw new Error(`expected 2 reference circles, got ${circles.length}`);
const named = circles.map(c => {
  let best = null, bd = Infinity;
  for (const t of texts.filter(t => /^(Salt|Anhydrite)$/.test(t.s.trim()))) {
    const d = Math.hypot(t.x + (t.w || 0) / 2 - c[0], t.y + 3.2 - c[1]);
    if (d < bd) { bd = d; best = t.s.trim(); }
  }
  return { label: best, pu: toPu(c[0]), dt: toDt(c[1]) };
});
const anh = named.find(p => p.label === "Anhydrite"), salt = named.find(p => p.label === "Salt");
const ptsOk = anh && salt && Math.abs(anh.dt - 50) <= 0.7 && Math.abs(salt.dt - 66.7) <= 1.0 && salt.pu > -4 && salt.pu < 0 && anh.pu > -4 && anh.pu < 0;
console.log(`GATE points: Salt (${salt.pu.toFixed(2)} pu, DT ${salt.dt.toFixed(2)}) expect ~(-2, 66.7); Anhydrite (${anh.pu.toFixed(2)} pu, DT ${anh.dt.toFixed(2)}) expect ~(-2, 50) ${ptsOk ? "OK" : "FAIL"}`);
if (!ptsOk) throw new Error("reference points gate failed");

// ---- emit ----
const out = [];
for (const o of overlays) {
  const def = {
    id: "por20" + o.fam.suffix,
    label: `Por-20 Sonic vs CNL neutron (${o.fam.tag})`,
    xAxis: "neutron", yAxis: "dt",
    curves: o.curves,
    isoConnect: true,
  };
  if (o.fam.suffix === "_ta") def.points = named.map(p => ({ x: Math.round(p.pu * 1e2) / 1e4, y: Math.round(p.dt * 1e3) / 1e3, label: p.label }));
  out.push(def);
}
writeFileSync("out_por20.json", JSON.stringify(out, null, 1));
console.log(`\nwrote out_por20.json: ${out.length} overlay defs, curves ${out.map(o => `${o.id}:${o.curves.map(c => c.grads.length).join("/")}`).join(" ")}`);
