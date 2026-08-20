import { getProjectDepthUnit } from "./ipc";
import { appState, setStatus } from "./state";
import type { DepthUnit } from "./units";
import { unitLabel } from "./units";

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
  return unit === PROJECT_DEPTH_UNIT_TOKEN ? unitLabel(appState.projectDepthUnit.get()) : unit;
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
