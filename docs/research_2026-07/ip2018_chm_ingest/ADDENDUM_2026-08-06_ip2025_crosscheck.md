# Addendum 2026-08-06 — corrections from the IP 2025 cross-check

The full IP 2025 CHM ingest (`..\ip2025_chm_ingest\`, 343/343 pages, 14 slice
reports + FINDINGS.md + DISCREPANCIES.md) differentially re-read every page of
this 2018 corpus. The 2018 reports below are **left unedited** as a preserved
record; this note lists what the cross-check showed to be wrong or now closed.
Where a 2018 file and this addendum disagree, this addendum wins.

## Corrections (2018 record was wrong or incomplete)

1. **`F_envcorr_tierc_citations.md` §F1.6 — EERC reference list.** Recorded as
   "not recoverable (raster)". Wrong: `_eercclip0007.png` is byte-identical in
   both editions and simply was not vision-read in 2018. All 10 entries are now
   captured in `..\ip2025_chm_ingest\F_qc_edit_corrections.md` §7.6.
2. **Juhász / Waxman-Smits equation transcription.** The 2018 mineral-solver
   extraction carries the ASCII form, which omits the `×Rw` factor (and the
   `m*` exponent for W&S). That form is dimensionally wrong — it adds a
   conductivity to a dimensionless 1. The correct raster form is adopted as
   global ledger item **D-12** (`..\ip2025_chm_ingest\DISCREPANCIES.md` Part 1);
   evidence in the 2025 B and C reports.
3. **Cutoff averaging equations.** Recorded in 2018 as unrecoverable raster
   content. All six are now transcribed (`..\ip2025_chm_ingest\D_cutoffs_montecarlo.md`
   §2) — the rasters were readable; 2018 extraction stopped short.
4. **Monte-Carlo iteration default.** Open in 2018; the shipped dialog default
   is **2000** iterations (with 300 minimum, 200 burn-in) — a product default,
   not workflow advice. Same D report, §6.

## Provenance transfers

5. **NeuroSolutions NN engine disclosure.** IP 2018's NN pages disclose
   "NeuroSolutions 5.5 … Hidden layers = 1"; **IP 2025 removes this statement
   entirely.** The Tier-C register's NN entry must cite the IP 2018 extract as
   its only source. (2025 G report §7.)

## 2018 ledger items resolved by the 2025 ingest

Statuses now maintained in `..\ip2025_chm_ingest\DISCREPANCIES.md` Part 1:
D-02 Hingle **resolved** (`Y = Rt^(−1/m)`, twice-independently); D-06 pore-size
array **vendor-fixed** in 2025 (0.01 µm). D-01 corroborated (0/98, doc bug still
shipped). D-07 remains OPEN — the malformed bracket string is byte-identical in
both editions' raw HTML, so it is a vendor-source defect, not a decompiler
artefact; resolve only against SPWLA Paper AA (Hill, Shirley & Klein 1979).
