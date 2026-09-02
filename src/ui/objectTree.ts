import {
  deleteGenericCurve,
  getCoreData,
  getScalPc,
  getWellPath,
  listAuxData,
  listAuxSets,
  listCoreSets,
  listGenericCurveInventory,
  listImageSets,
  listWellImages,
  listPinnedWells,
  listScalSets,
  listSurveys,
  listWells,
  promoteGenericCurve,
  renameDeliverySet,
  setActiveAuxSet,
  setActiveCoreSet,
  setActiveImageSet,
  setActiveScalSet,
  setActiveSurvey,
  setWellPin,
  type AuxSetInfo,
  type CoreSetInfo,
  type GenericCurveInventoryEntry,
  type ImageSetInfo,
  type ScalSetInfo,
  type SurveyInfo,
  type WellSummary,
} from "../ipc";
import { appState, bumpDataVersion, filterByActiveGroup, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { activateWellGroup, openWellGroupManager, syncWellGroups } from "./wellGroups";
import { showContextMenu, type ContextMenuEntry } from "./contextMenu";
import { openCurveMetaDialog } from "./curveMetaDialog";
import { ensureSessionOperator } from "./runCustody";

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
 *
 *  Set browser (T-IMP-02): the ▸ twisty expands a well into its CURVE SETS (RAW, FPROOH,
 *  SANDIMIN, …) and each set into its curves — the Geolog/IP tree, so a well carrying
 *  several deliveries can be read without opening the Curve Catalog. Loaded LAZILY on
 *  first expand and cached per well: at 2000 wells, eagerly fetching every catalog would
 *  cost thousands of queries for a pane that shows one well's curves at a time.
 *
 *  Below the curve sets sit the well's CORE, SCAL, SURVEYS and POINT DATA (T-IMP-08/-12).
 *  They follow a different rule and the tree shows it: curve sets are all readable at once,
 *  but only ONE of each — and one set per point dataset — is live (●), because two
 *  deliveries measure the same plugs or the same samples. Double-click switches which —
 *  reversible, and the destructive half (delete) stays in Data → Tools ▾ → Data Sets…,
 *  never a stray click in a tree.
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
  /** Wells whose set subtree is expanded — survives a refresh so an import or a pin click
   *  doesn't collapse what the user opened. */
  private expandedWells = new Set<string>();
  /** Sets expanded to their curve list, keyed `${wellId} ${setName}` with a plain space separator. */
  private expandedSets = new Set<string>();
  /** Lazily fetched per-well curve inventory. Data-changing refreshes invalidate it so a
   *  fresh import appears; pure expand/collapse rerenders retain it and issue no sample scan. */
  private catalogCache = new Map<string, GenericCurveInventoryEntry[]>();
  /** The well's non-curve deliveries, fetched with the catalog and cached the same way. */
  private dataSetsCache = new Map<
    string,
    { core: CoreSetInfo[]; scal: ScalSetInfo[]; surveys: SurveyInfo[]; aux: AuxSetInfo[]; images: ImageSetInfo[] }
  >();
  /** Called after a set is switched from the tree, so panels reading the old set repaint. */
  public onDataChanged: (() => void) | null = null;
  /** Opens the Inspector's Curve Catalog, filtered — the tree's "inspect/edit values" route.
   *  Injected by the workspace, which owns the panel. */
  public onOpenCurveCatalog: ((mnemonic?: string) => void) | null = null;
  /** Opens the Database Inspector (raw table view) — where point/core values are edited. */
  public onOpenDbInspector: (() => void) | null = null;
  /** Opens Data Sets… for the given well (switch/delete deliveries). */
  public onManageDataSets: ((well: WellSummary) => void) | null = null;
  /** Non-curve sets expanded to their contents, keyed `${wellId} ${kind} ${name}` with a plain space separator. */
  private expandedData = new Set<string>();
  /** Lazily fetched contents of an expanded non-curve set, same key as `expandedData`.
   *  Cleared with the other caches on refresh so a re-import or a set switch is picked up. */
  private dataItemsCache = new Map<string, { label: string; detail: string }[]>();

  constructor(container: HTMLElement) {
    this.container = container;
  }

  async refresh(invalidateDataCaches = true): Promise<void> {
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
    // A refresh follows a data change (import, module run, delete), so every cached
    // catalog is potentially stale. Expanded wells refetch below; collapsed ones simply
    // fetch on their next expand.
    if (invalidateDataCaches) {
      this.catalogCache.clear();
      this.dataSetsCache.clear();
      this.dataItemsCache.clear();
    }
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
      // ▸/▾ twisty: expands the well into its curve sets. Its click must NOT reach the
      // row handler, or expanding would also change the active well and drag every
      // panel in the workspace to it.
      const twisty = document.createElement("span");
      twisty.className = "tree-twisty";
      const isOpen = this.expandedWells.has(well.well_id);
      twisty.textContent = isOpen ? "▾" : "▸";
      twisty.title = isOpen
        ? "Collapse"
        : "Show this well's curve sets, core, surveys and point data";
      twisty.addEventListener("click", (e) => {
        e.stopPropagation();
        if (this.expandedWells.has(well.well_id)) this.expandedWells.delete(well.well_id);
        else this.expandedWells.add(well.well_id);
        void this.refresh(false);
      });
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
      node.prepend(twisty);
      node.append(star, labelSpan);
      node.title = `${well.well_id}\nClick: activate • Ctrl-click: multi-select • Shift-click: range\n▸ shows curve sets, core, surveys and point data\nRight-click for actions`;
      node.addEventListener("click", (e) => this.handleWellClick(e, well, index, node));
      this.attachMenu(node, () => [
        { heading: well.well_name },
        {
          label: isOpen ? "Collapse" : "Expand (sets, core, surveys, point data)",
          onClick: () => {
            if (isOpen) this.expandedWells.delete(well.well_id);
            else this.expandedWells.add(well.well_id);
            void this.refresh(false);
          },
        },
        "sep",
        { label: "Open Curve Catalog", onClick: () => this.onOpenCurveCatalog?.() },
        { label: "Open Database Inspector", onClick: () => this.onOpenDbInspector?.() },
        { label: "Data Sets…", onClick: () => this.onManageDataSets?.(well) },
        "sep",
        {
          label: isPinned ? "Unpin well" : "Pin well (favourite)",
          onClick: () => void this.togglePin(well.well_id, !isPinned),
        },
      ]);
      this.container.appendChild(node);
      if (isOpen) void this.renderSets(well, node, gen);
    });
  }

  /** Renders one well's curve sets (and any expanded set's curves) directly under its row.
   *  Async because the catalog is fetched on demand; `gen` guards against a refresh that
   *  started while this was in flight appending rows to a tree that no longer exists. */
  private async renderSets(well: WellSummary, afterNode: HTMLElement, gen: number): Promise<void> {
    const placeholder = document.createElement("div");
    placeholder.className = "tree-node tree-set-loading";
    placeholder.textContent = "loading sets…";
    afterNode.insertAdjacentElement("afterend", placeholder);

    let curves = this.catalogCache.get(well.well_id);
    if (!curves) {
      try {
        curves = await listGenericCurveInventory(well.well_id);
        this.catalogCache.set(well.well_id, curves);
      } catch (err) {
        console.error("Failed to load curve sets:", err);
        if (gen === this.refreshGen) {
          placeholder.textContent = "unable to load curve sets";
          placeholder.classList.add("tree-empty");
        }
        return;
      }
    }
    if (gen !== this.refreshGen || !placeholder.isConnected) return;

    // Group by set, preserving the backend's ORDER BY set_name, family, mnemonic.
    const bySet = new Map<string, GenericCurveInventoryEntry[]>();
    for (const c of curves) {
      const list = bySet.get(c.set_name);
      if (list) list.push(c);
      else bySet.set(c.set_name, [c]);
    }

    const rows: HTMLElement[] = [];
    if (bySet.size === 0) {
      const empty = document.createElement("div");
      empty.className = "tree-node tree-set-empty";
      empty.textContent = "no curve sets — import a LAS or DLIS onto this well";
      rows.push(empty);
    }
    for (const [setName, entries] of bySet) {
      const key = `${well.well_id} ${setName}`;
      const open = this.expandedSets.has(key);
      const row = document.createElement("div");
      row.className = "tree-node tree-set";
      const tw = document.createElement("span");
      tw.className = "tree-twisty";
      tw.textContent = open ? "▾" : "▸";
      const label = document.createElement("span");
      label.className = "tree-set-label";
      label.textContent = `${setName} (${entries.length})`;
      row.append(tw, label);
      // `source` is uniform within a set in practice (one delivery = one import), so the
      // first entry's is representative; it tells LAS from DLIS at a glance.
      row.title = `${entries.length} curve(s)${entries[0]?.source ? ` — ${entries[0].source}` : ""}\nClick to ${open ? "hide" : "list"} them • Right-click for actions`;
      row.addEventListener("click", (e) => {
        e.stopPropagation();
        if (this.expandedSets.has(key)) this.expandedSets.delete(key);
        else this.expandedSets.add(key);
        void this.refresh(false);
      });
      this.attachMenu(row, () => [
        { heading: `Curve set ${setName}` },
        {
          label: open ? "Collapse" : `List its ${entries.length} curve(s)`,
          onClick: () => {
            if (open) this.expandedSets.delete(key);
            else this.expandedSets.add(key);
            void this.refresh(false);
          },
        },
        "sep",
        {
          label: "Open in Curve Catalog",
          onClick: () => this.onOpenCurveCatalog?.(setName),
        },
        "sep",
        {
          // RAW keeps absolute priority in curve resolution (rule 10a) — the backend refuses
          // to rename it in either direction, so the menu says why instead of offering a
          // click that can only fail (same convention as the pinned curve's entry below).
          label:
            setName === "RAW"
              ? "RAW cannot be renamed — it keeps absolute priority"
              : "Rename this set…",
          disabled: setName === "RAW",
          onClick: () => void this.renameCurveSet(well, setName),
        },
      ]);
      rows.push(row);

      if (open) {
        for (const c of entries) {
          const cr = document.createElement("div");
          cr.className = "tree-node tree-curve";
          const unit = c.unit ? ` [${c.unit}]` : "";
          cr.textContent = `${c.mnemonic}${unit}`;
          const bits = [
            c.family ? `family ${c.family}` : null,
            c.run_no !== null ? `run ${c.run_no}` : null,
            c.pinned ? "pinned (wins name resolution)" : null,
          ].filter(Boolean);
          cr.title = `${bits.join(" • ")}\nDouble-click to edit • Right-click for actions`;
          cr.addEventListener("click", (ev) => ev.stopPropagation());
          // Double-click = the fast path to the edit dialog, matching the tops editor and the
          // inspector grids (a single click must stay inert: these rows sit in the same list
          // as wells, and a stray click must not move the workspace).
          cr.addEventListener("dblclick", (ev) => {
            ev.stopPropagation();
            openCurveMetaDialog(c, () => void this.refresh());
          });
          this.attachMenu(cr, () => [
            { heading: `${c.mnemonic}${unit}` },
            {
              label: "Open in Curve Catalog (values, stats, provenance)",
              onClick: () => this.onOpenCurveCatalog?.(c.mnemonic),
            },
            { label: "Edit name / unit / family…", onClick: () => openCurveMetaDialog(c, () => void this.refresh()) },
            "sep",
            {
              label: c.pinned ? "Already wins its name" : "Make this curve win its name",
              disabled: c.pinned,
              onClick: () => {
                void promoteGenericCurve(c.curve_id)
                  .then(() => {
                    setStatus(`${c.mnemonic} now wins its mnemonic — modules asking for it read this curve`);
                    bumpDataVersion();
                    void this.refresh();
                  })
                  .catch((err) => setStatus(`Promote failed: ${err}`));
              },
            },
            "sep",
            {
              label: "Delete this curve…",
              danger: true,
              onClick: () => {
                if (
                  !window.confirm(
                    `Delete ${c.mnemonic} from set ${c.set_name}?\n\n` +
                      "This removes the imported curve and its values. It cannot be undone.",
                  )
                ) {
                  return;
                }
                void deleteGenericCurve(c.curve_id)
                  .then(() => {
                    setStatus(`Deleted ${c.mnemonic} from ${c.set_name}`);
                    bumpDataVersion();
                    void this.refresh();
                  })
                  .catch((err) => setStatus(`Delete failed: ${err}`));
              },
            },
          ]);
          rows.push(cr);
        }
      }
    }

    rows.push(...(await this.buildDataSetRows(well, gen)));

    // Insert as one block, in order, immediately after the well row.
    let anchor: HTMLElement = placeholder;
    for (const r of rows) {
      anchor.insertAdjacentElement("afterend", r);
      anchor = r;
    }
    placeholder.remove();
  }

  /** Renames a curve set from the tree — the one delivery kind Data Sets… doesn't manage,
   *  because curve sets are browsed here. Custody first (a rename is audited, so the
   *  operator is demanded before anything moves), then the backend moves every row that
   *  carries the name — curve_meta, the import-set registry and array logs — in one
   *  transaction, or refuses by name (RAW, a taken name). */
  private async renameCurveSet(well: WellSummary, oldName: string): Promise<void> {
    const entered = window.prompt(`Rename curve set ${oldName} to:`, oldName);
    const newName = entered?.trim();
    if (!newName || newName === oldName) return;
    const operator = await ensureSessionOperator("Rename delivery set");
    if (!operator) return;
    try {
      const receipt = await renameDeliverySet(
        "curve",
        well.well_id,
        null,
        oldName,
        newName,
        operator.identity,
        operator.kind,
        "Wells",
      );
      setStatus(`Renamed curve set ${oldName} → ${newName} (${receipt.rows_moved} row(s)) — audited.`);
      recordProcess("Edit", `Renamed curve set ${oldName} → ${newName}`, well.well_name);
      // Keep the expansion under its new name, so a rename doesn't collapse what was open.
      if (this.expandedSets.delete(`${well.well_id} ${oldName}`)) {
        this.expandedSets.add(`${well.well_id} ${newName}`);
      }
      this.onDataChanged?.();
      void this.refresh();
    } catch (err) {
      setStatus(String(err));
    }
  }

  /** Core sets, surveys and point-data sets for one well, as tree rows.
   *
   *  Only ONE of each is live, so the row shows ● / ○ rather than a twisty, and a
   *  double-click switches it. Single click is deliberately inert: these rows sit in the
   *  same list as wells and curve sets, and a stray click must never repoint what every
   *  panel reads. Deleting stays in the manager dialog. */
  private async buildDataSetRows(well: WellSummary, gen: number): Promise<HTMLElement[]> {
    let sets = this.dataSetsCache.get(well.well_id);
    if (!sets) {
      try {
        const [core, scal, surveys, aux, images] = await Promise.all([
          listCoreSets(well.well_id),
          listScalSets(well.well_id),
          listSurveys(well.well_id),
          listAuxSets(well.well_id),
          listImageSets(well.well_id),
        ]);
        sets = { core, scal, surveys, aux, images };
        this.dataSetsCache.set(well.well_id, sets);
      } catch (err) {
        console.error("Failed to load data sets:", err);
        return [];
      }
    }
    if (gen !== this.refreshGen) return [];

    const rows: HTMLElement[] = [];
    const addKind = (
      kind: string,
      entries: {
        label: string;
        detail: string;
        active: boolean;
        activate: () => Promise<unknown>;
        /** Loads what this delivery contains, for the ▸ twisty. */
        contents: () => Promise<{ label: string; detail: string }[]>;
        /** Identifies the expansion in `expandedData`; unique per well+kind+set. */
        key: string;
      }[],
    ): void => {
      if (entries.length === 0) return;
      const head = document.createElement("div");
      head.className = "tree-node tree-set-kind";
      head.textContent = kind;
      rows.push(head);
      for (const e of entries) {
        const open = this.expandedData.has(e.key);
        const row = document.createElement("div");
        row.className = "tree-node tree-dataset" + (e.active ? " tree-dataset-active" : "");
        // Two marks, because these rows carry two independent facts: a twisty for "what is
        // inside" (like a curve set) and ●/○ for "is this the live delivery" (unlike a curve
        // set, where every set is readable at once).
        const tw = document.createElement("span");
        tw.className = "tree-twisty";
        tw.textContent = open ? "▾" : "▸";
        tw.title = open ? "Collapse" : "Show what this delivery contains";
        tw.addEventListener("click", (ev) => {
          ev.stopPropagation();
          if (open) this.expandedData.delete(e.key);
          else this.expandedData.add(e.key);
          void this.refresh(false);
        });
        const mark = document.createElement("span");
        mark.className = "tree-dataset-mark";
        mark.textContent = e.active ? "●" : "○";
        const label = document.createElement("span");
        label.className = "tree-set-label";
        label.textContent = e.label;
        row.append(tw, mark, label);
        row.title = `${e.detail}\n${e.active ? "Active — this is what every panel reads" : "Double-click to make this the active one"}\n▸ shows its contents • Right-click for actions`;
        row.addEventListener("click", (ev) => ev.stopPropagation());
        const activate = (): void => {
          if (e.active) return;
          void e
            .activate()
            .then(() => {
              this.onDataChanged?.();
              void this.refresh();
            })
            .catch((err) => setStatus(String(err)));
        };
        row.addEventListener("dblclick", (ev) => {
          ev.stopPropagation();
          activate();
        });
        this.attachMenu(row, () => [
          { heading: `${kind} — ${e.label}` },
          {
            label: open ? "Collapse" : "Show its contents",
            onClick: () => {
              if (open) this.expandedData.delete(e.key);
              else this.expandedData.add(e.key);
              void this.refresh(false);
            },
          },
          {
            label: e.active ? "Already the live delivery" : "Make this the live delivery",
            disabled: e.active,
            onClick: activate,
          },
          "sep",
          { label: "Open Database Inspector (edit values)", onClick: () => this.onOpenDbInspector?.() },
          { label: "Data Sets… (rename scope, delete)", onClick: () => this.onManageDataSets?.(well) },
        ]);
        rows.push(row);

        if (open) {
          rows.push(...this.buildDataItemRows(e.key, e.contents, gen));
        }
      }
    };

    // Contents are read through the ACTIVE-set readers, so only the live delivery can be
    // expanded — an inactive one would otherwise show the active one's contents and read as
    // a lie. Inactive rows say so instead.
    const onlyWhenActive = async (
      active: boolean,
      load: () => Promise<{ label: string; detail: string }[]>,
    ): Promise<{ label: string; detail: string }[]> =>
      active ? load() : [{ label: "(not the live delivery)", detail: "Double-click the row above to make it live, then expand" }];

    addKind(
      "Core",
      sets.core.map((c) => ({
        key: `${well.well_id} core ${c.set_name}`,
        label: `${c.set_name} (${c.rows})`,
        detail: `${c.rows} plug(s)${c.source ? ` — ${c.source}` : ""}`,
        active: c.active,
        activate: async () => {
          await setActiveCoreSet(well.well_id, c.set_name);
          setStatus(`Core set ${c.set_name} is now active for ${well.well_name}.`);
        },
        // Which measured properties this core actually carries, and how many plugs have each
        // — the core equivalent of listing a curve set's curves.
        contents: () =>
          onlyWhenActive(c.active, async () => {
            const series = await getCoreData(well.well_id);
            return series
              .filter((s) => s.value.length > 0)
              .map((s) => ({
                label: `${s.curve_name} (${s.value.length})`,
                detail: `${s.value.length} plug(s) carry ${s.curve_name}`,
              }));
          }),
      })),
    );
    addKind(
      "SCAL",
      sets.scal.map((s) => ({
        key: `${well.well_id} scal ${s.set_name}`,
        label: `${s.set_name} (${s.rows})`,
        detail: `${s.rows} Pc point(s)${s.source ? ` — ${s.source}` : ""}`,
        active: s.active,
        activate: async () => {
          await setActiveScalSet(well.well_id, s.set_name);
          setStatus(`SCAL set ${s.set_name} is now active for ${well.well_name}.`);
        },
        // One row per PLUG (a Pc curve belongs to a sample), with its point count.
        contents: () =>
          onlyWhenActive(s.active, async () => {
            const pts = await getScalPc(well.well_id);
            const bySample = new Map<string, { n: number; depth: number | null }>();
            for (const p of pts) {
              const k = p.sample_no !== null ? `Sample ${p.sample_no}` : p.depth !== null ? `@ ${p.depth}` : "unlabelled";
              const cur = bySample.get(k);
              if (cur) cur.n += 1;
              else bySample.set(k, { n: 1, depth: p.depth });
            }
            return [...bySample].map(([k, v]) => ({
              label: `${k} (${v.n})`,
              detail: `${v.n} Pc/Sw point(s)${v.depth !== null ? ` at ${v.depth}` : ""}`,
            }));
          }),
      })),
    );
    addKind(
      "Surveys",
      sets.surveys.map((s) => ({
        key: `${well.well_id} survey ${s.survey_name}`,
        label: `${s.survey_name} (${s.stations})`,
        detail: `${s.stations} station(s)${s.datum === null ? "" : `, datum ${s.datum}`}${s.source ? ` — ${s.source}` : ""}`,
        active: s.active,
        activate: async () => {
          const n = await setActiveSurvey(well.well_id, s.survey_name);
          setStatus(`Survey ${s.survey_name} is active; TVD/TVDSS rebuilt (${n} sample(s)).`);
        },
        // A survey's shape, not 300 station rows: the numbers you check it by.
        contents: () =>
          onlyWhenActive(s.active, async () => {
            const path = await getWellPath(well.well_id);
            if (path.length === 0) return [{ label: "(no stations)", detail: "" }];
            const last = path[path.length - 1];
            const maxInc = path.reduce((m, p) => Math.max(m, p.inc), 0);
            return [
              { label: `${path.length} stations`, detail: "minimum-curvature" },
              { label: `MD ${path[0].md.toFixed(1)} → ${last.md.toFixed(1)}`, detail: "measured-depth range" },
              { label: `TVD at TD ${last.tvd.toFixed(1)}`, detail: "true vertical depth at the last station" },
              { label: `max inclination ${maxInc.toFixed(1)}°`, detail: "0° = vertical" },
            ];
          }),
      })),
    );
    addKind(
      "Point data",
      sets.aux.map((a) => ({
        key: `${well.well_id} aux ${a.dataset} ${a.set_name}`,
        label: `${a.dataset} · ${a.set_name} (${a.rows})`,
        detail: `${a.rows} value(s)${a.source ? ` — ${a.source}` : ""}`,
        active: a.active,
        activate: async () => {
          await setActiveAuxSet(well.well_id, a.dataset, a.set_name);
          setStatus(`${a.dataset} set ${a.set_name} is now active for ${well.well_name}.`);
        },
        // The named ITEMS this dataset measures (XRD minerals, CEC values, core extras) —
        // the point-data equivalent of a curve set's curves, which is exactly the parallel
        // the tree is meant to draw.
        contents: () =>
          onlyWhenActive(a.active, async () => {
            const rows = await listAuxData(well.well_id, a.dataset);
            const byItem = new Map<string, { n: number; num: number }>();
            for (const r of rows) {
              const cur = byItem.get(r.item) ?? { n: 0, num: 0 };
              cur.n += 1;
              if (r.value_num !== null) cur.num += 1;
              byItem.set(r.item, cur);
            }
            return [...byItem]
              .sort((x, y) => x[0].localeCompare(y[0]))
              .map(([item, v]) => ({
                label: `${item} (${v.n})`,
                detail: v.num === v.n ? `${v.n} numeric value(s)` : `${v.n} value(s), ${v.n - v.num} textual`,
              }));
          }),
      })),
    );
    addKind(
      "Images",
      sets.images.map((im) => ({
        key: `${well.well_id} img ${im.dataset} ${im.set_name}`,
        label: `${im.dataset} · ${im.set_name} (${im.images})`,
        detail: `${im.images} picture(s), ${(im.bytes / 1048576).toFixed(1)} MB${im.source ? ` — ${im.source}` : ""}`,
        active: im.active,
        activate: async () => {
          await setActiveImageSet(well.well_id, im.dataset, im.set_name);
          setStatus(`${im.dataset} image set ${im.set_name} is now active for ${well.well_name}.`);
        },
        // The plates themselves, at their depths. Metadata only — expanding a delivery must
        // never pull a gigabyte of pixels into the tree.
        contents: () =>
          onlyWhenActive(im.active, async () => {
            const pics = await listWellImages(well.well_id, im.dataset);
            return pics.map((p) => ({
              label: `${p.name} @ ${p.depth_top}${p.depth_base === null ? "" : `–${p.depth_base}`}`,
              detail:
                `${p.width}×${p.height}, ${(p.bytes / 1024).toFixed(0)} kB` +
                (p.printable ? "" : " — prints as a labelled frame (not a JPEG)"),
            }));
          }),
      })),
    );
    return rows;
  }

  /** Rows for one expanded non-curve delivery. Returns a placeholder immediately and fills it
   *  in when the load resolves — the tree is rebuilt synchronously, so this cannot await. */
  private buildDataItemRows(
    key: string,
    load: () => Promise<{ label: string; detail: string }[]>,
    gen: number,
  ): HTMLElement[] {
    const cached = this.dataItemsCache.get(key);
    if (cached) return cached.map((i) => this.dataItemRow(i));

    const placeholder = document.createElement("div");
    placeholder.className = "tree-node tree-set-loading";
    placeholder.textContent = "loading…";
    void load()
      .then((items) => {
        if (gen !== this.refreshGen || !placeholder.isConnected) return;
        this.dataItemsCache.set(key, items);
        const rows = items.length > 0 ? items.map((i) => this.dataItemRow(i)) : [this.dataItemRow({ label: "(empty)", detail: "" })];
        let anchor: HTMLElement = placeholder;
        for (const r of rows) {
          anchor.insertAdjacentElement("afterend", r);
          anchor = r;
        }
        placeholder.remove();
      })
      .catch((err) => {
        if (!placeholder.isConnected) return;
        placeholder.textContent = `unable to load: ${err}`;
        placeholder.classList.add("tree-empty");
      });
    return [placeholder];
  }

  private dataItemRow(item: { label: string; detail: string }): HTMLElement {
    const el = document.createElement("div");
    el.className = "tree-node tree-data-item";
    el.textContent = item.label;
    el.title = item.detail;
    el.addEventListener("click", (ev) => ev.stopPropagation());
    this.attachMenu(el, () => [
      { heading: item.label },
      { label: "Open Database Inspector (edit values)", onClick: () => this.onOpenDbInspector?.() },
    ]);
    return el;
  }

  /** Right-click → context menu, built fresh on each open so it reflects current state.
   *  `stopPropagation` keeps the app-level guard (which suppresses the browser menu on tree
   *  rows) from also firing, and stops a row's menu leaking to the well row behind it. */
  private attachMenu(el: HTMLElement, entries: () => ContextMenuEntry[]): void {
    el.addEventListener("contextmenu", (ev) => {
      ev.preventDefault();
      ev.stopPropagation();
      showContextMenu(ev.clientX, ev.clientY, entries());
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
