# A harsh critique of the ML module

**Written 2026-08-07, at Jauhar's request, against the state of `feat/ml-pane-round3`.**

This document is deliberately unflattering. It is not a bug list — the bugs get fixed and the list
goes stale. It is an account of what is structurally weak about the machine-learning capability in
SandiBumi, written so that somebody picking this up in a year knows where the bodies are without
having to find them the expensive way.

Every claim below was checked against the tree rather than remembered. Where a number is quoted it
was counted.

---

## 0. The one-paragraph version

The ML module is a well-provenanced wrapper around a Python program that nobody type-checks, tests
in isolation, or can debug in place — and the green gate that guards this repository proves almost
nothing about it. The engineering around the models is genuinely better than the incumbents'. The
models themselves are the least-verified code in the product, they ship without any statement of
their own uncertainty, and the module has now grown a feature that manufactures detail. That
combination is the risk: a curve that is confident, detailed, provenance-stamped, and wrong is far
harder to catch than one that is obviously broken.

---

## 1. The green gate does not test the machine learning

`tools\check.ps1` returns **788 passed, 0 failed, 36 ignored**. That number is quoted in commit
messages, including mine, as evidence the ML work is sound. It is not.

Every test that actually fits a model, predicts a value, or clusters a sample is `#[ignore]`d,
because scikit-learn is an optional dependency and rule 5 forbids the gate depending on one. Seven
of `ml::tests`' tests are ignored. So the default gate exercises: curve naming, unit declaration,
serde round-trips, log-set bookkeeping, string contents of the runner, and the SQL around the model
store. It exercises no arithmetic that reaches a curve.

This is the correct decision about the gate and the wrong place to have left it. The consequence is
that **the module's correctness rests on somebody remembering to run `cargo test -- --ignored`**,
and nothing in the repo makes that happen. It is not in `check.ps1`. It is not in a hook. It is a
sentence in a commit message.

### 1.1 And some of those tests assert on source text, not behaviour

Several tests I wrote this session — and several that predate me — take this shape:

```rust
assert!(ML_RUNNER_BODY.contains("out[labels < 0] = CLUSTER_REJECT"));
```

That asserts a line of Python **was written**. It does not assert it runs, that it runs on the path
in question, or that the result is right. It is a guard against a future edit deleting the line,
which has some value, and it is not a test of the contract it is named after. A reader scanning the
test names would reasonably conclude SB-MLA-021 is verified. It is verified in the weakest available
sense.

The honest fix is a Python-level test harness the runner can be executed against directly, with a
fixture matrix and no Rust in the loop. It does not exist.

## 2. The runner is a Python program stored as a Rust string literal

`ml.rs` is **8,020 lines**. A large fraction of it is Python held in `r#"..."#` constants
(`ML_BUILD_MODEL`, `ML_RUNNER_BODY`, `ML_APPLY_RUNNER`, `ML_EVAL_RUNNER`).

What this costs, concretely:

- **No syntax checking until runtime.** A typo in the runner compiles cleanly in Rust and fails on a
  user's machine, in a subprocess, with a stderr line.
- **No linter, no formatter, no type checker** over what is by any measure the most numerically
  consequential code in the product.
- **No breakpoints.** Debugging is `print` to stderr and re-run.
- **Editing hazard.** The whole file is one Rust module, so a search-and-replace intended for Rust
  can land inside the Python, and vice versa.
- **It defeats the repo's own delegation rule.** `~\.claude\CLAUDE.md` says mechanical edits are safe
  because the compiler catches them. For the embedded Python, the compiler catches nothing.

The counter-argument is real and is why it is like this: shipping loose `.py` files means they can be
edited, lost, or shadowed on a client machine, and rule 7 wants the Python to be a subprocess rather
than a dependency. Embedding is a defensible answer to that. **But the runner could be embedded as an
`include_str!` of a real `.py` file** — same single binary, same subprocess model, and the file
becomes lintable, testable and diffable. That this was never done is the single largest piece of
avoidable structural debt in the module.

## 3. Blind testing is off by default, and the default is the flattering one

`splitOn.checked` is never set true (`mlDialog.ts`). Open the pane, pick an algorithm, press Run, and
you get `r2_train`, `r2_cv` and no blind score.

`r2_train` is a fit statistic and answers nothing. `r2_cv` is better — `cv_score` uses `GroupKFold`
over whole wells when groups are available, which is a genuinely good decision most tools get wrong.
But the number the user actually needs, "how does this behave on a well it has never seen", requires
ticking a box.

The module *knows* this. `blind_sentence` prints, in a delivered report, *"not blind-tested — nothing
was held back, so there is no measurement of how this model performs on data it has not seen"*. That
is excellent, and it is downstream of the problem. The product carefully explains, in the
deliverable, that the user did not do the thing the product made optional.

**A defensible default would hold wells back unless the user opts out.** The reason it does not is
that on two or three wells a blind split leaves very little to fit on — a real constraint, and one
that argues for a default that adapts to well count, not for off.

## 4. A regression ships no uncertainty at all

Counted: **zero** occurrences of prediction intervals, quantile regression, or per-sample predictive
standard deviation anywhere in `ml.rs`.

Classification gets `_PROB`. Clustering gets `_PROB` for GMM. Regression — the case that produces
PERM and PHIE curves that go into volumetrics — returns a bare point estimate.

So a petrophysicist handed a predicted permeability log has:

- a single global R², possibly not blind,
- no idea which depths the model was confident about,
- no way to propagate the prediction's error into an HPV number,
- and no way to distinguish "this interval is like the training data" from "this interval is an
  extrapolation the model has never seen anything like".

That last one is the serious omission. **Nothing in this module detects extrapolation.** A model
fitted on 8–22% porosity, applied to a tight streak at 3%, will return a confident number. Random
Forest will return something inside its training range and look plausible; a linear model will
happily return a negative permeability. There is no leverage statistic, no Mahalanobis distance to
the training cloud, no applicability domain of any kind. For a product whose entire pitch is honesty
about what a number is worth, this is the biggest gap in it.

## 5. The spectral texture feature is the most dangerous thing here

I built it this session, at Jauhar's explicit request and with his explicit choice of method, and it
should be viewed with more suspicion than anything else in the module.

It manufactures detail. The result is statistically correct and positionally arbitrary — the beds it
draws are the right *size* and in the *wrong place*. The guards are: it is off by default, it writes
to `<base>_SIM` rather than in place, and the run note says in words that the detail is not a
measurement.

Those guards are weaker than they look:

- **The note lives in the run result, not on the curve.** Close the pane and the warning is gone. The
  curve remains, named `PERM_SIM`, and `_SIM` means nothing to a reader who was not there.
- **Nothing stops it entering a downstream fit.** `SIM_SUFFIX` appears 12 times in `ml.rs` and not
  once as an exclusion. A `_SIM` curve can be ticked as an input feature to the next model, at which
  point simulated detail is being learned from as though measured, and the provenance chain says
  nothing about it. **This is the one I would fix first.**
- **It can be exported to LAS and handed to a client**, where the suffix is the only surviving
  warning.
- **It looks better than the honest curve.** That is the whole problem. Given two logs, one smooth
  and one detailed, a reviewer's eye picks the detailed one, and it is the one that knows less.

The feature is a legitimate technique — spectral simulation is standard geostatistics — and it was
asked for. But it inverts the module's usual bias. Everywhere else, SandiBumi refuses to show a
number it cannot defend. Here it draws one.

## 6. Two clustering engines, no test that they agree

`facies.rs` implements k-means and GMM natively so the product works with no Python. `ml.rs` runs
scikit-learn's. `ml_shared_constants_py` now emits `KMEANS_N_INIT`, `KMEANS_MAX_ITER`, `KMEANS_TOL`
and `SEED_DEFAULT` from the Rust constants so the two are *configured* identically — good, and it
closed SB-MLA-023.

**Nothing asserts they produce the same clusters.** Same data, same k, same seed, two engines, and no
test compares the outputs. Identical configuration is not identical behaviour: k-means++ seeding
differs between implementations, tie-breaking differs, and the two use different RNGs, so the same
seed means two different things. The PRD names this as the origin of most of its `PRESENT-DIVERGENT`
findings and it is still open.

A user who runs electrofacies from the Facies ribbon and then the same clustering from ML Models will
get different facies numbering with no warning, and no way to know which they are looking at
afterwards. SB-MLA-029 (a facies mnemonic names the engine that produced it) is still `PRESENT-DIVERGENT`
for exactly this reason.

## 7. Feature identity is a mnemonic, and a mnemonic is not a measurement

A saved model records its features as an ordered list of curve names. `apply_ml_model` drives the
fetch from that list and the artifact refuses a reordered matrix — a genuinely good contract, pinned
by `a_model_refuses_a_matrix_whose_columns_are_in_the_wrong_order`.

It refuses *reordering*. It cannot detect *re-meaning*. "GR" on the training wells and "GR" on the
apply wells may be different tools, different vintages, different normalisations, different hole
conditions. The model will predict confidently across that gap and nothing flags it.

The product already has the machinery to say something useful here — `gr_normalize` exists, the QC
section computes per-curve statistics, and `SplitBalance` already reports how alike two sets of wells
are per feature. **The same comparison is not made between the training wells and the apply wells at
apply time**, which is the moment it matters most.

## 8. The coverage-segment feature builds a mosaic and presents it as a log

`coverage_segments` fits one model per distinct curve-availability pattern, so a field where half the
wells lack a sonic still gets a prediction everywhere. That is the right behaviour and it was asked
for.

The output is a single curve assembled from several different models. The provenance records the
segments. The curve does not. A reader plotting `PERM_ML` sees one continuous log; they do not see
that above 1800 m it came from a four-curve model and below it from a three-curve model with
materially worse statistics, and there is no companion curve saying which model produced each sample.
A step change at a segment boundary will be read as geology.

## 9. The pane is now large enough to be its own usability problem

`mlDialog.ts` is **3,985 lines** and the pane has five sections. Between them: well scope, interval,
input log set, feature curves, target, transform, mask, algorithm, task, per-algorithm parameters,
standardize, seed, blind split with three sub-controls, coverage segments, output resolution with two
modes, spectral texture, output curve name, output log set, model save name, plus a whole second
section repeating scope/interval/names for propagation.

There is no path through this for a user who wants a reasonable answer without making twenty
decisions. Every individual control is justified — I wrote the justification for several of them —
and the aggregate is a cockpit. The absence of a *recommended* setting anywhere in it means the
defaults are doing all the work, and §3 shows the defaults are not chosen for honesty.

## 10. The artifact's durability is unproven

A saved model is a joblib pickle of a scikit-learn estimator. `sklearn_version` is recorded and drift
is *detected* at apply time (SB-MLA-005), which is more than most tools do.

Detection is not durability. A pickle written by scikit-learn 1.5 may simply fail to load under a
later version — pickles are not a stable format and scikit-learn does not promise cross-version
compatibility. For a feature whose entire purpose is "which model made this curve, and can I run it
again", the honest position is that **the answer to the second half is: probably, for a while**.

Nothing in the tree tests loading an artifact under a different scikit-learn version, because nothing
could without a second interpreter. The risk is real, unmitigated, and undocumented outside this
paragraph.

## 11. Smaller things that are still wrong

- **`k` has no guidance.** Clustering defaults to `k = 5` with no elbow plot, no silhouette sweep, no
  suggestion. Silhouette is computed *after* the choice, on a subsample of 5000, and reported as a
  bare number with no interpretation.
- **A metric computed on a subsample does not say so at the point of use** (SB-MLA-020, still
  `PRESENT-DIVERGENT`). The silhouette is subsampled at 5000 rows and reported next to metrics that
  are not.
- **Masking is optional.** Nothing requires a bad-hole flag before fitting. A washout trains the model
  as readily as good rock.
- **Cluster ids are ordered by ascending first-feature mean**, which is only meaningful if the first
  feature is GR. The convention is documented and nothing enforces it; put PEF first and the
  numbering means nothing, silently.
- **PCA component signs are unfixed** (SB-MLA-048, `ABSENT`). Re-run and PC1 can flip, which reverses
  every crossplot made from it.

## 12. What is actually good, so this reads as calibration and not sourness

- **The provenance model is better than any of the three incumbents.** A curve names its model, the
  model names its rows by hash, its runtime, its mask, its effective parameters including the ones
  nobody typed, and none of it can be deleted without a refusal that names what would break.
- **The scaler travelling inside the artifact**, and the refusal on reordered features, close two
  failure modes that are silent everywhere else.
- **`GroupKFold` over whole wells** as the default CV, with the protocol named in the output, is the
  correct decision and most tools get it wrong.
- **The blind sentence printed into deliverables** — including the refusal to substitute a training
  score where no blind test was run — is the best single thing in the module.
- **Refusing to write an all-NaN curve** and reporting the failure instead (SB-MLA-013) is exactly
  right: an all-missing track is indistinguishable from work never done.

The pattern is consistent. **Everything about the bookkeeping around a model is excellent. Everything
about knowing whether the model is any good is thin.**

---

## 13. If only three things get fixed

1. **Extrapolation detection.** A per-sample distance to the training cloud, written as a companion
   curve. Nothing else in this list changes as many wrong numbers into visibly-uncertain ones.
2. **Exclude `_SIM` curves from being model inputs**, and carry the "simulated" fact on the curve
   rather than in a run note that disappears.
3. **Move the runner into a real `.py` file** behind `include_str!`, and give it a Python test
   harness. Everything in §1 and §2 follows from that one change.

---

_Written by Claude (Opus 5) at the user's request. The judgements are mine and are open to argument;
the counts and file references are checked and are not._
