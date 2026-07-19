// Config-driven generator for the density-neutron-family charts (y = rhob is
// analytic per graduation; x is the empirical tool response read from dash tips).
// Covers Por-11/12 (CNL), Por-13/14 (APS, APLC+FPLC), Por-16 (adnVISION675),
// Por-18/19 (EcoScope BPHI/TNPH), Lith-3/4 (PEF).  node gen_dn.mjs
import { writeFileSync } from "fs";
import {
  loadPage, gridLines, gridFit, anchorAxis, curveChains, dashTipsMulti, chainExtreme,
  validateRows, polyLen, coloredPolys, mergeChains, minDistToChain,
} from "./lib.mjs";

const NEUTRON = ["NPHI", "TNPH", "NPOR", "PHIN", "CNC", "BPHI", "APLC", "FPLC"];
const DENSITY = ["RHOB", "RHOZ", "DEN", "ROBB"];
const PEF = ["PEF", "PE", "PEFZ"];

// Matrix rma candidate sets: the artwork sometimes uses a different rma than the
// textbook value (Por-11/12 dolomite is drawn at 2.85) â€” score candidates by how
// well long dashes land on 5-multiples and graduation estimates on integers.
const QTZ = { name: "Quartz sandstone", rmaCands: [2.65] };
const CAL = { name: "Calcite (limestone)", rmaCands: [2.71] };
const DOL = { name: "Dolomite", rmaCands: [2.87, 2.85] };

const CHARTS = [
  { page: "por11.json", id: "por11", label: "Por-11 CNL (fresh, rhof 1.0)", rhof: 1.0, xKind: "neutron", mode: "family", families: [{ color: "3,70,145", suffix: "" }] },
  { page: "por12.json", id: "por12", label: "Por-12 CNL (salt, rhof 1.19)", rhof: 1.19, xKind: "neutron", mode: "family", families: [{ color: "3,70,145", suffix: "" }] },
  { page: "p239.json", id: "por13", label: "Por-13 APS (fresh)", rhof: 1.0, xKind: "neutron", mode: "family", families: [{ color: "3,70,145", suffix: "_aplc", tag: " APLC" }, { color: "255,64,58", suffix: "_fplc", tag: " FPLC" }], famMats: [QTZ, DOL], sharedCalcite: "44,46,53" },
  { page: "p240.json", id: "por14", label: "Por-14 APS (salt, rhof 1.19)", rhof: 1.19, xKind: "neutron", mode: "family", families: [{ color: "3,70,145", suffix: "_aplc", tag: " APLC" }, { color: "255,64,58", suffix: "_fplc", tag: " FPLC" }], famMats: [QTZ, DOL], sharedCalcite: "44,46,53" },
  { page: "p242.json", id: "por16", label: "Por-16 adnVISION675 (fresh)", rhof: 1.0, xKind: "neutron", mode: "matrix", matrices: [{ ...QTZ, color: "255,64,58" }, { ...CAL, color: "44,46,53" }, { ...DOL, color: "3,70,145" }] },
  { page: "p244.json", id: "por18", label: "Por-18 EcoScope BPHI (fresh)", rhof: 1.0, xKind: "neutron", mode: "matrix", matrices: [{ ...QTZ, color: "255,46,23" }, { ...CAL, color: "44,46,53" }, { ...DOL, color: "1,66,130" }] },
  { page: "p245.json", id: "por19", label: "Por-19 EcoScope TNPH (fresh)", rhof: 1.0, xKind: "neutron", mode: "matrix", matrices: [{ ...QTZ, color: "255,46,23" }, { ...CAL, color: "40,38,44" }, { ...DOL, color: "1,66,130" }] },
  { page: "p208.json", id: "lith3", label: "Lith-3 PEF (fresh, rhof 1.0)", rhof: 1.0, xKind: "pef", mode: "family", families: [{ color: "3,70,145", suffix: "" }], labelEvery: 10, yStep: 0.01, chainMinLen: 12.001 },
  { page: "p209.json", id: "lith4", label: "Lith-4 PEF (salt, rhof 1.1)", rhof: 1.1, xKind: "pef", mode: "family", families: [{ color: "3,70,145", suffix: "" }], labelEvery: 10, yStep: 0.01, chainMinLen: 12.001 },
];

const out = [];

for (const cfg of CHARTS) {
  console.log(`\n=== ${cfg.id} (${cfg.page}) ===`);
  const { strokes, texts } = loadPage(cfg.page);
  const { hs, vs } = gridLines(strokes);
  const gy = gridFit(hs), gx = gridFit(vs);
  console.log(`grid: ${vs.length}v (${gx.slope.toFixed(3)}pt, rms ${gx.rms.toFixed(3)}) Ã— ${hs.length}h (${gy.slope.toFixed(3)}pt, rms ${gy.rms.toFixed(3)})`);

  // y axis: rhob labels 1.8-3.0; minor grid 0.02 g/cc per line (0.01 on Lith-3/4)
  const yStep = cfg.yStep ?? 0.02;
  const ya = anchorAxis(texts, gy, { labelRe: /^[123]\.\d$/, step: -yStep, side: "y", edge: vs[0] });
  // x axis: neutron pu labels every 5 lines of 1 pu, or Pe labels (probe both spacings)
  let xa, xStep;
  if (cfg.xKind === "neutron") {
    xStep = 1;
    xa = anchorAxis(texts, gx, { labelRe: /^(-5|0|5|10|15|20|25|30|35|40|45)$/, step: 1, side: "x", edge: hs[0] });
  } else {
    // Pe axis: minor spacing = (label span)/(line count-1); labels 0..6
    xStep = null; // decide from votes below
    for (const st of [0.1, 0.2, 0.25, 0.5]) {
      const cand = anchorAxis(texts, gx, { labelRe: /^[0-7]$/, step: st, side: "x", edge: hs[0] });
      const top = Math.max(...Object.values(cand.votes));
      if (top >= cand.nLabels - 1 && cand.nLabels >= 4) { xa = cand; xStep = st; break; }
    }
    if (!xa) throw new Error("Pe axis step not resolved");
    console.log(`pe axis step ${xStep}`);
  }
  console.log(`anchors: x base ${xa.base} votes ${JSON.stringify(xa.votes)} | y base ${ya.base} votes ${JSON.stringify(ya.votes)}`);
  const toX = px => xa.base + (px - gx.inter) / gx.slope * xStep;
  const toRhob = py => ya.base - yStep * (py - gy.inter) / gy.slope;
  const frame = { x0: vs[0], x1: vs[vs.length - 1], y0: hs[0], y1: hs[hs.length - 1] };

  // greedy matrix<-chain assignment: per matrix, the LONGEST curve chain whose
  // bottom-end rhob is within 0.05 of the matrix rma (junk arrows never match).
  // Excluded: thin axis-aligned rules (bbox < 3pt one way) and big axis-aligned
  // rectangles (frames) — but NOT near-vertical curves (e.g. Lith-4 quartz),
  // whose per-segment dx is tiny yet whose bbox width is several pt.
  const diagonalChains = color => curveChains(strokes, color, frame, cfg.chainMinLen ?? 13).filter(({ ch }) => {
    const xs = ch.map(q => q[0]), ys = ch.map(q => q[1]);
    const w = Math.max(...xs) - Math.min(...xs), h = Math.max(...ys) - Math.min(...ys);
    let axisLen = 0, total = 0;
    for (let i = 1; i < ch.length; i++) {
      const dx = Math.abs(ch[i][0] - ch[i - 1][0]), dy = Math.abs(ch[i][1] - ch[i - 1][1]);
      const l = Math.hypot(dx, dy);
      total += l;
      if (dx < 0.1 || dy < 0.1) axisLen += l;
    }
    const axisFrac = total ? axisLen / total : 1;
    if (axisFrac > 0.95 && (w < 3 || h < 3 || (w > 50 && h > 50))) return false;
    return true;
  });
  const pickChain = (chains, rma) => {
    for (const { ch } of chains) {
      const rho = toRhob(chainExtreme(ch, "minY")[1]);
      if (Math.abs(rho - rma) <= 0.05) return ch;
    }
    return undefined;
  };
  const jobs = []; // {suffix, tag, matrices: [{name, rmaCands, chain, color, key}]}
  let sharedCal = null;
  if (cfg.sharedCalcite) {
    const chains = diagonalChains(cfg.sharedCalcite);
    sharedCal = { ...CAL, color: cfg.sharedCalcite, chain: pickChain(chains, CAL.rmaCands[0]), key: "shared|CAL" };
  }
  if (cfg.mode === "family") {
    for (const fam of cfg.families) {
      const chains = diagonalChains(fam.color);
      const mats = (cfg.famMats ?? [QTZ, CAL, DOL]).map(m => ({
        ...m, color: fam.color, chain: pickChain(chains, m.rmaCands[0]), key: `${fam.suffix}|${m.name}`,
      }));
      if (sharedCal) mats.splice(1, 0, sharedCal);
      jobs.push({ suffix: fam.suffix, tag: fam.tag || "", matrices: mats });
    }
  } else {
    const mats = cfg.matrices.map(m => ({
      ...m, chain: pickChain(diagonalChains(m.color), m.rmaCands[0]), key: `|${m.name}`,
    }));
    jobs.push({ suffix: "", tag: "", matrices: mats });
  }

  // page-wide dash assignment: every dash goes to its nearest matrix chain
  const targets = [];
  for (const job of jobs) for (const m of job.matrices) {
    if (m.chain && !targets.some(t => t.key === m.key)) targets.push({ color: m.color, chain: m.chain, key: m.key });
  }
  const tipMap = dashTipsMulti(strokes, targets, { tipSide: "maxX", lenMax: 16, maxDist: 6 });

  for (const job of jobs) {
    const curves = [];
    for (const m of job.matrices) {
      if (!m.chain) { console.log(`${m.name}: NO CHAIN`); continue; }
      const tips = tipMap.get(m.key) ?? [];
      const lens = tips.map(t => t.len).sort((a, b) => a - b);
      const medLen = lens[Math.floor(lens.length / 2)] || 3;
      const bottom = chainExtreme(m.chain, "minY");
      // entries sorted bottom-up; the path tip IS the lowest graduation (drawn
      // into the path) â€” drop a separate dash that duplicates it.
      const entries = [{ x: toX(bottom[0]), r: toRhob(bottom[1]), len: 0, fromPath: true }];
      for (const t of tips) {
        const x = toX(t.tip[0]), r = toRhob(t.tip[1]);
        if (Math.abs(r - entries[0].r) > 0.004 || Math.abs(x - entries[0].x) > 0.6) entries.push({ x, r, len: t.len });
      }
      entries.sort((a, b) => b.r - a.r);
      const scoreRows = rows => {
        const badLongs = rows.filter(r => r.long && r.t % 5 !== 0).length;
        const rms = Math.sqrt(rows.reduce((s, r) => s + r.resid * r.resid, 0) / rows.length);
        const gaps = rows.slice(1).filter((r, i) => r.t - rows[i].t !== 1).length;
        return { badLongs, rms, gaps, score: badLongs * 10 + gaps * 3 + rms };
      };
      let best = null;
      for (const rma of m.rmaCands) {
        const phiOf = r => (rma - r) / (rma - cfg.rhof) * 100;
        // (a) independent rounding of each dash's analytic estimate
        const byT = new Map();
        for (const e of entries) {
          const est = phiOf(e.r);
          const row = { t: Math.round(est), resid: est - Math.round(est), x: e.x, long: e.len >= medLen * 1.3, fromPath: e.fromPath };
          const cur = byT.get(row.t);
          if (!cur || (row.fromPath && !cur.fromPath) || (!cur.fromPath && Math.abs(row.resid) < Math.abs(cur.resid))) byT.set(row.t, row);
        }
        const roundRows = [...byT.values()].sort((a, b) => a.t - b.t);
        // (b) sequential: consecutive dashes along the curve are consecutive
        //     graduations, anchored at the bottom estimate (immune to mid-curve
        //     artwork drift that flips independent rounding)
        const t0 = Math.round(phiOf(entries[0].r));
        const seqRows = entries.map((e, i) => ({ t: t0 + i, resid: phiOf(e.r) - (t0 + i), x: e.x, long: e.len >= medLen * 1.3 }));
        for (const [tag, rows] of [["round", roundRows], ["seq", seqRows]]) {
          const sc = scoreRows(rows);
          if (!best || sc.score < best.score) best = { rma, rows, method: tag, ...sc };
        }
      }
      console.log(`${m.name}${job.tag}: rma ${best.rma} via ${best.method} (rms ${best.rms.toFixed(2)}, badLongs ${best.badLongs}, gaps ${best.gaps}), ${best.rows.length} grads`);
      // outlier rejection on x (second differences), then strict validation
      let tab = best.rows;
      let changed = true;
      while (changed) {
        changed = false;
        for (let i = 1; i < tab.length - 1; i++) {
          const a = tab[i - 1], b = tab[i + 1];
          const pred = a.x + (b.x - a.x) * (tab[i].t - a.t) / (b.t - a.t);
          if (Math.abs(tab[i].x - pred) > (cfg.xKind === "pef" ? 0.12 : 0.5)) {
            console.log(`  drop outlier t=${tab[i].t} x=${tab[i].x.toFixed(2)} (pred ${pred.toFixed(2)})`);
            tab.splice(i, 1); changed = true; break;
          }
        }
      }
      // fill interior gaps by interpolation (a dropped/undetected dash between
      // valid neighbors â€” accurate to ~0.05 units at 1-graduation spacing)
      const filledTab = [];
      for (let i = 0; i < tab.length; i++) {
        filledTab.push(tab[i]);
        if (i + 1 < tab.length && tab[i + 1].t - tab[i].t > 1 && tab[i + 1].t - tab[i].t <= 3) {
          for (let t = tab[i].t + 1; t < tab[i + 1].t; t++) {
            const f = (t - tab[i].t) / (tab[i + 1].t - tab[i].t);
            filledTab.push({ t, x: tab[i].x + f * (tab[i + 1].x - tab[i].x), long: false, filled: true });
          }
        }
      }
      filledTab.sort((a, b) => a.t - b.t);
      tab = filledTab;
      validateRows(tab.map(r => ({ t: r.t, long: r.long })), { name: `${m.name}${job.tag}`, strict: false });
      const rma = best.rma;
      const rhobOf = t => (t / 100) * cfg.rhof + (1 - t / 100) * rma;
      const xScale = cfg.xKind === "neutron" ? 0.01 : 1; // pu -> v/v
      curves.push({
        name: m.name,
        rmaChart: rma,
        labelEvery: cfg.labelEvery || 5,
        grads: tab.map(r => [r.t, Math.round(r.x * xScale * 1e4) / 1e4, Math.round(rhobOf(r.t) * 1e4) / 1e4]),
      });
    }
    out.push({
      id: cfg.id + job.suffix,
      label: cfg.label + (job.tag || ""),
      xAxis: cfg.xKind === "neutron" ? "neutron" : "pef",
      yAxis: "density",
      curves,
      isoConnect: true,
    });
  }
}

writeFileSync("out_dn_family.json", JSON.stringify(out, null, 1));
console.log(`\nwrote out_dn_family.json: ${out.length} overlay defs`);
