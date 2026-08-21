import type { ReductionExportItem } from "./plotTypes";

export type PlotRecordLimitId =
  | "context_fetch_concurrency"
  | "context_point_budget"
  | "well_scope_name_preview_rows"
  | "context_well_legend_rows"
  | "context_well_name_characters"
  | "fit_scatter_legend_rows"
  | "vega_categorical_groups";

export type PlotRecordLimitPolicy =
  | "throttle_all"
  | "reduce_custom_with_manifest"
  | "reduce_prefix_with_manifest"
  | "reduce_text_with_manifest"
  | "refuse_above_hard_maximum";

export interface PlotRecordLimit {
  id: PlotRecordLimitId;
  subject_kind: ReductionExportItem["subject_kind"];
  policy: PlotRecordLimitPolicy;
  maximum: number;
  algorithm: string;
  source: string;
  consumers: readonly string[];
}

/**
 * Complete registry of limits that can reduce, throttle, or refuse plot records.
 *
 * SB-PLT-031 does not authorize choosing new maxima. The two cited values come from
 * PRD v2 chapter 23 section 5; the other values preserve the current as-built limits
 * while making their behavior and custody explicit. A retained value is not promoted
 * here into an independently validated product default.
 */
export const PLOT_RECORD_LIMITS: readonly PlotRecordLimit[] = [
  {
    id: "context_fetch_concurrency",
    subject_kind: "load",
    policy: "throttle_all",
    maximum: 8,
    algorithm: "bounded_parallel_fetch_without_record_reduction",
    source: "docs/PRD_v2/23_plotting-interactivity.md section 5",
    consumers: ["plotCommon.ts"],
  },
  {
    id: "context_point_budget",
    subject_kind: "points",
    policy: "reduce_custom_with_manifest",
    maximum: 60_000,
    algorithm: "fair_per_well_stride_with_forced_final_endpoint_and_manifest",
    source: "docs/PRD_v2/23_plotting-interactivity.md section 5",
    // AUDIT-2026-08-20 finding 57: the three panels used to resolve this each for themselves,
    // in three copies of one reload. They share createContextReload now, so the budget has a
    // single consumer and cannot be answered differently on one plot than on another.
    consumers: ["plotCommon.ts"],
  },
  {
    id: "well_scope_name_preview_rows",
    subject_kind: "wells",
    policy: "reduce_prefix_with_manifest",
    maximum: 40,
    algorithm: "first_well_names_with_remainder_count",
    source: "retained as-built from wellScope.ts at 3a4723b6",
    consumers: ["wellScope.ts", "plotCommon.ts"],
  },
  {
    id: "context_well_legend_rows",
    subject_kind: "legend",
    policy: "reduce_prefix_with_manifest",
    maximum: 10,
    algorithm: "first_context_well_rows_with_reported_remainder",
    source: "retained as-built from plotCommon.ts at 3a4723b6",
    consumers: ["plotCommon.ts", "crossplotPanel.ts", "histogramPanel.ts", "pickettPanel.ts"],
  },
  {
    id: "context_well_name_characters",
    subject_kind: "visual",
    policy: "reduce_text_with_manifest",
    maximum: 18,
    algorithm: "leading_characters_with_ellipsis_and_reported_remainder",
    source: "retained as-built from context legends at 3a4723b6",
    consumers: ["plotCommon.ts", "crossplotPanel.ts", "histogramPanel.ts", "pickettPanel.ts"],
  },
  {
    id: "fit_scatter_legend_rows",
    subject_kind: "legend",
    policy: "reduce_prefix_with_manifest",
    maximum: 12,
    algorithm: "first_group_rows_with_remainder_count",
    source: "retained as-built from fitScatter.ts at 3a4723b6",
    consumers: ["fitScatter.ts"],
  },
  {
    id: "vega_categorical_groups",
    subject_kind: "facets",
    policy: "refuse_above_hard_maximum",
    maximum: 24,
    algorithm: "hard_refusal_above_categorical_group_maximum",
    source: "retained as-built from vegaPanel.ts at 3a4723b6",
    consumers: ["vegaPanel.ts"],
  },
] as const;

const LIMIT_BY_ID = new Map(PLOT_RECORD_LIMITS.map((limit) => [limit.id, limit]));

export function plotRecordLimit(id: PlotRecordLimitId): PlotRecordLimit {
  const limit = LIMIT_BY_ID.get(id);
  if (!limit) throw new Error(`Unknown plot record limit '${id}'`);
  return limit;
}

function reductionItem(
  limit: PlotRecordLimit,
  subjectId: string,
  originalCount: number,
  displayedCount: number,
): ReductionExportItem {
  if (!Number.isInteger(originalCount) || originalCount < 0) {
    throw new Error(`${limit.id} original count must be a non-negative integer`);
  }
  if (!Number.isInteger(displayedCount) || displayedCount < 0 || displayedCount > originalCount) {
    throw new Error(`${limit.id} displayed count must be between zero and the original count`);
  }
  return {
    subject_kind: limit.subject_kind,
    subject_id: subjectId,
    original_count: originalCount,
    displayed_count: displayedCount,
    algorithm: limit.algorithm,
    stride: null,
    endpoints_forced: null,
  };
}

/** Count-only form used when the caller already owns the records and must not allocate a copy. */
export function plotRecordCountReduction(
  id: PlotRecordLimitId,
  originalCount: number,
  subjectId: string,
): ReductionExportItem | null {
  const limit = plotRecordLimit(id);
  if (limit.policy !== "reduce_prefix_with_manifest") {
    throw new Error(`${id} is not a prefix-with-manifest limit`);
  }
  if (originalCount <= limit.maximum) return null;
  return reductionItem(limit, subjectId, originalCount, limit.maximum);
}

export interface AppliedPlotRecordLimit<T> {
  displayed: T[];
  item: ReductionExportItem | null;
  refusal: string | null;
}

/** Applies only policies whose complete before/after behavior is expressible for a generic array. */
export function applyPlotRecordLimit<T>(
  id: PlotRecordLimitId,
  values: readonly T[],
  subjectId: string,
): AppliedPlotRecordLimit<T> {
  const limit = plotRecordLimit(id);
  if (limit.policy === "throttle_all") {
    return { displayed: Array.from(values), item: null, refusal: null };
  }
  if (limit.policy === "reduce_prefix_with_manifest") {
    const item = plotRecordCountReduction(id, values.length, subjectId);
    return {
      displayed: item ? Array.from(values).slice(0, limit.maximum) : Array.from(values),
      item,
      refusal: null,
    };
  }
  if (limit.policy === "refuse_above_hard_maximum") {
    if (values.length <= limit.maximum) {
      return { displayed: Array.from(values), item: null, refusal: null };
    }
    return {
      displayed: [],
      item: reductionItem(limit, subjectId, values.length, 0),
      refusal: `${subjectId} has ${values.length} records and exceeds hard maximum ${limit.maximum}`,
    };
  }
  if (limit.policy === "reduce_text_with_manifest") {
    throw new Error(`${id} is a text limit; use reducePlotLabel`);
  }
  throw new Error(`${id} uses a custom reducer that must provide its own complete manifest`);
}

export interface ReducedPlotLabel {
  displayed: string;
  item: ReductionExportItem | null;
}

/** Reduces by Unicode code point so one truncation never emits half of a surrogate pair. */
export function reducePlotLabel(
  id: PlotRecordLimitId,
  text: string,
  subjectId: string,
): ReducedPlotLabel {
  const limit = plotRecordLimit(id);
  if (limit.policy !== "reduce_text_with_manifest") {
    throw new Error(`${id} is not a text-with-manifest limit`);
  }
  const characters = Array.from(text);
  if (characters.length <= limit.maximum) return { displayed: text, item: null };
  const displayed = `${characters.slice(0, limit.maximum - 1).join("")}…`;
  return {
    displayed,
    item: reductionItem(limit, `context_well_name:${subjectId}`, characters.length, limit.maximum),
  };
}
