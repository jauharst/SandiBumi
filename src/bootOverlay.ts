import { bootReport } from "./ipc";

/** The boot overlay: what the user looks at while the project database opens.
 *
 *  Before this existed the project was opened BEFORE the window was created, so a slow first
 *  open (one-time storage upgrades on a field-scale project — ~15 minutes on a 2.5 GB one)
 *  showed nothing at all: you double-clicked SandiBumi and your machine appeared to ignore
 *  you. The window now comes up immediately and this covers the empty workspace until the
 *  database is live, showing the elapsed time and whatever the backend reports it is doing.
 */
export interface BootOverlay {
  /** Notices drained from the backend while waiting — handed back so the caller can record
   *  them in the processing history once the database (which stores it) is actually open. */
  readonly notes: string[];
  /** Tears the overlay down. Safe to call when it was never shown. */
  finish(): void;
}

/** Shown only if the open is slow enough to notice — a fast open must not flash a splash. */
const SHOW_AFTER_MS = 400;
const POLL_MS = 1000;

function fmtElapsed(sec: number): string {
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return m > 0 ? `${m}m ${String(s).padStart(2, "0")}s` : `${s}s`;
}

export function showBootOverlay(): BootOverlay {
  const notes: string[] = [];
  const started = Date.now();
  let el: HTMLElement | null = null;
  let msgEl: HTMLElement | null = null;
  let timeEl: HTMLElement | null = null;
  let hintEl: HTMLElement | null = null;

  const build = () => {
    el = document.createElement("div");
    el.className = "boot-overlay";
    // Organic design 1g: the launch surface is the identity column — logo,
    // display-face wordmark, one-line description — with the open progress
    // underneath. All static markup; every dynamic value goes in via textContent.
    el.innerHTML = `
      <div class="boot-card">
        <img class="boot-logo" src="/logo-mark.svg" alt="" width="72" height="72" />
        <div class="boot-title">SandiBumi</div>
        <div class="boot-desc">Multi-well petrophysical log analysis</div>
        <div class="boot-msg">Opening project…</div>
        <div class="boot-bar"><span></span></div>
        <div class="boot-time">0s</div>
        <div class="boot-hint"></div>
      </div>`;
    document.body.appendChild(el);
    msgEl = el.querySelector(".boot-msg");
    timeEl = el.querySelector(".boot-time");
    hintEl = el.querySelector(".boot-hint");
  };

  const showTimer = window.setTimeout(build, SHOW_AFTER_MS);

  const tick = window.setInterval(() => {
    const sec = Math.round((Date.now() - started) / 1000);
    if (timeEl) timeEl.textContent = fmtElapsed(sec);
    // After a while, say plainly that a long wait is expected and one-off rather than
    // leaving the user to guess whether it has hung.
    if (hintEl && sec >= 20 && !hintEl.textContent) {
      hintEl.textContent =
        "A first open after an update upgrades the project's storage and backs it up first. " +
        "This happens once — the next open is fast.";
    }
    // The backend queues one-time notices (migration backups, the memory cap). Draining them
    // here is why they are collected: the processing history needs a database to write to,
    // and there isn't one yet.
    void bootReport()
      .then((fresh) => {
        for (const n of fresh) {
          notes.push(n);
          if (msgEl) msgEl.textContent = n;
        }
      })
      .catch(() => {});
  }, POLL_MS);

  return {
    notes,
    finish() {
      window.clearTimeout(showTimer);
      window.clearInterval(tick);
      el?.remove();
      el = null;
    },
  };
}
