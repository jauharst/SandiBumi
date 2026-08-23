# Two spellings, and what admitting them does — 2026-08-23

**DEC-098.** Jauhar ruled on the two mnemonics `PERF-FIELD-FIXTURE-2026-08-23.md` §3 reported and
deliberately did not decide:

> "for grn cs yes to gr, and for nphi to nphi"

- **`GRN_CS` joins family `GR`.** `_CS` is a vendor suffix on a gamma curve. It previously belonged
  to no family at all.
- **`NPHI_COR` moves from family `POR` to family `NPHI`.** A corrected neutron log is a neutron
  measurement. It previously sat in the porosity bucket among SandiBumi's own output names
  (`PHIE_SSC`, `PHIFF_SSPW`, `PHIE_DN`), which is where an app-produced curve belongs.

The whole change is three alias entries in `registry/unit-registry.json` plus a regeneration of its
four consumers. **No Rust logic changed.** The reason this document exists anyway is that a
dictionary edit is the cheapest possible way to move an interpretation without anyone noticing, so
what it can and cannot reach is written down.

## 1. What it changes

Both effects run through **one function**, `equations::fetch_curve_frame`. Its generic-store
fallback resolves a requested name by **mnemonic OR family**
(`resolve_generic_curve_decision(.., CurveRequest::SemanticFamily)`), so a name that reaches a
family reaches everything built on that function:

- **Module inputs.** A module run with its manifest defaults asks for `GR` and `NPHI`. On a
  delivery spelled this way, both now resolve. Before, both refused by name and every module had to
  be pointed at the two curves by hand, every run. Counted: **5 modules declare `log_in("NPHI", …)`
  and 2 declare `log_in("GR", …)`** (`grep -c 'log_in("NPHI"' src-tauri/src/modules.rs` → 5;
  same for `"GR"` → 2).
- **Log-view tracks.** `equations::fetch_track_frames` sends an unqualified track name through the
  same function, so a track added as `GR` now draws on these wells instead of drawing nothing.
- **The curve catalog** shows the family, and the unit label canonicalizes to the family's spelling
  (`GAPI` → `gAPI`).

### Measured on the real delivery, not deduced

`pipeline_field_100well_stress` carries a probe that asks what a module resolves with its manifest
defaults and **nothing bound**. Run against the configured field fixture, release profile, with the
dictionary reverted and then restored — the only difference between the two runs:

```
before   vsh_gr with DEFAULT inputs, nothing bound: missing GR=GR
after    vsh_gr with DEFAULT inputs, nothing bound: missing nothing
```

Both runs print the same line above it, and it is the other half of the story:

```
source well: 1562 samples; finite in the six standard columns: GR=0 RES_DEEP=1178 NPHI=0 RHOB=1373 DT=0 SP=0
source well: 7 generic-store curves (CALI DRES FTEMP GRN_CS NPHI_COR PEF RHOB), depth unit FT
```

**The standard GR column is still empty after the change**, exactly as §3 says it must be. The
module resolves anyway, because the curve was in the generic store the whole time and only the
family was missing.

## 2. What it cannot change, and why that is checkable rather than reassuring

**No stored or displayed value moves.** `NPHI` and `POR` are the *same quantity in the same unit*:
both are `canonical_unit: "v/v"`, both `QuantityKind::Fraction`, and all three conversion rules
that name one name the other with identical arithmetic (`%`, `pu`, `p.u.` → ×0.01). Re-filing the
spelling changes which requests find the curve. It cannot change what the curve says.

That is pinned by `re_filing_a_spelling_between_two_fraction_families_cannot_change_a_stored_value`,
which converts the same value under both families across six unit spellings and asserts the results
and the applied/not-applied flags agree. **If a rule is ever added to one family and not the other,
that test fails** — which is exactly the moment this paragraph would stop being true.

Gamma has no conversion at all: `GR` is absent from `CONVERTIBLE_FAMILIES`, so admitting `GRN_CS`
normalizes a unit *label* and never a number.

## 3. What it deliberately does NOT touch

**The standard six columns are filled from a different table.** `parsers.rs` populates
`standard_curves` from `GR_ALIASES`, `RES_ALIASES`, `NPHI_ALIASES`, `RHOB_ALIASES`, `DT_ALIASES`
and `SP_ALIASES` — a separate list from `curves::FAMILIES`, and neither spelling is in it
(`GR_ALIASES = ["GR", "GRN", "SGR"]`; `NPHI_ALIASES` carries eleven entries, none of them
`NPHI_COR`).

So after this change, on such a delivery:

| | before | after |
|---|---|---|
| `standard_curves.gr` column | empty | **still empty** |
| `standard_curves.nphi` column | empty | **still empty** |
| module asking for `GR` / `NPHI` | refuses by name | **resolves** |
| log track added as `GR` | draws nothing | **draws** |

The second column is not a bug — it is the reason the first column being empty stopped mattering.
`fetch_curve_frame` treats an all-NaN standard column as an absence and falls through to the
generic store (`equations.rs:613`, `:625`), which is where the curve has been all along.

**Whether the parser lists should also learn these spellings is a separate decision and was not
made here.** It is a different question with a different consequence: those lists decide which
curve becomes *the* gamma of a well in the standard store, so admitting a spelling there elects one
channel over any other the file carries. That is worth asking about on its own terms rather than
folding into a family ruling.

## 4. The risk, recorded rather than argued away

Both spellings were observed on **one delivery**. The comment that recorded them said admitting
either "would turn one delivery's convention into an automatic interpretation for every import" —
a caution written by the test, which is the wrong body to settle it. It was put to Jauhar with that
framing and overridden.

The residual risk is real and worth stating plainly: if some other vendor ships `NPHI_COR` meaning
a computed porosity, SandiBumi now reads it as a neutron log on that delivery too, and the curve
will look entirely normal either way. The mitigation is the same one every alias in this table
relies on — the alias was admitted **by name**, so the blast radius is exactly one spelling, and
`grn_cs_is_a_gamma_curve_and_the_spelling_was_admitted_by_name_rather_than_by_wildcard` fails if
anyone later widens it to a pattern.

## 5. Verification

Two decision pins plus the safety property, all in `curves.rs`, **mutation-verified from both
sides**:

| mutation | caught by |
|---|---|
| `GRN_CS` reached by a `GRN*` wildcard instead of by name | the second half of the GR test (`GRN_XX must still resolve to nothing`) — the *first* half passed under the wildcard |
| `NPHI_COR` filed back under `POR` | the first assertion of the neutron test |
| `NPHI_COR` listed under **both** families | the generator itself, before any test runs: `unit registry: duplicate family alias 'NPHI_COR'` |

The third is worth noting: that mutation is not merely caught, it is **inexpressible** — the
registry cannot represent one spelling in two families. The test's "exactly one family may claim
the spelling" assertion is therefore belt-and-braces over a generator rule, and is kept because it
is the half that says *which* family, which the generator has no opinion about.
