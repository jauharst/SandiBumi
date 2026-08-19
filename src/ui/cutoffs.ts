import { listDocuments } from "../ipc";

/** The pay-summary cutoff quartet: sand (VSH ≤), reservoir (PHIE ≥), pay (SWE ≤) and an optional
 *  permeability floor (PERM ≥, off by default). */
export interface CutoffDefaults {
  vsh_max: number | null;
  phie_min: number | null;
  swe_max: number | null;
  perm_min: number | null;
}

/** SB-CUT-019. The unit each cut-off is entered and stored in. Volume fractions are held in `v/v`
 *  and permeability in `mD`, so a saved project default carries its unit to the backend rather
 *  than arriving as a bare number the engine would have to guess at. */
export const CUTOFF_UNITS: Record<keyof CutoffDefaults, string> = {
  vsh_max: "v/v",
  phie_min: "v/v",
  swe_max: "v/v",
  perm_min: "mD",
};

/** SB-CUT-019. Wrap a stored canonical value as the entered form the backend requires. */
export function asCutoffEntry(
  field: keyof CutoffDefaults,
  value: number | null,
): { value: number; unit: string } | null {
  return value === null ? null : { value, unit: CUTOFF_UNITS[field] };
}

/** The one canonical fallback, used when a project has no saved cutoffs. Every pay-cutoff pane seeds
 *  from `loadCutoffDefaults()` so Monte Carlo, the pay summary, the report and the cutoff editor can
 *  never quote different defaults for the same field. They used to: Monte Carlo hard-coded PHIE ≥
 *  0.08 / SWE ≤ 0.5 against the summary's 0.1 / 0.6, so an MC net-pay silently used *different*
 *  cutoffs than the deterministic pay summary and the two numbers could not be reconciled — while
 *  the MC settings tooltip claimed "Cutoffs match the pay summary". */
export const DEFAULT_CUTOFFS: CutoffDefaults = {
  vsh_max: null,
  phie_min: null,
  swe_max: null,
  perm_min: null,
};

/** Merge a saved (possibly partial or malformed) cutoff document over the canonical defaults, so a
 *  caller always gets a complete, finite set. Pure — the unit of behaviour worth testing. */
export function mergeCutoffs(saved: Partial<CutoffDefaults> | null | undefined): CutoffDefaults {
  const num = (v: unknown): number | null => (typeof v === "number" && Number.isFinite(v) ? v : null);
  return {
    vsh_max: num(saved?.vsh_max),
    phie_min: num(saved?.phie_min),
    swe_max: num(saved?.swe_max),
    // perm_min is genuinely optional (null = off); a finite saved value wins, everything else is off.
    perm_min: typeof saved?.perm_min === "number" && Number.isFinite(saved.perm_min) ? saved.perm_min : DEFAULT_CUTOFFS.perm_min,
  };
}

/** Load the project's saved default cutoffs (written by the Cutoffs pane to documents
 *  "cutoffs"/"__default__"), merged over DEFAULT_CUTOFFS. Any read/parse failure → canonical
 *  defaults, so a pane always opens with a coherent, complete set. */
export async function loadCutoffDefaults(): Promise<CutoffDefaults> {
  try {
    const docs = await listDocuments("cutoffs");
    const doc = docs.find((d) => d.name === "__default__");
    return mergeCutoffs(doc ? (JSON.parse(doc.json) as Partial<CutoffDefaults>) : null);
  } catch {
    return { ...DEFAULT_CUTOFFS };
  }
}
