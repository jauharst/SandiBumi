import { listPinnedWells, listWells, setWellPin, type WellSummary } from "../ipc";
import { appState, filterByActiveGroup, setStatus } from "../state";
import { activateWellGroup, openWellGroupManager, syncWellGroups } from "./wellGroups";

/** Project object tree: Wells, and (later) their curves/zones.
 *  A group bar at the top scopes the list to the active well group (for large fields the
 *  user works one group at a time — see wellGroups.ts).
 *
 *  Selection model (Petrel-style):
 *  - Plain click activates a well. With the 📌 pin ON (default) the whole workspace
 *    follows; with it OFF only the active panel follows (viewers hold their wells).
 *  - Ctrl-click toggles wells into a multi-selection, Shift-click selects a range,
 *    ⇄ inverts it within the visible list. The multi-selection feeds batch dialogs
 *    (module runs, workflows, ML, Monte Carlo, reports) as their pre-ticked wells.
 */
export class ObjectTree {
  private container: HTMLElement;
  public onSelectWell: ((well: WellSummary) => void) | null = null;
  /** Highlighted well; kept across refreshes (the workspace seeds it from appState). */
  public selectedWellId: string | null = null;
  /** Anchor for Shift-click range selection (index into the visible well list). */
  private anchorIndex = 0;
  private visibleWells: WellSummary[] = [];
  /** Bumped on every refresh; a stale in-flight refresh bails instead of double-rendering. */
  private refreshGen = 0;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  async refresh(): Promise<void> {
    const gen = ++this.refreshGen;

    // Groups + wells + pins in parallel; syncWellGroups also publishes the active group to state.
    const [groups, allWells, pinnedIds] = await Promise.all([
      syncWellGroups(),
      listWells().catch((err) => {
        console.error("Failed to load wells:", err);
        return null;
      }),
      listPinnedWells().catch(() => [] as string[]),
    ]);
    // A newer refresh started while we awaited — let it own the render. Without this, two
    // concurrent refreshes (init + the dataVersion subscription that fires immediately) both
    // clear-then-append and the pane shows the "Wells (N)" header — and every well — twice.
    if (gen !== this.refreshGen) return;
    this.container.innerHTML = "";
    appState.pinnedWellIds.set(pinnedIds);
    const pinned = new Set(pinnedIds);

    this.buildGroupBar(groups);

    if (allWells === null) {
      this.addEmptyNote("Unable to load wells");
      return;
    }

    const wells = filterByActiveGroup(allWells);
    this.visibleWells = wells;
    const multi = new Set(appState.multiSelectedWellIds.get());
    const activeGroup = appState.activeWellGroup.get();
    const base = activeGroup ? `Wells — ${activeGroup.name} (${wells.length})` : `Wells (${allWells.length})`;
    this.addGroupLabel(multi.size > 0 ? `${base} • ${multi.size} selected` : base);

    if (allWells.length === 0) {
      this.addEmptyNote("No wells ingested yet");
      return;
    }
    if (wells.length === 0) {
      this.addEmptyNote("No wells in this group — Edit wells to add some");
      return;
    }

    wells.forEach((well, index) => {
      const node = document.createElement("div");
      node.className =
        "tree-node tree-well" +
        (well.well_id === this.selectedWellId ? " tree-selected" : "") +
        (multi.has(well.well_id) ? " tree-multi" : "");
      const isPinned = pinned.has(well.well_id);
      const star = document.createElement("span");
      star.className = "tree-pin" + (isPinned ? " tree-pinned" : "");
      star.textContent = isPinned ? "★" : "☆";
      star.title = isPinned
        ? "Pinned — click to unpin. Pinned wells are reusable as a one-click run scope."
        : "Pin this well (favourites) — reusable as a one-click run scope in every tool.";
      star.addEventListener("click", (e) => {
        e.stopPropagation();
        void this.togglePin(well.well_id, !isPinned);
      });
      const labelSpan = document.createElement("span");
      labelSpan.className = "tree-well-label";
      labelSpan.textContent = well.field_name ? `${well.well_name} (${well.field_name})` : well.well_name;
      node.append(star, labelSpan);
      node.title = `${well.well_id}\nClick: activate • Ctrl-click: multi-select • Shift-click: range`;
      node.addEventListener("click", (e) => this.handleWellClick(e, well, index, node));
      this.container.appendChild(node);
    });
  }

  /** Pin / unpin a well (persisted) — updates state optimistically then refreshes the ★. */
  private async togglePin(wellId: string, pinned: boolean): Promise<void> {
    const next = new Set(appState.pinnedWellIds.get());
    if (pinned) next.add(wellId);
    else next.delete(wellId);
    appState.pinnedWellIds.set([...next]);
    await setWellPin(wellId, pinned).catch((e) => console.error("pin failed:", e));
    setStatus(pinned ? "Well pinned — available as a run scope" : "Well unpinned");
    void this.refresh();
  }

  private handleWellClick(e: MouseEvent, well: WellSummary, index: number, node: HTMLElement): void {
    if (e.ctrlKey || e.metaKey) {
      // Toggle in/out of the multi-selection without moving the active well, so a
      // batch set can be built while every view stays put.
      const multi = new Set(appState.multiSelectedWellIds.get());
      if (multi.has(well.well_id)) multi.delete(well.well_id);
      else multi.add(well.well_id);
      this.anchorIndex = index;
      this.setMulti(multi);
      return;
    }
    if (e.shiftKey) {
      const [from, to] = [Math.min(this.anchorIndex, index), Math.max(this.anchorIndex, index)];
      this.setMulti(new Set(this.visibleWells.slice(from, to + 1).map((w) => w.well_id)));
      return;
    }
    // Plain click: activate this well (and clear any multi-selection).
    this.anchorIndex = index;
    this.selectedWellId = well.well_id;
    if (appState.multiSelectedWellIds.get().length > 0) {
      appState.multiSelectedWellIds.set([]);
      void this.refresh();
    } else {
      for (const el of this.container.querySelectorAll(".tree-selected")) el.classList.remove("tree-selected");
      node.classList.add("tree-selected");
    }
    this.onSelectWell?.(well);
  }

  private setMulti(ids: Set<string>): void {
    appState.multiSelectedWellIds.set([...ids]);
    setStatus(
      ids.size > 0
        ? `${ids.size} well${ids.size > 1 ? "s" : ""} selected — batch dialogs will pre-tick them`
        : "Multi-selection cleared",
    );
    void this.refresh();
  }

  /** The active-group selector + pin/invert/manage buttons shown above the well list. */
  private buildGroupBar(groups: { group_id: string; name: string; active: boolean; member_count: number }[]): void {
    const bar = document.createElement("div");
    bar.className = "tree-group-bar";

    const select = document.createElement("select");
    select.className = "form-control tree-group-select";
    select.title = "Active well group — scopes the well list and batch runs";
    const allOpt = document.createElement("option");
    allOpt.value = "";
    allOpt.textContent = "All wells";
    select.appendChild(allOpt);
    for (const g of groups) {
      const opt = document.createElement("option");
      opt.value = g.group_id;
      opt.textContent = `${g.name} (${g.member_count})`;
      if (g.active) opt.selected = true;
      select.appendChild(opt);
    }
    select.addEventListener("change", () => {
      void activateWellGroup(select.value || null).then(() => this.refresh());
    });

    const pinBtn = document.createElement("button");
    pinBtn.className = "tree-group-manage tree-lock-btn";
    const pinned = appState.wellPinned.get();
    pinBtn.classList.toggle("active", pinned);
    pinBtn.textContent = "📌";
    pinBtn.title = pinned
      ? "Pin ON — selecting a well drives the whole workspace. Click to switch to working-pane mode (only the active panel follows)."
      : "Pin OFF — viewers keep their wells; only the active panel follows selection. Click to make everything follow again.";
    pinBtn.addEventListener("click", () => {
      const on = !appState.wellPinned.get();
      appState.wellPinned.set(on);
      setStatus(
        on
          ? "Pin ON — every view and plot follows the selected well"
          : "Pin OFF — only the active panel follows; other views keep their wells",
      );
      void this.refresh();
    });

    const invertBtn = document.createElement("button");
    invertBtn.className = "tree-group-manage";
    invertBtn.textContent = "⇄";
    invertBtn.title = "Invert the multi-selection within the visible wells";
    invertBtn.addEventListener("click", () => {
      const current = new Set(appState.multiSelectedWellIds.get());
      this.setMulti(new Set(this.visibleWells.filter((w) => !current.has(w.well_id)).map((w) => w.well_id)));
    });

    const manageBtn = document.createElement("button");
    manageBtn.className = "tree-group-manage";
    manageBtn.textContent = "⚙";
    manageBtn.title = "Manage well groups…";
    manageBtn.addEventListener("click", () => void openWellGroupManager());

    bar.append(select, pinBtn, invertBtn, manageBtn);
    this.container.appendChild(bar);
  }

  private addGroupLabel(label: string): void {
    const el = document.createElement("div");
    el.className = "tree-node tree-group";
    el.textContent = label;
    this.container.appendChild(el);
  }

  private addEmptyNote(text: string): void {
    const el = document.createElement("div");
    el.className = "tree-empty";
    el.textContent = text;
    this.container.appendChild(el);
  }
}
