import {
  applyFwlToZoneParams,
  checkContactConsistency,
  checkFwlAgreement,
  contactGroups,
  deleteFluidContact,
  listFluidContacts,
  listWells,
  listZones,
  upsertFluidContact,
  type ContactGroup,
  type FluidContact,
  type FwlCheck,
  type WellSummary,
} from "../ipc";
import { appState, bumpDataVersion, setStatus } from "../state";
import { recordProcess } from "../processLog";
import { pushUndo } from "../undo";
import { formRow } from "./modal";
import { metresInStored, shownDepthLabel, storedDepthLabel, toShownDepth } from "../depthUnitPref";

/**
 * Fluid contacts — the stored table, and whether it can be trusted
 * (Petrophysics ▸ Contacts ▸ Fluid Contacts…).
 *
 * A dock PANE, so it can sit beside the correlation panel or a log view while contacts are picked
 * and checked — which is Jauhar's ask (2026-08-01: *"so it can be called when user in layout or
 * correlation and also can QC stored fluid contact table"*).
 *
 * **A contact is identified by three things, and all three earn their place**, because each was a
 * way of pooling surfaces that are not the same surface:
 *
 * - **The markers it governs.** Two stacked sands routinely have two different oil-water contacts.
 * - **A SET of markers**, not one, because several stacked sands in one hydraulic unit just as
 *   routinely SHARE one contact. A single marker field can say the first and not the second.
 * - **The compartment.** Two fault blocks are not in pressure communication and have no reason to
 *   sit on the same contact at all.
 *
 * Before this, the QC pooled every contact of a type across the whole project — so a field with two
 * sands produced one plane fit through two genuinely different surfaces, landed between them, and
 * then flagged every well as disagreeing with a contact that was never there.
 *
 * **The FWL section is the one that changes numbers.** A free-water level lives in two places: here,
 * where it is picked and drawn, and in `zone_params`, where `sw_height` actually reads it. Nothing
 * reconciled them, so the panel could draw one surface while every saturation in the report was
 * computed from another — both entirely plausible. The disagreement is measured, named, and copied
 * across only when the user asks.
 */
export async function buildFluidContactsContent(): Promise<{ el: HTMLElement }> {
  const wrap = document.createElement("div");
  wrap.className = "module-pane";

  const intro = document.createElement("div");
  intro.className = "eq-note";
  intro.textContent =
    "Every fluid contact in the project. A contact belongs to the markers it governs and to a " +
    "compartment — several stacked sands can share one contact, and two fault blocks should not.";
  wrap.appendChild(intro);

  let contacts: FluidContact[] = [];
  let wells: WellSummary[] = [];
  /** Marker names, gathered across the wells that have zones — the picker's vocabulary. */
  let zoneNames: string[] = [];

  const wellName = (id: string | null): string =>
    id ? (wells.find((w) => w.well_id === id)?.well_name ?? id) : "";

  // ---- the stored table ---------------------------------------------------
  const tableBox = document.createElement("div");
  wrap.appendChild(tableBox);

  const zoneList = document.createElement("datalist");
  zoneList.id = "fc-zone-list";
  wrap.appendChild(zoneList);

  /** A committed edit writes straight through — the row IS the record, and a Save button on every
   *  row of a table this small is friction without a purpose. */
  const commit = (c: FluidContact): void => {
    void (async () => {
      try {
        await upsertFluidContact(c);
        bumpDataVersion();
      } catch (e) {
        setStatus(String(e));
      }
    })();
  };

  const textCell = (value: string | null, onSet: (v: string | null) => void, list?: string) => {
    const inp = document.createElement("input");
    inp.className = "form-control";
    inp.style.width = "9rem";
    inp.value = value ?? "";
    if (list) inp.setAttribute("list", list);
    inp.addEventListener("change", () => {
      const t = inp.value.trim();
      // A blank is "not stated", never an empty name: an unassigned contact is its own QC group,
      // and a zero-length compartment would be a group nobody can name.
      onSet(t === "" ? null : t);
    });
    return inp;
  };

  const drawTable = (): void => {
    tableBox.innerHTML = "";
    if (!contacts.length) {
      const none = document.createElement("div");
      none.className = "eq-note";
      none.textContent =
        "No contacts stored yet. Add one below, or pick one from the logs in the correlation panel.";
      tableBox.appendChild(none);
      return;
    }
    const table = document.createElement("table");
    table.className = "data-table";
    const hrow = document.createElement("tr");
    for (const h of ["Type", "Compartment", "Markers", "Scope", "Depth", "Reference", "Label", ""]) {
      const th = document.createElement("th");
      th.textContent = h;
      hrow.appendChild(th);
    }
    table.appendChild(hrow);

    for (const c of contacts) {
      const tr = document.createElement("tr");
      const cell = (el: HTMLElement | string): void => {
        const td = document.createElement("td");
        if (typeof el === "string") td.textContent = el;
        else td.appendChild(el);
        tr.appendChild(td);
      };

      cell(
        textCell(c.contact_type, (v) => {
          c.contact_type = (v ?? "").toUpperCase();
          commit(c);
          void refreshQc();
        })
      );
      cell(
        textCell(c.compartment ?? null, (v) => {
          c.compartment = v;
          commit(c);
          void refreshQc();
        })
      );
      // Comma-separated for TYPING only — what is stored is a link row per marker. A list in a
      // text column would be the bug this table exists to avoid; a list in a text BOX is just a
      // convenient way to type one.
      const zoneCell = textCell((c.zones ?? []).join(", "), (v) => {
        c.zones = (v ?? "")
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean);
        commit(c);
        void refreshQc();
      }, zoneList.id);
      zoneCell.style.width = "12rem";
      zoneCell.title =
        "The markers this contact governs. Several means stacked sands sharing ONE contact; " +
        "blank means it states no marker, which is a real answer for a field-wide datum.";
      cell(zoneCell);

      cell(
        c.well_id
          ? wellName(c.well_id)
          : c.field_name
            ? `field ${c.field_name}`
            : "every well"
      );

      const depth = document.createElement("input");
      depth.className = "form-control";
      depth.type = "number";
      depth.step = "0.1";
      depth.style.width = "7rem";
      depth.value = String(c.depth);
      depth.addEventListener("change", () => {
        const n = Number(depth.value);
        if (!Number.isFinite(n)) {
          depth.value = String(c.depth);
          setStatus("A contact needs a real depth");
          return;
        }
        c.depth = n;
        commit(c);
        void refreshQc();
      });
      cell(depth);

      const ref = document.createElement("select");
      ref.className = "form-control";
      ref.style.width = "7rem";
      for (const [v, t] of [
        ["tvdss", "TVDSS"],
        ["md", "Measured"],
      ]) {
        const o = document.createElement("option");
        o.value = v;
        o.textContent = t;
        ref.appendChild(o);
      }
      ref.value = c.is_tvdss ? "tvdss" : "md";
      ref.title =
        "TVDSS draws flat across wells and is what the consistency check needs. Measured depth is " +
        "converted per well through its deviation survey.";
      ref.addEventListener("change", () => {
        c.is_tvdss = ref.value === "tvdss";
        c.depth_datum = c.is_tvdss ? "TVDSS" : "MD";
        commit(c);
        void refreshQc();
      });
      cell(ref);

      cell(
        textCell(c.label, (v) => {
          c.label = v;
          commit(c);
        })
      );

      const del = document.createElement("button");
      del.className = "btn";
      del.textContent = "✕";
      del.title = "Delete this contact";
      del.addEventListener("click", () => {
        void (async () => {
          const gone = { ...c, zones: [...(c.zones ?? [])] };
          await deleteFluidContact(c.contact_id);
          pushUndo({
            label: `delete ${c.contact_type} contact`,
            undo: async () => {
              await upsertFluidContact(gone);
              await reload();
              bumpDataVersion();
            },
            redo: async () => {
              await deleteFluidContact(gone.contact_id);
              await reload();
              bumpDataVersion();
            },
          });
          await reload();
          bumpDataVersion();
        })();
      });
      cell(del);
      table.appendChild(tr);
    }
    tableBox.appendChild(table);
  };

  // ---- adding one ---------------------------------------------------------
  const addBox = document.createElement("div");
  addBox.style.display = "flex";
  addBox.style.gap = "8px";
  addBox.style.flexWrap = "wrap";
  addBox.style.margin = "8px 0";
  wrap.appendChild(addBox);

  const typeIn = document.createElement("select");
  typeIn.className = "form-control";
  typeIn.style.width = "7rem";
  for (const t of ["OWC", "GWC", "GOC", "GDT", "ODT", "FWL"]) {
    const o = document.createElement("option");
    o.value = t;
    o.textContent = t;
    typeIn.appendChild(o);
  }
  const wellIn = document.createElement("select");
  wellIn.className = "form-control";
  wellIn.style.width = "11rem";
  const depthIn = document.createElement("input");
  depthIn.className = "form-control";
  depthIn.type = "number";
  depthIn.step = "0.1";
  depthIn.style.width = "8rem";
  depthIn.placeholder = "TVDSS (positive down)";
  const addBtn = document.createElement("button");
  addBtn.className = "btn btn-accent";
  addBtn.textContent = "Add contact";
  addBox.append(typeIn, wellIn, depthIn, addBtn);

  addBtn.addEventListener("click", () => {
    const d = Number(depthIn.value);
    if (!Number.isFinite(d)) {
      setStatus("Enter a depth for the contact");
      depthIn.focus();
      return;
    }
    void (async () => {
      // Added bare: the compartment and the markers are stated in the table, where the pickers
      // and the QC below are, rather than in a row of boxes that would have to duplicate them.
      const c: FluidContact = {
        contact_id: crypto.randomUUID(),
        field_name: null,
        well_id: wellIn.value || null,
        contact_type: typeIn.value,
        depth: d,
        depth_datum: "TVDSS",
        is_tvdss: true,
        color: null,
        label: null,
        compartment: null,
        zones: [],
      };
      await upsertFluidContact(c);
      recordProcess("Edit", `Added ${c.contact_type} at ${d}`, wellName(c.well_id) || null);
      depthIn.value = "";
      await reload();
      bumpDataVersion();
    })();
  });

  // ---- QC: does each contact hold together? -------------------------------
  const qcBox = document.createElement("div");
  qcBox.style.borderTop = "1px solid var(--border)";
  qcBox.style.marginTop = "10px";
  qcBox.style.paddingTop = "8px";
  wrap.appendChild(qcBox);

  const qcHead = document.createElement("div");
  qcHead.className = "field-label";
  qcHead.textContent = "Does each contact hold together?";
  qcBox.appendChild(qcHead);

  const qcNote = document.createElement("div");
  qcNote.className = "eq-note";
  qcNote.textContent =
    "A contact is flat in TVDSS, so the wells sharing one should sit on one surface. Each group " +
    "below is one contact — one type, one compartment, one set of markers — and is fitted on its " +
    "own. Wells more than the tolerance off their group's surface are named.";
  qcBox.appendChild(qcNote);

  // "3 m off the surface is a different contact" is a judgement about DISTANCE, so the box is
  // pre-filled in the project's own unit and labelled with it. A bare 3 on a foot project is
  // 0.91 m, and two picks 5 ft apart - 1.5 m, well inside anyone's tolerance - would be reported
  // as disagreeing. The number reaches the backend unconverted, which is why the label names the
  // STORED unit and not the display one.
  const tolIn = document.createElement("input");
  tolIn.className = "form-control";
  tolIn.type = "number";
  tolIn.step = "0.5";
  tolIn.value = String(Math.round(metresInStored(3) * 100) / 100);
  tolIn.style.width = "6rem";
  qcBox.appendChild(
    formRow(
      `Flag beyond (${storedDepthLabel()})`,
      tolIn,
      "A well further than this from its group's surface is named."
    )
  );

  const qcOut = document.createElement("div");
  qcBox.appendChild(qcOut);

  const groupLabel = (g: ContactGroup): string => {
    const where = g.compartment ? `${g.compartment}` : "no compartment";
    const what = g.zones.length ? g.zones.join(" + ") : "no marker";
    return `${g.contact_type} — ${what} — ${where}`;
  };

  const refreshQc = async (): Promise<void> => {
    qcOut.innerHTML = "";
    const tol = Number(tolIn.value) || metresInStored(3);
    let groups: ContactGroup[] = [];
    try {
      groups = await contactGroups();
    } catch (e) {
      qcOut.textContent = String(e);
      return;
    }
    if (!groups.length) {
      const none = document.createElement("div");
      none.className = "eq-note";
      none.textContent = "Nothing to check yet.";
      qcOut.appendChild(none);
      return;
    }
    for (const g of groups) {
      const box = document.createElement("div");
      box.className = "eq-note";
      box.style.marginTop = "6px";
      const head = document.createElement("div");
      const strong = document.createElement("strong");
      strong.textContent = groupLabel(g);
      head.appendChild(strong);
      box.appendChild(head);

      const res = await checkContactConsistency(g.contact_type, g.compartment, g.zones, tol).catch(
        (e) => ({ error: String(e) }) as never
      );
      const line = document.createElement("div");
      if (res.error) {
        // Not a failure — one pick cannot disagree with anything, and saying so beats a blank.
        line.textContent = res.error;
        line.style.color = "var(--text-dim)";
      } else {
        const flagged = res.wells.filter((w) => w.flagged);
        // These are READ, so unlike the box above they follow the display preference and are
        // converted. Depths, a spread and residuals are all lengths in the depth dimension.
        const su = shownDepthLabel();
        line.textContent =
          `${res.n} well(s), mean ${toShownDepth(res.mean_tvdss).toFixed(1)} TVDSS (${su}), ` +
          `spread ${toShownDepth(res.rms).toFixed(2)} ${su}` +
          (res.plane ? " (fitted as a dipping surface)" : " (flat mean)") +
          (flagged.length
            ? ` — off the surface: ${flagged
                .map(
                  (w) =>
                    `${w.well_name} ${w.residual >= 0 ? "+" : ""}` +
                    `${toShownDepth(w.residual).toFixed(1)} ${su}`
                )
                .join(", ")}`
            : " — every well agrees.");
        if (flagged.length) line.style.color = "var(--warn)";
      }
      box.appendChild(line);
      qcOut.appendChild(box);
    }
  };

  tolIn.addEventListener("change", () => void refreshQc());

  // ---- the two FWLs -------------------------------------------------------
  const fwlBox = document.createElement("div");
  fwlBox.style.borderTop = "1px solid var(--border)";
  fwlBox.style.marginTop = "10px";
  fwlBox.style.paddingTop = "8px";
  wrap.appendChild(fwlBox);

  const fwlHead = document.createElement("div");
  fwlHead.className = "field-label";
  fwlHead.textContent = "Does the arithmetic use the FWL you picked?";
  fwlBox.appendChild(fwlHead);

  const fwlNote = document.createElement("div");
  fwlNote.className = "eq-note";
  fwlNote.textContent =
    "A free-water level lives in two places: here, where it is picked and drawn, and in the zone " +
    "parameters, where a saturation-height run reads it. Nothing reconciled them, so the panel " +
    "could draw one surface while every saturation was computed from another.";
  fwlBox.appendChild(fwlNote);

  const fwlOut = document.createElement("div");
  fwlBox.appendChild(fwlOut);

  const refreshFwl = async (): Promise<void> => {
    fwlOut.innerHTML = "";
    let rows: FwlCheck[] = [];
    try {
      rows = await checkFwlAgreement(0.1);
    } catch (e) {
      fwlOut.textContent = String(e);
      return;
    }
    if (!rows.length) {
      const none = document.createElement("div");
      none.className = "eq-note";
      none.textContent =
        "No FWL contact carries a marker, so there is nothing to compare. A contact needs the " +
        "marker it governs before it can be checked against a per-marker parameter.";
      fwlOut.appendChild(none);
      return;
    }
    const table = document.createElement("table");
    table.className = "data-table";
    const hrow = document.createElement("tr");
    for (const h of ["Well", "Marker", "Picked", "Computed from", "Difference", "", ""]) {
      const th = document.createElement("th");
      th.textContent = h;
      hrow.appendChild(th);
    }
    table.appendChild(hrow);

    const applicable: FwlCheck[] = [];
    for (const r of rows) {
      if (r.can_apply) applicable.push(r);
      const tr = document.createElement("tr");
      const cell = (t: string, hint?: string): void => {
        const td = document.createElement("td");
        td.textContent = t;
        if (hint) td.title = hint;
        tr.appendChild(td);
      };
      cell(r.well_name);
      cell(r.zone_name);
      cell(`${r.contact_depth.toFixed(2)}${r.contact_is_tvdss ? "" : " MD"}`);
      cell(r.param_value == null ? "—" : r.param_value.toFixed(2));
      cell(Number.isFinite(r.difference) ? `${r.difference >= 0 ? "+" : ""}${r.difference.toFixed(2)}` : "—");
      const v = document.createElement("td");
      v.textContent = r.verdict;
      v.style.maxWidth = "26rem";
      if (r.verdict.startsWith("Agrees")) v.style.color = "var(--text-dim)";
      else v.style.color = "var(--warn)";
      tr.appendChild(v);
      tr.appendChild(document.createElement("td"));
      table.appendChild(tr);
    }
    fwlOut.appendChild(table);

    if (applicable.length) {
      const apply = document.createElement("button");
      apply.className = "btn btn-accent";
      apply.style.marginTop = "6px";
      apply.textContent = `Write ${applicable.length} picked level(s) to the zone parameters`;
      apply.title =
        "Copies the picked FWL into the parameter a saturation-height run reads. An explicit copy " +
        "rather than a live read, so a stored run can always say which number it used.";
      apply.addEventListener("click", () => {
        void (async () => {
          apply.disabled = true;
          try {
            // The values being replaced, captured BEFORE the write so the undo restores what was
            // there rather than clearing the row — a parameter silently pinned to nothing is a
            // wrong answer that keeps computing.
            const before = applicable.map((r) => ({ ...r }));
            const picks: [string, string, number][] = applicable.map((r) => [
              r.well_id,
              r.zone_name,
              r.contact_depth,
            ]);
            const n = await applyFwlToZoneParams(picks);
            setStatus(`Wrote ${n} FWL parameter(s) from the picked contacts`);
            recordProcess("Edit", `FWL reconciled from contacts: ${n} marker(s)`, null);
            pushUndo({
              label: `FWL from contacts (${n})`,
              undo: async () => {
                const back: [string, string, number][] = before
                  .filter((r) => r.param_value != null)
                  .map((r) => [r.well_id, r.zone_name, r.param_value as number]);
                if (back.length) await applyFwlToZoneParams(back);
                await refreshFwl();
                bumpDataVersion();
              },
              redo: async () => {
                await applyFwlToZoneParams(picks);
                await refreshFwl();
                bumpDataVersion();
              },
            });
            await refreshFwl();
            bumpDataVersion();
          } catch (e) {
            setStatus(String(e));
          } finally {
            apply.disabled = false;
          }
        })();
      });
      fwlOut.appendChild(apply);
    }
  };

  // ---- loading ------------------------------------------------------------
  async function reload(): Promise<void> {
    contacts = await listFluidContacts().catch(() => [] as FluidContact[]);
    wells = await listWells({ kind: "all" }).catch(() => [] as WellSummary[]);

    wellIn.innerHTML = "";
    const anyWell = document.createElement("option");
    anyWell.value = "";
    anyWell.textContent = "— every well (a datum) —";
    wellIn.appendChild(anyWell);
    for (const w of wells) {
      const o = document.createElement("option");
      o.value = w.well_id;
      o.textContent = w.well_name;
      wellIn.appendChild(o);
    }
    const sel = appState.selectedWell.get();
    if (sel && wells.some((w) => w.well_id === sel.well_id)) wellIn.value = sel.well_id;

    // The marker vocabulary comes from the selected well first, then anything else already used on
    // a contact — enough to type against without a query per well on a 2000-well project.
    const names = new Set<string>();
    if (sel) {
      for (const z of await listZones(sel.well_id).catch(() => [])) names.add(z.zone_name);
    }
    for (const c of contacts) for (const z of c.zones ?? []) names.add(z);
    zoneNames = Array.from(names).sort();
    zoneList.innerHTML = "";
    for (const n of zoneNames) {
      const o = document.createElement("option");
      o.value = n;
      zoneList.appendChild(o);
    }

    drawTable();
    await refreshQc();
    await refreshFwl();
  }

  await reload();
  return { el: wrap };
}
