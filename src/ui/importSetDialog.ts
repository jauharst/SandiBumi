import { formRow, openModal } from "./modal";
import type { LasImportOptions, LasWellIdentityProbe } from "../ipc";
import {
  UNIT_REGISTRY_FAMILIES,
  UNIT_REGISTRY_POPULATION,
  UNIT_REGISTRY_UNITS,
  UNIT_REGISTRY_VERSION,
} from "../generated/unitRegistry";

/** The Import LAS "which set?" dialog (T-IMP-02 — the Geolog/IP set model).
 *
 *  A delivery folder is one SET: `well00025_2023_fprooh.las` and its 543 siblings are
 *  the FPROOH interpretation of the field, and a well's RAW, FPROOH and MULTIMIN curves
 *  belong on ONE well record, not on three same-named ones. This dialog names the set and
 *  decides whether same-named files attach to the existing well.
 *
 *  Non-blocking, per the app's dialog convention (modal.ts): the scrim is
 *  pointer-transparent and only Esc / ✕ / the buttons close it.
 */

/** Filename tokens that carry no set meaning — they appear in every file of a delivery. */
const NOISE_TOKENS = new Set([
  "las", "log", "logs", "final", "data", "well", "wells", "copy", "new", "old", "edit",
]);

/**
 * Derives a set-name suggestion from what the picked filenames have in COMMON.
 *
 * Files are split on `_`, `-`, `.` and space; a token is a candidate only if it appears in
 * EVERY file (so it describes the delivery, not one well), is not purely numeric or a year
 * (well numbers and delivery dates differ per file but say nothing about content),
 * and is not generic noise.
 *
 * A candidate at POSITION 0 is rejected: vendor names run `<well>_<project>_<product>`, so
 * a leading token shared by every file is the well or field prefix, not the set —
 * `SANDI-01/02/03` would otherwise suggest "SANDI" for what is plainly a raw log delivery.
 * Among the rest the LAST wins, which is the product suffix (`fprooh`, `multimin`, `ssc`).
 * Returns "" when nothing survives, and the caller falls back to RAW.
 */
export function suggestSetName(paths: string[]): string {
  if (paths.length === 0) return "";
  // Positions come from the UNFILTERED split, so "position 0" means "first token of the
  // filename" even when earlier tokens were dropped as noise.
  const tokensOf = (p: string): { token: string; index: number }[] => {
    const base = p.replace(/\\/g, "/").split("/").pop() ?? p;
    return base
      .replace(/\.[^.]*$/, "") // strip extension
      .split(/[_\-.\s]+/)
      .map((t, index) => ({ token: t.trim().toUpperCase(), index }))
      .filter(
        ({ token: t }) =>
          t.length >= 2 &&
          !NOISE_TOKENS.has(t.toLowerCase()) &&
          !/^\d+$/.test(t) && // a bare well number
          !/^\d{4}$/.test(t) && // a bare year
          !/^[A-Z]*\d{3,}[A-Z\d]*$/.test(t), // well ids like WELL00025 / 00358D1
      );
  };
  const first = tokensOf(paths[0]);
  if (first.length === 0) return "";
  const rest = paths.slice(1).map((p) => new Set(tokensOf(p).map((t) => t.token)));
  const common = first.filter(({ token, index }) => index > 0 && rest.every((s) => s.has(token)));
  if (common.length === 0) return "";
  return common[common.length - 1].token;
}

export interface ImportSetChoice extends LasImportOptions {
  setName: string;
  attach: boolean;
  fileDepthUnit: "M" | "FT" | null;
  undeclaredDrhoUnit: "g/cc" | "kg/m3" | null;
  samplingStyle: "CONTINUOUS_REGULAR" | "CONTINUOUS_IRREGULAR";
  samplingStyleVerifyTolerance: { value: number; unit: "M" | "FT" } | null;
  confirmedWellNames: Record<string, string>;
}

/**
 * Asks for the set name + attach behaviour. Resolves with the choice, or null if the user
 * cancels (Esc / ✕ / Cancel) — the caller must treat null as "import nothing".
 */
export function openImportSetDialog(
  paths: string[],
  identityProbes: LasWellIdentityProbe[],
): Promise<ImportSetChoice | null> {
  return new Promise((resolve) => {
    const wrap = document.createElement("div");
    const probesByPath = new Map(identityProbes.map((probe) => [probe.path, probe]));

    // Organic design 1e: the picked delivery as a file rail rather than one
    // truncated line — the user is naming what these files ARE, so they should
    // be able to see them. Filenames are user data: textContent + no-i18n.
    const rail = document.createElement("div");
    rail.className = "import-file-list";
    rail.setAttribute("data-no-i18n", "");
    const railHead = document.createElement("div");
    railHead.className = "import-file-count";
    railHead.textContent = `${paths.length} file${paths.length === 1 ? "" : "s"} picked`;
    rail.appendChild(railHead);
    const shown = paths.slice(0, 6);
    for (const p of shown) {
      const row = document.createElement("div");
      row.className = "import-file-row";
      const filename = p.replace(/\\/g, "/").split("/").pop() ?? p;
      const containerIdentity = probesByPath.get(p)?.container_well_name;
      row.textContent = containerIdentity
        ? `${filename} — container identity: ${containerIdentity}; filename not used`
        : `${filename} — source identity absent`;
      row.title = p;
      rail.appendChild(row);
    }
    if (paths.length > shown.length) {
      const more = document.createElement("div");
      more.className = "import-file-more";
      more.textContent = `+${paths.length - shown.length} more`;
      rail.appendChild(more);
    }
    wrap.appendChild(rail);

    const identityInputs = new Map<string, HTMLInputElement>();
    for (const path of paths) {
      const probe = probesByPath.get(path);
      if (probe?.container_well_name) continue;
      const input = document.createElement("input");
      input.type = "text";
      input.className = "form-control";
      input.value = probe?.filename_proposal ?? "";
      input.spellcheck = false;
      input.setAttribute("data-no-i18n", "");
      identityInputs.set(path, input);
      const filename = path.replace(/\\/g, "/").split("/").pop() ?? path;
      wrap.appendChild(
        formRow(
          `Confirm identity for ${filename}`,
          input,
          "The container has no WELL value. The filename is only a proposal; Import explicitly confirms the entered identity.",
        ),
      );
    }

    const setInput = document.createElement("input");
    setInput.type = "text";
    setInput.className = "form-control";
    setInput.value = suggestSetName(paths);
    setInput.placeholder = "RAW";
    setInput.spellcheck = false;
    wrap.appendChild(
      formRow(
        "Set name",
        setInput,
        "One delivery = one set. Curves land under this name so you can tell this run's PHIE from another's. Blank = RAW.",
      ),
    );

    const setHint = document.createElement("p");
    setHint.className = "form-hint";
    setHint.textContent =
      "Upper-cased; spaces become underscores. If a well already has a set with this name, " +
      "the new one is suffixed (FPROOH → FPROOH_1) — an import never overwrites an earlier delivery.";
    wrap.appendChild(setHint);

    const attachBox = document.createElement("input");
    attachBox.type = "checkbox";
    attachBox.className = "form-check";
    attachBox.checked = true;
    wrap.appendChild(
      formRow(
        "Attach to existing wells",
        attachBox,
        "Match by well name: a re-delivery of a well already in the project becomes a new set on THAT well.",
      ),
    );

    const attachHint = document.createElement("p");
    attachHint.className = "form-hint";
    attachHint.textContent =
      "On (recommended): re-importing the same field under a new set name keeps one record per well. " +
      "Off: every file creates its own well record, even when the name already exists. " +
      "A name matching several existing wells is always ambiguous — those import as separate records and say so.";
    wrap.appendChild(attachHint);

    const samplingStyle = document.createElement("select");
    samplingStyle.className = "form-control";
    for (const [value, label] of [
      ["", "Choose the delivery's declared style"],
      ["CONTINUOUS_REGULAR", "Continuous regular (verify STEP)"],
      ["CONTINUOUS_IRREGULAR", "Continuous irregular"],
    ] as const) {
      const option = document.createElement("option");
      option.value = value;
      option.textContent = label;
      samplingStyle.appendChild(option);
    }
    wrap.appendChild(
      formRow(
        "Sampling style",
        samplingStyle,
        "Required per set. SandiBumi stores both the declaration and its verified effective style; it never infers regularity from the samples.",
      ),
    );

    const toleranceRow = document.createElement("div");
    toleranceRow.className = "import-sampling-tolerance";
    const toleranceValue = document.createElement("input");
    toleranceValue.type = "number";
    toleranceValue.step = "any";
    toleranceValue.min = "0";
    toleranceValue.className = "form-control";
    toleranceValue.placeholder = "Required for regular";
    const toleranceUnit = document.createElement("select");
    toleranceUnit.className = "form-control";
    for (const [value, label] of [
      ["", "Choose unit"],
      ["M", "metres"],
      ["FT", "feet"],
    ] as const) {
      const option = document.createElement("option");
      option.value = value;
      option.textContent = label;
      toleranceUnit.appendChild(option);
    }
    toleranceRow.append(toleranceValue, toleranceUnit);
    wrap.appendChild(
      formRow(
        "Regular-step tolerance",
        toleranceRow,
        "Required only for a regular declaration. No default ships, and this is not the irregular-set snap tolerance.",
      ),
    );
    const syncSamplingControls = () => {
      const regular = samplingStyle.value === "CONTINUOUS_REGULAR";
      toleranceValue.disabled = !regular;
      toleranceUnit.disabled = !regular;
      if (!regular) {
        toleranceValue.value = "";
        toleranceUnit.value = "";
      }
    };
    samplingStyle.addEventListener("change", syncSamplingControls);
    syncSamplingControls();

    const undeclaredUnit = document.createElement("select");
    undeclaredUnit.className = "form-control";
    for (const [value, label] of [
      ["", "Require the file to declare it"],
      ["M", "Metres (explicit confirmation)"],
      ["FT", "Feet (explicit confirmation)"],
    ] as const) {
      const option = document.createElement("option");
      option.value = value;
      option.textContent = label;
      undeclaredUnit.appendChild(option);
    }
    wrap.appendChild(
      formRow(
        "File depth unit when undeclared",
        undeclaredUnit,
        "The project unit is not evidence of what a file meant. Leave this safe default to refuse any file that omits its index unit.",
      ),
    );

    const undeclaredDrhoUnit = document.createElement("select");
    undeclaredDrhoUnit.className = "form-control";
    for (const [value, label] of [
      ["", "Require each DRHO curve to declare it"],
      ["g/cc", "g/cc (explicit confirmation)"],
      ["kg/m3", "kg/m³ (explicit confirmation)"],
    ] as const) {
      const option = document.createElement("option");
      option.value = value;
      option.textContent = label;
      undeclaredDrhoUnit.appendChild(option);
    }
    wrap.appendChild(
      formRow(
        "DRHO unit when undeclared",
        undeclaredDrhoUnit,
        "Used only for a density-correction channel whose source unit is absent. Leave empty to refuse it; a mnemonic is not a unit declaration.",
      ),
    );

    const vocabulary = document.createElement("details");
    vocabulary.className = "import-unit-registry";
    const vocabularySummary = document.createElement("summary");
    vocabularySummary.textContent =
      `Recognized vocabulary ${UNIT_REGISTRY_VERSION} — ` +
      `${UNIT_REGISTRY_POPULATION.families} families, ` +
      `${UNIT_REGISTRY_POPULATION.aliases} aliases, ${UNIT_REGISTRY_POPULATION.units} unit tokens`;
    vocabulary.appendChild(vocabularySummary);
    const familyList = document.createElement("div");
    familyList.className = "form-hint";
    familyList.textContent = UNIT_REGISTRY_FAMILIES.map(
      (family) =>
        `${family.family} [${family.quantityKind}, ${family.canonicalUnit}]: ${family.aliases.join(", ")}`,
    ).join("\n");
    familyList.style.whiteSpace = "pre-wrap";
    vocabulary.appendChild(familyList);
    const unitList = document.createElement("div");
    unitList.className = "form-hint";
    unitList.textContent = `Unit tokens: ${UNIT_REGISTRY_UNITS.map((unit) => unit.token).join(", ")}`;
    vocabulary.appendChild(unitList);
    wrap.appendChild(vocabulary);

    // The mock's footer line, and a true statement of the store's rules: sets
    // auto-suffix (never overwrite) and RAW keeps absolute read priority.
    const provNote = document.createElement("div");
    provNote.className = "import-prov-note";
    provNote.textContent =
      "Every import is versioned with provenance — re-importing never overwrites RAW.";
    wrap.appendChild(provNote);

    const actions = document.createElement("div");
    actions.className = "form-actions";
    const cancelBtn = document.createElement("button");
    cancelBtn.className = "btn";
    cancelBtn.textContent = "Cancel";
    const okBtn = document.createElement("button");
    okBtn.className = "btn btn-accent";
    okBtn.textContent = "Import";
    actions.append(cancelBtn, okBtn);
    wrap.appendChild(actions);

    // `settled` guards the single-resolve contract: close() runs on Esc/✕ too, and without
    // it a user who clicks Import and then presses Escape would resolve the promise twice
    // (the second time as a cancel), silently discarding a running import's choice.
    let settled = false;
    const finish = (choice: ImportSetChoice | null) => {
      if (settled) return;
      settled = true;
      close();
      resolve(choice);
    };

    const close = openModal("Import LAS — curve set", wrap, 560);
    cancelBtn.addEventListener("click", () => finish(null));
    okBtn.addEventListener("click", () => {
      const confirmedWellNames: Record<string, string> = {};
      for (const [path, input] of identityInputs) {
        const confirmed = input.value.trim();
        if (!confirmed) {
          input.setCustomValidity("Enter and confirm an identity; the filename cannot be selected silently.");
          input.reportValidity();
          input.setCustomValidity("");
          return;
        }
        confirmedWellNames[path] = confirmed;
      }
      if (samplingStyle.value !== "CONTINUOUS_REGULAR" && samplingStyle.value !== "CONTINUOUS_IRREGULAR") {
        samplingStyle.setCustomValidity("Declare whether this curve set is continuous regular or continuous irregular.");
        samplingStyle.reportValidity();
        samplingStyle.setCustomValidity("");
        return;
      }
      let tolerance: { value: number; unit: "M" | "FT" } | null = null;
      if (samplingStyle.value === "CONTINUOUS_REGULAR") {
        const value = Number(toleranceValue.value);
        if (toleranceValue.value.trim() === "" || !Number.isFinite(value) || value < 0) {
          toleranceValue.setCustomValidity("Enter an explicit finite non-negative verification tolerance.");
          toleranceValue.reportValidity();
          toleranceValue.setCustomValidity("");
          return;
        }
        if (toleranceUnit.value !== "M" && toleranceUnit.value !== "FT") {
          toleranceUnit.setCustomValidity("Choose the tolerance unit.");
          toleranceUnit.reportValidity();
          toleranceUnit.setCustomValidity("");
          return;
        }
        tolerance = { value, unit: toleranceUnit.value };
      }
      finish({
        setName: setInput.value.trim(),
        attach: attachBox.checked,
        fileDepthUnit: undeclaredUnit.value === "M" || undeclaredUnit.value === "FT"
          ? undeclaredUnit.value
          : null,
        undeclaredDrhoUnit:
          undeclaredDrhoUnit.value === "g/cc" || undeclaredDrhoUnit.value === "kg/m3"
            ? undeclaredDrhoUnit.value
            : null,
        samplingStyle: samplingStyle.value,
        samplingStyleVerifyTolerance: tolerance,
        confirmedWellNames,
      });
    });
    setInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") okBtn.click();
    });
    // The dialog can also be dismissed by Esc/✕ inside openModal, which does NOT call
    // finish — so watch for the dialog leaving the DOM and resolve as a cancel.
    const root = document.querySelector<HTMLElement>("#modal-root");
    if (root) {
      const observer = new MutationObserver(() => {
        if (!wrap.isConnected) {
          observer.disconnect();
          finish(null);
        }
      });
      observer.observe(root, { childList: true });
    }
    setInput.focus();
    setInput.select();
  });
}
