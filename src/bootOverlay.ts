import { bootReport } from "./ipc";
import pkg from "../package.json";

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

/** Shown only if the open is slow enough to notice — a fast open must not flash a splash.
 *
 *  **This never delays the launch** (Jauhar, 2026-08-01: *"dont make start longer, its filler while
 *  waiting"*). It fills a wait that already exists and comes down the instant the database is live;
 *  there is deliberately no minimum display time, which is what turns a splash from a courtesy into
 *  an obstacle. */
const SHOW_AFTER_MS = 400;
const POLL_MS = 1000;

/** The release line, read from the package rather than typed here, so a version bump cannot leave
 *  the launch screen claiming an older build. The YEAR is part of the product name the way IP 2018
 *  and Petrel 2024 are — the edition, not the build. */
const EDITION = "2026";
const VERSION = `SandiBumi ${EDITION}  ·  v${(pkg as { version: string }).version}`;

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
    // A portrait launch card, the shape every subsurface application uses: artwork, the mark, the
    // product name, the edition, the copyright. The artwork is drawn in the BRAND's own colours and
    // is inline SVG rather than a file — a launch screen that waits on a network or a missing asset
    // is the one screen that must never fail to appear.
    el.innerHTML = `
      <div class="boot-card">
        <div class="boot-art" aria-hidden="true">
          <svg viewBox="0 0 320 240" preserveAspectRatio="xMidYMid slice">
            <defs>
              <linearGradient id="bg-strata" x1="0" y1="0" x2="0.35" y2="1">
                <stop offset="0" stop-color="#f5ead8"/><stop offset="1" stop-color="#eadcc2"/>
              </linearGradient>
            </defs>
            <rect width="320" height="240" fill="url(#bg-strata)"/>
            <g fill="none" stroke-linecap="round">
              <path d="M-10 62 Q 80 30 160 58 T 330 44" stroke="#c67139" stroke-width="26" opacity=".85"/>
              <path d="M-10 104 Q 90 74 165 100 T 330 86" stroke="#7a8a5e" stroke-width="18" opacity=".8"/>
              <path d="M-10 138 Q 70 116 158 140 T 330 126" stroke="#c67139" stroke-width="10" opacity=".55"/>
              <path d="M-10 168 Q 100 148 170 172 T 330 158" stroke="#7a8a5e" stroke-width="22" opacity=".55"/>
              <path d="M-10 206 Q 80 186 160 208 T 330 196" stroke="#c67139" stroke-width="16" opacity=".35"/>
            </g>
            <circle cx="228" cy="92" r="7" fill="none" stroke="#3f3428" stroke-width="2.5" opacity=".7"/>
          </svg>
        </div>
        <div class="boot-body">
          <img class="boot-logo" src="/logo-mark.svg" alt="" width="52" height="52" />
          <div class="boot-title" data-no-i18n>SandiBumi</div>
          <div class="boot-desc">Multi-well petrophysical log analysis</div>
          <div class="boot-msg">Opening project…</div>
          <div class="boot-bar"><span></span></div>
          <div class="boot-time">0s</div>
          <div class="boot-hint"></div>
          <div class="boot-foot">
            <div class="boot-version" data-no-i18n></div>
            <div class="boot-copy" data-no-i18n>© 2026 SandiBumi. All rights reserved.</div>
          </div>
        </div>
      </div>`;
    document.body.appendChild(el);
    const ver = el.querySelector(".boot-version");
    if (ver) ver.textContent = VERSION;
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
