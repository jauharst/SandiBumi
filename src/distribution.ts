/** Distribution summaries for depth-binned displays — box plots, whiskers, histograms.
 *
 *  Deliberately SOURCE-AGNOSTIC: everything here takes a bare array of values and knows
 *  nothing about where they came from. That is the whole point. Core plugs gathered into a
 *  depth interval, XRD sample counts, and the N realizations of a Monte Carlo array log at a
 *  single depth are the same statistical operation once the values are in hand — they differ
 *  only in how they were gathered. Both the point-data track and (later) array logs feed
 *  this module; neither owns it.
 *
 *  Mirrored in `src-tauri/src/distribution.rs` for the print/PDF exporter. KEEP THE TWO IN
 *  AGREEMENT — same percentile definition, same whisker rules — the way FACIES_PALETTE is
 *  kept in sync between plotCanvas.ts and composite.rs. The Rust side carries the unit tests
 *  that pin the numbers. */

/** How the whiskers are defined. A real interpretive choice, not cosmetics: Tukey answers
 *  "which plugs are unusual for this interval", a percentile pair answers "where do 80% of
 *  the plugs lie". Different questions, different pictures. */
export type WhiskerRule =
  | { kind: "tukey"; k: number }
  | { kind: "percentile"; lo: number; hi: number }
  | { kind: "minmax" };

export interface BoxStats {
  n: number;
  min: number;
  max: number;
  mean: number;
  /** Lower box edge (default P25). */
  lo: number;
  /** Median — always P50, whatever the box edges are set to. */
  med: number;
  /** Upper box edge (default P75). */
  hi: number;
  whiskerLo: number;
  whiskerHi: number;
  /** Samples beyond the whiskers, drawn individually. Empty for non-Tukey rules. */
  outliers: number[];
}

/** Linear-interpolated percentile of an ASCENDING-sorted array (R type 7 / NumPy default,
 *  and what Excel's PERCENTILE returns). `p` is 0–100 and is clamped. Chosen because it is
 *  what every reference a petrophysicist would check against uses — a nearest-rank
 *  definition would disagree with the same numbers computed in a spreadsheet. */
export function percentile(sorted: number[], p: number): number {
  if (sorted.length === 0) return NaN;
  if (sorted.length === 1) return sorted[0];
  const pos = (Math.min(100, Math.max(0, p)) / 100) * (sorted.length - 1);
  const lo = Math.floor(pos);
  const hi = Math.ceil(pos);
  if (lo === hi) return sorted[lo];
  return sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo);
}

/** Summarizes one set of values. Non-finite samples are dropped (never counted, never
 *  treated as zero); returns null when nothing finite is left, so the caller draws no glyph
 *  rather than an empty one at a false position. */
export function boxStats(
  values: ArrayLike<number>,
  boxLoP: number,
  boxHiP: number,
  whisker: WhiskerRule,
): BoxStats | null {
  const v: number[] = [];
  for (let i = 0; i < values.length; i++) {
    if (Number.isFinite(values[i])) v.push(values[i]);
  }
  if (v.length === 0) return null;
  v.sort((a, b) => a - b);
  const [pLo, pHi] = boxLoP <= boxHiP ? [boxLoP, boxHiP] : [boxHiP, boxLoP];
  const lo = percentile(v, pLo);
  const hi = percentile(v, pHi);
  const med = percentile(v, 50);
  const mean = v.reduce((s, x) => s + x, 0) / v.length;

  let whiskerLo: number;
  let whiskerHi: number;
  let outliers: number[] = [];
  if (whisker.kind === "minmax") {
    whiskerLo = v[0];
    whiskerHi = v[v.length - 1];
  } else if (whisker.kind === "percentile") {
    const [a, b] = whisker.lo <= whisker.hi ? [whisker.lo, whisker.hi] : [whisker.hi, whisker.lo];
    whiskerLo = percentile(v, a);
    whiskerHi = percentile(v, b);
  } else {
    // Fences from the BOX edges, whatever the user set those to — with the default 25/75
    // this is textbook Tukey; with 10/90 it is the same idea on a wider box. The whisker
    // lands on an ACTUAL sample, never on the fence value itself.
    const iqr = hi - lo;
    const fenceLo = lo - whisker.k * iqr;
    const fenceHi = hi + whisker.k * iqr;
    whiskerLo = v.find((x) => x >= fenceLo) ?? v[0];
    whiskerHi = [...v].reverse().find((x) => x <= fenceHi) ?? v[v.length - 1];
    const wl = whiskerLo;
    const wh = whiskerHi;
    outliers = v.filter((x) => x < wl || x > wh);
  }

  return { n: v.length, min: v[0], max: v[v.length - 1], mean, lo, med, hi, whiskerLo, whiskerHi, outliers };
}

/** Counts values into `bins` equal-width bins spanning [min, max]. Values outside the range
 *  are DROPPED, not clamped into the end bins — a clamped sample would invent a count the
 *  data never had at that value. Handles a reversed range (min > max), normal for porosity. */
export function histogram(values: ArrayLike<number>, min: number, max: number, bins: number): number[] {
  const nb = Math.max(1, Math.floor(bins));
  const out = new Array<number>(nb).fill(0);
  const lo = Math.min(min, max);
  const hi = Math.max(min, max);
  if (!(hi > lo)) return out;
  for (let i = 0; i < values.length; i++) {
    const x = values[i];
    if (!Number.isFinite(x) || x < lo || x > hi) continue;
    out[Math.min(nb - 1, Math.floor(((x - lo) / (hi - lo)) * nb))] += 1;
  }
  return out;
}

export interface DepthBin {
  top: number;
  base: number;
  values: number[];
}

/** Groups depth-tagged samples into equal-height depth bins. Empty bins are omitted
 *  entirely, so a sparse cored section draws no glyphs rather than a row of blanks. `bin` is
 *  in the project's depth unit; a non-positive height yields no bins. */
export function binByDepth(depth: ArrayLike<number>, value: ArrayLike<number>, bin: number): DepthBin[] {
  if (!(bin > 0)) return [];
  const n = Math.min(depth.length, value.length);
  const byKey = new Map<number, number[]>();
  for (let i = 0; i < n; i++) {
    const d = depth[i];
    const v = value[i];
    if (!Number.isFinite(d) || !Number.isFinite(v)) continue;
    const k = Math.floor(d / bin);
    const arr = byKey.get(k);
    if (arr) arr.push(v);
    else byKey.set(k, [v]);
  }
  return [...byKey.entries()]
    .sort((a, b) => a[0] - b[0])
    .map(([k, values]) => ({ top: k * bin, base: k * bin + bin, values }));
}

/** Low / median / high percentile of ONE distribution, in a single call.
 *
 *  This is what an uncertainty band is made of: at each depth an array log holds a whole set
 *  of values, and the band is three percentiles through it. Kept here rather than at the call
 *  site so the viewer and the print exporter cannot drift on how a band is derived — the same
 *  reason boxStats lives here.
 *
 *  Non-finite values are dropped. `null` means the depth had nothing finite to summarise and
 *  must be drawn as a GAP in the band, never as a zero. */
export function band(values: ArrayLike<number>, loP: number, hiP: number): { lo: number; med: number; hi: number } | null {
  const v: number[] = [];
  for (let i = 0; i < values.length; i++) if (Number.isFinite(values[i])) v.push(values[i]);
  if (v.length === 0) return null;
  v.sort((a, b) => a - b);
  const [a, b] = loP <= hiP ? [loP, hiP] : [hiP, loP];
  return { lo: percentile(v, a), med: percentile(v, 50), hi: percentile(v, b) };
}

/** Picks `want` indices spread EVENLY across `total`, for drawing a readable subset of a large
 *  set of realizations.
 *
 *  Evenly rather than the first `want`: a Latin-hypercube design lays its draws out in
 *  stratified order, so the first N are a biased corner of the sampled space and a spaghetti
 *  plot built from them would understate the true spread. */
export function evenIndices(total: number, want: number): number[] {
  if (total <= 0 || want <= 0) return [];
  if (want >= total) return Array.from({ length: total }, (_, i) => i);
  return Array.from({ length: want }, (_, k) => Math.min(total - 1, Math.floor(((k + 0.5) * total) / want)));
}
