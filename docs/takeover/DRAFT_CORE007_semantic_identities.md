# DRAFT — SB-CORE-007 semantic-identity registry and the two T23 boundary adjudications

**Status: DRAFT, delivered 2026-08-19 under DEC-073.** Engineering drafts the complete
inventory; Jauhar signs it and adjudicates the named questions. Nothing below is authoritative
until signed, and the engineering half (universal T19/T20/T23 without allowlist exemptions)
is not built on unsigned content. Every inventory row was harvested from the shipping catalog
(`modules::module_catalog()`, 52 specs) on 2026-08-19 and verified against the code cited.

The chapter's 2026-08-07 evidence has PARTLY been overtaken by the Gate 2 program and this
draft says which parts: the four-site GR endpoint spread is gone (all producers now ship the
endpoints ABSENT under the shared `GR_CLEAN_ENDPOINT`/`GR_SHALE_ENDPOINT` topics), and the
2.645/2.65 sandstone split is gone (every shipping `RHO_MA` default is 2.65 under
`MATRIX_DENSITY`). Two of its witnesses are STILL LIVE and are in Part D.

---

## Part A — output-mnemonic identity registry (proposed classes)

Eleven mnemonics are declared by more than one catalog module. Proposed classification —
the class tells DEC-051's build-time uniqueness check which declarations it applies to
(Part B2), and decides what the verification tests assert per mnemonic:

### WORKING — an elected working curve; producers are intentionally different methods

Numeric equality MUST NOT apply (forcing Coates equal to Wyllie-Rose at shipped defaults
would be forcing two different methods to agree). What the tests assert instead: each
producer ALSO writes its custody-named output, and run provenance identifies the
producer (both already true — verify, don't rebuild).

| Mnemonic | Producers | Custody names beside it |
|---|---|---|
| `VSH` | vsh_gr, vsh_dn | VSH_GR / VSH_DN |
| `PHIE`, `PHIT` | phi_den, phi_dn, phi_dnbk, phi_son | PHIE_DEN, PHIE_DN, PHIE_DNBK_LIM, PHIE_SON… (DEC-038/DEC-070 govern; ssc/sspw are already custody-only: PHIT_SSC…, PHIT_SSPW…) |
| `SWE` | sw_arch, sw_imts, sw_indo, sw_rtc, sw_sim | per-method custody names |
| `SWT` | sw_arch, sw_height, sw_imts, sw_rtc | per-method custody names |
| `PERM` | perm_wyllie_rose, perm_coates, perm_transform | PERM_WR / PERM_COATES / PERM_XFM |
| `VOL_UWAT` | the five SWE producers | rides the producing Sw method |

### CATEGORICAL — a method flag whose values are intentionally different per producer

| Mnemonic | Producers | Note |
|---|---|---|
| `SW_METHOD` | sw_arch, sw_height, sw_imts, sw_indo, sw_rtc, sw_sim | Its entire purpose is to differ by producer. Exempt from numeric equality BY TYPE, never by allowlist. |

### PLACEHOLDER — a manifest key resolved to an input-derived name before anything is stored

| Mnemonic | Producers | Note |
|---|---|---|
| `OUT_CURVE` | bed_detect, block, normalize | Resolved in ONE place (`workflow::resolve_output_names`) to `<CURVE>_…` names; never collides in storage. The condition family (despike/smooth/clip/fill_gaps/flip) is the same shape. |

### ADJUDICATE — Jauhar's call, options stated, nothing decided here

1. **`FTEMP`** (ftemp_grad, precalc) — the chapter's own 33.1 °C worked example: both
   producers are linear trends; the divergence lives entirely in shipped defaults
   (metric surface/gradient vs one study's feet-based °F fits). Options:
   (a) single producer — retire one module's `FTEMP` (the other keeps a custody name);
   (b) WORKING class — both keep writing `FTEMP`, custody names added
   (`FTEMP_GRAD` / `FTEMP_PC`), provenance identifies the producer, consumers unchanged;
   (c) force the two default sets into agreement (which set wins is itself a source
   decision). `SB-ENV-043` owns the module pair; this row owns the identity.
2. **`VSAND`** (ssc, thin_bed_ts) — same name, arguably different quantities: ssc's is the
   sand fraction of the projected matrix mix; thin_bed_ts's is the non-laminar net-sand
   fraction (1 − VLAM). Options: (a) declare them distinct identities under one name
   (documented, no rename); (b) rename one (e.g. `VSAND_TS`); (c) declare them the same
   quantity — which would put them in WORKING and demand custody names.

## Part B — the two T23 boundaries: one already ruled, one reconciliation to sign

**B1 — the no-execution boundary is ALREADY RULED and is recorded here, not re-asked.**
DEC-051 (2026-08-17, after this row's blocker text was written): the registry check is
*uniqueness of DECLARED DEFAULTS, made by declaration inspection at catalog build beside
`validate_parameter_sources` — never execution* (its constraint 1). That answers the
original question outright: a producer whose required parameter is ABSENT
(`perm_wyllie_rose`/`perm_coates` `SWE_IRR`, every `param_open` producer) is checkable by
construction, because nothing runs. A user-typed name is never checked (constraint 2, the
replace-a-result workflow), the rule is pinned from both sides (constraint 3), and a
same-run clash WARNs and runs with the overwrite named (its second ruling).

**B2 — what DEC-051 leaves genuinely open, and the reconciliation proposed for signature.**
DEC-051's premise was *"every module already carries its own distinct default output names
(`PHIE_DEN`, `PHIE_SON`)"* — true of the CUSTODY names, but the harvest shows nine mnemonics
deliberately declared as defaults by SEVERAL modules (`PHIE`/`PHIT` by four porosity
methods, `VSH` by two, `SWE`/`SWT`/`VOL_UWAT`/`SW_METHOD` by the saturation family, `PERM`
by three, `FTEMP` by two): the working-election pattern the porosity contracts and DEC-070's
pay path are BUILT on. Read literally, DEC-051's uniqueness rule would flag all nine as
registry bugs; read as intended, it needs Part A's classes to say WHICH declarations the
uniqueness applies to. Proposed reconciliation: the output manifest declares its class —
`canonical | working | categorical | placeholder` — and DEC-051's build-time check becomes:
CANONICAL (custody) default names MUST be unique across the catalog; a WORKING declaration
is shared by design and is valid only beside a unique custody-named canonical sibling in
the same manifest (pinned from both sides); CATEGORICAL is likewise shared by type;
PLACEHOLDER must resolve through `workflow::resolve_output_names` and never reach storage
unresolved. Under this reading the current catalog passes with ZERO exemptions and exactly
ONE unresolved case — `FTEMP`, whose two producers share the working name with no custody
siblings (Part A's adjudication 1 decides it). A future module declaring a duplicate
custody name walks into the build failure, not past it.

## Part C — semantic constant identities (topics as identity keys)

`param_sources.rs` topics already group same-quantity parameters across modules, and the
catalog gate already refuses an unsourced default. Proposal: **the topic registry IS the
semantic-constant-identity registry**, and `SB-CORE-T19` becomes *"within one topic, one
default value (or ABSENT), enforced at catalog construction."* Current state, harvested
2026-08-19 (210 parameter rows):

### C1 — already unified under a topic (T19 passes today; the test pins it)

`GR_CLEAN_ENDPOINT`/`GR_SHALE_ENDPOINT` (ABSENT everywhere), `MAX_EFFECTIVE_POROSITY`
(0.3), `DRY_SHALE_DENSITY` (2.70, DEC-071), `HIGH_SHALE_BRANCH_THRESHOLD` (0.95),
`FLUID_DENSITY`/`FORMATION_WATER_DENSITY` (1.0), `SHALE_DENSITY`/`SHALE_NEUTRON_ENDPOINT`
(ABSENT), `ARCHIE_A/M/N` (ABSENT), `CLUSTER_COUNT` (5), plus the shared code constants
`PHIE_FLOOR` and `SEED_DEFAULT` — the shape every row below should land in.

### C2 — same-name twins proposed to JOIN an existing or new topic (mechanical after signature)

| Parameter | Untopic'd sites | Proposed identity |
|---|---|---|
| `A`, `M`, `N` | sw_sim, sw_imts (M/N), gascorr | join `ARCHIE_A`/`ARCHIE_M`/`ARCHIE_N` |
| `RT_SH` | sw_sim | join `SHALE_RESISTIVITY` (sw_indo already carries it) |
| `RHO_FL` | condflag, gascorr, midplot | join `FLUID_DENSITY` |
| `RHO_W` | sw_height | join `FORMATION_WATER_DENSITY` |
| `GR_MA`, `GR_SH`, `NPHI_MA` | ssc | join the endpoint topics vsh_gr/vsh_dn carry |
| `NPHI_SH` | sspw | join `SHALE_NEUTRON_ENDPOINT` |
| `RW`, `RHOG` | sw_imts, sw_rtc (both ABSENT) | new shared topics (formation-water resistivity; gas density) |
| `SWE_IRR` | perm_wyllie_rose, perm_coates, sw_indo, sw_sim (all ABSENT) | one new topic (irreducible effective Sw) |
| `SWT_IRR` | sw_arch, sw_height (ABSENT) | one new topic (irreducible total Sw) |
| `SWIRR_MIN` | ssc, sspw (ABSENT) | one new topic |
| `P_LOW`, `P_HIGH` | gr_normalize, normalize (3/97 both) | one new topic (percentile reference pair, workflow_standards P3/P97) |
| `NPHI_FL`, `RHOB_FL` | ssc, sspw (1.0 both) | join/new flushed-zone-fluid topics |
| `WINDOW` | despike, smooth (ABSENT thickness) | one new topic, or accepted as UTILITY-class untopic'd |

### C3 — genuine adjudications (a value or identity decision, not a tag)

1. **`NPHISR_MAX` 0.40 (phi_dn) vs 1.0 (phi_dnbk), same topic `SHALE_REDUCTION_CLAMP`.**
   Either the basis-corrected form deliberately carries a different clamp ceiling (then the
   topic needs a per-role identity so T19 doesn't flag a deliberate difference forever) or
   one of them drifted. Which — and which value — is a method call, not engineering's.
2. **`K` = 5 (electrofacies/gmm_facies, `CLUSTER_COUNT`) vs `K` = 10 (log_predict, KNN
   neighbour count).** Same NAME, two different quantities. Proposal: declare them DISTINCT
   identities (cluster count vs neighbour count; log_predict's K joins a new
   KNN-neighbour topic) rather than renaming a shipped user-facing parameter — unless he
   prefers the rename.
3. **`RHO_MA` ABSENT (vsh_dn) vs 2.65 (phi_den/phi_dn/gascorr/condflag), same topic
   `MATRIX_DENSITY`.** Reads deliberate — the VSH indicators refuse to invent endpoints
   while porosity ships the 3-way-agreed 2.65 — but "deliberate" is his to confirm, and if
   confirmed the topic carries the role split explicitly so T19 pins it instead of
   tripping on it.

## Part D — T20 duplicated transforms: the live witnesses

Both sit in `ssc.rs`, a PROTECTED file — even the fix is not engineering's to make without a
row-scoped authorization, which is a further reason this row is draft-then-sign.

1. **The eight-transform GR ladder is STILL duplicated verbatim** (`ssc.rs:57-68` against
   the `modules.rs` original), with no test asserting the copies agree. Options:
   (a) authorize collapsing to one shared fn; (b) keep both and add the T20 both-copies
   equality test (fails if either changes alone). Either satisfies the requirement.
2. **The sspw gas weight still runs the 1.6 form** (`ssc.rs:498`:
   `phidi² − 1.6·|phidi² − np²|/2`) that the SSC fix replaced — the header (`:21`) and the
   fixed site's comment (`:185-186`) record that 1.6/2 = 0.8 per side overshoots the
   midpoint and inverts the D-N crossover, worth ~4.7 p.u. of porosity in gas. The
   chapter's "duplication defeats the fix" witness is live. Options: (a) authorize applying
   the documented fix to the sspw twin (then T20 pins the two against each other);
   (b) rule the sspw form deliberate (then docs/method_ssc_sspw.md must say so and the
   difference gets its own pinned test). Method math on the flagship module — entirely his
   call, and the reference-suite LAS validation noted since Phase 8.5 would adjudicate it
   empirically.

## What follows the signature

Implement T19 (per-topic single default, enforced at catalog construction), T20 (equality
tests over every surviving deliberate duplicate), and T23 in its DEC-051 form (build-time
declaration inspection with Part B2's class-scoped uniqueness, plus the same-run WARN) —
universally, classes derived from manifests, no allowlists. `ssc.rs` changes additionally
need their own row-scoped authorization.
