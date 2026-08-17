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

**Pending.** The inventory is still being assembled and will be added here.

---

## What is asked of Jauhar

Nothing in this file is adopted. For each list: confirm what is real, strike what is not, and name a
source for what survives. Where his answer is *"my own experience, no published reference"* — as it
was for the conditioning thresholds on 2026-08-17 (`DEC-059`) — that is a legitimate and sufficient
source, recorded as exactly that.
