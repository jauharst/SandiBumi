# Source inventories for review — drafted 2026-08-17

Three inventories Jauhar asked for on 2026-08-17, to unblock `SB-DBM-005`, `SB-DIO-011` and
`SB-DIO-057`. **These are drafts assembled from the repository, for his approval — not adopted.**

The discipline throughout: **report what the repo says, and be exhaustive about what it does not.**
No citation is supplied from general knowledge. Where a value or alias obviously matches a
well-known convention but the repo states no source, that is recorded as a **GAP**, because a
plausible attribution nobody checks is worse than an honest blank.

---

## 1. `SB-DIO-011` — deviation-survey column aliases

Source: three hard-coded `const` arrays at `src-tauri/src/parsers.rs:2807-2809`. Header matching is
**case-insensitive** — headers are uppercased at `parsers.rs:2816` before comparison.

| Role | Accepted aliases (verbatim) | Repo-stated source |
|---|---|---|
| Measured depth | `MD`, `DEPTH`, `DEPT`, `MEASURED_DEPTH` | **GAP** — no comment on any |
| Inclination | `INC`, `INCL`, `INCLINATION`, `DEVI` | **GAP** — no comment on any |
| Azimuth | `AZI`, `AZIM`, `AZIMUTH`, `HAZI`, `AZM` | **GAP** — no comment on any |
| TVD | *(none accepted)* | n/a |

**Findings for Jauhar to rule on.**

1. **All 13 aliases are uncited.** Not one carries a comment saying which vendor or format uses it.
   That is the whole of `SB-DIO-011`: the requirement asks for a named source per accepted alias.
   The fix is not research — it is Jauhar confirming which of these he has actually received in a
   delivery, and striking any he has not. **An alias nobody has seen in the wild is a liability, not
   a convenience**: it can only ever match a column it was not meant to match.
2. **`DEVI` deserves a second look.** It is the one inclination alias that is not an abbreviation of
   "inclination". If it comes from a specific vendor's export, that vendor is the citation.
3. **No TVD alias exists at all.** A survey file delivering a TVD column has it ignored. Whether
   that is correct — TVD is normally *computed* from MD/INC/AZI by minimum curvature, so accepting a
   delivered one could silently override the computed path — is a method decision, not an omission
   to fix by reflex. Recorded here because the inventory revealed it, not because the row asked.

---

## 2. `SB-DIO-057` — curve families and which are log-scale

Source: `FAMILIES` at `src-tauri/src/generated/unit_registry.rs:22`. Note `curves.rs` **consumes**
this table but does not define it.

| Family | Example mnemonics | Log-scale evidence in the repo |
|---|---|---|
| `RES_DEEP` | `RES_DEEP`, `RESD`, `RT`, `RDEEP`, `RDEP`, `DRES` | **YES** — `layout.rs:385`, `:434`, `:521` `ScaleType::Log` |
| `RES_MED` | `RES_MED`, `RESM`, `RMED`, `ILM`, `LLM`, `AT30` | none found |
| `RES_SHAL` | `RES_SHAL`, `RESS`, `RSHAL`, `SFL`, `SFLU`, `LL8` | none found |
| `RXO` | `RXO`, `RXOZ`, `MSFL`, `RMLL` | none found |
| `GR` | `GR`, `GRN`, `GRD`, `CGR`, `SGR`, `GRGC` | none found |
| `SP` | `SP`, `SPC`, `SPR` | none found |
| `CALI` | `CALI`, `CAL`, `CALS`, `CALX`, `CALY`, `HCAL` | none found |
| `BS` | `BS`, `BITSIZE`, `BIT` | none found |
| `RHOB` | `RHOB`, `RHOZ`, `DEN`, `ZDEN`, `ROBB` | none found |
| `DRHO` | `DRHO`, `HDRA`, `ZCOR`, `DCOR` | none found |
| `PEF` | `PEF`, `PE`, `PEFZ`, `PEB`, `PDPE` | none found |
| `NPHI` | `NPHI`, `TNPH`, `NPOR`, `NEUT` | none found |
| `POR` | `PHIE`, `PHIT`, `PHIA`, `DPHI` | none found |
| `VSH` | `VSH`, `VSH_NMR`, `VDSH`, `VSHGR`, `VSHND` | none found |
| `VSH_UNCLIPPED` | `VSH_GR`, `VSH_DN`, `VSH_DS`, `VSH_RES` | none found |
| `VCL` | `VCL`, `VCLAV`, `VCLMIX`, `VOL_CLAY` | none found |
| `CLY_STATE` | `MTH_VSH`, `VSH_DN_FLAG` | none found |
| `DT` | `DT`, `DTC`, `DTCO`, `AC`, `DTP` | none found |
| `DTS` | `DTS`, `DTSM`, `DTSH`, `DT_S` | none found |
| `TEMP` | `FTEMP` | none found |

**Findings for Jauhar to rule on.**

1. **`PERM` is not a registered family at all** — zero occurrences in `unit_registry.rs` or
   `curves.rs` — yet `layout.rs:478` draws it on a `ScaleType::Log` track. Permeability is the most
   obviously log-scale quantity in the suite and the rule this row exists for is *"a zero on a
   log-scale curve is not a reading."* **A family the registry does not know cannot be protected by
   that rule.** This is the single most important thing the inventory turned up.
2. **Only `RES_DEEP` carries log evidence, and it comes from a LAYOUT, not the family table.**
   `RES_MED`, `RES_SHAL` and `RXO` are the same measurement at different depths of investigation and
   almost certainly belong with it — but the repo does not say so, and **that classification is
   Jauhar's to make, not mine.** A display choice in one built-in layout is not a declaration that a
   family is logarithmic; the row asks for the latter.
3. **There is a second family list.** `registration.rs:83` defines `CORE_FAMILIES`. Whether it must
   agree with `FAMILIES` is unexamined; two lists that can disagree are the shape of a silent bug.

---

## 3. `SB-DBM-005` — registered-module derivation-source map

**51 registered modules** across 13 files, from `module_catalog()` in `modules.rs`. Note that
`thomeer.rs`, `hfu.rs`, `reframe.rs` and `multimin2.rs` (SandiMin) define no `_spec()` and are **not
reachable through `list_modules()`** — they are separate subsystems, not registered modules, so they
carry no row here.

The gaps are not 26 separate research problems. **They fall into three kinds, and only one of them
is real work.**

### Kind 1 — cited, nothing needed (21 modules)

`ssc` (Kuttan et al., 21st SPWLA + Jauhar's Loglan) · `sspw` (Jauhar's spec, flagged reconstructed
and unvalidated) · `nphimat` (SLB charts Por-4/Por-5) · `gascorr` (Standing; Papay 1968) ·
`sw_arch` (Archie 1942) · `sw_indo` (Poupon & Leveaux 1971) · `sw_sim` (Simandoux 1963; Bardon &
Pied 1969) · `sw_rtc` and `sw_imts` (Jauhar's LRLC study; Waxman & Smits 1968, Waxman & Thomas
1974, Juhasz 1979/1981) · `thin_bed_ts` (Thomas & Stieber 1975) · `log_predict` (Geolog Facimage
MRGC) · `sw_height` (Leverett 1941; Skelt & Harrison 1995) · `midplot` (SLB Lith-6) · `rocktyping`
(Amaefule 1993; Kolodzie 1980; Corbett & Potter 2004) · `lucia_rfn` (Lucia 1995; Jennings & Lucia
2003, SPE 78740) · `pittman_rx` (Pittman 1992, AAPG Bull. v76 — verified against the paper) ·
`toc_passey`, `kerogen`, `gip`, `brittleness` (Passey, Schmoker & Hester, Langmuir, Rickman et al.,
Jarvie, Wang & Gale) · `gr_normalize` and `normalize` (SandiBumi's own documented house preset —
internal, but a stated source).

Three of these carry the repo's **own** warnings and should not be read as clean: `sspw` is
reconstructed from spec and unvalidated; `lucia_rfn`'s transcribed constants are flagged unverified
against the primary paper; `sw_sim` discloses that the shipped `a = 0.8` is not attributed to
either cited paper.

### Kind 2 — generic utilities where no external method applies (8 modules)

`depth_shift` · `splice` · `clip` · `fill_gaps` · `flip` · `perm_transform` (a regression form meant
to be fitted from the user's own core) · `block` · `bed_detect`.

**These need no citation and should be recorded as such**, not left looking like gaps. A depth-shift
utility has no author. The honest source string is "mechanical operation, no external method".

### Kind 3 — the real work: classic methods named but never cited (17 modules)

This is the list that needs Jauhar. In every case the repo **names** the method in a comment but
cites no publication, so a reader cannot check the implementation against a source.

| Module | Method named in code | What is missing |
|---|---|---|
| `vsh_gr` | Larionov 1969, Stieber 1970, Clavier 1971 | No primary publication for any; `LARINOV3` is stated by coefficients only, with no attribution at all |
| `vsh_dn` | N-D crossplot VSH | Endpoints cite vendor tables; the technique itself cites nothing |
| `phi_den` | Density porosity | Same — endpoints sourced, equation not |
| `phi_son` | Wyllie time-average, Raymer-Hunt-Gardner, Hilchie `Cp` | No year or journal for any of the three |
| `phimax` | Athy-type exponential compaction | No citation, only an internal PRD section reference |
| `precalc` | Arps | Named with no year |
| `perm_wyllie_rose` | Timur, Morris-Biggs, Tixier | A "Western Atlas chartbook" attribution exists **only in the gitignored research corpus**, not in the module |
| `perm_coates` | Coates | Only the Geolog Loglan file it was ported from |
| `despike` | Hampel identifier | Method uncited; `K = 3.0` self-declared a convention |
| `smooth` | Savitzky-Golay | No publication cited |
| `electrofacies`, `gmm_facies` | k-means, Gaussian mixture / EM | Algorithms never attributed; only the cluster-count default is sourced |
| `badhole`, `gr_hole_corr`, `nphi_env_corr`, `rhob_hole_corr` | linear hole-size / environmental corrections | No chart number, vendor document or year — `nphi_env_corr` says only "the applicable CNL chart" |
| `rt_cutoff` | Vsh/PHIE cutoff ladder | Presented as SandiBumi's own construction; no external method |

**Findings for Jauhar to rule on.**

1. **Most of Kind 3 are textbook methods with well-known primary papers.** I can propose the
   citations for his approval — I will not adopt one unilaterally, because a wrong attribution on a
   physics method is exactly the silent failure this register exists to prevent. `condflag`'s coal
   thresholds are already closed by `DEC-058` and `despike`'s window floor by `DEC-059`.
2. **`perm_wyllie_rose` is the sharpest provenance issue here.** The only attribution anywhere is in
   `docs/research_2026-08/` — which is **gitignored**, so the shipped module has no source at all in
   the distributed repository. A licensed product shipping the Timur and Morris-Biggs coefficient
   sets with no citation in-tree is precisely the `IP_PROVENANCE.md` §2 problem.
3. **`vsh_gr`'s `LARINOV3` has no attribution even in a comment** — it is stated by its coefficients.
   That is the one entry here where the repo does not even claim to know what it implements.

---

## What is asked of Jauhar

Nothing in this file is adopted. For each list: confirm what is real, strike what is not, and name a
source for what survives. Where his answer is *"my own experience, no published reference"* — as it
was for the conditioning thresholds on 2026-08-17 (`DEC-059`) — that is a legitimate and sufficient
source, recorded as exactly that.
