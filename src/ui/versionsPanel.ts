import {
  listLogSets,
  listLogSetNames,
  previewVersionPurge,
  purgeLogSetVersions,
  restoreLogSet,
  setLogSetComment,
  type LogSetEntry,
  type VersionPurgePreview,
} from "../ipc";
import { activeGroupWellIds, appState, bumpDataVersion } from "../state";
import { recordProcess } from "../processLog";
import { ensureSessionOperator } from "./runCustody";

/** Versions — the working pane over a well's interpretation history.
 *
 *  Every module run is kept as a version (nothing is overwritten); this pane is where that
 *  history is READ, labelled, restored, and — since increment 1 of the retention work — purged.
 *  Purging is two-phase on purpose: a preview states exactly which versions would go and names
 *  every refusal (the latest of a lineage, and any version the live interpretation still reads),
 *  then the purge deletes precisely that list, audited under the session operator. A purge frees
 *  no disk by itself — the receipt says so and points at Compact Project, which does.
 */
export async function buildVersionsContent(
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const content = document.createElement("div");
  content.className = "module-pane";

  const intro = document.createElement("div");
  intro.className = "eq-note";
  intro.textContent =
    "Every run is kept as a version - nothing is overwritten. Label versions so a report can cite " +
    "them, restore an older one (as a NEW version - history never rewinds), or purge superseded " +
    "ones. A purge previews first, never touches the latest or live version, and is written to " +
    "the audit trail. Disk space is reclaimed afterwards by Compact Project (Data ribbon).";
  content.appendChild(intro);

  // ---- the selected well's lineage table ----------------------------------
  const wellTitle = document.createElement("h4");
  wellTitle.className = "field-label";
  content.appendChild(wellTitle);

  const tableHost = document.createElement("div");
  tableHost.style.overflowX = "auto";
  content.appendChild(tableHost);

  /** set_id -> checked, surviving reloads within one pane lifetime. */
  const checked = new Set<string>();
  let entries: LogSetEntry[] = [];
  /** A failed history fetch, shown in place of the table. An error must never render as
   *  the "no versions yet" empty state - that would read as a fact about the well. */
  let loadError: string | null = null;

  const cellText = (text: string): HTMLTableCellElement => {
    const td = document.createElement("td");
    td.textContent = text;
    return td;
  };

  const renderTable = (): void => {
    tableHost.textContent = "";
    if (!appState.selectedWell.get()) {
      const empty = document.createElement("p");
      empty.className = "modal-hint";
      empty.textContent = "Select a well in the Wells pane to see its version history.";
      tableHost.appendChild(empty);
      return;
    }
    if (loadError) {
      const failed = document.createElement("p");
      failed.className = "modal-hint";
      failed.textContent = `Version history could not be read: ${loadError}`;
      tableHost.appendChild(failed);
      return;
    }
    if (entries.length === 0) {
      const empty = document.createElement("p");
      empty.className = "modal-hint";
      empty.textContent = "No computed versions on this well yet - run a module first.";
      tableHost.appendChild(empty);
      return;
    }
    const table = document.createElement("table");
    table.className = "versions-table";
    const head = document.createElement("tr");
    for (const label of ["", "Version", "Module", "Created", "State", "Curves", "Label", ""]) {
      const th = document.createElement("th");
      th.textContent = label;
      head.appendChild(th);
    }
    table.appendChild(head);

    // Newest first per set (backend order). The newest row of each lineage is the one the
    // backend will refuse anyway, so its checkbox is disabled with the reason as its tooltip -
    // the refusal is still enforced server-side, this only saves a dead-end preview.
    const seenSets = new Set<string>();
    const newestIds = new Set<string>();
    for (const entry of entries) {
      if (!seenSets.has(entry.set_name)) {
        seenSets.add(entry.set_name);
        newestIds.add(entry.set_id);
      }
    }

    for (const entry of entries) {
      const row = document.createElement("tr");
      const pickCell = document.createElement("td");
      const pick = document.createElement("input");
      pick.type = "checkbox";
      pick.checked = checked.has(entry.set_id);
      pick.setAttribute("aria-label", `Purge ${entry.set_name}_${entry.version}`);
      const latest = newestIds.has(entry.set_id);
      if (latest || entry.is_current) {
        pick.disabled = true;
        const reason = latest
          ? "The latest version of a lineage is never purged"
          : "The live interpretation still reads this version";
        pick.title = reason;
        // A disabled control is unfocusable, so the hover tooltip is its only voice; the
        // reason also reaches keyboard and screen-reader users through the preview's
        // refusal list, which the backend re-states regardless of this pre-disable.
        pick.setAttribute("aria-label", `${entry.set_name}_${entry.version}: ${reason}`);
      }
      pick.addEventListener("change", () => {
        if (pick.checked) checked.add(entry.set_id);
        else checked.delete(entry.set_id);
      });
      pickCell.appendChild(pick);
      row.appendChild(pickCell);

      const name = document.createElement("td");
      name.textContent = `${entry.set_name}_${entry.version}`;
      if (entry.is_current) {
        const live = document.createElement("span");
        live.textContent = " ●";
        live.title = "Current: live curves still come from this version";
        live.style.color = "var(--accent-text)";
        name.appendChild(live);
      }
      row.appendChild(name);
      row.appendChild(cellText(entry.module));
      row.appendChild(cellText(entry.created_at));
      row.appendChild(cellText(entry.outcome_state ?? "-"));
      row.appendChild(cellText(entry.curve_names.join(", ")));

      // Label: the per-version comment, edited in place. The backend refuses a blank label,
      // so a label is overwritten rather than cleared.
      const labelCell = document.createElement("td");
      const labelInput = document.createElement("input");
      labelInput.className = "form-control";
      labelInput.style.minWidth = "10em";
      labelInput.value = entry.comment ?? "";
      labelInput.placeholder = "label (Enter saves)";
      labelInput.setAttribute("aria-label", `Label for ${entry.set_name}_${entry.version}`);
      labelInput.addEventListener("keydown", (event) => {
        if (event.key !== "Enter") return;
        const text = labelInput.value.trim();
        if (!text) return;
        setLogSetComment(entry.set_id, text)
          .then(() => {
            setStatus(`Labelled ${entry.set_name}_${entry.version}: ${text}`);
            void reload();
          })
          .catch((error) => setStatus(String(error)));
      });
      labelCell.appendChild(labelInput);
      row.appendChild(labelCell);

      const actionCell = document.createElement("td");
      if (!entry.is_current) {
        const restore = document.createElement("button");
        restore.type = "button";
        restore.className = "btn";
        restore.textContent = "Restore";
        restore.title = "Copies this version back as a NEW version - versions 1..N stay intact";
        restore.addEventListener("click", () => {
          restoreLogSet(entry.set_id)
            .then((result) => {
              bumpDataVersion();
              setStatus(
                `Restored ${entry.set_name}_${entry.version} as version ${result.new_version} (${result.rows_restored} rows)`,
              );
              recordProcess(
                "Versions",
                `Restored ${entry.set_name}_${entry.version} as version ${result.new_version}`,
                appState.selectedWell.get()?.well_name,
              );
              void reload();
            })
            .catch((error) => setStatus(String(error)));
        });
        actionCell.appendChild(restore);
      }
      row.appendChild(actionCell);
      table.appendChild(row);
    }
    tableHost.appendChild(table);
  };

  const reload = async (): Promise<void> => {
    const well = appState.selectedWell.get();
    wellTitle.textContent = well ? `Version history — ${well.well_name}` : "Version history";
    loadError = null;
    entries = [];
    if (well) {
      try {
        entries = await listLogSets(well.well_id);
      } catch (error) {
        loadError = String(error);
      }
    }
    const known = new Set(entries.map((entry) => entry.set_id));
    for (const id of Array.from(checked)) if (!known.has(id)) checked.delete(id);
    renderTable();
  };

  // ---- purge selection ----------------------------------------------------
  const purgeTitle = document.createElement("h4");
  purgeTitle.className = "field-label";
  purgeTitle.textContent = "Purge superseded versions";
  content.appendChild(purgeTitle);

  const modeRow = document.createElement("div");
  modeRow.className = "seg";
  let mode: "checked" | "keep" = "checked";
  const modeButtons = new Map<string, HTMLButtonElement>();
  for (const [value, label] of [
    ["checked", "Checked versions"],
    ["keep", "Keep latest N"],
  ] as const) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "seg-opt" + (value === mode ? " active" : "");
    button.textContent = label;
    button.addEventListener("click", () => {
      mode = value;
      for (const [key, other] of modeButtons) other.classList.toggle("active", key === mode);
      keepControls.hidden = mode !== "keep";
      clearPreview();
    });
    modeButtons.set(value, button);
    modeRow.appendChild(button);
  }
  content.appendChild(modeRow);

  const keepControls = document.createElement("div");
  keepControls.hidden = true;
  keepControls.style.display = "flex";
  keepControls.style.gap = "0.75em";
  keepControls.style.alignItems = "center";
  keepControls.style.flexWrap = "wrap";

  const keepInput = document.createElement("input");
  keepInput.type = "number";
  keepInput.className = "form-control";
  keepInput.min = "1";
  keepInput.value = "1";
  keepInput.style.width = "5em";
  keepInput.title = "How many newest versions each lineage keeps (1 = only the latest lives)";

  let keepScope: "well" | "all" = "well";
  const scopeRow = document.createElement("div");
  scopeRow.className = "seg";
  const scopeButtons = new Map<string, HTMLButtonElement>();
  for (const [value, label] of [
    ["well", "This well"],
    ["all", "All wells"],
  ] as const) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "seg-opt" + (value === keepScope ? " active" : "");
    button.textContent = label;
    button.addEventListener("click", () => {
      keepScope = value;
      for (const [key, other] of scopeButtons) other.classList.toggle("active", key === keepScope);
      clearPreview();
    });
    scopeButtons.set(value, button);
    scopeRow.appendChild(button);
  }

  const setSelect = document.createElement("select");
  setSelect.className = "form-control";
  const anySet = document.createElement("option");
  anySet.value = "";
  anySet.textContent = "(every set)";
  setSelect.appendChild(anySet);
  listLogSetNames()
    .then((names) => {
      for (const name of names) {
        const option = document.createElement("option");
        option.value = name;
        option.textContent = name;
        setSelect.appendChild(option);
      }
    })
    .catch(() => {});

  const keepLabel = document.createElement("span");
  keepLabel.className = "modal-hint";
  keepLabel.textContent = "Keep newest";
  keepControls.append(keepLabel, keepInput, scopeRow, setSelect);
  content.appendChild(keepControls);

  const previewBtn = document.createElement("button");
  previewBtn.type = "button";
  previewBtn.className = "btn";
  previewBtn.textContent = "Preview purge";
  content.appendChild(previewBtn);

  // ---- preview + commit ---------------------------------------------------
  const previewHost = document.createElement("div");
  content.appendChild(previewHost);
  /** The exact ids the shown preview covered - the ONLY thing the purge button may send. */
  let previewedIds: string[] = [];

  const clearPreview = (): void => {
    previewHost.textContent = "";
    previewedIds = [];
  };

  const renderPreview = (preview: VersionPurgePreview): void => {
    clearPreview();
    previewedIds = preview.candidates.map((candidate) => candidate.set_id);
    const box = document.createElement("div");
    box.className = "eq-note";

    const heading = document.createElement("p");
    heading.textContent =
      preview.candidates.length === 0
        ? "Nothing qualifies for this selection."
        : `${preview.candidates.length} version(s) would be purged — ${preview.total_archived_rows} archived rows:`;
    box.appendChild(heading);

    if (preview.candidates.length > 0) {
      const list = document.createElement("ul");
      for (const candidate of preview.candidates) {
        const item = document.createElement("li");
        item.textContent = `${candidate.set_name}_${candidate.version} — ${candidate.module}, ${candidate.created_at}, ${candidate.archived_rows} rows`;
        list.appendChild(item);
      }
      box.appendChild(list);
    }

    if (preview.refused.length > 0) {
      const refusedHead = document.createElement("p");
      refusedHead.textContent = `Refused (${preview.refused.length}):`;
      box.appendChild(refusedHead);
      const list = document.createElement("ul");
      for (const refusal of preview.refused) {
        const item = document.createElement("li");
        item.textContent = refusal.set_name
          ? `${refusal.set_name}_${refusal.version}: ${refusal.reason}`
          : refusal.reason;
        list.appendChild(item);
      }
      box.appendChild(list);
    }

    if (preview.candidates.length > 0) {
      const purgeBtn = document.createElement("button");
      purgeBtn.type = "button";
      purgeBtn.className = "btn btn-accent";
      purgeBtn.textContent = `Purge ${preview.candidates.length} version(s)`;
      purgeBtn.addEventListener("click", () => {
        void (async () => {
          purgeBtn.disabled = true;
          try {
            const operator = await ensureSessionOperator("Purge versions");
            if (!operator) return;
            const receipt = await purgeLogSetVersions(
              previewedIds,
              operator.identity,
              operator.kind,
              "Versions",
            );
            const summary =
              `Purged ${receipt.versions_removed} version(s), ${receipt.archive_rows_removed} archived rows, ` +
              `across ${receipt.wells_touched} well(s) — audited. Disk space is reclaimed by Compact Project (Data ribbon).`;
            setStatus(summary);
            // Checked versions and keep-N scoped to this well are the selected well's
            // history; an all-wells keep-N belongs to no single well, so it is recorded
            // unattributed rather than pinned on whichever well happened to be selected.
            recordProcess(
              "Versions",
              summary,
              mode === "checked" || keepScope === "well"
                ? appState.selectedWell.get()?.well_name
                : undefined,
            );
            checked.clear();
            clearPreview();
            void reload();
          } catch (error) {
            setStatus(String(error));
          } finally {
            purgeBtn.disabled = false;
          }
        })();
      });
      box.appendChild(purgeBtn);
    }
    previewHost.appendChild(box);
  };

  previewBtn.addEventListener("click", () => {
    void (async () => {
      try {
        if (mode === "checked") {
          if (checked.size === 0) {
            setStatus("Tick the versions to purge first, or switch to Keep latest N.");
            return;
          }
          renderPreview(await previewVersionPurge({ set_ids: Array.from(checked) }));
        } else {
          const keep = Math.floor(Number(keepInput.value));
          if (!Number.isFinite(keep) || keep < 1) {
            setStatus(
              "Keep newest needs a whole number of 1 or more - 1 keeps only the latest of each lineage.",
            );
            return;
          }
          const wellIds =
            keepScope === "well"
              ? [appState.selectedWell.get()?.well_id ?? ""].filter(Boolean)
              : (() => {
                  const group = activeGroupWellIds();
                  return group ? Array.from(group) : undefined;
                })();
          if (keepScope === "well" && (!wellIds || wellIds.length === 0)) {
            setStatus("Select a well first, or switch the scope to All wells.");
            return;
          }
          renderPreview(
            await previewVersionPurge({
              keep_latest: keep,
              well_ids: wellIds,
              set_name: setSelect.value || undefined,
            }),
          );
        }
      } catch (error) {
        setStatus(String(error));
      }
    })();
  });

  const unsubscribeWell = appState.selectedWell.subscribe(() => {
    checked.clear();
    clearPreview();
    void reload();
  });
  // A data change may add versions or flip which one is live, so a preview computed
  // before it is stale - drop it rather than leave a Purge button over yesterday's list
  // (the backend re-checks its guards regardless; this keeps the SCREEN honest).
  const unsubscribeData = appState.dataVersion.subscribe(() => {
    clearPreview();
    void reload();
  });

  return {
    el: content,
    dispose: () => {
      unsubscribeWell();
      unsubscribeData();
    },
  };
}
