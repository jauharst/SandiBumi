import { listWellGroups, listWells, type WellGroupEntry, type WellSummary } from "../ipc";
import { appState } from "../state";

/** Shared "which wells does this run cover" selector — the one control every batch dialog uses
 *  instead of hand-rolling a checkbox-per-well list. At field scale (2000+ wells) ticking wells
 *  one by one is unworkable, so the default is a *scope*, resolved live:
 *
 *    • Group     — any saved well group (defaults to the active one)
 *    • ★ Pinned  — the persisted pinned favourites (`appState.pinnedWellIds`)
 *    • Selection — the current multi-selection from the Wells pane (`multiSelectedWellIds`)
 *    • All       — every well in the project
 *    • Custom…   — an escape-hatch searchable checklist for the rare precise pick
 *
 *  `getWellIds()` re-reads live state at call time, so pinning/selecting more wells and then
 *  hitting Run just works. Counts update live while the dialog is open. */

export type ScopeMode = "active" | "group" | "pinned" | "selection" | "all" | "custom";

export interface WellScope {
  /** The self-contained control block — append it straight into the dialog (it is its own row). */
  el: HTMLElement;
  /** The wells the run should cover right now, resolved from live state. */
  getWellIds(): string[];
  /** Resolve well ids (e.g. the exact set a run used) to their display names. */
  namesFor(ids: string[]): string[];
  /** How many wells are currently in scope. */
  count(): number;
  /** Short label for a toolbar button ("Active", "All (142)", the group name…). */
  describe(): string;
  /** Round-trippable spec ("active", "group:<id>", "custom:<id,id>", …) for getState. */
  serialize(): string;
  /** Detach live-state subscriptions. Call from the dialog's dispose(). */
  dispose(): void;
}

export interface WellScopeOptions {
  /** Fired whenever the resolved set changes (mode switch, group pick, custom edit, live state). */
  onChange?: (ids: string[]) => void;
  /** Force a starting mode instead of the smart default. */
  defaultMode?: ScopeMode;
  /** Offer an "Active" mode resolving to the globally selected well (plot scopes use it;
   *  batch dialogs keep their multi-well modes only). */
  includeActive?: boolean;
  /** A spec previously returned by serialize() — restores mode/group/custom picks. Wins
   *  over defaultMode when it parses. */
  initial?: string;
}

export async function buildWellScope(opts: WellScopeOptions = {}): Promise<WellScope> {
  const wells: WellSummary[] = await listWells().catch(() => []);
  const groups: WellGroupEntry[] = await listWellGroups().catch(() => []);
  const wellById = new Map(wells.map((w) => [w.well_id, w] as const));
  const allIds = wells.map((w) => w.well_id);

  // Custom-mode explicit picks (seeded from whatever the previous mode resolved to).
  let customIds = new Set<string>();
  // Currently chosen group id for group mode.
  let groupId: string | null = appState.activeWellGroup.get()?.group_id ?? groups[0]?.group_id ?? null;

  const hasGroups = groups.length > 0;
  const pins = () => appState.pinnedWellIds.get().filter((id) => wellById.has(id));
  const selection = () => appState.multiSelectedWellIds.get().filter((id) => wellById.has(id));

  // Smart default: the most specific scope that is currently populated.
  const smartDefault = (): ScopeMode => {
    if (appState.activeWellGroup.get() && hasGroups) return "group";
    if (selection().length > 0) return "selection";
    if (pins().length > 0) return "pinned";
    return "all";
  };
  let mode: ScopeMode = opts.defaultMode ?? smartDefault();
  // Restore a serialized spec ("active" / "all" / "pinned" / "selection" / "group:<id>" /
  // "custom:<id,id>") — stale ids (deleted group, removed wells) fall through to the default.
  if (opts.initial) {
    const [head, rest] = [opts.initial.split(":", 1)[0], opts.initial.slice(opts.initial.indexOf(":") + 1)];
    if (head === "group" && rest && groups.some((g) => g.group_id === rest)) {
      mode = "group";
      groupId = rest;
    } else if (head === "custom") {
      mode = "custom";
      customIds = new Set(rest.split(",").filter((id) => wellById.has(id)));
    } else if (["active", "all", "pinned", "selection"].includes(head)) {
      mode = head as ScopeMode;
    }
  }
  if (mode === "group" && !hasGroups) mode = "all";
  if (mode === "active" && !opts.includeActive) mode = "all";

  // --- DOM -----------------------------------------------------------------
  const el = document.createElement("div");
  el.className = "well-scope";

  const head = document.createElement("div");
  head.className = "well-scope-head";
  const label = document.createElement("span");
  label.className = "well-scope-label";
  label.textContent = "Wells";
  const modesBox = document.createElement("div");
  modesBox.className = "well-scope-modes";
  const countEl = document.createElement("span");
  countEl.className = "well-scope-count";
  head.append(label, modesBox, countEl);

  const detail = document.createElement("div");
  detail.className = "well-scope-detail";
  el.append(head, detail);

  const MODES: { key: ScopeMode; label: string; show: boolean }[] = [
    { key: "active", label: "Active", show: !!opts.includeActive },
    { key: "group", label: "Group", show: hasGroups },
    { key: "pinned", label: "★ Pinned", show: true },
    { key: "selection", label: "Selection", show: true },
    { key: "all", label: "All", show: true },
    { key: "custom", label: "Custom…", show: true },
  ];
  const modeBtns = new Map<ScopeMode, HTMLButtonElement>();
  for (const m of MODES) {
    if (!m.show) continue;
    const b = document.createElement("button");
    b.type = "button";
    b.className = "well-scope-mode";
    b.textContent = m.label;
    b.addEventListener("click", () => setMode(m.key));
    modesBox.appendChild(b);
    modeBtns.set(m.key, b);
  }

  // --- Group detail: a dropdown of saved groups ----------------------------
  const groupSel = document.createElement("select");
  groupSel.className = "well-scope-group-sel";
  for (const g of groups) {
    const o = document.createElement("option");
    o.value = g.group_id;
    o.textContent = `${g.name} (${g.member_count})`;
    if (g.group_id === groupId) o.selected = true;
    groupSel.appendChild(o);
  }
  groupSel.addEventListener("change", () => {
    groupId = groupSel.value || null;
    emit();
  });

  // --- Custom detail: search + checklist -----------------------------------
  const customWrap = document.createElement("div");
  customWrap.className = "well-scope-custom";
  const customTools = document.createElement("div");
  customTools.className = "well-scope-custom-tools";
  const search = document.createElement("input");
  search.type = "search";
  search.placeholder = "filter wells…";
  search.className = "well-scope-search";
  const allBtn = miniBtn("Select shown", () => {
    for (const w of shownCustom()) customIds.add(w.well_id);
    renderCustomList();
    emit();
  });
  const noneBtn = miniBtn("Clear", () => {
    customIds.clear();
    renderCustomList();
    emit();
  });
  customTools.append(search, allBtn, noneBtn);
  const customList = document.createElement("div");
  customList.className = "well-scope-custom-list";
  customWrap.append(customTools, customList);
  search.addEventListener("input", renderCustomList);

  function shownCustom(): WellSummary[] {
    const q = search.value.trim().toLowerCase();
    return q ? wells.filter((w) => w.well_name.toLowerCase().includes(q)) : wells;
  }

  function renderCustomList(): void {
    customList.innerHTML = "";
    const shown = shownCustom();
    if (shown.length === 0) {
      const e = document.createElement("div");
      e.className = "well-scope-empty";
      e.textContent = "No wells match.";
      customList.appendChild(e);
      return;
    }
    for (const w of shown) {
      const lab = document.createElement("label");
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.checked = customIds.has(w.well_id);
      cb.addEventListener("change", () => {
        if (cb.checked) customIds.add(w.well_id);
        else customIds.delete(w.well_id);
        emit();
      });
      lab.append(cb, document.createTextNode(` ${w.well_name}`));
      customList.appendChild(lab);
    }
  }

  // --- Resolution ----------------------------------------------------------
  function resolveIds(): string[] {
    switch (mode) {
      case "active": {
        const w = appState.selectedWell.get();
        return w && wellById.has(w.well_id) ? [w.well_id] : [];
      }
      case "group": {
        const g = groups.find((x) => x.group_id === groupId);
        return g ? g.well_ids.filter((id) => wellById.has(id)) : [];
      }
      case "pinned":
        return pins();
      case "selection":
        return selection();
      case "custom":
        return allIds.filter((id) => customIds.has(id));
      case "all":
      default:
        return allIds;
    }
  }

  function hintFor(m: ScopeMode, n: number): string {
    if (m === "active")
      return n ? "The active well only — follows the Wells pane selection." : "No well selected yet — click one in the Wells pane.";
    if (m === "pinned")
      return n ? "Running on your pinned wells — ★ toggle them in the Wells pane." : "No pinned wells yet — ★ some in the Wells pane, or pick another scope.";
    if (m === "selection")
      return n ? "Running on the wells selected in the Wells pane (Ctrl/Shift-click)." : "No wells selected — Ctrl-click wells in the Wells pane, or pick another scope.";
    if (m === "all") return "Running on every well in the project.";
    return "";
  }

  function renderDetail(): void {
    detail.innerHTML = "";
    if (mode === "group") {
      detail.appendChild(groupSel);
    } else if (mode === "custom") {
      detail.appendChild(customWrap);
      renderCustomList();
    } else {
      const hint = document.createElement("div");
      hint.className = "well-scope-hint";
      hint.textContent = hintFor(mode, resolveIds().length);
      detail.appendChild(hint);
    }
  }

  function updateCount(): void {
    const ids = resolveIds();
    countEl.textContent = `${ids.length} well${ids.length === 1 ? "" : "s"}`;
    countEl.classList.toggle("well-scope-count-zero", ids.length === 0);
    // The names on hover give quick confidence without a big list.
    countEl.title = ids
      .slice(0, 40)
      .map((id) => wellById.get(id)?.well_name ?? id)
      .join(", ") + (ids.length > 40 ? ` … (+${ids.length - 40})` : "");
  }

  function reflectMode(): void {
    for (const [k, b] of modeBtns) b.classList.toggle("well-scope-mode-active", k === mode);
  }

  function emit(): void {
    updateCount();
    // Re-render the live hint (selection/pinned counts change the message).
    if (mode !== "group" && mode !== "custom") renderDetail();
    opts.onChange?.(resolveIds());
  }

  function setMode(m: ScopeMode): void {
    // Carry the currently-resolved set into Custom so it opens from what you had.
    if (m === "custom" && mode !== "custom") customIds = new Set(resolveIds());
    mode = m;
    reflectMode();
    renderDetail();
    emit();
  }

  // Live: pinning/selecting while the dialog is open updates the count and, when relevant, the
  // scope. `Observable.subscribe` fires its listener SYNCHRONOUSLY on subscribe (state.ts:29), so
  // without a guard that first fire runs `emit()` — and therefore the caller's `onChange` — while
  // the caller is still parked on `await buildWellScope(...)`, before it has declared the `const`s
  // the callback closes over. reportDialog's onChange reads `batchBtn`, a not-yet-initialised const,
  // so that synthetic fire throws a TDZ ReferenceError out of subscribe, rejects this promise, and
  // leaves the Report pane stuck on "Failed to open". `ready` gates the callbacks so only genuine
  // post-construction changes emit; every caller does its own first paint (reportDialog sets the
  // batch label from getWellIds(), cutoffDialog awaits refreshZoneDst()), so nothing is lost. Same
  // primed-flag shape as plotCommon.ts:349 / mapPanel.ts:434.
  let ready = false;
  const unsub: Array<() => void> = [];
  unsub.push(appState.pinnedWellIds.subscribe(() => { if (ready && mode === "pinned") emit(); }));
  unsub.push(appState.multiSelectedWellIds.subscribe(() => { if (ready && mode === "selection") emit(); }));
  unsub.push(appState.selectedWell.subscribe(() => { if (ready && mode === "active") emit(); }));

  reflectMode();
  renderDetail();
  updateCount();
  ready = true;

  return {
    el,
    getWellIds: resolveIds,
    namesFor: (ids) => ids.map((id) => wellById.get(id)?.well_name ?? id),
    count: () => resolveIds().length,
    describe: () => {
      const n = resolveIds().length;
      switch (mode) {
        case "active":
          return "Active";
        case "group":
          return groups.find((g) => g.group_id === groupId)?.name ?? "Group";
        case "pinned":
          return `★ ${n}`;
        case "selection":
          return `Sel ${n}`;
        case "custom":
          return `${n} well${n === 1 ? "" : "s"}`;
        default:
          return `All (${n})`;
      }
    },
    serialize: () => {
      if (mode === "group") return groupId ? `group:${groupId}` : "all";
      if (mode === "custom") return `custom:${[...customIds].join(",")}`;
      return mode;
    },
    dispose: () => unsub.forEach((u) => u()),
  };
}

function miniBtn(text: string, onClick: () => void): HTMLButtonElement {
  const b = document.createElement("button");
  b.type = "button";
  b.className = "well-scope-mini";
  b.textContent = text;
  b.addEventListener("click", onClick);
  return b;
}
