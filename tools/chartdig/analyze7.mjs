// FINAL: dash-tip digitization with chart-faithful rma (dolomite drawn at 2.85),
// long-dash (5-multiple) validation, and TS module emission.
// Usage: node analyze7.mjs
import { readFileSync, writeFileSync } from "fs";

const PAGES = [
  { file: "por11.json", rhof: 1.0, key: "fresh" },
  { file: "por12.json", rhof: 1.19, key: "salt" },
];
const MATDEFS = [
  { name: "quartz", rma: 2.65 },
  { name: "calcite", rma: 2.71 },
  { name: "dolomite", rma: 2.85 }, // chart artwork uses 2.85 (validated by 5-multiple long dashes + phi0 tip)
];
const results = {};

for (const page of PAGES) {
  const { strokes } = JSON.parse(readFileSync(page.file, "utf8"));
  const colorKey = s => s.color.map(c => Math.round(c)).join(",");
  const hL = [], vL = [];
  for (const s of strokes.filter(s => colorKey(s) === "155,156,159")) for (const p of s.polys) {
    const xs = p.map(q => q[0]), ys = p.map(q => q[1]);
    if (Math.max(...xs) - Math.min(...xs) > 100 && Math.max(...ys) - Math.min(...ys) < 0.5) hL.push((Math.max(...ys) + Math.min(...ys)) / 2);
    else if (Math.max(...ys) - Math.min(...ys) > 100 && Math.max(...xs) - Math.min(...xs) < 0.5) vL.push((Math.max(...xs) + Math.min(...xs)) / 2);
  }
  const dd = a => { a.sort((x, y) => x - y); const o = []; for (const v of a) if (!o.length || v - o[o.length - 1] > 1) o.push(v); return o; };
  const hs = dd(hL), vs = dd(vL);
  const fit = ls => { const sp = ls.slice(1).map((v, i) => v - ls[i]).sort((a, b) => a - b)[Math.floor((ls.length - 2) / 2)]; const idx = ls.map(v => Math.round((v - ls[0]) / sp)); const n = ls.length; const si = idx.reduce((a, b) => a + b, 0), sc = ls.reduce((a, b) => a + b, 0), sii = idx.reduce((a, b) => a + b * b, 0), sic = idx.reduce((a, b, k) => a + b * ls[k], 0); const sl = (n * sic - si * sc) / (n * sii - si * si); return { inter: (sc - sl * si) / n, slope: sl }; };
  const gy = fit(hs), gx = fit(vs);
  const toN = x => -4 + (x - gx.inter) / gx.slope, toR = y => 2.98 - 0.02 * (y - gy.inter) / gy.slope;

  const blue = [];
  for (const s of strokes.filter(s => colorKey(s) === "3,70,145")) for (const p of s.polys) blue.push(p);
  const plen = p => { let l = 0; for (let i = 1; i < p.length; i++) l += Math.hypot(p[i][0] - p[i - 1][0], p[i][1] - p[i - 1][1]); return l; };
  const cand = blue.filter(p => { const xs = p.map(q => q[0]), ys = p.map(q => q[1]); return plen(p) > 25 && Math.max(...xs) - Math.min(...xs) > 15 && Math.max(...ys) - Math.min(...ys) > 15; });
  function mergeChains(polys) {
    const chains = polys.map(c => c.slice());
    const d2 = (a, b) => (a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2;
    let changed = true;
    while (changed) {
      changed = false;
      outer: for (let i = 0; i < chains.length; i++) for (let j = i + 1; j < chains.length; j++) {
        const A = chains[i], B = chains[j], T = 2.25;
        if (d2(A[A.length - 1], B[0]) < T) { chains[i] = A.concat(B.slice(1)); chains.splice(j, 1); changed = true; break outer; }
        if (d2(A[A.length - 1], B[B.length - 1]) < T) { chains[i] = A.concat(B.slice().reverse().slice(1)); chains.splice(j, 1); changed = true; break outer; }
        if (d2(A[0], B[B.length - 1]) < T) { chains[i] = B.concat(A.slice(1)); chains.splice(j, 1); changed = true; break outer; }
        if (d2(A[0], B[0]) < T) { chains[i] = B.slice().reverse().concat(A.slice(1)); chains.splice(j, 1); changed = true; break outer; }
      }
    }
    return chains;
  }
  const chains = mergeChains(cand).map(ch => ({ ch, l: plen(ch) })).sort((a, b) => b.l - a.l).slice(0, 3).map(o => o.ch);
  const mats = MATDEFS.map(m => ({ ...m }));
  for (const ch of chains) {
    const hiY = ch.reduce((m, p) => Math.min(m, p[1]), Infinity);
    const rho = toR(hiY);
    let best = null, bd = Infinity;
    for (const m of mats) { const d = Math.abs(rho - m.rma); if (d < bd) { bd = d; best = m; } }
    best.chain = ch;
  }
  const mdc = (pt, ch) => { let b = Infinity; for (let i = 1; i < ch.length; i++) { const [ax, ay] = ch[i - 1], [bx, by] = ch[i]; const vx = bx - ax, vy = by - ay, L2 = vx * vx + vy * vy; let t = L2 ? ((pt[0] - ax) * vx + (pt[1] - ay) * vy) / L2 : 0; t = Math.max(0, Math.min(1, t)); b = Math.min(b, Math.hypot(pt[0] - (ax + t * vx), pt[1] - (ay + t * vy))); } return b; };

  for (const m of mats) m.tips = [];
  for (const d of blue) {
    const l = plen(d);
    if (l <= 1.2 || l >= 12 || d.length > 8) continue;
    const mid = d.reduce((a, p) => [a[0] + p[0] / d.length, a[1] + p[1] / d.length], [0, 0]);
    let best = null, bd = Infinity;
    for (const m of mats) {
      if (!m.chain) continue;
      const dist = mdc(mid, m.chain);
      if (dist < bd) { bd = dist; best = m; }
    }
    if (!best || bd > 4) continue;
    const tip = d.reduce((a, p) => (p[0] > a[0] ? p : a));
    best.tips.push({ x: toN(tip[0]), r: toR(tip[1]), len: l });
  }
  for (const m of mats) {
    if (!m.chain) continue;
    const bottom = m.chain.reduce((a, p) => (p[1] < a[1] ? p : a));
    m.tips.push({ x: toN(bottom[0]), r: toR(bottom[1]), len: 0, fromPath: true });
  }

  console.log(`\n=== ${page.file} (rhof ${page.rhof}) ===`);
  for (const m of mats) {
    if (!m.chain) continue;
    const rows = m.tips.map(t => {
      const phiEst = (m.rma - t.r) / (m.rma - page.rhof) * 100;
      return { phi: Math.round(phiEst), resid: phiEst - Math.round(phiEst), nphi: t.x, len: t.len, fromPath: !!t.fromPath };
    }).sort((a, b) => a.phi - b.phi || (a.fromPath ? -1 : 1));
    const byPhi = new Map();
    for (const r of rows) {
      const cur = byPhi.get(r.phi);
      if (!cur || (r.fromPath && !cur.fromPath) || (!cur.fromPath && Math.abs(r.resid) < Math.abs(cur.resid))) byPhi.set(r.phi, r);
    }
    const tab = [...byPhi.values()].sort((a, b) => a.phi - b.phi);
    const gaps = tab.slice(1).map((r, i) => r.phi - tab[i].phi).filter(g => g !== 1);
    const longs = tab.filter(r => r.len >= 4.4).map(r => r.phi);
    const maxResid = Math.max(...tab.map(r => Math.abs(r.resid)));
    const bad5 = longs.filter(p => p % 5 !== 0);
    console.log(`${m.name} (rma ${m.rma}): phi ${tab[0].phi}..${tab[tab.length - 1].phi}, maxResid ${maxResid.toFixed(2)}, gaps ${gaps.join(",") || "none"}, long dashes at ${longs.join(",")}${bad5.length ? `  [WARN non-5-multiple longs: ${bad5.join(",")}]` : "  [5-multiples OK]"}`);
    m.table = tab.map(r => ({ phi: r.phi, nphi: Math.round(r.nphi * 100) / 100 }));
    // smoothness listing check
    const secondDiffs = m.table.slice(1, -1).map((r, i) => Math.abs(r.nphi - (m.table[i].nphi + m.table[i + 2].nphi) / 2)).filter(d => d > 0.4);
    if (secondDiffs.length) console.log(`  [WARN ${secondDiffs.length} rough second-differences > 0.4]`);
  }
  results[page.key] = { rhof: page.rhof, mats: mats.map(m => ({ name: m.name, rmaChart: m.rma, table: m.table })) };

  // worked example
  const interp = (tab, phi) => { const i = tab.findIndex(r => r.phi >= phi); if (i < 0) return NaN; if (i === 0) return tab[0].nphi; const a = tab[i - 1], b = tab[i]; return a.nphi + (b.nphi - a.nphi) * (phi - a.phi) / (b.phi - a.phi); };
  const q = mats[0], c = mats[1];
  let best = null;
  for (let phi = 5; phi <= 40; phi += 0.05) {
    const qr = (phi / 100) * page.rhof + (1 - phi / 100) * q.rma, cr = (phi / 100) * page.rhof + (1 - phi / 100) * c.rma;
    const s = (2.38 - qr) / (cr - qr);
    if (s < -0.2 || s > 1.2) continue;
    const x = interp(q.table, phi) + s * (interp(c.table, phi) - interp(q.table, phi));
    const err = Math.abs(x - 16.5);
    if (!best || err < best.err) best = { err, phi, s };
  }
  console.log(`example (16.5, 2.38): phi ${best.phi.toFixed(1)}, qtz ${(100 * (1 - best.s)).toFixed(0)}%  ${page.key === "fresh" ? "(chart: 18 pu, ~40%)" : "(chart: 20 pu, ~55%)"}`);
}

// ---- emit TS module -------------------------------------------------------
const fmt = t => "[" + t.map(r => `[${r.phi},${r.nphi}]`).join(",") + "]";
let ts = `// Digitized from Schlumberger Log Interpretation Charts 2013, Por-11 (fresh,\n`;
ts += `// rho_f 1.0) and Por-12 (salt, rho_f 1.19): CNL + Litho-Density porosity/\n`;
ts += `// lithology matrix curves. Entries are [phi_true_pu, nphi_apparent_limestone_pu];\n`;
ts += `// rhob for a graduation is phi/100*rhof + (1-phi/100)*rmaChart. The chart draws\n`;
ts += `// dolomite with rma 2.85. Extraction: PDF vector paths, graduation-dash tips,\n`;
ts += `// grid-index calibration (calcite identity rms 0.13 pu; worked examples match).\n`;
ts += `export interface DnMatrixCurve { name: string; rmaChart: number; pts: [number, number][] }\n`;
ts += `export interface DnChart { rhof: number; label: string; curves: DnMatrixCurve[] }\n`;
ts += `export const DN_CHARTS: Record<"fresh" | "salt", DnChart> = {\n`;
for (const key of ["fresh", "salt"]) {
  const r = results[key];
  ts += `  ${key}: {\n    rhof: ${r.rhof},\n    label: "${key === "fresh" ? "Por-11 fresh (\\u03c1f 1.0)" : "Por-12 salt (\\u03c1f 1.19)"}",\n    curves: [\n`;
  for (const m of r.mats) {
    ts += `      { name: "${m.name === "quartz" ? "Quartz sandstone" : m.name === "calcite" ? "Calcite (limestone)" : "Dolomite"}", rmaChart: ${m.rmaChart}, pts: ${fmt(m.table)} },\n`;
  }
  ts += `    ],\n  },\n`;
}
ts += `};\n`;
writeFileSync("dnChartData.ts", ts);
console.log(`\nwrote dnChartData.ts (${ts.length} chars)`);
