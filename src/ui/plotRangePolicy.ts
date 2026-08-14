import type { AxisDisplayRange } from "./axisRange";
import { applyPlotChannelPolicy } from "./plotTypes";

/** One quantitative channel participating in a plot's sample population. Display limits
 * affect only glyph visibility; opt-in validity limits affect the analysis population. */
export interface PlotRangePolicyChannel {
  values: ArrayLike<number>;
  display: AxisDisplayRange | null;
  validity: AxisDisplayRange | null;
  /** A non-positive value is ineligible when the rendered channel uses a logarithmic axis. */
  log?: boolean;
}

/** One row is counted once even when more than one channel makes it ineligible. */
export interface PlotRangePolicyReport {
  inputCount: number;
  indices: number[];
  analysisCount: number;
  nonFiniteExcluded: number;
  logDomainExcluded: number;
  validityExcluded: number;
  displayHidden: number;
}

function outside(value: number, range: AxisDisplayRange | null): boolean {
  if (!range) return false;
  const low = Math.min(range.min, range.max);
  const high = Math.max(range.min, range.max);
  return value < low || value > high;
}

/** Apply the common SB-PLT-004 population policy to aligned channel arrays.
 *
 * The returned indices always describe the analysis/statistics/fit population. A display
 * limit never removes an index; it only increments `displayHidden`. Validity exclusion is
 * inactive until the analyst explicitly enables it.
 */
export function applyPlotRangePolicy(
  channels: readonly PlotRangePolicyChannel[],
  applyValidity: boolean,
): PlotRangePolicyReport {
  const inputCount = channels[0]?.values.length ?? 0;
  for (const channel of channels) {
    if (channel.values.length !== inputCount) {
      throw new Error("plot range policy requires aligned channels with equal sample counts");
    }
  }

  const report: PlotRangePolicyReport = {
    inputCount,
    indices: [],
    analysisCount: 0,
    nonFiniteExcluded: 0,
    logDomainExcluded: 0,
    validityExcluded: 0,
    displayHidden: 0,
  };
  const channelReports = channels.map((channel) => applyPlotChannelPolicy(
    Float32Array.from(channel.values),
    "cartesian",
    channel.display,
    !!channel.log,
  ));

  for (let index = 0; index < inputCount; index++) {
    if (channelReports.some((channel) => channel.exclusionReasons[index] === "non_finite")) {
      report.nonFiniteExcluded++;
      continue;
    }
    if (channelReports.some((channel) => channel.exclusionReasons[index] === "log_domain")) {
      report.logDomainExcluded++;
      continue;
    }
    if (applyValidity && channels.some((channel) => outside(channel.values[index], channel.validity))) {
      report.validityExcluded++;
      continue;
    }

    report.indices.push(index);
    if (channelReports.some((channel) => channel.displayOverflow[index] === 1)) {
      report.displayHidden++;
    }
  }
  report.analysisCount = report.indices.length;
  return report;
}

export interface PlotRangePolicySummaryOptions {
  /** State explicitly that descriptive statistics use the analysis population. */
  statistics?: boolean;
  /** Supply the actual fit-input count only when the surface has an active fit. */
  fitInputs?: number | null;
}

/** One observable vocabulary across every pilot plotting surface. */
export function formatPlotRangePolicySummary(
  report: PlotRangePolicyReport,
  options: PlotRangePolicySummaryOptions = {},
): string {
  const parts = [
    `n=${report.analysisCount}`,
    `non-finite excluded=${report.nonFiniteExcluded}`,
    `log-domain excluded=${report.logDomainExcluded}`,
    `display hidden=${report.displayHidden}`,
    `validity excluded=${report.validityExcluded}`,
  ];
  if (options.statistics) parts.push(`statistics n=${report.analysisCount}`);
  if (options.fitInputs !== undefined && options.fitInputs !== null) {
    parts.push(`fit inputs=${options.fitInputs}`);
  }
  return parts.join(" · ");
}
