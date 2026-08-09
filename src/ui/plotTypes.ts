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

export interface FinitePairWellInput {
  wellId: string;
  channels: Float32Array[];
}

export interface FinitePairWellAllocation {
  wellId: string;
  finitePairCount: number;
  quota: number;
  sourceIndices: number[];
}

export interface AbsentFinitePairWell {
  wellId: string;
  reason: string;
  quota: 0;
}

export interface FinitePairBudgetAllocation {
  wells: FinitePairWellAllocation[];
  absent: AbsentFinitePairWell[];
  refusal: string | null;
}

function endpointPreservingSubset(eligible: number[], count: number): number[] {
  if (count >= eligible.length) return [...eligible];
  if (count === 1) return [eligible[0]];
  return Array.from(
    { length: count },
    (_, position) => eligible[Math.floor(position * (eligible.length - 1) / (count - 1))],
  );
}

/** Screen finite aligned rows before assigning budget. Zero-pair wells receive no
 * quota and a durable reason; represented wells receive both endpoints before any
 * remaining capacity is shared in stable request order. */
export function allocateFinitePairBudget(
  inputs: FinitePairWellInput[],
  budget: number,
): FinitePairBudgetAllocation {
  const absent: AbsentFinitePairWell[] = [];
  const screened: { wellId: string; eligible: number[] }[] = [];
  for (const input of inputs) {
    const alignedLength = input.channels.length
      ? Math.min(...input.channels.map((channel) => channel.length))
      : 0;
    const eligible: number[] = [];
    for (let index = 0; index < alignedLength; index++) {
      if (input.channels.every((channel) => Number.isFinite(channel[index]))) eligible.push(index);
    }
    if (eligible.length === 0) {
      absent.push({ wellId: input.wellId, reason: "zero finite aligned pairs across required channels", quota: 0 });
    } else {
      screened.push({ wellId: input.wellId, eligible });
    }
  }
  const normalizedBudget = Math.max(0, Math.floor(budget));
  const minimumRequired = screened.reduce((sum, well) => sum + Math.min(2, well.eligible.length), 0);
  if (normalizedBudget < minimumRequired) {
    return {
      wells: [],
      absent,
      refusal: `point budget ${normalizedBudget} cannot retain both endpoints for ${screened.length} represented wells; at least ${minimumRequired} points are required`,
    };
  }
  const quotas = screened.map((well) => Math.min(2, well.eligible.length));
  let remaining = normalizedBudget - minimumRequired;
  while (remaining > 0) {
    let advanced = false;
    for (let index = 0; index < screened.length && remaining > 0; index++) {
      if (quotas[index] >= screened[index].eligible.length) continue;
      quotas[index]++;
      remaining--;
      advanced = true;
    }
    if (!advanced) break;
  }
  return {
    wells: screened.map((well, index) => ({
      wellId: well.wellId,
      finitePairCount: well.eligible.length,
      quota: quotas[index],
      sourceIndices: endpointPreservingSubset(well.eligible, quotas[index]),
    })),
    absent,
    refusal: null,
  };
}
