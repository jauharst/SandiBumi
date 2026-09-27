import { installationSupport, type InstallationSupport } from "../ipc";
import { openModal } from "./modal";

/** Product-wide runtime prerequisites. The backend supplies both the rows and current status
 * from the bundled capability manifest; this surface owns no package-name copy of its own.
 *
 * Check again asks the session Python afresh: the backend forgets every package answer it had
 * cached before probing, so a package installed with the command a refusal printed shows up here,
 * in the equation editor and in the ML notes without a restart. It never picks a new interpreter. */
export async function openInstallationSupportDialog(): Promise<void> {
  const content = document.createElement("div");
  content.className = "mc-dialog";
  const close = openModal("Capability prerequisites", content, 680);

  const body = document.createElement("div");
  const actions = document.createElement("div");
  actions.className = "form-actions";
  const again = document.createElement("button");
  again.className = "btn";
  again.textContent = "Check again";
  again.title = "Ask the session Python again, after installing a package";
  const done = document.createElement("button");
  done.className = "btn btn-accent";
  done.textContent = "Close";
  done.addEventListener("click", close);
  actions.append(again, done);
  content.append(body, actions);

  const hint = (text: string): HTMLParagraphElement => {
    const p = document.createElement("p");
    p.className = "form-hint";
    p.textContent = text;
    return p;
  };

  async function check(): Promise<void> {
    again.disabled = true;
    body.replaceChildren(hint("Checking the session runtime…"));
    try {
      render(await installationSupport());
    } catch (error) {
      body.replaceChildren(hint(`Prerequisite check failed: ${String(error)}`));
    } finally {
      again.disabled = false;
    }
  }
  again.addEventListener("click", () => void check());

  function render(support: InstallationSupport): void {
    body.replaceChildren();

    const summary = document.createElement("p");
    summary.textContent = support.selected_interpreter
      ? `Session Python: ${support.selected_interpreter} · selected by ${support.selected_interpreter_rule ?? "the recorded resolver rule"}`
      : `No session Python ${support.interpreter_minimum_version}+ interpreter is available. Native project, plotting and export paths remain available.`;
    body.appendChild(summary);
    if (!support.selected_interpreter) {
      body.appendChild(
        hint("SandiBumi chooses its Python once, when it starts: a Python installed since is found at the next start, not by Check again."),
      );
    }

    const attempted = document.createElement("section");
    attempted.className = "form-section";
    const attemptedHeading = document.createElement("h4");
    attemptedHeading.textContent = "Interpreter resolution";
    attempted.appendChild(attemptedHeading);
    for (const candidate of support.interpreter_candidates) {
      const resolved =
        candidate.resolved_executable && candidate.resolved_executable !== candidate.candidate
          ? ` → ${candidate.resolved_executable}`
          : "";
      attempted.appendChild(
        hint(`${candidate.accepted ? "Selected" : "Rejected"} · ${candidate.precedence_rule}: ${candidate.candidate}${resolved} · ${candidate.reason}`),
      );
    }
    body.appendChild(attempted);

    for (const capability of support.capabilities) {
      const row = document.createElement("section");
      row.className = "form-section";
      const heading = document.createElement("h4");
      const state = capability.available === true ? "Available" : capability.available === false ? "Unavailable" : "Probe required";
      heading.textContent = `${capability.display_name} — ${state}`;
      const packages = hint(
        capability.packages
          .map((pkg) => {
            const observed = capability.package_status.find(
              (status) => status.distribution.toLowerCase() === pkg.distribution.toLowerCase(),
            );
            const version = observed?.version ? ` ${observed.version}` : "";
            const state = observed ? (observed.available ? " ✓" : " ✕") : "";
            return `${pkg.distribution}${version}${pkg.required ? "" : " (optional)"}${state}`;
          })
          .join(", "),
      );
      row.append(heading, packages, hint(`${capability.reason} · owner ${capability.owning_domain}`));
      body.appendChild(row);
    }

    body.appendChild(
      hint(
        "Offline Python-backed capabilities are supported only through the signed, versioned SandiBumi-qualified pack. Exact package versions come from the release lock.",
      ),
    );
  }

  await check();
}
