# IP2018 CHM ingest — discrepancies & resolutions

Running list of internal contradictions found in the IP 2018 help manual, and how each was
resolved. Rule: nothing here is resolved by preference or by textbook knowledge — only by
evidence internal to the manual, or left OPEN.

---

## D-01 — `Clip Low %` default: 0% or 98%? — **RESOLVED: Low = 0%, High = 98%**

**Conflict**

| Source page | Clip Low % | Clip High % |
|---|---|---|
| `clayparameters.htm` — numbered parameter reference, entries (59) and (60) | `Default is (0%)` | `Default is (98%)` |
| `basicloganalysis.htm` — narrative parameter table | `Default is (98%)` | *no default stated* |

**Resolution — documentation defect in `basicloganalysis.htm`.**

The `basicloganalysis.htm` **Clip Low %** entry terminates with:

> "This allows a percentile of say 130% to be calculated with the removal of spikes in the data before the 130% is calculated."

That sentence is verbatim the trailing sentence of the **Clip High %** entry in
`clayparameters.htm`. The Clip High text block was duplicated onto the Clip Low row,
carrying its `98%` default with it — which is also why the Clip High row on
`basicloganalysis.htm` is left with **no default at all** (the orphan).

`clayparameters.htm` is the structured, numbered parameter reference and is internally
consistent. Its pairing (low = 0 %, high = 98 %) is also the only coherent one: the clip
exists to strip high-side GR spikes before percentile picking, so the low bound sits at the
bottom of the range and the high bound just under the top.

**Adopted:** `Clip Low % = 0`, `Clip High % = 98`, cited to `clayparameters.htm` (59)/(60).

**Why this one mattered.** Adopting the erroneous `Clip Low = 98%` would discard 98 % of the
gamma-ray population from below before percentile picking. It would not error, would not
look wrong on a plot, and would silently corrupt every percentile-derived clay endpoint
downstream — the exact silent-wrongness failure mode.

---

## D-02 — Hingle plot Y-axis definition stated two ways on one page — **OPEN**

Reported by the clay-volume extraction pass: `basicloganalysis.htm` describes the Hingle
plot Y-axis in two inconsistent ways within the same page. Not yet adjudicated; no internal
evidence identified that settles it.

**Do not adopt a Hingle axis convention from this manual until resolved.** Resolve against
the primary literature, not against IP's wording.

---

## D-03 — Vendor's own spelling is inconsistent: `Stieber` vs `Steiber` — **NOTED, not a defect**

- `clayparameters.htm`, `clayequationsandmethodology.htm` → **Stieber** ("As per Stieber et al
  (South Louisiana Miocene and Pliocene)")
- `swparameters.htm` → **Thomas Steiber** (in the laminated-shale / `Res Lam Shale` context)

Both spellings occur in the wild in the industry. Relevant only to alias/catalog matching —
any mnemonic or method-name lookup must accept both. No numeric consequence.

---

## D-04 — `RQI` defined two ways in one product — **RESOLVED: not a defect, a naming collision**

| Source page | Definition |
|---|---|
| `hfu.htm` — Hydraulic Flow Units, RQI/FZI method | `RQI = 0.0314 x Sqrt( K / Phi )` |
| `logswversusheightfunctions.htm` — Log Sw vs Height, "Rock Quality Index" method | `Sw = f(RQI.h) & RQI = √ ( Κ / ϕ )` — **no constant** |

Both are stated deliberately, on different pages, for different modules. The HFU form is the
Amaefule-style unit-bearing flow-zone quantity (the 0.0314 carries the µm conversion so FZI
lands in microns); the log-Sw-vs-height form is a bare shape term feeding a regression that
absorbs any scaling into its own fitted coefficients, so the constant is redundant there.

**Adopted:** both, namespaced. Do not unify them. An implementation that shares one `rqi()`
between the rock-typing and SHF paths silently rescales every fitted coefficient in one of
them — and produces a plausible curve either way.

---

## D-05 — Pc ↔ height stated with and without the `0.433` gradient — **RESOLVED: 0.433 form is computational**

| Source page | Statement |
|---|---|
| `cappressuresetup.htm` (Pc Height output) | `Height = Pc / 0.433 (ρWater - ρHc)` |
| `saturation_versus_height_curve.htm` (Run All Wells) | `Pc = h * 0.433 (ρWater - ρHc) * IFTCorrFactor` |
| `cappressurefunctions.htm` **and** `logswversusheightfunctions.htm` (Function XPlot format panel) | `Height above FWL = Pc / (Water Density ? Hydrocarbon Density)` — **no 0.433** |

**Resolution — the XPlot panel wording is a documentation shorthand.** Three lines of
internal evidence:

1. Only the 0.433 form carries a derivation — `cappressuresetup.htm` spends a full section
   explaining where `g` went ("The gradient of fresh water is 0.433 psi/ft for a density of
   1 g/cc and standard gravity g").
2. It is the form given in the actual compute path (Run All Wells), including the
   gas-over-oil two-component variant.
3. The module's own printed report reproduces it. Given `Hyd. Density : 0.2 gm/cc`,
   `Water Density : 1. gm/cc`, `IFT Corr. Factor : 1.14`, IP prints
   `Pc = (3128.8 - TVDSS) * 1.29659 psi`. With well depths in **metres**
   (0.433 psi/ft ≈ 1.4206 psi/m): 1.4206 × (1.0 − 0.2) × 1.14 = **1.2956**, against IP's
   printed 1.29659 — agreement to ~0.08 %. The no-constant form cannot reproduce it at all.

**Adopted:** `0.433 psi/ft`, and the conversion must be depth-unit aware (the report example
is only consistent in metres).

**OPEN sub-item.** Back-solving that same example gives an implied gradient of ≈ `0.4333`
psi/ft rather than `0.433`, so IP may carry more digits internally than it documents. The
manual states only `0.433`; that is what is recorded and adopted. Do not "improve" it to a
remembered value such as 0.4335 — if the extra digit matters for a deliverable, read it off
the live IP UI.

---

## D-06 — Pore-size array starts at `0.01 µm` or `0.1 µm`? — **RESOLVED by the manual's own arithmetic: 0.01 µm**

| Source page (both `cappressuresetup.htm`) | Statement |
|---|---|
| `Pore Size Curve Out` | "X dimension of **80** … first X value is a pore size of **0.01 microns** and the 80th X value is **100 microns**. The scale is logarithmic with **20 X values per decade**." |
| `Throat Size` | "array curve of **80** elements … first value is **0.1** (microns) last value **100** (microns) … logarithmically spaced" |

**Resolution.** The Pore Size spec is self-consistent and over-determined: 80 elements at
20 per decade spans exactly 4 decades, and 0.01 → 100 µm *is* 4 decades. The Throat Size
statement shares the 80-element count and the 100 µm top but a 0.1 → 100 start spans only
3 decades, which would require ~26.7 elements per decade. Only `0.01` satisfies every
number the manual states.

**Adopted:** `0.01 µm` for the 80-element, 20-per-decade log array. Low stakes — this is a
display/array bound, not a petrophysical parameter — but recorded so the next reader does
not re-litigate it.

---

## D-07 — Clay-bound-water correction factor `F` has unbalanced brackets — **OPEN**

`cappressuresetup.htm` states the Hill/Shirley/Klein 1979 clay correction factor as, verbatim:

```
F = 1 - [0.6425 * ( Salinity ^ (-0.5) + 0.22 ] * Qv ]
```

Three `]`, one `[`, one unmatched `(`. The expression as written does not parse, so at least
one bracket is wrong — but the manual gives no second statement of it and no worked numeric
example, so **there is no internal evidence to adjudicate which**.

**Do not implement this correction from the manual text, and do not repair the brackets by
pattern-matching to a remembered form of the Hill-Shirley-Klein equation.** Resolve against
the rendered help page (the decompile may have dropped a glyph) or against SPWLA 20th Annual
Symposium Paper AA directly.

Associated unit traps, which the manual *does* state unambiguously and which are the usual
source of silent error here: `Salinity` is in **Kppm** NaCl equivalent (not ppm), and `Qv` is
in **meq/ml** (not meq/L).

---

## Cross-check still owed

- IP2018 mineral endpoints vs IP2025 `MINDEF.PAR` vs Techlog `QM_MineralTable` vs SandiMin —
  a four-way comparison once the mineral-solver pass lands. Treat divergence as
  "independent libraries" until proven otherwise; the known open item is the SandiMin
  smectite density review (dry-grain vs wet-clay convention).
