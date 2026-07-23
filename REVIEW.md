# Review checklist — for Jauhar's click-through in `npm run tauri dev`

Everything below is implemented, unit/integration-tested, and browser-smoke-tested,
but has **not** been clicked through in the real desktop app with real field data.
Work through this list when you have time, marking items as you go.
Marks: **`[x]` = confirmed done** (works as described); `[ ]` = not yet checked. If something is
**wrong**, tell me directly (like your 540-well notes) and I'll fix it and log it in
**ROADMAP.md §4 (Field-review backlog)**.

## Round 28 — Unconventional #7 inc 1: TOC from Passey ΔlogR + Schmoker (2026-07-23)

First increment of the unconventional / shale suite (playbook Part II #7). A new **Unconventional**
group on the Petrophysics ribbon, with its first module: **TOC — Passey ΔlogR + Schmoker**. It
estimates total organic carbon two independent ways:

- **Passey (1990) ΔlogR** — the separation between deep resistivity and a *baselined* porosity curve.
  Choose the **overlay**: *sonic* (`ΔlogR = log10(R/R_base) + 0.02·(DT−DT_base)`) or *density*
  (`−2.5·(RHOB−RHOB_base)`). Set the baselines (`R_BASE`, `DT_BASE`/`RHOB_BASE`) on a clean, clay-rich,
  **non-source** interval where the two curves overlie (ΔlogR≈0), then
  `TOC = ΔlogR·10^(2.297−0.1688·LOM) + background`. LOM (maturity, 6..12) defaults to 10.6.
- **Schmoker-Hester (1983)** density-TOC `154.497/RHOB − 57.261` as an independent cross-check
  (writes `TOC_SCHMOKER` whenever a density curve is present, regardless of overlay).

Outputs: `DLOGR` (the raw separation, for the overlay panel coming in inc 5), `TOC` (Passey, wt%),
`TOC_SCHMOKER` (density cross-check). In non-source rock (ΔlogR<0) TOC floors to the *background*
value, not below it. Tier-B, cited in code (Passey et al. 1990; Schmoker & Hester 1983); the LOM and
baseline defaults are Tier-A IP seeds, per-well overridable. The **neutron** overlay is deferred — its
sign convention is inconsistent across the literature and needs core verification. Method math banked
in `docs/ref_unconventional.md` §1.

Verified: **311 cargo tests** (7 new — sonic/density overlays recover a known TOC, TOC decreases with
LOM, non-source floors to background, missing overlay curve falls back to Schmoker) + tsc green +
adversarial review (found & fixed one clamp-order defect pre-commit: a nonzero background must be the
floor, not zero). Additive — nothing existing moves.

> **Try:** open **Petrophysics → Unconventional → TOC — Passey ΔlogR + Schmoker**. Set **overlay =
> sonic**, **RES** = deep resistivity, **DT** = sonic. On a clean *non-source* bed read R and Δt and
> enter them as **R_BASE** / **DT_BASE** (so ΔlogR≈0 there); set **LOM** from your Ro/Tmax (or leave
> 10.6) and Run. Confirm **TOC** rises through the organic-rich section, and compare against
> **TOC_SCHMOKER** where RHOB exists. If you have core TOC, nudge LOM until the Passey curve matches.

## Round 27 — SandiMin per-depth formation temperature (FTEMP curve) (2026-07-23)

Formation temperature can now come from a **per-depth curve** instead of one fixed number. On the
**Fluids** tab there's a new **FTEMP curve (opt)** box next to *Formation temp (°F)*. Leave it blank to
use the fixed value (unchanged). Type a curve name (e.g. **FTEMP_F**, the curve Prep builds from a
gradient/BHT) and, for every depth where that curve is finite, SandiMin recomputes the temperature-
dependent quantities at that sample's temperature:

- **Cw / Cmf / Cbw** (formation-water, filtrate and clay-bound-water conductivities),
- the **auto CT/CXO uncertainties**,
- the **clay bound-water tie** (BNDWAT multiplier k, via t_c),
- the **Waxman-Smits B(T,Rw)**.

The α (diffuse-layer) expansion and salinities come from the *Rw/Rmf* sample temperatures, so they don't
move with formation temperature — only the conductivities do. A sample where the curve is missing or
out of range (a null like ±999.25, or anything outside 32–600 °F) quietly falls back to the fixed °F, so
selecting the curve is safe even on wells that lack it. With the box blank the solve is **byte-for-byte
identical** to before (a test pins that a constant FTEMP curve equal to the fixed value reproduces the
fixed-temperature run exactly), and the per-tool reconstruction-QC curves stay consistent under a curve.

> **Try:** run **Prep** so a **FTEMP_F** curve exists (or import one), then open **SandiMin → Fluids**,
> put **FTEMP_F** in **FTEMP curve (opt)**, and Run. Compare **SWE** with and without the curve over a
> long interval with a real geothermal gradient — the hotter, deeper section reads a bit lower Sw (hotter
> water is more conductive). Blank the box to confirm you get the fixed-temperature numbers back.

## Round 26 — SandiMin Waxman-Smits saturation model (2026-07-23)

The last of the Sw models. **Waxman-Smits (B·Qv)** joins the **Sw model** dropdown (Fluids tab). Like the
other post-solve forms it runs the mineral inversion untouched, then replaces the water/HC split from the
deep resistivity — here via `Ct = φt^m·(Cw·Swt^n + B·Qv·Swt^(n−1))`:

- **Qv** is built from the **solved clay volumes**: `Qv = Σ v_clay·CEC·ρ_clay / φt` (meq/mL). So each clay's
  **CEC** (Clay tab) drives the excess conductivity — a clean sand (no clay ⇒ Qv=0) collapses to Archie.
- **B** is the counterion conductance from the **Juhász (1981) B(T,Rw) fit** — the same closed form Techlog
  and IP use — computed from formation temperature and Rw automatically. Because that fit is known to
  overshoot above ~120 °C, a **B override (0 = auto)** box (shown only for this model) lets you pin a
  core-measured B.
- Uses your **m/n as m\*/n\***. PHIE/PHIT stay exactly as the mineral solve made them; only SWE/SWT/SXOT move.

Verified: the conductivity root and the B(T,Rw) fit are hand-anchored in unit tests (n=2 closed form, n=3
bisection, Qv=0/B=0 → Archie, B(25 °C,0.1)=3.895, B(100 °C,0.05)=15.51, monotonic in T and Rw), plus a
full-run integration test that recovers a known Sw. Nothing else moves — the default model is still linear
dual-water.

> **Try:** open **Petrophysics → SandiMin**, **Fluids** tab, set **Sw model → Waxman-Smits (B·Qv)**. Make sure
> a **CT** (deep-resistivity) tool and a **U-zone hydrocarbon** component are set, and that your clays carry a
> **CEC** (Clay tab). Run and compare **SWE** vs **Archie** (Waxman-Smits reads lower on shaly intervals) and
> vs **Juhász**. Leave **B override** at 0 for the auto B(T,Rw); enter a core B to pin it and re-run.

## Round 25 — SandiMin Constraints tab: porosity source + program-constraint toggles (2026-07-23)

The UI for item B (your image 2). A new **Constraints** tab (after Clay) holds two things:

- **Porosity source** radio — **Cation Exchange Capacity** (default) vs **Wet Clay Porosity**. This picks
  what drives the clay bound-water tie: CEC uses `α·96·CEC·ρ/(T+298)`; WCP uses the geometric `k = φ/(1−φ)`
  from a **per-clay φ editor** now on the **Clay** tab (pre-filled with Techlog WCLP defaults — Illite 0.104,
  Kaolinite 0.058, etc.). Running the dry-clay converter also fills a clay's φ, so the two stay consistent.
- **Program constraints** — enable toggles for **UNITY**, **POROSITY**, **X&U BNDWAT**, **WATER MUD**, plus a
  **Constraint tolerance σ** (default 0.01). All four already ran in the solver; this exposes them. UNITY moved
  here from the run footer (there's no longer a "Hard unity" box down by Run).

Defaults are unchanged behavior: CEC, all four on, σ=0.01 — so an untouched Constraints tab solves exactly as
before (a backend test pins that "absent request fields = on"). WATER MUD defaults on for water-based mud (it
keeps flushed-zone water ≥ virgin water; ignored for OBM) — tell me if you'd rather it default off.

> **Try:** open **Petrophysics → SandiMin**, click the **Constraints** tab. Flip **Porosity source** to
> **Wet Clay Porosity**, check the **Clay** tab's per-clay φ list, then Run and compare **PHIE/SWE** vs CEC
> (WCP moves PHIE for clays). Toggle a constraint off (e.g. **WATER MUD**) or change **σ** and re-run to see
> the effect. Confirm the run footer has **no** "Hard unity" box (it's now the UNITY toggle on this tab).

## Round 24 — SandiMin Wet-Clay-Porosity bound-water source (backend) (2026-07-23)

Starting item B (constraints editor + porosity source). This first slice is the **backend route** for
the **Porosity Source** choice from your image 2: the clay bound-water constraint can now be driven by
either **CEC** (default — `v_bw = α·96·CEC·ρ/(T+298)·v_dryclay`, nothing moves) or **Wet Clay Porosity**
(`v_bw = φ_clay/(1−φ_clay)·v_dryclay`, geometric). It's the same physics the Clay-tab wet→dry converter
already used (`dry_clay_calc`); this exposes it as a selectable source. Clays now carry Techlog's WCLP
defaults (Illite 0.104, Kaolinite 0.058, Chlorite 0.101, Glauconite 0.156, Montmorillonite 1.0, Clay 0.12).

Default stays **CEC**, so every reviewed number is untouched (verified: the CEC path is byte-identical to
before). The **UI radio + per-clay φ editor + the constraints panel (UNITY/POROSITY/X&U BNDWAT/WATER MUD)
land in the next slice** — nothing to click yet. Tests: the WCP multiplier equals the CEC route's
`cec_equiv` (the dry_clay_calc bridge) and drives the same bounded solve; Techlog WCLP defaults asserted;
adversarially reviewed. Note: the WCP source **moves PHIE** for clays (bound water is now geometric, not
CEC-derived) — that's the design you approved.

**Smectite fix (adversarial review caught this before commit).** Techlog carries `WCLP_Smectite = 1.0`,
but it only ever consumes that value *post-solve* for wet-clay-volume reporting (flooring `1−φ` at `1e-4`),
never as an inversion constraint. My first cut fed it straight into the BNDWAT *solver* row as `φ/(1−φ)`
with a `0.95` cap → `k ≈ 19`, ~100× every real clay and ~30× smectite's own CEC route — it would have
swamped the bound-water constraint and forced absurd bound water wherever montmorillonite appears. Fixed:
a degenerate `φ ≥ 0.5` (Techlog's real clays are all ≤ 0.156, so this cleanly isolates the `1.0`
placeholder) now **falls back to the CEC-calibrated multiplier** for that clay, so the two porosity
sources *agree* for smectite (`k ≈ 0.6`) instead of diverging 30×. New test
`wcp_degenerate_smectite_falls_back_to_cec` pins it; `library_has_expected_shape` asserts every
non-smectite clay's WCLP stays a physical geometric porosity. Real clays (Illite φ=0.104, etc.) are
unaffected — they still use the geometric `φ/(1−φ)` route.

## Round 23 — SandiMin Juhász / normalized-Qv Sw (the wet-shale model) (2026-07-23)

The **Juhász (normalized Waxman-Smits)** model — the wet-parameter one you grouped with Indonesia/
Simandoux — is now in the Sw dropdown as **"Juhász / normalized Qv."** Instead of dual water's
temperature-form clay conductivity, it reads the excess conductivity straight from the **shale point**:

    Cwsh = 1/(Rsh·φ_sh^m),   QVN = Vsh·φ_sh/φt,   Cw·Swt^n + QVN·(Cwsh−Cw)·Swt^(n−1) = Ct/φt^m   (a=1)

so it uses your wet-shale parameters directly (Rsh from a shale pick + **φ_sh = wet-clay porosity**, a new
input that appears only for this model). Runs **post-solve** like the others — the mineral solve is
untouched, **PHIE/PHIT/unity preserved**, only SWE/SWT/SXOT move. With Vsh=0 it collapses to clean-sand
Archie (tested). Equation matches the Geolog `sw_juha` / cookbook normalized-Qv form.

Internally, dual-water and Juhász now share one root solver (`sw_cond_root`) — the only difference is the
excess-conductivity coefficient (dual water `Swb·(Cwb−Cw)`; Juhász `QVN·(Cwsh−Cw)`). The dual-water
numbers are unchanged (same 30 tests green). Hand-computed literals at n=2 (closed form) and n=3
(bisection), Vsh=0→Archie, and NaN guards all pass; adversarially reviewed.

**Note on the porosity source:** Juhász here uses φ_sh only *inside the conductivity equation* — the
water/HC split still uses the CEC-solved bound water (so PHIE stays put). The *full* "Wet Clay Porosity"
porosity-source that redefines bound water (image-2 constraints panel) arrives with that editor; the two
are the same underlying mechanism and I'll wire them together there.

- [ ] **Juhász vs Simandoux/Indonesia.** On a shaly interval with a good shale pick (Rsh, φ_sh), confirm
      Juhász SWE sits in a sensible band with the other shaly-sand models; on a clean sand it should track
      Archie. Try: Fluid tab → Sw equation → *Juhász / normalized Qv*, set Rsh + φ_sh, Run.

## Round 22 — SandiMin log-input grid + tidy Run button (2026-07-23 field review)

Two visual fixes from your screenshots:

- **Log inputs (image 3 style).** The cramped single column with wrapping labels
  ("Formation Density" breaking across lines) is now a **multi-column grid** — one column
  when the pane is narrow, more as it widens, scrolling both ways, matching the mineral list.
  Labels ellipsis instead of wrapping so the checkboxes stay aligned; hover shows the full
  name + mnemonic.
- **Run button (image 1 style).** No longer a full-width slab — it's now a **tidy, left-aligned
  button** with standard module proportions like Porosity-from-Density, and (per your "then for run
  button" go-ahead) in the **theme accent** so it matches every other module's Run across the
  client-brand skins. This supersedes the earlier "distinct green" — say the word if you actually
  wanted it kept a different colour and I'll bring the green back.

Verified in the browser against the live CSS: log grid resolves to 2 columns at a 560 px pane
(1 when narrower), labels truncate with ellipsis (no wrap), Run renders 76 px wide (not full
width) in the accent colour (rgb(217,140,63) in the dark skin). tsc clean.

- [ ] **Log inputs read cleanly** at your usual pane width — columns wrap sensibly, no label
      overflow, checkboxes line up.
- [ ] **Run button** looks right in the accent colour where it sits at the top of the pane.

## Round 21 — SandiMin Archie (clean-sand) Sw + deduplicated menu decision (2026-07-23)

You chose the **deduplicated** Sw menu (one entry per distinct model). First of the remaining ones:
**Archie (clean sand)** — `Sw = (a·Rw/(φt^m·Rt))^(1/n)`, no shale term. It's the exactly-invertible
baseline (so there's no separate "Archie linear/nonlinear" — they'd be identical). Runs post-solve like
the others: PHIE/PHIT/unity preserved, only the water/HC split moves; on shaly sand it reads
optimistically high (by design — it's the baseline the shaly-sand forms correct). Tests: hand-computed
literals at n=2 and n=3, clamp/NaN guards, and a check that Archie ≡ Indonesia with Vsh=0. cargo + tsc clean.

Menu now: Linear dual-water (default) · Dual-water non-linear · **Archie** · Indonesia · Simandoux.
Still to come: **Waxman-Smits** (dry BQv, Waxman-Thomas B default) and **Juhasz / Normalized-Qv**
(wet-param — brings in the wet-clay-porosity input that also feeds the image-2 porosity-source toggle).

- [ ] **Archie baseline.** On a clean water/HC sand, confirm Archie SWE matches your quick-look Archie;
      on a shaly interval, confirm it reads higher than Simandoux/Indonesia (the expected over-estimate).

## Round 20 — SandiMin non-linear dual-water Sw (the 4th model you picked) (2026-07-23)

The **non-linear dual-water** you asked me to continue is now in the Fluid-tab "Sw equation" dropdown as
**"Dual-water non-linear (m, n separate)."** Unlike the default *linear* dual-water — which folds the
exponents into a single `w = 0.75m+0.25n` and solves the conductivity as a linear row inside the
inversion — this solves the **exact** Clavier-Coates-Dumanoir form honouring **m and n separately**:

    Ct = (φt^m · Swt^n / a) · [ Cw + (Cwb − Cw)·Swb/Swt ]

It runs **post-solve** (same as Indonesia/Simandoux): the mineral inversion runs untouched (the CT tool
stays in, so the split stays well-posed), then Swt is solved from that equation and the water/HC split
redistributed — **PHIT, PHIE and hard unity are preserved**, only SWE/SWT/SXOT move. The **bound-water
saturation comes straight from the solved bound-water volume** (Swb = v_bw/φt), so no lab Qv is needed,
and the clay-bound-water conductivity Cwb is the temperature form already in the fluid calc. Equation
verified against the Geolog `sw_dual` stdlib form.

Tests: hand-computed numeric-literal point (φt=0.3, Swb=0.2, Cw=2, Cwb=5, m=n=2 ⇒ Rt=10.288 ⇒ SWT=0.6),
the effective-Sw conversion (SWE=0.5), a general-n bisection round-trip, NaN guards, and an end-to-end
run recovering a known deep Sw with PHIE untouched. `linear_dw` stays the default — reviewed numbers
unmoved.

Still to come from your image-1 menu: Archie linear/nonlinear, Waxman-Smits, Juhasz + Normalized
Dual-Water (the wet-param normalized-Qv forms), and the wet/dry-clay-parameter wiring.

- [ ] **Dual-water non-linear.** On a well with CT + an HC component (ideally with a clay + BoundWater so
      Swb>0), run once on Linear dual-water then again on **Dual-water non-linear** with your m and n —
      confirm SWE/SWT move to the exact-equation answer while PHIE/PHIT come out identical to Linear.

## Round 19 — SandiMin dialog layout (your field review: run-on-top, tab order, multi-column) (2026-07-23)

Four layout fixes from your image markups, all in `src/ui/multiminDialog.ts` + `src/styles.css`:

- **Run / apply-to-wells on top.** The Apply-to-wells scope, output options, and the **Run** button now
  sit in a boxed section **above** the parameter tabs, so you launch a run without scrolling past every tab.
- **Run button is a distinct green** (`#2e7d4f`), set apart from other modules' accent-coloured runs.
- **Log inputs tab is first** (Log inputs → Minerals → Fluid → Clay) and the pane **opens on Log inputs**.
- **Minerals / Clays / Fluids lists are multi-column** — they wrap to as many columns as the pane width
  allows and scroll both ways, instead of one endless single column.

Browser-verified in the live DOM: tab order + default tab, run-section-before-tabs, the green run colour
(rgb 46,125,79 on white), and the minerals list laying out in 3 columns at a 920-px pane. tsc 0. Nothing
about the solve changed — this is layout only.

- [ ] **Layout sanity.** Open SandiMin: confirm Run + Apply-to-wells are on top, the Run button is green,
      Log inputs is the first/active tab, and the Minerals/Clays/Fluids lists show multiple columns
      (narrow the pane and confirm they reflow / scroll).

## Round 18 — SandiMin Sw-equation selector on the Fluid tab (your request, increment 3b) (2026-07-23)

The backend from Round 17 is now selectable. The **Fluid tab** has a new **"Sw equation"** dropdown —
**Linear dual-water (default)** / **Indonesia (Poupon-Leveaux)** / **Simandoux (modified)**. Pick a
shaly-sand form and two extra fields appear (**Rsh** shale resistivity, default 4.0 ohmm; **Archie a**,
default 1.0) plus a one-line note explaining it runs post-solve and needs a CT tool + a U-zone HC
component. Leave it on Linear and everything behaves exactly as before. Browser-verified: the three
options render, the Rsh/a fields + note show only for Indonesia/Simandoux and hide again on switch back,
and the selector lives inside the conductivity-gated fluid box (so it's present exactly when Rt is). tsc 0.

Still to come (the 4th option you picked — "all of them"): the **in-inversion non-linear dual-water**
(Gauss-Newton, honours m and n separately). It'll drop into this same dropdown when it's ready.

- [ ] **Pick your Sw equation.** Open SandiMin ▸ Fluid: confirm the "Sw equation" dropdown, that
      choosing Indonesia/Simandoux reveals Rsh + Archie a, and that Rsh prefills 4.0 (**set it from a
      shale pick — a too-high Rsh inflates Sw**, wrong-way for fresh-water LRLC pay).
- [ ] **It changes Sw, not porosity.** On a well with CT + an HC component, run once on Linear, then
      again on Indonesia (or Simandoux) with your Rw/Rsh — confirm SWE/SWT move to the shaly-sand answer
      while PHIE/PHIT come out identical to the Linear run.

## Round 17 — SandiMin saturation models: linear dual-water + Indonesia + Simandoux (your request, increment 3a) (2026-07-23)

You asked for a selectable conductivity/Sw equation, "linear and non linear," because it's significant
to the wet/dry clay framework. This increment lands the **backend + math**; the Fluid-tab selector that
exposes it is the next increment (3b), so there's nothing to click yet — this entry is for the record.

What's in the solver now (`src-tauri/src/multimin2.rs`), all behind a new `sw_model` request field that
**defaults to `linear_dw`, so every run you've already reviewed is byte-for-byte unchanged**:

- **Linear dual-water** (default) — the existing in-inversion `Ct^(1/w) = Σ v·C^(1/w)`, `w = 0.75m+0.25n`.
- **Indonesia (Poupon-Leveaux 1971)** — effective-porosity form `1/√Rt = [Vsh^(1−Vsh/2)/√Rsh + √(φe^m/(a·Rw))]·Sw^(n/2)`.
- **Modified Simandoux (Bardon-Pied)** — `1/Rt = φe^m·Sw^n/(a·Rw·(1−Vsh)) + Vsh·Sw/Rsh` (closed-form quadratic at n=2, bisection otherwise).

Both shaly-sand forms are **post-solve**: the mineral inversion runs as usual (the deep-conductivity tool
stays in, so the solve stays well-posed), then Sw is replaced by the closed form using the solved effective
porosity and shale volume, and the U-zone water/HC split is redistributed to honour it — **φe and hard unity
are preserved**, so only SWE/SWT/SXOT change, never PHIE. New fluid inputs `Rsh` (shale resistivity, default
4.0 ohmm) and Archie `a` (default 1.0) feed the shaly-sand forms; the dual-water model ignores them.

Adversarially reviewed (3 lenses — equation transcription, solver integration, contracts). Confirmed the
equations against the standard references and the linear default as unchanged; fixed a real defect (a
shared-zone fluid would be double-scaled by the U- then X-zone override → silent PHIE/unity corruption; now
the flushed override runs only on a zone-disjoint split) and hardened the tests (added an **independent**
hand-computed Archie/shale check so a transcription error fails rather than being self-confirmed by the
round-trips). cargo 288/0.

- [ ] *(No click-through yet — UI is 3b.)* When the selector ships, the check will be: on a fresh well pick
      **Indonesia** or **Simandoux**, set Rw/Rsh, Run, and confirm SWE moves to the shaly-sand answer while
      PHIE/PHIT stay exactly as the linear run produced them.

## Round 16 — SandiMin dialog polish: theme parity + shrinkable/scrollable lists (your review) (2026-07-23)

Two of the three things you flagged on the tabbed SandiMin pane (the third — the conductivity/Sw
equation selector — is a separate, larger change I'm holding for your model choice):

- **Theme parity.** The pane's inputs, selects, and checkboxes were rendering as raw browser
  controls (white box, OS-blue tick) instead of the themed look every module pane uses (your
  image 2, the Porosity Ceiling pane). They now use the brand surface — `--bg-app` fields with
  `--border`, and checkboxes/radios take the theme accent instead of OS blue — so the whole pane
  reads one theme. Scoped to SandiMin for now; a one-line global rule would fix every other pane's
  checkboxes the same way if you want that (say the word).
- **Shrinkable + scrollable lists.** The mineral list is now three collapsible groups —
  **Minerals** (open), **Clays** and **Fluids** (collapsed by default) — each capped-height and
  scrollable, with a live `selected/total` badge on the head. The **Log inputs** list is likewise
  one collapsible, scrollable group with a `on/total` badge. Click any head to shrink/expand.

Browser-verified: the four groups render with correct open/collapsed defaults and counts
(Minerals 1/4 open, Clays 1/2 collapsed, Fluids 2/3 collapsed, Log inputs 5/16-on open), clicking
a head toggles both the collapsed state and the body, and the themed fields/accent resolve to the
active theme's variables. tsc 0.

- [ ] **The pane matches the app theme.** Open SandiMin: the mineral checkboxes, the endpoint
      inputs, and the fluid/clay fields should look like the Porosity Ceiling pane — brand accent
      ticks, themed field backgrounds — not white boxes with blue ticks.
- [ ] **The lists shrink and scroll.** On **Minerals**, confirm Clays/Fluids start collapsed and
      click their heads to expand; on **Log inputs**, confirm the 16-row list scrolls within its
      box. The head badges should track what you've selected/turned on.

## Round 15 — SandiMin dialog: tabbed setup (your request) (2026-07-23)

The Mineral Solver pane was one long scroll — minerals, log inputs, fluid properties, and the
clay converter all stacked. It's now **tabbed**: **Minerals** (component selection + presets +
the endpoint matrix), **Log inputs** (the tool list + user-defined inputs), **Fluid** (Rw/temps/
m/n/mud + precalc autofill), and **Clay** (the wet→dry converter). The run controls — well scope,
output prefix, unity/reconstruction toggles, the Run button and the results/QC — stay in a
**persistent footer below the tabs**, so you set things up across tabs and run from anywhere
without losing your place. The Fluid tab shows a short hint (instead of going blank) when no
conductivity tool is active, since the fluid numbers only matter to CT/CXO. Nothing about the
solve, endpoints, or wiring changed — this is purely how the pane is organized. Browser-verified:
tab switch shows exactly one panel, the CT toggle flips the fluid hint/grid, the footer stays put.

- [ ] **The pane is easier to navigate.** Open SandiMin (Modules ▸ SandiMin): confirm the four
      tabs across the top, that clicking each shows only that section, and that the Apply-to-wells
      scope + Run button + results stay visible no matter which tab you're on.
- [ ] **Nothing regressed.** Set up a clastic run as before — pick minerals on **Minerals**,
      confirm your tools on **Log inputs**, set Rw/temp on **Fluid**, Run from the footer — and
      check you get the same curves and DOF/incoherence readout as before the reorganization.

## Round 14 — Saturation-height solvers: Thomeer, log-driven Leverett-J, per-rock-type laws (playbook #4, increment 4a) (2026-07-23)

The SHF fitting engine now covers all five families and can split by rock type. **Thomeer** joins
the height-domain forms: the carbonate-standard hyperbola `Sw(H) = 1 − (1−Swirr)·exp(−G/log10(H/Hd))`
(Thomeer 1960), fitted with the same bounded simplex as Skelt — Hd is the entry height (the
displacement pressure expressed in metres above the FWL) and G the pore-geometrical factor
(≈0.1 well-sorted → >2 poorly sorted). **Leverett-J now fits from logs**, not only at SCAL
import: each sample's height becomes reservoir Pc (0.433·Δρ·h_ft), J = 0.21645·Pc/σcosθ·√(k/φ)
from the PERM/PHIE curves, and Sw = A·J^B is regressed in ln-ln space (Leverett 1941). Fluid
defaults are Tier-A seeds — σ·cosθ 26 dyn/cm (IP cap-pressure table, Water-Oil 30 dyn/cm·cos 30°),
HC density 0.7 g/cc (Techlog) — all per-run overridable. **Per-rock-type fits**: hand any family
an RT/facies curve and it fits one law per rock-type class alongside the pooled law (the single
biggest SHF accuracy win on stacked Mahakam sands); classes that can't fit are reported with the
reason, never dropped. **Nothing is dropped silently anymore**: every excluded sample is counted
by reason (Sw > 1, at/below the FWL, below the φ cutoff, no permeability), scoped wells that
contributed zero samples are named in a note, and a Buckles check (Buckles 1965) flags when the
above-transition BVW isn't one constant — the classic sign you need per-rock-type laws. The
breakdown survives even when the fit itself fails — that's when you need it most.

Adversarially reviewed (37 agents, 4 lenses → 3-skeptic verification): 8 confirmed findings → 4
distinct defects, all fixed pre-commit — a Thomeer bounds panic on sub-millimetre height ranges
(HIGH), silent zero-contribution wells, discarded exclusion counters on two FOIL error paths, and
the failed-group NaN→null IPC contract. cargo 283/0, tsc 0. (Dialog UI for all of this = 4b, next.)

## Round 15 — Saturation-height dialog: 5 families, per-rock-type tabs, draggable FWL (playbook #4, increment 4b) (2026-07-23)

The Saturation-Height dialog now drives everything the 4a solvers added. The **SHF-form dropdown
has five entries** (FOIL / Brooks-Corey / Skelt / Thomeer / Leverett-J); picking **Leverett-J**
reveals a permeability-curve picker and a fluid-property block (system dropdown that flips σ·cosθ
between the Water-Oil 26 and Water-Gas 50 dyn/cm Tier-A seeds, plus ρw/ρhc — all editable). A
**"Fit per rock type" checkbox + RT-curve picker** turns any family into per-class fits: the
results panel grows a **tab strip** (All / RT 1 / RT 2 …), each tab showing that class's
parameters, R², and its own Sw-vs-height curve; classes that couldn't fit show a ⚠ tab with the
reason instead of vanishing. Every result now carries a **diagnostics line** — the excluded-sample
breakdown (Sw > 1, at/below the FWL, φ-cutoff, no-perm counts) and the honesty notes (zero-
contribution wells, the Buckles warning) — shown on both success and failure. The **FWL is
draggable**: drag horizontally on any result plot to nudge it (0.2 m/px) and it re-fits on release,
or click straight on the FWL-scan curve to pick a candidate. An **RMS** row joins R² in every
parameter table. tsc 0.

- [ ] **All five families fit.** Analysis ▸ Saturation-Height on BLSO: run each of FOIL,
      Brooks-Corey, Skelt, Thomeer, Leverett-J. Thomeer and Leverett-J should return sensible
      params (Thomeer G ~0.1–2, Leverett B negative) with a curve through the Sw-vs-H cloud.
- [ ] **Leverett-J uses PERM.** Pick Leverett-J → the PERM picker + fluid block appear; switch
      the system Water-Oil↔Water-Gas and watch σ·cosθ flip 26↔50. Fit with your PERM curve.
- [ ] **Per-rock-type split.** Tick "Fit per rock type", pick your RT curve, fit: a tab per RT
      class appears, each with its own law + curve; a thin class shows a ⚠ tab with the reason.
- [ ] **FWL by drag / click.** Drag left-right on the crossplot — the status shows the trial FWL
      and it re-fits on release; on FOIL with the scan on, click the scan curve to jump the FWL.
- [ ] **Nothing hides.** Set the FWL above the whole cloud and fit: the failure now shows an
      "Excluded: at/below the FWL: N" breakdown instead of a bare error.

## Round 14 — Saturation-height solvers: Thomeer, log-driven Leverett-J, per-rock-type laws (playbook #4, increment 4a) (2026-07-23)

## Round 13 — Theme sweep: canvas typography + color tokens (playbook #9A, increment A) (2026-07-22)

Every canvas font and the last hard-coded colors that bypassed the theme are now driven by the
theme system, so plots, dialogs, and overlays stay legible and on-brand across all eight skins
(light / dark / Pertamina / Halliburton / Schlumberger / LAPI-ITB / white / system). An inventory
workflow (4 parallel sweeps) found **111 bypasses across 20 files**; all fixed. New tokens:
`--font-canvas` (the Segoe-variable stack) and `--font-mono` in styles.css, a `canvasFont(theme,
size, weight)` helper on the shared plot scaffolding (`PlotTheme` gained `fontFamily`), so all
~55 `ctx.font` literals now resolve through one token. Color fixes: the well-diagram casing
strings/shoes (was mid-gray `#5a5a5a`/`#333` — invisible on dark) now use `--text`; perforation
ticks use `--warn`; the crossplot/Pickett "no-data" gray marker now derives from `--text-dim`;
the highlights default palette and the "Add curve" default color are built from the live theme
accents instead of fixed light-theme values. Browser-verified across all six branded palettes:
6 distinct accents, 6 distinct no-data markers, the font token resolves and stays stable, and the
derived palettes are all valid hex (safe for the color pickers). tsc 0, production build clean.

- [ ] **Themes stay legible everywhere.** Cycle the theme (ribbon ▸ theme) through dark and a
      client brand (Pertamina/SLB) with a log view, a crossplot, and the well-diagram track open:
      axis/label text, casing strings + perforations, and crossplot no-data points should all
      stay readable — nothing washes out or disappears the way the old mid-gray casing did on dark.
- [ ] **New curves + highlights adopt the brand.** On a branded theme, add a curve in Layout
      Properties and drag a highlight band: both should come up in the theme's accent, not the
      light-theme terracotta.

## Round 12 — Monte Carlo sampling engine: LHS, rank correlation, convergence (playbook #1, increment 1.1) (2026-07-22)

The Monte Carlo engine's draw generation is rebuilt to commercial grade. **Latin Hypercube
Sampling is now the default**: each parameter's probability range is split into N equal strata
with one jittered draw per stratum (order shuffled per parameter), so the sampled CDF matches the
distribution far tighter than independent draws at the same N — P10/P90 bands stabilize with
fewer iterations (McKay–Beckman–Conover 1979). The old scheme survives as `sampling: "random"`
and reproduces pre-upgrade results byte-for-byte at the same seed. Two new opt-ins: **parameter
rank correlations** (Iman–Conover 1982 — e.g. tie RHO_MA to GR_MA at ρ 0.7; marginals are only
reordered, never altered, and inconsistent/unknown pairs come back as notes, not errors) and a
**convergence check** (running P10/P50/P90 of total HPV per batch; in random mode the run stops
early once the trace goes stationary — LHS always runs its full design, since truncating one
would leave strata unsampled). `montecarlo.rs` + `ipc.ts`; 5 new tests (legacy request shapes parse with LHS defaults;
exactly-one-draw-per-stratum + analytic mean; achieved Spearman hits ±targets and marginals are
pure reorderings; flat series early-stops with a consistent truncated result; LHS never
truncates). cargo 274/0, tsc 0. The LHS/random toggle, correlation editor, and convergence
sparkline arrive in the dialog with increment 1.3 — until then the pane simply runs LHS.

Adversarially reviewed (18-agent workflow, 4 lenses × 2 skeptics); all 4 confirmed findings fixed
in the same round: (1) correlation targets are now pre-adjusted by the Spearman→Pearson map
2·sin(πρ/6), so the achieved rank correlation centers on your ρ instead of landing ~0.014 low;
(2) a duplicated/conflicting correlation pair now reports "last entry wins" in `notes` instead of
resolving silently; (3) the convergence trace folds the remainder into its final batch, so the
end-of-run "converged" verdict can't be inflated by a runt 4-realization checkpoint; (4) a
**pre-existing tornado bug**: with a zone that has no pay at the parameter medians, switching the
sensitivity metric to Avg PHIE/Avg SWE crashed the pane (`null.toFixed`), and a single dry sweep
endpoint drew a bar anchored at a fabricated 0 — the renderer now says the base case has no
anchor and drops non-finite endpoints.

- [ ] **LHS is quietly better, not different.** Monte Carlo pane ▸ your usual GR_MA/RHO_MA setup on
      a real well, 1 000 iterations, seed 42 ▸ Run twice — identical results (reproducibility
      holds). Then drop to 300 iterations and re-run a few seeds: the P10/P90 HPV band should sit
      noticeably steadier across seeds than you remember from the old sampler at 300.
- [ ] **Dry-zone tornado no longer crashes.** Monte Carlo ▸ tornado on ▸ pick a marginal zone that
      has no pay at your median cutoffs ▸ after the run, switch the sensitivity Metric to
      Avg PHIE: you should get the "base case yields no Avg PHIE" message (previously this threw
      `TypeError … toFixed` and left a half-drawn panel).

**Increments 1.2 + 1.3 (same round):** distributions can now be **zone-scoped** — each uncertainty
row has a zone box (suggestions from the scoped well's zonation); a scoped draw applies only inside
that zone, everything outside follows the deterministic zone parameters, and the tornado/Spearman
rows are labeled `PARAM @ ZONE`. **Save LOW/BASE/HIGH curves** writes per-sample uncertainty curves
to a fresh **version** of the MONTECARLO log set per well (never overwrites — the Sets manager can
restore any run): `MC_<KEY>_LOW/_P50/_HIGH` are per-sample percentiles across realizations and
`MC_<KEY>_BASE` is one deterministic run at every parameter's median, for each of VSH/PHIE/SWE/PERM
the chain produces. The dialog grew the **Sampling** select (Latin Hypercube default / Random
legacy), the **Correlations** mini-editor (param ↔ param, ρ), **Convergence check** and **Save
curves** checkboxes, a per-well **convergence sparkline** (running P-low/P50/P-high with a
converged/not-converged badge), and a notes panel that surfaces backend advisories (skipped
correlation pairs, persist confirmations). Status line reports sampling, early-stop count, and
saved-curve count. 5 more cargo tests (zone-scoped spread stays in its zone + unknown-zone note;
persisted curves ordered LOW ≤ P50 ≤ HIGH and versioned v1→v2; inverted zone; input-skip;
stale-family reclaim + degenerate base). Browser-smoke-tested end-to-end.

The 1.2 backend was adversarially reviewed too (27-agent workflow); all 7 distinct confirmed
findings fixed before commit: an inverted zone (top ≥ bottom, storable via the DB inspector) now
yields a note instead of **panicking the whole run**; correlations naming a parameter that appears
in several zone-scoped entries note that ρ binds only the first; persisted curves are gated on
what the chain **produces** (inputs it merely consumes no longer come back as zero-width fake
uncertainty bands); the kept-snapshot pool survives convergence early stops (first-N prefix
instead of a precomputed stride); a re-run that writes fewer curve families reclaims the previous
version's stale MC_* rows from the current store (archive keeps every version restorable); a
degenerate all-median base run skips only MC_*_BASE with a note instead of discarding the valid
percentile curves; and a well whose persist write fails now finishes its job item **Warned**, not
Ok.

- [ ] **Zone-scoped uncertainty stays in its zone.** Monte Carlo ▸ add GR_MA, type a real zone name
      in its zone box (the box suggests your zones) ▸ Run: the named zone's P10–P90 band spreads,
      every other zone's collapses to a single value, and the tornado row reads "GR_MA @ <zone>".
- [ ] **Saved uncertainty curves land as a versioned set.** Tick "Save LOW/BASE/HIGH curves" ▸ Run
      ▸ open a layout and add MC_PHIE_LOW/P50/HIGH on a track: a proper uncertainty envelope
      around the P50, with MC_PHIE_BASE hugging your deterministic PHIE. Re-run — the Sets manager
      shows MONTECARLO v2 alongside v1.
- [ ] **Correlated draws + convergence read sensibly.** Add GR_MA and RHO_MA, correlate them at
      ρ 0.7, tick Convergence check, sampling Random, 5 000 iterations ▸ Run: the sparkline
      flattens and the run stops early with "stationary after N" (with LHS it always runs full
      size and says so).



Backend for the SandiMin reconstruction check. The existing **RECON** curve is now documented as the
**incoherence** — the σ-weighted RMS of (reconstructed − measured) over the live tool rows (Quanti.Elan
Eq 79). With the new **`recon_qc`** request flag the reconstruction is **decomposed per tool**:
`<prefix>_<KEY>_REC` = the log rebuilt from the solved volumes (in the tool's display units, so it
overlays the measured curve) and `<prefix>_<KEY>_DIF` = that tool's σ-unit residual (whose RMS over
tools is RECON). The result also reports model **degrees of freedom** `dof = (tools + soft + unity) −
components`, with a note when `dof == 0` (exactly determined → RECON is forced to ~0 and can't validate
the model). `multimin2.rs` + `ipc.ts`; 2 new tests (a forward-modeled 3-mineral well reconstructs to
incoherence ~0 and a wrong illite density inflates it + localizes to the density residual; the
exactly-determined case flags its note). cargo 269/0, tsc 0. **The recon-QC view shipped in the same
round (increment 2d):** a **Reconstruction QC** checkbox in the SandiMin dialog turns the per-tool
curves on; after the run the result shows the **model DOF** (with the exactly-determined warning) and a
**measured-vs-reconstructed crossplot** (each tool min-max normalized, points on the dashed 1:1 line =
a perfect fit, scatter off it = that tool's incoherence). Browser-smoke-tested: checkbox → run → DOF
line + crossplot render.

**Increment 2c** completed **#2** per your call to keep smectite as-is: a **Preset** selector atop the
component picker with four named GROUPINGS of existing library components — **Clastic**
(quartz–illite/kaolinite–water+bound), **SSC-style** (quartz–feldspar–clay, to compare VOL_* against
the SSC module's VSAND/VSILT/VCLAY), **Carbonate** (calcite–dolomite–anhydrite), **Organic/coal**
(quartz–illite–coal–kerogen, whose VOL_KEROGEN feeds the upcoming unconventional workflow). Presets
carry **no endpoint values** — Montmorillonite keeps RHOB 2.63 etc., so no reviewed number changed;
manually ticking a component drops back to "— custom —". Browser-smoke-tested all four.

- [ ] **Presets assemble the right model.** SandiMin ▸ Preset ▸ each of the four: the component
      checklist follows the grouping (note under the selector explains each), endpoints stay exactly
      what the library/your edits hold, and a manual tick resets the selector to custom. Run the
      Clastic preset on a Mahakam well and sanity-check VOL_QUARTZ/VOL_ILLITE against your SSC results.

- [ ] **Reconstruction flags a bad model.** In **SandiMin ▸ tick "Reconstruction QC" ▸ Run**. On a
      good model the crossplot points hug the 1:1 line and the incoherence stays low; force a wrong
      endpoint (or drop a needed mineral) and confirm the points for the broken tool scatter off the
      diagonal and the incoherence rises. The written `<prefix>_<KEY>_REC` curves can also be laid over
      the measured logs in a log view for a depth-by-depth check.
- [ ] **DOF honesty.** Build a model with exactly as many inputs as components (e.g. 3 minerals, 2 logs
      + unity). Confirm the dialog shows **DOF 0** in orange and warns that RECON can't validate the
      model; add one more input log and DOF rises to 1 (RECON becomes meaningful).

## Round 10 — Stratigraphic Modified Lorenz Plot: flow-unit solver (playbook #3, increment 3a) (2026-07-22)

New backend `lorenz.rs` — the **Stratigraphic Modified Lorenz Plot** (Gunter et al. 1997, SPE 38679).
It walks a well's φ + k logs in **depth order**, accumulates flow capacity Σ(k·h) against storage
capacity Σ(φ·h) (each normalized 0..1), segments the depth-ordered log10(k/φ) profile into **flow
units** with an exact contiguous dynamic program (auto-K by marginal gain, or a caller-set K), and
reports the **Lorenz heterogeneity coefficient** (Schmalz & Rahme 1950). Command `run_lorenz` +
`runLorenz` in `ipc.ts`. cargo **265/0** (9 new `lorenz` tests, incl. a synthetic 3-flow-unit column →
3 units), tsc **0**. Adversarially reviewed (4 lenses → **1 confirmed** IPC-nullability fix applied;
math + segmentation lenses clean). Method banked in `docs/ref_rock_typing.md`.

The **visual** (increment 3c-1) shipped in the same round: new pane **Lorenz Plot (flow units)** in
the ＋ add-panel menu — well + φ/k curve pickers (group-filtered, defaults to the selected well;
PERM list prefers PERM/KLOGH/PERM_RT), auto or forced K, optional MD window, then the SMLP curve
coloured by flow unit against the dashed 45° homogeneous diagonal, the per-unit table (top/base,
storage %, flow %, slope, **speed/baffle** character), and the Lorenz-coefficient headline.
Browser-smoke-tested on a stubbed 3-regime column: 3 units recovered, unit 1 = speed with 90 % of
flow from 33 % of storage, row-click highlight dims the other units.

**Increment 3c-2** completed **#3**: (a) a **Winland/Pittman pore-throat grid** on the crossplot —
Crossplot Properties ▸ *Rock-type grid* draws iso-radius lines at the port-class bounds
(0.1/0.5/2.5/10 µm) when one axis is porosity and the other permeability (Kolodzie 1980 R35 or
Pittman 1992 r25/r35/r50); (b) the **facies tie-in now also reports k-variance-reduction** — how
much of the core log10(k) spread the predicted rock-type class explains (ANOVA 1 − SSw/SSt), so the
tie-in is validated against permeability, not just class purity; (c) **RT as a FACIES block track**
needs no new code — set any integer RT curve's fill to **Facies blocks** in the log-view layout
props. cargo 267/0, tsc 0. (3b, the Pittman full-apex r10–r75 table, was already the `pittman_rx`
module.)

- [ ] **SMLP + flow units on a real well.** On a well with PHIE + a permeability curve (imported
      KLOGH, computed PERM, or the rock-typing PERM_RT), open **＋ add-panel ▸ Lorenz Plot (flow
      units)** ▸ Build Lorenz Plot. Confirm the curve ends at (1,1), and steep **speed** segments
      coincide with your best reservoir sands (high k/φ) while flat **baffle** segments fall on
      shale / tight streaks — the flow-unit boundaries should track your net-sand tops.
- [ ] **Lorenz coefficient sanity.** A clean, well-sorted sand gives a **low** coefficient (near 0);
      a layered sand-shale interval a **high** one (→1). Use the MD window (a zone's top/base) to
      Lorenz two zones you know differ in heterogeneity and confirm the number moves the right way.
- [ ] **Winland/Pittman grid on a φ-k crossplot.** New Crossplot ▸ X = PHIE, Y = a permeability
      curve (log Y on) ▸ Properties ▸ **Rock-type grid = Winland R35** (or a Pittman rX). Confirm the
      dashed iso-radius lines (0.1/0.5/2.5/10 µm) fan across the cloud and your best plugs sit in the
      macro/mega band. Flip the axes — the grid should still draw (orientation auto-detected).
- [ ] **Facies tie-in explains permeability.** On a well with a core-derived RT + a log RT and core
      k, run **Facies Tie-in**. Besides purity, confirm the **k variance reduction %** appears and is
      high when the classes separate core k, low when they don't (needs core plugs within 1 m of the
      log samples).

## Round 9 — Cross-feature fix: survey TVD/TVDSS must not shadow an imported one (2026-07-22)

A cross-feature adversarial review of the four shipped feature_work commits (constants/TVD/ML-MASK/
DLIS) found one real HIGH seam bug between TVD materialization (Round 6) and the standard→computed→
generic resolution order (Round 8): importing a deviation survey wrote a **computed** TVD/TVDSS, which
outranks the generic store, so it silently shadowed an authoritative TVDSS a user had imported from a
vendor LAS/DLIS — with a possibly wrong datum (no-KB wells fall back to a sea-level datum) or NaN
outside the survey's MD range, and no recourse via Promote (disabled on a "served by computed" row).
Fixed in `materialize_tvd_curves` (ingest.rs): it now only materializes a name the well does not
already resolve from an import, and clears any stale survey-derived computed curve so the import keeps
winning. cargo 256/0, tsc unchanged. Test `materialize_tvd_keeps_imported_tvdss_authoritative`.

- [ ] **Vendor TVDSS survives a survey import.** On a well that has a TVDSS curve from its LAS, import
      a deviation survey. Confirm the plots/modules still read the **imported** TVDSS (unchanged values,
      full depth coverage) — not a survey-derived one. TVD (if not imported) still appears from the survey.
- [ ] **Recompute is still safe.** Edit KB and run Data ▸ Recompute TVD/TVDSS. A well WITHOUT an imported
      TVDSS refreshes its survey-derived TVDSS; a well WITH an imported TVDSS keeps the imported one.

## Round 8 — DLIS/LAS mnemonic-shadow resolution in the Curve Catalog (2026-07-22)

When a DLIS and an LAS (or two DLIS runs) carry the **same mnemonic**, the Curve Catalog now
detects the collision, badges the resolver's current winner, and lets you **Promote** the one you
want or **Delete** a duplicate — without editing files. Backend `db.rs` (new `pinned` column +
promote/delete), resolver tiebreak in `equations.rs` + `curve_edit.rs`, frontend
`inspectorPanel.ts`/`ipc.ts`/`styles.css`. cargo 255/0, tsc 0. Adversarially reviewed (4 lenses →
**5 confirmed findings, all fixed**): the resolver no longer lets a pin leak across a family, and the
Catalog no longer claims a Promote "wins" when a higher-priority store actually resolves the curve.

- [ ] **Promote resolves a real same-mnemonic shadow.** On a well where a DLIS and an LAS both carry a
      **non-standard** mnemonic (e.g. `PEF`, `CALI`, `DTS`, or a core `PERM` with no computed PERM),
      open the **inspector ▸ Curve Catalog**: the two rows show **`resolves`** / **`shadowed`** badges.
      Click **Promote** on the shadowed one → it flips to `resolves` + `pinned`, and any plot/module
      reading that curve now picks up the promoted values. **Delete** the loser → the sibling resolves.
- [ ] **No false "it now wins" for standard logs.** For `GR / RES_DEEP / NPHI / RHOB / DT / SP`, the
      real curve is served from the standard log column, not the RAW catalog copy. Those rows now show a
      neutral **`served by log`** badge and **Promote is disabled** (tooltip: "resolution comes from the
      standard log column — promoting has no effect"). Previously Promote here claimed victory but changed
      nothing on any plot — that lie is gone.
- [ ] **No false win when a computed curve owns the name.** If you've computed a curve (say `PERM` from
      Coates) and also imported a raw `PERM`, the raw row shows **`served by computed`** and Promote is
      disabled — the computed curve resolves first, so promoting the raw one would have been a silent
      no-op.
- [ ] **A pin doesn't hijack the family (deep-R sanity).** Promoting one same-mnemonic shadow must NOT
      change which curve a **family** request resolves. On a well whose deep-resistivity feeds Sw, promote
      an unrelated same-mnemonic shadow and confirm Sw is unchanged (the pin now applies only to its own
      mnemonic, and family requests rank by base run — deterministic across re-import/reopen).

## Round 7 — MASK support in the ML pipeline (2026-07-22)

Optional flag curve in the ML dialog: samples where the mask = 1 are excluded from training AND left
blank (NaN) in the prediction — the same 0/1 convention as the module MASK. Backend `ml.rs` + frontend
`mlDialog.ts`/`ipc.ts`. cargo 253/0, tsc 0. Adversarially reviewed (3 lenses → 2 confirmed honesty
fixes applied).

- [ ] **Masked training + apply.** On a well carrying a BADHOLE / FLAG_PAY / COAL 0-1 flag curve, open
      ML Models, pick a **Mask (exclude)** curve, run a regression/classification → confirm the output
      curve is BLANK (NaN) at flagged depths and the per-well "Predicted samples" count drops.
- [ ] **Mask governs clustering/PCA too.** For an unsupervised task the mask keeps flagged samples out
      of the fit AND leaves them blank — facies with vs without a mask differ (bad-hole shouldn't shape
      facies).
- [ ] **Leaderboard honesty.** In **Compare algorithms** with a mask that empties a whole training
      well, the header shows the TRUE contributing-well count and a note that blind-well CV fell back
      to random KFold (previously it hid the collapse behind the requested well count).

## Round 6 — TVD/TVDSS as fetchable curves (2026-07-22)

Materialize the deviation survey onto the log depth grid as `TVD` and `TVDSS` computed curves,
so height-based tools can consume them by name. Backend `deviation.rs`/`ingest.rs`/`lib.rs` +
frontend `ipc.ts`/`ribbon.ts`. cargo 250/0, tsc 0.

- [ ] **Deviation import now writes TVD/TVDSS curves.** On a **deviated** well with logs loaded,
      Data ▸ Import Deviation… a survey → confirm `TVD` and `TVDSS` appear as computed curves
      (Curve Catalog / any module's log-input dropdown). TVD should be shallower than MD in the
      built section; TVDSS = KB − TVD.
- [ ] **`sw_height` TVD input now works.** Run the Saturation-Height module selecting the new `TVD`
      curve for the TVD input — on a deviated well the height (HAFWL) and SWH now use true vertical
      depth instead of MD (previously the TVD input was a silent no-op → MD fallback → optimistic pay).
- [ ] **SHF fits can use the materialized TVDSS.** In the Cuddy FOIL / Brooks-Corey / Skelt / Thomeer
      panes, pick the new `TVDSS` curve as the vertical-depth input and confirm the fit runs.
- [ ] **Correlation TVDSS depth-mode** now works from the survey (not only from an imported TVDSS log).
- [ ] **Data ▸ Recompute TVD/TVDSS Curves** — run after importing logs *after* the survey, or after a
      KB edit. Status reports "computed for X of Y surveyed well(s), N samples"; surveyed wells with no
      logs yet are counted as pending. *(Note: the survey-derived TVDSS is written to the computed store,
      which takes precedence over an imported TVDSS log of the same name when fetched.)*

## Round 5 — Rock-typing constants verification vs papers (2026-07-22)

Read-only cross-check of every hardcoded literature constant in `rocktyping.rs` / `shf_fit.rs` /
`thomeer.rs` / `hfu.rs` (+ `satheight.rs`) against `docs/research_2026-07/ref_rocktyping_shf.md` and
the published sources. Full write-up: `docs/constants_verification_2026-07-22.md`. **2 corrections
applied (both number-changing, Jauhar approved); 1 held pending a primary-source glance.** cargo
247/0, tsc N/A (no TS).

- [ ] **GHE FZI bins corrected** (`rocktyping.rs`). Was `…1.5, 2.5, 4, 6, 8`; now the Corbett-Potter
      2004 ×2 series `…1.5, 3, 6, 12, 24`. Run the **Rock Typing (FZI/R35/PGS)** module with
      `METHOD=ghe` on a cored well and confirm the `RT` (GHE class) curve looks right for the
      best-quality rock — high-FZI samples now land in the correct GHE6–GHE10 bands (previously
      compressed). `PERM_RT` follows the class, so it shifts too.
- [ ] **PGS definitions corrected** (`rocktyping.rs`). `PGEOM` is now `√(k/φ)` (was `k/φ`) and the
      `PS_EXP` default is `3.0` (was `3.5`) — the ACS Omega 2024 / Kozeny-Carman form. Diagnostic
      curves only (RT class is unaffected). Confirm `PGEOM`/`PSTRUC` plot sensibly; `PS_EXP` is still
      an editable param if you want a different exponent.
- [ ] **Pittman r75 — HELD (not changed).** The code's r75 row `(1.243, 0.674, −1.517)` diverges from
      the widely-cited `≈(0.778, 0.626, −1.205)` while r10–r50 all match. Couldn't confirm online
      (Pittman's Table 1 is an image; primary is paywalled). If you can check **AAPG Bull. v76 (1992)
      p191-198, Table 1**, tell me the r75 coefficients and I'll fix the one row. Only affects `PR75`
      and `RT_PITT` when APEX=r75 (default r35 is fine).

## Round 4 — AUDIT-2026-07-21 safe-bucket follow-through (2026-07-22): correctness / honesty / robustness

Continuation of task #159 (the 65-finding full-QC audit). After batches 1–3 (`1d6b521`/`5e44620`/`1dcfeba`)
and the RT≤0 fix (`f33e126`), this round works the remaining **safe** bucket — fixes that harden behaviour
or improve reporting honesty WITHOUT changing interpretation numbers for valid data. Audit references were
re-verified against CURRENT code first (several were already fixed by the round-2/3 refactors — e.g.
correlation already subscribes to dataVersion; recordProcess already wired in ML/multimin/inspector).
**cargo 247 pass / 0 fail / 7 ignored; tsc EXIT 0. Nothing committed.**

Backend (Rust, unit-tested):
- [ ] **Cutoff-sweep geometric clamp.** `run_cutoff_sweep` now integrates each sample's clamped overlap
      with the zone ∩ DST interval (mirrors `run_pay_summary`), so NTG can no longer exceed 1 when a
      zone/DST boundary lands mid-sample. Sample-aligned results are byte-identical. **Try:** run Cutoff
      Sensitivity with a DST interval whose edges don't fall on log samples — NTG should stay ≤ 1 and agree
      with the Pay Summary for the same well/zone/cutoff.
- [ ] **Per-well isolation** in `run_pay_summary` + `run_cutoff_sweep`: one well's fetch/zone read error
      now skips just that well instead of zeroing the whole Field Dashboard / sweep response.
- [ ] **All-NaN module runs report honestly.** A module run whose every output sample is MISSING (e.g.
      gascorr with no precalc, or a module fed an all-NaN input, or SW-RtC on a well with no PHIT) is now
      reported as an error / Warned in the Processing panel — not a green "N samples → …" success. Same
      guard on Rhai + Python equations (an unresolvable input/output curve → error, not a clean success).
- [ ] **Python in-place equation guard.** An equation whose output curve name collides with an input
      (a "clean this curve in place" script) no longer silently writes the untouched input back when the
      script forgot to (re)assign it. (Also fixed a worker crash when the output was named `np`/`numpy`.)
- [ ] **LRLC SSPW fallback.** SW-RtC / SW-IMTS now fall back to the SSPW-named curves (PHIT_SSPW /
      CAPBW_SSPW / CBW_SSPW) when the SSC ones are absent — so they run on an SSPW-processed well instead
      of silently producing all-NaN. SSC-only wells are unchanged. **Try:** run SW-RtC on a well processed
      through SSPW porosity (no SSC curves).
- [ ] **LAS duplicate-name warning.** Importing a LAS whose (normalized) well name already exists now
      warns (still creates a separate record — merge is a deliberate action, not automatic). **Try:**
      import the same LAS twice; the second shows a "already exists" warning.
- [ ] **New test coverage** (no behaviour change): phi_den / phi_dn edge cases (VSH≥0.95 shale branch,
      SHALE_REDUCED-vs-MAXIMUM cap, density shale-reduction clamp, AVERAGE-vs-GAS_RMS), SSC `*_GR` family
      closure + degenerate-VWSH guard, and `run_ml`'s DB-integration guards.

Frontend (TS, tsc-clean):
- [ ] **History attribution.** A scoped module run records the wells actually run (single by name, batch
      as null) instead of the globally-selected well (which a scoped run may not have touched).
- [ ] **Blank "(none)" for optional inputs.** Optional log-input dropdowns now offer "(none)" so you can
      deliberately drop a curve slot even when a curve of that name exists in the project.
- [ ] **dataVersion refresh** after equation / ML / report runs and on workflow-chain **cancel/fail**
      (a cancelled chain routinely committed the earlier wells) — open plots/log views no longer show stale
      curves.
- [ ] **Race guards** on the module pane's data refresh (a slow refresh can't overwrite a fresher one) and
      SandiMin's **Autofill-from-precalc** (a well switch mid-fetch no longer stamps stale FTEMP/RMF).
- [ ] **Pay Summary → Processing History** (the FLAG-writing Compute now leaves a trace); **curve-edit
      Set-constant** rejects non-finite (Infinity) input; the deprecated legacy **`multimin`** module is
      filtered out of the Workflow step picker (use SandiMin).

Deferred / needs your call (see the summary I sent):
- Report "Tables only" still computes the composite geometry (efficiency, not correctness) — a truly safe
  fix must reproduce the cover interval exactly, which needs the same expensive fetch. Held.
- Low-value polish left: MC histogram theme-repaint; ml/wellScope dataVersion subscribe.
- **6 findings that WOULD change interpretation numbers** await your sign-off (perm_coates default 100→70;
  phi_son OPT_CP DT_SH>100 gate; log_predict masked-fill survival; legacy-multimin RECON_ERR at 3 tools;
  MC PERM cutoff when chain-produced; MC MASK/computed_only parity).

## Round 3 — Feature Wave B chain (2026-07-22): fluid contacts, ML leaderboard, well-diagram, rock typing + SHF

Four Wave B features built back-to-back after the round-2 commit (`d64bdc7`). Each is tsc-clean and
either cargo-tested or cargo-check-clean; the novel math in each is unit-tested. **Not yet clicked
through in the real app with field data. Nothing committed.**

- [ ] **(9) Fluid contacts in Correlation.** New `fluid_contacts` store (well/field/global scope,
      OWC/GWC/GOC/GDT/ODT/FWL, depth, TVDSS flag, colour) + editor (Correlation ▸ **Contacts…**).
      Contacts draw as horizontal lines + cross-well connectors. New **MD / TVDSS depth mode** on the
      Correlation toolbar: in TVDSS a TVDSS-stored contact is **flat across every well** (converted per
      well via the TVDSS curve; falls back to MD == TVDSS for vertical wells). *(Verified: the TVDSS↔MD
      round-trip math — a TVDSS contact renders flat across two wells with different deviation, an MD
      contact flat only in MD mode; cargo check + tsc clean.)* **Try:** open Correlation, add an OWC as
      TVDSS, switch MD↔TVDSS, watch it flatten.
- [ ] **(3) ML comparison leaderboard.** In the ML pane (supervised tasks), a **Compare algorithms**
      button ranks every algorithm × a curve-subset strategy (full / leave-one-out / singles) by
      **blind-well GroupKFold CV** — whole wells are held out, fixing the depth-leak in the old random
      5-fold. Shows a sortable leaderboard (R²/accuracy + RMSE/macro-F1), **permutation importance** bars,
      and a **confusion matrix** for the selected row. *(Verified: 2 new Rust tests exercise the real
      sklearn GroupKFold path — blind-well R²≈1 for a linear law across 3 wells, 2×2 confusion for a
      classifier. Needs ≥2 train wells.)* **Try:** ML ▸ regression, pick ≥2 train wells + curves ▸ Compare.
- [ ] **(16) Well-diagram track.** Any layout track can be set to **kind = Well diagram** (Layout editor ▸
      Track type). It draws casing/tubing/liner strings (with shoe symbols) + perforation ticks from the
      well's **COMPLETION** and **PERFORATION** aux datasets (Data ▸ Import aux data; value_num = OD in
      inches, depth_top..depth_base = the run). Renders in the log view **and** the composite/report SVG.
      Old saved layouts still load (kind defaults to "curves"). *(Verified: cargo check + tsc clean;
      renderer skips curves for diagram tracks so nothing draws underneath.)* **Try:** import a COMPLETION
      CSV, add a track, set it to Well diagram.
- [ ] **(8) Rock typing + SHF — increment 1.** Two pieces:
      **(a) Rock Typing module** (Petrophysics ribbon ▸ new *Rock Typing* group) — from φ + k writes
      RQI, PHIZ, FZI (Amaefule), Winland **R35**, PGS **PGEOM/PSTRUC**, an **RT class** (GHE fixed FZI
      bins or Winland port classes) and **PERM_RT** (class-grouped geometric-mean-FZI perm estimate).
      *(4 unit tests: FZI→GHE7 for φ0.2/k100, Winland R35→macro, perm predictor, MISSING handling.)*
      **(b) Cuddy FOIL SHF fit** (workspace ▸ **SHF Fit (Cuddy FOIL)**) — pools computed PHIE/SW/TVDSS
      across wells, fits **BVW = a·H^b** above the FWL with a BVW-vs-H log-log crossplot, and an optional
      **FWL scan** (Cuddy 1993 Eq 19) that finds the common contact. *(3 unit tests: recovers a known
      power law, rejects degenerate input, scan lands on the true 2000 m contact.)*
      **NOTE (per the reference doc):** the PGS exponent (3.5) and GHE bins are literature/recall values —
      flagged in the module doc for verification before field release.
- [ ] **(8) increment 2 — first chunk (2026-07-22):** **Lucia Rock-Fabric Number** module
      (Petrophysics ▸ Rock Typing, carbonate) — inverts the Jennings-Lucia transform analytically for
      RFN + a 1–3 class; completes the FZI / Winland / PGS / Lucia rock-typing quartet. *(1 new test:
      Lucia round-trips RFN 1.0/3.0.)* **Try:** run it on a well with carbonate stringers. *(A Mahakam
      phi-k perm preset was built and tested but PULLED from the repo — those are proprietary Pertamina
      Hulu Mahakam production constants; kept out per the client-data rule.)*
- [ ] **(8) increment 2 — SHF forms (2026-07-22):** the **SHF Fit** pane got a form selector — besides
      Cuddy FOIL it now fits **Brooks-Corey** (Sw = Swirr + (1−Swirr)·(He/H)^λ, via a Swirr-grid + log-log
      linear fit) and **Skelt-Harrison** (Sw = 1 − A·exp(−(B/(H+D))^C), via a compact Nelder-Mead) to the
      log-derived Sw-vs-height cloud, with a Sw-vs-H scatter + fitted-curve overlay and a params/R² table.
      *(3 new tests: Brooks-Corey recovers a synthetic curve, Skelt reaches R²>0.98 + monotone Sw, both
      reject too-few points.)* **Try:** SHF Fit ▸ pick Brooks-Corey / Skelt-Harrison. *(Increment 2
      remainder — Thomeer Pc fit, SCAL importers, Pittman full rX table, and Ward/histogram HFU
      clustering — is now all shipped; see the entries below. Task #158 is complete.)*
- [ ] **(8) increment 2 — electrofacies tie-in (2026-07-22):** two parts. **Rock Type from Cutoffs**
      module (Petrophysics ▸ Rock Typing) — a Vsh + PHIE cutoff ladder → **RT_LOG** (1 best / 2 moderate
      / 3 non-net), to propagate rock types to uncored intervals. **Facies Tie-in** pane (workspace ▸
      *Facies Tie-in (RT confusion)*) — cross-tabulates the predicted log RT against a reference/core RT
      curve across wells and reports the **confusion matrix + dominant-class purity** (the check that
      the log classification faithfully reproduces core rock types). *(3 new tests: the cutoff ladder
      classifies clean/moderate/shaly correctly, the confusion tally scores purity, empty input is
      rejected.)* **Try:** run `rt_cutoff` to make RT_LOG, then Facies Tie-in ▸ RT_LOG vs your core RT.
- [ ] **(8) increment 2 — SCAL importers (2026-07-22):** **Import SCAL…** (Data ▸ Import Data) now
      takes **multiple files** and **three formats** (or **Auto-detect** per file): the existing flat
      PC/SW CSV, the **porous-plate wide table** (Corelab-style: preamble junk tolerated, pressure
      columns 1…150 psi as headers, one row per plug with Sample/Depth/Perm/Poro, cells = brine Sw
      %PV — unpivoted to long Pc points), and **centrifuge per-plug blocks** (SAMPLE/DEPTH/PERM/PORO
      key-value lines then a Pc/Sw table; several blocks per file, or multi-select one file per plug —
      the digitized-workbook shape). All selected files land in ONE combined replace-write of the
      well's `scal_pc` rows, then the Leverett-J fit runs over the pooled points as before. Lettered
      plug ids ("12A", "S-16A") keep their numeric part; %PV and %-porosity auto-convert; a bad file
      fails the whole import (nothing partial) and names the file. Also fixed on the way: a `PORO`
      header now resolves as porosity in every core/SCAL CSV import (it previously matched no alias).
      *(6 new tests: wide-table unpivot incl. a missing cell, headerless-file rejection, two-block
      centrifuge parse with no metadata leak between plugs, table-less block rejection, the format
      sniffer on all three shapes, multi-file import + replace-not-append + bad-file atomicity.)*
      **Try:** Import SCAL… ▸ multi-select your W-MND-1 porous-plate/centrifuge CSV exports ▸
      Auto-detect ▸ Import & Fit; then re-import to confirm points replace, and check SHF Fit sees
      the pooled cloud.
      **Post-review hardening (same day, ultracode 3-lens adversarial review — 10 confirmed
      findings, all fixed):** (1) an import that parses ZERO points now refuses the replace-write
      instead of silently wiping the well's existing SCAL data; (2) auto-detect no longer misroutes
      files whose cover sheets contain "No. of Samples,6"/"Sample Type,plug" lines — the centrifuge
      verdict now needs corroboration (a numeric DEPTH/PERM/PORO key-value line or a bare PC/SW
      header); (3) merged centrifuge files where the table header appears only above the first plug
      no longer silently drop plugs 2..N (header carries over); (4) repeated per-page header rows
      and numeric "Average" footers in wide tables no longer import as phantom Sw points (a data
      row must carry a sample id or depth); (5) regional Excel formats parse: ';' list separator
      (sniffed from line 1) and ',' decimals/thousands ("2,695.3", "98,5", "1,000"); (6) the flat
      parser keeps lettered plug ids ("12A"→12) like the other two. The dialog also now warns: ONE
      lab fluid system per import (mixed air-brine + mercury multi-selects would bias the pooled
      J-fit). *(+7 tests, suite 211 passed / 0 failed, tsc EXIT 0.)* **Deferred to the Thomeer /
      J-from-SCAL chunk:** a per-row fluid-system/IFT column in `scal_pc` (schema migration) so
      mixed-system imports can be stored and standardized properly, per the reference doc's long-
      table spec. *(→ delivered same day, see the Thomeer entry below.)*
- [ ] **(8) increment 2 — Thomeer Pc fit (2026-07-22):** new **Pc Fit (Thomeer)** pane (workspace ▸
      add pane). Fits the Thomeer (1960) hyperbola **Bv = Bv∞·exp(−G/log₁₀(Pc/Pd))** per plug over
      the scoped wells' imported SCAL Pc points (Bv = φ·(1−Sw); poro-less plugs are skipped and
      counted, not silently dropped). Per-plug table (row click selects) + the **Bv-vs-Pc QC plot**
      with the fitted hyperbola and Pd marker + the **Pd–G plane** — the Thomeer-class rock-typing
      crossplot. Also reports the Swanson apex (Bv/Pc)max and **Swanson k = 399·(Bv%/Pc)^1.691**
      (constants flagged: verify vs Swanson 1981 before field release, same policy as PGS). ONE
      pore system per plug this increment; multi-modal stacking (2–3 systems, dBv/dlogPc detection)
      is a later increment. **Schema:** `scal_pc` gained per-row **`system` + `ift`** columns
      (ALTER-migrated on old projects; the deferred review item) — the Import SCAL dialog now has a
      **Fluid system** select (air-brine 72 / air-mercury 367 / oil-brine 26 / custom) that
      auto-fills the sigma·cosθ and stamps every stored point. *(3 new tests: synthetic-hyperbola
      recovery pd/G/Bv∞ + R²>0.98, too-few/uninvaded rejection, DB-level grouping + poro-less skip
      + system/ift round-trip.)* **Try:** import MICP as
      Air-mercury ▸ Pc Fit (Thomeer) ▸ Fit — check the Pd–G clusters against your rock types.
      **Post-review hardening (same day, ultracode 2-lens review — 7 confirmed findings, all
      fixed):** (1) **Pc now standardizes to Hg-air equivalent (×367/σcosθ) BEFORE fitting** — the
      review caught Swanson k being applied to raw air-brine/oil-brine Pc (16–88× inflation) and
      the Pd–G plane mixing lab systems; G is scale-invariant so only Pd/apex move, and plugs from
      any system now share one comparable plane. Rows without a recorded σcosθ fit raw, show
      "(raw)" in the new System column, and get NO Swanson k. (2) Plugs group per **well_id** (two
      same-named wells no longer pool) and numbered plugs key on the sample number alone (blank
      depth cells no longer split a plug). (3) The long parser **forward-fills merged-cell plug
      context** (sample/depth/perm/poro on first row only — the common Excel export shape). (4)
      Entry-truncated curves flag **Pd ⚠ (pinned at a search bound)** instead of posing as resolved
      entries; plateau-only data no longer reports R²=0 for a perfect constant fit. (5) "Other"
      fluid system clears the σcosθ field (no stale preset silently stored). (6) perm/swanson_k
      typed `number | null` (NaN→null over IPC). *(+2 tests: air-brine plug recovers the same
      Hg-equivalent Pd as its mercury twin & legacy no-ift rows suppress Swanson; merged-cell
      forward-fill. Suite 216 passed / 0 failed; tsc EXIT 0.)*
- [ ] **(8) increment 2 — Pittman rX + HFU clustering (2026-07-22, closes task #158):** two pieces.
      **Pittman pore-throat radii** — new `pittman_rx` module (Petrophysics ▸ Rock Typing) writes the
      full **Pittman (1992) r10…r75** family (PR10…PR75 µm, each log₁₀ rX = C0 + C1·log₁₀ k + C2·log₁₀ φ%),
      an **APEX** selector (r10…r75, default r35) → **RAPEX** + its Hartmann-Beaumont **RT_PITT** port
      class. The r35 row (0.255/0.565/−0.523) matches the reference doc; the full table is transcribed
      from Pittman 1992 and flagged verify-before-release. **HFU Clustering** — new **HFU Clustering
      (FZI)** pane (workspace ▸ add pane). Reads the scoped wells' **core φ-k** (routine core analysis,
      not log curves), computes FZI, and partitions log₁₀(FZI) into K units by **Ward** (exact
      minimum-variance K-partition via DP — the global optimum, no greedy drift) or **histogram**
      (boundaries at the log-FZI histogram antimodes). Per-HFU table (FZI min/max, geometric-mean FZI,
      φ mean, and the Amaefule perm-transform R²) + the **RQI–φz** unit-slope crossplot coloured by HFU
      + the **log₁₀ FZI histogram** with the cut lines; row click highlights a unit. Read-only (writes
      no curves). *(10 new tests: Pittman r35 vs the published regression, apex-selector switching, Ward
      DP splits two separated bands + recovers each k, histogram finds the bimodal valley, invalid-plug
      skip + distinct-level cap note, empty-input error.)* **Try:** run `pittman_rx` (pick APEX) for the
      radius family; then HFU Clustering (FZI) ▸ pick Ward or Histogram + K ▸ Cluster — check the RQI–φz
      unit-slope lines and the FZI histogram breaks against your rock types.
      **Post-review hardening (same day, ultracode 4-lens adversarial review — 6 confirmed findings, all
      fixed; 2 refuted correctly):** (1) the **histogram path could emit an empty interior HFU**
      (two valleys flanking an empty bin gap) → non-contiguous ids like {1,3} and a boundaries/clusters
      count mismatch; ids are now remapped to contiguous 1..K and boundaries are recomputed from the
      final assignment (one cut per populated pair) for BOTH methods. (2) the selected-row highlight
      (`ml-diag`) was a no-op outside `.ml-confusion` tables → CSS broadened to cover plain mc-table
      selection rows (also repairs the Thomeer pane's identical latent no-op). (3) FZI_gm unit-slope
      lines now **clip to the plot rectangle** (a line whose slope-1 extension overshot could paint over
      the axis label/frame). (4) the pane now **redraws its canvases on resize** (was stale/blurry until
      a row click). (5) frontend histogram bins aligned to the backend clamp (8–40) so bars and cut
      lines share resolution. *(+1 regression test locking HFU-id contiguity across an empty gap. Suite
      227 passed / 0 failed; tsc EXIT 0.)*
- [ ] **Correctness — RT ≤ 0 → +Infinity in the Sw modules (2026-07-22, closes AUDIT-2026-07-21):**
      the three deterministic saturation modules (`sw_arch`, `sw_indo`, `sw_sim`) only screened
      **missing** RT (NaN). A genuine RT value **≤ 0** — almost always a null coded as `0`, or a bad
      processing artifact — flowed through: `sw_arch`'s `(a·Rw/(φ^m·RT))^(1/n)` and `sw_indo`'s
      `1/(RT·…)` both **diverged to +Infinity**, and since the "missing" test is NaN-only, +Inf leaked
      into the *unlimited* raw curves (`SWT_ARCH` / `SWE_INDO`) and **poisoned catalog min/max and plot
      autoscale** (the *limited* SWT/SWE looked fine because `limit()` clamps +Inf → 1.0, which masked
      it). `sw_sim` instead let the Newton-Raphson solver diverge and silently drop the sample. **Fix:**
      added `r <= 0.0` to each module's input guard, so an RT ≤ 0 sample is dropped to **missing (NaN)** —
      exactly matching the existing convention already used by `sw_rtc` / `sw_imts` (LRLC modules) which
      guard `rt_i <= 0.0`. *(Proven complete: an f32-sourced RT can't overflow f64 even at the smallest
      positive value, so no tiny-positive-RT can sneak a +Inf through; the LAS null −999.25 is negative
      → caught. Downstream contract verified safe — `classify_sample` already treats a missing SWE as
      "exclude from PAY", so a garbage RT that used to read as a fabricated `Sw=1.0` water sample now
      simply drops out; net pay is unchanged and average-SWE-over-reservoir is if anything cleaner.)*
      **Verification:** +3 regression tests (RT = 0 *and* −5 → NaN, never ±Inf, in all three modules);
      **suite 230 passed / 0 failed / 7 ignored**. Ran a 3-lens adversarial review (physics / downstream
      contract / edge-cases, 2 skeptics per finding, static-read only) → **0 confirmed, 7 refuted**.
      Two accurate-but-inconsequential observations were recorded, not fixed: *(i)* for the
      doubly-degenerate `(PHIE<0.005 AND RT≤0)` sample the porosity-state branch order makes `sw_arch`→NaN
      but `sw_indo`/`sw_sim`→SWE=1.0 (a non-reservoir sample excluded from pay either way; unifying it
      would mean restructuring `sw_arch`'s tested branch for zero benefit); *(ii)* `resolve_rw` could
      emit +Inf only at FTEMP = *exactly* −21.5 °C in the non-default MEASURED/SALINITY mode
      (physically impossible, pre-existing, orthogonal to this fix). **Try:** load a well whose deep
      resistivity has a zero/null streak and run `sw_arch` — the streak now reads as a gap in `SWT_ARCH`
      instead of pinning the curve autoscale to a huge number.
- [ ] **AUDIT-2026-07-21 full-QC triage — backend robustness batch 1 (2026-07-22):** a 65-finding
      parallel QC audit was triaged against current code (3 already fixed incl. the RT≤0 one above; 51
      safe-to-fix; 6 need your sign-off; 1 needs a live 100-well run; 4 feature-work). **This batch = 12
      safe backend fixes, none of which change any valid interpretation value** (suite 236/0/7):
      **(1)** `vsh_dn` now skips a **degenerate matrix/shale/fluid triangle** (`|c−d|<1e-6`) instead of
      writing ±Infinity into the unlimited VSH_DN (was poisoning catalog min/max + autoscale, same class
      as the RT≤0 bug). **(2)** `ftemp_grad` BHT mode skips a **TD_BHT ≤ 0** zone override (was a
      finite-looking ±Inf FTEMP). **(3)** `perm_wyllie_rose` now skips **negative PHIE** uniformly — the
      integer MORRIS_BIGGS/TIXIER exponent used to fabricate a plausible PERM from it while TIMUR NaN'd it.
      **(4)** `perm_transform` emits **MISSING instead of +Infinity** when `10^(PT_A·φ+PT_B)` overflows the
      f32 cast (reachable at in-range PT_A=100/PT_B=5). **(5)** `nphi_env_corr`'s FTEMP is now a
      **computed-only** input (a raw degF FTEMP can no longer be silently applied as degC), matching
      gascorr. **(6)** SandiMin **output prefix is upper-cased** so a re-cased prefix can't leave a stale
      curve. **(7)** the four computed-curve **delete-then-append writers now DELETE case-insensitively**
      (`upper(curve_name)`), closing the root-cause shadow-row bug where a re-cased equation output left a
      duplicate row that could silently win; the log-set restore subquery too. **(8)** curve-edit
      `locate_curve` got a deterministic `ORDER BY`. **(9)** **LAS export** looks up columns by upper-cased
      name, so a mixed-case computed curve ("Vsh_final") exports its real values instead of an all-NULL
      column. **(10)** Monte Carlo `summarize()` returns **NaN (→ "—")** for a dry/no-data metric instead
      of a fabricated 0.00. **(11)** the IMTS method doc's clay-term formula fixed to divide by Sw (matches
      code). *(+6 new tests locking the guards. No TS changed, so tsc unaffected.)*
- [ ] **AUDIT-2026-07-21 — import-robustness batch 2 (2026-07-22):** five importer fixes so one bad row no
      longer aborts a whole import, all mirroring existing verified patterns (LAS `depth_keep_indices`
      sanitize + the locations importer). **(1)** Core-CSV import **dedups duplicate plug depths** (first
      kept) instead of aborting the well's core import on the `core_data (well_id, depth)` PK. **(2)**
      Deviation-survey import **dedups duplicate station MDs**. **(3)** DLIS import **sanitizes each frame's
      depth** (drops non-finite + dedups) so one bad sample can't abort the file. **(4)** Tops import is now
      **transaction-wrapped** like the sibling Locations importer — a mid-file error no longer strands half
      the tops. **(5)** Tops import now **skips a blank WELL cell in a multi-well file** (was misrouting it
      to the selected well, silently attaching a top to an unrelated well) and reports the dropped count.
      *(+2 tests updated for the new `has_well_column` flag; suite 236/0.)*
- [ ] **AUDIT-2026-07-21 — dead-code removal batch 3 (2026-07-22):** deleted two dead source files and
      their IPC surface. **(1)** `petrophysics.rs` was fully dead (never declared as a `mod`, zero
      references; its math — linear Vsh, density porosity, plain Archie — is long since live in
      `modules.rs`). **(2)** `inversion.rs` was a hardcoded-stub solver (`run_stochastic_inversion`
      returned a fixed `[0.25,0.15,0.20,0.40]` regardless of input) still exposed over IPC as
      `start_inversion`/`get_inversion_status` with **zero frontend callers** and a latent
      `tokio::spawn`-from-sync-command panic; removed both commands from the handler, the
      `.manage(inversion::new_registry())`, the `mod`, and the file. *(No behavior change — nothing
      called either. Suite 236/0.)*

## Round 2 — panes, shift-select, MC plot props + table + polish (2026-07-21, Jauhar feedback batch #2)

Follow-up batch after the first round: (1) Shift-select was painting a native blue text
highlight; (2) the "4 main panes" clarification — they should always **STAY** (never vanish when
other panes pop/close) but stay manually resizable; (3) MC + other UI polish toward the **Cutoff
Sensitivity** panel look (image 3); (4) MC — add **plot property panels** (resize, colour, axes)
for the histogram + tornado, and make the histogram look like a **real histogram**; (5) MC — move
the **results table to the very bottom**. **tsc EXIT 0; browser-verified on an isolated vite (port
1428, never touched your 1420). Nothing committed.**

- [ ] **Shift-select no longer turns blue.** Range-select (Shift-click) was triggering the browser's
      native text selection across the well labels. Added `user-select: none` to the tree nodes and
      both tree bodies (Wells + Tops). *(Verified: `.tree-node` computes `user-select: none`.)*
- [ ] **The 4 anchor panes now STAY.** Wells / Tops / Processing / Performance can no longer be closed
      — the ✕ is hidden on their window header, Close panel/Close window are dropped from their
      right-click menu, and they can't be floated out of the sidebar. So opening/closing other windows
      can never make them disappear. They remain **freely resizable** (drag the splitter; the
      minimum-width floor only stops full collapse). A restored old layout that had lost the Wells pane
      re-adds it. *(If you'd rather they could still be closed, say so.)*
- [ ] **The anchor panes keep their WIDTH when other panes/windows pop up or close.** dockview lays out
      proportionally (that option is hardcoded on and not exposed), so opening/closing a pane was
      reflowing the sidebar. The fix pins each anchor group to a **fixed width (min == max)**, which
      dockview excludes from redistribution entirely — so no add, close, or window resize can move it.
      You can still resize it: grabbing the splitter (`.dv-sash`, caught in the capture phase so the
      drag goes live) unlocks the anchors for the drag, and they're re-pinned at the new width on
      release. *(Two earlier heuristic attempts — restore-on-layout-change — held on close but not on
      add, because an add fires extra reflow passes. This fixed-width approach needs no heuristic.
      Verified end-to-end against the real dockview build in isolation: add 4 panes → held 260; close 2
      → held 260; real DOM sash-drag → 340; add 3 more → held 340.)*
- [ ] **MC results table is at the very bottom.** Order is now **histogram → tornado → table**.
      Click a table row to plot that well-zone's HPV distribution in the histogram above.
      *(Browser-verified: the three result blocks render in that DOM order, table last.)*
- [ ] **Histogram is a real histogram now.** Added a frequency **y-axis** (nice-stepped count ticks
      0/20/40/… with a "count" title), horizontal **gridlines**, x-axis HPV min/mid/max labels, and the
      P10/P50/P90 markers. *(Browser-verified by capturing the canvas draw calls: count ticks + "count"
      + "HPV" + P10/P50/P90 all drawn; canvas re-rasterises crisply on resize.)*
- [ ] **⚙ Plot properties on both plots.** A gear on the histogram and the tornado opens an inline
      panel: **Height (resize)**, **colour** (bar colour / low-side + high-side bar colours), and
      toggles — histogram: P-markers, gridlines, y-axis; tornado: row stripes, ρ labels. Height 0 on the
      tornado = auto-size to the parameter count. *(Browser-verified: height 220→320 px live; bar colour
      set to #1f77d0 and the sampled bar pixel read back rgb(31,119,208).)*
- [ ] **MC UI polished toward the Cutoff panel.** Full-width brown **Run** button (matches Compute),
      `form-control`-styled selects/inputs, and tidier uncertainty-parameter rows (flexible param name,
      compact distribution pill). *(Browser-verified: Run button is full-width with the accent
      background.)*
- [ ] **Rw-for-PHIE gating still holds** after the tornado rewrite. *(Re-verified by capturing drawn
      labels: RW is drawn for HPV — it drives HPV via Sw — and dropped for Avg PHIE.)*

## Pane layout + MC/workflow polish + well-scope selector (2026-07-21, Jauhar feedback batch)

Jauhar's batch: (1) panes — two "Wells", tops-in-wells, non-resizable anchors; (2) MC — polish,
percentiles, table, ugly/stretching plots, and Rw showing sensitivity for PHIE it doesn't affect;
(3) workflow polish; (4) cross-cutting: stop checklisting wells one-by-one — use groups + pins.
**tsc EXIT 0; Rust montecarlo suite 7/7 (1 new: configurable percentiles); cargo check EXIT 0;
browser-verified on an isolated vite (port 1428, never touched your 1420) — see the proofs noted
per item. Nothing committed.**

### Panes
- [ ] **No more "two Wells".** The wells pane had a static "WELLS" title *and* the ObjectTree's own
      "Wells (N)" header — plus a **concurrent-refresh race** that appended the header (and every well)
      **twice**. Fixed both: dropped the static title; added a generation guard to `ObjectTree.refresh`.
      *(Browser-verified: 1 header, 9 well nodes — not 18 — for a 9-well group.)*
- [ ] **Tops is its own pane now.** Split out of the combined "Wells & Tops": a standalone **Tops** dock
      panel that follows the selected well through app state, docked directly below the **Wells** pane.
      It's a real dockview panel — drag it anywhere, tab it, resize it. *(Verified: panel list shows
      separate "Wells" and "Tops".)* Old saved layouts get the Tops pane auto-added on open.
- [ ] **Sidebar panes are resizable.** The Wells / Tops / Processing / Performance anchors were locked
      at a fixed width (min == max). Now they have a **minimum-width floor only** — drag the splitter to
      any width; they still won't collapse or auto-stretch when a neighbour closes. *(This reverses the
      earlier fixed-width lock, per your request — tell me if you preferred fixed.)*
- [ ] **★ pin a well** in the Wells pane (the star to the left of each name; persisted per project).

### Well scope — no more well-by-well checklists (imagine 2000 wells)
- [ ] Every run dialog (**Monte Carlo, Workflow, every module pane, Multimin, ML-apply, Cutoff,
      Summary, Report-batch**) now shows one compact **scope selector** instead of a checkbox per well:
      **Group** (defaults to the active group) · **★ Pinned** · **Selection** (your Ctrl-click set) ·
      **All** · **Custom…** (a searchable checklist for the rare precise pick), with a live "N wells"
      count. *(Verified: defaults to the active group and resolves 9 wells.)*
- [ ] Groups already existed and already scoped dialogs — the gap was purely the UI. **Pinned wells are
      new** (a `well_pins` table + ★ toggle) since a reusable pin-subset didn't exist before (the old 📌
      is only the workspace-follow toggle, unchanged). ML's *Train wells* and Auto-correlation's *targets*
      are deliberately **not** scope-swapped (they're a different concept, not "run on N wells").

### Monte Carlo
- [ ] **Rw no longer shows sensitivity for PHIE.** This was **not** a calculation bug — Rw is correctly
      routed only to the saturation step, so the PHIE *curve* is independent of it. The tornado was
      rendering statistically-insignificant **noise** (finite-N Spearman ≈0.05) and zero-width OAT rows.
      Fixed at the display layer, principled: a parameter appears for a metric **only if its one-at-a-time
      sweep actually moves that metric** (the sweep is deterministic → a non-contributor moves it by
      exactly 0), and ρ labels show **only above the significance floor** (1.96/√N). *(Browser-verified by
      capturing the canvas text: the tornado draws Rw for **HPV** — it does drive HPV via Sw — but **drops
      Rw for Avg PHIE**, while GR_SH/RHO_MA/NPHI_SH/GR_MA remain.)*
- [ ] **Percentile option.** A **Percentiles** dropdown in Settings (P10/P90 default, P25/P75, P5/P95,
      P1/P99) drives both the reported spread **and** the tornado's input sweep. *(Verified: switching to
      P5/P95 re-labels the table columns and the histogram markers.)*
- [ ] **Tidier table.** P50 as the headline number with the (P10–P90) band on a quiet sub-line, a new
      **Gross** column, tabular figures, zebra rows, and dynamic Pxx headers.
- [ ] **Plots don't stretch on pane resize any more.** Both the histogram and tornado canvases now
      re-rasterize to the pane's width via a ResizeObserver (before, the browser scaled a stale bitmap →
      the blur/stretch you saw). *(Verified: shrinking the pane redrew the bitmaps 618→484 px.)* Tornado
      also got rounded bars, alternating row shading, and a height that tracks the parameter count.

### Workflow
- [ ] Same scope selector replaces the well checklist; the rest of the builder (steps, grid, cons in/out)
      is unchanged.

## Monte Carlo parameter sensitivity + tornado (Wave B #13, 2026-07-21)

The uncertainty engine already ran N realizations but **threw away the parameter draws** — it only
kept the resulting P10/P50/P90. It now retains them and reports **which parameters actually drive
the result**. **tsc EXIT 0; Rust montecarlo suite 6/6 pass (3 new); off-by-default so existing runs
are byte-identical.** Nothing committed yet.

- [ ] **Open Monte Carlo** (Advance ribbon → Monte Carlo). There are two new checkboxes under a
      **Sensitivity** row — *Rank sensitivity (Spearman)* and *Tornado sweep (P10 / P90)*, both on by
      default. Add one or two uncertain parameters (e.g. GR_MA, GR_SH, RW), pick a well, **Run**.
- [ ] **Tornado chart** appears below the HPV histogram with a **Zone** and **Metric** selector
      (HPV / Net pay / NTG / Avg PHIE / Avg SWE). With the tornado box ticked it shows **range bars**:
      each parameter swept to its P10↔P90 with the others held at their medians, sorted most-influential
      on top, around a common **base** line, annotated with the Spearman ρ. Untick *Tornado* (leave
      *Rank sensitivity* on) → it falls back to **signed correlation bars** on a −1…+1 axis.
- [ ] **Sanity checks**: (a) the parameter you'd expect to matter most (usually GR_SH or Rw) sits at
      the top; (b) switching **Metric** re-sorts and re-scales; (c) switching **Zone** redraws for that
      well-zone; (d) a parameter you give **zero spread** (sd = 0) shows ρ = NaN / no bar (it can't be
      ranked); (e) unticking **both** boxes → no tornado section, and the headline P10/P50/P90 table is
      unchanged. Verified: Spearman sign+magnitude, tornado low≤base≤high ordering, and opt-out
      reproducibility are covered by unit tests; the live chart render awaits your click-through.

## Highlight tool + ribbon overflow + trademark scrub + typography (2026-07-21)

B2 UI/workflow polish + two follow-ups. **tsc EXIT 0; `cargo check --tests` EXIT 0; Rust 177 pass / 0 fail.** Nothing committed yet.

- [ ] **Ribbon overflow chevrons (Office-style)** (ribbon.ts, styles.css). When the window is too narrow
      to show all the tools on a tab, the raw scrollbar is gone — a boxed **‹ / ›** appears at the
      overflowing edge and scrolls the group row a page at a time (like PowerPoint's ribbon). Test: narrow
      the window until a tab's groups don't all fit → a **›** box appears at the right edge; click it →
      the row scrolls and a **‹** appears at the left; at the end only **‹** shows. Switch tabs / resize →
      the chevrons re-evaluate. (Verified live: at 720px the Petrophysics row overflows 238px, right
      chevron shows at scroll-start, left appears after scrolling, correct box at the right edge.)

- [ ] **Highlight tool — colored depth bands in the Log View** (new `highlightsOverlay.ts`; `highlights`
      table + `list/upsert/delete_highlight` in db.rs/lib.rs; IPC in ipc.ts). Open a **Log View**, then
      in that view's toolbar click **🖍** (next to the 🏷 tops button). Drag vertically over a depth
      interval → a **translucent colored band** appears across the tracks and an **Edit highlight**
      dialog opens. Give it a label (e.g. "Pay") + color → **Save**. Add a couple more with different
      colors. Test: (a) bands render across all tracks, translucent so curves read through; (b) they
      **track pan/zoom**; (c) switch to another well and back → bands **persist** and reload; (d)
      **double-click** a band → dialog to recolor / relabel / edit top+bottom / **Delete** / **Convert
      to zone**; (e) **Convert to zone** creates a zone (check it appears in **Zones** / pay summary);
      (f) **Ctrl+Z** undoes add / edit / delete / convert; (g) **🖍 and 🏷 are mutually exclusive** —
      turning one on turns the other off. Bands sit **below** the tops lines so tops stay legible.
- [ ] **Text sharpness — font hinting** (tauri.conf.json `additionalBrowserArgs`). You flagged text as
      slightly fuzzy/washed-out. I confirmed the CSS is clean and contrast is already high (~12.9:1), so
      it's not a color issue — the softness is Chromium's GPU grayscale AA (WebGPU forces GPU on) plus
      Windows display scaling. I added `--font-render-hinting=medium`. **This only takes effect on a full
      relaunch** (`npm run tauri dev` restart). Test: relaunch, eyeball the panel text vs before. If it's
      still soft, check **Windows Settings ▸ Display ▸ Scale** — at 125%/150% the webview raster-scales;
      tell me and we can add a text-size control or bump the base font. (Not verifiable from my side —
      the browser tools can't reproduce WebView2 rendering.)
- [ ] **AspenTech trademark scrubbed repo-wide (keeping Loglan)** (per your request — "except loglan").
      The prior-tool name is now gone from the whole tree — shipped app, code comments, and dev docs —
      except: **Loglan / `.lls`** (kept deliberately: SandiBumi runs Loglan, so those stay), your real
      data-folder paths in test fixtures (can't rename your disk), the English word "geology", and your
      own verbatim words in `Review.txt`. The comment/doc pass replaced the vendor name with neutral
      wording ("the reference suite", "commercial suite", etc.). Nothing to click-test — grep the repo
      for the old name and you'll only find the exceptions above. Test the one user-visible change: hover
      the **DB Inspector** ribbon button + open **Help** → reads "spreadsheet-style".

## 540-well test — perf & crash fixes (2026-07-21)

From your ~540-well stress test. A read-only 5-agent diagnosis traced every "not responding"
freeze to one root cause (heavy commands run **synchronously on the UI thread**) plus a specific
speed bug per subsystem. **Rust 176 pass / 0 fail; tsc EXIT 0. Nothing committed.** The async
piece can't be verified without running the app, so these especially want your click-through.

- [ ] **Field Dashboard no longer crashes on ~540 wells** (dashboardPanel.ts, summaryDialog.ts).
      "Compute failed: TypeError: Cannot read properties of null (reading 'toFixed')" was a zero-net
      zone row whose avg VSH/PHIE/SWE come back as NaN → serde encodes non-finite floats as JSON
      **null**, and the old `Number.isNaN` guard doesn't catch null. The formatter now shows "—" for
      null/NaN. Test: **Field Dashboard ▸ Compute** across all wells → the grid renders (empty
      aggregates show "—"), no crash. Same latent fix in the single-well **Cutoffs & Summary** table.
- [ ] **Field Dashboard is fast now** (workflow.rs `stats_only`). Compute took >5 min because it
      secretly wrote 3 FLAG_* curves per well (~1,600 DB transactions) on every press, though the
      panel only reads the returned numbers. It now computes the stats without persisting anything.
      Test: Compute on all wells → **seconds, not minutes**; tweak a cutoff and re-Compute → still
      fast. Behavior change to note: the dashboard no longer leaves FLAG_* curves in the wells —
      persist those from **Cutoffs & Pay Summary** (unchanged) when you actually want them written.
      Test `pay_summary_stats_only_persists_nothing`.
- [ ] **Workflow chain runs without freezing the app + live progress works** (lib.rs — `DbState` is
      now `Arc<Mutex<Connection>>`; `run_workflow_chain` now runs on a background thread). Build a
      chain (e.g. vsh_gr → phi_dn → sw_indo) and run it on a batch of wells: (a) the window stays
      **draggable/responsive** during the run (was frozen "not responding"), (b) the **progress bar
      advances** step-by-step, and (c) **Cancel** actually stops it. This is the first of the async
      conversions — import, dashboard, multimin, Monte Carlo and equations will follow the *same*
      pattern, so confirming this one works validates the whole approach.
- [ ] **A chain of many wells now finishes in seconds, and Cancel is near-instant**
      (workflow.rs two-phase batched write; equations.rs `create_log_sets_batch` +
      `write_computed_curves_versioned_batch`; chain.rs). The 30-min chain / 30-min-to-cancel
      was ~2 fsync-bound DB transactions **per well** per step (≈1,000 commits on 500 wells). Each
      step now computes every well in parallel (reads only), then does **one** batched versioned
      write — ~2 commits per step. Cancel is checked per well, so it drains in a well or two.
      Test: run a chain (vsh_gr → phi_dn → sw_indo) on a big well set → **seconds**, and **Cancel
      stops almost immediately**. Test `batched_module_run_writes_every_well_correctly` proves two
      wells write distinct, un-crossed, correctly-versioned results.
- [ ] **Cancel empties the progress bar** (workflowDialog.ts). Pressing **Cancel** now clears the
      bar to empty (and hides it) the instant you click, then the status confirms "Cancelled at
      step N". Test: start a chain, hit Cancel → the bar goes empty right away.
- [ ] **Input/Output are now "cons" (constellation) pickers, not free text** (workflowDialog.ts,
      moduleDialog.ts, new `list_log_set_names` command). Terminology changed from "set" to **cons**
      throughout the UI (Workflow, module dialogs, Curve Catalog "Constellations"). **Input cons** is
      a strict dropdown of existing constellations (blank = latest values — you can only read from
      one that exists). **Output cons** is an editable combobox: pick an existing constellation *or*
      type a brand-new name. Both are filled from the project's real constellation names. Test: open
      Workflow / any module → Input cons lists your existing constellations; Output cons suggests
      them but also accepts a new name like `FINAL2`.
- [ ] **Universal Processing panel — live per-well progress + Cancel for the whole run**
      (new `jobs.rs` registry + `list_jobs`/`cancel_job`; `processingPanel.ts`; ribbon **Processing**
      button). New dock panel that shows, for a running workflow chain: a **progress bar with an
      integrated Cancel**, the current **"Step 2/3: sw_indo"** line, a live **counts row**
      (▶ running · ✓ done · ⚠ warned · ✗ failed · ⏳ pending), and a **details** toggle listing the
      *notable* wells (running/warned/failed) with messages — so you can see **which well failed and
      why** at 500-well scale without a 500-row dump. It **auto-opens** when you press Run in the
      Workflow Builder, or open it anytime from the **Processing** ribbon button. Cancel here shares
      the *same* flag as the run, so it stops the chain whether launched from the panel or the
      builder. This is the reusable spine: import, module runs, multimin, Monte Carlo and reports
      will each report into it as they move off the IPC thread. Test: Run a chain → the Processing
      panel opens and fills live; click a well's ⚠/✗ in details to read the message; hit Cancel →
      the bar stops within a well or two.
- [ ] **Processing panel: the step-boundary "pause" now says what it's doing** (workflow.rs).
      Each chain step computes every well (bar fills), then does ONE big batched DB write with no
      per-well signal — so the bar used to sit at the boundary / 100% looking frozen. It now shows
      **"Writing N well(s)…"** during that write, so the wait reads as working, not stuck. Test: run
      a chain and watch between steps and at the end → the current line reads "Writing … well(s)"
      during the pause, then advances/completes.
- [ ] **Workflow Builder no longer shows its own redundant progress bar** (workflowDialog.ts). The
      inline `<progress>` bar is gone now that the Processing panel owns the live bar + Cancel; the
      builder keeps a one-line status ("Step 2/2: … — see Processing panel", "Done: …"). Test: run a
      chain → progress shows only in the Processing panel; the builder just shows a status line.
- [ ] **Hardware Health Monitor** (new `health.rs` + `health_snapshot` command; `healthPanel.ts`;
      ribbon **Health** button). A Petrel-PHM-style panel of four colour-coded gauges — **MEM System**
      (system memory %), **GPU Memory** (GPU video-memory current/budget %), **USER Objects** and
      **GDI Objects** (this process's handle counts vs the 10,000-per-process ceiling — the classic
      desktop-leak signal, raw count shown in the value). **Green < 60% · Yellow 60–80% · Red > 80%**,
      polled every 1.5 s. Open from the **Health** ribbon button (next to Processing). Metrics are
      Windows-only; any unavailable value shows **n/a** (so if GPU Memory reads "n/a" on your machine,
      tell me — the DXGI path is best-effort and I'll adjust). Note: this is GPU *memory* load, not
      engine-utilisation % (that needs PDH GPU counters — a possible refinement). Test: open Health →
      MEM/USER/GDI show live %; leave a few heavy panels open and watch GDI/USER climb.

## P0 senior-audit backlog — correctness & data-integrity fixes (2026-07-20)

The eight P0 findings from `AUDIT-2026-07-20.md` (ROADMAP §4b), plus the LAS-import
robustness residual (#118) that closing them surfaced, are implemented and unit-tested
(full lib suite **160 pass / 0 fail**, tsc clean). These are the ones that made answers
wrong or silently lost data, so they matter most for Mahakam work:

- [x] **MASK now blanks module INPUTS, not just outputs** (workflow.rs). Run **GR
      Normalize** (or **Log Predict**) with a BADHOLE / COND*FLAG curve set as the
      **Mask**. The well P3/P97 (and the KNN training set) are now computed from the
      unmasked samples only — casing/washout/hot-streak GR no longer shifts the two-point
      transform, so good-hole output stops drifting. For log_predict the repaired synthetic
      now survives \_inside* the masked (washout) interval it exists to fill, instead of
      being blanked there. Test: `mask_excludes_flagged_samples_from_gr_normalize_percentiles`.
- [x] **SW-height uses TVD and allows a subsea FWL** (satheight.rs). The sw_height module
      now takes an optional **TVD** input (defaults to measured depth when absent) and the
      **FWL** field accepts negative (subsea TVDSS) values. On a deviated well, height above
      the contact — and therefore SWH — is no longer optimistically overstated by ~1/cos(inc).
      Run on a deviated Mahakam well with the TVD curve mapped and confirm SWH rises vs the
      old MD-based result. Test: the negative-TVDSS deviated case in satheight.rs.
- [x] **Pay summary: thin-zone clamp + honest averages** (workflow.rs). Each sample's
      thickness is clamped to its overlap with the zone, so the last in-zone sample no longer
      bleeds a full step past the zone base and **net can never exceed gross** (sub-step-thick
      zones). SAND-row `avg_phie` is normalised over the thickness where PHIE is actually
      valid, so a sample with good VSH but missing PHIE no longer drags the average toward
      zero. Cross-check a well with thin zones / patchy PHIE against the old numbers. Test:
      `pay_summary_clamps_thin_zone_and_normalizes_avg_phie_over_valid`.
- [x] **Crash-safe curve writes** (db.rs `with_txn`). Every delete-then-append writer
      (computed curves, restore/delete log set, core/aux/SCAL/curve-sample/well-path inserts,
      group members, zones-from-tops) now runs inside a single BEGIN/COMMIT/ROLLBACK, so an
      app kill mid-write can no longer leave the DELETE committed but the re-append lost.
      Nothing to click — just note that a tauri-dev restart mid-run won't silently drop curves.
- [x] **IMTS clay-conductivity direction fixed** (lrlc.rs sw*imts). The excess-conductivity
      term now \_divides* by Sw (Waxman-Smits `Cw + B·Qv_eff/Sw`), so it grows as hydrocarbon
      displaces water instead of vanishing. IMTS SwE now sits at/just below Waxman-Smits in
      pay (the old `·Sw` form gave Sw^(n\*+1) and over-stated Sw exactly in the LRLC pay this
      method exists to find). Re-run sw_imts on an LRLC interval and confirm SwE dropped.
      Test: `imts_credits_clay_conductivity_in_pay_zone`.
- [ ] **DLIS null sentinels + no silent overwrite** (dlis.rs). DLIS absent/sentinel values
      (−999.25/−9999, non-finite, |v|>1e30) are screened to MISSING on import, and each DLIS
      frame gets its own run number so a frame-0 channel no longer silently replaces a
      same-mnemonic LAS curve — the status line reports "replaced N existing curve(s)" when a
      collision does happen. Re-import a DLIS over a well that already has LAS curves and check
      the replaced-count note.
- [ ] **SandiMin refuses under-determined models** (multimin2.rs). Selecting fewer than
      (components − 1) input tools is now rejected up front ("need at least N input logs to
      constrain M components") instead of solving to an arbitrary vertex, and per-sample the
      solver skips depths with too few live curves. Test: `rejects_underdetermined_request`.
- [ ] **LAS import survives duplicate/odd-depth files on BOTH stores** (parsers.rs, ingest.rs).
      Non-finite and duplicate depths are dropped (first occurrence kept) before insert, so a
      **spliced/merged LAS with a repeated depth section** imports instead of aborting on the
      `(well_id, depth)` PK — and the fix now covers the _generic_ store too (PEF/CALI/extra
      runs), which previously PK-failed silently and left the well without those curves. Extras:
      a Schlumberger **TDEP**-indexed (or any non-`DEPT`) file resolves depth via the first
      column; an auxiliary **MD/TDEP track** in a later column can no longer steal the depth
      role from the true first-column index; a file whose depth is entirely null now **errors
      cleanly** instead of creating an empty orphan well; and a non-monotonic depth (column 0
      wasn't really the index) is surfaced as an import warning. The dropped/duplicate/odd-index
      counts appear in the import status line and History. Import a re-spliced LAS and a TDEP
      file and confirm both load with the expected row counts. Tests:
      `duplicate_depth_las_imports_standard_and_generic_curves`,
      `all_null_depth_las_errors_without_creating_well`,
      `parse_las_2_auxiliary_md_curve_does_not_steal_depth`,
      `sanitize_dedups_signed_zero_depths`, `parse_las_2_tdep_index_populates_depth`.

## P1 — reliability (frontend state) (2026-07-20)

The three P1 reliability findings from ROADMAP §4b (stale plots, async races, listener
leaks). Frontend-only (TypeScript, `tsc` clean); these are async-lifecycle behaviors with
no unit tests, so they were hardened by an adversarial review (4 lenses → per-finding
skeptical verify: **6 confirmed / 0 refuted**, all fixed — including one real HIGH bug in
the first-pass init guard) plus a focused **second-pass verify of the fixes** (renderer-dispose

- sticky-reset lenses: **clean, 0 defects**). Click-through in `npm run tauri dev`:

* [ ] **Plots refresh after a module run, keeping their viewport** (histogramPanel,
      crossplotPanel, pickettPanel, correlationPanel, logViewPanel). Open a Histogram of
      PHIE (zoom in), then run a module/equation that recomputes PHIE (or import / undo —
      anything that bumps `dataVersion`). The plot now re-reads the new curve **in place**,
      preserving your current zoom/pan, instead of showing the pre-run curve until you close
      and reopen the panel. Each builder subscribes to `appState.dataVersion` and calls
      `reload(preserveView=true)`; a `dataPrimed` guard swallows the subscribe's immediate
      fire so nothing double-loads on open.
* [ ] **Fast well/curve/zone switching never shows stale data** (loadWell, reload,
      createPlot). Click quickly through 5 wells in the Log view, or spam curve/zone changes
      in the Crossplot/Pickett/Histogram. A slow earlier load can no longer land after and
      overwrite a newer selection — each async load captures a generation token before its
      first `await` and bails if superseded. And a viewport **reset** intent (switching wells
      / changing the curve) still fires exactly once even if a `preserveView` refresh commits
      first, via a sticky `resetPending`/`viewResetPending` flag — so you neither keep a stale
      zoom that should have reset nor lose a reset that a background refresh raced past.
* [ ] **Opening/closing Log panels doesn't leak listeners or GPU loops** (logViewPanel
      dispose, LogCanvasRenderer). Repeatedly open and close Log view panels. The renderer's
      `window` pointerup/pointermove handlers are now removed and its `requestAnimationFrame`
      loop cancelled on `dispose()` (previously leaked one set per open); disposing a panel
      _during_ WebGPU init now disposes the fully-initialized local renderer rather than a
      no-op on an already-nulled field. Nothing visible per-close, but memory/handler count
      stays flat over a long session.
* [ ] **Dialog Escape is scoped to the dialog — closes the P1 modal-Escape sliver** (modal.ts).
      The carried-over P1 sliver ("overlapping dialogs share one Escape handler"). The listener-
      leak half was already handled — `openModal` single-instances via `activeClose` (a new dialog
      closes the prior one and removes its keydown listener; no modal opens a nested modal, so
      there's no stack). The remaining gap: the dialog's Escape was a `document`-level handler that
      closed the dialog but didn't `stopPropagation`, so one Escape also bubbled to `window`/app-level
      Escape handlers. It now stops there — kept on the **bubble** phase so the numeric-edit guard's
      capture-phase `stopPropagation` still shields a dialog from closing while you edit a number
      field. Also tears down any in-flight title-bar drag listeners on close (no leak if a dialog is
      dismissed mid-drag). tsc clean. Test: start drawing a **map polygon**, open any dialog (⚙
      Properties, an import dialog…), press **Escape** → the dialog closes and the half-drawn polygon
      is **still there** (Escape no longer cancels it too); double-click a number field in a dialog,
      press Escape → you exit the field's edit mode and the **dialog stays open**; a second Escape
      closes the dialog.

## Polish — UX (veteran-interpreter friction) (2026-07-20)

Hardening-backlog **Polish** tier (ROADMAP §4b, was "P3"). Small, mostly-frontend fixes;
each tsc-clean. Mapped by a read-only investigation wave before implementing.

- [ ] **Cursor readout: real units + no more mangled values** (plotCommon.ts `formatValue`,
      viewerChrome.ts `renderReadout`, logViewPanel.ts). The log-view cursor readout used a
      blanket `toFixed(2)`, which flattened permeability 0.003 → "0.00" and showed no units.
      It now uses an adaptive significant-figures formatter (perm stays "0.003", RT reads
      "2151", φ "0.18") and appends the unit the curve catalog already carries (RT "ohm.m",
      PHI "v/v", RHOB "g/cc"). Units are cached per well-load and refresh on `dataVersion`.
      Test: hover the log view over a permeability track and a deep-resistivity track — values
      keep resolution and each shows its unit. (Values whose catalog unit is blank show no unit.)
- [ ] **Correlation: fresh well list + Ctrl+wheel zoom** (correlationPanel.ts). The Wells
      menu was built once and never refreshed, so a newly imported well never appeared. It now
      re-fetches the well list on `dataVersion` — new wells appear (and draw as strips), deleted
      wells drop out, active-group filter re-applies. Also added **Ctrl/Cmd+wheel zoom about the
      cursor** (same factors as the other plots); plain wheel still pans through depth. Test:
      open Correlation, import/deviation-load another well → it shows up without reopening the
      panel; Ctrl+wheel over the strips zooms at the cursor depth.
- [ ] **Processing history now covers every operation** (processLog.ts + call sites across
      ribbon / inspectorPanel / mlDialog / monteCarloDialog / workflowDialog / zonesDialog /
      topsEditor / mapPanel / cutoffDialog). The audit trail (History panel / QAT History)
      previously logged only LAS import/export, module runs, core shift, well header, curve edits,
      project/session, exports. It now ALSO records: **DLIS / deviation / SCAL / core / tops
      imports; equation runs; ML runs; Monte Carlo; workflow chains; log-set restore/delete; zone
      add/edit/delete + per-zone parameter overrides; manual tops add/edit/delete; cutoff-default
      saves; map-polygon→group assignment.** Test: perform each and confirm it appears in the
      History panel with the right `[kind]` and detail. (Batch/field-wide actions — equation, ML,
      MC, workflow, log-set, cutoffs — intentionally show no well name.)
- [ ] **Pickett v2 — properties dialog, typed M/Rw, configurable axes, Z-color** (pickettPanel.ts).
      The Pickett plot's RT/PHIE axes were hard-coded (0.1–1000 / 0.01–1) with no properties
      dialog. Now a **⚙ / right-click Properties** dialog sets RT & PHIE axis ranges, point size,
      and **color-by-curve** (rainbow/viridis, optional log-Z), persisted via plotprops. The
      toolbar gained **M and Rw** fields next to N — type them and the Sw=1 / iso-Sw lines follow;
      a two-point pick still fits the line and fills the same M/Rw fields (one shared source).
      Zoom/pan and the line survive a data refresh (P1 preserveView). Test: open Pickett on a well
      with RES_DEEP + computed PHIE; pick two points on the wet trend → M/Rw fill and lines draw;
      type a different Rw → lines follow; right-click → set RT axis 0.2–200 and color by SW/VSH →
      points recolor; reopen the panel → settings persist.
- [ ] **Pay-summary provenance — FLAG\_\* versioned + cutoffs recorded** (workflow.rs, backend).
      run*pay_summary wrote FLAG_SAND/RESERVOIR/PAY with the old in-place `write_computed_curve`
      — no version history, and the VSH/PHIE/SWE cutoffs that produced them were recorded nowhere.
      Now the explicit **Cutoffs & Pay Summary** run versions the three flags into a **PAYFLAG**
      log set whose provenance = module `pay_summary` + the cutoffs (in `log_sets.params_json`) +
      inputs, exactly like every other module output — so a re-run keeps history and any version
      is restorable/prunable from the Curve Catalog. The **Field Dashboard** and **report** passes
      set `skip_version` (field-wide QC side-effect) so they keep overwriting in place — no version
      churn per refresh. Test `pay_summary_versions_flags_with_cutoffs_in_provenance` (161 lib
      tests pass / 0 fail / 7 ignored; tsc clean). Click-through: run Cutoffs & Pay Summary on a
      well → Curve Catalog shows a PAYFLAG version whose provenance lists the cutoffs; re-run →
      version N+1; run the Field Dashboard → FLAG*\* update but no new version piles up.

## Performance (field-scale speed) (2026-07-20)

Hardening-backlog **Performance** tier (ROADMAP §4b, was "P2"). The rest of the tier (#128–132)
changes DB/IPC semantics and needs a live 100-well benchmark to sign off; this first item is the
one pure-frontend, low-risk win.

- [ ] **Crossplot: Z coloring memoized across pan/zoom/hover** (crossplotPanel.ts). Every crossplot
      redraw rebuilt the whole per-point color array from scratch — for a continuous Z that's **two
      `percentile` sorts** (each allocates + sorts a NaN-filtered copy of all samples) plus an
      N-length `colorRampEx` string array; for a discrete Z, an N-length `categoricalColors` map.
      That ran on **every** pan-drag / zoom-wheel / handle-drag `mousemove` and every synchronized-
      hover frame, even though the colors depend only on the Z data + colormap, never the viewport.
      The color computation is now a pure `computeCrossplotColors()` that the panel **memoizes**,
      keyed by (Z curve, colormap, log-Z, fixed color, data generation); pan/zoom/hover reuse the
      cached array and only a data or color-setting change recomputes. Output is pixel-identical —
      this is a speed change only, most visible on dense (100-well / full-field) clouds. tsc clean.
      Test: open a crossplot colored by a curve (e.g. NPHI-RHOB by GR, or a PERM Z with log-Z on),
      drag the parameter handle / Ctrl+wheel-zoom / pan / hover from a log view — motion stays
      smooth on a big cloud; the colors, color-bar range, and facies legend are unchanged; switching
      the Z curve, colormap, or log-Z toggle still recolors immediately; a module re-run (dataVersion)
      recolors against the new data.

## Low-tier correctness & data-integrity sweep (2026-07-21)

The 15 low-severity findings from `AUDIT-2026-07-20.md` (never adversarially verified at audit time)
were each re-checked against the current code by an independent per-finding verifier. One was
**already fixed** (SandiMin all-zero conductivity-row guard, `multimin2.rs`), two are **held for your
sign-off** because they change numeric output (Wyllie compaction correction; histogram re-bin), one is
**held to land with the depth-scale-ratio fix** (scale-dropdown staleness). The rest — the safe
correctness/crash/data-integrity fixes below — are applied. **Rust suite green; tsc clean.** Nothing
committed.

Backend (with new regression tests):

- [ ] **SSC `SWIRR_EFF` no longer 0 at a 100 %-shale point** (`ssc.rs`). At the wet-clay point effective
      porosity is floored to 0, and the `1 − φt·(1−SWIRR_T)/φie` divide gave `−inf→0` ("all water
      movable") or `0/0→NaN` — exactly backwards. Now a zero-effective-porosity sample reports
      `SWIRR_EFF = 1.0` (fully bound). _Only the degenerate φie==0 samples change; every producing
      point is unchanged._ (The deeper SWIRR_T/SWIRR_EFF ordering inconsistency is the separate held
      item.) Test: run SSC on a shale-heavy well; SWIRR_EFF in massive shale reads ~1, not 0.
- [ ] **Archie `SWT_ARCH` no longer writes `+Infinity`** (`modules.rs` `sw_arch`). A coal/tight sample
      with PHIT=0 but PHIE absent used to fall through to `a/0^m = +inf` and store it in the SWT_ARCH
      curve, poisoning catalog min/max and plot autoscale. The zero-porosity "all water" guard now
      keys on PHIT alone. Test (regression `sw_arch_zero_porosity_missing_phie_is_all_water_not_inf`):
      the curve catalog's SWT_ARCH min/max stays finite over coal/tight zones.
- [ ] **Simandoux (SCHLUMBERGER) no longer divides by zero at VSH=1** (`modules.rs` `sw_sim`). Pure
      shale hit a `1/(1−VSH)` singularity and the sample was silently dropped; it now resolves to
      all-water (SWE=1), matching the low-porosity and Indonesia branches. Test:
      `sw_sim_schlumberger_pure_shale_is_all_water`.
- [ ] **LAS import fails loudly on a truncated row** (`parsers.rs`, both parsers). A physically short
      `~A` row used to shift every following value one column left silently (GR into RES, RHOB into
      DT) for the rest of the file. Leftover tokens at EOF now raise a clear import error instead of
      mis-columning. Test: import a LAS whose last data line is cut mid-row → you get an explicit
      "leftover token(s)…truncated or corrupt LAS?" error, not corrupted curves.
- [ ] **DB-inspector edit no longer reports success on a 0-row update** (`db.rs`, all three sample
      editors). If the matched depth had moved/been rewritten, the UPDATE hit 0 rows but the UI said
      "saved" and pushed a bogus undo entry. It now errors, and the inspector already reverts the cell + shows "Edit failed". Test (`…is_err()` assertion added): edit a sample, then edit a
      non-existent depth → "no … sample matched depth …", cell reverts, no phantom undo.
- [ ] **Well Header shows current TD / KB** (`db.rs` list_wells + `ipc.ts` + `ribbon.ts`). The dialog
      used to open with blank TD/KB, so you edited the datum blind — and KB silently drives TVDSS in
      deviation import. TD/KB are now carried on `WellSummary` and prefilled. Test: open Well Header on
      a well with a KB set → the field shows it, not an empty box.

Frontend:

- [ ] **Stats / regression reject `±Infinity`, not just NaN** (`plotCanvas.ts`: basicStats, linearFit,
      percentile, drawScatter/drawDiamonds). One inf sample (e.g. a Python `1/phi` at phi=0) used to
      make the histogram's Mean/Std chips read "Infinity" and silently kill a crossplot regression.
      Now non-finite values are skipped everywhere. Test: compute an equation that divides by a
      zero-porosity sample, then histogram/crossplot it — chips and the fit stay sane.
- [ ] **Zone-param "Set" button surfaces write failures** (`plotCommon.ts` pickRow — histogram Pick
      A/B, Pickett M/Rw). A rejected `setZoneParam` used to be swallowed while the status still said
      "set". It now shows "Failed to set …". Test: (hard to force by hand) — behaviour only differs on
      a backend write error; the success path is unchanged.
- [ ] **Duplicate track titles prevented** (`layoutPropsDialog.ts`). Renaming a track to an existing
      track's title collapsed both in every title-keyed lookup (weights, cursor hit-testing, core
      overlay, drag-drop). A colliding rename is now auto-suffixed ("RES 2"). Test: in Layout
      Properties, rename a track to another track's exact name → it becomes "name 2"; retyping a
      track's own name is a no-op.
- [ ] **Histogram: constant curves render; the `n` never silently disagrees** (`histogramPanel.ts`).
      A constant curve (flag/class curve, single-sample zone) used to show "No valid data"; it now
      draws one central bar. And when the P2–P98 axis window clips tail samples, the axis label reads
      `n = X of Y` so it no longer contradicts the stats chips (which count all samples). _(The full
      full-range re-bin — which would change every bar height — is the held item.)_ Test: histogram a
      constant/flag curve (draws), and a curve with fat tails (label shows "of").
- [ ] **Log-view smoothness** (`LogCanvasRenderer.ts`, speed only). The clear color is no longer read
      via `getComputedStyle` every rendered frame (cached, invalidated on theme change), and the
      cursor readout uses a binary search instead of scanning every sample per mouse-move. Values and
      colors are identical. Test: drag-pan a busy log view — motion is smoother; theme switch still
      repaints; the cursor readout still tracks correctly.

### Held-item resolutions (2026-07-21 — your call: 1 yes / 2 leave / 3 yes / 4 yes + Bahasa Jawa)

Your answers to the four held items above. **Rust suite 164 pass / 0 fail; tsc EXIT 0; the two
browser-observable pieces verified live in the vite preview.**

- [ ] **Wyllie lack-of-compaction (Cp) correction — shipped as opt-in** (`modules.rs` `phi_son`,
      `OPT_CP` **default OFF**). ON divides the WYLLIE porosity by `Cp = DT_SH/100`; RHG is
      self-compacting and is never touched. Nothing changes until you switch it on. Test
      (`phi_son_wyllie_cp_opt_in_only_scales_wyllie`): OFF unchanged; ON ≈ +11 % at DT_SH=90; RHG
      unaffected. In the app: run Porosity → Sonic with OPT_CP=ON on a shallow well → PHIT rises a
      few p.u.; OPT_CP=OFF reproduces the old numbers exactly.
- **Histogram full-range re-bin — left as-is** at your request (bars keep clipping the extreme tails;
  bar heights and the mode/P50 you read off them are unchanged).
- [ ] **Depth-scale dropdown now shows the TRUE scale + the mislabel is fixed**
      (`LogCanvasRenderer.ts`, `logViewPanel.ts`). The default was labelled "1:100" but was really
      ~1:3937, and the `[0.02, 20]` px/unit clamp made **1:20, 1:50 and 1:100 all collapse to the same
      zoom**. Now: a true 1:1 = `96/0.0254` px per depth unit is single-sourced; the view opens at an
      honest **1:2000**; the clamp reaches a true 1:10; and after any Ctrl+wheel/± zoom the selector
      re-reads the live ratio (a transient "1:N ⟳" entry when it's between presets). Test: pick 1:50
      then 1:100 → visibly different scales (identical before); Ctrl+wheel zoom → the box tracks the
      real ratio instead of freezing on the last preset.
- [ ] **Quiet Ctrl+S save + Escape closes ribbon menus** (`ribbon.ts`). Ctrl/Cmd+S re-saves the
      current session in place once it has a name (no dialog), falling back to Save Session As the
      first time; it's ignored while typing in an input/CodeMirror so editors keep their own Save.
      Escape closes any open ribbon dropdown without disturbing modal Escape handling. _(A Ctrl+P
      print-active-plot shortcut was deliberately deferred — resolving "the active canvas" from the
      ribbon is fragile; the per-plot Print button still works.)_ Test: name a session, edit the
      workspace, Ctrl+S → "Session … saved" with no dialog and the unsaved dot clears.
- [ ] **Bahasa Jawa (jv) added + fuller Bahasa Indonesia / Basa Sunda** (`i18n.ts`, `index.html`).
      A full Javanese (ngoko) dictionary joins id/su, and ~55 common UI phrases (New/Open/Edit/Search/
      Print/Value/Zone/Session/… + statuses) were added to all three — petrophysics jargon still stays
      English by design. Test: Project → Language → **Basa Jawa** → menus/buttons switch (Save→Simpen,
      Depth→Jero, Reload→"Muat manèh"); switch back to English → everything reverts from source.

## Reference-library correctness fixes (2026-07-20)

Two physics fixes distilled from the ITB team reference shelf (Ellis Ch12/14, Halliburton FE Ch27):

- [ ] **Multimin — PEF now converts to U before mixing.** In the Multimin (SandiMin) dialog,
      select **Photoelectric (PEF)** as an input tool (instead of, or alongside, U) and run on
      a well with a PEF curve + RHOB. Confirm VOL\_\* volumes are sensible and RECON is low in
      clean zones. Physics: per-electron PEF does NOT mix by volume — the solver now converts
      the PEF curve to U = Pe·ρe per sample (ρe from RHOB) and mixes against the U endpoints.
      Picking U directly is unchanged. Needs a RHOB curve present; where RHOB is missing the
      PEF row is simply skipped that sample.
- [ ] **VSH from Density-Neutron — new VSH_DN_FLAG clay-type guard.** Run **VSH from
      Density-Neutron** with the optional **GR** input mapped and set GR_MA/GR_SH/FLAG_TOL.
      Confirm a new `VSH_DN_FLAG` curve = 1 where the N-D VSH is off-model (gas crossover /
      beyond the shale point) or diverges from the GR VSH by more than FLAG_TOL (0.25 v/v
      default) — the signature of clay-type or gas ambiguity. Leaving GR unmapped still flags
      off-model samples. VSH/VSH_DN themselves are unchanged.

## Field Map — well surface coordinates + polygon → group (2026-07-20 #27)

**Field Map** (View/Batch ribbon map button, or the Petrophysics ▸ Field Map… button) — a
standalone dock pane that posts wells by UTM surface location and lets you rubber-band a
polygon to select wells into a well group. Coordinates arrive two ways: **Data ▸ Import Well
Locations…** (a CSV/TXT with EASTING/NORTHING, optional WELL and ZONE columns, plus a
choosable default UTM zone covering Indonesia — zones 46–54, N and S — applied to rows/files
without a ZONE column), or per-well via **Tools ▸ Well Header** (Surface X / Surface Y / UTM
zone fields). Coordinates persist as DOUBLE in new `wells` columns
(surface_x/surface_y/utm_zone) — southern-hemisphere northings ≈ 9.4e6 exceed f32's ~1 m
precision, so f64 is required. The pane draws markers with pan (drag), cursor-anchored
wheel-zoom, a faint coordinate grid, a scale bar, and labels for ≤80 wells; the active well
group is ringed. Draw mode: click to drop polygon vertices, close near the first vertex /
double-click / Enter; vertices are draggable; enclosed wells highlight live (a TS
point-in-polygon mirrors the Rust ray-cast). **Assign to group…** runs the authoritative
backend `wells_in_polygon` (PNPOLY, half-open crossing rule) and unions the result into a new
or existing group. Raw easting/northing is plotted directly — no reprojection; a multi-zone
project is a documented follow-on. The polygon is a transient selection tool — the persistent
artifact is the well-group membership (persisting polygon shapes as documents is a noted
follow-on vs the roadmap's original wording).

Adversarial review (4 lenses — geometry-math / import-parse / integration /
frontend-robustness, each finding skeptically verified): **3 defects confirmed, all fixed; 0
refuted.** (1) [high] Well Header Save wrote surface_x/y/zone unconditionally from a stale
`selectedWell` snapshot that is never re-broadcast on a data change, so re-saving after an
import (or a prior save) NULLed out the just-set coordinates — fixed by re-reading the well
from the DB when the dialog opens, so the fields always reflect current state. (2) [medium] A
blank WELL cell in a multi-well locations file collapsed into the same "no well column" case
as a headerless file and was routed to the selected well, silently overwriting an unrelated
well's location — fixed by returning a `has_well_column` flag from the parser so the importer
only falls back to the selected well for a genuinely column-less file, and skips (and
reports) blank-cell rows; the import loop is now wrapped in a transaction so a mid-file error
rolls back instead of leaving a partial write reported as a total failure. (3) [medium] When
coordinates first arrived while the pane was already open, the data-driven reload never fit
the view, so markers rendered off-screen until a manual Fit — fixed by fitting on the first
appearance of laid-out wells. cargo test 143/143; tsc clean.

- [ ] **Data ▸ Import Well Locations…** — pick your Indonesia UTM zone (e.g. 50S for
      Mahakam), import a CSV with WELL/EASTING/NORTHING → the status line reports N wells
      located; open **Field Map** and confirm the wells post at the right relative geometry.
- [ ] Import a file that has a WELL column but a blank cell in one row → that row is skipped
      and surfaced as "1 blank-WELL row(s)", and no unrelated well's location changes.
- [ ] **Tools ▸ Well Header** on a located well → Surface X/Y/zone show the imported values
      (not blank); change only TD and Save → the coordinates survive (were being wiped before).
- [ ] With Field Map already open on a project that had no coordinates, run Import Well
      Locations → the map fits to the new wells automatically (no manual Fit needed).
- [ ] Field Map ▸ **Draw polygon**, enclose a few wells, **Assign to group…** → the enclosed
      wells land in the chosen/new well group; the group filter elsewhere reflects it.

## φmax porosity ceiling — phimax module (2026-07-20 #26)

**Petrophysics ▸ Porosity ▸ Porosity Ceiling (φmax)** — caps a computed porosity at
the field's compaction-controlled upper limit (the deck slide-64 "max core porosity"
line). A `MODE` dropdown picks the ceiling model: **constant** (a flat `PHIMAX0`,
per-zone overridable — the literal max-line), **linear** (`φmax = PHIMAX0 −
PHIMAX_GRAD·(TVDSS − TVDSS_REF)/1000`), or **athy** (`φmax = PHIMAX0·exp(−ATHY_K·
(TVDSS − TVDSS_REF)/1000)`, the exponential compaction law). TVDSS is a
positive-downward depth-below-datum curve (same convention as **precalc**), so deeper
= larger TVDSS = lower ceiling; with no TVDSS curve it falls back whole-curve to
measured DEPTH (fine for near-vertical wells). All four parameters are zone-overridable,
so each formation (Post-Main / Main / Massive / Talang Akar) can carry its own ceiling
or its own trend coefficients. Writes `<PHI>_CAP = min(PHI, φmax)` (preserving MISSING)
and the ceiling curve `<PHI>_MAX` for a QC overlay; the input porosity is never
modified. The dialog is auto-generated from the manifest, so it appears in the Porosity
dropdown with no bespoke UI. Standalone by design — it caps _any_ porosity output
(phi_den/phi_dn/phi_son or SandiMin's PHIT); a solver-internal φmax box constraint is a
noted follow-on.

Adversarial review (4 lenses — math / integration / edge / contract — each finding
verified): **0 defects confirmed, 4 refuted** (all were test-coverage/doc-completeness
notes over correct, deliberate behaviour). Two of the flagged-untested paths were
locked in with regression guards anyway: the ceiling clamp to [0,1] (a sub-zero trend
ceiling forces porosity to 0; a super-unit one clamps to 1), and the partial-NaN TVDSS
pass-through (a NaN-depth sample gets a MISSING ceiling and passes porosity through
uncapped). cargo test 136/136; tsc clean.

- [ ] **Petrophysics ▸ Porosity ▸ Porosity Ceiling (φmax)** opens (auto-dialog). Run
      **constant** mode, PHIMAX0 = your field max (e.g. 0.35), input **PHIE** → the
      `PHIE_CAP` curve should equal PHIE below 0.35 and flatten at 0.35 above it;
      `PHIE_MAX` is a flat 0.35 line.
- [ ] Crossplot `PHIE_CAP` vs depth (or overlay `PHIE_MAX` on the porosity track) — the
      capped cloud should sit under the ceiling with no points poking above it.
- [ ] Switch to **linear** (or **athy**) with your TVDSS trend: PHIMAX0 at a shallow
      `TVDSS_REF`, a sensible `PHIMAX_GRAD` (or `ATHY_K`). `PHIE_MAX` should fall with
      depth; confirm a deep zone's ceiling is lower than a shallow zone's.
- [ ] Set **per-zone** `PHIMAX0` (or trend coeffs) in Zones/zone params and re-run —
      each formation's ceiling should honour its own value.
- [ ] Well with **no TVDSS curve**: linear/athy should still run (trend reads against MD
      DEPTH) — sanity-check that the ceiling still declines with depth. Deviated wells
      want a real TVDSS curve (survey→TVDSS bridge is a follow-on).
- [ ] Feed `PHIE_CAP` into **Cutoffs & Pay Summary** as the PHIE input — pay should drop
      where the cap trimmed optimistic porosity.

## Cutoff Sensitivity pane (2026-07-20 #25)

**Reporting ▸ Cutoff Sensitivity** — two ways to defend a VSH/PHIE/SWE pay cutoff
against DST-tested rock (KKT ONWJ deck slides 84–87), in one dock pane with a
Sweep / DST-Crossplot toggle. **Sweep** varies one cutoff across a range while the
other two stay fixed and plots the pay response per well — net thickness, HC
pore-thickness (HPV), or net-to-gross — so the _elbow_ shows where loosening the
cutoff stops adding real pay; the shared pay math is the **same `classify_sample`
the pay summary uses**, so the numbers reconcile. **DST Crossplot** is PHIE vs a
shale/Sw curve with every sample dim and DST-interval samples coloured per well,
plus a draggable red crosshair at the candidate cutoffs. Either mode's pick writes
into the VSH/PHIE/SWE fields and can be **saved as the pay-summary default** so the
cutoff you defended flows straight into the report. Optional zone and DST/perf
filters scope both modes.

Adversarial review raised 13, confirmed 10 (from two independent full passes),
all fixed before shipping: the sweep's PERM-cutoff scope now matches the pay
summary exactly (whole-well, not just the analysed window); overlapping
perforation/DST intervals are unioned so N:G isn't understated; switching the
swept property/metric after a run clears the stale plot so a pick can't be written
into the wrong cutoff; the "(all samples)" DST choice survives editing the well
set; the zone/DST pickers union over _all_ checked wells (was capped at 16); the
crosshair stays inside the plotted range; empty-state text is centred on HiDPI; the
plot repaints on theme change; wells with no pay / missing inputs are flagged
rather than shown as a silent flat line. cargo test 131/131; tsc clean.

- [ ] **Reporting ▸ Cutoff Sensitivity** opens as a dock pane. Tick a few wells,
      keep **Sweep**, property **VSH**, metric **Net**, Compute — one line per well;
      the curve should rise and flatten (an elbow), not be a straight ramp.
- [ ] Click/drag on the plot to place the red cutoff line; the readout shows the
      net/HPV/N:G each well delivers _at that cutoff_. Click **Use pick as VSH
      cutoff** → the VSH field updates.
- [ ] Switch the metric to **N:G**, Compute again; switch property to **PHIE** — the
      plot should **clear** and ask you to Compute (it must not keep showing the VSH
      sweep while the button says "PHIE").
- [ ] Cross-check one well against **Cutoffs & Pay Summary** at the same fixed
      VSH/PHIE/SWE (whole well, no zone/DST): the sweep's Net at those cutoffs should
      match the pay summary's Net for that well. Repeat with a **PERM ≥** cutoff set.
- [ ] **DST Crossplot** mode with a well that has a DST/perf set: dim cloud + coloured
      DST points; drag the crosshair; **Apply crosshair → cutoffs** writes PHIE and
      VSH (or SWE if the X curve is an Sw). Pick a "PHIE vs Sw" preset and confirm the
      Apply maps to SWE.
- [ ] Pick a DST set, then switch the DST dropdown to **(all samples)**, then tick/untick
      a well — the dropdown must **stay** on "(all samples)" (not snap back to the DST set).
- [ ] **Save as pay-summary default** → open **Cutoffs & Pay Summary**: its VSH/PHIE/SWE
      inputs should already carry your saved cutoffs.
- [ ] Switch the app theme with the pane open — the plot repaints immediately in the
      new palette (no stale colours).

## All tools as dockview panes (2026-07-20 #24)

Your ask: "i want all tools shows as pane, for existing and future tools." Every
computation/analysis tool now opens as a **dockable pane** instead of a pop-up. The
big one: the **auto-generated module form** (every Petrophysics ▸ Data Prep / VSH /
Porosity / Saturation module) is now a pane — one per module — so you can keep
several docked side by side and re-run each as you iterate, and **any new module I
add in Rust gets its pane automatically** with no extra UI work. **Zones,
Autocorrelate Tops, Composite Log, and Report** are panes too; they follow the
selected well the way the plots do, and refresh their lists when data changes. Quick
pop-ups stayed pop-ups on purpose (curve editor, layout properties, save/open
session, import prompts). Adversarial review found 9 real issues, all fixed before
this shipped (pin-off panes catching up to a selection, no stale-well writes after a
project switch, the autocorrelate "pick a top first" message re-checking itself once
you pick one, etc.). tsc clean; module-pane behavior browser-verified.

- [ ] Open a module (e.g. Gas Correction) from the Petrophysics tab — it should
      appear as a pane you can dock/split/float, not a pop-up. Run it; the result
      lines stay in the pane (no auto-close). Open a second module — both panes
      coexist (the old pop-ups could not).
- [ ] With a module pane open, compute a curve, then open another module: the new
      curve should already be selectable in its input dropdowns (the pane refreshes
      its lists on data changes without losing what you'd already picked).
- [ ] Multi-select several wells in Wells & Tops, THEN open a module: all selected
      wells should be pre-ticked (not just the active one).
- [ ] Open the **Zones** / **Composite** / **Report** pane with no well selected —
      it shows "Select a well… will follow" instead of a "select a well first"
      toast; pick a well and it fills in and the tab title updates.
- [ ] **Autocorrelate Tops** on a well with no tops: the pane says "pick one in the
      log view first" — go pick a top, and the pane should update itself (no need to
      close/reopen). Apply a correlation: the proposals clear.
- [ ] Switch projects with a Zones/Report pane docked in a background tab: it must
      reset to the "select a well" hint, NOT keep showing a well from the old
      project (this prevents editing the new project with a stale well).
- [ ] Docking sanity: the panes save/restore with the workspace layout, appear in
      the ＋ "add panel" menu, and the log-view right-click "Print / export layout…"
      opens the Composite pane.

## Gas Correction module — iterated density de-gassing (2026-07-20 #23)

**Petrophysics ▸ Data Prep ▸ Gas Correction (density, iterated)** — the KKT deck
slide-65 loop. Density porosity and Archie SWT are solved from the current density,
then RHOB_GC = RHOB + Φt·(1−Sw)·(RHO_FL − GASDEN) replaces gas with liquid, iterated
to |ΔΦt| < 1e-4 (non-converging samples stay MISSING). GASDEN is the real-gas density
of an SG_GAS 0.65 gas at FPRESS/FTEMP (Standing pseudo-criticals + Papay z, pinned
0.1297 g/cc at the KK example's 2743 psi / 93.9 °C) — **run precalc first**; FTEMP and
FPRESS accept only precalc/log-set curves, never a raw import (a the reference suite LAS's degF
FTEMP can't sneak in as degC). Default **OPT_GATE = FLAGGED** corrects only where the
gas flag > 0.5 (chain condflag's XOVER_FLAG, which excludes coal and washout) and
errors loudly if the flag curve has no data; **EVERYWHERE** is there for wells without
condflag, but beware coals/resistive washouts — high RT + low density reads as gas to
the Archie loop. The adversarial review raised 13 confirmed findings → all fixed
(FLAGGED default, flag > 0.5 gate, no-flag-data error, degenerate RHO_MA/RHO_FL and
RHOB<RHO_FL and Rw≤0 guards, non-convergence → MISSING, NaN-proof Archie cap,
computed-only P/T inputs, RHOG→GASDEN rename, doc rewrite). 127 cargo tests green.

- [ ] Run precalc → condflag → Gas Correction (defaults) on a KK-style gas well: the
      detached high-porosity gas cloud on PHIE vs wet-clay (slides 66–67) should
      collapse after correction; RHOB_GC ≈ RHOB in water zones (self-limiting there).
- [ ] Check a coal streak stays untouched under the FLAGGED default (XOVER_FLAG
      excludes coal) — no phantom high-porosity pay in coals.
- [ ] Without condflag run: the FLAGGED default must error "gas flag has no data —
      run condflag first or set OPT_GATE = EVERYWHERE", not silently pass through.
- [ ] Without precalc run: outputs stay MISSING (never uncorrected pass-through),
      even if the well's LAS carries its own raw FTEMP/FPRESS curves.
- [ ] Feed RHOB_GC to **phi_den** (or use PHIT_GC directly). Do NOT feed it to phi_dn
      or a SandiMin solve that includes NPHI — their gas handling assumes an
      uncorrected density-neutron pair (the module doc says this too).

## SandiMin: wet→dry clay converter + fluid autofill from precalc (2026-07-20 #22)

Two additions inside the **SandiMin** pane (Advance tab), from your Multimin
Parameters.xlsx workflow (Wave E item 18). **Wet clay → dry clay** panel: enter the
wet-clay picks from a shale interval (RHOB/NPHI/GR, optional DT) and the assumed
dry-clay density (2.70 marine / 2.78 deltaic per the KKT deck slide 60); it computes
φ_clay = (ρdry−ρwet)/(ρdry−1) and the dry endpoints with the xlsx formulas verbatim
(water 1.00 g/cc, 189 µs/ft), previews them live, and **Apply** writes them to the
chosen clay, ticks it + BoundWater, and sets a **CEC_eq** on the clay that makes the
solver's Dual-Water bound-water constraint enforce exactly v_bw = φ/(1−φ)·v_dryclay —
the deck's slide-59 bookkeeping (SWB = VOL_UBNDWAT/PHIT). Unphysical picks error
instead of applying: NPHI must be a fraction (percent entry rejected — the reference suite habit
guard), GR positive, wet DT above the 189·φ water term. **Autofill from precalc**
(fluid box): pick a zone of the selected well and **Read** — fills Formation temp
from FTEMP_F and the Rmf sample from precalc's RMF (retied to formation temp, an
Arps no-op, only when both curves came back; a raw RMF without FTEMP_F is refused
as not-precalc). The zone dropdown follows your well selection live.

- [ ] KK-1 Post Main check: wet 2.18333/0.48958/110 with dry density 2.70 → the
      preview must read φ_clay 0.3039, NPHI 0.2667, GR 158.0 (the xlsx values).
- [ ] Apply to Illite, then run SandiMin with CT on: solved VOL_UBNDWAT/VOL_DRYCLAY
      should sit at ~0.4366 (= φ/(1−φ)) in clay-rich intervals; SWB = VOL_UBNDWAT/PHIT
      comparable to the deck's slide-59 CWB-panel behaviour.
- [ ] Note the pairing rule: CEC_eq is tied to the clay's **RHOB endpoint** and the
      fluid **T/Rw/α** at Apply time — if you edit any of those afterwards, re-Apply
      (the status line and the CEC column tooltip both say so now).
- [ ] Autofill on a precalc'd well: Read (whole well and one zone) fills FTEMP/Rmf
      and the previews update; on a well without precalc it must refuse with "run
      the precalc module first", not fill garbage.
- [ ] Switch wells with the SandiMin pane open: the autofill zone list must follow
      the selection (it re-reads the new well's zones).

## Neutron Matrix Conversion module — NPHI LS/SS/DOL (2026-07-20 #21)

New Prep module **Neutron Matrix Conversion** (`nphimat`) in the Data Prep dropdown
and workflow builder (your request 2026-07-20). Converts a neutron log recorded in
one matrix convention into all three — **NPHI_LS / NPHI_SS / NPHI_DOL** — using the
chartbook porosity-equivalence curves digitized at vector precision: **Por-5** for
the CNL thermal tools (**NPHI** ratio method; **TNPH** env-corrected, FRESH / 250 kppm
SALT variants) and **Por-4** for the epithermal tools (**APLC/FPLC** = APS, **SNP** =
legacy sidewall). Tell it what the log is (TOOL + MATRIX_IN); the input convention
passes through unchanged and the other two are read through the chart (SS/DOL inputs
invert back to the apparent-limestone axis first). The book's printed worked example
(TNPH 18 pu @ 250 kppm → sandstone 24 pu) reproduces to 0.04 pu. Feed the output
matching your RHO_MA (NPHI_SS with 2.65) — that removes the ~0.04 LS-vs-SS offset the
condflag doc warns about, so XOVER_MIN can stay at 0.04. Also in this increment:
APS/legacy neutron mnemonics (APLC/FPLC/SNP/NPOR/HNPO/NEUT/FSTP) now fill the
standard NPHI column at LAS import, an all-NaN standard column now falls back to the
raw store (family alias) instead of silently feeding NaN to modules, and workflow-
builder input dropdowns now offer every module's outputs so `nphimat → phi_dn
(NPHI = NPHI_SS)` is buildable in a fresh project.

- [ ] Run nphimat on a Mahakam well (TOOL matching the delivery, MATRIX_IN per the
      LAS header — usually LS or SS): NPHI_SS ≈ NPHI_LS + 0.03-0.04 in clean sand,
      NPHI_DOL well below both (thermal dolomite bow).
- [ ] Sanity vs the paper chart: pick one depth, read Por-5 by hand, compare all
      three outputs (expect agreement within ~0.5 pu).
- [ ] Feed NPHI_SS + RHO_MA 2.65 into phi_dn / condflag: crossover in a known gas
      sand appears at XOVER_MIN 0.04 without the limestone-unit offset fudge.
- [ ] Workflow builder in a fresh project: chain nphimat → phi_dn with the NPHI
      input overridden to NPHI_SS (now offered in the dropdown before any run).
- [ ] If you have an APS well (APLC): import fills NPHI now — check the curve
      arrives and nphimat TOOL=APLC gives sensible (small) matrix shifts.

## Data Conditioning Flags module — coal / tight / crossover + shoulder (2026-07-20 #20)

New Prep module **Data Conditioning Flags** (`condflag`) in the Data Prep dropdown
and workflow builder (your request 2026-07-20). One run writes five 0/1 flag
curves: **COAL_FLAG** (RHOB < 1.9 & NPHI > 0.35, plus DT > 100 µs/ft where a sonic
exists; samples with BADHOLE = 1 are never called coal — washouts mimic coal),
**TIGHT_FLAG** (density porosity and NPHI both < 0.05; DPHI uses **RHO_MA/RHO_FL —
the same params and zone overrides as the density-porosity modules**),
**XOVER_FLAG** (gas crossover DPHI − NPHI > 0.04; coal and bad hole excluded —
NPHI must be matrix-consistent with RHO_MA, else raise XOVER_MIN to ~0.08 for
limestone-unit neutron), **SHOULDER_FLAG** (the adjustment you asked for: samples
within SHOULDER of a coal/tight bed edge — or a bad-hole interval ≥ MIN_THICK —
carry boundary-averaged readings and get flagged so no shoulder log survives the
mask), and **COND_FLAG** (combined mask: coal | tight | badhole | shoulder, plus
crossover only when OPT_XCOND = YES). Beds thinner than MIN_THICK are dropped as
spikes; a missing sample inside a bed does not split it. MIN_THICK/SHOULDER are
in the depth curve's unit (defaults suit metres — roughly ×3 for feet). Run
badhole first; feed COND_FLAG as the Mask on later runs, but leave the Mask empty
on the condflag run itself. BADHOLE and COND_FLAG are now always offered in every
Mask dropdown, even in a fresh project where they haven't been computed yet.

- [ ] Run badhole → condflag on a Mahakam well with coals: COAL_FLAG picks the
      coal streaks (check against the density track), and no coal call inside
      washouts.
- [ ] TIGHT_FLAG on a calcite-cemented/tight streak; XOVER_FLAG on a known gas
      sand; crossover NOT flagged over coals.
- [ ] SHOULDER_FLAG brackets each coal/tight bed by ~SHOULDER depth units; a
      lone one-sample BADHOLE blip is masked in COND_FLAG but does NOT dilate.
- [ ] MIN_THICK: single-sample spikes dropped; a real bed with one null sample
      in the middle is kept whole.
- [ ] Feed COND_FLAG as Mask on a porosity run: flagged + shoulder samples go
      missing in the outputs; confirm COND_FLAG appears in the Mask dropdown of
      a fresh workflow before condflag has ever run.
- [ ] Zone overrides: RHO_MA 2.71 in a carbonate zone shifts TIGHT/XOVER there
      (same override the density-porosity modules use).

## Wave E-17: pre-calculation module — P / T / Rmf / Ct / Cxo (2026-07-20 #19)

New Prep module **Pre-Calculation (P / T / Rmf / Ct / Cxo)** in the Data Prep
dropdown and the workflow builder (ROADMAP §4c item 17, from your KKT ONWJ
workflow). One run writes six curves: FTEMP (**always degC** — the unit every
downstream module assumes) plus FTEMP_F (the degF twin, for SandiMin fluid
entry) and FPRESS as linear trends in TVDSS (gradients per depth unit of the
TVDSS curve — per-metre values for metric wells; no TVDSS curve → measured
depth is used), RMF at formation temperature (ARPS from a surface Rmf
measurement, or TREND regression `RMF_A + RMF_B·log10(TVDSS)` for wells
without mud data — the shipped defaults are the ONWJ **feet-based** fit), and
CT = 1000/RT, CXO = 1000/RXO in mmho/m as QC/plotting conductivities (note:
SandiMin's CT/CXO tool rows read the resistivity curves directly — don't feed
these to them). Params are SURF_TEMP/TEMP_GRAD (own names, so zone overrides
never cross-apply with Formation Temperature's degC-only TSURF/TGRAD); entry
unit degF/degC via OPT_TU.

- [ ] Run it on a KKT-style well with your fits (SURF_TEMP 77 / TEMP_GRAD
      0.0260292, PSURF 44.2823 / PGRAD 0.539812, degF): FTEMP_F/FPRESS match
      the deck's trend lines; spot-check one depth by hand; FTEMP = same in degC.
- [ ] Deep resistivity input defaults to the RES*DEEP family (same as the sw*\*
      modules) so CT fills for wells whose deep curve is ILD/LLD/AT90 etc. —
      confirm CT is not blank on a standard import.
- [ ] ARPS mode: RMF at depth ≈ your surface Rmf pulled down by (T₁+6.77)/(T₂+6.77);
      TREND mode with A 0.517068 / B −0.116517 reproduces the field regression.
- [ ] degC mode on a metric well (e.g. SURF_TEMP 25, TEMP_GRAD 0.03 degC/m):
      FTEMP in degC, FTEMP_F in degF, RMF still Arps-correct.
- [ ] CT/CXO: 1000/RT and 1000/RXO, missing where RT/RXO are missing or ≤ 0.
- [ ] Zone overrides: give one zone a different TEMP_GRAD in the Zones dialog —
      the FTEMP trend kinks at the zone boundary (per-zone params resolve per
      sample).

## Wave A-4: workflow grid inspector (2026-07-20 #18)

The Workflow Builder pane has a **List | Grid** toggle above the step list
(ROADMAP §4c item 12). Grid = the multi-line inspector: rows are your chain's
steps, columns are the union of every step's inputs/params/options (+ Mask), so a
parameter shared by several modules lines up in one column. The italic **Set all**
row under the header edits a parameter across every step that takes it in one go.

- [ ] Build your standard chain (vsh → phi → sw\_\* …), switch to **Grid**: input
      curves come first, then numeric params, then options, then Mask; steps that
      don't take a column show "—". Header tooltips = parameter descriptions.
- [ ] **Set all → RW**: type one RW in the Set-all row — every sw\_\* step that takes
      RW updates at once (status bar reports how many). A value outside one
      module's allowed range is skipped for that module only and reported.
- [ ] Edited cells tint amber and the step's override badge counts up — same
      only-store-differences rule as the per-step editors, so a value typed equal
      to a module's default clears that override (cell untints). Zone params still
      override these whole-well values per zone at run time, as before.
- [ ] **Set all → Mask** sets opts.MASK (e.g. BADHOLE) on every step in one edit.
- [ ] Toggle List ↔ Grid: values, badges and invalid-input flagging stay in sync
      (both views edit the same steps). The chosen view is remembered.
- [ ] Save the workflow, reload it, re-run — saved JSON is unchanged in shape, so
      old saved workflows load into the grid fine.

## Wave A-3: project open/switch, IP style (2026-07-20 #17)

You can now keep separate project databases (balam.duckdb, minas.duckdb, …) and
switch between them inside the app (ROADMAP §4c item 2). Project ribbon tab, new
group left of Appearance:

- [ ] **New Project…** creates a fresh, empty .duckdb and switches to it — import a
      couple of Balam South LAS files there, confirm they do NOT appear in your main
      project, then switch back.
- [ ] **Open Project…** switches to an existing file; **Recent ▾** lists the last 12
      projects (current one marked ●, deleted files greyed "(missing)"), stored in
      `%APPDATA%\SandiBumi\projects.json` — outside any project.
- [ ] On switch: window title + group caption show the project name, well list /
      plots / catalogs all reload, well selection and undo history clear (old-project
      undo entries would corrupt the new one — deliberate).
- [ ] **Next launch reopens the last project you had open** (falls back to the old
      `project.duckdb` if the recents list is empty — first launch after this update
      behaves exactly as before).
- [ ] Switching is refused while a workflow chain is running (try it: start a long
      chain, then Open Project — you should get a clear error, not a corrupted run).
- [ ] Note: QAT **Save Project As** stays a backup copy (app keeps working on the
      current file) — tell me if you'd rather it switch to the copy, IP-style.

## Wave A-2: compact import ribbon (2026-07-20 #16)

The Data tab's eleven flat import buttons are now three Office-style dropdowns
(ROADMAP §4c item 4) — same handlers, just organized:

- [ ] **Import Logs ▾** (LAS, DLIS), **Import Data ▾** (Core, SCAL, Tops, Aux,
      Deviation), **Export LAS** (unchanged flat button), **Tools ▾**
      (Autocorrelate Tops, Shift Core, Well Header). Run one import of each kind —
      behaviour must be identical to the old buttons; tooltips moved onto the
      menu entries.
- [ ] Only one menu opens at a time; picking an item or clicking elsewhere closes it.
- [ ] Bahasa Indonesia / Basa Sunda: the new labels translate (Impor Log / Impor
      Data / Alat) including the previously untranslated Import Tops / Import Aux /
      Autocorrelate entries.

## Wave A-1: tool panes + theme compliance (2026-07-20 #15)

Four tools moved from popup dialogs to dock panes (ROADMAP §4c item 14) — they now
dock/float/tab like the Workflow Builder and can't be dismissed by a stray click:

- [ ] **Cutoffs & Pay Summary**, **ML Models**, **Monte Carlo**, **SandiMin** ribbon
      buttons each open a PANE (singleton: clicking again focuses the existing one).
      Run each on Balam South data — results should be identical to the old popups.
- [ ] The ＋ add-panel menu on any window now lists all four (under Workflow Builder);
      the right-click menu inside each pane shows its own heading.
- [ ] SandiMin's endpoints matrix now uses the full pane width (was capped at 620px).
- [ ] Panes reopen after an app restart (from the autosaved workspace) in their
      docked position — internal selections (cutoff values etc.) reset, same as the
      Workflow Builder.
- [ ] **Theme check** (switch to Dark, then Pertamina): the log-view cursor readout
      pill now inverts with the theme (was unreadable in dark); crossplot/Pickett/
      histogram pick swatches + histogram pick markers follow the theme accents
      (Pertamina = blue/lime, was always brown/green); core-plug diamond outlines
      visible in dark; workflow invalid-input red and error text use the theme warn
      color; the composite preview surface is no longer light grey in dark themes.

## Chartbook overlay library + audit quick fixes (2026-07-20 #14)

The single D-N overlay grew into a **chart overlay library** (Properties → Overlays →
Chart overlay): every crossplot-family chart from your 2013 chartbook, digitized from
the PDF vector artwork with the same validation stack (graduation sequences, 5-multiple
long dashes, worked examples). Charts matching the current axes are listed first; a
chart draws only when the plot axes actually match it (either orientation).

- [x] **CNL Por-11/12** (as before, now via the new select — old saved props migrate).
- [x] **EcoScope Por-18 (BPHI) / Por-19 (TNPH)** on an LWD well — these are the ones
      that matter for your Mahakam development wells; check a known sand against the
      sandstone line for both BPHI and TNPH inputs.
- [x] **adnVISION675 Por-16** if you have ADN wells.
- [x] **APS Por-13/14** (APLC and FPLC variants listed separately).
- [x] **PEF: Lith-3/4** on a PEF-RHOB crossplot — quartz ~1.65-1.8, calcite ~5.08,
      dolomite ~3.1 curves with 10-pu labels.
- [x] **Sonic-neutron Por-20** (both time-average AND field-observation families) on
      a DT-NPHI crossplot — TA curves reproduce Wyllie with tf 190 to R² 0.99999.
- [x] **Density-sonic Por-22** (TA + FO) on a DT-RHOB crossplot, with the 7 mineral
      points (Sylvite, Salt, Trona, Gypsum, Sulfur, Polyhalite, Anhydrite).
- [x] **Th-K clay chart Lith-2** on a POTA-THOR crossplot — the Th/K ratio fan is
      drawn at the _labeled_ ratios (the chartbook's own printed lines sag a few %
      off their labels; ours are exact), plus the dashed clay/feldspar lines and
      mineral-field labels. Judge your Mahakam illite/kaolinite mix against it.
- [x] **Pe-K and Pe-Th/K clay boxes Lith-1** (the Th/K variant needs the X axis in
      log mode — turn on X log in Properties).
- [x] **Umaa-Rhomaa MID Lith-6** — the ternary triangle with 20/40/60/80 subdivisions + K-feldspar/Barite/Anhydrite/Kaolinite/Illite/Salt points. Needs computed
      UMAA/RHOMAA curves (equation engine for now; a dedicated module is a good next
      increment if you want it).

**Audit quick fixes** (from the full senior audit — see AUDIT-2026-07-20.md and
ROADMAP §4b for the 35-finding backlog):

- [x] **Pay summary change**: with a PERM cutoff active, samples with **missing PERM
      now FAIL the cutoff** (they silently passed before). Re-run a pay summary on a
      well with patchy PERM — net pay may legitimately decrease. Tell me if you'd
      rather missing-PERM samples pass (the reference suite's default behavior differs by setup).
- [ ] **LAS import**: the file's own ~W NULL declaration is now honored (deliveries
      using -99999 etc. no longer import sentinels as data), and **multi-word well
      names survive** ("BALAM SOUTH-01" no longer truncates to "SOUTH-01"). Re-import
      one such file and check the Wells pane name.
- [ ] **Depth scale presets are now TRUE ratios** (1:200 = 1 m of well per 5 mm of
      screen at standard DPI). They were ~39x too compressed before, so 1:200 will
      look much more stretched than you're used to — the numbers are honest now.
- [ ] **Tops editor**: adding a top with an existing name is an overwrite; Ctrl+Z now
      restores the previous depth instead of deleting the top.
- [x] Case-insensitive computed-curve lookup (lowercase equation outputs now resolve).

## P2-f+ — D-N chartbook overlay (2026-07-20 #13)

Digitized from the Schlumberger 2013 chartbook you sent (Por-11 fresh / Por-12 salt,
extracted from the PDF's vector artwork — graduation-dash positions, not eyeballed;
calcite identity check rms 0.13 pu, both charts' worked examples reproduce).

- [x] **Crossplot Properties → Overlays → D-N chart**: pick _Fresh mud (Por-11)_ on an
      NPHI-RHOB crossplot → quartz/calcite/dolomite curves appear with porosity
      graduation dots + labels every 5 pu, dashed iso-porosity connectors, and curve
      names written along the lines. Compare against your paper chartbook page 225.
- [ ] **A real Mahakam sand interval** should plot on/left of the quartz sandstone line
      (shale pulls points right/down toward higher NPHI). Crossplot porosity read off
      the graduations should match your PHIE within ~1-2 pu in clean sand.
- [x] **Salt variant** (Por-12) shifts the curves left at high porosity — only relevant
      if you ever work salt-mud wells; check it renders and the graduations differ from
      Fresh.
- [x] **Zoom/pan**: the overlay must stay registered to the data under Ctrl+wheel zoom
      (it's drawn in data space). Also check the flipped orientation (X=RHOB, Y=NPHI).
- [ ] **Gating**: on a GR-RHOB plot or with a log axis the overlay silently stays off
      (chart geometry only means something on linear NPHI-RHOB).
- [x] **Note**: the chartbook draws its dolomite curve for ρma **2.85** (validated
      against the chart's own graduation ticks), while the _Matrix points_ overlay keeps
      the textbook single point at 2.87 — so Dol point and Dol curve start won't
      coincide exactly. Tell me if you'd rather I move the matrix point to 2.85.

## Fix batch from your o/x review (2026-07-19 #2)

Your full review is triaged in **ROADMAP.md §4** — these five landed immediately:

- [x] **Ctrl+wheel = zoom** on Histogram / Crossplot / Pickett. Plain wheel now scrolls the
      page/pane like you asked; hold **Ctrl** to zoom toward the cursor. Drag-pan and
      double-click-reset unchanged.
- [x] **Pertamina theme** rebuilt from your swatch card: blue #006BB8 (accent), green
      #A6C210 (secondary), red #ED1A2F (warnings/alerts), text #161B22 on white. If you'd
      rather have **red** as the main accent (it's the dominant brand color), say so —
      one-line swap.
- [x] **Theme dropdown**: "Light" is now called **Default** (also translated: Bawaan / Baku).
- [x] **Advance tab regrouped**: a single **Advance Methods** group holds SSC, SSPW, RtC,
      IMTS and **Thin Beds** (moved out of Petrophysics — its old dropdown is gone). The
      wrong "Sand-Silt-Clay" caption over SSPW is gone.
- [x] **Multimin → SandiMin**: the generalized solver button/dialog is now **SandiMin —
      Mineral Solver** (original name, no plagiarism concern). The legacy fixed 4-component
      "Multimin — Mineral Inversion" is **removed from the Saturation dropdown** (mineral
      solving is independent of Sw); it still runs inside saved workflow chains. Tell me if
      you want the legacy one back as its own button.
- [x] **Blurry text fix** (your answer: blurry; your display is at 100% scale, so it's not
      Windows scaling): the desktop app now launches WebView2 with `--enable-lcd-text`,
      which forces ClearType subpixel antialiasing on GPU-composited panels (dockview
      layers otherwise fall back to fuzzy grayscale smoothing). **Needs the `npm run tauri
dev` restart** (config change). Look closely at ribbon/dialog text afterward — if it
      still reads soft, next steps are a base-size bump 12→13px and/or semibold.
- [ ] **T-S triangle now appears** (your "not showing (?)"): the triangle is drawn on
      VSH (0–1) vs PHIT axes — before, ticking it on the default NPHI-RHOB crossplot put
      every line off-scale, so nothing visibly happened. Now ticking **T-S triangle**
      auto-switches the X/Y axes to the well's VSH/PHIT curves (status bar tells you), and
      if the well has no VSH/porosity curves yet it says to run those modules first.
      Check: tick it on a fresh crossplot → axes flip, triangle + drag handles visible.

## P1-a — Interaction safety batch (2026-07-19 #3)

- [x] **Right-click lockdown**: right-click anywhere that has no SandiBumi menu (ribbon,
      buttons, tables, empty space) → **nothing** appears (the WebView menu with its
      dangerous Refresh is gone). Panel backgrounds still show our own menus; right-click
      inside a text box still shows the normal cut/copy/paste menu.
- [ ] **Reload guard**: press **F5** or **Ctrl+R** → a blocking confirm appears instead of
      an instant refresh; Cancel keeps everything, Reload restarts the workspace. Alt+←/→
      and the mouse back/forward side-buttons do nothing.
- [x] **Double-click-to-edit numbers** (app-wide): single-click any numeric parameter
      field (module dialogs, plot properties, SandiMin, zones…) → it focuses with a dashed
      outline but typing/arrows/wheel change **nothing**; **double-click** → solid outline,
      value selected, editing works. Tab-into-field still edits directly (deliberate).
      Scrolling a dialog with the wheel can no longer spin a value.
- [x] **Workflow Builder is a pane**: Petrophysics → Workflow… now opens a docked
      **Workflow Builder** pane (tab, movable/floatable like any panel) instead of a popup.
      No more losing a half-built chain to a stray click; it survives layout changes and
      reopens via the ＋ panel menu too. Run/cancel/progress unchanged; closing the pane
      mid-run cancels the chain.

## P1-b — Crash safe-mode, autosave, unsaved markers (2026-07-19 #3)

- [x] **Autosave**: the workspace (panes, arrangement, active well, every log view's
      layout) autosaves every 10 seconds. Nothing to click — just know it's there.
- [x] **Crash recovery**: if the app dies abnormally (crash, force-kill, power loss),
      the next launch shows a choice **before** anything loads: _Restore autosaved
      workspace_ (everything back as it was moments before the exit) or _Start in Safe
      Mode_ (clean default layout; the autosaved workspace is stashed as a "Recovered …"
      session under Open Session, so nothing is lost). To test without crashing for real:
      end the task from Task Manager while the app is open, then relaunch.
- [x] **Normal restart is less lossy now**: on a clean exit + relaunch, the app also
      brings back the **active well** and each log view's **layout/track state** (before,
      only the pane arrangement survived).
- [ ] **Unsaved markers**: edit a log view (track widths, properties, curve visibility)
      → its tab shows **●** and the QAT Save-Session button gets a red dot. **Save
      Layout** clears that panel's ●; **Save Session** clears everything. The dot means
      "not in a named save yet" — the crash autosave protects you regardless.

## P1-c — Log sets: versioning, provenance, catalog search (2026-07-19 #3)

- [ ] **Never overwrite**: every module dialog now has an **Output set** field (default
      INTERP; type any name — FINAL, TEST, …). Run a module, then re-run it with different
      parameters: the Curve Catalog's "Log sets" section shows **v1 AND v2** — the old
      run's values are kept, not destroyed. Plots/log views show the latest (v2).
- [ ] **Restore a version**: in Inspector → Curve Catalog, click **Restore** on v1 → all
      open log views and plots flip back to the v1 curves. Restore v2 to return.
- [ ] **Per-curve provenance**: the catalog now lists every computed curve's **set + version,
      module, and timestamp** (hover a set row for the exact parameters and input curves
      it was run with). Answering "where did this VSH come from?" is now one glance.
- [ ] **Catalog search/filter/sort**: one search box matches mnemonic, set, module, unit,
      or date; click any column header (Mnemonic, Set, When, n, Min, Max, Mean…) to sort,
      click again to reverse. Statistics (n/min/max/mean) shown per computed curve.
- [ ] **One version per chain run**: the Workflow Builder also has an Output set field —
      a whole chain run (VSH → porosity → Sw) lands as ONE version, not one per step.
- [ ] **Prune old versions**: Delete on a set version (two clicks — it asks "Confirm
      delete") removes only that version's history; current curves are never touched.
      Equation runs land in set EQUATION, ML in ML, SandiMin in SANDIMIN, automatically.
- [ ] **Input set** (the other half of set in/out): run VSH into Output set **FINAL**,
      then re-run with different parameters into **INTERP** (current values are now
      INTERP's). Open a module that consumes VSH (e.g. sw_indo), set **Input set =
      FINAL** → the run uses FINAL's VSH, not the current one. Blank Input set = normal
      behavior (latest values). Works in the Workflow Builder too; curves the input set
      never wrote (GR, RHOB…) still come from the usual sources.

## P2-a — Tops-style imports (2026-07-19 #4)

- [ ] **Import Tops…** (Data tab): pick a CSV or TXT tops file. With a WELL column
      (WELL/WELLNAME/UWI…) every matching project well gets its tops in one import —
      names match case-insensitively, unmatched names are reported in the status bar.
      Without a WELL column the tops land in the selected well. Columns understood:
      TOP/MARKER/SURFACE/FORMATION/HORIZON + DEPTH/MD/TOP_MD; also bare headerless
      "NAME DEPTH" text lines. Delimiters auto-detected (comma / semicolon / tab /
      spaces). Re-import updates depths but keeps colors you've set.
- [ ] **Import Aux…** (Data tab): petrography, XRD, or perforation data for the
      selected well (or a custom-named dataset). Needs a TOP/DEPTH column; a
      BASE/TO column makes rows intervals (perforations); every other column becomes
      an item — numbers (mineral %, grain size) and text (status, remarks) both kept.
      Re-importing a dataset replaces only that dataset for that well.
- [ ] **View it**: Data → DB Inspector → table "Aux Data" shows the imported rows
      per well (read-only — re-import the file to change values). Tops appear
      immediately in the Wells & Tops pane and all log views/correlation.

## P2-f — Crossplot v2 (2026-07-20 #12)

- [x] **Properties dialog**: double-click or right-click the crossplot (or ⚙ Properties)
      → sectioned dialog (Plot / Axes / Z color / Regression / Overlays). The old
      always-visible properties row is gone; the toolbar is just X/Y/Color/Zone.
- [x] **Marginal histograms + percentiles**: enable marginals on NPHI-RHOB — X histogram
      on top, Y histogram on the right, aligned with the axes (RHOB's inverted axis
      included). Percentiles `25, 75` draw dashed reference lines on both axes.
- [x] **Regression options**: on a PHIE-vs-PERM cloud try Power + RMA — the fit line
      must be straight on log axes and curved on linear ones, equation + R² + method
      tag shown top-left. Compare Y-on-X vs RMA slope on a noisy cloud (RMA steeper).
- [x] **Log-safe Z coloring**: color by PERM with "Log Z scale" + Viridis — low and high
      decades must stay distinguishable (rainbow + linear crams everything in one hue);
      the color bar is labeled "(log)".
- [x] **Plot size**: set Fixed 500×400 — the plot stops stretching with the pane
      (consistent exported figures). "Fill panel" restores the old behavior.
- [x] **Universal defaults**: Qtz/Cal/Dol matrix points no longer appear on NPHI-RHOB
      unless ticked in Properties; Color has a "— None —" option (custom point color
      applies); the pick rows + drag handle can be hidden ("Show parameter pickers" —
      still ON by default so your drag-to-set-shale-point workflow is unchanged).

## P2-e — Histogram v2 (2026-07-20 #11)

- [x] **Properties dialog**: double-click or right-click the histogram plot (or the ⚙
      Properties button) → one dialog holds display mode (bars/line), bins, normalize,
      cumulative overlay, box plot, color, percentiles, statistics placement, and the
      parameter-picker toggle. When zoomed, the first double-click resets the zoom, the
      next one opens properties.
- [x] **Box plot + cumulative overlay together**: enable both on a GR histogram — the
      P25–P75 box with P50 line and P5/P95 whiskers sits under the marker labels, and
      the cumulative % curve (secondary color, % labels on the right edge) tracks the
      bars. Zoom in with Ctrl+wheel: box and whiskers follow the axis.
- [x] **User percentiles**: type `10, 90` in Properties → P10/P90 marker lines on the
      plot and removable chips above it (click a chip to drop that percentile). Values
      must match what you'd read off the cumulative curve.
- [x] **Statistics inside the plot**: set Statistics → "Inside the plot" (chips hide) or
      "Both" — the in-plot block shows the active stats incl. new Min/Max. Check it in a
      dark theme too (block background must follow the theme).
- [x] **Universal by default**: a fresh histogram opens with NO Pick A/B rows and clicking
      the plot does nothing — enable "Show parameter pickers" in Properties to get the
      GR_MA/GR_SH picking workflow back. Your saved bar color / percentiles / etc. must
      survive closing and reopening the panel.

## P2-d — Log-view layout interaction (2026-07-19 #10)

- [x] **Collapsible track headers**: ▤ in the log-view toolbar cycles full → compact
      (curve names as inline chips, no scale lines) → titles only. Headers also cap at
      ~a third of the pane and scroll inside, so a 15-curve track can't eat the screen.
      Try it on your densest layout.
- [x] **Move/copy curves between tracks**: drag a curve name from one track header onto
      another track's header — the curve MOVES there (its color/scale/fill travel with
      it). Hold **Ctrl** while dropping to COPY instead (e.g. overlay NPHI on the GR
      track). Ctrl+Z undoes either.
- [x] **Track borders**: ▦ in the toolbar — solid / dashed / none, width 1–4 px, theme
      color (follows light/dark) or a custom color. Default is a thin solid separator
      at every track boundary; check it looks right in dark themes too.
- [x] **Readout follows ONE track now**: hovering shows only the curves of the track
      under the cursor (not all 15). CLICK a track to lock the readout to it (header
      highlights, click again to release) — then you can run the cursor over the whole
      layout while reading just that track's values.
- [x] **Right-click log editing**: right-click on a track → "Edit CURVE…" for each of
      its curves. Ops: **Wireline shift** (whole-curve depth shift, resampled onto its
      own grid — NaN where it slides past the logged interval), **Set constant**,
      **Blank (erase)**, **Interpolate across** (bridge a bad interval linearly),
      **Scale a·v + b** (recalibration). Works on raw (GR/RHOB…), computed, and
      imported generic-store curves alike; every apply is ONE Ctrl+Z entry that
      restores the previous samples bit-exactly, and lands in the History panel.
      Suggested check: blank a washout interval on RHOB, interpolate across it,
      then Ctrl+Z twice — the original curve must come back exactly.

## P2-c — Well pin rework + multi-select (2026-07-19 #9)

- [x] **Pin is now a mode, not a lock.** 📌 ON (default): clicking a well in Wells &
      Tops moves EVERY log view and plot to it — the old behavior. 📌 OFF: each view
      keeps the well it's showing; only the panel you're working in (the active tab)
      follows your clicks. Open two log views, turn the pin off, activate the second
      view, click different wells — only the second view changes. That's the
      side-by-side compare workflow.
- [x] **The old lock is gone**: no more "Active well is locked" blocking when you
      click other wells, and no more weird interaction with a second wells pane.
- [x] **Multi-select**: Ctrl-click wells to build a selection (highlighted with an
      accent edge, count shown in the Wells label), Shift-click for a range,
      ⇄ inverts within the visible list, plain click clears it. Then open any batch
      dialog (module run, Workflow Builder, Multimin, ML, Monte Carlo, Cutoffs &
      Summary) — the multi-selected wells come pre-ticked instead of just the active
      well.

## P2-b — Petrel-style tops editor + autocorrelation (2026-07-19 #4/#13)

- [ ] **Tops lines in the log view**: every log view now draws the well's tops as
      colored labeled lines across all tracks (like the correlation view). They track
      pan/zoom exactly and repaint on theme change.
- [ ] **🏷 edit mode** (log view toolbar): toggle it on, then — **click** an empty
      depth to add a top (name/depth/color dialog, name auto-uppercased); **drag** a
      line to move it (dashed preview while dragging); **double-click** a line to
      rename, change color, or delete. Mouse-wheel zoom still works while editing.
      Everything is undoable (Ctrl+Z) and instantly visible in Wells & Tops, other
      log views, and correlation.
- [ ] **Crossing warnings**: after any pick/move, SandiBumi compares this well's top
      order with every other well. If a pair is reversed vs the majority (e.g. TOP_B
      above TOP_A here but below it elsewhere), a ⚠ warning appears in the status bar
      naming the pair and the vote (e.g. "below it in 4 of 5 other wells").
- [ ] **Autocorrelate…** (Data tab): pick a top in the selected (source) well, choose
      the log (GR default), pattern window ±m and search range ±m — SandiBumi slides
      the source log shape over each target well (active group) and proposes the
      best-match depth with its correlation coefficient r. Strong matches (r ≥ 0.7)
      come pre-ticked; weak ones are dimmed for your judgment. **Apply** writes the
      ticked picks as ONE undoable batch. Try it on a marker you know — e.g. pick an
      MFS on GR in one Balam well and propagate to the rest, then check r values
      against your hand picks.

Issues you marked `[x]` that need real work (all in ROADMAP §4, P1/P2): well-pin
semantics rework, right-click lockdown (accidental refresh), TVD depth scale UI.
Everything you marked `[o]` has been cleared out of this file.

## Theme switch repaints everything immediately (2026-07-19)

- [x] Open a log view + histogram + crossplot, switch Dark ↔ Default ↔ a client theme —
      every pane recolors instantly, no mouse-over needed
- [x] Switch theme while a second tabbed panel is inactive, then activate it — correct colors

## SandiMin — the reference suite-parity mineral solver (2026-07-19, v2)

Rebuilt to the reference suite Multimin / IP Mineral Solver conventions (spec extracted from your
the reference install helpset + IP2018 install → `docs/multimin_ref_spec.md`, `docs/multimin_ip_spec.md`).

- [ ] **Advance → SandiMin…** now shows the full IP mineral list, grouped: 12 minerals (Calcite,
      Quartz, Dolomite, Orthoclase, Albite, Anhydrite, Halite, Gypsum, Pyrite, Siderite, Muscovite,
      Biotite), 6 clays (Glauconite, Kaolinite, Chlorite, Illite, Montmorillonite, Clay — each with
      an editable **CEC**), and 7 zone-typed fluids (Water Sxo / Water Sw / BoundWater / Oil Sxo /
      Oil Sw / Gas Sxo / Gas Sw; "flushed"/"unflushed" badges). Defaults: Quartz, Illite,
      Water Sxo, Water Sw.
- [ ] **Input logs**: 16 tools — Density, Neutron, Sonic, Total GR on by default; PEF, U, spectral
      Th/K/U, Vp, Vs, EPT, EATT, Sigma optional; **CT (Unflushed Conductivity, from RES_DEEP)** on
      by default and **CXO (from RXO)** optional — CT/CXO take a RESISTIVITY mnemonic; the backend
      converts to conductivity (dual-water linear: Ct^(1/w) row, w = 0.75m + 0.25n). Their σ is
      auto (0.03·C^(1/w)) unless you type one. **+ Add user-defined input** adds a custom log with
      its own endpoint column (default σ 0.015, the reference suite's user-defined default).
- [ ] **Endpoints matrix**: editable per component×tool; unflushed-zone fluid cells show "—" for
      nuclear tools (only CT sees them — the reference suite convention); CT/CXO cells show "auto"; per-row
      **Max** bound (fluids default 0.5, the reference suite's cap).
- [ ] **Fluid properties** panel (visible when CT/CXO on): Rw@temp, Rmf@temp, formation temp, m, n,
      mud type. The preview line shows the computed w, Cw, Cmf, Cbw, α(x/u) and auto CT/CXO σ —
      sanity-check Cw against your Pickett Rw (Cw = 1/Rw@FT, mho/m).
- [ ] **Run** on a Balam well with RHOB+NPHI+DT+GR+RES*DEEP: writes VOL*\* per component +
      MM_PHIE, MM_PHIT, MM_SWE, MM_SWT (+ MM_SXOT, MM_MOVEDHC when both zones present),
      MM_VSH (clays + bound water), MM_RECON. Check: **Σ(minerals + unflushed fluids) ≈ 1**,
      **MM_SWT is sensible vs your sw_indo/RtC runs** (this is the new resistivity coupling —
      "resistivity convert to ct and cxo" as requested), and MM_RECON spikes where the model fails.
- [ ] Add **BoundWater** with Illite selected: VOL_BOUNDWATER should track ≈ 0.18×VOL_ILLITE at
      ~150°F (the the reference suite dual-water bound-water constraint, k = 96·CEC·ρ/(T°C+298)).
- [ ] Add **Oil Sxo + Oil Sw** with CXO available: SXOT ≥ SWT in water-based mud (WATER MUD
      constraint) and MM_MOVEDHC = unflushed HC − flushed HC ≥ 0 across invaded pay.
- [ ] Requested upgrade (ROADMAP §4 P3): optional **nonlinear Sw equation iterated to
      convergence** inside the solve loop.

## ML suite (2026-07-19)

- [x] **Petrophysics → ML Models…** opens the Machine Learning dialog (non-blocking, like all
      dialogs now). Four tasks: regression, classification, clustering, reduction — algorithm
      list, hyperparameters, and default output name switch with the task.
- [x] **Field-wide electrofacies**: task = clustering, K-Means or GMM, check GR first in the
      input curves, check ALL wells under Apply — one model over the pooled samples, so class
      ids are consistent across wells (class 0 = cleanest by GR). Set the output (FACIES_ML)
      to "Facies blocks" in a layout and compare wells side by side (📌 pin one panel).
- [x] **Supervised prediction**: task = regression, target = a curve you trust (e.g. CPERM-
      calibrated PERM or RHOB in a well where it's good), train on wells that have it, apply
      to a well missing it. Check r2_cv5 in the metrics table before trusting the output.
- [x] **Classification with core/interpreted labels**: target = FACIES (or an imported
      lithology curve), train on interpreted wells, apply elsewhere — writes ML_CLASS +
      ML_CLASS_PROB; PROB should dip where the log character is ambiguous.
- [x] **PCA/t-SNE**: reduction task writes PC1..PCn (metrics show explained variance %) or
      TSNE1/TSNE2 — crossplot TSNE1 vs TSNE2 colored by FACIES to sanity-check cluster
      separation. t-SNE refuses >20000 samples by design.
- [x] **DBSCAN noise**: noisy/rare samples get NaN (empty in a blocks track), noise_pct in
      metrics. If everything is noise, raise eps.
- [x] Machine needs Python with numpy + scikit-learn (already present — the test suite used
      it); xgboost optional (falls back to sklearn boosting with a note in metrics).

## GMM soft electrofacies (2026-07-19)

- [ ] **Run "Electrofacies (GMM, soft)"** (Petrophysics → Facies dropdown) on a well where you
      already ran the k-means Electrofacies: FACIES_GMM should broadly agree with FACIES in
      clean intervals. Add FPROB to a track (0–1): it should dip at facies boundaries and in
      mixed/transitional beds — that dip is the point of the module.
- [ ] **Crossplot QC**: color a crossplot by FACIES_GMM (categorical palette + F0/F1/… legend,
      same as FACIES); optionally set FACIES_GMM to "Facies blocks" fill in a layout.

## Click-through fix batch (2026-07-19) — remaining item

- [x] **Monte Carlo / Batch buttons no longer clipped.** Petrophysics tab: Workflow, Monte
      Carlo, and Field Dashboard now sit in one row inside the Batch group.

## FACIES block track (2026-07-19)

- [ ] **Facies layout renders colored blocks.** Run Electrofacies on a well (Petrophysics →
      Electrofacies), then pick the new built-in "Facies" layout in the ribbon layout picker:
      the FACIES track should show solid colored blocks (same colors as the crossplot's
      categorical Z-coloring), with gaps where FACIES is missing. The track header shows a
      striped swatch and "class blocks" instead of a min/max scale.
- [ ] **Blocks survive pan/zoom and well switching**, and the header swatch toggles the whole
      track's visibility like any other curve.
- [ ] **Any discrete curve can be block-rendered.** Layout Properties → a curve's Fill
      dropdown now has "Facies blocks" — try it on FLAG_PAY in a custom layout.
- [ ] **Composite export shows the blocks.** Export a composite (SVG or PDF) with the Facies
      layout: the FACIES track should print as colored rectangles at true scale.

## Electrofacies — k-means (Phase 10 increment 1, 2026-07-18)

- [x] **Petrophysics ribbon → Facies → "Electrofacies (K-means)…"**: pick input curves
      (defaults GR + RHOB + NPHI + DT + SP; leave a slot blank/absent and it's dropped),
      set **K** (number of facies, 2–12) and a **seed**, run on one or several wells. It
      writes a **FACIES** curve (integer 0..K-1). Re-running with the same seed must give
      identical facies (deterministic).
- [x] **Facies numbering is monotone in GR**: FACIES 0 should be your cleanest/sandiest
      class and the highest index your shaliest — confirm on a well where you know the
      sand/shale split. (Clustering is **per well**; the GR ordering is what makes the
      numbers roughly line up between wells.)
- [ ] **Crossplot QC**: open a Crossplot, set **Color = FACIES**. Points should be colored
      by discrete class from a qualitative palette with a **swatch legend (F0, F1, …)**
      top-right — not the blue→red continuous ramp.

## Monte Carlo uncertainty (2026-07-18)

- [x] **Petrophysics ribbon → Batch → "Monte Carlo…"**: pick a chain (the default VSH→φ→Sw, or
      one you saved in the Workflow Builder), click **+ Add uncertain parameter**, choose a
      parameter, pick a distribution (normal / uniform / triangular), set cutoffs + iterations,
      and **Run**. You get a per-well-per-zone table of **P10/P50/P90** net pay, NTG, avg PHIE,
      avg SWE and HPV, plus an **HPV histogram** (click a row to switch zones) with P10/P50/P90
      markers.
- [x] Requested upgrade (ROADMAP §4 P3): **finalize parameters → print LOW / BASE / HIGH
      curves** from the chosen result percentiles.

## Phase 8.5 — your method suite in core (remaining validations)

- [ ] **SSC — Sand-Silt-Clay (Advance tab)**: run on an LQR-style well with
      GRN + RHOB + NPHI (sandstone units). Check VSAND/VSILT/VDCL/VWCL, PHIT/PHIE/PHIFF,
      CBW/CWSH/BW, SWIRR_T/SWIRR_EFF and the `*_GR` GR-equivalent volumes against your
      the reference suite run. Defaults are the LQR `.info` values (wet clay 2.3/0.6, dry clay 2.71,
      wet silt NPHI 0.3, DCLF_SI 0.1). Two deliberate deviations, flag if they matter:
      (1) `RANNORMAL(SWIRR_MIN·PHIT, 0.005)` is deterministic here; (2) the Loglan's
      NPHIMA limit 0.5–5 (a copy-paste of the RHOMA limit) is corrected to 0–1.
- [ ] **SSPW (Advance tab)**: the Loglan exec body wasn't on disk, so the
      arithmetic (PHIT from VSH-mixed dry matrix, CBW = VSH·VOL_CBW_SH,
      CAPBW = VSH·(PHIT_SH − VOL_CBW_SH), PHIE = PHIT − CBW, PHIFF, SWIRR floor) is
      **reconstructed from the spec — please validate against your the reference suite "LAS PHIT
      PHIE" exports** and tell me any systematic difference; I'll adjust the equations.

## Phase 8b — report generator (2026-07-18)

- [x] **Report… dialog**: select a well first, then set study title, author, cutoffs
      (VSH ≤ / PHIE ≥ / SWE ≤ / optional PERM ≥), layout + print scale + page size, and
      **Render** — page through the preview (◀ ▶). Check: cover (title/well/field/
      interval/TD/KB), methodology table, zone parameter table (from your zone_params),
      pay summary (SAND/RESERVOIR/PAY rows with gross/net/NTG/avg PHIE/VSH/SWE/HPV —
      needs VSH+PHIE+SWE computed curves), then the composite pages.
- [x] **Methodology table is editable**: one line per row, `Parameter | Method | Remarks`.
      Blank = a built-in default reflecting your standard workflow. **Save Template**
      persists it (documents table) and it reloads next time.
- [x] **Save PDF…** writes the whole report as one multi-page PDF — open in Acrobat and
      check the tables (word-wrap in Remarks cells, header row repeated on overflow
      pages) and that ≤/≥ symbols render.
- [x] **Save PNG (page)…** rasterizes the CURRENT preview page at ~150 dpi for slide decks.
- [x] **Batch (N wells)…** exports one report PDF per well into a folder you pick,
      named `<WELL>_report.pdf`, using the same settings for every well. Wells that
      fail (no curves) are reported without aborting the rest.
- [x] **Tables only** checkbox skips the composite pages (fast parameter/pay-summary
      handout).

## Field Dashboard (Phase 9 increment 4, 2026-07-18)

- [ ] **By zone** table aggregates across wells: well count, Σ net, Σ HPV, mean N/G,
      net-weighted mean PHIE/SWE per zone.

## Deferred small item (Phase 7)

- [ ] **QC plot for sat-height**: the Pc/J-vs-Sw QC plot with the fitted curve + core
      points overlaid is NOT built yet — the `get_scal_pc` IPC is ready for it. Say "go"
      when you want it.

## Module-panel cleanup, Help tool, bulk Processing report, responsive resize (2026-07-21)

Five asks from your VSH-panel screenshot (SandiMin deferred for later review).

- [ ] **Module form no longer lists per-well results**: run a module (e.g. VSH from
      Gamma Ray) — the form now shows one summary line ("All N well(s) computed. Per-well
      details are in the Processing panel." or "…N need attention…") and the Processing
      panel comes forward. The old `✓ well: samples → curves` list is gone from the form.
- [ ] **Per-well detail lives in Processing → details**: expand a job's **▸ details** —
      running wells show individually; the narration paragraph that used to sit at the top
      of the module form is gone (it moved to Help, below).
- [ ] **Bulk failure report**: when many wells fail the SAME way, Processing → details shows
      **one card per reason** — "N well(s) failed — <message>", the well list (first 12 +
      "…(+K more)"), and a "→ what to do" advice line — instead of one row per well.
- [ ] **Help (?) tool**: click the **?** in the top quick-access bar (or right-click any
      panel → **Help for this panel…**). A guide opens for whatever panel is active — a
      module pane shows that method's description; other panels show a short blurb. (This is
      the placeholder that will later link to the full illustrated help library.)
- [ ] **Ribbon dropdowns still work**: open **Petrophysics → VSH/Porosity/Saturation**, and
      **Data → Import Logs/Import Data/Tools**, and **Project → Recent** — each menu must drop
      fully below the ribbon (this was a regression the review caught and fixed).
- [ ] **Resize the whole window**: the content panes (log views / plots / inspector) reflow to
      fill; **Wells & Tops, Processing, and Performance keep their width** (they're a fixed
      sidebar now). Try both wider and narrower — nothing should get clipped or leave dead space,
      and the ribbon stays reachable (scrolls if very narrow).
- [ ] **Close panes without the sidebar growing**: close a plot/log view — the freed space goes
      to the other content, NOT to the sidebar. Close *everything* down to just the sidebar and a
      blank **Workspace** pane fills the rest (rather than the sidebar stretching); open any log
      view/plot and the blank pane disappears again.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
