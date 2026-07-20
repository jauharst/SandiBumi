import {
  deleteZone,
  listZoneParams,
  listZones,
  setZoneParam,
  upsertZone,
  zonesFromTops,
  type WellSummary,
  type ZoneEntry,
  type ZoneParamEntry,
} from "../ipc";
import { recordProcess } from "../processLog";

/** Zone manager for the selected well: build zones from tops, add/edit/delete zones,
 *  and set per-zone interval parameter overrides (Geolog interval-parameter model —
 *  any numeric module parameter, e.g. GR_MA, GR_SH, RW, M, N, applied over that zone's
 *  depth range at run time).
 *  Hosted as a dock pane (workspace component "zones"), not a popup. */
export async function buildZonesContent(
  well: WellSummary,
  setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose?: () => void }> {
  const content = document.createElement("div");
  content.className = "zones-pane";

  const zoneList = document.createElement("div");
  const paramList = document.createElement("div");

  const refresh = async () => {
    const [zones, params] = await Promise.all([listZones(well.well_id), listZoneParams(well.well_id)]);
    renderZones(zones, params);
  };

  const renderZones = (zones: ZoneEntry[], params: ZoneParamEntry[]) => {
    zoneList.innerHTML = "";
    const table = document.createElement("table");
    table.className = "zone-table";
    table.innerHTML = "<thead><tr><th>Zone</th><th>Top</th><th>Bottom</th><th></th></tr></thead>";
    const tbody = document.createElement("tbody");
    for (const zone of zones) {
      const tr = document.createElement("tr");
      const del = document.createElement("button");
      del.className = "zone-del";
      del.textContent = "✕";
      del.title = "Delete zone";
      del.addEventListener("click", async () => {
        await deleteZone(well.well_id, zone.zone_name);
        recordProcess("Zone", `Deleted zone ${zone.zone_name}`, well.well_name);
        await refresh();
      });
      tr.innerHTML = `<td>${zone.zone_name}</td><td>${zone.top_depth.toFixed(1)}</td><td>${zone.bottom_depth.toFixed(1)}</td>`;
      const td = document.createElement("td");
      td.appendChild(del);
      tr.appendChild(td);
      tbody.appendChild(tr);
    }
    if (zones.length === 0) {
      tbody.innerHTML = `<tr><td colspan="4" class="zone-empty">No zones — use "From Tops" or add one below.</td></tr>`;
    }
    table.appendChild(tbody);
    zoneList.appendChild(table);

    paramList.innerHTML = "";
    if (params.length > 0) {
      const ptable = document.createElement("table");
      ptable.className = "zone-table";
      ptable.innerHTML = "<thead><tr><th>Zone</th><th>Parameter</th><th>Value</th><th></th></tr></thead>";
      const ptbody = document.createElement("tbody");
      for (const p of params) {
        const tr = document.createElement("tr");
        tr.innerHTML = `<td>${p.zone_name}</td><td>${p.param_name}</td><td>${p.value_num ?? p.value_text ?? ""}</td>`;
        const del = document.createElement("button");
        del.className = "zone-del";
        del.textContent = "✕";
        del.title = "Remove override";
        del.addEventListener("click", async () => {
          await setZoneParam(well.well_id, p.zone_name, p.param_name, null, null);
          recordProcess("Zone", `Removed ${p.param_name} override on zone ${p.zone_name}`, well.well_name);
          await refresh();
        });
        const td = document.createElement("td");
        td.appendChild(del);
        tr.appendChild(td);
        ptbody.appendChild(tr);
      }
      ptable.appendChild(ptbody);
      paramList.appendChild(ptable);
    }
  };

  // --- Actions row: build from tops ---
  const actions = document.createElement("div");
  actions.className = "zone-actions";
  const fromTopsBtn = document.createElement("button");
  fromTopsBtn.className = "form-run-btn";
  fromTopsBtn.textContent = "From Tops";
  fromTopsBtn.title = "Rebuild zones from this well's formation tops";
  fromTopsBtn.addEventListener("click", async () => {
    const zones = await zonesFromTops(well.well_id);
    setStatus(`Built ${zones.length} zone(s) from tops for ${well.well_name}`);
    recordProcess("Zone", `Built ${zones.length} zone(s) from tops`, well.well_name);
    await refresh();
  });
  actions.appendChild(fromTopsBtn);
  content.appendChild(actions);

  const zonesTitle = document.createElement("h4");
  zonesTitle.textContent = "Zones";
  content.appendChild(zonesTitle);
  content.appendChild(zoneList);

  // --- Add zone row ---
  const addRow = document.createElement("div");
  addRow.className = "zone-add-row";
  const nameIn = document.createElement("input");
  nameIn.className = "form-control";
  nameIn.placeholder = "Zone name";
  const topIn = document.createElement("input");
  topIn.className = "form-control";
  topIn.type = "number";
  topIn.step = "any";
  topIn.placeholder = "Top";
  const botIn = document.createElement("input");
  botIn.className = "form-control";
  botIn.type = "number";
  botIn.step = "any";
  botIn.placeholder = "Bottom";
  const addBtn = document.createElement("button");
  addBtn.className = "form-run-btn";
  addBtn.textContent = "Add / Update Zone";
  addBtn.addEventListener("click", async () => {
    const name = nameIn.value.trim();
    const top = parseFloat(topIn.value);
    const bottom = parseFloat(botIn.value);
    if (!name || Number.isNaN(top) || Number.isNaN(bottom) || bottom <= top) return;
    await upsertZone(well.well_id, name, top, bottom);
    recordProcess("Zone", `Set zone ${name} (${top}–${bottom})`, well.well_name);
    nameIn.value = "";
    await refresh();
  });
  addRow.appendChild(nameIn);
  addRow.appendChild(topIn);
  addRow.appendChild(botIn);
  addRow.appendChild(addBtn);
  content.appendChild(addRow);

  // --- Parameter overrides ---
  const paramsTitle = document.createElement("h4");
  paramsTitle.textContent = "Per-zone parameter overrides";
  content.appendChild(paramsTitle);
  const paramsHint = document.createElement("p");
  paramsHint.className = "modal-hint";
  paramsHint.textContent =
    "Any module parameter (GR_MA, GR_SH, RW, M, N, RT_SH, …) can vary per zone. Zone '*' applies well-wide. Overrides beat the value typed in a module dialog.";
  content.appendChild(paramsHint);
  content.appendChild(paramList);

  const setRow = document.createElement("div");
  setRow.className = "zone-add-row";
  const zoneIn = document.createElement("input");
  zoneIn.className = "form-control";
  zoneIn.placeholder = "Zone (* = all)";
  const paramIn = document.createElement("input");
  paramIn.className = "form-control";
  paramIn.placeholder = "Parameter";
  const valueIn = document.createElement("input");
  valueIn.className = "form-control";
  valueIn.type = "number";
  valueIn.step = "any";
  valueIn.placeholder = "Value";
  const setBtn = document.createElement("button");
  setBtn.className = "form-run-btn";
  setBtn.textContent = "Set";
  setBtn.addEventListener("click", async () => {
    const zone = zoneIn.value.trim();
    const param = paramIn.value.trim().toUpperCase();
    const value = parseFloat(valueIn.value);
    if (!zone || !param || Number.isNaN(value)) return;
    await setZoneParam(well.well_id, zone, param, value, null);
    recordProcess("Zone", `Set ${param} = ${value} on zone ${zone}`, well.well_name);
    await refresh();
  });
  setRow.appendChild(zoneIn);
  setRow.appendChild(paramIn);
  setRow.appendChild(valueIn);
  setRow.appendChild(setBtn);
  content.appendChild(setRow);

  await refresh();
  return { el: content };
}
