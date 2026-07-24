//! Shared helpers for putting untrusted text into the DOM safely.
//!
//! Consolidates three byte-identical private `escapeHtml` copies (dashboardPanel, inspectorPanel,
//! topsPanel) into one home, and adds `messageNode` for the common "status/placeholder line"
//! pattern. The motivating bug: a LAS `~W WELL` value is stored verbatim (parsers.rs
//! `extract_well_name` filters no characters), and several panels wrote it straight into
//! `innerHTML` — with `csp: null` in tauri.conf.json, an `<img onerror=…>` in a vendor's LAS
//! header then ran arbitrary code. LAS files arrive from service companies and clients, so this
//! is untrusted input. A single shared home means the next interpolated-innerHTML site has an
//! obvious safe primitive to reach for instead.

/** HTML-escape `text` for interpolation into an innerHTML string. Uses the browser's own encoder
 *  (set textContent, read innerHTML), so `<`, `&`, `>` and quotes become entities. Prefer
 *  `messageNode`/textContent where you control the whole node — this is for when you must build
 *  an HTML string. */
export function escapeHtml(text: string): string {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

/** `escapeHtml` plus double-quote encoding, for a value placed inside a double-quoted attribute
 *  (`title="${escapeAttr(x)}"`). */
export function escapeAttr(text: string): string {
  return escapeHtml(text).replace(/"/g, "&quot;");
}

/** A one-line status/placeholder `<div>` whose text is set via `textContent`, so any interpolated
 *  value — a well or curve name from a vendor LAS, a backend error object — is inert text and can
 *  never be parsed as markup. Use `host.replaceChildren(messageNode(cls, msg))` in place of
 *  `host.innerHTML = \`<div class="cls">…${x}…</div>\`` whenever `x` is not a string literal. */
export function messageNode(className: string, text: string): HTMLDivElement {
  const div = document.createElement("div");
  div.className = className;
  div.textContent = text;
  return div;
}
