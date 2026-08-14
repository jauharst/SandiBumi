import type { PlotChannelBinding, ResolvedPlotCurve } from "../ipc";
import { UNIT_REGISTRY_FAMILIES } from "../generated/unitRegistry";

export type AxisRangeTier =
  | "user"
  | "header_display"
  | "audited_family_display"
  | "finite_data";

export interface AxisDisplayRange {
  min: number;
  max: number;
}

export interface AxisRangeCandidates {
  user: AxisDisplayRange | null;
  headerDisplay: AxisDisplayRange | null;
  auditedFamilyDisplay: AxisDisplayRange | null;
  finiteData: AxisDisplayRange | null;
  /** Scientific validity controls filtering and statistics; it never supplies display limits. */
  validity: AxisDisplayRange | null;
}

export interface AxisRangeResolution extends AxisDisplayRange {
  tier: AxisRangeTier;
}

export interface PlotAxisRangeExport extends AxisRangeResolution {
  axis: string;
}

function usable(range: AxisDisplayRange | null): range is AxisDisplayRange {
  return !!range && Number.isFinite(range.min) && Number.isFinite(range.max) && range.min !== range.max;
}

/** SB-PLT-002's only display-limit precedence chain. Validity is deliberately present in
 * the input type and deliberately absent from the ordered candidates. */
export function resolveAxisRange(candidates: AxisRangeCandidates): AxisRangeResolution | null {
  const ordered: Array<[AxisDisplayRange | null, AxisRangeTier]> = [
    [candidates.user, "user"],
    [candidates.headerDisplay, "header_display"],
    [candidates.auditedFamilyDisplay, "audited_family_display"],
    [candidates.finiteData, "finite_data"],
  ];
  for (const [range, tier] of ordered) {
    if (usable(range)) return { ...range, tier };
  }
  return null;
}

export function axisRangeExportRecord(axis: string, range: AxisRangeResolution): PlotAxisRangeExport {
  return { axis, min: range.min, max: range.max, tier: range.tier };
}

function tierLabel(tier: AxisRangeTier): string {
  switch (tier) {
    case "user": return "user";
    case "header_display": return "header display";
    case "audited_family_display": return "audited family display";
    case "finite_data": return "finite data";
  }
}

export function formatAxisRangeLabel(axis: string, range: AxisRangeResolution): string {
  return `${axis} range: ${tierLabel(range.tier)} · ${range.min} → ${range.max}`;
}

export function formatAxisRangeSummary(ranges: PlotAxisRangeExport[]): string {
  return ranges.map((range) => formatAxisRangeLabel(range.axis.toUpperCase(), range)).join(" · ");
}

function sameRange(left: AxisDisplayRange, right: AxisDisplayRange): boolean {
  return left.min === right.min && left.max === right.max;
}

function commonRange(
  curves: ResolvedPlotCurve[],
  get: (curve: ResolvedPlotCurve) => AxisDisplayRange | null,
): AxisDisplayRange | null {
  if (curves.length === 0) return null;
  const first = get(curves[0]);
  if (!first) return null;
  for (let index = 1; index < curves.length; index++) {
    const next = get(curves[index]);
    if (!next || !sameRange(first, next)) return null;
  }
  return first;
}

export function bindingHeaderDisplayRange(binding: PlotChannelBinding | null): AxisDisplayRange | null {
  if (!binding) return null;
  return commonRange(binding.resolved, (curve) => curve.header_display
    ? { min: curve.header_display.low, max: curve.header_display.high }
    : null);
}

function normalizedUnit(unit: string): string {
  return unit.trim().toLowerCase().replace(/\s+/g, "");
}

function registeredFamily(mnemonic: string): string | null {
  const key = mnemonic.trim().toUpperCase();
  const family = UNIT_REGISTRY_FAMILIES.find((entry) =>
    entry.aliases.some((alias) => alias.toUpperCase() === key));
  return family?.family ?? null;
}

/** Screened seed set from plotting-interactivity.md dossier §5.2. Every unit alternative
 * listed here passed that dossier's §3.3a conversion audit; no wider vendor table is imported. */
export function auditedFamilyDisplayRange(curve: ResolvedPlotCurve | null): AxisDisplayRange | null {
  if (!curve) return null;
  const mnemonic = curve.mnemonic.trim().toUpperCase();
  const family = registeredFamily(mnemonic)
    ?? (mnemonic === "PHIE" ? "PHIE" : null)
    ?? (["SW", "SWE", "SWT"].includes(mnemonic) ? "SW" : null);
  const unit = normalizedUnit(curve.display_unit);
  switch (family) {
    case "GR":
      return unit === "gapi" ? { min: 0, max: 150 } : null;
    case "RHOB":
      if (["g/cc", "g/c3", "gm/cc"].includes(unit)) return { min: 1.95, max: 2.95 };
      if (unit === "kg/m3") return { min: 1950, max: 2950 };
      return null;
    case "NPHI":
      return unit === "v/v" ? { min: 0.45, max: -0.15 } : null;
    case "PEF":
      return unit === "b/e" ? { min: 0, max: 10 } : null;
    case "PHIE":
      return unit === "v/v" ? { min: 0.5, max: 0 } : null;
    case "SW":
      return unit === "v/v" ? { min: 1, max: 0 } : null;
    case "DT":
      if (["us/ft", "us/f"].includes(unit)) return { min: 240, max: 40 };
      if (unit === "us/m") return { min: 780, max: 120 };
      return null;
    default:
      return null;
  }
}

export function bindingAuditedFamilyDisplayRange(binding: PlotChannelBinding | null): AxisDisplayRange | null {
  if (!binding) return null;
  return commonRange(binding.resolved, auditedFamilyDisplayRange);
}

export function bindingForChannel(
  bindings: PlotChannelBinding[],
  channel: string,
): PlotChannelBinding | null {
  return bindings.find((binding) => binding.intent.channel === channel) ?? null;
}

export function resolveBoundAxisRange(args: {
  binding: PlotChannelBinding | null;
  user: AxisDisplayRange | null;
  finiteData: AxisDisplayRange | null;
  validity?: AxisDisplayRange | null;
  log?: boolean;
}): AxisRangeResolution | null {
  const admissible = (range: AxisDisplayRange | null): AxisDisplayRange | null =>
    range && (!args.log || (range.min > 0 && range.max > 0)) ? range : null;
  return resolveAxisRange({
    user: admissible(args.user),
    headerDisplay: admissible(bindingHeaderDisplayRange(args.binding)),
    auditedFamilyDisplay: admissible(bindingAuditedFamilyDisplayRange(args.binding)),
    finiteData: admissible(args.finiteData),
    validity: args.validity ?? null,
  });
}
