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

export type Locale = "en" | "id" | "su" | "jv";

const STORAGE_KEY = "sandibumi.locale";

/** English → Bahasa Indonesia. Keys must match the visible English text exactly (trimmed). */
const ID: Record<string, string> = {
  // ribbon tabs
  Project: "Proyek",
  Petrophysics: "Petrofisika",
  View: "Tampilan",
  // Project tab
  "Save Project As…": "Simpan Proyek Sebagai…",
  "Open Project…": "Buka Proyek…",
  "New Project…": "Proyek Baru…",
  Recent: "Terbaru",
  "No recent projects": "Belum ada proyek terbaru",
  // Project tab — Session / Edit / Monitor groups (the old quick-access strip)
  "Save Session…": "Simpan Sesi…",
  "Open Session…": "Buka Sesi…",
  Undo: "Batalkan",
  Redo: "Ulangi",
  History: "Riwayat",
  Processing: "Pemrosesan",
  Performance: "Kinerja",
  Monitor: "Pemantau",
  // Workflow builder (List/Grid inspector)
  List: "Daftar",
  Grid: "Kisi",
  "Set all": "Atur semua",
  "(set all)": "(atur semua)",
  Step: "Langkah",
  all: "semua",
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
  "Import Logs": "Impor Log",
  "Import Data": "Impor Data",
  Tools: "Alat",
  "Import LAS…": "Impor LAS…",
  "Export LAS…": "Ekspor LAS…",
  "Import Core…": "Impor Core…",
  "Import SCAL…": "Impor SCAL…",
  "Import Tops…": "Impor Tops…",
  "Import Aux…": "Impor Aux…",
  "Import Images…": "Impor Gambar…",
  Images: "Gambar",
  Dataset: "Kumpulan Data",
  Placement: "Penempatan",
  "Anchored at depth": "Ditambat pada kedalaman",
  "Scaled to interval": "Diskalakan ke interval",
  "Width of track": "Lebar track",
  Align: "Perataan",
  Fit: "Penyesuaian",
  "Whole picture": "Seluruh gambar",
  "Fill and crop": "Penuhi dan potong",
  "Name label": "Label nama",
  Frame: "Bingkai",
  Caption: "Keterangan",
  "Delivery name": "Nama pengiriman",
  "Depths are in": "Kedalaman dalam",
  "Long edge (px)": "Sisi panjang (px)",
  Pixels: "Piksel",
  "Base (optional)": "Dasar (opsional)",
  "Autocorrelate Tops…": "Autokorelasi Tops…",
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
  "Workbook…": "Buku Kerja…",
  "Export workbook (Excel)": "Ekspor buku kerja (Excel)",
  "Study title": "Judul studi",
  "Pay Summary sheet": "Lembar Pay Summary",
  "Field Summary sheet": "Lembar Field Summary",
  "Zone Parameters sheet": "Lembar Zone Parameters",
  "Deck…": "Paparan…",
  "Export deck (PowerPoint)": "Ekspor paparan (PowerPoint)",
  "Deck title": "Judul paparan",
  "Presented by": "Dipaparkan oleh",
  "Summarise at": "Ringkas pada",
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
  "SandiBumi did not close properly last time.": "SandiBumi tidak tertutup dengan benar sebelumnya.",
  "Start in Safe Mode": "Mulai dalam Mode Aman",
  "Restore autosaved workspace": "Pulihkan ruang kerja tersimpan otomatis",
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
  // Common actions, labels and statuses — fuller coverage (2026-07-21). Generic UI words only;
  // petrophysics jargon stays English per the module doc above.
  New: "Baru",
  Open: "Buka",
  Edit: "Ubah",
  Rename: "Ganti nama",
  Duplicate: "Duplikat",
  Search: "Cari",
  Filter: "Saring",
  Clear: "Bersihkan",
  Copy: "Salin",
  Print: "Cetak",
  Export: "Ekspor",
  Import: "Impor",
  Refresh: "Segarkan",
  Reset: "Atur ulang",
  Help: "Bantuan",
  Settings: "Pengaturan",
  Options: "Opsi",
  Done: "Selesai",
  Next: "Berikutnya",
  Back: "Kembali",
  Yes: "Ya",
  No: "Tidak",
  "Save Session": "Simpan Sesi",
  "Open Session": "Buka Sesi",
  "Save Session As": "Simpan Sesi Sebagai",
  "Session name": "Nama sesi",
  Session: "Sesi",
  Sessions: "Sesi",
  Value: "Nilai",
  Type: "Tipe",
  Unit: "Satuan",
  Units: "Satuan",
  Method: "Metode",
  Model: "Model",
  Zone: "Zona",
  Zones: "Zona",
  Interval: "Interval",
  Top: "Puncak",
  Base: "Dasar",
  Field: "Lapangan",
  Group: "Grup",
  Groups: "Grup",
  Layout: "Tata Letak",
  Layouts: "Tata Letak",
  Track: "Trek",
  Tracks: "Trek",
  Parameter: "Parameter",
  Parameters: "Parameter",
  "Curve name": "Nama kurva",
  "Well name": "Nama sumur",
  "No data": "Tidak ada data",
  "No valid data": "Tidak ada data valid",
  Loading: "Memuat",
  Saving: "Menyimpan",
  Failed: "Gagal",
};

/** English → Basa Sunda. Same key set discipline as ID (visible English text, trimmed). */
const SU: Record<string, string> = {
  Project: "Proyék",
  Petrophysics: "Petrofisika",
  View: "Pintonan",
  "Save Project As…": "Simpen Proyék Jadi…",
  "Open Project…": "Buka Proyék…",
  "New Project…": "Proyék Anyar…",
  Recent: "Panganyarna",
  "No recent projects": "Can aya proyék panganyarna",
  // Project tab — Session / Edit / Monitor groups (the old quick-access strip)
  "Save Session…": "Simpen Sési…",
  "Open Session…": "Buka Sési…",
  Undo: "Bolaykeun",
  Redo: "Balikan deui",
  History: "Riwayat",
  Processing: "Prosés",
  Performance: "Kinerja",
  Monitor: "Pamantau",
  // Workflow builder (List/Grid inspector)
  List: "Daptar",
  Grid: "Kisi",
  "Set all": "Atur sadayana",
  "(set all)": "(atur sadayana)",
  Step: "Léngkah",
  all: "sadayana",
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
  "Import Logs": "Impor Log",
  "Import Data": "Impor Data",
  Tools: "Alat",
  "Import LAS…": "Impor LAS…",
  "Export LAS…": "Ékspor LAS…",
  "Import Core…": "Impor Core…",
  "Import SCAL…": "Impor SCAL…",
  "Import Tops…": "Impor Tops…",
  "Import Aux…": "Impor Aux…",
  "Import Images…": "Impor Gambar…",
  Images: "Gambar",
  Dataset: "Kumpulan Data",
  Placement: "Panempatan",
  "Anchored at depth": "Dijangkarkeun dina jero",
  "Scaled to interval": "Diskalakeun ka interval",
  "Width of track": "Rubak track",
  Align: "Panyaruaan",
  Fit: "Panyaluyuan",
  "Whole picture": "Sakabeh gambar",
  "Fill and crop": "Pinuhan jeung poton",
  "Name label": "Label ngaran",
  Frame: "Pigura",
  Caption: "Katerangan",
  "Delivery name": "Ngaran pangiriman",
  "Depths are in": "Jero dina",
  "Long edge (px)": "Sisi panjang (px)",
  Pixels: "Piksel",
  "Base (optional)": "Dasar (opsional)",
  "Autocorrelate Tops…": "Autokorélasi Tops…",
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
  "Workbook…": "Buku Gawé…",
  "Export workbook (Excel)": "Ékspor buku gawé (Excel)",
  "Study title": "Judul studi",
  "Pay Summary sheet": "Lambaran Pay Summary",
  "Field Summary sheet": "Lambaran Field Summary",
  "Zone Parameters sheet": "Lambaran Zone Parameters",
  "Deck…": "Paparan…",
  "Export deck (PowerPoint)": "Ékspor paparan (PowerPoint)",
  "Deck title": "Judul paparan",
  "Presented by": "Dipaparkeun ku",
  "Summarise at": "Ringkeskeun dina",
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
  "SandiBumi did not close properly last time.": "SandiBumi teu katutup kalayan bener saméméhna.",
  "Start in Safe Mode": "Mimitian dina Mode Aman",
  "Restore autosaved workspace": "Balikkeun rohang gawé nu disimpen otomatis",
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
  // Common actions, labels and statuses — fuller coverage (2026-07-21).
  New: "Anyar",
  Open: "Buka",
  Edit: "Robah",
  Rename: "Ganti ngaran",
  Duplicate: "Duplikat",
  Search: "Téangan",
  Filter: "Saring",
  Clear: "Beresihan",
  Copy: "Salin",
  Print: "Citak",
  Export: "Ékspor",
  Import: "Impor",
  Refresh: "Seger deui",
  Reset: "Atur deui",
  Help: "Pitulung",
  Settings: "Setélan",
  Options: "Pilihan",
  Done: "Réngsé",
  Next: "Salajengna",
  Back: "Balik",
  Yes: "Enya",
  No: "Henteu",
  "Save Session": "Simpen Sési",
  "Open Session": "Buka Sési",
  "Save Session As": "Simpen Sési Jadi",
  "Session name": "Ngaran sési",
  Session: "Sési",
  Sessions: "Sési",
  Value: "Niléy",
  Type: "Tipe",
  Unit: "Unit",
  Units: "Unit",
  Method: "Métode",
  Model: "Modél",
  Zone: "Zona",
  Zones: "Zona",
  Interval: "Interval",
  Top: "Puncak",
  Base: "Dasar",
  Field: "Lapangan",
  Group: "Grup",
  Groups: "Grup",
  Layout: "Tata Perenah",
  Layouts: "Tata Perenah",
  Track: "Trek",
  Tracks: "Trek",
  Parameter: "Paraméter",
  Parameters: "Paraméter",
  "Curve name": "Ngaran kurva",
  "Well name": "Ngaran sumur",
  "No data": "Teu aya data",
  "No valid data": "Teu aya data valid",
  Loading: "Ngamuat",
  Saving: "Nyimpen",
  Failed: "Gagal",
};

/** English → Basa Jawa (ngoko). Same key set discipline as ID/SU (visible English text,
 *  trimmed). Petrophysics jargon deliberately stays English. */
const JV: Record<string, string> = {
  Project: "Proyèk",
  Petrophysics: "Petrofisika",
  View: "Tampilan",
  "Save Project As…": "Simpen Proyèk Dadi…",
  "Open Project…": "Bukak Proyèk…",
  "New Project…": "Proyèk Anyar…",
  Recent: "Pungkasan",
  "No recent projects": "Durung ana proyèk pungkasan",
  // Project tab — Session / Edit / Monitor groups (the old quick-access strip)
  // "Sési" (not "Sèsi") to match the existing `Session` caption key further down — the group
  // caption and its buttons must not disagree on the spelling of the same word.
  "Save Session…": "Simpen Sési…",
  "Open Session…": "Bukak Sési…",
  Undo: "Batalké",
  Redo: "Balèni",
  History: "Riwayat",
  Processing: "Pangolahan",
  Performance: "Kinerja",
  Monitor: "Pamantau",
  List: "Dhaptar",
  Grid: "Kisi",
  "Set all": "Setel kabèh",
  "(set all)": "(setel kabèh)",
  Step: "Langkah",
  all: "kabèh",
  Theme: "Tema",
  Appearance: "Penampilan",
  Standard: "Standar",
  Client: "Klièn",
  Light: "Padhang",
  Default: "Baku",
  Dark: "Peteng",
  System: "Sistem",
  "White / Grey": "Putih / Kelabu",
  Language: "Basa",
  "Import Logs": "Impor Log",
  "Import Data": "Impor Data",
  Tools: "Piranti",
  "Import LAS…": "Impor LAS…",
  "Export LAS…": "Ekspor LAS…",
  "Import Core…": "Impor Core…",
  "Import SCAL…": "Impor SCAL…",
  "Import Tops…": "Impor Tops…",
  "Import Aux…": "Impor Aux…",
  "Import Images…": "Impor Gambar…",
  Images: "Gambar",
  Dataset: "Kumpulan Data",
  Placement: "Panggonan",
  "Anchored at depth": "Diancer ing jerone",
  "Scaled to interval": "Diskalakake menyang interval",
  "Width of track": "Ambane track",
  Align: "Panjajaran",
  Fit: "Panyelarasan",
  "Whole picture": "Sakabehe gambar",
  "Fill and crop": "Kebakake lan keteh",
  "Name label": "Label jeneng",
  Frame: "Pigura",
  Caption: "Katrangan",
  "Delivery name": "Jeneng kiriman",
  "Depths are in": "Jerone ing",
  "Long edge (px)": "Sisih dawa (px)",
  Pixels: "Piksel",
  "Base (optional)": "Dhasar (opsional)",
  "Autocorrelate Tops…": "Autokorélasi Tops…",
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
  Manage: "Ngatur",
  "Zones…": "Zona…",
  Intervals: "Interval",
  "Cutoffs & Summary…": "Cutoff & Ringkesan…",
  Reporting: "Palaporan",
  "Workflow…": "Alur Kerja…",
  "Field Dashboard…": "Dasbor Lapangan…",
  Porosity: "Porositas",
  Saturation: "Saturasi",
  Permeability: "Permeabilitas",
  Facies: "Fasies",
  "New Log View": "Tampilan Log Anyar",
  "Save Layout…": "Simpen Tata Letak…",
  "Properties…": "Properti…",
  "Log Views": "Tampilan Log",
  "Active Layout": "Tata Letak Aktif",
  "Parameter Selection": "Pamilihan Parameter",
  Correlation: "Korélasi",
  "Multi-Well": "Multi-Sumur",
  "Composite…": "Komposit…",
  "Report…": "Laporan…",
  "Workbook…": "Buku Kerja…",
  "Export workbook (Excel)": "Ekspor buku kerja (Excel)",
  "Study title": "Irah-irahan studi",
  "Pay Summary sheet": "Lembar Pay Summary",
  "Field Summary sheet": "Lembar Field Summary",
  "Zone Parameters sheet": "Lembar Zone Parameters",
  "Deck…": "Paparan…",
  "Export deck (PowerPoint)": "Ekspor paparan (PowerPoint)",
  "Deck title": "Irah-irahan paparan",
  "Presented by": "Dipaparake dening",
  "Summarise at": "Ringkes ing",
  Deliverables: "Asil Pungkasan",
  "New Window": "Jendhéla Anyar",
  "Reset Workspace": "Reset Papan Kerja",
  Workspace: "Papan Kerja",
  Ready: "Siap",
  Cancel: "Batal",
  Close: "Tutup",
  Reload: "Muat manèh",
  "Workflow Builder": "Panyusun Alur Kerja",
  "Reload SandiBumi? The workspace re-opens from its last saved state — unsaved picks, layouts and dialog inputs are lost.":
    "Muat manèh SandiBumi? Papan kerja dibukak manèh saka kahanan pungkasan sing kasimpen — pilihan, tata letak, lan isian dialog sing durung kasimpen bakal ilang.",
  "Number fields arm on click — double-click to edit":
    "Kolom angka kekunci pas diklik — klik kaping pindho kanggo ngowahi",
  "SandiBumi did not close properly last time.": "SandiBumi ora ditutup kanthi bener sadurungé.",
  "Start in Safe Mode": "Miwiti ing Mode Aman",
  "Restore autosaved workspace": "Balèkna papan kerja sing kasimpen otomatis",
  Apply: "Terapna",
  Run: "Jalanké",
  Save: "Simpen",
  Delete: "Busak",
  Add: "Tambah",
  Remove: "Busak",
  Name: "Jeneng",
  Well: "Sumur",
  Wells: "Sumur",
  Depth: "Jero",
  Color: "Warna",
  Curve: "Kurva",
  Curves: "Kurva",
  Inspector: "Inspektur",
  Output: "Kaluaran",
  Algorithm: "Algoritma",
  // Common actions, labels and statuses — fuller coverage (2026-07-21).
  New: "Anyar",
  Open: "Bukak",
  Edit: "Sunting",
  Rename: "Ganti jeneng",
  Duplicate: "Gandakna",
  Search: "Golèk",
  Filter: "Saring",
  Clear: "Resiki",
  Copy: "Salin",
  Print: "Cithak",
  Export: "Ekspor",
  Import: "Impor",
  Refresh: "Seger manèh",
  Reset: "Setel manèh",
  Help: "Pitulung",
  Settings: "Setèlan",
  Options: "Pilihan",
  Done: "Rampung",
  Next: "Sabanjuré",
  Back: "Bali",
  Yes: "Ya",
  No: "Ora",
  "Save Session": "Simpen Sési",
  "Open Session": "Bukak Sési",
  "Save Session As": "Simpen Sési Dadi",
  "Session name": "Jeneng sési",
  Session: "Sési",
  Sessions: "Sési",
  Value: "Nilai",
  Type: "Tipe",
  Unit: "Unit",
  Units: "Unit",
  Method: "Metode",
  Model: "Model",
  Zone: "Zona",
  Zones: "Zona",
  Interval: "Interval",
  Top: "Puncak",
  Base: "Dhasar",
  Field: "Lapangan",
  Group: "Grup",
  Groups: "Grup",
  Layout: "Tata Letak",
  Layouts: "Tata Letak",
  Track: "Trek",
  Tracks: "Trek",
  Parameter: "Parameter",
  Parameters: "Parameter",
  "Curve name": "Jeneng kurva",
  "Well name": "Jeneng sumur",
  "No data": "Ora ana data",
  "No valid data": "Ora ana data valid",
  Loading: "Ngemot",
  Saving: "Nyimpen",
  Failed: "Gagal",
};

const DICTS: Partial<Record<Locale, Record<string, string>>> = { id: ID, su: SU, jv: JV };

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
  current = stored === "id" || stored === "su" || stored === "jv" ? stored : "en";
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
