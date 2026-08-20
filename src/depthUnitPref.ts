import { getProjectDepthUnit } from "./ipc";
import { appState, setStatus } from "./state";
import type { DepthUnit } from "./units";
import { convertDepth, unitLabel } from "./units";

/** Loading and persisting the depth-unit settings.
 *
 *  The STORED unit comes from the project and is authoritative — it is a fact about the
 *  data, not a preference. The DISPLAY unit is a per-machine preference that defaults to
 *  the stored unit, so someone who never touches it sees the depths their files came in.
 */

const DISPLAY_KEY = "sandibumi.displayDepthUnit";

function readStoredPreference(): DepthUnit | null {
  try {
    const v = localStorage.getItem(DISPLAY_KEY);
    return v === "M" || v === "FT" ? v : null;
  } catch {
    return null; // private mode / storage disabled — fall back to the project unit
  }
}

/** Reads the project's declared depth unit and seeds the display unit from it. Call at
 *  startup and after any project switch: a different project can be in a different unit,
 *  and a stale stored unit would mislabel every depth AND skew the 1:N print scale. */
export async function syncDepthUnits(): Promise<void> {
  let project: DepthUnit = "M";
  try {
    const [code] = await getProjectDepthUnit();
    project = code === "FT" ? "FT" : "M";
  } catch {
    // No backend (vite-only preview) or a project that predates unit handling: metres is
    // the documented default and what those projects were already assuming.
  }
  appState.projectDepthUnit.set(project);
  // An explicit preference wins; otherwise follow the project so depths read the way the
  // source files did.
  appState.displayDepthUnit.set(readStoredPreference() ?? project);
}

/** SB-ENV-057. The one manifest token (`modules::PROJECT_DEPTH_UNIT_TOKEN`) a module argument
 *  uses to say "this length is in whatever unit the project stores". */
export const PROJECT_DEPTH_UNIT_TOKEN = "depth";

/** The unit label for a depth a user TYPES — the project's stored unit.
 *
 *  Every such field reaches the backend unconverted and is compared against, or added to, the
 *  stored depth grid: a curve shift, a core shift, a top/bottom interval, a correlation window,
 *  a print range, a TD or KB. Labelling any of them with the display preference would invite a
 *  metre number typed against a foot grid, which lands as a plausible wrong answer rather than
 *  an error. Deliberately the mirror of [`shownDepthLabel`]. */
export function storedDepthLabel(): string {
  return unitLabel(appState.projectDepthUnit.get());
}

/** The unit label for a depth a user READS — the display unit, converted for viewing.
 *
 *  For an axis, a heading, a computed thickness, an exported column. Nothing is being entered,
 *  so following the view preference is free and is what the reader asked for. */
export function shownDepthLabel(): string {
  return unitLabel(appState.displayDepthUnit.get());
}

/** A stored length converted for reading, the numeric partner of [`shownDepthLabel`].
 *
 *  Always used together with it. A converted value under a stale heading and a stale value under
 *  a converted heading are the same lie, and both have shipped from this codebase — so the pair
 *  lives in one place rather than being re-derived per panel. Applies to any length in the
 *  project's depth dimension: a depth, a thickness, and hydrocarbon pore thickness, which is one.
 *  Never to a ratio, a volume fraction or a sample count.
 *
 *  The trap is a unit-free number sitting BESIDE length-bearing ones. The **Lorenz coefficient**
 *  is the worked example: Σk·h and Σφ·h each carry one factor of depth and must convert, while
 *  the coefficient built from cumulative FRACTIONS of those same sums must not — the factor
 *  cancels out of it exactly. Converting the whole line reports a heterogeneity of 0.128 where
 *  the answer is 0.420, which is the entire point of the plot, wrong and plausible. Decide per
 *  VALUE, never per line. */
export function toShownDepth(value: number): number {
  return convertDepth(value, appState.projectDepthUnit.get(), appState.displayDepthUnit.get());
}

/** The unit to PRINT beside a module argument.
 *
 *  A project-native length has no fixed unit of its own: the number the user types goes to the
 *  backend unconverted and is compared straight against the stored depth grid. So the label
 *  must name the unit those depths are in — the **stored** unit, deliberately not the display
 *  unit. That is the opposite choice from a read-only panel like the Field Dashboard, and for
 *  the opposite reason: there the number is leaving, here it is arriving. Labelling an input
 *  with a view preference would invite exactly the mis-entry the token exists to prevent —
 *  a free-water level typed in metres against a foot-stored depth grid.
 *
 *  Everything else is a real fixed unit (g/cc, v/v, mD, ohm·m, and the genuinely metric lengths
 *  the module converts itself) and passes through untouched. */
export function argumentUnitLabel(unit: string | null | undefined): string {
  if (!unit) return "";
  return unit === PROJECT_DEPTH_UNIT_TOKEN ? storedDepthLabel() : unit;
}

/** The default "these two measurements are the same plug" tolerance, in the project's STORED
 *  unit — one standard 6-inch log sample.
 *
 *  **Kept in step with `units::same_depth_tolerance` in Rust**, which is the authority; the two
 *  numbers are deliberately not a conversion of each other (0.15 m is what shipped and what the
 *  documentation says, 0.5 ft is six inches exactly). This copy exists so a dialog can SHOW the
 *  default it is about to send — the backend still resolves it for any caller that sends nothing.
 *
 *  It belongs to `storedDepthLabel`, not `shownDepthLabel`: the user types it, it reaches the
 *  backend unconverted, and it is compared straight against the stored depth grid. */
export function sameDepthTolerance(): number {
  return appState.projectDepthUnit.get() === "FT" ? 0.5 : 0.15;
}

/** A default that was chosen as a PHYSICAL SIZE in metres, restated in the project's STORED unit.
 *
 *  **The twin of `units::metres_in` in Rust**, and used for the same reason on this side of the
 *  bridge: a correlation window, a search range or a flag threshold pre-filled in a box is a
 *  judgement about how much SECTION the operation needs, and it reaches the backend unconverted
 *  to meet the stored depth grid. A bare `10` in that box is 10 m on a metre project and 10 ft —
 *  3 m — on a foot one, which is too little section to match a GR pattern on. The label already
 *  names the unit (see [`storedDepthLabel`]); this makes the number under it mean the same thing.
 *
 *  Belongs to `storedDepthLabel`, never `shownDepthLabel`: the value is about to be SENT. */
export function metresInStored(valueM: number): number {
  return convertDepth(valueM, "M", appState.projectDepthUnit.get());
}

/** Switches the unit depths are DISPLAYED in. Stored data is untouched — this is why the
 *  two units are separate settings in the first place. */
export function setDisplayDepthUnit(unit: DepthUnit): void {
  if (appState.displayDepthUnit.get() === unit) return;
  appState.displayDepthUnit.set(unit);
  try {
    localStorage.setItem(DISPLAY_KEY, unit);
  } catch {
    // Not persisting is survivable; the switch still applies for this session.
  }
  const stored = appState.projectDepthUnit.get();
  setStatus(
    unit === stored
      ? `Depths shown in ${unitLabel(unit)} — the unit they are stored in`
      : `Depths shown in ${unitLabel(unit)}, converted from the stored ${unitLabel(stored)} (data unchanged)`,
  );
}
