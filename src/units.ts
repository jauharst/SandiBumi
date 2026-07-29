/** Depth units on the frontend — the mirror of `src-tauri/src/units.rs`.
 *
 *  Two units, kept strictly apart:
 *
 *  **Stored unit** (`appState.projectDepthUnit`) — what every depth in the database is in.
 *  Fixed per project, declared at its first import, never changed while wells exist.
 *  Everything that comes back from the backend is in this unit.
 *
 *  **Display unit** (`appState.displayDepthUnit`) — what the user READS. Switchable at any
 *  moment, purely a view setting: it changes labels and the numbers shown, never data.
 *
 *  Keeping them apart is what makes the toggle safe. Converting on the way to the screen
 *  can't corrupt anything; converting on the way to the database can, which is why that
 *  path lives in Rust and happens once, at import.
 *
 *  Note the print scale does NOT follow the display unit: a "1:200" is a physical ratio of
 *  section length to paper length, so it depends on how long a STORED unit actually is.
 *  See `pxPerUnitAt1to1`.
 */

export type DepthUnit = "M" | "FT";

/** Exact international foot (NIST SP 811) — the same constant as `units.rs::M_PER_FT`. */
export const M_PER_FT = 0.3048;

export function unitLabel(u: DepthUnit): string {
  return u === "M" ? "m" : "ft";
}

/** Converts one depth between units. Non-finite passes through, so a missing depth stays
 *  missing rather than becoming a real number. */
export function convertDepth(value: number, from: DepthUnit, to: DepthUnit): number {
  if (from === to || !Number.isFinite(value)) return value;
  return from === "FT" ? value * M_PER_FT : value / M_PER_FT;
}

/** CSS px per STORED depth unit at a true 1:1 print scale.
 *
 *  96 CSS px/in ÷ 0.0254 m/in for metres; for feet it is exactly 96 × 12 = 1152 px/ft.
 *  This must key off the stored unit, not the display unit — the renderer's viewport is in
 *  stored units, and a "1:N" label claims N units of rock per unit of paper. Deriving it
 *  from metres unconditionally (as `PX_PER_UNIT_1_1` used to) mislabelled every named
 *  scale on a foot project by 3.28×.
 */
export function pxPerUnitAt1to1(stored: DepthUnit): number {
  return stored === "M" ? 96 / 0.0254 : 96 * 12;
}

/** Formats a stored depth for display, converting to the display unit. `decimals` follows
 *  the unit: a foot reading needs one fewer decimal than a metre one for the same
 *  precision, and depth axes are cleaner without spurious digits. */
export function formatDepth(
  storedValue: number,
  stored: DepthUnit,
  display: DepthUnit,
  decimals?: number,
): string {
  if (!Number.isFinite(storedValue)) return "—";
  const v = convertDepth(storedValue, stored, display);
  return v.toFixed(decimals ?? (display === "FT" ? 0 : 1));
}
