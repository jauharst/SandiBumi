# Calibrating a saturation model to the user's own data

Build record for the two saturation calibrations and the fluid-contact work. Each fit is the
algebraic inverse of the module's own equation, never a re-derivation from the method note — so a
change to the saturation equation breaks the fit visibly instead of leaving a calibration that
quietly no longer inverts it.

> Moved out of `CLAUDE.md` on 2026-08-07 so it is read when it is needed rather than
> loaded every session. The contracts below are binding exactly as they were there.
> `CLAUDE.md` keeps the one-line contract and points here.

---

## RtC calibration (2026-07-31)

`sw_rtc`'s coefficients are a REGRESSION, not a constant, and `lrlc::run_rtc_fit` (Advance ▸
Calibrate RtC…, `rtcFitDialog.ts`) fits them to the user's own water leg. Four rules.

**The regression is the algebraic inverse of `sw_rtc`'s own equation, never a re-derivation
from the method note.** Set Sw = 1 in `Sw = [Rw·(1/Rt − Cex)/φt^M]^(1/N)` and the measured
excess falls out as `1/Rt − φt^M/Rw`; dividing by `φt·RSF` gives a plain 3-parameter OLS in
(CAPBW, Qv, 1). Deriving it this way means a future change to the saturation equation breaks
the fit visibly instead of leaving a calibration that quietly no longer inverts it. `qv_at()`
is shared by the module and the fit for the same reason.

**A water zone must be DECLARED — the fit refuses without a depth range or a wet-flag curve.**
There is no way to find a water zone without already knowing the saturation the calibration is
for, so inferring it would beg the question. The stakes are asymmetric: hydrocarbon REMOVES
conductivity, so pay samples make the apparent excess too small, the fitted model
under-predicts excess, Rt is under-corrected and Sw comes back too HIGH — it erases pay rather
than inventing it. A NaN wet flag is not wet.

**The `Cex <= 0` rejection is a second line of defence, not a substitute, and the tests record
exactly how it leaks**: it drops most obvious pay, but where the rock is most microporous the
true excess is large enough to mask the hydrocarbon and the sample survives — the guard is
weakest precisely where this method is used.

**RSF is held fixed and is not fitted.** It multiplies the whole bracket, so (a, b, c, RSF) are
not jointly identifiable; the returned coefficients belong to the RSF they were fitted with and
the result says so. An unfittable term (constant Qv) is reported as 0 with a note, never
guessed, and every excluded sample is counted and named. The dialog offers **Copy**, not
auto-apply — a calibration is a judgement made after reading R² and the exclusions.

## IMTS S-factor calibration (2026-07-31)

`sw_imts`'s S is the RtC problem again: it is *defined* as a measurement — S = lab CEC / XRD-
theoretical CEC (`docs/method_lrlc_rtc_imts.md`, IMTS §1) — and the app shipped a placeholder
for it. S multiplies the entire clay-charge term, so a wrong S scales Qv_eff directly and moves
SwT with nothing on the log to show for it. `lrlc::run_s_factor_fit` (Advance ▸ Calibrate S…,
`sFactorFitDialog.ts`) fits it from the user's own core. Five rules.

**The regression is the algebraic inverse of the module's own line, not a re-derivation** —
same discipline as RtC. `sw_imts` computes `cec_bulk = S · cec_theo_at(vk, vi, CEC_KAOL,
CEC_ILL, RHO_KAOL, RHO_ILL, RHOG, phit)`, so `S = CEC_lab / cec_theo_at(...)`, and `cec_theo_at`
is **shared by the module and
the fit** exactly as `qv_at` is shared with the RtC fit. Pinned by
`the_fitted_s_makes_the_module_reproduce_the_measured_cec`, which runs the fitted S back through
`sw_imts` and checks QVEFF lands on the laboratory value.

**CEC is a charge per unit MASS, so the denominator is dry-rock mass, not bulk volume**
(2026-08-22, DEC-094, AUDIT-2026-08-20 finding 9). `cec_theo_at` summed `V_clay × CEC_lit` -
a volume fraction times a meq/100 g - which is not a quantity at all. Because S is fitted as
the RATIO of the measurement to that sum, the dimensional error cancelled inside the fit and
re-appeared as a wrong Qv the moment the run met a different porosity than the plugs did.
That is why the fit is affected as well as the run, and why the parameter was RENAMED
`S_FACTOR` -> `S_FACTOR_GW` and the module now REFUSES without it (Jauhar's ruling, 2026-08-22)
rather than silently accepting a stale number that sits inside the declared range and plots
smoothly. The fit therefore takes a PHIT curve as a required input: the plug's own porosity is
part of the basis it inverts.

**The clay must come from the curves the RUN will use, not from the XRD table.** This is the
trap the dialog exists to close: calibrate S against XRD weight fractions, run against a
VDCL-derived VKAOL curve, and S is wrong by the ratio between those two estimates of clay —
silently, because both look like clay volumes.

**Through the origin, and least squares rather than the mean of the ratios.** S is a pure
scaling factor; an intercept would assert cation exchange where the clay model says there is no
clay, a claim the module's equation has nowhere to put. Through-origin OLS weights each plug by
its clay content, which is right — those are the plugs where Qv drives the answer, and on a
nearly clean plug the ratio is measurement noise over a small number.

**The drift detector is the SPREAD of the per-plug ratios (P10-P90), not the median-vs-fit gap.**
Two central values can only differ by as much as the ratio changes between the median plug and
the clay-weighted one; on a 12x clay range with a ratio running 1.13 → 0.40 that is barely 28%,
so a gap threshold loose enough to survive noise never fires on real drift. The spread has no
such ceiling and catches the same case at 2.8x. Both are reported; the gap note is secondary and
says only that the disagreement is *systematic with clay content*. Pinned by
`a_drifting_s_shows_up_in_the_spread_of_the_per_plug_ratios`.

**S above 1 is a note, never a clamp.** The method expects lab CEC *below* the XRD-theoretical
value. Above 1 the clay model is under-calling exchange capacity, and the usual cause is a
mineral it does not carry — smectite runs 80-150 meq/100g against illite's 25, so a few percent
dwarfs the modelled charge — which makes that S wrong wherever the missing mineral's fraction
differs from the cored plugs.

Two further contracts. A plug further than `depth_tol` (default 0.15, one standard 6-inch
sample) from any log sample is **dropped and counted, never snapped** to the nearest one — but
the test records the honest limit: a shift that is a whole number of sample intervals is
invisible to any depth-tolerance check, so this is not a substitute for depth-shifting the core.
And **S and the literature CEC constants are not jointly identifiable** (S multiplies them), so
the constants are held fixed, echoed in the result and copied alongside S.

**Calibration QC scatter** — `fitScatter.ts` is shared by both fit dialogs, because a calibration
reduces a core or a water leg to two or three numbers and R² cannot say *how* it failed:
curvature, one well parked off the trend, a cluster the fit is being dragged by. That is what both
backends return `points` for. Two rules are the reason it is one module rather than two plots.
**A measured-vs-fitted plot forces both axes to the SAME range** so the 1:1 line lands at 45° —
scale them independently and the aspect ratio alone makes a good fit look biased or a biased one
look clean. **A through-the-origin plot forces the origin onto the page**, because proportionality
is the model's claim and cropping to the data hides whether the cloud actually heads for zero.
Points are coloured by WELL (a single well pulling a field calibration is the question the table
cannot answer), out-of-window points are SKIPPED rather than clamped to an edge, and the hover
readout names the well and depth. RtC plots measured against fitted; the S fit plots the
regression itself, lab CEC against modelled CEC, because with one predictor only that version puts
clay content on the x axis and turns the P10-P90 spread into a shape with a name.

**The first paint is synchronous and must stay that way.** `requestAnimationFrame` only fires
while the tab is compositing, so deferring it leaves the plot blank in an occluded or background
window — and `attachResizeRedraw` schedules through rAF too, so there is no fallback. The handle
exposes `redraw()` and each caller invokes it right after inserting the element. Related and
equally load-bearing: the canvas context is scaled by the `dpr` that `fitCanvasBackingStore`
returns, or a HiDPI screen draws the whole plot at half scale in the corner.

**Picking the CEC measurement** — `db::list_aux_item_catalog` returns every measurement name in
the project's point data (from the ACTIVE delivery of each dataset) with its row count, well count
and **numeric-row count**, and the S dialog turns it into two dependent selects. Project-wide and
unfiltered by well for the same reason `list_well_param_overrides` is: one grouped scan beats N
round trips or an `IN (...)` list long enough to hit a binding limit on a 2000-well project, and
"what could this box name" is the question a picker actually asks — a run's own exclusion counts
still report what the chosen wells turned out to hold.

`numeric_rows` is the part that matters. A descriptive item cannot set a scaling factor, so a
text-only one is shown **greyed with "no numeric values"** rather than hidden: "LITHOLOGY is
there but it is text" answers the question the user was about to ask by running the fit. A dataset
with nothing numeric in it gets an explicit "(nothing numeric in this dataset)" placeholder rather
than a silently empty select. With no point data at all the dialog falls back to typed names and
says so in a VISIBLE note — `formRow`'s hint is a tooltip, and "there is nothing here to pick
from" is not something to hide behind a hover.

**Accepting a calibration** — `calibrationApply.ts`, shared by both fit dialogs, writes the
coefficients as `zone_params` overrides through the new atomic `db::set_zone_param_batch(conn,
zone_name, entries)`. `set_well_param_overrides` is now just its `"*"` scope, so the parameter
grid and an accepted calibration take the same transactional path. Four rules.

**The default scope is `wells_fitted`, a field on both results and NOT derived from `points`** —
the display points are decimated, so a well can vanish from them entirely, and a scoped well that
contributed nothing was never calibrated. Applying to the wider scope is offered (fit-here-apply-
there is the point of a field calibration) but it is a choice, it names the uncalibrated wells,
and the option is hidden when it would be identical to the default.

**The held-fixed constants are written in the same batch or not at all.** RtC writes RSF with
A_CAP/B_QV/C0, the S fit writes CEC_KAOL/CEC_ILL and the two clay grain densities RHO_KAOL/RHO_ILL with S_FACTOR_GW. In both cases the constant and the
coefficients are not jointly identifiable, so writing one without the other yields a calibration
that is silently for different rock.

**One transaction, one undo.** A half-applied saturation calibration would leave a field carrying
two answers with nothing on the log to say where the boundary fell.

**Undo restores "no override", not zero.** The previous values are read first — from
`list_well_param_overrides` for `*` (one project-wide query) or `list_zone_params` for a named
zone — and a `None` in the batch DELETEs the row. A parameter silently pinned to zero is a wrong
answer that keeps computing. Pinned by
`a_none_in_a_zone_batch_clears_the_row_instead_of_writing_zero` and
`a_named_zone_batch_leaves_the_whole_well_scope_alone`.

Both fit dialogs offer **Copy** as well as Apply. Both also paint their own run-button label:
`buildWellScope` deliberately does not fire `onChange` during construction (`wellScope.ts`), so
a caller that relies on it opens with a blank, disabled button — `rtcFitDialog.ts` did, and is
fixed here.


## A fluid contact is identified by three things (2026-08-01)

Every calculation parameter in this app already lives at MARKER level: `zone_params`
`(well_id, zone_name, param_name)`, where `zone_name = '*'` is the whole well and a named zone is a
top (`db::zones_from_tops` builds one zone per marker, named after it). RHO_SH, NPHI_SH, A_CAP,
B_QV, C0, RSF, M, N, RW, FWL, the cutoffs — all of them, resolved by `workflow::resolve_param_arrays`
as manifest default → dialog value → `*` → named marker.

**Fluid contacts were the exception, and it cost numbers.** `fluid_contacts` carried no marker at
all, so `check_contact_consistency` pooled every contact of a type across the project. Three fixes,
each closing a way of pooling surfaces that are not the same surface:

- **`contact_zones` is a link table, not a column.** The relationship is many-to-one in BOTH
  directions a field is built: two stacked sands can each have their own contact, and several
  stacked sands in one hydraulic unit can SHARE one. A single column says the first and not the
  second, and a comma-separated list in a column is not a list. No rows = no marker stated, which
  stays a real answer — a field-wide datum cuts across markers, which is why the plane fit exists.
- **`fluid_contacts.compartment`** names the fault block. Two compartments are not in pressure
  communication and have no reason to sit on the same contact; pooled, the fit lands between them
  and flags BOTH blocks. Pinned by `two_compartments_are_two_contacts_even_in_the_same_sand`, whose
  control asserts the pooled version really is wrong (rms > 10) — otherwise an implementation that
  had stopped fitting anything would pass.
- **The QC group is `(type, compartment, marker SET)`**, and the markers are SORTED in `group_key`,
  so a contact entered as [B, A] and one entered as [A, B] are one group. Passing none checks the
  contacts that state none — it never means "all of them".

`db::migrate_fluid_contact_zone` is ADD COLUMN + CREATE TABLE, no rebuild, no backup. Existing
contacts get a NULL compartment and no marker rows: nothing in a stored contact says which sand or
block it was picked in, and inventing the association would be worse than admitting there is none.

**The two FWLs, which is the defect this was really for.** A free-water level lived in
`fluid_contacts` (drawn on the correlation panel) AND in `zone_params` (what `sw_height` computes
from), with nothing reconciling them — so the log could show one surface while every saturation in
the report came from another, both entirely plausible. `contacts::check_fwl_agreement` measures the
gap and `apply_fwl_to_zone_params` copies the pick across. Four rules:

- **An explicit copy, never a live read.** Having `sw_height` resolve its FWL from the contact table
  at run time would give the project two sources of truth reconciled invisibly at the moment of
  calculation, and no stored run could afterwards say which it used. Same shape as every other
  calibration here: look, Apply, one transaction, one undo.
- **A contact governing SEVERAL markers produces one row per marker**, because the parameter is per
  marker — one row for the contact would hide a sand whose parameter had drifted.
- **An MD contact is reported, never converted.** The stored parameter carries no reference of its
  own (`satheight.rs` documents FWL as "the same reference as the vertical-depth input", a property
  of the RUN), so converting to force a comparison would assert something the project never said.
- **A contact with no marker is skipped**, not matched against `*` — one pick must not silently
  rewrite every zone.

UI: Plot ▸ Multi-Well ▸ **Fluid Contacts…** (`fluidContactsPanel.ts`), a pane so it sits beside the
correlation panel it is about. The stored table with per-row editing, the QC group by group, and the
FWL reconcile with its undo.

