# Saturation-height function (SHF) reference — methods banked for `shf_fit.rs` / `satheight.rs`

Portable method notes for the SHF engine (playbook #4). Tier-B math cited to primaries;
Tier-A seeds cited to the ingest files. Units: Pc psi, σ·cosθ dyn/cm, k mD, φ v/v, H m.

## Height ↔ pressure

Reservoir capillary pressure from height above the free-water level:
`Pc = 0.433·(ρw − ρhc)·h_ft` (psi; 0.433 psi/ft per unit specific gravity; h_ft = H·3.28084).
FWL and the vertical-depth input must share a reference (TVDSS for deviated wells).

## The five families

| Family | Form | Fit method | Primary |
|---|---|---|---|
| Cuddy FOIL | BVW = a·H^b (b<0) | OLS in log10-log10 | Cuddy et al. 1993 (SPWLA); Cuddy 2017 fractal update |
| Brooks-Corey | Sw = Swirr + (1−Swirr)·(He/H)^λ, H≥He | Swirr grid + log-log OLS on Se | Brooks & Corey 1964 |
| Skelt-Harrison | Sw = 1 − A·exp(−(B/(H+D))^C) | bounded Nelder-Mead (4-param SSE) | Skelt & Harrison 1995 (SPWLA) |
| Thomeer | Sw = 1 − (1−Swirr)·exp(−G/log10(H/Hd)), H>Hd; Sw=1 below Hd | bounded Nelder-Mead (3-param SSE, 4th dim pinned) | Thomeer 1960 (JPT 12(3)/Trans. AIME 219) |
| Leverett-J | Sw = A·J^B, J = 0.21645·(Pc/σcosθ)·√(k/φ) | OLS in ln-ln on per-sample J | Leverett 1941 (Trans. AIME 142) |

Thomeer height translation: lab hyperbola is Bv/B∞ = exp(−G/log10(Pc/Pd)); Pc ∝ H makes
Pc/Pd = H/Hd, so the entry pressure becomes an entry HEIGHT Hd (m); plateau B∞/φ = 1 − Swirr.
G ≈ 0.1 (well-sorted) → >2 (poorly sorted) — the carbonate-standard shape parameter.

Leverett-J display curve: the law is per-sample in J; the single overlay curve uses the median
√(k/φ) of the fitted samples (echoed as `sqrt_k_phi_med`) — representative, not the model.

## FWL scan (Cuddy Eq 19)

Step a candidate common FWL through [lo, hi]; recompute the FOIL fit at each step; pick the FWL
minimizing the mean-squared log10 residual. Applies to FOIL only; other families take the FWL
as given (from the scan, the contacts store, or hand entry).

## Per-rock-type laws

Any family + an RT/facies curve → one law per rounded RT class alongside the pooled law
(BTreeMap-ascending). Failed classes are reported with the reason, never dropped. Samples with
NaN RT join only the pooled fit (noted). Motivation: one field-pooled law mixes rock qualities;
the per-class split is the biggest SHF accuracy win on stacked deltaic sands.

## Honesty rules (result contract)

- `excluded`: (reason, count) for every dropped candidate — Sw > 1 (non-physical), Sw ≤ 0,
  at/below the FWL, below the φ cutoff, no permeability (Leverett only). Survives error returns.
- `notes`: scoped wells contributing zero samples are named (absent curve names come back as
  all-NaN columns — a "field-wide" law from a subset must say so); Buckles check (Buckles 1965)
  flags when top-height-quartile BVW has IQR/median > 0.6 — the sign per-RT laws are needed.
- Failed per-RT groups carry NaN numerics → JSON null over IPC; TS types them `number | null`.

## Tier-A seeds (per-run overridable — a vendor default is a seed, not field truth)

- σ·cosθ reservoir: Water-Oil 30 dyn/cm · cos 30° ≈ 26; Water-Gas 50 · cos 0° = 50
  (`ip_ingest/C_toplevel_par_defaults.json` → Cap_Pressure_Fluid_Prop_Defaults.par; lab systems:
  mercury 480/140°, centrifuge & porous plate 72/0°).
- Fluid densities: ρw 1.0 g/cc; ρhc 0.7 g/cc default (0.1–0.8 range) per Techlog sand-summary
  defaults (`techlog_ingest/FINDINGS.md` §C).
- Techlog `CPMParameters\` holds Lambda/φ-dependent SwH models — flagged in the ingest shortlist
  but not deep-mined yet; revisit before adding a Lambda family.

## Forward apply

`sw_height` module (`satheight.rs`): LEVERETT and SKELT forward laws, zone-overridable FWL,
TVD-aware height. Extension for the fitted families lands with the 4b dialog export.
