# DRAFT — SB-DBM-005 module → derivation-source map (awaiting sign-off)

**Status: DRAFT, delivered 2026-08-19 under DEC-073 item 4.** The ruling: engineering drafts
the COMPLETE map from `docs/` and the code's own recorded sources; Jauhar signs it and supplies
sources only for named gaps. Per DEC-073's scope note, the approval covered the approach — this
CONTENT is not authoritative until he signs it row by row. Nothing here invents a citation:
every source below was verified to exist in the repo (file and, where stated, section/line)
before being written down.

**Coverage.** One row per module `modules::module_catalog()` ships — 52 specs, the exact list
`list_modules()` returns. Not covered, with reasons: the `synthetic_*` and `sb_dbm_t06_fixture`
specs (test-only, registered outside the shipping catalog under `#[cfg(test)]` paths) and
`sw_probe` (internal probe, not a catalog module).

**Two facts the map builds on.** (1) Every module PARAMETER already carries a per-default
source enforced at catalog construction (`validate_parameter_sources` panics on an unsourced
default; `validate_saturation_methods` additionally refuses any saturation model without the
paper its answer traces to — `param_sources.rs::SATURATION_METHODS`). This map is the
METHOD-derivation layer above that: where the equation itself comes from. (2) The four status
classes below are PROPOSED — CITED and NAMED GAP are DEC-073's words; UTILITY and IN-HOUSE are
engineering's proposed refinement, because forcing a depth-shift or a splice into NAMED GAP
would manufacture gaps no source can ever fill.

## Post-signature amendment — 2026-08-26, three source strings scrubbed of personal-work identifiers

**Three rows were reworded after signature: `ssc`, `sw_rtc`, `sw_imts`.** They are the three the
legend below used to call out as a class of their own, and they were the only rows in the 52
carrying an identifier for Jauhar's own work (verified by token sweep over the whole map).

**Why a signed row was touched at all.** This map is not an internal register.
`modules::METHOD_DERIVATIONS` copies the source string into the run's ancestry at registration,
and the LAS provenance sidecar embeds the whole ancestry, so **every string here reaches a
client verbatim**.
The `ssc` row named a study programme, a study identifier and the source Loglan filename; the two
LRLC rows named a study title and the two institutions it ran under. Under the standing rule that
nothing client-bound cites his own work, those cannot travel.

**What did NOT change, deliberately.** No status class moved — each row still traces to the same
`docs/` method note it always did, which is what CITED means here. No source of record changed.
And **the attribution itself is not deleted anywhere it belongs**: `docs/method_ssc_sspw.md`,
`docs/method_lrlc_rtc_imts.md` and the `ssc.rs` / `lrlc.rs` headers keep the full provenance,
because attribution comes out only when the asset comes out. What changed is which of those two
audiences the string is written for.

**One thing the rewording had to add rather than only remove.** Dropping "GAP-2023 LQR edit" from
`ssc` would have left a bare Kuttan et al. citation for a model the shipped code has modified — a
cleaner-looking string that over-claims. The row now says the shipped form is modified and where
that is written down.

Engineering-initiated; not carrying a DEC number, and awaiting Jauhar's acknowledgement that the
signed map now reads this way.

## Post-signature amendment — 2026-08-26, `sw_rtc` reclassified IN-HOUSE

**One class moved: `sw_rtc`, CITED → IN-HOUSE.** This supersedes "No status class moved" in the
amendment above, which was accurate when written — that pass reworded three strings and left
every class deliberately alone. This is the follow-up it flagged.

**The reason is in this document's own legend.** The IN-HOUSE entry distinguished itself from
`ssc`/`sw_rtc`/`sw_imts` on the grounds that each "traces to a `docs/` method note of its own"
— but IN-HOUSE is *defined* as having exactly that ("derivation IS the repo documentation
(module doc + method note)"), so a method note cannot be what separates them. The clause that
actually discriminates is the one right after it: "and, for `ssc` and `sw_imts`, to published
papers besides." `sw_rtc` is absent from that list, and always was.

**What the sources say.** `ssc` traces to Kuttan et al.; `sw_imts` is a scaling of the
Waxman-Smits family and cites Waxman & Smits 1968, Waxman & Thomas 1974, Juhasz 1979/1981.
`sw_rtc` cites no paper: its equation is a regressed excess conductivity fitted in the
originating research, and `docs/method_lrlc_rtc_imts.md` lists Waxman-Smits, Clavier dual-water
and Juhasz under "Context" rather than as its source. That is the IN-HOUSE test as written —
SandiBumi's own algorithm whose derivation IS the repo documentation.

**The three were grouped by shared provenance, not by shared class.** All three came out of one
research programme, which is why they sat together; scrubbing all three of the same identifiers
in the first amendment is what made the mismatch visible.

**Scope, and what does not move.** `ssc` and `sw_imts` stay CITED — their published lineage is
real and is the reason. No source of record changed and no equation changed. **No number moves
either:** `DerivationClass` is a documentation field, destructured away at `ancestry.rs:170` and
in `modules::validate_method_derivations`, so only the source STRING reaches a deliverable. This
corrects the register, not the software.


## Status legend (proposed)

- **CITED** — the method derivation traces to a repo-recorded source (a `docs/` method note, a
  PRD_v2 chapter used as source, a Geolog/IP/Techlog artifact named in code, or a named paper).
- **CITED (port)** — the source of record is a Geolog Loglan `.lls` port named in the code; the
  ORIGINAL paper is not separately cited in-repo. Open question 1 below.
- **UTILITY** — a definitional data transform (shift, splice, clip, mirror, average…). Its
  statement is its own derivation; there is no physical method to source. Parameter defaults
  remain individually sourced via the `param_sources` gate.
- **IN-HOUSE** — SandiBumi's own algorithm whose derivation IS the repo documentation (module
  doc + method note). Distinct from `ssc` and `sw_imts`, which are CITED because each traces
  to a published paper besides its own `docs/` method note — `ssc` to Kuttan et al., `sw_imts`
  to the Waxman-Smits family. Having a `docs/` method note is not what makes a row CITED:
  IN-HOUSE is defined as having one. That distinction is what moved `sw_rtc` on 2026-08-26,
  recorded in the second amendment above.
- **RETIRED** — kept in the catalog only so saved chains resolve; blocked at `run_module`.

## The map

### VSH

| Module | Title | Derivation source | Status |
|---|---|---|---|
| `vsh_gr` | VSH from Gamma Ray | docs/PRD_v2/10_clay-volume.md §3.2 (linear + Larionov/Clavier/Steiber response forms); Geolog vsh_gr.info L48–L49 and vsh_gr.lls L109–L139 (cited in the module's parameter sources) | CITED |
| `vsh_dn` | VSH from Neutron-Density | docs/PRD_v2/10_clay-volume.md; Geolog vsh_dn.info; Techlog petrophysics-vsh-from-neutrondensity.htm (parameter-source citations) | CITED |

### Porosity

| Module | Title | Derivation source | Status |
|---|---|---|---|
| `phi_den` | Density porosity | docs/PRD_v2/11_porosity.md §5; Geolog V14 phi_den.info/.lls (parameter sources; the VSH ≥ 0.95 high-shale branch cited across all six phi modules) | CITED |
| `phi_dn` | Neutron-density porosity | docs/PRD_v2/11_porosity.md; Geolog V14 phi_*.lls family | CITED |
| `phi_dnbk` | Neutron-density (bulk-corrected) | docs/PRD_v2/11_porosity.md; Geolog V14 phi_*.lls family; DEC-025 declared-matrix-basis contract | CITED |
| `phi_son` | Sonic porosity | Wyllie time-average and Raymer-Hunt-Gardner forms stated in the module doc; Geolog phi_son.info DT_FL 620 µs/m and IP swparameters.htm sonic-fluid 189 (parameter sources); DT_MA < DT_SH validity per DEC-063 | CITED |
| `phimax` | Porosity ceiling | docs/PRD_v2/11_porosity.md §5 compaction-ceiling parameters; Athy exponential trend named in the module doc | CITED |
| `ssc` | Sand-Silt-Clay (Kuttan) | docs/method_ssc_sspw.md — Sand-Silt-Clay on the N-D crossplot, after Kuttan et al., *Log Interpretation in the Malay Basin*, 21st SPWLA symposium; the shipped form carries SandiBumi modifications (gas conditioning, silt and dry-clay endpoint treatment) stated in the method note | CITED |
| `sspw` | Sandstone workflow (quartz-shale-water) | docs/method_ssc_sspw.md (SSPW reconstructed from spec). Validation against the reference-suite LAS exports is still outstanding — a validation gap, not a derivation gap; open question 3 | CITED |

### Prep / QC / corrections

| Module | Title | Derivation source | Status |
|---|---|---|---|
| `ftemp_grad` | Formation temperature | docs/PRD_v2/20_envcorr-qc.md §5 formation-temperature parameters (linear TVDSS trend) | CITED |
| `precalc` | Reservoir-condition inputs | docs/PRD_v2/20_envcorr-qc.md §5 mud-filtrate parameters (linear T/P trends; RMF/CT/CXO chain) | CITED |
| `badhole` | Bad-hole QC flag | docs/PRD_v2/20_envcorr-qc.md bad-hole rows; one-hot cause-flag group per DEC-060(b) (dissolving DEC-032's coded table) | CITED |
| `condflag` | Coal/tight/conductive flags | docs/PRD_v2/20_envcorr-qc.md and 11_porosity.md condflag rows; thresholds ship as starting values per DEC-057(c) as re-ruled in DEC-060(c); coal>blank / tight-flag-only / cond-flag-only policy is Jauhar's 2026-08 ruling in docs/takeover/DECISIONS.md | CITED |
| `nphimat` | Neutron matrix conversion | Schlumberger chartbook porosity-equivalence curves Por-5 (CNL thermal) and Por-4 (APS epithermal), vector-digitized into `neutron_charts.rs` (modules.rs header above the spec); declared-basis contract per DEC-025 | CITED |
| `gascorr` | Gas correction (density) | docs/PRD_v2/11_porosity.md gascorr rows (iterated density-neutron replacement of gas volume with liquid, stated in full in the module doc); parameter sources Geolog phi_dnh.info / 3-way matrix-density agreement | CITED |
| `gr_hole_corr` | GR hole-size correction | docs/PRD_v2/20_envcorr-qc.md SB-ENV-006 and §6.2 T11/T12; DEC-031 | CITED |
| `nphi_env_corr` | Neutron environmental correction | docs/PRD_v2/20_envcorr-qc.md NPHI_EC rows. Linearized temperature+salinity form; the coefficients are deliberately USER-SUPPLIED from the applicable CNL chart at run time (the module doc says so), so no vendor chart data is embedded | CITED |
| `rhob_hole_corr` | Density hole-size correction | docs/PRD_v2/20_envcorr-qc.md SB-ENV-006 and §6.2 T12 | CITED |
| `gr_normalize` | GR normalization | docs/workflow_standards.md two-point percentile (P3/P97) method; docs/PRD_v2/20_envcorr-qc.md normalization rows. Reference endpoints are generic by the provenance rule (no client calibration ships) | CITED |
| `log_predict` | Synthetic log (KNN) | Facimage-style distance-weighted K-nearest-neighbour regression; precedent cited to Geolog V14 facimage_05_using_hc.5.05.html (K default 10); MAX_RAW washout rule per docs/workflow_standards.md | CITED |
| `depth_shift` | Depth shift | Definitional transform (linear-interpolated block shift; input never modified) | UTILITY |
| `splice` | Run-to-run splice | Definitional transform (TOP above / BOT below SPLICE_DEPTH) | UTILITY |

### Conditioning and frame

| Module | Title | Derivation source | Status |
|---|---|---|---|
| `despike` | Despike | docs/PRD_v2/20_envcorr-qc.md §5.3 conditioning parameters | UTILITY |
| `smooth` | Smooth | Definitional windowed statistics (mean/median/least-squares over a THICKNESS window); docs/PRD_v2/20_envcorr-qc.md §5.3 | UTILITY |
| `clip` | Clip | Definitional range hold (BLANK/CLAMP semantics stated in the doc) | UTILITY |
| `fill_gaps` | Fill gaps | Definitional interpolation; every invented sample flagged in `<OUT>_FILL` per Jauhar's 2026-08-05 rule recorded in the module doc | UTILITY |
| `flip` | Flip (mirror) | Definitional mirror about a pivot | UTILITY |
| `normalize` | Curve normalization | docs/workflow_standards.md P3/P97 two-point percentile; docs/PRD_v2/20_envcorr-qc.md §5.3; the reference pair has no default and the run refuses without one (docs/record_data_tools.md) | CITED |
| `block` | Block (upscale to beds) | docs/PRD_v2/20_envcorr-qc.md §5.3 frame parameters; frame contract in docs/record_data_tools.md (values replaced at the well's own depths; `draw_style: "step"`) | UTILITY |
| `bed_detect` | Bed detect | SandiBumi's own segmentation heuristic — the frame.rs module doc IS the derivation (new bed when a sample departs from the running bed mean by SENS × the curve's own first-difference noise, subject to MIN_BED; MIN_BED has no default by design). No external source exists to cite; sign-off is on the class, not a citation | IN-HOUSE |

### Saturation

Method citations below are enforced in code by
`param_sources::validate_saturation_methods` (`SATURATION_METHODS`) — the catalog panics if a
saturation model ships without its paper. Quoted verbatim from that table.

| Module | Title | Derivation source | Status |
|---|---|---|---|
| `sw_arch` | Archie | Archie 1942 Trans. AIME 146:54–62 (Geolog sw_arch.info References block; docs/PRD_v2/12_saturation.md:470) | CITED |
| `sw_indo` | Indonesia | Poupon & Leveaux 1971 SPWLA 12th Paper O (Geolog sw_indo.info References block; docs/PRD_v2/12_saturation.md:472); Worthington type 4 per Geolog | CITED |
| `sw_sim` | Simandoux (both branches) | Simandoux 1963 Revue de l'IFP (SPWLA 'Shaly Sand' Reprint Volume 1982 translation); Bardon & Pied 1969 SPWLA 10th Paper Z (Geolog sw_sim.info References block; docs/PRD_v2/12_saturation.md:470–471, :158); bisection substitution ruled in DEC-065 | CITED |
| `sw_rtc` | RtC (LRLC) | SandiBumi RtC excess-conductivity correction for low-resistivity low-contrast pay — the docs/method_lrlc_rtc_imts.md RtC sections ARE the derivation (regressed excess conductivity over the clay-chemistry Qv path + the capillary-bound-water path, then Archie on the corrected term); lrlc.rs:1–13 | IN-HOUSE |
| `sw_imts` | IMTS (LRLC) | SandiBumi IMTS mineral-textural scaling of the Waxman-Smits family; docs/method_lrlc_rtc_imts.md IMTS sections; Waxman & Smits 1968 SPEJ, Waxman & Thomas 1974 SPEJ, Juhasz 1979 SPWLA 20th Paper AA and 1981 SPWLA 22nd (docs/PRD_v2/12_saturation.md:473) | CITED |
| `multimin` | Multimin (retired) | multimin.rs:1–14 — superseded by SandiMin (`multimin2`, whose physics traces to docs/multimin_ref_spec.md + docs/multimin_ip_spec.md); spec kept only so saved chains resolve; execution blocked at `run_module` | RETIRED |
| `sw_height` | Saturation-height | docs/PRD_v2/15_sat-height-rocktyping.md §5 Leverett and Skelt-Harrison parameters; satheight.rs (scal_pc + Leverett-J fit + sw_height) | CITED |

### Permeability

| Module | Title | Derivation source | Status |
|---|---|---|---|
| `perm_wyllie_rose` | Wyllie-Rose family | **Wyllie, M.R.J. & Rose, W.D. (1950)**, *Some Theoretical Considerations Related to the Quantitative Evaluation of the Physical Characteristics of Reservoir Rock from Electrical Log Data*, Trans. AIME **189**, 105–118 — owns the generalized `k^½ = C·φ^D/Swi^E` form. Ported from Geolog Loglan `perm_wyllie_rose.lls`. **Two of the four constant sets are mislabelled by the port** (see below) | CITED |
| `perm_coates` | Coates | **Coates, G.R. & Denoo, S. (1981)**, *The Producibility Answer Product*, Schlumberger Technical Review **29**(2) — the FFI form this module implements. Ported from Geolog Loglan `perm_coates.lls`. NOT Coates & Dumanoir (1974) | CITED |
| `perm_transform` | Por-perm transform | Definitional core-calibrated regression log10(PERM) = PT_A·PHIE + PT_B; PT_A/PT_B are the user's own per-zone RCAL calibration, so there is no external derivation to cite | UTILITY |

### Lithology, rock typing, facies

| Module | Title | Derivation source | Status |
|---|---|---|---|
| `midplot` | MID plot (RHOMAA/UMAA) | Schlumberger chartbook Lith-6 MID plot (vector-digitized overlay) with the chartbook U definition rho_e = (RHOB + 0.1883)/1.0704 stated in the module doc; crossplot-porosity via the digitized Por-11 lookup (lithology.rs) | CITED |
| `rocktyping` | Rock typing (Winland/FZI) | ref_rocktyping_shf.md; Permadi-Susilo exponent re-verification in docs/constants_verification_2026-07-22.md | CITED |
| `lucia_rfn` | Lucia rock-fabric number | Lucia 1995; Jennings & Lucia 2003, SPE 78740 (rocktyping.rs header; global-transform constants A/B/C/D from the paper via ref_rocktyping_shf.md, carrying that doc's own "VERIFY before field release" caveat) | CITED |
| `pittman_rx` | Pittman pore-throat radii | Pittman, E. D., 1992 (rocktyping.rs:287 header; `PITTMAN_TABLE1` holds the published table in full, no coefficients of our own — docs/record_fixes.md) | CITED |
| `rt_cutoff` | Cutoff rock types | ref_rocktyping_shf.md §Cutoff-based electrofacies tie-in (rocktyping.rs header above the spec) | CITED |
| `electrofacies` | K-means electrofacies | Standard k-means (k-means++ seeding, z-scored features); SandiBumi conventions (GR-ordered labels, seeded SplitMix64, best-of-8) documented in facies.rs; K default corroborated against two Techlog modules (parameter source) | CITED |
| `gmm_facies` | GMM facies | Standard Gaussian mixture (EM); same convention set and parameter sourcing as `electrofacies` | CITED |

### Thin beds and unconventional

| Module | Title | Derivation source | Status |
|---|---|---|---|
| `thin_bed_ts` | Thomas-Stieber | Thomas & Stieber, 1975 (modules.rs header above the spec); docs/PRD_v2/17_thinbed-laminated.md | CITED |
| `toc_passey` | TOC (Passey ΔlogR) | Passey (1990) ΔlogR overlay with the 10^(2.297−0.1688·LOM) maturity term, named in the module doc; docs/PRD_v2/19_toc-unconventional.md §5 resistivity/porosity baselines | CITED |
| `kerogen` | Kerogen volume | docs/PRD_v2/19_toc-unconventional.md §5; endpoint corroboration across IP/Techlog/Geolog recorded in the parameter sources | CITED |
| `gip` | Gas in place | docs/PRD_v2/19_toc-unconventional.md SB-TOC-019 and §5 | CITED |
| `brittleness` | Brittleness index | Rickman et al. SPE 115258 (parameter source names the paper); docs/PRD_v2/19_toc-unconventional.md §5 | CITED |

## Named gaps

**None at method level.** Every shipping module traces to a repo-recorded source or falls in a
proposed no-derivation class. What remains open is classification and sufficiency, not missing
sources — the three questions below.

## Open questions for sign-off

1. **Are the Geolog `.lls` ports sufficient as derivation sources of record for
   `perm_wyllie_rose` and `perm_coates`?** If the original papers should be cited beside the
   ports, those citations are Jauhar's to supply (collaboration rule 6) — engineering does not
   add paper citations it cannot trace in-repo.
2. **Accept or reclassify the proposed UTILITY and IN-HOUSE classes** (11 UTILITY rows,
   1 IN-HOUSE row — `bed_detect`). Any row he reclassifies as needing an external source
   becomes a NAMED GAP for him to fill.
3. **`sspw` validation** against the reference-suite LAS exports is still outstanding (noted in
   CLAUDE.md since Phase 8.5). Its derivation source exists; the validation is a separate task.

## What follows the signature

Per DEC-073 item 4 and the SB-DBM-005 design note: only after sign-off does the engineering
half proceed (fail-closed derivation field on `ModuleSpec`, `CurveAncestry` method-derivation
propagation, the T07/T10 arms). SB-DBM-010 stays blocked behind the same signature. Nothing is
built on unsigned content.

#### Permeability constants — open question 1, closed 2026-08-21

Researched on Jauhar's instruction (`D:\XX. Clauding` session `437da72d`, 2026-08-21). Every
constant traces, and **the numbers are not in question** — the arithmetic was independently
verified correct before this. What was wrong is what two of them are CALLED.

The whole table is a unit-convention artefact: every published form is `k = a·φ^b/Swi^c`, while
this module squares instead, so `C = √a`, `D = b/2`, `E = c/2`. That is why 250 and 79 look like
new constants — they are √62500 and √6241.

| Branch | What it actually is | Status |
|---|---|---|
| `TIMUR` (C=100, D=2.25) | **Schlumberger Chart K-3** (`a=10000, b=4.5`) | **Mislabelled.** Timur (1968), SPWLA 9th, Paper X / The Log Analyst 9, published `k = 0.136·φ^4.4/Swi²` in PERCENT = `8581·φ^4.4/Swi²` in fractions, i.e. `C=92.63, D=2.2`. Agreement is within 3% in good rock and about 14% low in tight rock — where a cutoff is decided |
| `MORRIS_BIGGS_OIL` / `_GAS` (250 / 79) | Wyllie-Rose oil and gas constants | **Disputed.** IP 2025 and Techlog attribute them to Morris & Biggs (1967), SPWLA 8th, Paper X; a 2024 *Scientific Reports* paper and several teaching references credit Wyllie & Rose (1950). Balan et al. (SPE 30978, 1995), the most careful comparative review found, omits Morris & Biggs entirely. **The 1967 paper is paywalled and was not read**, so this is vendor convention, not established provenance |
| `TIXIER` (250, 3, 1) | A post-1950 simplification of Wyllie-Rose, oil constant | **Mislabelled.** Tixier (1949), Oil & Gas Journal 48, is a resistivity-gradient method. This is also why `TIXIER` and `MORRIS_BIGGS_OIL` are byte-identical here — the same lineage arrived at twice, which `the_wyllie_rose_variants_carry_their_own_constants_and_two_are_one_equation` already pins |
| generalized `(C·φ^D/Swi^E)²` | Wyllie & Rose (1950), Trans. AIME 189, 105–118 | **Solid.** Carman-Kozeny with irreducible water saturation substituted for specific surface area; their shape factor is 2.0–2.5, suggested as 2.25, and they warned the result carries only order-of-magnitude significance |
| `perm_coates` FFI | Coates & Denoo (1981), Schlumberger Technical Review 29(2) | **Corrected.** The registry named Coates & Dumanoir (1974) — a different, heavier model (a common exponent replacing both `m` and `n`, driven by φ, Rₜ at Swirr, hydrocarbon density and rock class) that this module does not implement |

Lineage review: Balan, Mohaghegh & Ameri, **SPE 30978** (1995),
<http://www.danubianenergy.com/publications/Articole/Balan_SPE_30978.pdf>.

**Three things not resolved, stated rather than papered over.** OnePetro returned 403 for
Morris & Biggs (1967) and Coates & Dumanoir (1974); the Coates & Denoo (1981) Technical Review
is not reachable online in any form; and the two circulating page ranges for Timur's *Log
Analyst* printing disagree. The ids are unchanged because they are stored in `params_json` on
every saved run — the corrections ride on `choice_labels`, the mechanism this file added after
`OPT_GR`'s bare `LARINOV1`/`LARINOV2` sent a user to the wrong coefficient.

**Still open for Jauhar:** whether to add a true `TIMUR_1968` branch at `C=92.63, D=2.2` beside
the chart curve, or to say in the doc that Timur's published form is not offered. The doc
currently says the latter.
