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
export type PlotChannelExclusion = "none" | "non_finite" | "log_domain";

export interface PlotDisplayRange {
  min: number;
  max: number;
}

export interface PlotChannelPolicyReport {
  /** Derived display values. The caller's source array is never mutated. */
  values: Float32Array;
  /** Zero means the sample is excluded for this channel. */
  included: Uint8Array;
  exclusionReasons: PlotChannelExclusion[];
  /** One means the finite, domain-valid source value lies outside the display range. */
  displayOverflow: Uint8Array;
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
  display: PlotDisplayRange | null,
  logAxis = false,
): PlotChannelPolicyReport {
  const values = source.slice();
  const included = new Uint8Array(source.length);
  const exclusionReasons: PlotChannelExclusion[] = new Array(source.length).fill("none");
  const displayOverflow = new Uint8Array(source.length);
  const edgeMarks: PlotRangeEdge[] = new Array(source.length).fill("none");
  let nonFiniteExcluded = 0;
  let logDomainExcluded = 0;
  let displayClipped = 0;
  let clamped = 0;
  const low = display ? Math.min(display.min, display.max) : Number.NEGATIVE_INFINITY;
  const high = display ? Math.max(display.min, display.max) : Number.POSITIVE_INFINITY;
  for (let index = 0; index < source.length; index++) {
    const value = source[index];
    if (!Number.isFinite(value)) {
      nonFiniteExcluded++;
      exclusionReasons[index] = "non_finite";
      continue;
    }
    if (logAxis && value <= 0) {
      logDomainExcluded++;
      exclusionReasons[index] = "log_domain";
      continue;
    }
    included[index] = 1;
    if (value < low) {
      displayOverflow[index] = 1;
      if (policy === "cartesian") displayClipped++;
      else {
        values[index] = low;
        edgeMarks[index] = "low";
        clamped++;
      }
    } else if (value > high) {
      displayOverflow[index] = 1;
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
    exclusionReasons,
    displayOverflow,
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
  manifest: ReductionManifest;
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

export interface ReductionManifest {
  originalCount: number;
  displayedCount: number;
  algorithm: "stride_from_first_with_forced_final_endpoint";
  stride: number;
  endpointsForced: boolean;
  sourceIndices: number[];
}

export interface ReductionExportItem {
  subject_kind: "points" | "wells" | "facets" | "legend" | "visual" | "load";
  subject_id: string;
  original_count: number;
  displayed_count: number;
  algorithm: string;
  /** Exact stride for a stride reduction; null when the named algorithm is not stride-based. */
  stride: number | null;
  /** Whether the final eligible source index was appended; null for non-index reductions. */
  endpoints_forced: boolean | null;
}

export interface PlotReductionExport {
  schema_version: 1;
  plot_type: string;
  items: ReductionExportItem[];
  absent: { subject_id: string; reason: string }[];
  refusal: string | null;
}

export interface SharedChannelReduction {
  channels: Float32Array[];
  manifest: ReductionManifest;
}

function strideSourceIndices(eligible: number[], stride: number): { sourceIndices: number[]; endpointsForced: boolean } {
  if (!Number.isInteger(stride) || stride < 1) throw new RangeError("decimation stride must be an integer of at least 1");
  for (let index = 1; index < eligible.length; index++) {
    if (eligible[index - 1] >= eligible[index]) throw new RangeError("eligible source indices must be strictly increasing");
  }
  const sourceIndices: number[] = [];
  for (let position = 0; position < eligible.length; position += stride) sourceIndices.push(eligible[position]);
  let endpointsForced = false;
  const last = eligible.length ? eligible[eligible.length - 1] : undefined;
  if (last !== undefined && sourceIndices[sourceIndices.length - 1] !== last) {
    sourceIndices.push(last);
    endpointsForced = true;
  }
  return { sourceIndices, endpointsForced };
}

/** Apply one index vector to depth/X/Y/Z (or any other channel bundle), preserving
 * sample identity and recording enough information to reproduce the view. */
export function decimateSharedChannels(
  channels: Float32Array[],
  eligible: number[],
  stride: number,
): SharedChannelReduction {
  const { sourceIndices, endpointsForced } = strideSourceIndices(eligible, stride);
  const finalIndex = sourceIndices.length ? sourceIndices[sourceIndices.length - 1] : undefined;
  if (finalIndex !== undefined && channels.some((channel) => finalIndex >= channel.length)) {
    throw new RangeError("shared decimation index exceeds one or more channel lengths");
  }
  return {
    channels: channels.map((channel) => Float32Array.from(sourceIndices.map((index) => channel[index]))),
    manifest: {
      originalCount: eligible.length,
      displayedCount: sourceIndices.length,
      algorithm: "stride_from_first_with_forced_final_endpoint",
      stride,
      endpointsForced,
      sourceIndices,
    },
  };
}

export interface DepthChannelInput {
  depth: Float32Array;
  values: Float32Array;
}

export interface DepthGridReconciliation {
  depth: Float32Array;
  channels: Float32Array[];
  coarsestStep: number;
  decimationFactors: number[];
  mode: "unchanged" | "decimated_to_coarsest";
  intervalClosure: "[lo,hi)";
}

export type DepthStepManifest = Omit<DepthGridReconciliation, "depth" | "channels">;

export class DepthGridReconciliationError extends RangeError {
  readonly route = "reframe" as const;
  readonly actionLabel = "Open Reframe" as const;
  readonly automaticResampling = false as const;

  constructor(message: string) {
    super(message);
    this.name = "DepthGridReconciliationError";
  }
}

/** One unit in the last place of an f32 at this magnitude. A stored depth is the f32
 *  rounding of the true grid value, so it sits within half an ulp of it — which makes
 *  the ulp the resolution limit of the stored grid, not a tolerance anyone chose. */
function f32Ulp(value: number): number {
  const abs = Math.abs(value);
  if (abs === 0 || !Number.isFinite(abs)) return 2 ** -149;
  return 2 ** (Math.floor(Math.log2(abs)) - 23);
}

/** A regular grid stored as f32 does NOT have bit-identical consecutive differences: no
 *  metric step (0.1524, 0.1) is a dyadic rational, and above 1024 m the ulp (2^-13) exceeds
 *  the rounding error of any real step, so the stored differences alternate between two
 *  adjacent representable values on essentially every real metric well. Each stored depth is
 *  within half an ulp of the true grid, so any two consecutive differences agree within 2 ulp
 *  — the guard admits exactly that quantization noise and nothing else. A genuine step change
 *  is at least a whole sample interval, five orders of magnitude above the guard, so every
 *  refusal SB-PLT-016 pinned (a 0.5-then-0.75 grid, a 0.5-versus-0.8 pair) still refuses.
 *  The returned step is the endpoint mean, whose quantization error shrinks with 1/(n-1)
 *  instead of carrying one sample's full ulp. */
function exactDepthStep(depth: Float32Array): number {
  if (depth.length < 2) throw new RangeError("at least two depth samples are required to identify a step");
  const first = depth[1] - depth[0];
  if (!Number.isFinite(first) || first <= 0) throw new RangeError("depth step must be finite and positive");
  const tolerance = 4 * f32Ulp(Math.max(Math.abs(depth[0]), Math.abs(depth[depth.length - 1])));
  for (let index = 2; index < depth.length; index++) {
    if (Math.abs(depth[index] - depth[index - 1] - first) > tolerance) {
      throw new DepthGridReconciliationError(
        "depth grid is not exact and regular; use Reframe to create an explicit shared depth frame",
      );
    }
  }
  return (depth[depth.length - 1] - depth[0]) / (depth.length - 1);
}

export function reconcileDepthSteps(steps: number[]): { coarsestStep: number; decimationFactors: number[] } {
  if (steps.length === 0 || steps.some((step) => !Number.isFinite(step) || step <= 0)) {
    throw new RangeError("depth steps must be finite and positive");
  }
  const coarsestStep = Math.max(...steps);
  const decimationFactors = steps.map((step) => {
    const ratio = coarsestStep / step;
    // Steps arrive as f32-quantized estimates (see exactDepthStep), so an integer ratio
    // computed from them lands NEAR the integer, not on it. Worst case is a two-sample
    // window, where each step carries a full ulp of error (~8e-4 relative on a metric
    // grid), so the snap window is 4e-3 relative — while the closest genuinely
    // non-integer ratio between real step families (0.5 versus 0.8 = 1.6) misses its
    // nearest integer by 0.4, a hundred times the window. Refusals stay refusals.
    const rounded = Math.round(ratio);
    if (rounded < 1 || Math.abs(ratio - rounded) > 4e-3 * rounded) {
      throw new DepthGridReconciliationError(
        `depth steps are not exact integer multiples; use Reframe to create an explicit shared depth frame (${step} versus ${coarsestStep})`,
      );
    }
    return rounded;
  });
  return { coarsestStep, decimationFactors };
}

/** Align already-loaded channels by exact depth identity. This never interpolates:
 * exact multiples use the coarsest input grid; every non-integer relationship refuses. */
export function reconcileDepthChannels(inputs: DepthChannelInput[]): DepthGridReconciliation {
  if (inputs.length === 0) throw new RangeError("at least one depth channel is required");
  for (const input of inputs) {
    if (input.depth.length !== input.values.length) {
      throw new RangeError("depth and value arrays must have identical lengths");
    }
  }
  const steps = inputs.map((input) => exactDepthStep(input.depth));
  const referenceDepth = inputs[0].depth;
  const identicalGrids = inputs.every((input) =>
    input.depth.length === referenceDepth.length
    && input.depth.every((depth, index) => depth === referenceDepth[index]));
  if (identicalGrids) {
    const coarsestStep = steps[0];
    return {
      depth: referenceDepth.slice(),
      channels: inputs.map((input) => input.values.slice()),
      coarsestStep,
      decimationFactors: inputs.map(() => 1),
      mode: "unchanged",
      intervalClosure: "[lo,hi)",
    };
  }
  const { coarsestStep, decimationFactors } = reconcileDepthSteps(steps);
  const targetIndex = steps.findIndex((step) => step === coarsestStep);
  const targetDepth = inputs[targetIndex].depth;
  const sourceMaps = inputs.map((input) => {
    const map = new Map<number, number>();
    for (let index = 0; index < input.depth.length; index++) map.set(input.depth[index], index);
    return map;
  });
  const alignedDepth: number[] = [];
  const alignedValues = inputs.map(() => [] as number[]);
  for (const depth of targetDepth) {
    const indices = sourceMaps.map((map) => map.get(depth));
    if (indices.some((index) => index === undefined)) continue;
    alignedDepth.push(depth);
    for (let channel = 0; channel < inputs.length; channel++) {
      alignedValues[channel].push(inputs[channel].values[indices[channel]!]);
    }
  }
  return {
    depth: Float32Array.from(alignedDepth),
    channels: alignedValues.map((values) => Float32Array.from(values)),
    coarsestStep,
    decimationFactors,
    mode: decimationFactors.every((factor) => factor === 1) ? "unchanged" : "decimated_to_coarsest",
    intervalClosure: "[lo,hi)",
  };
}

export function halfOpenDepthIndices(
  depth: Float32Array,
  low: number | null,
  high: number | null,
): number[] {
  const indices: number[] = [];
  for (let index = 0; index < depth.length; index++) {
    const value = depth[index];
    if (!Number.isFinite(value)) continue;
    if (low !== null && value < low) continue;
    if (high !== null && value >= high) continue;
    indices.push(index);
  }
  return indices;
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
    wells: screened.map((well, index) => {
      const stride = quotas[index] >= well.eligible.length
        ? 1
        : Math.ceil((well.eligible.length - 1) / (quotas[index] - 1));
      const { sourceIndices, endpointsForced } = strideSourceIndices(well.eligible, stride);
      const manifest: ReductionManifest = {
        originalCount: well.eligible.length,
        displayedCount: sourceIndices.length,
        algorithm: "stride_from_first_with_forced_final_endpoint",
        stride,
        endpointsForced,
        sourceIndices: [...sourceIndices],
      };
      return {
        wellId: well.wellId,
        finitePairCount: well.eligible.length,
        quota: sourceIndices.length,
        sourceIndices,
        manifest,
      };
    }),
    absent,
    refusal: null,
  };
}
