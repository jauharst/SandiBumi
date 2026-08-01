import { currentProject, listRecentProjects, type RecentProject } from "../ipc";

/** The start sheet (Organic design 1g), shown on the blank-canvas placeholder — the
 *  surface a user sees when no content pane is open. Left column: identity and the two
 *  project actions; right sheet: recent projects and a tip about sessions.
 *
 *  Deliberately thin on wiring: New/Open click the EXISTING ribbon tools (the id-based
 *  buttons every handler already hangs off), and a recent row dispatches
 *  `sandibumi:open-recent-project`, which the ribbon resolves through the same
 *  switchProject guard the Recent ▾ menu uses — so a busy chain blocks a switch here
 *  exactly as it does there. Project names/paths are user data: textContent only, and
 *  `data-no-i18n` so the translator never keys on them. */
export function buildStartSheet(host: HTMLElement): void {
  host.innerHTML = "";
  const sheet = document.createElement("div");
  sheet.className = "start-sheet";

  // --- Left column: identity + actions -------------------------------------
  const side = document.createElement("div");
  side.className = "start-side";
  const logo = document.createElement("img");
  logo.className = "start-logo";
  logo.src = "/logo-mark.svg";
  logo.alt = "";
  logo.width = 72;
  logo.height = 72;
  const word = document.createElement("div");
  word.className = "start-wordmark";
  word.textContent = "SandiBumi";
  const desc = document.createElement("div");
  desc.className = "start-desc";
  desc.textContent = "Multi-well petrophysical log analysis";
  const newBtn = document.createElement("button");
  newBtn.type = "button";
  newBtn.className = "btn btn-accent start-action";
  newBtn.textContent = "New Project";
  newBtn.addEventListener("click", () => document.getElementById("new-project-btn")?.click());
  const openBtn = document.createElement("button");
  openBtn.type = "button";
  openBtn.className = "btn start-action";
  openBtn.textContent = "Open Project";
  openBtn.addEventListener("click", () => document.getElementById("open-project-btn")?.click());
  side.append(logo, word, desc, newBtn, openBtn);
  sheet.appendChild(side);

  // --- Right sheet: recent projects + sessions tip -------------------------
  const recentsBox = document.createElement("div");
  recentsBox.className = "start-recents";
  const heading = document.createElement("div");
  heading.className = "start-heading";
  heading.textContent = "Recent projects";
  recentsBox.appendChild(heading);
  const list = document.createElement("div");
  list.className = "start-recent-list";
  recentsBox.appendChild(list);

  const tip = document.createElement("div");
  tip.className = "start-tip";
  tip.textContent =
    "Sessions save your workspace arrangement — which panes and plots are open, and the " +
    "active well. Save one from Project ▸ Save Session…, reopen it from Open Session.";
  recentsBox.appendChild(tip);
  sheet.appendChild(recentsBox);
  host.appendChild(sheet);

  // Recents load after the shell paints; a vite-only preview (no backend) just
  // shows the empty note.
  void (async () => {
    const [recents, current] = await Promise.all([
      listRecentProjects().catch(() => [] as RecentProject[]),
      currentProject().catch(() => null),
    ]);
    list.innerHTML = "";
    if (recents.length === 0) {
      const empty = document.createElement("div");
      empty.className = "start-empty";
      empty.textContent = "No recent projects yet.";
      list.appendChild(empty);
      return;
    }
    for (const r of recents.slice(0, 8)) {
      const isCurrent = current !== null && r.path === current.path;
      const row = document.createElement("button");
      row.type = "button";
      row.className = "start-recent-row";
      row.setAttribute("data-no-i18n", "");
      row.disabled = !r.exists || isCurrent;
      row.title = r.path;
      const chip = document.createElement("span");
      chip.className = "start-db-chip";
      chip.innerHTML =
        `<svg viewBox="0 0 20 20" width="18" height="18" fill="none" stroke="currentColor" ` +
        `stroke-width="1.8" stroke-linecap="round"><ellipse cx="10" cy="5" rx="6" ry="2.4"/>` +
        `<path d="M4 5v10c0 1.3 2.7 2.4 6 2.4s6-1.1 6-2.4V5"/><path d="M4 10c0 1.3 2.7 2.4 6 2.4s6-1.1 6-2.4"/></svg>`;
      const meta = document.createElement("span");
      meta.className = "start-recent-meta";
      const name = document.createElement("span");
      name.className = "start-recent-name";
      name.textContent = r.name + (r.exists ? "" : "  (missing)");
      const sub = document.createElement("span");
      sub.className = "start-recent-sub";
      sub.textContent = r.last_opened > 0 ? new Date(r.last_opened * 1000).toLocaleDateString() : "";
      meta.append(name, sub);
      row.append(chip, meta);
      if (isCurrent) {
        const tag = document.createElement("span");
        tag.className = "start-recent-tag";
        tag.textContent = "open now";
        row.appendChild(tag);
      }
      row.addEventListener("click", () => {
        window.dispatchEvent(new CustomEvent("sandibumi:open-recent-project", { detail: r.path }));
      });
      list.appendChild(row);
    }
  })();
}
