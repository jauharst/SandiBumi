import {
  createWellGroup,
  deleteWellGroup,
  listWellGroups,
  listWells,
  renameWellGroup,
  setActiveWellGroup,
  setWellGroupMembers,
  type WellGroupEntry,
  type WellSummary,
} from "../ipc";
import { appState, bumpWellGroupsVersion, setStatus } from "../state";
import { openModal } from "./modal";

/** Reloads the group list from the backend and syncs `appState.activeWellGroup` to the
 *  one flagged active (or null). Every place that changes groups calls this so the Wells
 *  pane and batch dialogs see a consistent active group. Returns the full list. */
export async function syncWellGroups(): Promise<WellGroupEntry[]> {
  let groups: WellGroupEntry[] = [];
  try {
    groups = await listWellGroups();
  } catch {
    groups = [];
  }
  const active = groups.find((g) => g.active) ?? null;
  const current = appState.activeWellGroup.get();
  // Only publish when it actually changed (by id + membership) to avoid redundant repaints.
  const changed =
    (active?.group_id ?? null) !== (current?.group_id ?? null) ||
    (active && current && active.well_ids.join() !== current.well_ids.join());
  if (changed || (!active && current) || (active && !current)) {
    appState.activeWellGroup.set(active);
  }
  return groups;
}

/** Activates a group (or clears it for "All wells"), persisting and broadcasting. */
export async function activateWellGroup(groupId: string | null): Promise<void> {
  await setActiveWellGroup(groupId);
  await syncWellGroups();
  bumpWellGroupsVersion();
  const g = appState.activeWellGroup.get();
  setStatus(g ? `Active well group: ${g.name} (${g.member_count} wells)` : "Well group cleared — showing all wells");
}

/** The Well Groups manager: create/rename/delete groups, set which one is active, and
 *  edit each group's membership from a searchable checklist of every well. Non-blocking
 *  dialog (Esc / ✕ to close). */
export async function openWellGroupManager(): Promise<void> {
  const [wells, groups0] = await Promise.all([safeWells(), syncWellGroups()]);
  let groups = groups0;

  const content = document.createElement("div");
  content.className = "well-group-manager";

  const groupList = document.createElement("div");
  groupList.className = "wg-list";
  content.appendChild(groupList);

  // New-group row.
  const newRow = document.createElement("div");
  newRow.className = "wg-new-row";
  const newInput = document.createElement("input");
  newInput.className = "form-control";
  newInput.placeholder = "New group name…";
  const newBtn = document.createElement("button");
  newBtn.className = "form-run-btn";
  newBtn.textContent = "Create";
  newBtn.addEventListener("click", async () => {
    const name = newInput.value.trim();
    if (!name) return;
    const id = await createWellGroup(name, []);
    newInput.value = "";
    await reload();
    editGroupId = id; // jump straight to editing membership of the new group
    renderMembership();
  });
  newRow.appendChild(newInput);
  newRow.appendChild(newBtn);
  content.appendChild(newRow);

  // Membership editor.
  const memberSection = document.createElement("div");
  memberSection.className = "wg-members";
  content.appendChild(memberSection);

  let editGroupId: string | null = null;
  const checked = new Set<string>();

  const renderGroups = () => {
    groupList.innerHTML = "";
    const header = document.createElement("div");
    header.className = "wg-row wg-header";
    header.innerHTML = `<span class="wg-active-col"></span><span class="wg-name">Group</span><span class="wg-count">wells</span><span class="wg-actions"></span>`;
    groupList.appendChild(header);

    // "All wells" = clear active group.
    const allRow = document.createElement("div");
    allRow.className = "wg-row";
    const allRadio = document.createElement("input");
    allRadio.type = "radio";
    allRadio.name = "wg-active";
    allRadio.checked = !groups.some((g) => g.active);
    allRadio.addEventListener("change", () => void activateWellGroup(null).then(reload));
    const allName = document.createElement("span");
    allName.className = "wg-name";
    allName.textContent = "All wells";
    const allCount = document.createElement("span");
    allCount.className = "wg-count";
    allCount.textContent = String(wells.length);
    allRow.append(wrapCol(allRadio), allName, allCount, document.createElement("span"));
    groupList.appendChild(allRow);

    for (const g of groups) {
      const row = document.createElement("div");
      row.className = "wg-row" + (g.group_id === editGroupId ? " wg-editing" : "");

      const radio = document.createElement("input");
      radio.type = "radio";
      radio.name = "wg-active";
      radio.checked = g.active;
      radio.title = "Make this the active group (filters the workspace)";
      radio.addEventListener("change", () => void activateWellGroup(g.group_id).then(reload));

      const name = document.createElement("span");
      name.className = "wg-name";
      name.textContent = g.name;
      name.title = "Double-click to rename";
      name.addEventListener("dblclick", async () => {
        const next = prompt("Rename group", g.name);
        if (next && next.trim() && next.trim() !== g.name) {
          await renameWellGroup(g.group_id, next.trim());
          await reload();
        }
      });

      const count = document.createElement("span");
      count.className = "wg-count";
      count.textContent = String(g.member_count);

      const actions = document.createElement("span");
      actions.className = "wg-actions";
      const editBtn = document.createElement("button");
      editBtn.className = "wg-btn";
      editBtn.textContent = "Edit wells";
      editBtn.addEventListener("click", () => {
        editGroupId = g.group_id;
        renderGroups();
        renderMembership();
      });
      const delBtn = document.createElement("button");
      delBtn.className = "wg-btn wg-danger";
      delBtn.textContent = "Delete";
      delBtn.addEventListener("click", async () => {
        if (!confirm(`Delete group "${g.name}"? Wells are not affected.`)) return;
        await deleteWellGroup(g.group_id);
        if (editGroupId === g.group_id) editGroupId = null;
        await reload();
        renderMembership();
      });
      actions.append(editBtn, delBtn);

      row.append(wrapCol(radio), name, count, actions);
      groupList.appendChild(row);
    }
  };

  const renderMembership = () => {
    memberSection.innerHTML = "";
    const group = groups.find((g) => g.group_id === editGroupId);
    if (!group) return;

    checked.clear();
    for (const id of group.well_ids) checked.add(id);

    const title = document.createElement("div");
    title.className = "wg-members-title";
    title.textContent = `Wells in “${group.name}”`;
    memberSection.appendChild(title);

    const controls = document.createElement("div");
    controls.className = "wg-members-controls";
    const search = document.createElement("input");
    search.className = "form-control";
    search.placeholder = "Filter wells…";
    const allBtn = document.createElement("button");
    allBtn.className = "wg-btn";
    allBtn.textContent = "Select all (shown)";
    const noneBtn = document.createElement("button");
    noneBtn.className = "wg-btn";
    noneBtn.textContent = "Clear (shown)";
    controls.append(search, allBtn, noneBtn);
    memberSection.appendChild(controls);

    const listEl = document.createElement("div");
    listEl.className = "wg-well-list";
    memberSection.appendChild(listEl);

    const visible = (): WellSummary[] => {
      const q = search.value.trim().toLowerCase();
      return q
        ? wells.filter((w) => `${w.well_name} ${w.field_name ?? ""}`.toLowerCase().includes(q))
        : wells;
    };

    const renderList = () => {
      listEl.innerHTML = "";
      for (const w of visible()) {
        const label = document.createElement("label");
        label.className = "wg-well-check";
        const box = document.createElement("input");
        box.type = "checkbox";
        box.checked = checked.has(w.well_id);
        box.addEventListener("change", () => {
          if (box.checked) checked.add(w.well_id);
          else checked.delete(w.well_id);
        });
        label.append(box, document.createTextNode(w.field_name ? `${w.well_name} (${w.field_name})` : w.well_name));
        listEl.appendChild(label);
      }
    };
    search.addEventListener("input", renderList);
    allBtn.addEventListener("click", () => {
      for (const w of visible()) checked.add(w.well_id);
      renderList();
    });
    noneBtn.addEventListener("click", () => {
      for (const w of visible()) checked.delete(w.well_id);
      renderList();
    });
    renderList();

    const saveRow = document.createElement("div");
    saveRow.className = "wg-save-row";
    const countLbl = document.createElement("span");
    const updateCount = () => (countLbl.textContent = `${checked.size} selected`);
    updateCount();
    const saveBtn = document.createElement("button");
    saveBtn.className = "form-run-btn";
    saveBtn.textContent = "Save membership";
    saveBtn.addEventListener("click", async () => {
      await setWellGroupMembers(group.group_id, [...checked]);
      await reload();
      renderMembership();
      setStatus(`Group “${group.name}” now has ${checked.size} wells`);
    });
    // Live count while toggling.
    listEl.addEventListener("change", updateCount);
    saveRow.append(saveBtn, countLbl);
    memberSection.appendChild(saveRow);
  };

  const reload = async () => {
    groups = await syncWellGroups();
    bumpWellGroupsVersion();
    renderGroups();
  };

  renderGroups();
  openModal("Well Groups", content, 560);
}

function wrapCol(el: HTMLElement): HTMLElement {
  const span = document.createElement("span");
  span.className = "wg-active-col";
  span.appendChild(el);
  return span;
}

async function safeWells(): Promise<WellSummary[]> {
  try {
    return await listWells();
  } catch {
    return [];
  }
}
