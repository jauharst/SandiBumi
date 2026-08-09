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
