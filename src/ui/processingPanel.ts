import { cancelJob, listJobs, type JobItem, type JobView } from "../ipc";

/** Universal Processing panel (Phase 11): one dock pane showing live progress, the well
 *  being processed, per-well ✓/⚠/✗ outcomes, and a Cancel button — for EVERY long
 *  operation that reports into the shared job registry (workflow chains today; module runs,
 *  imports, SandiMin, Monte Carlo, reports as each is moved off the IPC thread).
 *
 *  It polls `list_jobs` on a timer (same registry-and-poll model as the Workflow Builder's
 *  own bar). The Cancel button here shares the SAME cancel flag as the run, so cancelling
 *  from this panel stops the job whether it was launched here or from its own dialog.
 *
 *  At field scale (hundreds of wells) the per-well list would be huge, so a job shows a
 *  compact counts row always, and — when expanded — only the NOTABLE wells (running now,
 *  warned, or failed). That directly answers "which wells failed or warned?" without
 *  rendering 500 green rows twice a second. */
export async function buildProcessingContent(
  _setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const el = document.createElement("div");
  el.className = "proc-panel";

  const list = document.createElement("div");
  list.className = "proc-list";
  el.appendChild(list);

  // Which job cards the user has expanded (by id) survives re-renders.
  const expanded = new Set<string>();
  let lastSig = "";

  const ICON: Record<JobItem["state"], string> = {
    pending: "⏳",
    running: "▶",
    ok: "✓",
    warned: "⚠",
    failed: "✗",
  };
  const PHASE_LABEL: Record<JobView["phase"], string> = {
    queued: "Queued",
    running: "Running",
    completed: "Done",
    cancelled: "Cancelled",
    failed: "Failed",
  };

  function counts(job: JobView): Record<JobItem["state"], number> {
    const c: Record<JobItem["state"], number> = { pending: 0, running: 0, ok: 0, warned: 0, failed: 0 };
    for (const it of job.items) c[it.state] += 1;
    return c;
  }

  // A cheap signature so we only rebuild the DOM when something actually changed (the poll
  // fires every 500ms; most ticks are identical once a run finishes).
  function signature(jobs: JobView[]): string {
    return jobs
      .map((j) => {
        const c = counts(j);
        return `${j.id}:${j.phase}:${j.outcome ?? "pending"}:${j.done}/${j.total}:${j.current ?? ""}:${c.running},${c.warned},${c.failed}`;
      })
      .join("|") + `#${[...expanded].sort().join(",")}`;
  }

  function chip(text: string, cls: string): HTMLElement {
    const s = document.createElement("span");
    s.className = `proc-count ${cls}`;
    s.textContent = text;
    return s;
  }

  /** One line of "what should the user do?" per failure/warning family, matched on the
   *  message text. Deliberately generic — the aim is to point Jauhar at the right next step,
   *  not diagnose every case. Falls back to a safe default when nothing matches. */
  function advice(message: string): string {
    const m = message.toLowerCase();
    if (/missing|not found|no curve|absent|unknown curve|no such/.test(m))
      return "Check these wells have the input curves (Curve Catalog), or pick different inputs.";
    if (/no data|no valid|empty|no samples|no output|all null|out of range|no overlap/.test(m))
      return "No usable samples in the chosen inputs/interval — verify the depth range and inputs for these wells.";
    if (/under-?determined|degrees of freedom|too few|need (more|at least)|insufficient|\brank\b/.test(m))
      return "Give the solver more input logs, or reduce the number of components it must resolve.";
    if (/converg|iteration|singular|\bnan\b|infinit|unstable|diverg/.test(m))
      return "The solve didn't converge — loosen bounds/constraints or review the input QC for these wells.";
    // \bread\b, not bare "read" — a bare token matches "already", "thread", "spread" and would
    // mis-route benign warnings (e.g. "VSH already exists") to file-import advice.
    if (/parse|format|column|header|\bread\b|decode|invalid file/.test(m))
      return "The source file couldn't be read — check its format/columns and re-import.";
    if (/lock|busy|in use|another/.test(m))
      return "The database was busy — let the current job finish, then re-run.";
    return "Open Processing History or the wells' Curve Catalog to investigate, then re-run.";
  }

  function truncWells(labels: string[], max = 12): string {
    if (labels.length <= max) return labels.join(", ");
    return `${labels.slice(0, max).join(", ")} … (+${labels.length - max} more)`;
  }

  function renderItemsBlock(job: JobView): HTMLElement {
    const box = document.createElement("div");
    box.className = "proc-items";
    const running = job.items.filter((it) => it.state === "running");
    const problems = job.items.filter((it) => it.state === "warned" || it.state === "failed");
    const c = counts(job);
    if (running.length === 0 && problems.length === 0) {
      const none = document.createElement("div");
      none.className = "proc-item proc-item-note";
      none.textContent = c.ok > 0 ? `All ${c.ok} well(s) OK.` : "No wells reported yet.";
      box.appendChild(none);
      return box;
    }

    // In-flight wells: show individually (there are only ever a few) so the user sees motion.
    for (const it of running.slice(0, 20)) {
      const row = document.createElement("div");
      row.className = `proc-item proc-item-${it.state}`;
      const icon = document.createElement("span");
      icon.className = "proc-item-icon";
      icon.textContent = ICON[it.state];
      const name = document.createElement("span");
      name.className = "proc-item-label";
      name.textContent = it.label;
      row.append(icon, name);
      box.appendChild(row);
    }

    // Failures/warnings collapse into ONE bulk row per distinct reason — at field scale the
    // same error hits hundreds of wells and a per-well list is noise. Each family reports the
    // reason, how many/which wells, and a suggested next step. Failures rank above warnings.
    const families = new Map<string, { state: JobItem["state"]; message: string; labels: string[] }>();
    for (const it of problems) {
      const message = it.message?.trim() || (it.state === "failed" ? "Failed (no reason reported)" : "Warning (no detail)");
      const key = `${it.state}\n${message}`;
      const fam = families.get(key) ?? { state: it.state, message, labels: [] };
      fam.labels.push(it.label);
      families.set(key, fam);
    }
    const ordered = [...families.values()].sort(
      (a, b) => (a.state === "failed" ? 0 : 1) - (b.state === "failed" ? 0 : 1) || b.labels.length - a.labels.length,
    );
    for (const fam of ordered) {
      const group = document.createElement("div");
      group.className = `proc-group proc-item-${fam.state}`;

      const head = document.createElement("div");
      head.className = "proc-item proc-group-head";
      const icon = document.createElement("span");
      icon.className = "proc-item-icon";
      icon.textContent = ICON[fam.state];
      const summary = document.createElement("span");
      summary.className = "proc-group-summary";
      const verb = fam.state === "failed" ? "failed" : "warned";
      summary.textContent = `${fam.labels.length} well(s) ${verb} — ${fam.message}`;
      head.append(icon, summary);
      group.appendChild(head);

      const wells = document.createElement("div");
      wells.className = "proc-group-wells";
      wells.textContent = truncWells(fam.labels);
      group.appendChild(wells);

      const tip = document.createElement("div");
      tip.className = "proc-group-advice";
      tip.textContent = `→ ${advice(fam.message)}`;
      group.appendChild(tip);

      box.appendChild(group);
    }
    return box;
  }

  function renderJob(job: JobView): HTMLElement {
    const active = job.phase === "running" || job.phase === "queued";
    const card = document.createElement("div");
    card.className = `proc-job proc-job-${job.phase}`;

    const head = document.createElement("div");
    head.className = "proc-job-head";
    const kind = document.createElement("span");
    kind.className = "proc-kind";
    kind.textContent = job.kind;
    const label = document.createElement("span");
    label.className = "proc-label";
    label.textContent = job.label;
    label.title = job.label;
    const phase = document.createElement("span");
    phase.className = `proc-phase proc-phase-${job.phase}${job.outcome ? ` proc-outcome-${job.outcome}` : ""}`;
    phase.textContent =
      job.phase === "completed" && job.outcome === "degraded"
        ? "Done with warnings"
        : job.phase === "completed" && job.outcome === "failed"
          ? "Done with failures"
          : PHASE_LABEL[job.phase];
    head.append(kind, label, phase);
    card.appendChild(head);

    // Progress bar + integrated Cancel (Cancel sits on the same row as the bar).
    const barRow = document.createElement("div");
    barRow.className = "proc-bar-row";
    const barWrap = document.createElement("div");
    barWrap.className = "proc-bar-wrap";
    const fill = document.createElement("div");
    fill.className = "proc-bar-fill";
    const pct = job.total > 0 ? Math.min(100, Math.round((job.done / job.total) * 100)) : job.phase === "completed" ? 100 : 0;
    fill.style.width = `${pct}%`;
    barWrap.appendChild(fill);
    const pctLabel = document.createElement("span");
    pctLabel.className = "proc-pct";
    pctLabel.textContent = `${pct}%`;
    barRow.append(barWrap, pctLabel);
    if (active && job.cancellable) {
      const cancel = document.createElement("button");
      cancel.className = "proc-cancel";
      cancel.type = "button";
      cancel.textContent = "Cancel";
      cancel.addEventListener("click", () => {
        cancel.disabled = true;
        cancel.textContent = "Cancelling…";
        void cancelJob(job.id).catch(() => {});
      });
      barRow.appendChild(cancel);
    } else if (active) {
      // No Cancel button on a job whose worker never observes the flag — offering one was a
      // control that did nothing (R3's visible half). Say so plainly instead of leaving a bare
      // bar that looks like a missing button.
      const tag = document.createElement("span");
      tag.className = "proc-uninterruptible";
      tag.textContent = "can't be interrupted";
      tag.title = "This operation runs in one step and cannot be stopped partway.";
      barRow.appendChild(tag);
    }
    card.appendChild(barRow);

    // "Step 2/3: sw_indo · 340/1000" line.
    const sub = document.createElement("div");
    sub.className = "proc-sub";
    const bits: string[] = [];
    if (job.current) bits.push(job.current);
    if (job.total > 0) bits.push(`${job.done}/${job.total}`);
    if (job.error) bits.push(job.error);
    sub.textContent = bits.join(" · ");
    if (bits.length) card.appendChild(sub);

    // Compact counts row (only non-zero states).
    const c = counts(job);
    const countsRow = document.createElement("div");
    countsRow.className = "proc-counts";
    if (c.running) countsRow.appendChild(chip(`▶ ${c.running}`, "proc-count-running"));
    if (c.ok) countsRow.appendChild(chip(`✓ ${c.ok}`, "proc-count-ok"));
    if (c.warned) countsRow.appendChild(chip(`⚠ ${c.warned}`, "proc-count-warned"));
    if (c.failed) countsRow.appendChild(chip(`✗ ${c.failed}`, "proc-count-failed"));
    if (c.pending) countsRow.appendChild(chip(`⏳ ${c.pending}`, "proc-count-pending"));

    const notableCount = c.running + c.warned + c.failed;
    if (notableCount > 0 || job.items.length > 0) {
      const toggle = document.createElement("button");
      toggle.className = "proc-toggle";
      toggle.type = "button";
      const isOpen = expanded.has(job.id);
      toggle.textContent = isOpen ? "▾ details" : "▸ details";
      toggle.addEventListener("click", () => {
        if (expanded.has(job.id)) expanded.delete(job.id);
        else expanded.add(job.id);
        void refresh(); // re-render immediately, don't wait for the next poll
      });
      countsRow.appendChild(toggle);
    }
    card.appendChild(countsRow);

    if (expanded.has(job.id)) card.appendChild(renderItemsBlock(job));
    return card;
  }

  async function refresh(): Promise<void> {
    const jobs = await listJobs().catch(() => [] as JobView[]);
    const sig = signature(jobs);
    if (sig === lastSig) return; // nothing changed — leave the DOM (and scroll) untouched
    lastSig = sig;
    // Drop expansion state for jobs that have been pruned away.
    const live = new Set(jobs.map((j) => j.id));
    for (const id of [...expanded]) if (!live.has(id)) expanded.delete(id);

    const scroll = list.scrollTop;
    list.innerHTML = "";
    if (jobs.length === 0) {
      const empty = document.createElement("div");
      empty.className = "proc-empty";
      empty.textContent = "No processing jobs yet. Run a workflow chain and its progress appears here.";
      list.appendChild(empty);
    } else {
      for (const job of jobs) list.appendChild(renderJob(job));
    }
    list.scrollTop = scroll;
  }

  await refresh();
  const timer = window.setInterval(() => void refresh(), 500);

  return {
    el,
    dispose: () => window.clearInterval(timer),
  };
}
