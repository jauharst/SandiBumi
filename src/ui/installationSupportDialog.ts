import { installationSupport } from "../ipc";
import { openModal } from "./modal";

/** Product-wide runtime prerequisites. The backend supplies both the rows and current status
 * from the bundled capability manifest; this surface owns no package-name copy of its own. */
export async function openInstallationSupportDialog(): Promise<void> {
  const content = document.createElement("div");
  content.className = "mc-dialog";
  const close = openModal("Capability prerequisites", content, 680);

  const loading = document.createElement("p");
  loading.className = "form-hint";
  loading.textContent = "Checking the session runtime…";
  content.appendChild(loading);

  try {
    const support = await installationSupport();
    content.replaceChildren();

    const summary = document.createElement("p");
    summary.textContent = support.selected_interpreter
      ? `Session Python: ${support.selected_interpreter} · selected by ${support.selected_interpreter_rule ?? "the recorded resolver rule"}`
      : `No session Python ${support.interpreter_minimum_version}+ interpreter is available. Native project, plotting and export paths remain available.`;
    content.appendChild(summary);

    const attempted = document.createElement("section");
    attempted.className = "form-section";
    const attemptedHeading = document.createElement("h4");
    attemptedHeading.textContent = "Interpreter resolution";
    attempted.appendChild(attemptedHeading);
    for (const candidate of support.interpreter_candidates) {
      const row = document.createElement("p");
      row.className = "form-hint";
      const resolved =
        candidate.resolved_executable && candidate.resolved_executable !== candidate.candidate
          ? ` → ${candidate.resolved_executable}`
          : "";
      row.textContent = `${candidate.accepted ? "Selected" : "Rejected"} · ${candidate.precedence_rule}: ${candidate.candidate}${resolved} · ${candidate.reason}`;
      attempted.appendChild(row);
    }
    content.appendChild(attempted);

    for (const capability of support.capabilities) {
      const row = document.createElement("section");
      row.className = "form-section";
      const heading = document.createElement("h4");
      const state = capability.available === true ? "Available" : capability.available === false ? "Unavailable" : "Probe required";
      heading.textContent = `${capability.display_name} — ${state}`;
      const packages = document.createElement("p");
      packages.className = "form-hint";
      packages.textContent = capability.packages
        .map((pkg) => {
          const observed = capability.package_status.find(
            (status) => status.distribution.toLowerCase() === pkg.distribution.toLowerCase(),
          );
          const version = observed?.version ? ` ${observed.version}` : "";
          const state = observed ? (observed.available ? " ✓" : " ✕") : "";
          return `${pkg.distribution}${version}${pkg.required ? "" : " (optional)"}${state}`;
        })
        .join(", ");
      const reason = document.createElement("p");
      reason.className = "form-hint";
      reason.textContent = `${capability.reason} · owner ${capability.owning_domain}`;
      row.append(heading, packages, reason);
      content.appendChild(row);
    }

    const note = document.createElement("p");
    note.className = "form-hint";
    note.textContent =
      "Offline Python-backed capabilities are supported only through the signed, versioned SandiBumi-qualified pack. Exact package versions come from the release lock.";
    content.appendChild(note);

    const actions = document.createElement("div");
    actions.className = "form-actions";
    const done = document.createElement("button");
    done.className = "btn btn-accent";
    done.textContent = "Close";
    done.addEventListener("click", close);
    actions.appendChild(done);
    content.appendChild(actions);
  } catch (error) {
    loading.textContent = `Prerequisite check failed: ${String(error)}`;
  }
}
