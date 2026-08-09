export interface PercentileP {
  kind: "percentile_p";
  value: number;
}

export interface RangePositionPct {
  kind: "range_position_pct";
  value: number;
}

export function parsePercentileP(value: number): PercentileP {
  if (!Number.isFinite(value) || value < 0 || value > 100) {
    throw new RangeError("PercentileP must be finite and inside [0,100]");
  }
  return { kind: "percentile_p", value };
}

/** Range position is deliberately not clamped: negative values and values above
 * 100 describe extrapolation beyond the chosen range and must survive templates/export. */
export function parseRangePositionPct(value: number): RangePositionPct {
  if (!Number.isFinite(value)) throw new RangeError("RangePositionPct must be finite");
  return { kind: "range_position_pct", value };
}

export type PlotChannelPolicy = "cartesian" | "colour" | "array_waveform";
export type PlotRangeEdge = "none" | "low" | "high";

export interface PlotDisplayRange {
  min: number;
  max: number;
}

export interface PlotChannelPolicyReport {
  /** Derived display values. The caller's source array is never mutated. */
  values: Float32Array;
  /** Zero means the sample is excluded for this channel. */
  included: Uint8Array;
  edgeMarks: PlotRangeEdge[];
  nonFiniteExcluded: number;
  logDomainExcluded: number;
  displayClipped: number;
  clamped: number;
}

/** Shared per-channel out-of-range contract from SB-PLT-013. Cartesian values retain
 * their source value and are clipped by the viewport. Colour and waveform values are
 * clamped only in the derived display vector, with colour-edge metadata retained. */
export function applyPlotChannelPolicy(
  source: Float32Array,
  policy: PlotChannelPolicy,
  display: PlotDisplayRange,
  logAxis = false,
): PlotChannelPolicyReport {
  const values = source.slice();
  const included = new Uint8Array(source.length);
  const edgeMarks: PlotRangeEdge[] = new Array(source.length).fill("none");
  let nonFiniteExcluded = 0;
  let logDomainExcluded = 0;
  let displayClipped = 0;
  let clamped = 0;
  const low = Math.min(display.min, display.max);
  const high = Math.max(display.min, display.max);
  for (let index = 0; index < source.length; index++) {
    const value = source[index];
    if (!Number.isFinite(value)) {
      nonFiniteExcluded++;
      continue;
    }
    if (logAxis && value <= 0) {
      logDomainExcluded++;
      continue;
    }
    included[index] = 1;
    if (value < low) {
      if (policy === "cartesian") displayClipped++;
      else {
        values[index] = low;
        edgeMarks[index] = "low";
        clamped++;
      }
    } else if (value > high) {
      if (policy === "cartesian") displayClipped++;
      else {
        values[index] = high;
        edgeMarks[index] = "high";
        clamped++;
      }
    }
  }
  return {
    values,
    included,
    edgeMarks,
    nonFiniteExcluded,
    logDomainExcluded,
    displayClipped,
    clamped,
  };
}
