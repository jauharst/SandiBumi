// Shared chartbook-digitization functions (method proven on Por-11/12; see
// D:\XX. SandiBumi\tools\chartdig\README.md). All 2013-edition charts share the
// style: gray grid (155,156,159), graduation dashes whose TIP marks the exact
// data coordinate, label-anchored grid-index calibration.
import { readFileSync } from "fs";

export function loadPage(file) {
  const { strokes, texts } = JSON.parse(readFileSync(file, "utf8"));
  return { strokes, texts };
}
export const colorKey = s => s.color.map(c => Math.round(c)).join(",");

export function gridLines(strokes, gridColor = "155,156,159", minSpan = 100) {
  // keep only lines spanning ~the full plot (legend-box borders etc. are shorter)
  const hRaw = [], vRaw = [];
  for (const s of strokes.filter(s => colorKey(s) === gridColor)) for (const p of s.polys) {
    const xs = p.map(q => q[0]), ys = p.map(q => q[1]);
    const w = Math.max(...xs) - Math.min(...xs), h = Math.max(...ys) - Math.min(...ys);
    if (w > minSpan && h < 0.5) hRaw.push({ c: (Math.max(...ys) + Math.min(...ys)) / 2, span: w });
    else if (h > minSpan && w < 0.5) vRaw.push({ c: (Math.max(...xs) + Math.min(...xs)) / 2, span: h });
  }
  const filt = raw => {
    const maxSpan = Math.max(...raw.map(r => r.span), 0);
    return raw.filter(r => r.span >= 0.85 * maxSpan).map(r => r.c);
  };
  const dd = a => { a.sort((x, y) => x - y); const o = []; for (const v of a) if (!o.length || v - o[o.length - 1] > 1) o.push(v); return o; };
  return { hs: dd(filt(hRaw)), vs: dd(filt(vRaw)) };
}

export function gridFit(lines) {
  // sequential position IS the grid index (the artwork's spacing can wobble a
  // little — spacing-derived indices drift off by a full step mid-grid); warn if
  // a line looks missing (a diff far above the median).
  const diffs = lines.slice(1).map((v, i) => v - lines[i]);
  const sortedDiffs = [...diffs].sort((a, b) => a - b);
  const med = sortedDiffs[Math.floor(sortedDiffs.length / 2)];
  const missing = diffs.filter(d => d > 1.6 * med).length;
  if (missing) console.log(`  [WARN gridFit: ${missing} gap(s) > 1.6x median spacing — indices may shift]`);
  const idx = lines.map((_, i) => i);
  const n = lines.length;
  const si = idx.reduce((a, b) => a + b, 0), sc = lines.reduce((a, b) => a + b, 0);
  const sii = idx.reduce((a, b) => a + b * b, 0), sic = idx.reduce((a, b, k) => a + b * lines[k], 0);
  const slope = (n * sic - si * sc) / (n * sii - si * si), inter = (sc - slope * si) / n;
  const rms = Math.sqrt(lines.reduce((s2, v, k) => s2 + (inter + slope * idx[k] - v) ** 2, 0) / n);
  return { inter, slope, n, rms };
}

const num = t => parseFloat(t.s.replace("–", "-").replace("−", "-"));

/** Anchor the integer grid using width-centered labels for X (below the grid) or
 *  baseline labels for Y (left of the grid). step = data increment per grid line
 *  (negative when the value decreases with the coordinate). Returns the data value
 *  of grid index 0 plus the vote tally for reporting. */
export function anchorAxis(texts, g, { labelRe, step, side, edge }) {
  const labels = texts
    .filter(t => labelRe.test(t.s.trim()))
    .filter(t => (side === "x" ? t.y < edge - 3 : t.x < edge - 5))
    .map(t => ({ v: num(t), c: side === "x" ? t.x + (t.w || 0) / 2 : t.y }));
  const ests = labels.map(l => ({ v: l.v, fi: (l.c - g.inter) / g.slope }));
  const fracs = ests.map(e => e.fi - Math.round(e.fi)).sort((a, b) => a - b);
  const medFrac = fracs[Math.floor(fracs.length / 2)];
  const counts = {};
  for (const e of ests) {
    const b = e.v - step * Math.round(e.fi - medFrac);
    const key = Math.round(b * 1e6) / 1e6;
    counts[key] = (counts[key] || 0) + 1;
  }
  const entries = Object.entries(counts).sort((a, b) => b[1] - a[1]);
  return { base: parseFloat(entries[0][0]), votes: counts, nLabels: labels.length };
}

export const polyLen = p => { let l = 0; for (let i = 1; i < p.length; i++) l += Math.hypot(p[i][0] - p[i - 1][0], p[i][1] - p[i - 1][1]); return l; };

export function coloredPolys(strokes, color) {
  const out = [];
  for (const s of strokes.filter(s => colorKey(s) === color)) for (const p of s.polys) out.push(p);
  return out;
}

export function mergeChains(polys, tol2 = 2.25) {
  const chains = polys.map(c => c.slice());
  const d2 = (a, b) => (a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2;
  let changed = true;
  while (changed) {
    changed = false;
    outer: for (let i = 0; i < chains.length; i++) for (let j = i + 1; j < chains.length; j++) {
      const A = chains[i], B = chains[j];
      if (d2(A[A.length - 1], B[0]) < tol2) { chains[i] = A.concat(B.slice(1)); chains.splice(j, 1); changed = true; break outer; }
      if (d2(A[A.length - 1], B[B.length - 1]) < tol2) { chains[i] = A.concat(B.slice().reverse().slice(1)); chains.splice(j, 1); changed = true; break outer; }
      if (d2(A[0], B[B.length - 1]) < tol2) { chains[i] = B.concat(A.slice(1)); chains.splice(j, 1); changed = true; break outer; }
      if (d2(A[0], B[0]) < tol2) { chains[i] = B.slice().reverse().concat(A.slice(1)); chains.splice(j, 1); changed = true; break outer; }
    }
  }
  return chains;
}

export function minDistToChain(pt, ch) {
  let best = Infinity;
  for (let i = 1; i < ch.length; i++) {
    const [ax, ay] = ch[i - 1], [bx, by] = ch[i];
    const vx = bx - ax, vy = by - ay, L2 = vx * vx + vy * vy;
    let t = L2 ? ((pt[0] - ax) * vx + (pt[1] - ay) * vy) / L2 : 0;
    t = Math.max(0, Math.min(1, t));
    best = Math.min(best, Math.hypot(pt[0] - (ax + t * vx), pt[1] - (ay + t * vy)));
  }
  return best;
}

/** Long curve chains of a color, filtered to the plot frame area, sorted by length. */
export function curveChains(strokes, color, frame, minLen = 13) {
  const polys = coloredPolys(strokes, color).filter(p => {
    const xs = p.map(q => q[0]), ys = p.map(q => q[1]);
    const inside = p.every(q => q[0] > frame.x0 - 30 && q[0] < frame.x1 + 30 && q[1] > frame.y0 - 30 && q[1] < frame.y1 + 30);
    return inside && polyLen(p) > minLen && (Math.max(...xs) - Math.min(...xs) > 10 || Math.max(...ys) - Math.min(...ys) > 10);
  });
  return mergeChains(polys).map(ch => ({ ch, len: polyLen(ch) })).sort((a, b) => b.len - a.len);
}

/** Short dash strokes assigned to the NEAREST of several chains (each dash claimed
 *  once, page-wide — close-bunched curves must compete for their dashes). targets:
 *  [{color, chain, key}]. Returns Map key -> tips. tipSide: which endpoint is the
 *  data tip in device coords (y up). */
export function dashTipsMulti(strokes, targets, { tipSide = "maxX", maxDist = 4, lenMin = 1.2, lenMax = 12 } = {}) {
  const byKey = new Map(targets.map(t => [t.key, []]));
  const colors = [...new Set(targets.map(t => t.color))];
  const pick = {
    maxX: (a, p) => (p[0] > a[0] ? p : a),
    minX: (a, p) => (p[0] < a[0] ? p : a),
    maxY: (a, p) => (p[1] > a[1] ? p : a),
    minY: (a, p) => (p[1] < a[1] ? p : a),
  }[tipSide];
  for (const color of colors) {
    for (const d of coloredPolys(strokes, color)) {
      const l = polyLen(d);
      if (l <= lenMin || l >= lenMax || d.length > 8) continue;
      const mid = d.reduce((a, p) => [a[0] + p[0] / d.length, a[1] + p[1] / d.length], [0, 0]);
      let best = null, bd = Infinity;
      for (const t of targets) {
        if (t.color !== color || !t.chain) continue;
        const dist = minDistToChain(mid, t.chain);
        if (dist < bd) { bd = dist; best = t; }
      }
      if (best && bd <= maxDist) byKey.get(best.key).push({ tip: d.reduce(pick), len: l, mid });
    }
  }
  return byKey;
}

/** The chain endpoint-region point with extreme device y (the phi-0 dash drawn
 *  into the path). dir "minY" = bottom of page (max data-y when y-axis inverted). */
export function chainExtreme(chain, dir = "minY") {
  return chain.reduce((a, p) => ((dir === "minY" ? p[1] < a[1] : p[1] > a[1]) ? p : a));
}

/** Validation: given rows [{t, ...}] where long dashes are flagged, check long
 *  dashes sit on multiples of `mult`, gaps are 1, and report. Throws on failure
 *  when strict. */
export function validateRows(rows, { name, mult = 5, strict = true }) {
  const gaps = rows.slice(1).map((r, i) => r.t - rows[i].t).filter(g => g !== 1);
  const longs = rows.filter(r => r.long).map(r => r.t);
  const badLongs = longs.filter(t => t % mult !== 0);
  const msg = `${name}: t ${rows[0]?.t}..${rows[rows.length - 1]?.t}, gaps ${gaps.join(",") || "none"}, longs ${longs.join(",")}${badLongs.length ? ` BAD:${badLongs.join(",")}` : " OK"}`;
  console.log(msg);
  if (strict && (gaps.length || badLongs.length)) throw new Error(`VALIDATION FAILED ${msg}`);
  return { gaps, badLongs };
}
