import type { Layout, WellGroupEntry, WellSummary } from "./ipc";

/** Shared application state with a tiny pub/sub — replaces the closure variables the
 *  single-window main.ts used to hold. Dock panels subscribe to what they care about;
 *  anything may update state via the setters. */

type Listener<T> = (value: T) => void;

class Observable<T> {
  private value: T;
  private listeners = new Set<Listener<T>>();

  constructor(initial: T) {
    this.value = initial;
  }

  get(): T {
    return this.value;
  }

  set(value: T): void {
    this.value = value;
    for (const fn of this.listeners) fn(value);
  }

  /** Subscribes and immediately fires with the current value. Returns unsubscribe. */
  subscribe(fn: Listener<T>): () => void {
    this.listeners.add(fn);
    fn(this.value);
    return () => this.listeners.delete(fn);
  }
}

/** A depth window derived from the Tops pane: the clicked top down to the next top
 *  (depthMax null = down to TD). Plots window their data to it and log views scroll
 *  to it; cleared when a different well is selected. */
export interface TopInterval {
  wellId: string;
  topName: string;
  depthMin: number;
  depthMax: number | null;
}

export const appState = {
  /** The globally selected well (object tree click). */
  selectedWell: new Observable<WellSummary | null>(null),
  /** The top interval selected in the Wells & Tops pane (null = none). */
  selectedInterval: new Observable<TopInterval | null>(null),
  /** The active layout definition chosen in the ribbon's layout picker. New log views
   *  open with this layout; existing views keep their own copy. */
  activeLayout: new Observable<Layout | null>(null),
  /** Monotonic counter bumped whenever computed curves change (module run, equation run,
   *  pay summary) so open panels can refresh their data. */
  dataVersion: new Observable<number>(0),
  /** The depth under the cursor in whichever log view the mouse is over (null = none).
   *  Every open log view draws a synchronized crosshair at this depth. */
  hoverDepth: new Observable<number | null>(null),
  /** Monotonic counter bumped when the colour theme changes. Canvas-based panels (log
   *  views, correlation) read their colours from CSS variables at draw time, so they
   *  subscribe to this to repaint immediately on a theme switch instead of on next
   *  interaction. */
  themeVersion: new Observable<number>(0),
  /** The active well group filtering the whole workspace (null = "All wells"). When set,
   *  the Wells & Tops pane shows only its members and batch runs default to them — the
   *  way a 2000-well field stays workable. At most one group is active. */
  activeWellGroup: new Observable<WellGroupEntry | null>(null),
  /** Bumped whenever the set of groups or their membership changes, so the Wells pane and
   *  batch dialogs reload their group list. */
  wellGroupsVersion: new Observable<number>(0),
  /** Pin ON (default): selecting a well drives the whole workspace — every log view and
   *  plot follows. Pin OFF: viewers keep the well they're showing and only the ACTIVE
   *  panel follows the selection (Petrel-style "working pane" model), which is how
   *  side-by-side multi-well viewing works. Browsing panes (Tops, Inspector, DB
   *  Inspector) and dialogs always track the selection either way. */
  wellPinned: new Observable<boolean>(true),
  /** Wells multi-selected in the Wells & Tops pane (Ctrl-click toggle, Shift-click
   *  range, ⇄ invert); empty = no multi-selection. Batch dialogs pre-tick these
   *  instead of just the active well. */
  multiSelectedWellIds: new Observable<string[]>([]),
  /** Persisted "pinned" wells — a favourites subset independent of groups (the ★ toggle in the
   *  Wells pane). Reused by the shared well-scope selector as a one-click run scope. Loaded from
   *  the project on open and kept in sync as pins toggle. */
  pinnedWellIds: new Observable<string[]>([]),
};

/** The wells a run/batch dialog should pre-tick: the multi-selection when one exists
 *  (intersected with the wells actually shown), otherwise just the active well. */
export function defaultRunWellIds(wells: WellSummary[]): Set<string> {
  const multi = new Set(appState.multiSelectedWellIds.get());
  if (multi.size > 0) {
    return new Set(wells.filter((w) => multi.has(w.well_id)).map((w) => w.well_id));
  }
  const selected = appState.selectedWell.get();
  return new Set(selected ? [selected.well_id] : []);
}

export function bumpDataVersion(): void {
  appState.dataVersion.set(appState.dataVersion.get() + 1);
}

export function bumpWellGroupsVersion(): void {
  appState.wellGroupsVersion.set(appState.wellGroupsVersion.get() + 1);
}

/** Filters a well list to the active group's members; unchanged when no group is active. */
export function filterByActiveGroup(wells: WellSummary[]): WellSummary[] {
  const g = appState.activeWellGroup.get();
  if (!g) return wells;
  const ids = new Set(g.well_ids);
  return wells.filter((w) => ids.has(w.well_id));
}

/** The active group's member ids, or null when no group is active ("All wells"). Batch
 *  dialogs default their well selection to this. */
export function activeGroupWellIds(): Set<string> | null {
  const g = appState.activeWellGroup.get();
  return g ? new Set(g.well_ids) : null;
}

export function bumpThemeVersion(): void {
  appState.themeVersion.set(appState.themeVersion.get() + 1);
}

export function setStatus(text: string): void {
  const el = document.querySelector<HTMLElement>("#status-bar");
  if (el) el.textContent = text;
}
