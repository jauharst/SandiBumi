/** A lightweight right-click context menu, appended to <body> and positioned at the
 *  pointer, kept flush against the viewport edges. Mirrors the look of the dock ＋ menu.
 *  Callers pass a list of items whose contents vary by the panel that was clicked — this
 *  is what makes the menu "personalized by active window". */

export interface ContextMenuItem {
  label: string;
  onClick: () => void;
  disabled?: boolean;
  /** Renders in the danger colour (Close Window etc.). */
  danger?: boolean;
}

export type ContextMenuEntry = ContextMenuItem | "sep" | { heading: string };

let openMenu: HTMLElement | null = null;

/** Closes any open context menu (also called on outside click / Escape / scroll). */
export function closeContextMenu(): void {
  openMenu?.remove();
  openMenu = null;
}

export function showContextMenu(x: number, y: number, entries: ContextMenuEntry[]): void {
  closeContextMenu();
  const menu = document.createElement("div");
  menu.className = "context-menu";

  for (const entry of entries) {
    if (entry === "sep") {
      const sep = document.createElement("div");
      sep.className = "context-menu-sep";
      menu.appendChild(sep);
      continue;
    }
    if ("heading" in entry) {
      const h = document.createElement("div");
      h.className = "context-menu-heading";
      h.textContent = entry.heading;
      menu.appendChild(h);
      continue;
    }
    const item = document.createElement("button");
    item.className = "context-menu-item";
    if (entry.danger) item.classList.add("danger");
    item.textContent = entry.label;
    item.disabled = !!entry.disabled;
    if (!entry.disabled) {
      item.addEventListener("click", () => {
        closeContextMenu();
        entry.onClick();
      });
    }
    menu.appendChild(item);
  }

  // Position off-screen first so we can measure, then clamp inside the viewport.
  menu.style.left = "-9999px";
  menu.style.top = "-9999px";
  document.body.appendChild(menu);
  const { width, height } = menu.getBoundingClientRect();
  menu.style.left = `${Math.max(4, Math.min(x, window.innerWidth - width - 4))}px`;
  menu.style.top = `${Math.max(4, Math.min(y, window.innerHeight - height - 4))}px`;
  openMenu = menu;

  const onOutside = (e: MouseEvent) => {
    if (!menu.contains(e.target as Node)) closeContextMenu();
  };
  const onKey = (e: KeyboardEvent) => {
    if (e.key === "Escape") closeContextMenu();
  };
  // Defer the outside-click listener so the opening right-click doesn't immediately close it.
  window.setTimeout(() => {
    document.addEventListener("mousedown", onOutside);
    document.addEventListener("keydown", onKey);
    window.addEventListener("blur", closeContextMenu);
    document.addEventListener("scroll", closeContextMenu, true);
  }, 0);
  // Clean the global listeners up when the menu is removed.
  const observer = new MutationObserver(() => {
    if (!document.body.contains(menu)) {
      document.removeEventListener("mousedown", onOutside);
      document.removeEventListener("keydown", onKey);
      window.removeEventListener("blur", closeContextMenu);
      document.removeEventListener("scroll", closeContextMenu, true);
      observer.disconnect();
    }
  });
  observer.observe(document.body, { childList: true });
}
