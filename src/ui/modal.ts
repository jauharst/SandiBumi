/** Minimal dialog helper: renders `content` in a centered, draggable dialog that does
 *  NOT block the rest of the app — the scrim is pointer-transparent (see styles.css),
 *  so the user can keep clicking panels/ribbon while the dialog stays open.
 *  Returns a close function; Escape (scoped to this dialog) and the ✕ button close. */
let activeClose: (() => void) | null = null;
let dialogSeq = 0;
let rowSeq = 0;

export function openModal(
  title: string,
  content: HTMLElement,
  widthPx = 520,
  onClose?: () => void,
): () => void {
  const root = document.querySelector<HTMLElement>("#modal-root");
  if (!root) return () => {};

  // Only one dialog exists at a time (single #modal-root): close the previous one
  // properly so its document-level keydown listener doesn't leak.
  activeClose?.();

  root.hidden = false;
  root.innerHTML = "";

  const scrim = document.createElement("div");
  scrim.className = "modal-scrim";

  // Where focus should go back to when this closes. Captured BEFORE the previous dialog
  // is torn down, so it is the element that asked for THIS dialog rather than whatever
  // that teardown restored.
  const returnFocus = document.activeElement as HTMLElement | null;

  const dialog = document.createElement("div");
  dialog.className = "modal-dialog";
  dialog.style.width = `${widthPx}px`;
  // Announced as a dialog, named by its own title bar.
  //
  // Deliberately NOT `aria-modal`. These dialogs are non-blocking by design - the scrim
  // is pointer-transparent and the user keeps working in the panels behind them - and
  // `aria-modal="true"` asserts the rest of the application is inert. It would hide the
  // whole workspace from a screen reader while everyone else carried on using it, which
  // is a worse failure than the one it would fix.
  dialog.setAttribute("role", "dialog");
  dialog.tabIndex = -1;
  const titleId = `modal-title-${(dialogSeq += 1)}`;
  dialog.setAttribute("aria-labelledby", titleId);

  const head = document.createElement("div");
  head.className = "modal-head";
  const h = document.createElement("h3");
  h.id = titleId;
  h.textContent = title;
  const closeBtn = document.createElement("button");
  closeBtn.className = "modal-close";
  // The glyph IS the accessible name without this, and a screen reader reads a
  // multiplication sign or nothing at all. The name is translated like any other
  // visible string - i18n covers aria-label.
  closeBtn.setAttribute("aria-label", "Close");
  closeBtn.textContent = "✕";
  head.appendChild(h);
  head.appendChild(closeBtn);

  const body = document.createElement("div");
  body.className = "modal-body";
  body.appendChild(content);

  dialog.appendChild(head);
  dialog.appendChild(body);
  scrim.appendChild(dialog);
  root.appendChild(scrim);

  // Tracks the teardown for an in-flight title-bar drag so close() can run it too.
  let dragCleanup: (() => void) | null = null;

  // Drag-to-move: grab the title bar anywhere except the close button. The dialog is
  // centered by the scrim's flex layout until the first drag, then pinned via left/top.
  head.addEventListener("pointerdown", (e) => {
    if (e.target === closeBtn) return;
    e.preventDefault();
    const rect = dialog.getBoundingClientRect();
    const offsetX = e.clientX - rect.left;
    const offsetY = e.clientY - rect.top;
    scrim.style.justifyContent = "flex-start";
    scrim.style.alignItems = "flex-start";
    dialog.style.position = "absolute";
    const place = (x: number, y: number) => {
      const maxX = window.innerWidth - 60;
      const maxY = window.innerHeight - 40;
      dialog.style.left = `${Math.min(Math.max(x - offsetX, -rect.width + 80), maxX)}px`;
      dialog.style.top = `${Math.min(Math.max(y - offsetY, 0), maxY)}px`;
    };
    place(e.clientX, e.clientY);
    const onMove = (ev: PointerEvent) => place(ev.clientX, ev.clientY);
    const onUp = () => {
      document.removeEventListener("pointermove", onMove);
      document.removeEventListener("pointerup", onUp);
      dragCleanup = null;
    };
    document.addEventListener("pointermove", onMove);
    document.addEventListener("pointerup", onUp);
    // If the dialog is closed mid-drag (Escape / ✕ while the pointer is still down),
    // close() runs this so the document-level drag listeners never outlive the dialog.
    dragCleanup = onUp;
  });

  const close = () => {
    if (root.hidden) return;
    root.hidden = true;
    root.innerHTML = "";
    document.removeEventListener("keydown", onKey);
    dragCleanup?.(); // detach any in-flight title-bar drag listeners
    if (activeClose === close) activeClose = null;
    // Hand focus back to whatever opened this. Without it focus is left on a node that
    // has just been removed, which drops it to the body - so the next Tab restarts from
    // the top of the application rather than from the button the user was on.
    if (returnFocus?.isConnected) returnFocus.focus();
    onClose?.();
  };
  const onKey = (e: KeyboardEvent) => {
    if (e.key !== "Escape") return;
    // Escape belongs to the top dialog only. Stop it here so it doesn't also reach
    // window/app-level Escape handlers — e.g. cancelling an in-progress map polygon
    // (mapPanel) while a dialog is open. Keep this on the BUBBLE phase, NOT capture: the
    // numeric-edit guard (interactionGuard.ts) stops Escape in capture phase while a
    // number field is being edited, which must shield the dialog from closing — a capture
    // listener here would defeat that and close the dialog mid-edit.
    e.stopPropagation();
    close();
  };
  closeBtn.addEventListener("click", close);
  document.addEventListener("keydown", onKey);
  activeClose = close;
  // Focus the dialog itself, not its first field: a screen reader then reads the title
  // and the word "dialog" before anything else, and the user tabs in from there. Without
  // this, focus stays on the ribbon button behind and nothing says a dialog appeared.
  // A caller that wants a particular field focused still wins - it runs after this.
  dialog.focus();
  return close;
}

/** The controls a label can name. Deliberately not `button`: a button already carries
 *  its own name from its text, and pointing a label at one would make the label text
 *  CLICK it - so "Core file" beside a Browse button would open a file dialog. */
const LABELABLE = "input:not([type=hidden]), select, textarea";

/** The control a row's label should name: the element itself when it is one, otherwise
 *  the first one inside it, because several callers pass a wrapper holding the real
 *  input. A row that holds no control at all is left alone. */
function labelableIn(el: HTMLElement): HTMLElement | null {
  return el.matches(LABELABLE) ? el : el.querySelector<HTMLElement>(LABELABLE);
}

/** A labeled form row: label text on the left, the control on the right.
 *
 *  The row is a two-column grid, so the label is a SIBLING of its control rather than
 *  its parent - and with no `for` there was nothing for the browser to associate them
 *  with. Measured on the running app: a select built this way resolved to no accessible
 *  name at all, so a screen reader announced "combo box" and the visible word beside it
 *  was never read. That is every control in every dialog in the application.
 *
 *  Fixed here rather than at the 333 call sites, for the `requireWell` reason: one place
 *  to get right instead of 333 chances to forget. */
export function formRow(label: string, control: HTMLElement, hint?: string): HTMLElement {
  const row = document.createElement("div");
  row.className = "form-row";
  const lab = document.createElement("label");
  lab.className = "form-label";
  lab.textContent = label;
  if (hint) lab.title = hint;
  const target = labelableIn(control);
  if (target) {
    if (!target.id) target.id = `form-row-${(rowSeq += 1)}`;
    lab.htmlFor = target.id;
  }
  row.appendChild(lab);
  row.appendChild(control);
  return row;
}
