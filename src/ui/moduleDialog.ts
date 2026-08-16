import {
  despikeContaminationPreview,
  listCurveCatalog,
  moduleInputAvailability,
  moduleOutputNames,
  runWorkflowModule,
  type ModuleInputAvailability,
  type ModuleSpec,
  type OutputName,
  type RunModuleRequest,
  type DespikeContaminationPreview,
  type ValidityCondition,
} from "../ipc";
import { appState } from "../state";
import { buildLogSetPicker } from "./logSetPicker";
import { formRow, openModal } from "./modal";
import { withParamSources } from "./paramSources";
import { buildRunCustodyControls } from "./runCustody";
import { buildWellScope } from "./wellScope";

export interface ModulePaneCallbacks {
  /** Called after a successful run so the host can refresh the catalog/layout and record which
   *  wells were actually run (for the History panel) rather than the globally-selected well. */
  onRunComplete: (outputCurves: string[], wellNames: string[]) => void;
  setStatus: (text: string) => void;
}

/** Flag curves offered as Mask choices even before they first exist in the catalog,
 *  so the canonical chain (badhole → condflag → solvers with Mask) is composable in a
 *  fresh project. The catalog only lists curves that have actually been computed. */
export const MASK_CURVE_SUGGESTIONS = ["BADHOLE", "COND_FLAG"];
export const PRECONDITION_POLICY_OPT = "__PRECONDITION_POLICY";
export const PRECONDITION_POLICY_REFUSE = "REFUSE";
export const PRECONDITION_POLICY_FLAG_VALID_SAMPLES = "FLAG_VALID_SAMPLES";
export const PRECONDITION_FLAG_OUTPUT_ARG = "__PRECONDITION_FLAG";
export const AUTO_INPUT_ALIAS = "__AUTO_PREFERRED_ALIAS__";

/** Catalog names with the well-known flag curves prepended when absent. */
export function maskCurveNames(curveNames: string[]): string[] {
  return [...MASK_CURVE_SUGGESTIONS.filter((s) => !curveNames.includes(s)), ...curveNames];
}

/** The module manifest is the user-facing custody surface for a method's preconditions. Keep the
 * statement, activation branch and source together so the dialog never shows a bare range whose
 * authority has to be guessed. */
export function argumentHint(arg: ModuleSpec["args"][number]): string {
  const defaultSource = arg.kind !== "param"
    ? ""
    : arg.default_source === "ABSENT"
      ? "Default: ABSENT — no numeric value ships; supply an interpreter value when the selected method requires it."
      : `Default source: ${arg.default_source}.`;
  const unitCustody = arg.kind === "param" && arg.default_unit_custody
    ? `Artefact value: ${arg.default_unit_custody.artefact_value} ${arg.default_unit_custody.artefact_unit}. `
      + `Canonical value: ${arg.default_unit_custody.canonical_value} ${arg.default_unit_custody.canonical_unit}. `
      + `Named conversion: ${arg.default_unit_custody.conversion.identity}. `
      + `Derivation: ${arg.default_unit_custody.conversion.derivation}.`
    : "";
  const conditions = (arg.validity_conditions ?? []).map((condition) => {
    const branch = "when" in condition && condition.when
      ? ` When ${condition.when.argument} = ${condition.when.equals}.`
      : "";
    return `Condition ${condition.id}: ${condition.statement}${branch} Source: ${condition.source}.`;
  });
  const aliasPreference = arg.preferred_aliases?.length
    ? `Automatic input preference: ${arg.preferred_aliases.join(" → ")}. The curve resolved for each well is recorded in run provenance.`
    : "";
  const guidance = (arg.guidance ?? []).map(
    (item) => `Guidance: ${item.text} Source: ${item.source}.`,
  );
  return [arg.desc, aliasPreference, ...guidance, unitCustody, defaultSource, ...conditions].filter(Boolean).join(" ");
}

export interface ValidityConditionView {
  id: string;
  statement: string;
  source: string;
  state: "checking" | "evaluable" | "un_evaluable" | "inactive" | "check_failed";
  status: string;
}

function conditionDependencies(
  owner: ModuleSpec["args"][number],
  condition: ValidityCondition,
  spec: ModuleSpec,
): { all: string[]; any: string[] } {
  const isLog = (name: string) => spec.args.some((argument) => argument.name === name && argument.kind === "log_in");
  if (condition.kind === "required_companion") return { all: [], any: condition.any_of };
  if (condition.kind === "required_where_finite") return { all: [condition.input, owner.name], any: [] };
  if (condition.kind === "numeric_range" && owner.kind === "log_in") return { all: [owner.name], any: [] };
  if (condition.kind === "less_than") {
    return { all: [owner.name, condition.other].filter(isLog), any: [] };
  }
  return { all: [], any: [] };
}

/** Convert the backend's finite-input preflight into the visible state of each sourced condition.
 * The same helper is used by the acceptance test and the live pane, including the positive control:
 * “available” is never inferred merely because a project-wide picker offered the mnemonic. */
export function validityConditionViews(
  owner: ModuleSpec["args"][number],
  spec: ModuleSpec,
  selectedValues: Record<string, string>,
  availability: ModuleInputAvailability[] | null,
  wellName: (wellId: string) => string,
): ValidityConditionView[] {
  return (owner.validity_conditions ?? []).map((condition) => {
    const base = { id: condition.id, statement: condition.statement, source: condition.source };
    const branch = "when" in condition ? condition.when : null;
    if (branch && selectedValues[branch.argument] !== branch.equals) {
      return {
        ...base,
        state: "inactive" as const,
        status: `Not active for this run — applies when ${branch.argument} = ${branch.equals}.`,
      };
    }
    const dependencies = conditionDependencies(owner, condition, spec);
    if (dependencies.all.length === 0 && dependencies.any.length === 0) {
      return {
        ...base,
        state: "evaluable" as const,
        status: "Evaluated from the entered or selected value before the module body runs.",
      };
    }
    if (availability === null) {
      return { ...base, state: "checking" as const, status: "Checking selected well inputs…" };
    }
    if (availability.length === 0) {
      return {
        ...base,
        state: "un_evaluable" as const,
        status: "Cannot evaluate before run — no well is in scope.",
      };
    }
    const failed = availability.filter((row) => row.error);
    if (failed.length > 0) {
      return {
        ...base,
        state: "check_failed" as const,
        status: `Cannot complete the pre-run input check for ${failed.map((row) => wellName(row.well_id)).join(", ")}.`,
      };
    }
    const missing = availability.filter((row) => {
      const available = new Set(row.available_arguments);
      return dependencies.all.some((name) => !available.has(name))
        || (dependencies.any.length > 0 && !dependencies.any.some((name) => available.has(name)));
    });
    const selectedLabel = (name: string) => {
      const selected = selectedValues[name] ?? spec.args.find((argument) => argument.name === name)?.default ?? "";
      return selected && selected !== name ? `${name} (${selected})` : name;
    };
    const required = [
      ...dependencies.all.map(selectedLabel),
      ...(dependencies.any.length > 0 ? [`one of ${dependencies.any.map(selectedLabel).join(" / ")}`] : []),
    ].join(" and ");
    if (missing.length > 0) {
      return {
        ...base,
        state: "un_evaluable" as const,
        status: `Cannot evaluate before run for ${missing.map((row) => wellName(row.well_id)).join(", ")} — required input ${required} is absent.`,
      };
    }
    return {
      ...base,
      state: "evaluable" as const,
      status: `Inputs available before run for ${availability.length} well${availability.length === 1 ? "" : "s"}: ${required}.`,
    };
  });
}

/** Render source, condition and preflight state as text beside the field. A title/tooltip is not
 * sufficient: the user must see an unavailable condition without discovering it by hovering. */
export function renderValidityConditions(host: HTMLElement, views: ValidityConditionView[]): void {
  host.innerHTML = "";
  for (const view of views) {
    const item = document.createElement("div");
    item.className = `module-validity-item module-validity-${view.state}`;
    const condition = document.createElement("div");
    condition.className = "module-validity-condition";
    condition.textContent = `${view.id} — ${view.statement}`;
    const source = document.createElement("div");
    source.className = "module-validity-source";
    source.textContent = `Source: ${view.source}`;
    const status = document.createElement("div");
    status.className = "module-validity-status";
    status.textContent = view.status;
    item.append(condition, source, status);
    host.appendChild(item);
  }
}

const DESPIKE_MASKING_MEANING =
  "Above this fraction of contaminated samples in a window, spikes mask each other and are not detected.";

/** Render the data-resolved estimator branches rather than inferring one formula from K. A run
 * may legitimately contain both branches, so both remain visible with their sample counts. */
export function renderDespikeContaminationPreview(
  host: HTMLElement,
  preview: DespikeContaminationPreview,
  wellName: (wellId: string) => string,
): void {
  host.innerHTML = "";
  const meaning = document.createElement("div");
  meaning.className = "module-contamination-meaning";
  meaning.textContent = DESPIKE_MASKING_MEANING;
  host.appendChild(meaning);

  for (const branch of preview.branches) {
    const row = document.createElement("div");
    row.className = "module-contamination-branch";
    const name = branch.estimator === "TRUE_MAD"
      ? "True MAD"
      : branch.estimator === "MEAN_DEVIATION_FALLBACK"
        ? "Mean-deviation fallback (zero MAD)"
        : "Mean ± kσ (population σ)";
    row.textContent = `${name}: ${branch.ceiling_pct.toFixed(2)}% — ${branch.sample_count} evaluated sample window${branch.sample_count === 1 ? "" : "s"}`;
    host.appendChild(row);
  }

  const coverage = document.createElement("div");
  coverage.className = "module-contamination-coverage";
  coverage.textContent = preview.evaluated_wells > 0
    ? `Computed from the selected curve and current window in ${preview.evaluated_wells} well${preview.evaluated_wells === 1 ? "" : "s"}.`
    : "No selected well currently has an evaluable Hampel window.";
  host.appendChild(coverage);

  if (preview.unavailable_well_ids.length > 0) {
    const unavailable = document.createElement("div");
    unavailable.className = "module-contamination-warning";
    unavailable.textContent = `Curve unavailable after the current mask: ${preview.unavailable_well_ids.map(wellName).join(", ")}.`;
    host.appendChild(unavailable);
  }
  if (preview.issues.length > 0) {
    const issues = document.createElement("div");
    issues.className = "module-contamination-warning";
    issues.textContent = preview.issues.map((issue) => `${wellName(issue.well_id)} — ${issue.error}`).join("; ");
    host.appendChild(issues);
  }
}

/** Builds the auto-generated parameter form for one module: input-curve selectors,
 *  option dropdowns, and validated numeric parameters — all straight from the manifest
 *  (module-manifest model). Hosted as a dock pane (workspace component "module", panel id
 *  "module:<name>"), not a popup — one singleton pane per module, so every module the
 *  backend registers gets its pane with no frontend work. The pane is persistent: the
 *  well list and curve dropdowns refresh on data changes (keeping the user's choices),
 *  so curves produced by one run are immediately selectable in the next. Zone-level
 *  overrides come from the Zones pane; values here are the whole-well defaults. */
export async function buildModuleContent(
  spec: ModuleSpec,
  callbacks: ModulePaneCallbacks,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  let catalog = await listCurveCatalog();
  let curveNames = catalog.map((c) => c.name);
  let disposed = false;
  let refreshValidityConditions: () => Promise<void> = async () => {};
  let refreshDespikeCeiling: () => Promise<void> = async () => {};
  const scope = await buildWellScope({
    onChange: () => {
      void refreshValidityConditions();
      void refreshDespikeCeiling();
    },
  });
  const custodyControls = buildRunCustodyControls();

  const content = document.createElement("div");
  content.className = "module-pane";

  // --- Header (design 1d): icon chip + display-face title + Help on the right.
  // The chip shows the module's initial — the manifest carries no icon, and a
  // hand-kept icon registry would break rule 9 (new modules need no UI work).
  // The Help button surfaces spec.doc in the same modal the ribbon ? uses; the
  // pane already holds the spec, so no workspace wiring is needed.
  const shortName = (spec.title.split("—")[0] ?? "").trim() || spec.name.toUpperCase();
  const head = document.createElement("div");
  head.className = "module-head";
  const chip = document.createElement("span");
  chip.className = "module-chip";
  chip.textContent = (shortName[0] ?? "?").toUpperCase();
  const titleEl = document.createElement("span");
  titleEl.className = "module-title";
  titleEl.textContent = spec.title;
  const helpBtn = document.createElement("button");
  helpBtn.type = "button";
  helpBtn.className = "btn module-help";
  helpBtn.textContent = "? Help";
  helpBtn.addEventListener("click", () => {
    const body = document.createElement("p");
    body.className = "help-body";
    body.textContent = spec.doc || "Documentation for this module is unavailable.";
    openModal(`Help — ${spec.title}`, body, 480);
  });
  head.append(chip, titleEl, helpBtn);
  content.appendChild(head);

  // --- Well selection (scope, not a checklist) ---
  // The scope resolves against live state at run time (group / ★ pinned / selection / all), so a
  // 2000-well field never needs a well-by-well checklist here.
  content.appendChild(scope.el);

  // Labeled fields live in a responsive two-column grid (design 1d); at narrow
  // pane widths it collapses to one column on its own.
  const argsGrid = document.createElement("div");
  argsGrid.className = "module-args";

  // --- Args from manifest ---
  const logSelects = new Map<string, HTMLSelectElement>();
  const optSelects = new Map<string, HTMLSelectElement>();
  const textInputs = new Map<string, HTMLInputElement>();
  const paramInputs = new Map<string, HTMLInputElement>();
  const validityHosts = new Map<string, HTMLElement>();

  const withVisibleValidity = (arg: ModuleSpec["args"][number], control: HTMLElement): HTMLElement => {
    if (!(arg.validity_conditions?.length)) return control;
    const wrap = document.createElement("div");
    wrap.className = "module-control-with-validity";
    const host = document.createElement("div");
    host.className = "module-validity-list";
    validityHosts.set(arg.name, host);
    renderValidityConditions(host, validityConditionViews(arg, spec, {}, null, (wellId) => wellId));
    wrap.append(control, host);
    return wrap;
  };

  /** `labels` is parallel to `names` and optional — a missing or short entry shows the id, which
   *  is what every un-labelled module still does. The VALUE is always the id, because it is what
   *  `params_json` stores on every saved run (`docs/review_triage.md` finding 21). */
  const fillSelect = (
    select: HTMLSelectElement,
    names: string[],
    selected: string,
    labels?: string[]
  ) => {
    select.innerHTML = "";
    for (const [i, name] of names.entries()) {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = labels?.[i] || name;
      if (name === selected) option.selected = true;
      select.appendChild(option);
    }
  };
  /** Fills a log-input <select>: catalog names, plus the current/default choice prepended when
   *  absent (so a selection never disappears from its own dropdown), plus a leading "(none)"
   *  for OPTIONAL inputs so the user can deliberately drop a slot even when a curve of that
   *  name exists — the module doc's advertised "any unwanted curve slot is dropped" behaviour.
   *  An empty value ("") is sent to the backend, which resolves it as an absent (all-NaN) input. */
  const fillLogSelect = (select: HTMLSelectElement, arg: (typeof spec.args)[number], current: string) => {
    select.innerHTML = "";
    if (arg.preferred_aliases?.length) {
      const automatic = document.createElement("option");
      automatic.value = AUTO_INPUT_ALIAS;
      automatic.textContent = `Auto — ${arg.preferred_aliases.join(" → ")}`;
      select.appendChild(automatic);
    }
    if (!arg.required) {
      const none = document.createElement("option");
      none.value = "";
      none.textContent = "(none)";
      select.appendChild(none);
    }
    const names = current === "" || current === AUTO_INPUT_ALIAS || curveNames.includes(current)
      ? curveNames
      : [current, ...curveNames];
    for (const name of names) {
      const o = document.createElement("option");
      o.value = name;
      o.textContent = name;
      select.appendChild(o);
    }
    select.value = current; // "" → (none) for optional; else the chosen/default mnemonic
  };

  for (const arg of spec.args) {
    if (arg.kind === "log_in") {
      const select = document.createElement("select");
      select.className = "form-control";
      fillLogSelect(select, arg, arg.preferred_aliases?.length ? AUTO_INPUT_ALIAS : arg.default);
      logSelects.set(arg.name, select);
      const requirement = arg.required_any_of?.length
        ? `(one of ${[arg.name, ...arg.required_any_of].join(" / ")})`
        : arg.required
          ? ""
          : "(optional)";
      argsGrid.appendChild(formRow(`${arg.name} ${requirement}`, withVisibleValidity(arg, select), argumentHint(arg)));
    } else if (arg.kind === "option") {
      const select = document.createElement("select");
      select.className = "form-control";
      const optionalAndUnset = !arg.required && arg.default === "";
      fillSelect(
        select,
        optionalAndUnset ? ["", ...arg.choices] : arg.choices,
        arg.default,
        optionalAndUnset
          ? ["Choose when the corresponding input is present", ...(arg.choice_labels ?? [])]
          : arg.choice_labels,
      );
      optSelects.set(arg.name, select);
      argsGrid.appendChild(
        formRow(`${arg.name}${arg.required ? "" : " (optional)"}`, withVisibleValidity(arg, select), argumentHint(arg)),
      );
    } else if (arg.kind === "text") {
      // Free-typed run option (ArgKind::Text) — the Condition family's user-named output curve.
      const input = document.createElement("input");
      input.className = "form-control";
      input.type = "text";
      input.value = arg.default;
      // The description carries what a blank means, so it belongs in the field too — a
      // placeholder is where a user looks before reading a hint.
      input.placeholder = arg.desc.includes("blank") ? arg.desc.split("—").pop()!.trim() : "";
      textInputs.set(arg.name, input);
      argsGrid.appendChild(formRow(arg.name, withVisibleValidity(arg, input), argumentHint(arg)));
    } else if (arg.kind === "param") {
      const input = document.createElement("input");
      input.className = "form-control";
      input.type = "number";
      input.step = "any";
      input.value = arg.default;
      // A manifest with NO default opens EMPTY on purpose (`modules::param_open`) — a despike
      // window and a gap limit have no value that is right in two basins, so the user states one.
      if (!arg.default) {
        input.placeholder = arg.required
          ? "set a value"
          : arg.min === null && arg.max === null
            ? "optional value"
            : "no bound";
      }
      if (arg.min !== null) input.min = String(arg.min);
      if (arg.max !== null) input.max = String(arg.max);
      paramInputs.set(arg.name, input);
      // The unit sits to the RIGHT of the input (design 1d), no longer inside
      // the label — the value and its unit read as one fact.
      let control: HTMLElement = input;
      if (arg.unit) {
        const wrap = document.createElement("div");
        wrap.className = "input-unit";
        const u = document.createElement("span");
        u.className = "unit-suffix";
        u.textContent = arg.unit;
        wrap.append(input, u);
        control = wrap;
      }
      // SB-CORE-013. Where the corpus records competing shipped values for this parameter, they
      // are shown WITH the field rather than in a manual — the point of choice is the only place
      // the disagreement can change a decision. Renders nothing when the topic has no entries, so
      // this is safe on every numeric arg.
      if (arg.sources_topic) {
        control = withParamSources(control, arg.sources_topic);
      }
      argsGrid.appendChild(formRow(arg.name, withVisibleValidity(arg, control), argumentHint(arg)));
    }
  }

  // --- Universal bad-hole mask (optional) ---
  // Applies to every module via the runner: samples where the chosen flag curve == 1 are
  // set missing in all outputs, so flagged intervals never pollute results. Typically the
  // BADHOLE curve produced by the Bad-Hole QC module.
  const maskSelect = document.createElement("select");
  maskSelect.className = "form-control";
  const rebuildMaskOptions = (selected: string) => {
    maskSelect.innerHTML = "";
    const none = document.createElement("option");
    none.value = "";
    none.textContent = "(none)";
    maskSelect.appendChild(none);
    const names = maskCurveNames(curveNames);
    if (selected && !names.includes(selected)) names.unshift(selected);
    for (const name of names) {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = name;
      if (name === selected) option.selected = true;
      maskSelect.appendChild(option);
    }
  };
  rebuildMaskOptions("");
  argsGrid.appendChild(
    formRow("Mask (optional)", maskSelect, "Flag curve (=1 bad) to blank out of every output — e.g. BADHOLE."),
  );

  // A source-bearing precondition still refuses by default. The partial-result policy is an
  // explicit interpreter choice because it adds a degraded run, a companion flag curve and
  // durable violation provenance; silently changing the default would make old saved runs mean
  // something new.
  const preconditionPolicySelect = document.createElement("select");
  preconditionPolicySelect.className = "form-control";
  for (const [value, label] of [
    [PRECONDITION_POLICY_REFUSE, "Refuse this well"],
    [PRECONDITION_POLICY_FLAG_VALID_SAMPLES, "Keep valid samples and write a flag curve"],
  ] as const) {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = label;
    preconditionPolicySelect.appendChild(option);
  }
  argsGrid.appendChild(
    formRow(
      "When only some samples violate a condition",
      preconditionPolicySelect,
      "Refusal remains the default. The flag policy keeps unaffected samples, writes 1 at every violating sample and records the condition, value, expected range and source in run provenance.",
    ),
  );

  // --- Input / output log set: the ONE shared control (`logSetPicker.ts`). Was two bespoke
  // blocks here labelled "Input cons" / "Output cons" — the store, the backend and the docs all
  // say LOG SET, and only this UI said constellation, which is why the word did not connect.
  const setPicker = buildLogSetPicker({
    write: "INTERP",
    onInputChange: () => {
      void refreshValidityConditions();
      void refreshDespikeCeiling();
    },
  });
  for (const row of setPicker.rows) argsGrid.appendChild(row);
  for (const row of custodyControls.rows) argsGrid.appendChild(row);

  content.appendChild(argsGrid);

  const contaminationCard = document.createElement("section");
  contaminationCard.className = "module-contamination";
  contaminationCard.hidden = spec.name !== "despike";
  const contaminationTitle = document.createElement("div");
  contaminationTitle.className = "module-contamination-title";
  contaminationTitle.textContent = "Live contamination ceiling";
  const contaminationBody = document.createElement("div");
  contaminationBody.className = "module-contamination-body";
  contaminationBody.textContent =
    "Set WINDOW and K to inspect the estimator branch the selected curve will actually use.";
  contaminationCard.append(contaminationTitle, contaminationBody);
  content.appendChild(contaminationCard);

  // --- Zone-override callout (design 1d): the precedence rule, stated where
  // the whole-well defaults are being typed rather than in a help page.
  const callout = document.createElement("div");
  callout.className = "module-callout";
  callout.textContent =
    "Values here are the whole-well defaults — per-zone parameters from the Zones pane take precedence inside their zones.";
  content.appendChild(callout);

  // --- Output curves: a row per declared output with the name it will be written under.
  //
  // Jauhar, 2026-08-05: *"naming each output curve in bulk when modules gonna run"*. Every name a
  // run is about to write is on screen at once, editable, before the run — rather than discovered
  // afterwards in the Curve Catalog. The Condition and Frame families used to carry their own
  // "Output curve name" text field for this; that field is gone, because the grid IS it and one
  // control for forty modules cannot drift the way two could.
  //
  // The DEFAULT names come from the backend (`moduleOutputNames`), never from expanding the
  // manifest's patterns here — a `{CURVE}_C` expanded in TypeScript would be a second copy of a
  // naming rule, agreeing with the run right up until one of them changed.
  const outArgs = spec.args.filter((a) => a.kind === "log_out");
  const outSection = document.createElement("div");
  outSection.className = "module-outputs";
  const outHead = document.createElement("div");
  outHead.className = "module-outputs-head";
  const outTitle = document.createElement("span");
  outTitle.className = "module-outputs-title";
  outTitle.textContent = outArgs.length === 1 ? "Output curve" : "Output curves";
  // The bulk fill: one prefix over every name below, for a trial run that must not land on the
  // interpretation the field is already using. Applied by the backend on top of the names in the
  // grid, so the two compose rather than competing.
  const prefixInput = document.createElement("input");
  prefixInput.className = "form-control module-outputs-prefix";
  prefixInput.type = "text";
  prefixInput.placeholder = "prefix all, e.g. TEST_";
  prefixInput.title =
    "Prefixes every curve this run writes, on top of the names below — TEST_ gives TEST_VSH. " +
    "A Monte Carlo study refuses a renamed step, because its cutoffs are resolved from the " +
    "module's declared output names.";
  outHead.append(outTitle, prefixInput);
  outSection.appendChild(outHead);

  const outNameInputs = new Map<string, HTMLInputElement>();
  const outGrid = document.createElement("div");
  outGrid.className = "module-outputs-grid";
  for (const arg of outArgs) {
    const label = document.createElement("label");
    label.className = "module-output-label";
    label.textContent = arg.desc || arg.name;
    if (arg.unit) label.textContent += ` (${arg.unit})`;
    if (arg.flag_kind === "EXCLUSION_MASK") label.textContent += " — exclusion mask";
    if (arg.flag_kind === "DIAGNOSTIC_INDICATOR") label.textContent += " — diagnostic indicator";
    if (arg.porosity_output) {
      const role = arg.porosity_output.output_role.toLowerCase().replace(/_/g, " ");
      const method = arg.porosity_output.method.replace(/_/g, " ");
      label.textContent += ` — POR ${role} · ${method}`;
      label.title =
        `Declared as ${arg.name}; family ${arg.porosity_output.family}; ` +
        `convention ${arg.porosity_output.convention}; limit policy ` +
        `${arg.porosity_output.limiting_policy} (${arg.porosity_output.limiting_policy_source}); ` +
        `reason contract ${arg.porosity_output.flag_contract} is ${arg.porosity_output.flag_emission}; ` +
        `output naming ${arg.porosity_output.output_naming_contract}.`;
    } else {
      label.title = `Declared as ${arg.name}`;
    }
    const input = document.createElement("input");
    input.className = "form-control module-output-name";
    input.type = "text";
    input.spellcheck = false;
    label.appendChild(input);
    outNameInputs.set(arg.name, input);
    outGrid.appendChild(label);
  }
  const preconditionFlagLabel = document.createElement("label");
  preconditionFlagLabel.className = "module-output-label";
  preconditionFlagLabel.textContent = "Precondition companion flag — diagnostic indicator (1 = violation)";
  preconditionFlagLabel.title = "Framework-owned companion output; its deterministic name cannot be separated from the flagged-result contract.";
  preconditionFlagLabel.hidden = true;
  const preconditionFlagName = document.createElement("input");
  preconditionFlagName.className = "form-control module-output-name";
  preconditionFlagName.type = "text";
  preconditionFlagName.readOnly = true;
  preconditionFlagName.tabIndex = -1;
  preconditionFlagLabel.appendChild(preconditionFlagName);
  outGrid.appendChild(preconditionFlagLabel);
  outSection.appendChild(outGrid);
  const outWarn = document.createElement("div");
  outWarn.className = "module-outputs-warn";
  outWarn.hidden = true;
  outSection.appendChild(outWarn);
  content.appendChild(outSection);

  /** The run options as the form currently stands — the same map the preview and the run send, so
   *  what the grid shows can never describe a different run from the one the button starts. */
  const collectOpts = (): Record<string, string> => {
    const opts: Record<string, string> = {};
    for (const [name, select] of optSelects) opts[name] = select.value;
    // A blank Text arg is sent as a blank, not dropped: the module's own default is what an empty
    // box means, and omitting the key would make a cleared field indistinguishable from one that
    // was never on the dialog.
    for (const [name, input] of textInputs) opts[name] = input.value.trim();
    for (const [arg, input] of outNameInputs) {
      const typed = input.value.trim();
      if (typed) opts[`__OUT_${arg}`] = typed;
    }
    if (maskSelect.value) opts.MASK = maskSelect.value;
    if (preconditionPolicySelect.value === PRECONDITION_POLICY_FLAG_VALID_SAMPLES) {
      opts[PRECONDITION_POLICY_OPT] = PRECONDITION_POLICY_FLAG_VALID_SAMPLES;
    }
    if (prefixInput.value.trim()) opts.OUT_PREFIX = prefixInput.value.trim();
    return opts;
  };
  const collectLogInputs = (): Record<string, string> => {
    const logInputs: Record<string, string> = {};
    for (const [name, select] of logSelects) {
      if (select.value !== AUTO_INPUT_ALIAS) logInputs[name] = select.value;
    }
    return logInputs;
  };

  const collectConditionSelections = (): Record<string, string> => {
    const selected = collectLogInputs();
    for (const [name, select] of optSelects) selected[name] = select.value;
    for (const [name, input] of textInputs) selected[name] = input.value.trim();
    return selected;
  };

  const collectFiniteParams = (): Record<string, number> => {
    const params: Record<string, number> = {};
    for (const [name, input] of paramInputs) {
      if (!input.value.trim()) continue;
      const value = Number(input.value);
      if (Number.isFinite(value)) params[name] = value;
    }
    return params;
  };

  let validityGen = 0;
  refreshValidityConditions = async () => {
    if (validityHosts.size === 0) return;
    const gen = ++validityGen;
    const selected = collectConditionSelections();
    const needsWellInputs = spec.args.some((arg) =>
      (arg.validity_conditions ?? []).some((condition) => {
        const dependencies = conditionDependencies(arg, condition, spec);
        return dependencies.all.length > 0 || dependencies.any.length > 0;
      }));
    const nameFor = (wellId: string) => scope.namesFor([wellId])[0] ?? wellId;
    const render = (availability: ModuleInputAvailability[] | null) => {
      for (const arg of spec.args) {
        const host = validityHosts.get(arg.name);
        if (host) renderValidityConditions(host, validityConditionViews(arg, spec, selected, availability, nameFor));
      }
    };
    render(null);
    if (!needsWellInputs) return;
    if (scope.getWellIds().length === 0) {
      render([]);
      return;
    }
    try {
      const availability = await moduleInputAvailability(
        spec.name,
        scope.backend(),
        collectLogInputs(),
        setPicker.inputSet(),
      );
      if (disposed || gen !== validityGen) return;
      render(availability);
    } catch (error) {
      if (disposed || gen !== validityGen) return;
      render(scope.getWellIds().map((wellId) => ({
        well_id: wellId,
        available_arguments: [],
        error: String(error),
      })));
    }
  };

  let ceilingGen = 0;
  let ceilingBusy = false;
  let ceilingQueued = false;
  refreshDespikeCeiling = async () => {
    if (spec.name !== "despike" || disposed) return;
    ++ceilingGen;
    if (ceilingBusy) {
      ceilingQueued = true;
      return;
    }
    do {
      ceilingBusy = true;
      ceilingQueued = false;
      const gen = ceilingGen;
      const method = optSelects.get("OPT_METHOD")?.value ?? "HAMPEL";
      const windowValue = Number(paramInputs.get("WINDOW")?.value ?? "");
      const kValue = Number(paramInputs.get("K")?.value ?? "");
      if (method !== "HAMPEL") {
        contaminationBody.textContent = `${method} does not use K; no K-based contamination ceiling applies.`;
      } else if (!(windowValue > 0) || !(kValue > 0)) {
        contaminationBody.textContent =
          "Set WINDOW and K to inspect the estimator branch the selected curve will actually use.";
      } else if (scope.getWellIds().length === 0) {
        contaminationBody.textContent = "No wells are in scope, so the estimator branch cannot be evaluated.";
      } else {
        contaminationBody.textContent = "Checking the selected curve, window and mask…";
        try {
          const preview = await despikeContaminationPreview(
            scope.backend(),
            collectLogInputs(),
            collectFiniteParams(),
            collectOpts(),
            setPicker.inputSet(),
          );
          if (!disposed && gen === ceilingGen) {
            const nameFor = (wellId: string) => scope.namesFor([wellId])[0] ?? wellId;
            renderDespikeContaminationPreview(contaminationBody, preview, nameFor);
          }
        } catch (error) {
          if (!disposed && gen === ceilingGen) {
            contaminationBody.textContent =
              `Cannot evaluate the live ceiling: ${String(error).replace(/^Error:\s*/, "")}`;
          }
        }
      }
      ceilingBusy = false;
    } while (ceilingQueued && !disposed);
  };

  // The placeholder in each name box is the name the run would write if the box is left alone —
  // so an untouched form still SHOWS its answer rather than an empty field. Re-asked whenever an
  // input curve or a rename changes, because a pattern is built from those: renaming the despiked
  // curve to GR_ED moves its flag to GR_ED_SPK, and the grid has to say so.
  let outGen = 0;
  const refreshOutNames = async () => {
    const gen = ++outGen;
    try {
      const names: OutputName[] = await moduleOutputNames(spec.name, collectLogInputs(), collectOpts());
      if (disposed || gen !== outGen) return;
      for (const n of names) outNameInputs.get(n.arg)?.setAttribute("placeholder", n.name);
      const flag = names.find((name) => name.arg === PRECONDITION_FLAG_OUTPUT_ARG);
      preconditionFlagLabel.hidden = !flag;
      preconditionFlagName.value = flag?.name ?? "";
      outTitle.textContent = names.length === 1 ? "Output curve" : "Output curves";
      outWarn.hidden = true;
    } catch (e) {
      if (disposed || gen !== outGen) return;
      // A refusal (a shadowed name, a collision) is shown HERE, beside the box that caused it,
      // rather than saved up for the run — the user is looking at this grid.
      // The backend's message is the whole message; a leading "Error:" from the Error wrapper
      // just pushes the part that names the curve further from the eye.
      outWarn.textContent = String(e).replace(/^Error:\s*/, "");
      outWarn.hidden = false;
    }
  };
  for (const [, select] of logSelects) select.addEventListener("change", () => {
    void refreshOutNames();
    void refreshValidityConditions();
    void refreshDespikeCeiling();
  });
  for (const [, select] of optSelects) select.addEventListener("change", () => {
    void refreshOutNames();
    void refreshValidityConditions();
    void refreshDespikeCeiling();
  });
  for (const [, input] of textInputs) input.addEventListener("input", () => void refreshValidityConditions());
  let ceilingTimer: number | undefined;
  const scheduleCeilingRefresh = () => {
    if (ceilingTimer !== undefined) window.clearTimeout(ceilingTimer);
    ceilingTimer = window.setTimeout(() => {
      ceilingTimer = undefined;
      void refreshDespikeCeiling();
    }, 250);
  };
  for (const [, input] of paramInputs) input.addEventListener("input", scheduleCeilingRefresh);
  maskSelect.addEventListener("change", () => void refreshDespikeCeiling());
  preconditionPolicySelect.addEventListener("change", () => void refreshOutNames());
  for (const [, input] of outNameInputs) input.addEventListener("input", () => void refreshOutNames());
  void refreshOutNames();
  void refreshValidityConditions();
  void refreshDespikeCeiling();

  // --- Footer (design 1d): primary pill naming the module, last-run status
  // right-aligned beside it.
  const runBtn = document.createElement("button");
  runBtn.className = "btn btn-accent";
  runBtn.textContent = `Run ${shortName}`;
  const resultBox = document.createElement("div");
  resultBox.className = "modal-result module-status";
  const footer = document.createElement("div");
  footer.className = "module-footer";
  footer.append(runBtn, resultBox);
  content.appendChild(footer);

  // --- Persistent-pane refresh: keep the pickers current without touching user choices.
  // Data changes (imports, module runs — including this pane's own) refresh the well list
  // and curve dropdowns in place; selecting another well only updates the pre-tick when
  // the checklist is empty and the set-name suggestions.
  let refreshGen = 0;
  const refreshData = async () => {
    // Race guard: a slower refresh resolving after a fresher one must not overwrite it with a
    // stale catalog (dataVersion can bump several times in quick succession).
    const gen = ++refreshGen;
    try {
      const freshCatalog = await listCurveCatalog();
      if (disposed || gen !== refreshGen) return;
      catalog = freshCatalog;
      curveNames = catalog.map((c) => c.name);
      for (const [name, select] of logSelects) {
        const arg = spec.args.find((a) => a.name === name)!;
        // Preserve the current selection, including a deliberate "(none)" ("") on an optional arg.
        fillLogSelect(select, arg, select.value);
      }
      rebuildMaskOptions(maskSelect.value);
      void refreshValidityConditions();
      void refreshDespikeCeiling();
    } catch {
      // No backend / transient failure: keep the current form as-is.
    }
  };
  let dataPrimed = false;
  const unsubData = appState.dataVersion.subscribe(() => {
    if (!dataPrimed) {
      dataPrimed = true; // subscribe fires immediately; the initial fetch already ran
      return;
    }
    void refreshData();
  });
  // The log-set pickers can gain names as the active well changes; the well scope tracks live
  // state on its own, so nothing well-related to re-tick here.
  const unsubWell = appState.selectedWell.subscribe(() => setPicker.refresh());

  runBtn.addEventListener("click", async () => {
    const wellIds = scope.getWellIds();
    if (wellIds.length === 0) {
      resultBox.textContent = "No wells in scope — pick a group, pin/select wells, or choose All.";
      return;
    }

    // Validate numeric params against manifest ranges.
    const params: Record<string, number> = {};
    for (const [name, input] of paramInputs) {
      const arg = spec.args.find((a) => a.name === name)!;
      // A blank field on a no-default param is a real state, not a parse failure. Required →
      // refused BY NAME here, where the user is looking (the needWell.ts rule), rather than as N
      // per-well failures in the Processing panel. Optional → omitted, and the module reads the
      // absence as NaN. For Clip that means "no bound on this side"; for an unbounded optional
      // input such as BADHOLE bit size it means the corresponding criterion is unavailable.
      if (!input.value.trim()) {
        if (arg.required) {
          resultBox.textContent = `${name} must be set — ${arg.desc}`;
          input.focus();
          return;
        }
        continue;
      }
      const v = parseFloat(input.value);
      if (Number.isNaN(v) || (arg.min !== null && v < arg.min) || (arg.max !== null && v > arg.max)) {
        resultBox.textContent = `${name}: value must be between ${arg.min} and ${arg.max}.`;
        input.focus();
        return;
      }
      params[name] = v;
    }
    let custody;
    try {
      custody = custodyControls.collect();
    } catch (error) {
      resultBox.textContent = String(error);
      return;
    }
    const req: RunModuleRequest = {
      module: spec.name,
      well_ids: wellIds,
      log_inputs: collectLogInputs(),
      params,
      opts: collectOpts(),
      output_set: setPicker.outputSet(),
      input_set: setPicker.inputSet(),
      custody,
    };
    runBtn.disabled = true;
    // Live progress and the per-well ✓/⚠/✗ breakdown now live in the Processing panel (this
    // run reports into the shared job registry). Surface that panel and keep only a one-line
    // outcome here, so the form isn't a second, redundant results log.
    resultBox.className = "modal-result module-status";
    resultBox.textContent = `Running ${spec.name} on ${wellIds.length} well(s)… see the Processing panel for progress.`;
    window.dispatchEvent(new Event("sandibumi:open-processing"));
    try {
      const results = await runWorkflowModule(req, scope.backend());
      const clean = results.filter((r) => r.outcome === "clean").length;
      const degraded = results.filter((r) => r.outcome === "degraded").length;
      const failed = results.filter((r) => r.outcome === "failed").length;
      const skipped = results.filter((r) => r.outcome === "skipped").length;
      const computed = clean + degraded;
      resultBox.textContent = `${clean} clean · ${degraded} degraded · ${failed} failed${
        skipped ? ` · ${skipped} skipped` : ""
      }. Open Processing → details for the per-well report.`;
      callbacks.setStatus(
        `${spec.name}: ${clean} clean, ${degraded} degraded, ${failed} failed`,
      );
      // The names the run actually WROTE, not the manifest's declared ones. Those two used to
      // differ silently for any module that built its own name, so the History panel recorded
      // curves the project did not hold — and now that a name can be renamed they differ by
      // design. The results carry the truth; a well that failed carries none, hence the union.
      const written = [...new Set(results.flatMap((r) => r.output_curves))];
      if (computed > 0) callbacks.onRunComplete(written, scope.namesFor(wellIds));
    } catch (err) {
      resultBox.textContent = `Run failed: ${err}`;
    } finally {
      runBtn.disabled = false;
    }
  });

  return {
    el: content,
    dispose: () => {
      disposed = true;
      if (ceilingTimer !== undefined) window.clearTimeout(ceilingTimer);
      unsubData();
      unsubWell();
      scope.dispose();
    },
  };
}
