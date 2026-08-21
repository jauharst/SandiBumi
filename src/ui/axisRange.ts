import type { PlotChannelBinding, ResolvedPlotCurve } from "../ipc";
import {
  UNIT_REGISTRY_RULES,
  UNIT_REGISTRY_UNITS,
  unitRegistryFamilyFor,
} from "../generated/unitRegistry";

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
  /** Present when a concrete curve family was checked against SB-PLT-005's activation registry. */
  familyLimitAudit?: FamilyLimitAudit;
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
  return { axis, ...range };
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
  const disabled = range.familyLimitAudit && !range.familyLimitAudit.enabled
    ? ` · family limit ${range.familyLimitAudit.reason}`
    : "";
  return `${axis} range: ${tierLabel(range.tier)} · ${range.min} → ${range.max}${disabled}`;
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

const UNIT_LIMIT_CONVERSION_TOLERANCE = 0.15;
const UNIT_LIMIT_SOURCE = "plotting-interactivity dossier §§2.2, 3.3a and 5.2 (T1)";

export interface UnitLimitRow {
  id: string;
  family: string;
  quantityKind: string;
  unit: string;
  range: AxisDisplayRange;
  source: string;
  familyDefault: boolean;
  baseId?: string;
  baseUnit?: string;
  baseRange?: AxisDisplayRange;
  /** Audit-only primary derivation. It can prove a refusal but never authorizes activation. */
  reviewedConversion?: {
    factor: number;
    offset: number;
    derivation: string;
  };
}

export interface UnitLimitAudit {
  id: string;
  enabled: boolean;
  divergenceFactor: number;
  maxRelativeDivergence: number;
  reason: string;
  source: string;
}

export interface FamilyLimitAudit extends UnitLimitAudit {
  family: string;
  unit: string;
  range: AxisDisplayRange | null;
}

/** The entire shipped seed set from the cited dossier plus its required T05 refusal.
 * This list is deliberately small and explicit: it is not an importer for the incumbent table. */
export const UNIT_LIMIT_ROWS: readonly UnitLimitRow[] = [
  {
    id: "GR:gAPI", family: "GR", quantityKind: "gamma_ray", unit: "gAPI",
    range: { min: 0, max: 150 }, source: `${UNIT_LIMIT_SOURCE}; Gamma Ray row`, familyDefault: true,
  },
  {
    id: "RHOB:g/cc", family: "RHOB", quantityKind: "bulk_density", unit: "g/cc",
    range: { min: 1.95, max: 2.95 }, source: `${UNIT_LIMIT_SOURCE}; Bulk Density base row`, familyDefault: true,
  },
  {
    id: "RHOB:kg/m3", family: "RHOB", quantityKind: "bulk_density", unit: "kg/m3",
    range: { min: 1950, max: 2950 }, source: `${UNIT_LIMIT_SOURCE}; Bulk Density alternate row`,
    familyDefault: true, baseId: "RHOB:g/cc",
  },
  {
    id: "NPHI:v/v", family: "NPHI", quantityKind: "fraction", unit: "v/v",
    range: { min: 0.45, max: -0.15 },
    source: `${UNIT_LIMIT_SOURCE}; Thermal Neutron Porosity ft3/ft3 is volume/volume = v/v exactly`,
    familyDefault: true,
  },
  {
    id: "PEF:b/e", family: "PEF", quantityKind: "photoelectric_factor", unit: "b/e",
    range: { min: 0, max: 10 }, source: `${UNIT_LIMIT_SOURCE}; Photoelectric Factor row`, familyDefault: true,
  },
  {
    id: "PHIE:v/v", family: "POR", quantityKind: "fraction", unit: "v/v",
    range: { min: 0.5, max: 0 },
    source: `${UNIT_LIMIT_SOURCE}; Effective Porosity ft3/ft3 is volume/volume = v/v exactly`
      + "; family default for POR, every member of which is v/v",
    familyDefault: true,
  },
  {
    id: "SW:v/v", family: "SW", quantityKind: "fraction", unit: "v/v",
    range: { min: 1, max: 0 },
    source: `${UNIT_LIMIT_SOURCE}; Water Saturation ft3/ft3 is volume/volume = v/v exactly`,
    familyDefault: true,
  },
  {
    id: "DT:us/ft", family: "DT", quantityKind: "slowness", unit: "us/ft",
    range: { min: 240, max: 40 }, source: `${UNIT_LIMIT_SOURCE}; Compressional Slowness base row`, familyDefault: true,
  },
  {
    id: "DT:us/m", family: "DT", quantityKind: "slowness", unit: "us/m",
    range: { min: 780, max: 120 }, source: `${UNIT_LIMIT_SOURCE}; Compressional Slowness alternate row`,
    familyDefault: true, baseId: "DT:us/ft",
  },
  {
    id: "ACOUSTIC_ATTENUATION_RATE:dB/m", family: "ACOUSTIC_ATTENUATION_RATE",
    quantityKind: "attenuation_rate", unit: "dB/m", range: { min: 0, max: 50 },
    baseUnit: "dB/ft", baseRange: { min: 0, max: 100 },
    source: `${UNIT_LIMIT_SOURCE}; SB-PLT-T05 documented divergent pair`, familyDefault: false,
    reviewedConversion: {
      factor: 0.3048,
      offset: 0,
      derivation: "1 international ft = 0.3048 m exactly; dB/m × m/ft = dB/ft",
    },
  },
];

function registeredUnit(unit: string) {
  const key = normalizedUnit(unit);
  return UNIT_REGISTRY_UNITS.find((entry) => normalizedUnit(entry.token) === key) ?? null;
}

function registeredRule(fromUnit: string, toUnit: string) {
  const from = registeredUnit(fromUnit);
  const to = registeredUnit(toUnit);
  if (!from || !to || from.quantityKind !== to.quantityKind) return null;
  return UNIT_REGISTRY_RULES.find((rule) =>
    normalizedUnit(rule.fromUnit) === normalizedUnit(from.canonicalUnit)
      && normalizedUnit(rule.toUnit) === normalizedUnit(to.canonicalUnit)) ?? null;
}

function endpointAudit(expected: number, actual: number): { relative: number; factor: number } {
  if (expected === actual) return { relative: 0, factor: 1 };
  if (!Number.isFinite(expected) || !Number.isFinite(actual) || expected === 0 || actual === 0) {
    return { relative: Number.POSITIVE_INFINITY, factor: Number.POSITIVE_INFINITY };
  }
  const ratio = Math.abs(expected / actual);
  return {
    relative: Math.abs(expected - actual) / Math.abs(expected),
    factor: Math.max(ratio, 1 / ratio),
  };
}

export function auditUnitLimitRow(row: UnitLimitRow): UnitLimitAudit {
  if (!row || !row.source) {
    return {
      id: row?.id ?? "unknown",
      enabled: false,
      divergenceFactor: Number.POSITIVE_INFINITY,
      maxRelativeDivergence: Number.POSITIVE_INFINITY,
      reason: "disabled: unit-limit row has no numeric source",
      source: row?.source ?? "",
    };
  }
  const unit = registeredUnit(row.unit);
  if (!row.baseId && !row.baseRange) {
    const typed = unit?.quantityKind === row.quantityKind;
    return {
      id: row.id,
      enabled: row.familyDefault && typed,
      divergenceFactor: typed ? 1 : Number.POSITIVE_INFINITY,
      maxRelativeDivergence: typed ? 0 : Number.POSITIVE_INFINITY,
      reason: row.familyDefault && typed
        ? "enabled: source-owned registered unit row"
        : `disabled: ${row.unit} is not registered as ${row.quantityKind}`,
      source: row.source,
    };
  }

  const base = row.baseId ? UNIT_LIMIT_ROWS.find((candidate) => candidate.id === row.baseId) : null;
  const baseUnit = base?.unit ?? row.baseUnit;
  const baseRange = base?.range ?? row.baseRange;
  const rule = baseUnit ? registeredRule(row.unit, baseUnit) : null;
  const conversion = rule ?? row.reviewedConversion ?? null;
  if (!baseUnit || !baseRange || !conversion) {
    return {
      id: row.id,
      enabled: false,
      divergenceFactor: Number.POSITIVE_INFINITY,
      maxRelativeDivergence: Number.POSITIVE_INFINITY,
      reason: "disabled: no registered dimensional conversion proves this unit-limit row",
      source: row.source,
    };
  }
  const converted = {
    min: (row.range.min + conversion.offset) * conversion.factor,
    max: (row.range.max + conversion.offset) * conversion.factor,
  };
  const low = endpointAudit(baseRange.min, converted.min);
  const high = endpointAudit(baseRange.max, converted.max);
  const maxRelativeDivergence = Math.max(low.relative, high.relative);
  const divergenceFactor = Math.max(low.factor, high.factor);
  const sameDirection = Math.sign(baseRange.max - baseRange.min) === Math.sign(converted.max - converted.min);
  if (!sameDirection) {
    return {
      id: row.id, enabled: false, divergenceFactor, maxRelativeDivergence,
      reason: "disabled: converted unit-limit row reverses the base-row direction",
      source: row.source,
    };
  }
  if (maxRelativeDivergence > UNIT_LIMIT_CONVERSION_TOLERANCE) {
    return {
      id: row.id, enabled: false, divergenceFactor, maxRelativeDivergence,
      reason: `disabled: ${(maxRelativeDivergence * 100).toFixed(2)}% divergence exceeds the cited 15% screen (${divergenceFactor.toFixed(2)}×)`,
      source: row.source,
    };
  }
  if (!row.familyDefault || !rule || unit?.quantityKind !== row.quantityKind) {
    return {
      id: row.id, enabled: false, divergenceFactor, maxRelativeDivergence,
      reason: "disabled: audit-only row has no registered family-default activation route",
      source: row.source,
    };
  }
  return {
    id: row.id,
    enabled: true,
    divergenceFactor,
    maxRelativeDivergence,
    reason: maxRelativeDivergence === 0
      ? "enabled: exact registered conversion"
      : `enabled: registered conversion within the cited 15% screen (${(maxRelativeDivergence * 100).toFixed(2)}%)`,
    source: row.source,
  };
}

function registeredFamily(mnemonic: string): string | null {
  return unitRegistryFamilyFor(mnemonic);
}

/** Returns the row-level activation result, including a preserved refusal reason. */
export function auditedFamilyDisplayDecision(curve: ResolvedPlotCurve | null): FamilyLimitAudit | null {
  if (!curve) return null;
  const mnemonic = curve.mnemonic.trim().toUpperCase();
  // No `PHIE` arm: the registry resolves every porosity mnemonic to POR, so such an arm could
  // never run, and keying the row to it disabled the row for every porosity curve instead.
  // The saturation arm IS reachable - no SW family is registered.
  const family = registeredFamily(mnemonic)
    ?? (["SW", "SWE", "SWT"].includes(mnemonic) ? "SW" : null);
  if (!family) return null;
  const registered = registeredUnit(curve.display_unit);
  const unit = registered?.canonicalUnit ?? normalizedUnit(curve.display_unit);
  const row = UNIT_LIMIT_ROWS.find((candidate) =>
    candidate.familyDefault && candidate.family === family && normalizedUnit(candidate.unit) === normalizedUnit(unit));
  if (!row) {
    return {
      id: `${family}:${unit}`,
      family,
      unit,
      range: null,
      enabled: false,
      divergenceFactor: Number.POSITIVE_INFINITY,
      maxRelativeDivergence: Number.POSITIVE_INFINITY,
      reason: `disabled: no audited unit-limit row for ${family} ${curve.display_unit}`,
      source: "",
    };
  }
  const audit = auditUnitLimitRow(row);
  return {
    ...audit,
    family,
    unit: row.unit,
    range: audit.enabled ? { ...row.range } : null,
  };
}

export function auditedFamilyDisplayRange(curve: ResolvedPlotCurve | null): AxisDisplayRange | null {
  return auditedFamilyDisplayDecision(curve)?.range ?? null;
}

function bindingAuditedFamilyDisplayDecision(binding: PlotChannelBinding | null): FamilyLimitAudit | null {
  if (!binding || binding.resolved.length === 0) return null;
  const first = auditedFamilyDisplayDecision(binding.resolved[0]);
  if (!first) return null;
  for (let index = 1; index < binding.resolved.length; index += 1) {
    const next = auditedFamilyDisplayDecision(binding.resolved[index]);
    if (!next) {
      return { ...first, enabled: false, range: null, reason: "disabled: represented curves do not share one audited family" };
    }
    if (!next.enabled) return next;
    if (!first.enabled || !first.range || !next.range || !sameRange(first.range, next.range)) {
      return { ...first, enabled: false, range: null, reason: "disabled: represented curves do not share one audited family display range" };
    }
  }
  return first;
}

export function bindingAuditedFamilyDisplayRange(binding: PlotChannelBinding | null): AxisDisplayRange | null {
  return bindingAuditedFamilyDisplayDecision(binding)?.range ?? null;
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
  const familyLimitAudit = bindingAuditedFamilyDisplayDecision(args.binding);
  const resolved = resolveAxisRange({
    user: admissible(args.user),
    headerDisplay: admissible(bindingHeaderDisplayRange(args.binding)),
    auditedFamilyDisplay: admissible(familyLimitAudit?.range ?? null),
    finiteData: admissible(args.finiteData),
    validity: args.validity ?? null,
  });
  return resolved && familyLimitAudit ? { ...resolved, familyLimitAudit } : resolved;
}
