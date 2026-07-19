import { listWells, type WellSummary } from "../ipc";
import { appState, filterByActiveGroup, isSelectionBlocked, setPinnedWell, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { activateWellGroup, openWellGroupManager, syncWellGroups } from "./wellGroups";

/** Techlog/Geolog-style project object tree: Wells, and (later) their curves/zones.
 *  A group bar at the top scopes the list to the active well group (for large fields the
 *  user works one group at a time — see wellGroups.ts). */
export class ObjectTree {
  private container: HTMLElement;
  public onSelectWell: ((well: WellSummary) => void) | null = null;
  /** Highlighted well; kept across refreshes (the workspace seeds it from appState). */
  public selectedWellId: string | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  async refresh(): Promise<void> {
    this.container.innerHTML = "";

    // Groups + wells in parallel; syncWellGroups also publishes the active group to state.
    const [groups, allWells] = await Promise.all([
      syncWellGroups(),
      listWells().catch((err) => {
        console.error("Failed to load wells:", err);
        return null;
      }),
    ]);

    this.buildGroupBar(groups);

    if (allWells === null) {
      this.addEmptyNote("Unable to load wells");
      return;
    }

    const wells = filterByActiveGroup(allWells);
    const activeGroup = appState.activeWellGroup.get();
    this.addGroupLabel(activeGroup ? `Wells — ${activeGroup.name} (${wells.length})` : `Wells (${allWells.length})`);

    if (allWells.length === 0) {
      this.addEmptyNote("No wells ingested yet");
      return;
    }
    if (wells.length === 0) {
      this.addEmptyNote("No wells in this group — Edit wells to add some");
      return;
    }

    const pinnedId = appState.pinnedWellId.get();
    for (const well of wells) {
      const node = document.createElement("div");
      const isPinned = well.well_id === pinnedId;
      node.className =
        "tree-node tree-well" +
        (well.well_id === this.selectedWellId ? " tree-selected" : "") +
        (isPinned ? " tree-pinned" : "");
      const label = well.field_name ? `${well.well_name} (${well.field_name})` : well.well_name;
      node.textContent = isPinned ? `📌 ${label}` : label;
      node.title = isPinned ? `${well.well_id} — locked (pinned)` : well.well_id;
      node.addEventListener("click", () => {
        // While a well is locked, browsing other wells must not move the active well.
        if (isSelectionBlocked(well.well_id)) {
          setStatus("Active well is locked — click the 📌 lock to unpin before switching");
          return;
        }
        this.selectedWellId = well.well_id;
        for (const el of this.container.querySelectorAll(".tree-selected")) el.classList.remove("tree-selected");
        node.classList.add("tree-selected");
        this.onSelectWell?.(well);
      });
      this.container.appendChild(node);
    }
  }

  /** The active-group selector + manage button shown above the well list. */
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
    const pinnedId = appState.pinnedWellId.get();
    pinBtn.classList.toggle("active", pinnedId !== null);
    pinBtn.textContent = pinnedId !== null ? "📌" : "📍";
    pinBtn.title =
      pinnedId !== null
        ? "Active well is locked — click to unpin (let selection move again)"
        : "Lock the active well — every view, plot, and batch run stays on it";
    pinBtn.addEventListener("click", () => {
      if (appState.pinnedWellId.get() !== null) {
        setPinnedWell(null);
        setStatus("Active well unlocked — selection follows clicks again");
      } else {
        const well = appState.selectedWell.get();
        if (!well) {
          setStatus("Select a well first, then lock it");
          return;
        }
        setPinnedWell(well.well_id);
        setStatus(`Locked to ${well.well_name} — views and batch runs stay on it`);
        recordProcess("Pin", `Locked active well to ${well.well_name}`, well.well_name);
      }
      void this.refresh();
    });

    const manageBtn = document.createElement("button");
    manageBtn.className = "tree-group-manage";
    manageBtn.textContent = "⚙";
    manageBtn.title = "Manage well groups…";
    manageBtn.addEventListener("click", () => void openWellGroupManager());

    bar.append(select, pinBtn, manageBtn);
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
