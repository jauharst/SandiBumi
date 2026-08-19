# DRAFT — SB-DIO-057 logarithmic-family registry (classification for sign-off)

**Status: DRAFT, delivered 2026-08-19 under DEC-073 item 7** (corpus-drafted logarithmic-
family registry; Jauhar adjudicates rows the corpus cannot source). The requirement
(21_data-io.md §5.6): on a curve whose family is logarithmic, exact zeros MUST be counted
and surfaced for confirmation before commit — never rewritten automatically (T84/T85). The
chapter's O-5 deliberately leaves membership unclassified because *"UI log scales and
mnemonic intuition are not import authority"* — a cited classification is. This draft
classifies the ENTIRE reviewed family vocabulary (the 20 families of
`registry/unit-registry.json`, the source `curves.rs`'s tables are generated from), so no
family is left implicitly decided, and names the two prerequisite gaps.

## The classification (every family, none skipped)

### LOGARITHMIC — a zero cannot be a reading; the T84 confirmation gate applies

| Family | Source |
|---|---|
| `RES_DEEP`, `RES_MED`, `RES_SHAL`, `RXO` | The requirement's own P-tier source (memory `reference_mudlog_gas_curve_traps`, cited at 21_data-io.md §5.6: a zero in a resistivity column is an exporter's encoding of "no reading"). Structurally: every saturation method in the catalog consumes resistivity through `log`/ratio forms and the Pickett construction is log-log by definition (corpus `L_plotting_viz.md` §2) — a true zero is outside the method domain, not merely off-scale. |

### LINEAR — zero is a representable reading; commits without a confirmation gate

| Family | One-line ground |
|---|---|
| `SP`, `DRHO` | SIGNED quantities; the zero crossing is routine and meaningful (SP shale baseline; DRHO good-hole). |
| `NPHI`, `POR`, `VSH`, `VSH_UNCLIPPED`, `VCL` | Fractions where 0 is a legitimate physical answer (tight rock; clean sand). |
| `GR`, `CALI`, `BS`, `RHOB`, `PEF`, `DT`, `DTS`, `TEMP` | Linear-scale physical measurements; a zero is implausible on several (RHOB, DT) but implausible-vs-encoded is SB-DIO-030/QC territory, and promoting any of these to the confirmation gate without a source would be this draft inventing a classification. Presented as LINEAR; he moves any row he wants moved. |

### CATEGORICAL — outside the scale question entirely

| Family | Ground |
|---|---|
| `CLY_STATE` | A typed state vocabulary, not a magnitude; zero-as-reading does not apply. |

## The two named gaps — prerequisites, not classifications

1. **GAS (total gas and chromatograph components) has NO family in the reviewed registry.**
   The P-tier source names gas FIRST among the zero-as-null offenders, but the import gate
   hangs off family resolution and there is no `GAS_*` bucket to classify — a mudlog gas
   curve today imports family-less and the rule cannot reach it. Registering gas families
   (mnemonics, canonical unit, aliases) is a reviewed `unit-registry.json` change with its
   own source needs — his call on vocabulary and aliases; nothing is proposed from memory.
2. **PERM has no family in the reviewed registry either.** Structural evidence for LOG is
   in-repo (the `perm_transform` regression is `log10(PERM)` by definition; Winland/Pittman
   work in log r35), but the same prerequisite applies: no bucket, no gate. Same path as
   gap 1.

Both gaps are REGISTRATION gaps, named per the ruling ("he adjudicates rows the corpus
cannot source") — the classification for both, once registered, is expected LOG on the
already-cited grounds, but that expectation is not a registry entry until he signs it.

## What follows the signature

Implement pre-commit zero counting keyed off the signed registry (import boundary: count
exact zeros per log-family curve, surface for confirmation, never rewrite), record the
explicit keep/convert decision (T85 — the DECLINE commits the zeros as values and the
decision is recorded), and pin T84/T85 as written. The registry ships versioned beside the
unit registry so membership changes are reviewed diffs, not code edits.
