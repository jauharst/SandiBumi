# Findings that moved numbers

Build record for the review-triage fixes — the four ways an operation reported success having done
nothing, what reached a client folder, the cutoff rules, and the two Pittman rows that were never
in the published table. The triage itself is `review_triage.md`.

> Moved out of `CLAUDE.md` on 2026-08-07 so it is read when it is needed rather than
> loaded every session. The contracts below are binding exactly as they were there.
> `CLAUDE.md` keeps the one-line contract and points here.

---

## Four ways an operation reported success having done nothing (2026-08-01)

`docs/review_triage.md` findings 13, 17, 19 and 20, fixed together because they are one defect
wearing four faces: **the honest signal existed but was gated on the failure being total.** A
script that raised on every sample was caught; one that raised on half was not. A chain that
reported a terminal status released the project switch; one that died did not. A backend that
refused a non-finite constant never saw the empty field the frontend had already turned into 0.0.
Three sample editors checked their UPDATE's row count; the fourth did not.

Each was pinned AS-IS by a test written to go red when fixed, so the fix and the test rewrite are
one action rather than two.

**A partial failure is a WARNING, not an error, and not silence.** `EquationRunResult.note` is a
third channel beside `error`: the curve WAS written and an equation guarded by a domain check
legitimately refuses some depths, so calling that a failure trains the user to ignore the channel —
but a holed curve with nothing on the log to prompt a second look is indistinguishable from one
whose inputs were simply absent, which is the ordinary innocent case. **The raises are counted at
the evaluator, never from the output**: counting MISSING output samples would flag every equation
ever run over a washout, and a warning that always fires is one nobody reads. Samples whose inputs
were already MISSING never reach the evaluator (the `has_nan` short-circuit), which is exactly what
makes the count mean something — these are depths where the script had real numbers and still could
not answer. A non-finite result (`exp(1000)`) is counted and worded separately, because telling the
user their script threw when it did not sends them reading the wrong line. This is Rhai-specific:
`run_python_equation` runs the whole well in one call, so a `raise` fails that well outright.

**A dead chain worker releases the project switch, and the entry is still not pruned.** The chain
registry has no prune (contrast `jobs.rs`) — `register` inserts, `set_status` mutates, nothing
removes — so a worker that died without a terminal status left `any_active` answering true and
Open/New/Compact Project refused for the rest of the session, each telling the user to wait for a
job that would never finish. `catch_unwind` in `run_workflow_chain` now reports the panic on BOTH
registries (`chain::failed` for the Builder's poll, `job.failed` for the Processing panel). The
entry deliberately survives, because the reason is the whole point of not pruning it; what changed
is that it reaches a terminal status. **The panic's own message is carried through** —
`panic!("literal")` gives a `&str` and `panic!("{x}")` a `String`, anything else has no readable
message and says so rather than printing a type name. Honest limit, recorded at the call site: a
panic that was HOLDING the DB mutex poisons it, and no catch here rescues that.

**A dialog refuses its own bad field, in the dialog.** `curveEditDialog`'s numeric helper fell back
to a default on an unparseable value — which is safe only where the default is the identity. It is
for `mul` (1) and `add` (0); it is not for `value`, where 0.0 gAPI over an interval looks like a
measurement of very clean rock, and it is not for `top`, where 0 does not mean "no interval" but
"from surface". Those three are refused by name in `var(--warn)` with focus on the first, which is
`needWell.ts`'s rule: the user is looking at this dialog, and a refusal in a corner of the window is
one they will not read before clicking Apply again. Passing the non-finite value down to the
backend's existing guard was the one-character alternative and gives a message that cannot say
which of six fields was wrong.

**`update_well_field` checks its row count like its three siblings.** Without it, an edit against a
well deleted in the Wells & Tops pane returned Ok: the cell showed the new value, the status bar
reported the edit, and an undo entry was pushed for a change that never happened. The message
deliberately does NOT name the well — the identity here is a UUID the user has never seen, unlike
the depth its siblings quote — so it says what happened and what to do. A bad column stays a
separate refusal: that is a programming error, this is a stale grid.

## What lands in a client folder (2026-08-01)

`docs/review_triage.md` findings 12, 15 and 18 — three defects in the PDF report, all of them
invisible to the person who exported it.

**A batch never writes two wells to one file.** `well_name` carries no uniqueness constraint, and an
import with attach OFF creates a second record under the same name by design; the filename sanitizer
widens the collision further, because every non-alphanumeric maps to `_`, so two distinct names can
land on one stem. When they collided the second write silently OVERWROTE the first and both paths
were still reported as written — a 3-well batch said "wrote 3 file(s)" over 2 files on disk, and the
report kept was the last well's under the first well's name. `unique_stem` suffixes the duplicate.

**The first well of a colliding pair keeps the plain name**, so a delivered folder does not rename
the well anybody was expecting, and **only collisions WITHIN one batch are suffixed** — re-running a
batch into the same folder should overwrite its own previous output, and suffixing around files
already on disk would grow a folder of `_2`, `_3`, `_4` every time the button was pressed. A name
that sanitizes to nothing falls back to the well id, because `_report.pdf` is not a deliverable.

**The well name is resolved BEFORE the render, and that is the root rather than a tidy-up.** The
success path used to look the name up for the filename while the failure path reported the raw
UUID, so an error nobody could attribute and a success that silently replaced a file were one gap
with two faces. One lookup now serves both.

**The cover dates itself to the LOG, not to the composite's print window.** It read the interval
off the composite pagination, which honours the render's depth window — so setting a print window
re-dated the whole report, including the tables the window never touched (`run_pay_summary` works
per zone and knows nothing about it). A report rendered over 5 m carried a pay table covering every
zone in the well, and on a **tables-only** render there were no log pages left to show the reader
that the window was only ever a print setting. `db::logged_interval` is the replacement: two
aggregates over the leading column of a primary key, standard curves first, computed curves as the
fallback for a well carrying only derived logs.

**A print window is stated BESIDE the interval, never instead of it**, only when it genuinely
narrows, and never on tables-only where it describes nothing in the document. A line that always
appeared would train the reader to skip it. The pagination remains the fallback for a well with no
curve rows at all, which prints the same 0.0 – 0.0 it always did rather than inventing a new failure
mode. This also unblocks the audit's tables-only slowness — the composite render was what supplied
the cover's one remaining fact — though skipping it still needs `pw`/`ph` and the well name to come
from somewhere else, and that is a separate change.

**Every report page carries the mark.** The cover had it, every composite page had it, the Word
document and the PowerPoint deck had it — `table_pages` and `note_page` did not, so the methodology,
zone-parameter and pay-summary pages were the only unmarked surface in the deliverable set, and a
reader who extracted or photocopied the pay summary got an unattributed page. It is applied **after
pagination**, once per finished page, rather than at either `pages.push`: there are two of those,
and a mark added at one would silently miss every continuation page of a long pay summary — exactly
the page most likely to be read on its own. The test asserts it on EVERY page rather than sampling
one, because the failure mode is a page type being missed and a spot check is how it stayed missed.

## Cutoffs, and what a run is allowed to claim (2026-08-01)

`docs/review_triage.md` findings 8, 7 and 10 — three ways the pay engine and the module runner said
more than they could support.

**A permeability cutoff survives a chain that MODELS permeability.** `montecarlo`'s `has_perm_cut`
asked whether PERM was in `raw_pool`, and `build_plans` fills that pool only from LogIn mnemonics
**no step produces**. So a chain reading PERM from the project got a working cutoff, and the moment
a `perm_coates` was inserted ahead of it PERM became a produced curve, left the external set, and
the cutoff went quiet. Exactly backwards: a study that models permeability is the study whose
permeability cutoff matters. It now asks `produced` as well — not turning a cutoff on with no data
behind it, because the realization pool carries produced curves and PERM really is there when
`zone_metrics` reads it.

**A well with no permeability used to escape the cutoff; it now fails it.** See "Four answers that
moved numbers" below — the well-level test is gone and `PaySummaryRow.perm_cutoff_no_data` (renamed
from `perm_cutoff_skipped`) carries the inverted meaning.

**A run that reports failure must not also version an interpretation.** Phase 2 of
`run_workflow_module_into` wrote for any well whose outcome was `Computed` with a non-empty output
map — and an all-MISSING map is still non-empty — so rocktyping on a well with porosity but no
permeability reported its failure AND versioned RQI, PHIZ, FZI, R35, PGEOM, PSTRUC, RT and PERM_RT
into the Curve Catalog as curves blank from top to bottom. The values were honestly MISSING; the
cost was that the catalog stopped distinguishing *"never run"* from *"ran and could not answer"*,
burning a log-set version on the second as though it were the first.

The rule is deliberately not "drop blank curves". One helper, `answered`, now decides all four
things that have to agree: the Processing panel's item state, whether a log-set version is
allocated, whether anything is written, and what the result reports. **A single all-MISSING output
ALONGSIDE finite ones is still written** — a flag curve nothing triggered is a real answer, and
dropping one output would leave the written set inconsistent with the one the module declares. The
gate is over the whole output map, never per curve. The all-MISSING case is also checked BEFORE the
set/write branches in the result assembly, because that well is now deliberately given no output
set and falling through would report "no output set allocated", naming the mechanism instead of the
cause.

## A dropdown that says which Larionov it is (2026-08-01)

`docs/review_triage.md` finding 21, plus the bookkeeping of 11 and 14.

`ArgSpec` gained `choice_labels` (`#[serde(default)]`, parallel to `choices`, empty means "show the
id") and `opt_labelled` builds them. Only `vsh_gr` uses it so far, because that is the one the
finding proved was dangerous.

**The IDS are unchanged, and every LABEL leads with its own id.** `choices` are stored in
`params_json` on every saved run, so renaming one would orphan every run that used it, and a label
that REPLACED the id would leave a user reading a stored run unable to match the two. The label is
a superset of what was there before, never a substitute.

**Why this option and not the others.** `OPT_GR`'s choices were the bare strings `LARINOV1`,
`LARINOV2`, `LARINOV3`, `STIEBER1..3` — no rock age, no coefficient, no tooltip — so the only place
a user was told which is which was the manual test plan, and the plan had the two Larionov
attributions the wrong way round. `LARINOV1` is `0.33·(2^(2·IGR) − 1)`, published for Mesozoic and
older rocks, giving 0.330 at IGR 0.5; `LARINOV2` is `0.083·(2^(3.7·IGR) − 1)`, the Tertiary /
unconsolidated set, giving 0.216. Picking the wrong one returns a shale volume more than half again
too high through the whole intermediate-GR interval — which is exactly where the VSH cutoff decides
net pay, on a curve that looks entirely normal at both endpoints with nothing downstream able to
catch it.

**`LARINOV3` is stated by its coefficients rather than attributed.** Nothing in the repo cites a
source for `0.127·(3.15^(2·IGR) − 1)`, and inventing a rock age to make the dropdown look complete
would read exactly as authoritative as the two that are real.

**The label and the arithmetic are pinned to each other.**
`every_vsh_gr_transform_lands_on_its_published_coefficient` ties the code to the closed forms;
`the_vsh_gr_labels_agree_with_the_coefficients_they_describe` ties the closed forms to what the user
is told. Between them the loop closes — and it needs to, because a label claiming Tertiary above a
Mesozoic coefficient set is the same defect moved one layer out, and just as invisible.

## Four answers that moved numbers (2026-08-01)

`docs/review_triage.md` findings 6, 7 and 16 — three the triage held back because each was a
petrophysical judgement rather than a defect. Jauhar answered them, and every one changes what a
run computes. (Finding 9, the fourth, has its own section below.)

**A geothermal gradient belongs to the WELL, so a per-zone override is refused** (finding 6,
*"temperature is curves only"*). `precalc` evaluates `SURF_TEMP + TEMP_GRAD × TVDSS` from surface at
every sample rather than integrating down through the zones above it, so a lower zone's own gradient
did not bend the profile, it STEPPED it — a 0.03 °C/m well with 0.035 below 1500 m jumped 10.5 °C
across 100 m where the trend rises 3.0. Rock temperature is continuous, and it reaches Sw through
Arps.

`ArgSpec.well_scope` (`#[serde(default)]`) marks such a parameter and `param_well` builds one.
`precalc`'s SURF_TEMP/TEMP_GRAD and `ftemp_grad`'s TSURF/TGRAD/BHT/TD_BHT carry it;
`workflow::resolve_param_arrays` is the one place that enforces it.

**Refused by name with the fix, not silently ignored** — dropping the override would change the
well's temperature, and so its Sw, with nothing on the log to say why. **The `*` well-wide scope
still applies**, and that distinction IS the rule: `*` gives the well one value, which is what a
trend has, while a named zone gives it a different value part way down. It also keeps the per-well
parameter grid working, which matters because wells in one field genuinely differ. **Only an
override that would actually bite is refused** — one naming a zone the well lacks was always inert
and must not become a new failure.

**PSURF/PGRAD are deliberately NOT well-scoped, and the asymmetry is the physics**: a pressure step
at a formation top is a pressure compartment, which is a real thing rock does. Asserted from both
ends — `a_geothermal_gradient_is_refused_per_zone_and_accepted_per_well`, and, driving the real
runner, `a_per_zone_pressure_gradient_reaches_exactly_its_own_samples` (the old T-PREP-05 test,
moved onto pressure).

**A requested permeability cutoff is always active** (finding 7, *"no relation between em, wells
still can have perm curves"*). Whether a cutoff applies has no relation to whether this well was
cored, and permeability can be MODELLED where it was not measured, so lacking a measured PERM is not
a reason to be let off. `has_perm_cut` is now just `req.perm_min.is_some()`; `classify_sample`'s
rule — a sample that cannot be SHOWN to pass, fails — is the only one left, and the two halves of
what was one rule finally agree. **This moves reserves**: an uncored well books zero net pay where
it booked full.

`PaySummaryRow.perm_cutoff_no_data` (renamed from `perm_cutoff_skipped`) survives with its meaning
inverted, because the reader's problem is unchanged and only its direction moved: a well booking
zero across every zone looks exactly like a wet well. It means *"a cutoff was requested and this
well has nothing to answer it with"*, never *"this well has no permeability"* — a flag that fired
without a cutoff would appear on every report anyone ever ran. It is surfaced **where the number is
read**: the client PDF prints a note under the pay table saying the zero records an absence of
evidence rather than a dry reservoir, and the Field Dashboard names the wells whose zeros are being
averaged in. `n_classified` could never carry this — the well is fully interpreted, so it is above
zero either way.

**PHIE is floored at 0.001, at the curve AND at every pay path** (finding 16, *"always limit phie to
0.001"*). *Always* is load-bearing: the motivating case is a **vendor** PHIE arriving by LAS, which
never passes through a porosity module, so flooring only where modules write would have missed it. A
tight carbonate streak on a sandstone matrix reads slightly negative, clears the VSH cutoff, is
flagged SAND — and its `PHIE·(1−SWE)·h` was SUBTRACTED from the zone's hydrocarbon column, measured
at more than 20 % below the floored answer while RESERVOIR and PAY stayed byte-identical.

`modules::PHIE_FLOOR` is the constant; `workflow::floored_phie` is the one helper both the pay
summary and the cutoff sweep use, because the two must agree at the same cutoffs. **0.001 rather
than 0.0** is his call and not an epsilon: a hard zero is a legitimate reading — shale has no
effective porosity and the ≥95 % VSH branch says so — so flooring at zero would make "no porosity"
and "the arithmetic went below zero" indistinguishable. **The floor lands on `PHIE` only**;
`PHIE_DEN`/`PHIE_DN` are the declared unlimited twins and keep the excursion, because it is the
evidence that RHO_MA is wrong. **The NaN guard is load-bearing, not defensive**: `f32::max` returns
the other side when one is NaN, so without it a MISSING sample becomes a real 0.001 and starts
counting toward `n_classified`. And the floor must stay far below any real cutoff, or it would stop
a stringer being subtracted by quietly promoting it into reservoir instead.

## Pittman's table, and the two rows that were never in it (2026-08-01)

Finding 9, closed with the paper in hand: Pittman, E. D., 1992, AAPG Bulletin v. 76 no. 2,
p. 191–198, **Table 1** (p. 196). Two of the nine shipped rows were wrong — PR50 carried Table 1's
**r45** coefficients, and PR75 matched **no published equation at all**. r10 through r40 were
correct.

**The mechanism is the lesson.** Pittman publishes FOURTEEN rows in 5 % steps; `rocktyping.rs`
writes nine. r10–r40 are contiguous, so reading straight down the paper works — until the first
skip, after which every line below is read one high. A slip between two ADJACENT rows would have
produced a plausible number with no symptom whatever; the only reason this one was caught is that it
made the family non-monotone, and it inverted across **65 %** of the paper's own sample range,
including ordinary 25 % porosity sand.

So the fix is not two corrected numbers. **`PITTMAN_TABLE1` holds the published table in full and
`PITTMAN_RX` carries no coefficients at all** — only a mnemonic and a saturation, with the numbers
resolved by `pittman_coef`. There is one copy of the paper's arithmetic in this repo and the shipped
subset cannot drift from it; adding r45/r55/r60/r65/r70 as outputs is a line plus a `log_out`, never
a re-reading of the paper. `every_shipped_pittman_row_is_a_published_one` is the check that was
missing. `pittman_coef` returns a NaN triple rather than panicking on an unpublished saturation —
release builds are `panic = "abort"`, and NaN coefficients yield MISSING, which is the module's own
word for "no answer".

**The published table is not monotone either, below about 11 % porosity, and that is NOT
corrected.** Found while writing the test meant to close the finding. The rows are INDEPENDENT
regressions whose porosity exponent steepens down the table (−0.385 at r10 to −2.626 at r75 —
Pittman notes the porosity term is statistically insignificant through r35 and significant from
r40), so as porosity falls the high-saturation rows climb faster and overtake the low ones. At 5 %
porosity and 1 mD the family falls correctly to r40 = 0.767 µm and then turns back UP through
PR50 = 0.862 and PR75 = 1.108. Measured over the paper's own range, the shipped set is strictly
monotone at every permeability once porosity reaches **11.16 %**.

Forcing the ordering — a running minimum, a refit — would report radii Pittman never published,
which is the move the provenance rules exist to prevent. What ships instead is the boundary, stated
in the module doc: use a LOW apex (r25–r35, where the porosity term is insignificant) in tight rock,
and treat PR50/PR75 there as extrapolation. The doc also carries the paper's own caution that
accuracy diminishes above the 55th percentile (R falls from 0.926 at r20 to 0.820 at r75), and that
k must be UNCORRECTED air permeability.
`the_published_pittman_rows_cross_over_in_tight_rock_and_that_is_the_papers_own_arithmetic` pins the
crossover, the 12 % boundary and the old table's inversion at 25 % sand side by side — so "still not
perfectly monotone" can never be read as "the correction did not work".

