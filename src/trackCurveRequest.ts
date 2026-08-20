/** One curve requested by the log viewer. A blank set means the established current
 * standard/computed/RAW resolver; an explicit set means that imported set's native grid. */
export interface TrackCurveRequest {
  curve_name: string;
  set_name?: string | null;
  /** True when the layout draws this curve as CLASS BLOCKS. The backend cannot see a style,
   * and min/max decimation is meaningless for a class index - see `class_curve` in
   * `equations.rs`. Optional so an older saved payload still means "ordinary curve". */
  class_curve?: boolean;
}

/** Stable renderer key. The unit-separator cannot occur in a set name entered through the
 * product UI, and keeps an unqualified curve backward-compatible (`GR` remains `GR`). */
export function trackCurveKey(request: TrackCurveRequest): string {
  const curve = request.curve_name.trim().toUpperCase();
  const set = request.set_name?.trim();
  return set ? `${set}\u001f${curve}` : curve;
}

/** Whether the current well inventory contains this exact explicit source identity. */
export function hasTrackCurve(available: TrackCurveRequest[], request: TrackCurveRequest): boolean {
  const set = request.set_name?.trim();
  if (!set) return true;
  const curve = request.curve_name.trim().toUpperCase();
  return available.some(
    (candidate) =>
      candidate.curve_name.trim().toUpperCase() === curve && candidate.set_name?.trim() === set,
  );
}

/** Sets that actually carry `curveName` in the current well. `currentSet` is retained even
 * when absent so opening a layout on a different well never silently rewrites its provenance. */
export function availableTrackSets(
  available: TrackCurveRequest[],
  curveName: string,
  currentSet?: string | null,
): string[] {
  const curve = curveName.trim().toUpperCase();
  const sets = new Set(
    available
      .filter((candidate) => candidate.curve_name.trim().toUpperCase() === curve)
      .map((candidate) => candidate.set_name?.trim())
      .filter((set): set is string => Boolean(set)),
  );
  const saved = currentSet?.trim();
  if (saved) sets.add(saved);
  return [...sets].sort();
}
