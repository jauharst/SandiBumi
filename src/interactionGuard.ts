import { setStatus } from "./state";

/** App-wide interaction safety (Jauhar field review 2026-07-19, P1):
 *
 *  1. Right-click lockdown — the WebView's default context menu contains Refresh/Back,
 *     and a stray "Refresh" wipes the whole workspace. The native menu is killed
 *     everywhere except editable fields, whose native menu is the harmless edit menu
 *     (undo/cut/copy/paste — no navigation items). Custom app menus preventDefault
 *     before this handler runs, so they are unaffected.
 *
 *  2. Reload guard — F5 / Ctrl+R (and Ctrl+Shift+R, Ctrl+F5) ask for confirmation
 *     instead of instantly reloading; Alt+arrows and mouse back/forward buttons
 *     (history navigation, which would leave the app entirely) are blocked outright.
 *
 *  3. Click-to-arm, double-click-to-edit numeric fields — a single click on any
 *     `input[type=number]` only focuses ("arms") it read-only; the caret and typing
 *     require a double click. Prevents a stray click + key/wheel from silently
 *     changing a petrophysical parameter. Keyboard (Tab) focus is deliberate and
 *     stays editable. Opt out per input with the `data-free-edit` attribute.
 */
export function installInteractionGuards(): void {
  installContextMenuLockdown();
  installReloadGuards();
  installNumericEditGuard();
}

/** Elements whose native context menu is the edit menu (no Refresh) — leave those alone. */
const EDITABLE_SELECTOR = "input, textarea, [contenteditable], .cm-editor";

function installContextMenuLockdown(): void {
  document.addEventListener("contextmenu", (e) => {
    const target = e.target as HTMLElement | null;
    if (target?.closest?.(EDITABLE_SELECTOR)) return;
    e.preventDefault();
  });
}

function installReloadGuards(): void {
  window.addEventListener(
    "keydown",
    (e) => {
      const reload = e.key === "F5" || ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "r");
      if (reload) {
        e.preventDefault();
        e.stopPropagation();
        confirmReload();
        return;
      }
      if (
        (e.altKey && (e.key === "ArrowLeft" || e.key === "ArrowRight")) ||
        e.key === "BrowserBack" ||
        e.key === "BrowserForward"
      ) {
        e.preventDefault();
      }
    },
    { capture: true },
  );
  // Mouse side buttons also trigger history navigation in the WebView.
  for (const type of ["mousedown", "mouseup", "auxclick"] as const) {
    document.addEventListener(type, (e) => {
      if (e.button === 3 || e.button === 4) e.preventDefault();
    });
  }
}

/** Blocking confirm shown on a reload attempt. Deliberately NOT the shared openModal —
 *  that scrim is pointer-transparent by design; a destructive confirm must block. */
function confirmReload(): void {
  if (document.querySelector(".guard-confirm-scrim")) return;
  const scrim = document.createElement("div");
  scrim.className = "guard-confirm-scrim";
  const box = document.createElement("div");
  box.className = "guard-confirm";
  const msg = document.createElement("p");
  msg.textContent =
    "Reload SandiBumi? The workspace re-opens from its last saved state — unsaved picks, layouts and dialog inputs are lost.";
  const row = document.createElement("div");
  row.className = "guard-confirm-row";
  const cancel = document.createElement("button");
  cancel.type = "button";
  cancel.textContent = "Cancel";
  const ok = document.createElement("button");
  ok.type = "button";
  ok.className = "danger";
  ok.textContent = "Reload";
  const close = () => scrim.remove();
  cancel.addEventListener("click", close);
  ok.addEventListener("click", () => window.location.reload());
  scrim.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      e.stopPropagation();
      close();
    }
  });
  row.append(cancel, ok);
  box.append(msg, row);
  scrim.appendChild(box);
  document.body.appendChild(scrim);
  cancel.focus();
}

function installNumericEditGuard(): void {
  let tipShown = false;

  const asGuarded = (t: EventTarget | null): HTMLInputElement | null =>
    t instanceof HTMLInputElement && t.type === "number" && !t.disabled && !t.hasAttribute("data-free-edit")
      ? t
      : null;

  const unlock = (input: HTMLInputElement): void => {
    input.readOnly = false;
    delete input.dataset.dcArmed;
    input.dataset.dcEditing = "1";
    input.classList.remove("num-armed");
    input.classList.add("num-editing");
  };

  document.addEventListener(
    "pointerdown",
    (e) => {
      if (e.button !== 0) return;
      const input = asGuarded(e.target);
      if (!input) return;
      if (input.dataset.dcEditing) return; // already unlocked by double click
      // readOnly set by app code (not by us) is a deliberate state — don't touch it.
      if (input.readOnly && !input.dataset.dcArmed) return;
      if (!input.dataset.dcArmed) {
        input.readOnly = true;
        input.dataset.dcArmed = "1";
        input.classList.add("num-armed");
        if (!input.title) input.title = "Double-click to edit";
        if (!tipShown) {
          tipShown = true;
          setStatus("Number fields arm on click — double-click to edit");
        }
      }
    },
    { capture: true },
  );

  document.addEventListener("dblclick", (e) => {
    const input = asGuarded(e.target);
    if (!input || !input.dataset.dcArmed) return;
    unlock(input);
    input.select();
  });

  // Right-clicking an armed field means the user wants the edit menu (paste etc.) —
  // treat it as deliberate edit intent so paste isn't silently blocked by readOnly.
  document.addEventListener(
    "contextmenu",
    (e) => {
      const input = asGuarded(e.target);
      if (input?.dataset.dcArmed) unlock(input);
    },
    { capture: true },
  );

  document.addEventListener(
    "focusout",
    (e) => {
      const input = asGuarded(e.target);
      if (!input) return;
      if (input.dataset.dcArmed) input.readOnly = false;
      delete input.dataset.dcArmed;
      delete input.dataset.dcEditing;
      input.classList.remove("num-armed", "num-editing");
    },
    { capture: true },
  );

  // While editing, Enter commits (change fires on blur) and Escape just exits edit
  // mode — stopPropagation so Escape doesn't also close a surrounding dialog.
  document.addEventListener(
    "keydown",
    (e) => {
      const input = asGuarded(e.target);
      if (!input || !input.dataset.dcEditing) return;
      if (e.key === "Enter" || e.key === "Escape") {
        if (e.key === "Escape") e.stopPropagation();
        input.blur();
      }
    },
    { capture: true },
  );

  // A focused number input spins on wheel in Chromium — a scroll gesture over the
  // field would silently change the value. Block the spin (Ctrl-free wheel over an
  // unfocused field scrolls the panel as usual because the input isn't the target).
  document.addEventListener(
    "wheel",
    (e) => {
      const input = asGuarded(e.target);
      if (input && document.activeElement === input) e.preventDefault();
    },
    { capture: true, passive: false },
  );
}
