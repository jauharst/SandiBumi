/** Minimal dialog helper: renders `content` in a centered, draggable dialog that does
 *  NOT block the rest of the app — the scrim is pointer-transparent (see styles.css),
 *  so the user can keep clicking panels/ribbon while the dialog stays open.
 *  Returns a close function; Escape and the ✕ button close. */
let activeClose: (() => void) | null = null;

export function openModal(title: string, content: HTMLElement, widthPx = 520): () => void {
  const root = document.querySelector<HTMLElement>("#modal-root");
  if (!root) return () => {};

  // Only one dialog exists at a time (single #modal-root): close the previous one
  // properly so its document-level keydown listener doesn't leak.
  activeClose?.();

  root.hidden = false;
  root.innerHTML = "";

  const scrim = document.createElement("div");
  scrim.className = "modal-scrim";

  const dialog = document.createElement("div");
  dialog.className = "modal-dialog";
  dialog.style.width = `${widthPx}px`;

  const head = document.createElement("div");
  head.className = "modal-head";
  const h = document.createElement("h3");
  h.textContent = title;
  const closeBtn = document.createElement("button");
  closeBtn.className = "modal-close";
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
    };
    document.addEventListener("pointermove", onMove);
    document.addEventListener("pointerup", onUp);
  });

  const close = () => {
    root.hidden = true;
    root.innerHTML = "";
    document.removeEventListener("keydown", onKey);
    if (activeClose === close) activeClose = null;
  };
  const onKey = (e: KeyboardEvent) => {
    if (e.key === "Escape") close();
  };
  closeBtn.addEventListener("click", close);
  document.addEventListener("keydown", onKey);
  activeClose = close;
  return close;
}

/** A labeled form row: label text on the left, the control on the right. */
export function formRow(label: string, control: HTMLElement, hint?: string): HTMLElement {
  const row = document.createElement("div");
  row.className = "form-row";
  const lab = document.createElement("label");
  lab.className = "form-label";
  lab.textContent = label;
  if (hint) lab.title = hint;
  row.appendChild(lab);
  row.appendChild(control);
  return row;
}
