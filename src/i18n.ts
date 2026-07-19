/** UI language setting (English / Bahasa Indonesia / Basa Sunda).
 *
 *  Design: English remains the SOURCE language everywhere in code and markup. This module
 *  translates what the user SEES — visible DOM text nodes plus title/placeholder/aria-label
 *  attributes — by exact-phrase dictionary lookup, live via a MutationObserver so dynamically
 *  built panels and dialogs are covered without threading t() through every call site.
 *
 *  Deliberate consequence: any phrase NOT in the dictionary stays English. That is the
 *  requested behaviour — technical petrophysics vocabulary (Thin Beds, Monte Carlo, Pickett,
 *  curve mnemonics, LAS/DLIS, cutoff names) must not be force-translated. Never "complete"
 *  the dictionary with jargon; only add phrases that read naturally in id/su.
 *
 *  User data (well names, curve mnemonics, layout names) passes through untouched because
 *  dictionary keys are UI phrases that data strings don't collide with. Elements carrying
 *  data-no-i18n (and their subtrees) are skipped entirely.
 */

export type Locale = "en" | "id" | "su";

const STORAGE_KEY = "sandibumi.locale";

/** English → Bahasa Indonesia. Keys must match the visible English text exactly (trimmed). */
const ID: Record<string, string> = {
  // ribbon tabs
  Project: "Proyek",
  Petrophysics: "Petrofisika",
  View: "Tampilan",
  // Project tab
  "Save Project As…": "Simpan Proyek Sebagai…",
  Theme: "Tema",
  Appearance: "Penampilan",
  Standard: "Standar",
  Client: "Klien",
  Light: "Terang",
  Default: "Bawaan",
  Dark: "Gelap",
  System: "Sistem",
  "White / Grey": "Putih / Abu-abu",
  Language: "Bahasa",
  // Data tab
  "Import LAS…": "Impor LAS…",
  "Export LAS…": "Ekspor LAS…",
  "Import Core…": "Impor Core…",
  "Import SCAL…": "Impor SCAL…",
  "Shift Core…": "Geser Core…",
  "Import DLIS…": "Impor DLIS…",
  "Import / Export": "Impor / Ekspor",
  "Import Deviation…": "Impor Deviasi…",
  "Well Header…": "Header Sumur…",
  "Well Data": "Data Sumur",
  "Wells & Tops": "Sumur & Tops",
  "Curve Catalog": "Katalog Kurva",
  "DB Inspector": "Inspektur DB",
  "SQL Query": "Kueri SQL",
  Manage: "Kelola",
  // Petrophysics tab
  "Zones…": "Zona…",
  Intervals: "Interval",
  "Cutoffs & Summary…": "Cutoff & Ringkasan…",
  Reporting: "Pelaporan",
  "Workflow…": "Alur Kerja…",
  "Field Dashboard…": "Dasbor Lapangan…",
  Porosity: "Porositas",
  Saturation: "Saturasi",
  Permeability: "Permeabilitas",
  Facies: "Fasies",
  // Plot tab
  "New Log View": "Tampilan Log Baru",
  "Save Layout…": "Simpan Layout…",
  "Properties…": "Properti…",
  "Log Views": "Tampilan Log",
  "Active Layout": "Layout Aktif",
  "Parameter Selection": "Pemilihan Parameter",
  Correlation: "Korelasi",
  "Multi-Well": "Multi-Sumur",
  "Composite…": "Komposit…",
  "Report…": "Laporan…",
  Deliverables: "Hasil Akhir",
  // View tab
  "New Window": "Jendela Baru",
  "Reset Workspace": "Reset Ruang Kerja",
  Workspace: "Ruang Kerja",
  // status + common dialog vocabulary
  Ready: "Siap",
  Cancel: "Batal",
  Close: "Tutup",
  Reload: "Muat ulang",
  "Workflow Builder": "Penyusun Alur Kerja",
  "Reload SandiBumi? The workspace re-opens from its last saved state — unsaved picks, layouts and dialog inputs are lost.":
    "Muat ulang SandiBumi? Ruang kerja dibuka kembali dari kondisi tersimpan terakhir — pick, layout, dan isian dialog yang belum disimpan akan hilang.",
  "Number fields arm on click — double-click to edit":
    "Kolom angka terkunci saat diklik — klik dua kali untuk mengedit",
  Apply: "Terapkan",
  Run: "Jalankan",
  Save: "Simpan",
  Delete: "Hapus",
  Add: "Tambah",
  Remove: "Hapus",
  Name: "Nama",
  Well: "Sumur",
  Wells: "Sumur",
  Depth: "Kedalaman",
  Color: "Warna",
  Curve: "Kurva",
  Curves: "Kurva",
  Inspector: "Inspektur",
  Output: "Keluaran",
  Algorithm: "Algoritma",
};

/** English → Basa Sunda. Same key set discipline as ID (visible English text, trimmed). */
const SU: Record<string, string> = {
  Project: "Proyék",
  Petrophysics: "Petrofisika",
  View: "Pintonan",
  "Save Project As…": "Simpen Proyék Jadi…",
  Theme: "Téma",
  Appearance: "Penampilan",
  Standard: "Standar",
  Client: "Klién",
  Light: "Caang",
  Default: "Baku",
  Dark: "Poék",
  System: "Sistem",
  "White / Grey": "Bodas / Kulawu",
  Language: "Basa",
  "Import LAS…": "Impor LAS…",
  "Export LAS…": "Ékspor LAS…",
  "Import Core…": "Impor Core…",
  "Import SCAL…": "Impor SCAL…",
  "Shift Core…": "Gésér Core…",
  "Import DLIS…": "Impor DLIS…",
  "Import / Export": "Impor / Ékspor",
  "Import Deviation…": "Impor Déviasi…",
  "Well Header…": "Header Sumur…",
  "Well Data": "Data Sumur",
  "Wells & Tops": "Sumur & Tops",
  "Curve Catalog": "Katalog Kurva",
  "DB Inspector": "Inspéktur DB",
  "SQL Query": "Kueri SQL",
  Manage: "Ngatur",
  "Zones…": "Zona…",
  Intervals: "Interval",
  "Cutoffs & Summary…": "Cutoff & Ringkesan…",
  Reporting: "Palaporan",
  "Workflow…": "Alur Gawé…",
  "Field Dashboard…": "Dasbor Lapangan…",
  Porosity: "Porositas",
  Saturation: "Saturasi",
  Permeability: "Permeabilitas",
  Facies: "Fasies",
  "New Log View": "Pintonan Log Anyar",
  "Save Layout…": "Simpen Layout…",
  "Properties…": "Properti…",
  "Log Views": "Pintonan Log",
  "Active Layout": "Layout Aktif",
  "Parameter Selection": "Pilihan Paraméter",
  Correlation: "Korélasi",
  "Multi-Well": "Multi-Sumur",
  "Composite…": "Komposit…",
  "Report…": "Laporan…",
  Deliverables: "Hasil Ahir",
  "New Window": "Jandéla Anyar",
  "Reset Workspace": "Reset Rohang Gawé",
  Workspace: "Rohang Gawé",
  Ready: "Siap",
  Cancel: "Batal",
  Close: "Tutup",
  Reload: "Muat deui",
  "Workflow Builder": "Panyusun Alur Gawé",
  "Reload SandiBumi? The workspace re-opens from its last saved state — unsaved picks, layouts and dialog inputs are lost.":
    "Muat deui SandiBumi? Rohang gawé dibuka deui tina kaayaan panungtungan nu disimpen — pick, layout, jeung eusian dialog nu can disimpen bakal leungit.",
  "Number fields arm on click — double-click to edit":
    "Kolom angka konci basa diklik — klik dua kali pikeun ngédit",
  Apply: "Larapkeun",
  Run: "Jalankeun",
  Save: "Simpen",
  Delete: "Pupus",
  Add: "Tambah",
  Remove: "Pupus",
  Name: "Ngaran",
  Well: "Sumur",
  Wells: "Sumur",
  Depth: "Jero",
  Color: "Warna",
  Curve: "Kurva",
  Curves: "Kurva",
  Inspector: "Inspéktur",
  Output: "Kaluaran",
  Algorithm: "Algoritma",
};

const DICTS: Partial<Record<Locale, Record<string, string>>> = { id: ID, su: SU };

/** Attributes whose values are user-visible text (label covers <optgroup>). */
const TEXT_ATTRS = ["title", "placeholder", "aria-label", "label"];

let current: Locale = "en";

// Original English text is remembered the first time a node is translated so any later
// locale switch (including back to English) re-derives from the source, never from a
// previous translation. `applied` breaks the observer feedback loop: a characterData
// mutation whose new value is exactly what we just wrote is our own write, not app code.
const originals = new WeakMap<Text, string>();
const applied = new WeakMap<Text, string>();
const attrOriginals = new WeakMap<Element, Map<string, string>>();
const attrApplied = new WeakMap<Element, Map<string, string>>();

export function getLocale(): Locale {
  return current;
}

/** Translate one phrase for authored code (dialog titles etc.). Unknown → unchanged. */
export function t(phrase: string): string {
  return DICTS[current]?.[phrase] ?? phrase;
}

export function setLocale(locale: Locale): void {
  current = locale;
  localStorage.setItem(STORAGE_KEY, locale);
  document.documentElement.lang = locale;
  translateTree(document.body);
}

/** Reads the stored locale, translates the initial DOM, and starts the live observer. */
export function initI18n(): void {
  const stored = localStorage.getItem(STORAGE_KEY);
  current = stored === "id" || stored === "su" ? stored : "en";
  document.documentElement.lang = current;
  if (current !== "en") translateTree(document.body);

  const observer = new MutationObserver((mutations) => {
    if (current === "en") return;
    for (const m of mutations) {
      if (m.type === "characterData") {
        const node = m.target;
        if (node instanceof Text && node.data !== applied.get(node)) {
          // app code rewrote the text — the new data is the new English original
          originals.set(node, node.data);
          translateText(node);
        }
      } else {
        for (const added of m.addedNodes) {
          if (added instanceof Text) translateText(added);
          else if (added instanceof Element) translateTree(added);
        }
      }
    }
  });
  observer.observe(document.body, { childList: true, characterData: true, subtree: true });
}

function skip(el: Element | null): boolean {
  return el !== null && (el.closest("[data-no-i18n]") !== null || el.closest("script,style") !== null);
}

function translateTree(root: Element): void {
  if (skip(root)) return;
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    translateText(n as Text);
  }
  translateAttrs(root);
  for (const el of root.querySelectorAll(TEXT_ATTRS.map((a) => `[${a}]`).join(","))) {
    translateAttrs(el);
  }
}

function translateText(node: Text): void {
  if (skip(node.parentElement)) return;
  let orig = originals.get(node);
  if (orig === undefined) {
    if (current === "en") return; // untouched node in English mode — nothing to do or record
    orig = node.data;
    originals.set(node, orig);
  }
  const key = orig.trim();
  if (!key) return;
  const tr = DICTS[current]?.[key];
  const next = tr !== undefined ? orig.replace(key, tr) : orig;
  if (node.data !== next) {
    applied.set(node, next);
    node.data = next;
  }
}

function translateAttrs(el: Element): void {
  if (skip(el)) return;
  for (const attr of TEXT_ATTRS) {
    const value = el.getAttribute(attr);
    if (value === null) continue;
    let origMap = attrOriginals.get(el);
    if (origMap === undefined) {
      origMap = new Map();
      attrOriginals.set(el, origMap);
    }
    let appliedMap = attrApplied.get(el);
    if (appliedMap === undefined) {
      appliedMap = new Map();
      attrApplied.set(el, appliedMap);
    }
    let orig = origMap.get(attr);
    // A value differing from what we last wrote means app code changed the attribute:
    // that new value is the new English original. (Our own writes match appliedMap.)
    if (orig === undefined || (value !== orig && value !== appliedMap.get(attr))) {
      if (current === "en" && orig === undefined) continue;
      orig = value;
      origMap.set(attr, orig);
    }
    const tr = DICTS[current]?.[orig.trim()];
    const next = tr !== undefined ? tr : orig;
    if (el.getAttribute(attr) !== next) {
      appliedMap.set(attr, next);
      el.setAttribute(attr, next);
    }
  }
}
