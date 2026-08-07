# 24. Machine learning and advanced analysis — requirements

**Dossier.** `docs/research_2026-08/cross_tool/ml-advanced.md` — 2,222 lines, read in full: the
method inventory (§1, including §1.5 "explicit no evidence held"), the equation comparison (§2),
the nine differences that matter (§3), the per-item optimal choice and the ledger disposition
table (§4, §4.7), the adoption spec (§5.1 forms F1–F18, §5.2 parameter table, §5.3 fixtures, §5.4
`FINDINGS.md` rule bindings), the gaps and escalations (§6), the source register (§7) and the
authoritative `## Critique disposition` (§8). The disposition is treated as authoritative over
any body text it corrects, per CONTRACT §4 rule 2 — it is the reason four of this chapter's
findings read differently from the dossier body (ML-9, ML-10, ML-12, ML-13). `ml-advanced_critique.md`
was **not** read, per CONTRACT §4 rule 3.

**Evidence tiers held.** **T1** — SandiBumi's own Rust and its embedded Python (read directly for
§3), Techlog's bundled Python ML stack read from the install tree, and Techlog's shipped
`MissingValue` sentinel. **T2** — the IP 2025 and IP 2018 CHM ingests; this domain's most
consequential equation (the Cuddy combination rule, F2) rests on a **single T2 raster** and that
is stated wherever it is used. **T3** — Techlog 2018.2 shipped HTML help and Geolog V14 shipped
help, both read read-only at source. **T4** — the 2006 Rabiller Facimage guide in petro-kb, whose
own note records that only its overview and the first MRGC pages were read. The dossier's **PKB**
label (delivered-project decision records) is carried as a labelled subclass of **T4** and is
never treated as a vendor default.

**A tier caveat this chapter carries forward.** IP is the **only printed SOM source in the
corpus and has no external arbiter** (dossier §3.3, ML-4): Techlog ships no SOM training math at
all — a verified negative result over the whole shipped `Doc\` tree — and Geolog exposes
`Iterations`/`Shakings` without a decay law. Every SOM requirement below is therefore
single-sourced, and the one place where the single source is provably degenerate is handled by
deviating from it explicitly rather than by transcribing it.

**Author date.** 2026-08-07.

**Requirements.** 65 (`SB-MLA-001` … `SB-MLA-065`). **P0: 10.**

**Parameters.** 105 rows in §5, counted by parsing the table rather than by estimate. Of those,
**15 ship `ABSENT — ships with no default`**, 5 are `NON-ADOPTABLE — cited for verification`, and
3 are cross-references to `15_sat-height-rocktyping.md`. The remaining 82 carry a value with a
checkable source — 44 are SandiBumi's own constants at `file.rs:line` (T1) and 38 are vendor
defaults at page-level citations (T2/T3). Fifteen absences in one domain is not a gap in the
research; it is what the evidence supports, and §5's preamble says why for each.

**Acceptance tests.** 61 (`SB-MLA-T01` … `SB-MLA-T61`). Four are labelled `CHARACTERIZATION`.

**Traceability.** §8 carries **218 disposition rows** against the dossier's items, on a counting
basis stated in §8.0 together with the three decisions that could have been taken differently and
the one place where the dossier's own two counts measure different things. **24 requirements have
no dossier antecedent** — they come from reading the shipped source — and are enumerated in §8.13.

**Cross-cutting requirements this chapter carries.** `SB-CORE-002` (a degraded or failed result
is never presented as a clean one), `SB-CORE-006` (one name, one equation), `SB-CORE-007` (one
definition per constant and transform), `SB-CORE-010` (every computed curve answers "how was I
made?") and `SB-CORE-011` (byte-identical re-run). This chapter is the **hardest case in the set
for `SB-CORE-010` and `SB-CORE-011`**, and §1 states why. All five are defined in
`04_CORE_REQUIREMENTS.md` §15.1; **no new `SB-CORE` identifier is minted here.** Two candidate
`SB-CORE` gaps are raised for Jauhar in §7 rather than minted.

---

## 1. Scope and boundary

This chapter owns the machinery that learns a relationship from data instead of asserting one:
unsupervised clustering (k-means, Gaussian mixture, hierarchical/agglomerative, SOM, density-based),
supervised prediction of a continuous or categorical curve from other curves (regression,
classification, k-NN, tree ensembles, neural nets), the fuzzy-logic curve and facies predictors,
dimensionality reduction (PCA, MCA, t-SNE) and its diagnostics, model propagation from a trained
model to unseen wells, the algorithm-comparison leaderboard and its cross-validation protocol,
cluster-count and cluster-quality diagnostics, contingency/confusion tabulation between a
predicted class curve and a reference one, and — the load-bearing part — **the provenance record
that makes any of the above defensible in a deliverable.**

It owns those as *methods*. Every one of them is applied to a petrophysical question that belongs
to another chapter, and the boundary is the same in each case: **this chapter owns the estimator,
the other chapter owns the quantity being estimated and the units it is reported in.**

**The chapter's own thesis, stated once here because every section serves it.** A trained model
is the least reproducible object in petrophysics. A Simandoux `Sw` is reproducible from six
numbers and an equation; a random-forest permeability is reproducible only from the exact training
rows, the feature list *in order*, the scaler fitted on those rows, the hyperparameters, the
random seed, the library version, and the train/apply split — and if any one of those is missing,
the number cannot be regenerated, only re-approximated. The dossier's §3.7 finds that **none of
the three incumbents seeds anything**: IP documents SOM and neural training as irreproducible run
to run and ships no seed control anywhere, Techlog states outright that "K.mod does not display
the same results twice", and Geolog's random-kernel DYNCLUST branch is unseeded in its own
documentation. SandiBumi is already seeded end to end. That is necessary and it is not
sufficient — a seed makes a run repeatable *on the same machine with the same inputs*, and §3
shows that SandiBumi does not currently record enough to establish that those inputs were the
same. The gap between "seeded" and "reproducible" is where `SB-CORE-011` lives, and closing it is
the largest single opportunity in this domain.

### Seams

**Seam — `SHR` (saturation-height and rock typing).** This is the widest seam in the chapter and
the one most likely to be discovered late. `SHR` owns the *petrophysical* definitions of a rock
type: Amaefule `RQI`/`FZI`, Corbett-Potter GHE bands, Lucia rock-fabric numbers, Pittman
pore-throat radii, and the flow-unit concept itself. This chapter owns the *partitioning
machinery* those definitions are fed into. Concretely, the tree today contains **three separate
implementations of a within-cluster-sum-of-squares partition** — `hfu.rs:103` (`ward_partition`,
exact O(K·m²) dynamic program over sorted `FZI`), `lorenz.rs:152` (`segment_dp`, the same exact DP
but over the *depth-ordered* profile so segments are true depth intervals), and
`ml.rs:170` (`AgglomerativeClustering(linkage="ward")` via scikit-learn). The first two are
deliberately different algorithms for deliberately different questions and that is correct; what
is not correct is that they are three unrelated code paths under one method name. `SB-MLA-025`
states the obligation and it is stated **here**, because it is a "one name, one equation"
obligation (`SB-CORE-006`) rather than a rock-typing one. The `RQI_C = 0.0314` and
`PERM_C = 1014.24` constants at `hfu.rs:22` and `hfu.rs:24` are `SHR`'s to source and are cited in
§5 for cross-reference only, marked as belonging to that chapter.

**Seam — `PLT` (plotting, display, interactivity).** Every diagnostic this chapter computes has a
rendering that `PLT` owns: the dendrogram, the SOM node map and its U-matrix, the PCA
correlation circle, the silhouette plot, the fall-off curve across restarts, the confusion matrix
as a heat map, and the crossplot coloured by cluster id. The split is that **this chapter owns the
numbers and their axis semantics; `PLT` owns their appearance.** One requirement sits on the line
and is stated here deliberately: `SB-MLA-051` (a confusion matrix must carry both normalisations,
each labelled with the axis it was normalised on). That looks like a display rule and is not — the
dossier §2.8.d finds Geolog's "recognition rates" and Techlog's "row frequency" are normalised on
**opposite axes**, so a bare "72 %" is ambiguous *across vendors* and the ambiguity is in the
number, not in its rendering.

**Seam — `DBM` (database and project data model).** Most of Group A in §4 is a storage contract.
The `ml_models` table at `db.rs:675` and the ordered-feature contract stated in its schema comment
at `db.rs:668` are the existing surface; the requirements that extend it (`SB-MLA-002` through
`SB-MLA-007`, `SB-MLA-011`, `SB-MLA-012`) are specified here because the *obligation* arises from
this domain's evidence, and `DBM` owns the schema realisation, the migration, and the referential
rule that `SB-MLA-007` depends on. The log-set versioning machinery (`create_log_set` +
`write_computed_curves_versioned`, used at `ml.rs:676`) is likewise `DBM`'s; this chapter
specifies only what an ML run must put into it.

**Seam — `DIO` (data import, export, formats).** Three obligations cross here. First, the null
discipline: `−999.25` written with an explicit `NULL.` line, and `−999` / `−9999` / `−99` screened
on read as suspected undeclared nulls — this chapter carries it because IP's documented
"intermediate nulls become numeric `−999`" behaviour (dossier §3.9) is a *machine-learning input*
defect, and because Techlog's `Min and Max threshold` default of `−9999` collides with Techlog's
own `MissingValue` sentinel (ML-16), which is the strongest argument in the corpus for a separate
null flag. Second, model-artifact portability: a joblib blob is a pickled scikit-learn object and
is only loadable by a compatible library version, which makes it a format question as much as a
storage one (`SB-MLA-012`). Third, an exported curve that came from a model must carry its
provenance out of the product (`SB-MLA-010`) — `DIO` owns the LAS/DLIS realisation of that block.

**Seam — `ENV` (environmental corrections and log QC).** ML consumes QC output rather than
producing it. The bad-hole flag that excludes a sample from training is an `ENV` product; this
chapter owns only the *convention* by which it excludes (`MASK` semantics: a value of exactly
`1.0` excludes, implemented at `ml.rs:1348` and mirrored in the training path) and the obligation
to report how many samples the mask removed (`SB-MLA-004`). Techlog's HRA inputs table names the
same optional bad-hole flag, so the convention is corroborated rather than invented.

**Seam — `CUT` (cutoffs, summation, Monte Carlo).** A predicted curve is a normal curve
downstream: an ML permeability enters a `k` cut-off, an ML facies code scopes a cut-off set. The
obligation this creates is one-directional and belongs here — `SB-MLA-009`, that blind-well
metrics travel with the curve — because a net-pay number computed from a predicted `k` whose
blind-well `R²` was 0.31 is a different claim from one computed from a measured `k`, and `CUT` has
no way to know which it received unless this chapter attaches it.

**Seam — `MIN` (multi-mineral solver), `POR`, `SAT`, `CLY`.** These are the chapters whose
deterministic answers ML is used to *substitute for* when a log is missing or bad. The house rule
recorded in project-kb and restated in the dossier §5.3 field-acceptance list is that synthetic-log
substitution is **conditional** — override only where the bad-hole flag is set *and* the synthetic
moves in the physically correct direction. That rule is owned by the chapter that owns the curve
being substituted; this chapter owns the estimator that produces the candidate and the requirement
that the candidate is never written over a measured sample silently. Where a mineral or
lithology *class* is predicted rather than solved, `MIN` owns the endpoint set and this chapter
owns the classifier.

**Seam — `INS` (install, deployment, packaging blockers).** This chapter's entire supervised and
scikit-learn-backed surface runs in an out-of-process Python discovered at run time
(`python_engine.rs:177`, `find_python`), gated on `numpy` and additionally on `scikit-learn` and
`joblib` for anything in `ml.rs`. That is a deployment fact with a product consequence: a customer
machine with no suitable interpreter loses regression, classification, PCA, t-SNE, the
leaderboard and model persistence, while keeping the native `facies.rs`, `hfu.rs` and `lorenz.rs`
paths. `INS` owns packaging and the decision about whether an interpreter ships; this chapter owns
the requirement that the loss is **named and actionable rather than a missing menu item**
(`SB-MLA-061`).

**Not in scope.** Deconvolution and thin-bed resolution enhancement (`TBD`), NMR `T2` inversion
(`NMR`) and rock-physics regression fits (`RPH`) all use numerical optimisation, and none of them
is in this chapter — the test is whether the method learns its parameters from a *training set of
other wells*. Geolog's SandPit `S1`/`S2` transform is out of domain by the dossier's own §1.1 note
and is dispositioned accordingly in §8.

---

## 2. What the incumbents do — the requirement-bearing findings

Fourteen findings. Each generates at least one requirement in §4. Findings from the dossier that
generate no obligation — the capacity-limit drift table, the Techlog editorial-guidance
divergence, the SandPit transform — are dispositioned in §8, not restated here.

### 2.1 The Cuddy combination rule is a reciprocal sum, and the obvious wrong guess picks a different bin

**Tier T2, single raster, no external arbiter. IP 2025 only.**

IP's fuzzy curve/facies predictor combines per-curve bin probabilities as a **parallel sum**,
`P(b) = 1 / Σ_j (1 / P_j(b))` (dossier §5.1 F2, from `statisticalcurveprediction.htm`,
`[img-read: embim633.png]`). A reimplementer's default assumption is the naive-Bayes **product**
`Π_j P_j`. These are not monotone transforms of one another. The dossier's counter-example (§3.1),
three input curves and two competing bins:

| Bin | per-curve `P` | reciprocal sum | product |
|---|---|---|---|
| A | 0.99, 0.99, **0.05** | 1/22.0202 = **0.04541** | **0.049005** |
| B | 0.22, 0.22, 0.22 | 1/13.63636 = **0.07333** | 0.010648 |

**IP's rule selects bin B; the product rule selects bin A.** The reciprocal sum is a soft-minimum
— one bad curve vetoes a bin — while the product is dominated by the geometric mean. The
consequence is a different facies code, or a permeability drawn from a different bin, at that
depth, with **both results looking entirely plausible on a log plot**. Geolog implements the same
Cuddy method (`PT15_Facimage/fuzzy_hc.2.08.html`, T3) and is a second implementation but not a
second *source for this equation*.

Two things about the evidence must travel with any implementation. The equation is a **single T2
raster**, which is the weakest support any load-bearing equation in this chapter has; and IP does
not state whether the reciprocal sum is applied to the raw `√n_b`-weighted `P(C_b)` or to a
per-curve normalised probability (ML-11). Since `P` carries `√counts`, the two differ **whenever
bin populations differ**, which under variable-size binning is always. The chapter adopts the rule
and escalates the sub-question rather than choosing.

→ `SB-MLA-037`, `SB-MLA-040`. Escalation E-4.

### 2.2 IP's printed SOM decay law is provably degenerate — transcribing it faithfully ships a SOM that is not a SOM

**Tier T2, single source, no external arbiter.**

`som.htm` prose gives `λ = t / log σ₀` with `t` the *current* iteration. Substituted into the
printed `L_t = L₀ exp(−t/λ)`:

```
L_t = L₀ · exp( −t / (t / log σ₀) ) = L₀ · exp( −log σ₀ )
```

— **independent of `t`**. The learning rate never decays. The same substitution into
`σ_t = σ₀ exp(−t/λ)` gives `σ_t = σ₀ · exp(−log σ₀)`, which for a natural log is **exactly 1.0
from the first iteration onward**: the neighbourhood collapses to a single node before any global
ordering can occur, and the map degenerates into a nearest-prototype vector quantiser with no
topology preservation. The printed pair cannot be what the product does.

There is no arbiter. Techlog ships **no** SOM training math anywhere in its documentation tree —
a verified negative result (ML-4): a content grep of `concept/`, `task/` and `reference/` for
`learning rate` / `neighbo[u]rhood` / `Kohonen` returns only two Ipsom pages that say "based on the
neutral [*sic*] network technology (The Kohonen algorithm)" with no citation and no equations, and
`modulesDescription\Ipsom\index.html` and `…\Kmod\index.html` are 336- and 298-byte marketing
stubs. Geolog parameterises by `Iterations` + `Shakings` and never exposes a decay law.

The obligation is not to guess IP's intent. It is to **refuse the degenerate parameterisation
loudly** and to carry SandiBumi's own total-iteration form with a source string that says so.

→ `SB-MLA-041`. Escalation E-1.

### 2.3 Three vendors, three unrelated cluster-count criteria — and they are complementary, not competing

**Tier T2 (IP) + T3 (Techlog) + T3 (Geolog).**

- **Cluster Randomness Index** (IP, `cluster_analysis.htm` and `som.htm`, printed as **ASCII, not
  a raster**, identically in both places, unchanged from IP 2018): `RI = Av_thickness /
  Random_thickness` where `Av_thickness = n_depth_levels / n_cluster_layers` and
  `Random_thickness = Σ_i p_i/(1−p_i)`. It measures **vertical bed coherence**; 1 is totally
  random, higher is less random, and the user picks the peaks. Its blind spot is that it ignores
  geometric separation entirely.
- **Silhouette** (Techlog HRA): per-point own-cluster versus nearest-other-cluster distance. Blind
  spot: ignores depth ordering completely.
- **Fall-off** (Techlog HRA): cumulative Euclidean distance across the 50 restarts, sorted, with a
  **±10 % rule** — a solution found about 10 % of the time is the happy medium. The vendor states
  its own limit: it "necessarily always decreases with increasing number of classes", so it is a
  *convergence* diagnostic, not a `K` diagnostic.
- **MRGC auto-optimum** (Geolog): "automatically determines the optimal number of clusters, yet
  allows the geologist to control the level of detail". Internals not held (ML-6).

The two that matter most are the two that measure **orthogonal** things: silhouette is a geometric
criterion computed with no knowledge that the samples are ordered in depth, and the randomness
index is a stratigraphic criterion computed with no knowledge of where the clusters sit in feature
space. A `K` that is good on both is a genuinely different claim from a `K` that is good on either.
**No general-purpose ML library provides the randomness index** — it is not in scikit-learn, and
implementing it is four lines of arithmetic from a printed ASCII formula that is not a lookup
table and is not vendor data.

→ `SB-MLA-043`, `SB-MLA-044`, `SB-MLA-045`.

### 2.4 The widest silent divergence in the domain is normalisation, and one of its three traps changes units without changing labels

**Tier T2 (IP) + T3 (Techlog, Geolog).**

Run the *same* GR/RHOB/NPHI/RT clustering job in IP and in Techlog HRA at default settings and
you get different clusters, because **Techlog automatically `log10`s every log-scale family**
(resistivity included) and **IP does not**. Neither answer is wrong; they answer different
questions. Techlog announces the transform in its Output window; IP announces nothing.

The normalisation choice diverges just as widely. Geolog Facimage offers **Data Range (default) /
Plot Limits / Standard Deviation / Histogram-percentile** with **Euclidean (default) / Variance /
Mahalanobis** metrics; IP forces a **z-score** and offers no alternative; Techlog HRA normalises
into PCA space with a `PCA Variance` cut-off defaulting to 0.95. The consequential one is
Geolog's **Plot Limits**, which ties the feature space to the display scale the analyst chose —
stable across wells — against **Data Range**, which ties it to the training data and is not.
Because IP's z-score is per-analysis, **adding one well to the model-build set silently rescales
the entire feature space and moves every cluster boundary in the wells that were already there.**
Geolog's Plot Limits option is the correct answer to that problem and IP does not have it.

The third trap is the worst and it is a **units** defect. IP 2025 states that the `log10` flag
"changes the reported statistics, not just the internals — reported minima/maxima/means become
logarithmic values" (`statisticalcurveprediction.htm`; visible as a negative `PERMCORE` mean in
`[img-read: _flclip0007.png]`, T2). A cluster-statistics table reading `PERMCORE mean = −0.4` is
not an error state: it is `10^−0.4 = 0.398 mD` printed in log units under a header that says mD.
It renders, it prints, it reaches a client deck, and **the only reader who can catch it is one who
already knows the flag was set** — because the neighbouring rows read `−0.4`, `1.2`, `2.8` and the
eye takes them for a plausible spread. Traps 1 and 2 change *which* clusters you get, and a facies
code carries no units to be wrong about. Trap 3 changes the units of a reported number while
leaving its label alone.

→ `SB-MLA-032`, `SB-MLA-033`, `SB-MLA-034`, `SB-MLA-035`.

### 2.5 Three tools ship one word "fuzzy" over two different algorithm families

**Tier T2 (IP) + T3 (Geolog, Techlog).**

IP and Geolog both implement **Cuddy** — a binned, per-curve probability method. Techlog Ipsom's
"fuzzy" is a **fuzzy c-means** node classifier, a different family with a different parameter
(`QQ`) and a different output. Geolog even scopes the word differently: its fuzzy modules are a
*sibling* of the Facimage clustering suite (`Petrophysics | Facimage | Fuzzy Logic`), while
Techlog's fuzzy is an *indexation method inside* Ipsom. A user migrating a Techlog Ipsom fuzzy
model and a Geolog `fuzzy6_*` model into one product gets two incomparable things under one menu
label.

This is the domain's cleanest instance of `SB-CORE-006`. The name must disambiguate **at the call
site, not at the menu** — `fuzzy.cuddy` and `fuzzy.cmeans` are two methods, and neither is
addressable as "fuzzy".

→ `SB-MLA-036`, and Refusal R-3 (Techlog's printed c-means is quarantined, not implemented).

### 2.6 Techlog's printed fuzzy c-means is unusable as printed, and the dossier's own severity split is the finding

**Tier T3, three equation images read at 1×.**

`concept/geology-fuzzy-classification-method.html` prints a barycenter with **no normalising
denominator** (ML-1), a membership with **no outer reciprocal and an inverted ratio** (ML-2), and
prose whose stated direction for `QQ` **contradicts** the printed exponent `1/(QQ−1)` (ML-3), with
**no default for `QQ` stated anywhere**. The dossier records the transcription as `F18` and marks
it **DO NOT IMPLEMENT**.

The requirement-bearing part is the severity split the critique disposition forced (§5.1, added
2026-08-06). ML-2 is **recoverable**: the printed form factors to `ecart_kn^(−1/(QQ−1))` times a
`k`-independent constant, so its `argmax` already selects the nearest barycenter and row-normalising
recovers a proper membership exactly, verified numerically to 1e−9 over four cases. ML-1 is **not
recoverable from the page**: a barycenter printed as an unnormalised sum scales with `n` and lands
outside the data cloud, corrupting every `ecart` fed to the membership — so the recoverable
equation is being fed a broken input. **The quarantine therefore stands on ML-1 and ML-3, not on
ML-2.** That distinction is what makes the refusal auditable rather than a blanket avoidance, and
it is what an implementer would need if the method is ever built: ML-1 must close first, because
no amount of fixing the membership repairs a barycenter in the wrong place.

The pattern recurs. Geolog's KNN log prediction has the **same missing denominator** in the same
idiom — `facimage_05_using_hc.5.05.html` says the prediction is "the **summation** of the weighted
associated log values" (stated twice, for KNN and for Barycenter) while
`facimage_06_reference_hc.6.8.html` says "an exponential distance weighted **average**" (ML-12).
The two agree if and only if `Σ w_i = 1`, which is stated nowhere. Two vendors, two continents,
the same defect: **"summation of the weighted values" is a vendor idiom for "weighted average" and
it is unsafe to transcribe literally.** The failure it produces is concrete — a porosity that
doubles when you ask for one more neighbour.

→ `SB-MLA-049`; Refusals R-3, R-4. Escalations E-2, E-3.

### 2.7 Geolog's exponential distance weight is an un-sourced function inside an otherwise adopted equation

**Tier T3.**

Geolog says only "an exponential distance weighting of the K nearest neighbors" and calls it "the
main prediction". **No base, no length scale, no normalisation constant.** Both `w = exp(−d/h)`
and `w = exp(−d²/h²)` are consistent with the printed words and give materially different
predictions, and `h` is exposed nowhere in the help set. The dossier is explicit that this "must
not be presented as Geolog's".

This is a general obligation with a specific trigger: an adopted equation form may contain a
sub-function that the vendor never printed, and shipping it as though the vendor specified it is a
provenance failure of exactly the kind `SB-CORE-004` forbids. The weight function must be
SandiBumi's own, named as such, with `h` a first-class parameter, and the deviation declared.

→ `SB-MLA-049`, `SB-MLA-050`. Escalation E-2.

### 2.8 All three ship a contingency table and two of them normalise on opposite axes

**Tier T2 (IP) + T3 (Techlog Ancor, Geolog Facimage Comparison).**

Geolog's Facimage Comparison tab reports "recognition rates" normalised **by column**; Techlog's
Ancor reports "row frequency" normalised **by row**. A bare "72 %" therefore means two different
things depending on which product produced it, and there is no way to tell from the number. The
quantified consequence is direct: on a deliberately non-square, unbalanced 3-reference × 4-model
table, the row-% and column-% values of the *same cell* differ by an amount bounded only by the
class imbalance — for a reference class holding 10 % of the samples mapped onto a model class
holding 60 %, the same cell reads as a high recognition rate on one axis and a low one on the
other.

Geolog's Comparison tab also does something more than tabulate: it compares a cluster model
against a **supervising lithology log** and then writes the most probable lithologies back into
the model (`Assign Facies to Model`, "Facies are automatically merged and renamed as required").
That is supervision entering **at the labelling step, not at the clustering step**, and it is the
best-supported reading of the apparent contradiction in ML-7 — Geolog's statement that "Facimage
offers only Unsupervised Classifications" is literally true of the *clusterer* while supervision is
available downstream. It matches the delivered-work precedent of a re-clustering pass using the
first output as a supervised input. **Presented as the better-supported reading, not adjudicated.**

→ `SB-MLA-051`, `SB-MLA-052`.

### 2.9 Every vendor loses on determinism, and each says so in its own documentation

**Tier T2 (IP) + T3 (Techlog, Geolog).**

- **IP**: SOM and neural training documented as irreproducible run to run; **no seed control
  anywhere**; the formula language's `RANDOM` has no seed either.
- **Techlog**: "For the same data, K.mod does not display the same results twice." HRA
  acknowledges "the pseudo-random nature of the initial seeding". The only mitigation offered is
  `Duplicate networks`, which copies an existing net rather than reproducing training.
- **Geolog**: DYNCLUST's random-kernel branch is unseeded in the documentation; SOM weight
  initialisation is not stated.

The seeding stories diverge just as widely, and the divergence is a real cross-edition
discrepancy rather than a documentation artefact. **IP 2018** documents PCA-based `Seed Clusters`
as functional, including the failure advice "try re-running the clustering **or changing the
default seed points**". **IP 2025 states twice that seed values are ignored** (ML-5). Either the
behaviour regressed or the 2018 documentation was already wrong; both readings are presented and
neither is adopted. Techlog HRA seeds from a preliminary clustering of a random **10 %** subset and
keeps the best of **50** runs by lowest cumulative Euclidean distance. Geolog DYNCLUST seeds with
`NBCR` random kernels **plus** `NBCM` inertial-momentum (farthest-point) kernels.

The stakes are quantified in the dossier at realistic scale: on a five-well pooled set of roughly
40,000 complete GR/RHOB/NPHI samples at `K = 15`, an unseeded single-restart Lloyd run can land in
a local optimum materially worse than the 50-restart best and — worse for a deliverable —
**produces different cluster ids on every re-run, so the facies track in a client report cannot be
reproduced.** Techlog's fall-off diagnostic exists precisely because the vendor knows this.

For a commercial deliverable this is the single largest differentiator in the domain, and
SandiBumi already holds most of it (§3). The requirement is to hold **all** of it — a seed makes
a run repeatable only if the inputs to that run are pinned too.

→ `SB-MLA-001` … `SB-MLA-008`.

### 2.10 IP fails silently in three named places, and the dossier records the vendor's own words for each

**Tier T2.**

- **Cross-validation is silently disabled under zonal averaging.** The user sets a
  cross-validation percentage, selects zonal averaging, and the validation does not run. No
  message.
- **The `Seed Clusters` button silently does nothing** in IP 2025 (ML-5).
- **Per-well PCA gives "no indication that the analysis has run".**

Against this, Techlog's auto-`log10` **is** announced in the Output window — the same class of
hidden behaviour, handled correctly by one vendor and not the other, in the same domain. That
asymmetry is the argument: announcing is not expensive, and the vendor that does it proves the
one that does not had a choice.

The rule this generates is narrower than "fail loud" in general: **SandiBumi refuses or announces,
never both-and-quiet.** A combination that cannot be honoured is a refusal with a named reason,
not a silently dropped option.

→ `SB-MLA-034`, and `SB-MLA-013` … `SB-MLA-021` inward (§3 shows SandiBumi has its own instances).

### 2.11 Geolog ships almost no stated defaults, and that absence is itself the finding

**Tier T3.**

Geolog Facimage states **no default** for: MRGC `Minimum`/`Maximum Number of Electrofacies`,
`Number of Optimal Models`, `Initial Neurons for CFSOM`; DYNCLUST `NBCR`, `NBCM`, `Iterations`,
`Minimum Interclass Stability Variation`; SOM `X`/`Y Neurons`, `Shakings`, `Iterations`; AHC
`Number of Classes`; ANN `Maximum Number of Training Epochs`, `Neurons in Hidden Layer`; STM
`Maximum K-Nearest Neighbors`, `Maximum K'-Strongest Membership` and both a-priori reassignment
rates; fuzzy `QQ` (ML-8). That is roughly eighteen parameter gaps behind one live-session
acquisition of about an hour.

IP's position is different and worse for a reimplementer: values *are* visible, but only in
**documentation screenshots**, and a screenshot is not a factory default (G-9.2). The dossier is
strict about this and this chapter inherits the strictness — SOM `Map Width 20`, spherical `642`,
`60000` iterations, `L₀ = 0.1`, `Cluster K = 15`, fuzzy `10 bins`, `Er = 25` all carry the source
string "IP2025 documentation screenshot — NOT verified as a factory default" **or they are not
used**.

The obligation is `SB-CORE-004` applied without exception: nineteen of §5's rows ship
`ABSENT — ships with no default` and that is the correct outcome, not a research failure.

→ §5 in full; `SB-MLA-031`. Escalation E-5.

### 2.12 Techlog contradicts itself on the outlier-tolerance denominator, and the resolution is dimensional

**Tier T3, four pages, all read at source.**

Techlog's quality-log outlier rule is `|x − μ| > a·s`, with `a = 2` a **stated vendor default** on
three pages across two modules (`petrophysics-kmod-properties.html`, `petrophysics-quality-log-outliers.html`,
`geology-outliers-quality-log-appearance.html`), each pairing it with "gives 5 % of outliers".
Two of the four pages name `s` as the **standard deviation**; one (`geology-quality-log-outliers.html`)
names it the **variance**; the fourth does not say (ML-9, downgraded from a 1–1 tie to 2 : 1 by the
critique disposition).

The resolution is not a vote. Under the SD reading, `a = 2` gives ±2σ ≈ 95.45 %, so ≈ 4.6 % outside
— "in general equal to 5 %", exactly as printed. Under the variance reading the relation is not
even **dimensionally stable**: `a·s` would carry curve units *squared* and be compared against a
quantity in curve units. **SD is the only reading that is both consistent with the vendor's own
5 % pairing and dimensionally coherent.** Presented as the strongly corroborated reading, not
adjudicated.

The product consequence is a naming rule, not a value: the field is `tolerance_sd`, **never**
`tolerance`, because a bare "tolerance = 2" is ambiguous across the vendor's own pages and would
be imported wrongly.

→ `SB-MLA-053`.

### 2.13 Three vendor worked examples in this domain fail their own arithmetic

**Tier T3, proven from the pages themselves.**

- **Techlog PCA** (`utility-techstat-principal-components-analysis-pca.html`): the third
  cumulative-information value prints **0.9930097** where the page's own addends give
  **0.9300971** — provably a digit transposition, because only 0.9300971 lets the fourth line
  reach 1. Separately, the concluding sentence claims "axis 1 and axis 2 carry together
  **63.62 + 27.61 = 91.23 %**" against the page's own cumulative figures, which give **83.83 %**,
  and neither 63.62 nor 27.61 appears anywhere else on the page.
- **Techlog MCA**: "axis 1 and 2 carry together **0.350187 %**", where 0.350187 is the cumulative
  *fraction* — i.e. **35.02 %**, a fraction printed with a percent sign.
- **Techlog decision tree** (`utility-techstat-decision-tree.html`): one binary split printed with
  **two different thresholds three lines apart**, `Gamma Ray:> 53.198` and `Gamma Ray:<=53.189`
  (ML-17). A binary split has exactly one threshold; the two values are a digit transposition.

The consequence of the first is concrete: the page's own `Save projections` rule — "select only
92 %, only eigen vectors 1 and 2 are saved" — is satisfied by **no** reading of its own numbers
(ML-13). The example is quarantined.

The requirement-bearing conclusion is not "Techlog has typos". It is that **a vendor worked
example is only usable as a fixture if it reproduces**, and that IP's PCA worked examples do
reproduce — verified independently, twice — which is why they become acceptance tests here and
Techlog's do not.

→ `SB-MLA-047`, `SB-MLA-048`; tests `SB-MLA-T25`, `SB-MLA-T26`.

### 2.14 Two vendor sentinel and naming collisions that a reimplementer would inherit

**Tier T1 + T3.**

**The sentinel collision (ML-16).** Techlog's `Min and Max threshold` **default value is −9999**,
which is also Techlog's own `MissingValue` sentinel (T1, from the shipped Python package). A
user-set threshold of exactly −9999, and a curve legitimately carrying −9999, and "no threshold
set" are **three states that are indistinguishable**. This is the strongest single argument in the
corpus for SandiBumi's separate-null-flag design, and it is a vendor defect proven from two
independent sources rather than inferred.

**The spelling collision (rule 7).** Geolog's Facimage help spells the metric **`Euclidian`**
(`facimage_03_generate_hc.3.6.html`, verbatim) while IP, Techlog and the literature spell it
*Euclidean*. Any importer that matched a Geolog model's metric on the display string would
**silently fall through to a default**. The same class of defect appears inside IP itself, where
linkage method #1 is named `Minimum` on one page and `Minimise` on a sibling (G-6.10), and where
`Cfit` is used for two different quantities (G-6.4).

Both generate the same rule: **enum ids with separate display labels, vendor spellings as input
aliases only, and a mismatch on load is an error rather than a silent remap.**

→ `SB-MLA-036`, `SB-MLA-029`, `SB-MLA-030`, `SB-MLA-057`.

---

## 3. SandiBumi as-built

Written from the source. Every claim below was read at the file and line cited, in this pass, on
2026-08-07.

The domain is implemented across **two disjoint engines** that share no code:

- **A Python sidecar** (`ml.rs`, 2,174 lines) holding three independent embedded programs —
  `ML_RUNNER` (`ml.rs:31`–`:255`, fit + predict), `ML_APPLY_RUNNER` (`ml.rs:259`–`:312`, apply a
  saved model), `ML_EVAL_RUNNER` (`ml.rs:1100`–`:1228`, the algorithm leaderboard) — executed
  through an out-of-process interpreter discovered at run time by `python_engine.rs:177`
  (`find_python`), which requires `numpy` and additionally, for anything in `ml.rs`,
  `scikit-learn` and `joblib`.
- **A native Rust path** with no external dependency at all: `facies.rs` (581 lines, k-means and
  Gaussian-mixture electrofacies as module-framework modules), `hfu.rs` (560 lines, hydraulic flow
  units by exact Ward dynamic programming or histogram antimodes), `lorenz.rs` (654 lines,
  stratigraphic modified Lorenz plot with contiguous-segment Ward), and `facies_tie.rs` (about 300
  lines, the confusion/purity tie-in).

That split is a deliberate and good design — the native path survives a customer machine with no
Python — and it is also the origin of most of this section's `PRESENT-DIVERGENT` findings, because
the same method is implemented on both sides of it without anything asserting the two agree.

### 3.1 Status by capability

| Capability | Status | Evidence |
|---|---|---|
| Supervised regression (RF, GBDT/XGBoost, SVR, MLP, linear/polynomial) | `PRESENT-OK` | `ml.rs:84`–`:129` |
| Supervised classification (SVM, k-NN, RF, GaussianNB, logistic) | `PRESENT-OK` | `ml.rs:131`–`:156` |
| Clustering — Python (k-means, GMM, agglomerative, DBSCAN) | `PRESENT-DIVERGENT` | `ml.rs:158`–`:200` vs `facies.rs:134`, `:192` |
| Clustering — native (k-means, GMM) | `PRESENT-DIVERGENT` | `facies.rs:23`, `:24`, `:134`, `:192` |
| Dimensionality reduction (PCA, t-SNE) | `PARTIAL` | `ml.rs:202`–`:215`; variance ratios only, no loadings |
| PCA correlation circle | `ABSENT` | no `sqrt(lambda)` scaling anywhere in the tree |
| Fuzzy logic (Cuddy) curve/facies prediction | `ABSENT` | no implementation |
| Self-organising map | `ABSENT` | no implementation |
| Cluster Randomness Index | `ABSENT` | no implementation |
| Silhouette — Python path | `PRESENT-DIVERGENT` | `ml.rs:189`–`:197` (subsampled, unlabelled) |
| Silhouette — native path | `ABSENT` | no diagnostic of any kind in `facies.rs` |
| Multi-restart fall-off diagnostic | `ABSENT` | restarts happen (`facies.rs:23`), spread is discarded |
| Model persistence and re-apply | `PRESENT-DIVERGENT` | `ml.rs:229`–`:254`, `db.rs:675`, `ml.rs:733` |
| Ordered-feature contract on apply | `PRESENT-OK` | `ml.rs:294`–`:297`, `db.rs:668` |
| Blind-well leaderboard | `PRESENT-DIVERGENT` | `ml.rs:1093`–`:1097`, `:1130`, `:1132`–`:1169` |
| Confusion / contingency tabulation | `PARTIAL` | `facies_tie.rs:100`, `:130` — one normalisation only |
| Hydraulic flow units (Ward, histogram) | `PRESENT-OK` | `hfu.rs:103`, `:154`, `:206` |
| Stratigraphic modified Lorenz | `PRESENT-OK` | `lorenz.rs:152`, `:228` |
| Seeded determinism | `PRESENT-DIVERGENT` | `ml.rs:64` (42) vs `facies.rs:80` (7) |
| ML provenance into the deliverable | `ABSENT` | `report.rs` holds no ML reference at all |
| ML provenance into export | `ABSENT` | `export.rs` holds no module/provenance reference |

### 3.2 What is genuinely strong, and worth protecting

Four things in this tree are better than what the dossier finds in any incumbent, and the
requirements below are written so as not to regress them.

**The model artifact is designed as an artifact, and the reason is in the source.** `ml.rs:229`–`:247`
dumps the scaler, the model, the feature names, the task and the algorithm into one joblib blob,
with the comment stating the reason plainly: refitting a `StandardScaler` on the apply wells
"would be a different transform, and the predictions would be quietly wrong rather than obviously
broken". Against a corpus where IP does not document its normalisation scheme at all (G-9.5) and
Techlog offers `Duplicate networks` — copying a net rather than reproducing training — this is a
material lead.

**The ordered-feature contract is enforced inside the artifact, not around it.** `ml.rs:294`–`:297`
refuses an apply whose column order differs from the fit, by name:

```
this model was fitted on <have> - refusing to apply it to <want>
```

and `db.rs:668` states the same contract in the schema comment: `feature_curves` is an ordered
JSON array, applying resolves exactly those curves in exactly that order, and a well missing one
**fails by name rather than substituting or reordering**. This is `SB-CORE-003` behaviour in the
one place where a silent substitution would be undetectable.

**Retraining never overwrites.** `db.rs:2602` (`resolve_model_name`) auto-suffixes a colliding
name so a retrain produces a **new** model, with the stated reason that "silently replacing the one
a delivered curve was made with would destroy its provenance". The user is told
(`ml.rs:750`–`:754`). No incumbent in the corpus offers this.

**Refusals in the training path are specific and cause-aware.** `ml.rs:571`–`:575` refuses fewer
than 10 labelled training samples and names the likely cause; `ml.rs:580`–`:587` reports training
wells that contributed nothing, so "a 20-well selection fit on 3 wells" cannot look like a clean
20-well run; `ml.rs:589`–`:597` distinguishes "every row is missing an input" from "every row is
excluded by the mask", because masking is a second independent way to empty the pool.
`hfu.rs:314`–`:326` reports a shortfall when the requested cluster count exceeds the distinct
data available, rather than returning empty clusters — which is precisely the failure IP documents
and answers with "re-run".

### 3.3 `PRESENT-DIVERGENT` — one product, one method name, two different computations

This is the most valuable finding in the section and it has four instances.

**(a) Two k-means implementations that do not compute the same thing.** The native module runs
best-of-`RESTARTS = 8` k-means++ restarts (`facies.rs:23`) with a Lloyd iteration cap of
`MAX_ITERS = 100` (`facies.rs:24`), keeping the lowest inertia (`facies.rs:144`–`:151`). The
Python path runs `KMeans(n_clusters=k, n_init=10, random_state=seed)` (`ml.rs:163`), which is 10
restarts at scikit-learn's default `max_iter=300`. **8 restarts at 100 iterations against 10
restarts at 300 iterations**, on the same data, from the same product, under the same menu word
"electrofacies". On a well-separated synthetic the two agree; on a real pooled multi-well field set
at `K = 15` — the regime the dossier quantifies at ~40,000 samples — restart count and iteration
cap are exactly the two knobs that decide which local optimum you land in. Neither result is
wrong. They are two answers to one question, and nothing in the product tells the interpreter
which engine produced the curve in front of them.

**(b) Three facies mnemonics from three implementations, and the mnemonic does not name the
engine.** `facies.rs:160` writes `FACIES`; `facies.rs:186` writes `FACIES_GMM` plus `FPROB`; the
Python clustering path writes the frontend's `defaultOut` of `FACIES_ML` (`src/ui/mlDialog.ts:113`)
plus a `_PROB` suffix (`ml.rs:200`). A `FACIES` curve and a `FACIES_ML` curve in the same well are
two different computations, and only one of the three names says which engine made it. This is
`FINDINGS` rule 8 ("no bare reused symbol") in SandiBumi's own tree.

**(c) Two seed defaults for one concept.** `ml.rs:64` defaults the seed to **42**; `facies.rs:80`
falls back to **7** and the module spec declares 7 as the parameter default (`facies.rs:41`,
`facies.rs:179`). The frontend always supplies 42 (`src/ui/mlDialog.ts:280`, `:469`, `:633`), so a
UI-driven ML run is seeded 42 and a UI-driven facies module run is seeded 7. Both are deterministic
and both are defensible; **one product with two values for one concept is `SB-CORE-007`.**

**(d) The leaderboard does not evaluate the model the run will fit.** This is the sharpest
instance and it is worth stating precisely, because the leaderboard is the surface an interpreter
trusts to *choose* a method.

`ML_RUNNER` builds its estimators from user-supplied parameters with defaults — for example
`RandomForestRegressor(n_estimators=int(p.get("n_estimators", 200)), max_depth=int(p.get("max_depth", 0)) or None, …)`
at `ml.rs:87`–`:89`, `SVR(C=float(p.get("C", 10.0)), epsilon=float(p.get("epsilon", 0.1)))` at
`ml.rs:105`, and a polynomial branch at `ml.rs:113`–`:119` that wraps `LinearRegression` in
`PolynomialFeatures(deg)` when `degree > 1`.

`ML_EVAL_RUNNER` re-declares every one of these **independently**, in a second `make_model`
function at `ml.rs:1132`–`:1169`, and accepts **no user parameters at all** — `MlEvalRequest`
carries `seed`, `folds` and `standardize` and no parameter map, and the frontend sends only the
algorithm id list (`src/ui/mlDialog.ts:466`). Three consequences follow, all verifiable at the
lines above:

1. **A user who tunes a hyperparameter is ranking a different model from the one they will fit.**
   Set `degree = 3` and the leaderboard still scores plain `LinearRegression` (`ml.rs:1150`–`:1152`),
   because the polynomial branch does not exist in the eval copy.
2. **On a machine without XGBoost the two copies diverge even at defaults.** Both fall back to
   `HistGradientBoosting`, but `ML_RUNNER` sets `max_iter=300, learning_rate=0.1, max_depth=4`
   (`ml.rs:98`–`:101`) while `ML_EVAL_RUNNER` constructs `HistGradientBoostingRegressor(random_state=seed)`
   with **no arguments at all** (`ml.rs:1143`) — scikit-learn defaults, `max_iter=100` and
   `max_depth=None`. The leaderboard ranks a 100-iteration unbounded-depth model and the run then
   fits a 300-iteration depth-4 model. Same name, same seed, different estimator.
3. **Nothing asserts the copies agree.** The test module (`ml.rs:1532`–`:2174`) contains no test
   comparing `make_model` against `ML_RUNNER`'s construction. A future edit to either is free to
   drift.

**(e) The leaderboard leaks the held-out well into its own scaler.** The comment at
`ml.rs:1093`–`:1097` states the design correctly — whole wells are held out via `GroupKFold`
because "the plain random 5-fold in `ML_RUNNER` leaks depth correlation because adjacent samples
from one well land in both folds". The implementation then computes

```python
Xs = StandardScaler().fit_transform(X) if standardize else X          # ml.rs:1130
```

**before** the splitter is built at `ml.rs:1175`–`:1176`. The scaler is fitted on the *entire*
pooled matrix, held-out well included, so every fold's model is trained on features whose centring
and scaling already encode the blind well's mean and standard deviation. The in-line comment at
`ml.rs:1129` justifies a different thing — that per-column scaling commutes with column
subselection, which is true and is not the issue.

The magnitude is data-dependent and is not zero: the leak is exactly one mean and one standard
deviation per feature, contributed by the held-out group. With `nsplits` bounded by the number of
contributing wells (`ml.rs:1173`–`:1174`), a three-well blind-well run gives the held-out well
roughly a **third** of the weight in the transform it is supposed to be blind to. The direction is
always optimistic. This matters more here than in a generic ML setting, because the number the
leaderboard exists to produce is the *blind* score — the honest one — and the delivered-work
precedent the dossier cites as its cautionary fixture is exactly a case where training and blind
disagreed violently (correlation 0.99 training against 0.31–0.70 blind).

**(f) Two cross-validation protocols, two metric names, no statement of which is which.** The run
path reports `r2_cv5` / `accuracy_cv5` from `cross_val_score(model, Xs, y, cv=5, …)` at
`ml.rs:75`–`:81`, gated on `n_train >= 30`. `cv=5` with a bare integer is scikit-learn's
**unshuffled** `KFold` (or `StratifiedKFold` for a classifier) — so the folds are contiguous
blocks of a matrix that was pooled in training-well order, which is neither blind-well nor random
but an unspecified function of how many samples each well contributed. The leaderboard reports a
genuinely blind-well `GroupKFold` score under the field name `score`. **Both appear in the same
dialog, and neither carries its protocol.** As a secondary point, the comment at `ml.rs:1095`
describes `ML_RUNNER`'s CV as "plain random 5-fold"; it is not random, it is unshuffled — so the
code comment mischaracterises the very thing it is contrasting against.

### 3.4 Silent degradation in SandiBumi's own tree

`03_EVIDENCE_BASE.md` §14.3 says the fail-loud claim "applies inward as hard as outward". Three
instances, all in the native path.

**(a) An unclusterable well emits a clean all-NaN facies curve and no error.** `prep_samples`
returns `None` when no supplied slot carries any data (`facies.rs:72`–`:74`) or when the count of
complete samples is fewer than `k` (`facies.rs:95`–`:97`). Both callers then return the
pre-allocated all-NaN output as a **successful** result: `facies.rs:137`–`:139` returns
`{"FACIES": out}` and `facies.rs:196`–`:198` returns `{"FACIES_GMM": out, "FPROB": prob}`. The
module framework has no channel here for "this ran and produced nothing"; the curve is written,
the run reports success, and on a log plot an all-NaN track is indistinguishable from a track that
was simply not computed. This is the exact scenario the dossier's `T-ML-EMPTY-1` fixture pins, and
IP — the vendor — at least prints "One or more of the clusters had zero data points!". The
Python path does not have this defect: `ml.rs:589`–`:597` refuses with a cause. **Two engines, one
menu concept, opposite failure behaviour.**

**(b) The GMM variance floor is applied silently.** `facies.rs:215` sets `VAR_FLOOR = 1e-4` and the
EM loop applies it without recording that it fired. A component whose variance has been floored is
a component that has collapsed onto a handful of points — usually a sign that `K` is too high for
the data — and the posterior it produces (`FPROB`) is correspondingly meaningless while looking
confident. The floor is correct as a numerical guard; its silence is the defect.

**(c) EM convergence versus iteration exhaustion are not distinguished.** The loop tests
`(ll - prev_ll).abs() < 1e-6 * m as f64` (`facies.rs:289`) inside a `MAX_ITERS = 100` bound
(`facies.rs:24`). A run that exhausts 100 iterations without meeting the tolerance returns the same
shape of answer as one that converged in 12, with nothing distinguishing them.

By contrast `hfu.rs` gets this right and is the in-tree model to copy: `hfu.rs:273` computes
`eff_k = requested.min(distinct).max(1)`, `hfu.rs:289`–`:300` remaps to contiguous ids so an empty
gap cannot masquerade as a cluster, and `hfu.rs:314`–`:326` emits a note when the effective count
falls short of the requested one. The `run_hfu_skips_invalid_and_notes_capped_k` test at
`hfu.rs:489` pins it.

### 3.5 The provenance record — where the chapter's thesis meets the code

An ML run writes curves through the standard versioned log-set machinery at `ml.rs:670`–`:677`:

```rust
let spec = crate::equations::LogSetSpec {
    set_name: out_set.clone(),
    module: format!("ml:{}:{}", req.task, req.algorithm),
    params_json: serde_json::to_string(&req.params).unwrap_or_default(),
    inputs_json: serde_json::to_string(&req.feature_curves).unwrap_or_default(),
};
```

and, if a supervised fit was asked to be saved, persists the artifact at `ml.rs:733`–`:748` into
the `ml_models` table declared at `db.rs:675`–`:692` (`model_id`, `name`, `task`, `algorithm`,
`feature_curves`, `target_curve`, `params_json`, `metrics_json`, `trained_on`, `n_train`,
`standardize`, `sklearn_version`, `note`, `model_blob`, `created_at`).

That is more than any incumbent records. It is still not enough to satisfy `SB-CORE-011`, and the
gaps are specific:

1. **The input log set is not recorded on the model.** `MlRequest` carries `input_set` and it is
   used to fetch the training frame, but it is **not** among the arguments passed to
   `insert_ml_model` at `ml.rs:733`–`:748`. The doc comment on the field states its purpose is
   exactly this — that without it "a model trained today and one trained after the next porosity
   re-run are fitted on different rock with nothing in either artifact able to say so". The
   parameter exists for provenance and the provenance record drops it.
2. **The mask curve is not recorded, and neither is how many samples it removed.** `mask_curve`
   determines which rows trained the model. It is absent from both the `LogSetSpec` and the
   `ml_models` row.
3. **The fit path does not record the model it produced.** The `LogSetSpec` at `ml.rs:670` is
   built *before* the model is saved at `ml.rs:711` onward, and carries no `model_id`. The apply
   path does record it — `ml.rs:949` writes `format!("ml:apply:{}", info.name)` — so a curve made
   by *re-applying* a model names it and a curve made by the run that *created* the model does
   not. The asymmetry is backwards: the training run is the one whose configuration is hardest to
   reconstruct.
4. **An effective default is not recorded when it is not supplied.** `seed` defaults to 42 inside
   the Python at `ml.rs:64` and `standardize` to `True` at `ml.rs:67`. `standardize` is captured
   into its own column via `req.params.get("standardize").…unwrap_or(true)` (`ml.rs:744`), but the
   seed is only present in `params_json` if the caller put it there. The shipped dialog always
   does (`src/ui/mlDialog.ts:469`, `:633`), so this is latent rather than live — but the
   Tauri command `run_ml` (`lib.rs:1779`) is a public surface and a workflow or script that omits
   `seed` produces a run whose effective seed is unrecoverable.
5. **The training rows are not identified.** Neither a content hash of the training matrix nor a
   row index set is stored. `trained_on` records well **names** (`ml.rs:720`–`:732`, correctly
   filtered to wells that actually contributed) and `n_train` records a count, which narrows a
   re-run but does not pin it: the same wells at a later log-set version are different rows.
6. **Deleting a model is unconditional.** `db.rs:2740` (`delete_ml_model`) removes the row with no
   check for curves whose provenance cites it, so a delivered `ml:apply:<name>` curve can be
   orphaned — its provenance string pointing at a model that no longer exists.
7. **The environment record is one field wide.** `sklearn_version` is captured; the Python
   version, `numpy`, `joblib` and `xgboost` versions are not. Since the artifact is a pickled
   scikit-learn object, an unloadable blob is a live risk and the record cannot say why.

**And none of it reaches the deliverable.** `report.rs` — 1,442 lines, the client PDF generator
whose structure is cover page → methodology (a parameter/method/remarks table) → per-zone
parameter table → pay summary → composite log pages — contains **no reference to `ml`, `facies`,
`cluster` or a model of any kind.** `export.rs` (270 lines) contains no reference to a module,
provenance, log set or parameter record. So a predicted permeability can be computed with full
in-app lineage, exported to LAS, and printed into a client report as a curve with **no indication
that a model produced it, which model, or what its blind-well score was**. That is the specific
form `SB-CORE-010`'s absence takes in this domain, and it is the gap between "SandiBumi is more
reproducible than the incumbents" — true today — and "SandiBumi's numbers are defensible in a
deliverable", which is the claim `01_PRODUCT.md` §3.1 actually needs.

### 3.6 Tests: two of the highest-value contracts are pinned by ignored tests

`ml.rs` carries a substantial test module (`ml.rs:1532`–`:2174`) and most of it is real: the
mask-exclusion tests (`run_ml_mask_excludes_apply_samples`, `run_ml_mask_excludes_training_outlier`),
the no-overwrite test (`a_retrained_model_never_overwrites_the_one_a_delivered_curve_was_made_with`),
`listing_models_never_carries_their_bytes`, `run_ml_eval_mask_collapse_is_reported`, and the
blind-well ranking test all execute.

Two do not. Both are marked `#[ignore]`:

- `a_saved_model_applies_to_an_unseen_well_without_refitting` (`ml.rs:1782`) — the round trip that
  is the entire point of persisting a model.
- `a_model_refuses_a_matrix_whose_columns_are_in_the_wrong_order` (`ml.rs:1832`) — the ordered-feature
  contract, which §3.2 identifies as one of the four things this tree does better than any
  incumbent.

The reason is legitimate: both require a real interpreter with `scikit-learn` and `joblib`, which
a bare `cargo test` cannot assume. The consequence is not: **the two contracts most likely to be
broken by a refactor, and least likely to fail visibly when broken, are the two nothing checks on
the default gate.** Status for both is `PRESENT-UNVERIFIED` in the sense of CONTRACT §3 — the code
exists, the test exists, and neither runs.

The native path is better served: `facies.rs:454`–`:581` holds six pure-Rust tests including
`deterministic_for_fixed_seed` and `labels_are_ordered_by_first_curve`, and `hfu.rs:414`–`:559`
holds eight. Neither suite contains a test asserting that the native k-means and the Python
k-means agree — which, given §3.3(a), is a test that would currently fail.

### 3.7 Diagnostics and downstream surfaces

`facies_tie.rs` builds a confusion matrix from `(reference, predicted)` integer pairs
(`facies_tie.rs:100`), reports per-reference-class **dominant-class purity** and an overall purity
(`facies_tie.rs:114`–`:128`), and adds a genuinely good extra the dossier does not find in any
vendor: an ANOVA-style **variance reduction of `log10` core permeability grouped by the predicted
class** (`facies_tie.rs:72`–`:97`, matched to plugs within `CORE_MATCH_TOL_M = 1.0` m at
`facies_tie.rs:144`). That answers "does this typing explain the permeability" rather than merely
"do the two label sets agree", which is the question a reservoir engineer asks.

Two gaps against §2.8. The result exposes `matrix` as raw counts and `purity` as a row-wise
fraction (`facies_tie.rs:121`–`:126`) — **row-normalised only**, so the column-wise
"recognition rate" that Geolog reports is not available and the axis is not named in the payload.
And `overall_purity` is returned with no threshold: the cited method note says "accept the mapping
if dominant-class purity is above a threshold", and no threshold ships. That absence is correct
under `SB-CORE-004` — no source in the corpus states a value — but it must be *visible* as an
absence rather than left implicit.

`lorenz.rs` is the strongest-documented module in the domain: `AUTO_K_TOL = 0.02` at `lorenz.rs:33`
is a marginal-gain rule with its rationale in the comment ("a new flow-unit boundary must explain
≥ 2 % of the total slope variance to be kept"), `AUTO_K_MAX = 12` at `lorenz.rs:35` matches the
HFU cap, `segment_dp` at `lorenz.rs:152` is an exact O(kmax·m²) contiguous segmentation, and
`segment_dp_matches_known_best_split` (`lorenz.rs:593`) pins it against a hand-computed optimum.
Both constants are SandiBumi's own and are recorded as such in §5.

The Tauri command surface is `run_ml` (`lib.rs:1779`), `apply_ml_model` (`:1800`),
`list_ml_models` (`:1821`), `rename_ml_model` (`:1832`), `delete_ml_model` (`:1838`),
`run_ml_eval` (`:1847`), `run_hfu_cluster` (`:1931`) and `run_facies_confusion` (`:1958`),
registered at `lib.rs:3269`–`:3283`.

---

## 4. Requirements

Sixty-five requirements in seven groups. Group A is the chapter's spine and carries most of its
value; Group B holds SandiBumi to its own cardinal rule; Group C is `SB-CORE-006`/`-007` applied
to a tree that currently violates both; Groups D–G follow the dossier's method evidence, the
Tier-C boundary, and the platform seams.

RFC-2119 verbs are used strictly per CONTRACT §1.4.

### Group A — Provenance and reproducibility of a trained model

#### SB-MLA-001 — Record the effective parameter set, not the supplied one          [P0] [status: PRESENT-OK]

**Requirement.** Every ML run MUST persist the **effective** value of every parameter that
influenced the result, including values that were defaulted rather than supplied by the caller.
A parameter whose value was chosen by a default MUST be recorded together with the fact that it
was defaulted and with the identifier of the default's source.

**Rationale.** `SB-CORE-011` requires a byte-identical re-run; a re-run cannot be constructed from
a record that omits a value that changed the answer. This is not a theoretical gap in this domain:
`seed` is defaulted inside the Python program, and a seed is the single parameter with the largest
effect on a clustering result. Against the corpus this is also where the differentiator lives —
dossier §3.7 finds IP ships no seed control anywhere, Techlog states K.mod "does not display the
same results twice", and Geolog's random-kernel branch is undocumented as to seeding (T2/T3).

**As-built.** `PRESENT-OK` as of 2026-08-07 for the python path. Every parameter read in the
runner now goes through `P(p, key, default)`, which records the value AND whether it was
defaulted AND the identifier of the default's source; `P_used` records the value a request was
clamped to beside the one asked for, so a narrowed t-SNE perplexity is not misstated as the
number the user typed. The record is emitted as `metrics["effective_params"]`, and it is that
record — not `req.params` — which is persisted into the log set's `params_json` and into the saved
model's params column. The runner has to be the author: Rust does not know which of the caller's
keys a given algorithm actually read, nor what was substituted for the ones it did not send. The
one parameter Rust chooses, the blind-split seed, Rust adds to the same record.

> The dialog now shows this back as a collapsible **"Settings this run actually used"** table with
> defaulted rows marked and sorted first — they are the only rows the user has not seen elsewhere.

`facies.rs:80`'s mirror-image case (a non-finite `SEED` falling back to 7 with nothing recorded)
is NOT closed by this: the native modules report through `ModuleOutputs`, which has no parameter
record, and giving them one is its own increment.

**Verified by.** SB-MLA-T01, SB-MLA-T08 — python path closed by
`every_parameter_the_runner_reads_is_recorded_as_supplied_or_defaulted` (no `p.get` survives in
either runner; the recorder cannot recurse) and by the assertions added to
`regression_linear_recovers_line`, which check against a REAL runner that a supplied value reads
as supplied and an unsupplied one reads as defaulted with its source named. Native path open.

#### SB-MLA-002 — A saved model records the input log set it was trained from          [P1] [status: PRESENT-OK]

**Requirement.** A persisted model MUST record the identifier and version of the log set its
training frame was read from. Applying a model whose recorded training log set no longer exists,
or has been superseded, MUST warn by name.

**Rationale.** The `input_set` parameter exists in the request specifically to make a run read
stored values rather than current ones, and its own doc comment states the reason: without it, a
model trained today and one trained after the next porosity re-run "are fitted on different rock
with nothing in either artifact able to say so". A model is a function of its training rock; the
identity of that rock is part of the model's identity.

**As-built.** ~~`ABSENT`~~ → **`PRESENT-OK` (closed 2026-08-07).** Was: `MlRequest::input_set` was
used to fetch the frame but was not among the arguments passed to `insert_ml_model`, and there was
no column for it in the schema.

`ml_models.training_json` now carries a `TrainWellRecord` **per contributing well** — well id, well
name, the rows it gave, and the `set_name` / `set_id` / `set_version` those rows were read from.
Recorded per well rather than once per model because `resolve_input_set` is resolved per well: a run
asking for `FINAL` across a field where three wells have no `FINAL` reads stored values for the rest
and CURRENT values for those three, and a single model-level field would have to state one of those
two and be wrong about the others.

*A well with no matching set is recorded as `None`, and that is a statement rather than a blank.* It
means the rows came from the live store, where they can move under the model without anything
changing name or version — weaker provenance than a frozen set, so it is said in those words
(`describeTrainingSets`) rather than left to read as a missing field. The run itself says so too, in
a note naming those wells, because the moment to notice is before the model is saved.

*The warning is `training_set_drift`, and it distinguishes the two ways a set stops being what it
was:* deleted ("no longer exists") and superseded — a set whose current version has moved past the
recorded one. Both name the well and the set. Capped at four with "and N more", because a message
naming ninety wells is one nobody finishes reading.

*It runs at both moments, from one implementation.* On the apply path it joins the `MlResult` notes,
which is what the requirement's "applying … MUST warn" asks for literally. It also runs in
`ml::model_warnings`, behind the `ml_model_warnings` command the saved-model picker calls — the same
reasoning as SB-MLA-005: by the time an apply run can say anything, the curves are written, and the
moment that changes a decision is the one where somebody is choosing which model to push across a
field. Same function, same sentence, both places.

*Only wells that contributed rows are recorded.* A well in `apply_well_ids` that yielded nothing —
missing curve, everything masked — is not part of the training rock, and listing it would make the
record disagree with the fingerprint of `SB-MLA-003`.

**Verified by.** SB-MLA-T02 —
`ml::tests::a_model_records_the_log_set_its_rows_came_from_and_names_it_when_it_has_moved` (pins the
per-well set record round-tripping through the schema, a deleted set naming the well, a superseded
version naming the well, and an unchanged set staying silent).

#### SB-MLA-003 — A saved model identifies the exact training rows          [P1] [status: PRESENT-OK]

**Requirement.** A persisted model MUST carry a content hash of the training matrix it was fitted
on, computed over the feature values, the target values and the row order. Re-fitting with the
same configuration MUST reproduce the same hash; a differing hash MUST be reported as a different
training set even when the well list and sample count are unchanged.

**Rationale.** `trained_on` plus `n_train` narrows a re-run but does not pin it — the same wells at
a later log-set version are different rows with the same names and possibly the same count. A hash
is the only record that distinguishes "these are the rows" from "these are the wells". This is the
requirement that makes `SB-CORE-011` checkable rather than merely asserted for an ML curve.

**As-built.** `PRESENT-OK` (2026-08-07). `ml.rs::training_fingerprint` hashes the feature names in
order, the feature values, the target values, the per-row well index and the row order; the digest
lands in `ml_models.train_hash` (added via `ALTER … ADD COLUMN IF NOT EXISTS`, so existing projects
converge) and rides on every curve's log-set record beside the model reference — on the apply path
it is COPIED from the model rather than recomputed, since it is a property of the fit and the apply
path has no fit.

*Where it is taken matters as much as what it covers.* After the target transform, because a
log-fitted model was fitted on different numbers and a record that could not tell those apart would
be SB-MLA-035's defect wearing a hash. Before the blind split, because the split is a deterministic
function of these rows plus the recorded seed and mode, so this one value and those two pin the fit
rows exactly — where hashing only the fit side would make an otherwise identical run read as a
different training set the moment somebody changed the blind percentage.

*FNV-1a/64, written out rather than taken as a dependency.* The threat model is an ACCIDENT — two
training sets that differ and are reported as the same — not an adversary constructing a collision,
so a cryptographic digest buys nothing here. `DefaultHasher` is explicitly not stable across Rust
releases, which for a value written into a project file would mean the same rows hashing differently
after a toolchain upgrade.

*Two canonicalisations, both load-bearing.* An f32 NaN has millions of bit patterns and −0.0 is not
0.0's bit pattern, so hashing raw bytes would let numerically identical matrices hash differently —
"nothing changed" reported as "something changed" is how a provenance record becomes noise nobody
reads. Both are pinned.

A NULL hash on a model saved before the column existed is the honest answer for such a model, and
`insert_ml_model` stores NULL rather than `""` for the same reason `curve_unit` does.

**Verified by.** SB-MLA-T03 —
`ml::tests::a_training_fingerprint_is_stable_for_the_same_rows_and_changes_for_one_different_value`
(pure; pins stability, one changed feature value, one changed target value, row order, feature
names, feature ORDER, and both canonicalisations).

#### SB-MLA-004 — A saved model records the exclusion mask and its effect          [P1] [status: PRESENT-OK]

**Requirement.** A persisted model MUST record the mask curve used (or its explicit absence) and
the count of samples the mask removed, per well. A run whose mask removed samples MUST report that
count to the user.

**Rationale.** The mask decides which rows trained the model, so under `SB-MLA-003` it is part of
the model's identity. It is also the parameter most likely to differ between an analyst's run and
a reviewer's re-run, because a bad-hole flag is itself a computed curve owned by `ENV`. Techlog's
HRA inputs table names the same optional bad-hole flag, so the convention is corroborated.

**As-built.** ~~`ABSENT`~~ → **`PRESENT-OK` (closed 2026-08-07).** Was: the mask was applied in both
paths but appeared in neither the `LogSetSpec` nor the `ml_models` row. The leaderboard reported
mask-driven **well** collapse, which was the right instinct at the wrong granularity.

`ml_models.training_json` is a `TrainingRecord`: the run-level `mask_curve` wrapped around a
per-well roster carrying `masked` and `incomplete`. The run reports the total to the user with the
**worst well named** — a mask that removed a fifth of the field evenly and one that emptied a single
well are different situations behind the same total, and only the second is a data problem.

*The curve's NAME is half the requirement and it nearly went missing.* An earlier reading of this
As-built claimed the name was already in `params_json`; it is not — that field holds the estimator's
effective hyperparameters, which is a different question. A record saying only "3 samples excluded"
cannot be re-run, because the next analyst has no way to know whether that was a bad-hole flag, a
coal flag or a hand-drawn interval. The mask is wrapped AROUND the roster rather than repeated on
each well, since it is one decision applied to the whole run and a field copied onto ninety rows is
ninety chances to find two of them disagreeing.

*`mask_curve: null` is written explicitly, and that is the requirement's "or its explicit absence".*
The distinction it reaches for is real: no mask at all, versus a mask that was applied and flagged
nothing. The second reads as a name with every `masked` at zero — and an all-zero bad-hole flag
across a field usually means the flag was never computed, which is worth noticing rather than
reading as clean hole. `describeMaskEffect` says each of the three in its own words.

*`masked` and `incomplete` are counted separately, and this is the whole design of the record.* Both
are rows that did not train the model, so one combined count would satisfy the requirement's letter.
But they call for opposite fixes: `masked` means the flag curve excluded real rock, and the response
is to widen the interval or revisit the flag; `incomplete` means a feature curve was missing or NaN,
and the response is to find that curve. A well that gave nothing at all is the case where confusing
them costs most — its rows are `incomplete = depth.len()`, so the record says "this well has no
RHOB", not "your mask ate a well".

*The counts are per well and the total is derived*, not the other way round, because the per-well
form answers a question the total cannot: which well. `describeMaskEffect` re-derives the percentage
from the roster for the model tooltip, so the run message and the saved-model row cannot disagree.

**Verified by.** SB-MLA-T04 —
`ml::tests::the_mask_effect_is_recorded_per_well_and_is_never_confused_with_a_missing_curve` (drives
one well that is masked AND incomplete, pins both counts separately against a constructed flag, pins
that they sum to every depth so neither can borrow from the other, pins the same well unmasked, and
pins the mask name round-tripping with `"mask_curve":null` written out rather than omitted).

#### SB-MLA-005 — A saved model records the runtime that produced it          [P1] [status: PRESENT-OK]

**Requirement.** A persisted model MUST record the interpreter version and the version of every
library that participated in fitting or serialising it. Loading an artifact under a runtime that
differs in any recorded version MUST warn before the model is applied, naming the differing
component.

**Rationale.** The artifact is a pickled scikit-learn object, so it is loadable only under a
compatible library set. A blob that fails to load, or silently loads with changed behaviour, is
the failure mode this record exists to diagnose. One version field cannot do that.

**As-built.** ~~`PARTIAL`~~ → **`PRESENT-OK` (closed 2026-08-07).** Was: only `sklearn_version` was
captured; the Python version, `numpy`, `joblib` and `xgboost` were not.

`ML_RUNTIME_PY` is ONE probe — interpreter, `numpy`, `scipy`, `sklearn`, `joblib`, `xgboost` —
textually shared by the fitting runner, the apply runner and the standalone `ml::ml_runtime()`. One
definition rather than three, because the entire value of this record is a comparison, and two probes
that named their components differently would report a mismatch between `scikit-learn` and `sklearn`
on a machine where nothing had changed. It lands in `ml_models.runtime_json`; `sklearn_version` stays
as its own column so models saved before this still read (`describeRuntime` falls back to it and
says so).

*The set is not arbitrary.* `joblib` is the SERIALISER — a pickle written by one version and read by
another is the exact failure this record exists to name, and it is the component nobody thinks of as
participating in a fit. `scipy` because scikit-learn's solvers reach into it, `numpy` because the
arrays are its, the interpreter because a pickle protocol is a property of it, and `xgboost` because
when it is installed it *is* the estimator for `gbdt`.

*A missing package is written as an explicit JSON `null`, never omitted and never `""`.* Three states,
and the comparison needs all three: an absent KEY means this build never probed that component, so
nothing can be said; `null` means it was probed and was not installed; a string is the version. The
middle one is what lets the check report the case that matters most here — a model fitted with no
`xgboost`, and therefore on the substituted scikit-learn estimator, now being applied on a machine
that has it. A "compare the versions we both have" check cannot see that step at all, because one
side has no version. An empty string would read as "version unknown", which calls for a different
response again.

*The warning had to move to where the decision is.* The requirement says "before the model is
applied", and an apply run can only report its own runtime once it has already predicted — the reply
header arrives after the prediction, by which time the curves are written. So `ml_runtime()` probes
separately, `OnceLock`-cached like `python_status` (the answer cannot change while the app runs, and
probing per row would spawn a subprocess per model in the list). The apply path keeps its own
comparison as well, since a model can be applied from a chain that never opened the list.

*One implementation, not two.* The picker calls `ml_model_warnings`, which runs `runtime_drift` and
`training_set_drift` in Rust and returns the finished sentences; the frontend renders strings and
compares nothing. A first cut mirrored the comparison in TypeScript and it was wrong to: a warning
worded one way on the row it is picked from and another way in the run result that follows reads as
two different problems. Only models with something to say are returned, so the picker is not handed a
row per model to filter.

*`runtime_drift` is deliberately silent in three cases, and each silence is a decision.* Nothing
recorded — a model saved before this existed — is an absence of evidence, not a mismatch. Identical
values, obviously, `null` against `null` included. And a key MISSING from either side: the model
predates the probe knowing about that component, or this probe did not ask, and neither is evidence
of a change. What is named is a component both sides answered for and that differs —
`sklearn 1.5.0 -> 1.6.0`, `scipy 1.14.0 -> not installed`, `xgboost not installed -> 2.1.0`.

*On the row, the tag appears only when something has moved.* A badge on every model would be a badge
nobody reads, and the one row that matters would be lost in a column of reassurance. It is
`--qc-warn`, not `--qc-alert`: a changed library is a reason to look, not proof the prediction is
wrong.

**Divergence from the requirement, stated.** The requirement's As-built cited the `xgboost`
substitution as evidence. Recording `xgboost` closes the *runtime* half — an xgboost model applied
under a different xgboost is now named. The substitution itself, where an absent `xgboost` yields a
scikit-learn estimator recorded under the requested `algorithm` string, is a distinct defect and
stays open under **SB-MLA-012**, which is where it belongs: it is a lie about the algorithm, not
about the runtime.

**Verified by.** SB-MLA-T05 —
`ml::tests::a_runtime_step_is_named_component_by_component_and_an_unrecorded_one_is_not_a_mismatch`
(pins per-component naming, one note rather than one per component, a matching component staying out
of the message, a removed library, an absence that has BECOME a presence, and all four silences).
SB-MLA-T12 remains open with SB-MLA-012.

#### SB-MLA-006 — A curve produced by a fitted model names that model          [P0] [status: PRESENT-OK]

**Requirement.** Every curve written by an ML run MUST carry the identifier of the model that
produced it, whether the model was fitted by that run or applied from storage. A run that fits and
predicts in one operation MUST record the fitted model's identifier on the curves it wrote, not
only on the model row.

**Rationale.** `SB-CORE-010` requires every computed curve to answer "how was I made?". For an ML
curve the honest answer is a model identifier, because the parameters alone do not determine the
number. The current asymmetry is backwards: the *apply* path — the cheap case, where the model
already exists and is named — records the model, while the *training* path — the expensive case,
whose configuration is hardest to reconstruct — does not.

**As-built.** ~~`PARTIAL`~~ → **`PRESENT-OK` (closed 2026-08-07).** Was: the apply path wrote
`ml:apply:<name>` with the model id in `params_json`, while the fit path wrote
`module = ml:<task>:<algorithm>` and no model reference at all — and it could not have done
otherwise, because the model was not persisted until after the well loop, so its id did not exist
when each log set was created.

`persist_fitted_model`'s block now runs BEFORE the write loop, and each fit-path `LogSetSpec`
carries `{model_id, model_name, algorithm, params}` in `params_json` — the same shape the apply
path already wrote, so one reader answers "which model made this curve?" for both. `module` keeps
its `ml:<task>:<algorithm>` spelling: a curve made by a fit and a curve made by an apply are
different events and the catalog should keep saying which.

**The ordering rule survives the move.** "A storage problem costs the artifact, not the work" still
holds — every failure in that block is a `note`, never a return, and the curves are written either
way. Where nothing was kept, the reference is ABSENT rather than empty: that is the truth about such
a curve, and a null id invites no lookup that has to fail.

**Verified by.** `a_curve_from_a_fitting_run_names_the_model_and_a_run_that_kept_none_names_none`,
which pins both halves — the citation resolves to a real `ml_models` row, and a run that saved
nothing still writes its curves and cites nothing.

**Verified by.** SB-MLA-T06

#### SB-MLA-007 — A model cited by a stored curve cannot be deleted silently          [P1] [status: PRESENT-OK]

**Requirement.** Deleting a persisted model whose identifier appears in the provenance of any
stored curve MUST be refused, naming the wells and curves that cite it. An explicit
force-delete MUST record the deletion in the project history and MUST mark the citing curves as
having an unresolvable model reference.

**Rationale.** A provenance string pointing at a model that no longer exists is worse than no
provenance string: it asserts an audit trail and cannot honour it. The product already understands
this principle in the adjacent case — `db.rs:2602` auto-suffixes rather than overwrite, "because
silently replacing the one a delivered curve was made with would destroy its provenance" — and
deletion is the same hazard by a different route.

**As-built.** `PRESENT-OK` (2026-08-07) — `lib.rs::delete_ml_model` refuses by default, calling
`ml.rs::model_citations` and naming the wells, sets and curves that would be orphaned; `force` is the
caller's explicit second decision, taken after reading that list. `mlDialog.ts` catches the refusal
and re-asks quoting it, then writes the forced deletion into the project history with the refusal
text, so the record names the curves rather than saying a deletion merely happened.

`model_citations` counts only sets that still carry curves, driven off `computed_curves.set_id` the
way `ml_provenance` is, so a superseded version does not protect its model — a guard that fired on
every model would be the one people learn to force past. The id is matched with a `LIKE` on the
recorded JSON because the reference sits at two depths: the ordinary path writes `model_id` at the
top level, the coverage path records one per segment.

**One deliberate divergence in mechanism, not in obligation.** The requirement says a forced delete
must *mark* the citing curves; `ml_provenance` instead *derives* the unresolvable reference at read
time, printing the model name followed by "DELETED from this project". Two reasons. A stamp can be
missed — a project restored from a backup taken before the deletion carries the curve and not the
mark — whereas resolving the id on every read cannot go stale. And `params_json` is the run record,
a statement of what was configured when the run happened; editing it afterwards to describe a later
event is the same category of error this requirement guards against.

**Verified by.** SB-MLA-T07 — `ml.rs::a_model_a_delivered_curve_cites_is_not_deletable_without_a_word`
(refusal names the wells and curves; a model nothing cites, and one whose set carries no curves,
both delete clean) and `ml.rs::a_curve_whose_model_was_deleted_says_so_and_one_whose_model_remains_does_not`
(the deliverable marks the unresolvable reference, and does not mark a live one).

#### SB-MLA-008 — A recorded ML run re-runs to byte-identical curves          [P0] [status: PRESENT-OK]

**Requirement.** Re-executing a stored ML run record on unchanged inputs MUST produce
byte-identical output curves, including cluster identifiers, probability curves and every reported
metric. Where byte-identity cannot be guaranteed for a given algorithm, the product MUST say so
before the run and MUST name the source of the non-determinism.

**Rationale.** This is `SB-CORE-011` in the domain where it is hardest and worth the most.
Dossier §3.7 finds no incumbent can offer it — the stakes are quantified at §3.4: on a five-well
pooled set of roughly 40,000 GR/RHOB/NPHI samples at `K = 15`, an unseeded run "produces different
cluster ids on every re-run, so the facies track in a client report cannot be reproduced". The
escape clause in the second sentence is deliberate: some algorithms are genuinely
platform-sensitive, and saying so is worth more than a guarantee that silently does not hold.

**As-built.** ~~`PARTIAL`~~ → **`PRESENT-OK` (closed 2026-08-07).** Was: seeding was thorough, but
what was missing was not the seed — it was the record of everything else the answer depends on. That
record is now complete (`SB-MLA-001` the effective parameters, `SB-MLA-002` the input log set,
`SB-MLA-003` the training rows, `SB-MLA-004` the mask, `SB-MLA-005` the runtime), and this
requirement is what turns those five into a claim.

*The guarantee is MEASURED, not asserted.*
`the_same_run_twice_produces_byte_identical_curves_for_every_algorithm` runs fifteen configurations —
every algorithm across all four tasks — twice each through the real subprocess and compares the
results. **On the bits, not the values**: a tolerance would hide exactly the drift the test exists to
catch, and `f32::NAN != f32::NAN`, so a run that turned a cluster into noise on the second pass would
slip past a value comparison as "both NaN, both fine". `to_bits` makes a missing sample equal to a
missing sample and unequal to everything else. Metrics are compared too, since the requirement says
"every reported metric" and a moved silhouette is the same instability arriving by another door —
into a report's methodology table.

*Every algorithm has a seed on the record even when nobody typed one*, because the runner reads it
through `P(p, "seed", SEED_DEFAULT)`, which records the value AND that it was defaulted
(`SB-MLA-001`). So the dossier's failure case — an unseeded k-means at `K = 15` returning different
cluster ids every run, and a facies track in a delivered report that cannot be reproduced — cannot
arise here by omission.

*The escape clause, scoped to what the product can observe.* The requirement asks that where
byte-identity cannot be guaranteed, the product says so **before** the run and names the source. The
honest scope of that is SandiBumi's own code, not second-hand claims about which library is
deterministic on which machine — a claim nobody here can check is worth less than no claim. Exactly
one case qualifies today, and it is ours: `gbdt` fits `XGBRegressor` where `xgboost` is installed and
substitutes `HistGradientBoosting` where it is not, recorded as `gbdt` either way. Same request, same
seed, same rows, two estimators depending on the machine. `determinism_note` says so under the
algorithm picker, before Run.

*Which estimator the test proves is itself machine-dependent, and that is the point.* On the
reference machine `xgboost` is absent, so the `gbdt` case that passed is the substitute's
determinism, not XGBoost's. A test asserting determinism for an estimator it never executed would be
precisely the silent guarantee this requirement's second sentence exists to prevent.

*What is deliberately NOT claimed.* Byte-identity is a guarantee **within one runtime**. A re-run
under stepped libraries is not covered, and rather than assert anything about that the product
detects and names it — `runtime_drift` on the model row and in the run result (`SB-MLA-005`). The
same applies to changed rows (`SB-MLA-003`'s fingerprint) and a superseded input set
(`SB-MLA-002`'s `training_set_drift`). Those are cross-run facts, not properties of an algorithm,
and each is named where it belongs.

**Verified by.** SB-MLA-T08 —
`ml::tests::the_same_run_twice_produces_byte_identical_curves_for_every_algorithm` (`#[ignore]`d,
needs real scikit-learn; run 2026-08-07, **15 configurations, all byte-identical**, 96 s). SB-MLA-T01
closed separately with SB-MLA-001.

#### SB-MLA-009 — Blind-well performance travels with the curve          [P1] [status: PRESENT-OK]

**Requirement.** A curve produced by a supervised model MUST carry the model's blind-well
performance metric, the protocol that produced it, and the number of wells held out. Where no
blind-well evaluation was performed, the curve MUST carry that fact explicitly rather than
carrying a training metric in its place.

**Rationale.** A net-pay number computed from a predicted permeability whose blind-well `R²` was
0.31 is a different claim from one computed from a measured permeability, and `CUT` cannot know
which it received unless the curve says. The dossier's cautionary fixture is a delivered project
where a predicted `NPHI` reached a training correlation of 0.99 against a blind-well range of
0.31–0.70 (PKB, T4) — a factor of three between the number an analyst sees by default and the
number that describes the curve's actual predictive power.

**As-built.** `PRESENT-OK` (2026-08-07), on a foundation that did not exist when this was written:
SB-MLA-008's work gave `run_ml` a real blind split of its own, so the run path now produces a
genuine blind score rather than only the leaderboard doing so. What was still missing was the
attachment, and that is what this closes.

`ml.rs::blind_record` builds the statement ONCE, in the fitting run. It goes into the model's
metrics — so the apply path copies it verbatim instead of re-deriving it, and a curve made by
applying a model says exactly what a curve made by the run that fitted it says — and onto every
fit-path curve's log-set record, **including one whose model was not kept**: "how well does this
travel" is a question about the curve, not about whether anybody saved the fit.

*The protocol is part of the claim, not a footnote.* A random-row split scores the model on depths
centimetres from ones it was fitted on, so its number does not answer "will this work on the next
well". The record carries `protocol` and an explicit `answers_new_well`, because those two numbers
get quoted as each other otherwise — and only one of them is the claim a reserves figure needs.

*The second half is the one that matters.* Where no blind test was run, the record carries
`performed: false` and **no value at all**, with the reason in words. A training metric standing in
for a blind one IS the dossier's cautionary fixture — 0.99 training against 0.31–0.70 blind — and
it is worse than a blank because it reads as an answer. The same absence is reported where a split
ran but produced no score, so a half-answer cannot borrow the training number either.

*Visible where the decision is made.* The saved-models list shows a `blind R2 0.31` pill on the row
you pick a model from, graded on the `--qc-*` status tokens (never the brand accent — a client skin
must not be able to re-roll the meaning of "this model does not travel"), with the wells, rows and
protocol in the tooltip. A model that was never blind-tested reads "not blind-tested" in neutral
colour: an absence of evidence must not look like evidence of a problem, nor like an endorsement.
`readBlind` in `mlDialog.ts` is the single reader, so the wording cannot drift as more surfaces
show it — SB-MLA-010's deliverable block is the next consumer.

**Verified by.** SB-MLA-T09 —
`ml::tests::a_curve_carries_the_blind_score_or_says_there_was_none_and_never_a_training_one`, which
puts a flattering `r2_train: 0.99` in the same object as `r2_blind: 0.31` and pins that the record
takes the blind one; and pins the no-split, unscored-split and classification cases from the other
side.

#### SB-MLA-010 — The deliverable carries the ML provenance block          [P1] [status: PRESENT-OK]

**Requirement.** Where a report or export includes a curve produced by an ML model, the output
MUST include a provenance block naming the model, its algorithm, its feature list in order, its
training well count, its training log set, its blind-well metric, and the run date. A report MUST
NOT present a model-derived curve as though it were measured or deterministically computed.

**Rationale.** This is the point of the whole group and the place `03_EVIDENCE_BASE.md` §14.4
becomes a sellable property rather than an internal discipline: "a parameter that carries the
paper it came from, through the computation, into the deliverable, is a claim no incumbent can
make." Today the lineage stops at the database boundary.

**As-built.** `PRESENT-OK` — `ml::ml_provenance` builds the block and both document renderers print
it: a **Machine-learning provenance** section in `report.rs::report_pages`, immediately after the
methodology table, and its twin in `office.rs::build_report_blocks` for the editable Word document.
Six columns — curve(s) and the quantity predicted, model and algorithm, inputs *in order*, what it
was trained on, the SB-MLA-009 blind sentence, and the log set / run date / SB-MLA-003 training
hash. The requirement's second sentence is printed, not assumed: both documents carry the caveat
that these curves were **predicted, not measured and not deterministically computed**, and that
every number derived from them inherits the stated blind performance.

Three decisions are load-bearing.

*It is driven from `computed_curves.set_id`, not from `log_sets`.* The question a reader brings to a
provenance table is "is the PERM on this page measured?", so the table must describe the run whose
curves are **live**, not every ML run the well has ever seen. A superseded version named beside the
number on the page is worse than no table: it credits a model that did not make it. The query
therefore requires `EXISTS (SELECT 1 FROM computed_curves cc WHERE cc.set_id = ls.set_id)`, and
because `write_computed_curves_versioned` deletes a curve name's rows before appending, a
superseded set drops out by construction rather than by a rule somebody has to maintain.

*Its own section, not rows in the methodology table.* The methodology table describes the METHOD;
this describes a specific fitted artifact. Same algorithm, two different models, two different sets
of rock — and a methodology row cannot say which one made this well's curve. It sits immediately
after, so a reader who has just read "Permeability — por-perm transform" meets "and this well's
PERM was predicted, here is how well it travels" before any number built on it.

*One definition, both renderers.* `ML_PROV_HEADERS`, `ML_PROV_CAVEAT` and `MlProvenanceRow::cells()`
live in `ml.rs` and are consumed by both — the same discipline `office.rs` applies to the pay
summary. The caveat is ASCII deliberately: `composite.rs::pdf_escape` replaces every non-ASCII
character (Helvetica/WinAnsi), so an em dash would degrade in the PDF and survive in the `.docx`,
leaving one study's two documents setting the same legally-weighted sentence differently.

Not yet extended to LAS export (`export.rs`), where the honest realisation is a `~Other` block and
the seam note at §2 assigns that to `DIO`; and not to the workbook or deck, which are statistical
roll-ups rather than the interpretation record.

**Verified by.** SB-MLA-T10 —
`ml::tests::a_deliverable_names_every_model_derived_curve_it_prints_and_no_superseded_one`, which
writes two runs over the same curve name plus a deterministic equation's log set on the same well,
and pins that exactly one row appears, carrying the surviving run's blind score and not the
superseded one's; that the inputs print in fitted order; that the target is named beside the curve;
and, from the other side, that a well with no ML curve produces no block at all rather than an
empty table under a heading implying a model exists.

#### SB-MLA-011 — Training and apply membership are recorded per well          [P1] [status: PRESENT-OK]

**Requirement.** An ML run MUST record, per well, whether that well contributed training samples,
received predictions, or both; and MUST record the sample count in each role. A well that was
selected for training but contributed nothing MUST be recorded as such, not merely warned about at
run time.

**Rationale.** The distinction between a well the model learned from and a well the model was
applied to is the difference between an interpolation and an extrapolation, and it is invisible
downstream. It is also the fact a reviewer asks for first. The run-time warning already exists and
is well-judged; the defect is that it is transient.

**As-built.** `PRESENT-OK` (2026-08-07) — every ML curve's `params_json` now carries `well_role` for the well it was written to, plus `n_trained_wells` and `n_applied_wells`. Three cases, not two: trained-and-applied, applied-only, and **selected for training but contributed no usable rows** — the last kept distinct because the user believed that well was training rock and the record should say the fit disagreed, rather than folding it in with wells nobody chose. `ml_provenance` appends it to the training description so it reaches the PDF, the Word twin and the workbook without changing a table shape four renderers agree on; it belongs there because it qualifies that description — "300 samples from 8 wells" reads very differently once you know this well was not one of them. The distinction is interpolation versus extrapolation, and it was previously visible only as a run-time warning, which is to say for as long as the pane stayed open.

**Verified by.** SB-MLA-T11

#### SB-MLA-012 — Artifact version skew fails loudly, and a substituted algorithm is never silent          [P1] [status: PRESENT-DIVERGENT]

**Requirement.** Loading a model artifact that cannot be deserialised under the current runtime
MUST fail with a message naming the recorded runtime, the current runtime and the differing
component. Where an algorithm is substituted at run time because its library is unavailable, the
substitution MUST be recorded as the algorithm actually used, and MUST NOT be recorded under the
requested algorithm's name.

**Rationale.** `SB-CORE-006` — one name, one equation. An `algorithm` field reading `gbdt` that
sometimes means XGBoost's gradient boosting and sometimes means scikit-learn's histogram gradient
boosting is one name over two methods, and the two have different defaults, different regularisation
and different results.

**As-built.** `PRESENT-DIVERGENT` — `ml.rs:91`–`:102` catches `ImportError` and substitutes
`HistGradientBoostingRegressor`, recording the substitution only in a free-text
`metrics["note"]` string. The stored `algorithm` column still reads the requested id. The
divergence is not cosmetic: the substituted estimator is constructed with
`max_iter = n_estimators (300)`, `learning_rate = 0.1`, `max_depth = 4` here, while the leaderboard's
copy constructs it bare (`ml.rs:1143`) at scikit-learn's `max_iter = 100`, `max_depth = None`.

**Verified by.** SB-MLA-T12, SB-MLA-T27

### Group B — Fail loud inward

#### SB-MLA-013 — An unclusterable well fails; it never emits a clean empty curve          [P0] [status: PRESENT-OK]

**Requirement.** A clustering run that cannot produce a labelling for a well — because no input
curve carries data, or because the count of complete samples is fewer than the requested cluster
count — MUST fail that well with a message naming the cause. It MUST NOT write an all-missing
output curve as a successful result.

**Rationale.** `SB-CORE-002`: a degraded or failed result is never presented as a clean one. An
all-NaN facies track is visually indistinguishable from a track that was never computed, so the
failure is not merely silent — it is disguised as an absence of work. The vendor benchmark is
instructive: IP, whose fail-silent behaviour this corpus catalogues in three other places, **does**
print "One or more of the clusters had zero data points!" for this case (T2).

**As-built.** `PRESENT-OK` as of 2026-08-07. `prep_samples` now returns `Result<Prep, String>` with
two distinct named causes, and `electrofacies`/`gmm_facies` return `Result<ModuleOutputs, String>`,
so `modules.rs` propagates the refusal instead of wrapping an all-NaN vector in `Ok`. On the Python
path a well that yields no rows is refused **before** `create_log_set` runs, so a run that reports
failure does not also version an interpretation — the rule `docs/record_fixes.md` already states for
every other module. The refusal names *which* emptiness it is (`no_rows_reason`): masked out and
never measured call for opposite fixes, and the old wording "no complete samples in this well" said
both.

> **Correction, 2026-08-07.** The as-built above previously read that "the Python path does this
> correctly at `ml.rs:589`–`:597`, distinguishing 'missing an input' from 'excluded by the mask'".
> That was false in two ways, and checked at source before this requirement was closed. Those lines
> are the scatter-back index loop, not a refusal; and the actual per-well outcome was
> `ItemState::Warned` with `error: Some("no complete samples in this well")` set **after**
> `write_computed_curves_versioned` had already written the all-NaN curve and allocated a log-set
> version for it. Both engines had the defect. The whole-run guards at `ml.rs:659`/`:933` masked it
> in testing: they refuse when *no* apply well has data, so the per-well case only surfaces in the
> field-scale run where some wells are good and some are not — which is the only run where it
> matters. This is the second as-built claim in this chapter found to certify behaviour that was
> not there; see the same note under `SB-MLA-050`.

**Verified by.** SB-MLA-T13 — closed by `a_well_with_no_input_data_is_refused_by_name_not_returned_as_a_clean_curve`
and `fewer_complete_samples_than_clusters_is_refused_naming_the_count_and_k` (`facies.rs`, both
engines, the second pinned from both sides at the 4-vs-5 sample boundary), plus
`a_well_with_nothing_to_predict_names_which_emptiness_it_is` and the `#[ignore]`d end-to-end
`an_empty_well_beside_a_good_one_is_refused_and_writes_nothing` (`ml.rs`), which asserts the refused
well leaves no row in `computed_curves`. SB-MLA-T23 remains open against `SB-MLA-023`.

#### SB-MLA-014 — A reduced cluster count is reported, never substituted silently          [P1] [status: PRESENT-OK]

**Requirement.** Where the effective number of clusters differs from the number requested — capped
by the data, collapsed by an empty cluster, or reduced by a merge — the run MUST report the
effective count, the requested count and the reason.

**Rationale.** `SB-CORE-002` again, and a direct answer to the vendor behaviour in §2.9: IP's
documented remedy for an empty cluster is "re-run", which converts a diagnosable data condition
into a lottery. The in-tree model to copy already exists.

**As-built.** `PRESENT-OK` (2026-08-07) — both directions. `k` is clamped to the sample count with the clamp named (`k_clamped`) and recorded through `P_used`, so the effective-parameter record does not misstate it; and a run that came back with fewer clusters than were asked for reports `k_short`. A silent 4 under a request for 12 reads as twelve clusters that happened to merge, which is a different statement about the rock.

**Verified by.** SB-MLA-T14

#### SB-MLA-015 — A floored mixture component is reported          [P1] [status: PRESENT-OK]

**Requirement.** Where a numerical guard alters a fitted quantity — a variance floor, a
regularisation term added to a singular covariance, a clamped weight — the run MUST report that
the guard fired, on which component, and how many times.

**Rationale.** A component whose variance has been floored has collapsed onto a handful of points,
which is the standard signature of a cluster count too high for the data. The posterior it emits
is meaningless while presenting as confident — and that posterior ships as a curve (`FPROB`,
`facies.rs:187`) that an interpreter will read as a membership confidence. The guard is correct;
its silence is the defect.

**As-built.** `PRESENT-OK` (2026-08-07) — a mixture component holding under 1% of the weight is reported as `degenerate_components`. It is not a cluster the rock has; it is the fit saying `k` is higher than the data supports, and counting it makes a six-component answer out of a five-component one.

**Verified by.** SB-MLA-T15

#### SB-MLA-016 — Convergence and iteration exhaustion are distinguished          [P1] [status: PRESENT-OK]

**Requirement.** Any iterative fit MUST report whether it converged to its stated tolerance or
terminated on its iteration cap, together with the iteration count reached and the final
convergence measure.

**Rationale.** A result that exhausted its cap is not wrong, but it is a different claim from one
that converged, and the two are currently indistinguishable in the output. This is the cheapest
requirement in the chapter to satisfy and it converts a whole class of "the numbers moved and
nobody knows why" support questions into a readable line.

**As-built.** `PRESENT-OK` (2026-08-07) — `note_convergence` records `converged`, `n_iter` and `max_iter` for k-means and GMM, and adds a sentence when the cap was hit. A run that stopped because it converged and one that stopped because it ran out of iterations return labels that plot identically; the second is a partial answer presented as a final one, and scikit-learn's own signal for it is a warning nobody sees from a subprocess.

**Verified by.** SB-MLA-T16

#### SB-MLA-017 — A cancelled run leaves no partially populated log set          [P1] [status: PRESENT-OK]

**Requirement.** Cancelling an ML run MUST leave the project in a state where no output log set
contains predictions for some wells and not others without that fact being recorded on the log
set itself. The cancellation MUST be reported per well.

**Rationale.** `SB-CORE-002` and `SB-CORE-036` (honest cancellation). A partially written facies
set is the worst possible artifact: it looks like a completed run over a smaller well selection.

**As-built.** `PRESENT-OK` (2026-08-07) — the per-well half was already right: the write-back loop
checks cancellation before each well and marks the remaining wells `Warned` with `"cancelled"`. The
missing half, the mark on the **log set**, is now `ml.rs::mark_cancelled_sets`. The run keeps the set
ids it actually wrote; if any well was cut, each of those sets gains a `cancelled` object in its
`params_json` recording wells written, wells in scope, and the fact in words — *"the wells missing
this set were cut, not excluded"* — because the object is read by a person deciding whether to
deliver the curve, not only by code deciding whether to draw a badge. The run result carries the same
counts in `metrics.cancelled` and a note.

The stamp **adds to** `params_json`, never rebuilds it: the mark shares that object with the model
reference (`SB-MLA-006`) and the blind record (`SB-MLA-009`), so a stamp that replaced it would erase
the provenance it was written to qualify. A set whose write failed is not stamped — there is nothing
to qualify — and a stamp that fails costs the mark, not the curves, since it runs after they are
stored.

**Written after the fact, deliberately, and the line is worth stating** because `SB-MLA-007` resolves
the opposite way. Editing a run record months later to describe a *separate* event — a model deleted
in another session — would be rewriting history, which is why that case is derived at read time
instead. A cancellation is not a separate event: it is how this run ended, and the run record is not
complete until the run is. Stamping it finishes the record rather than revising it.

**Verified by.** SB-MLA-T17 — `ml.rs::a_log_set_written_before_a_cancel_says_so_and_a_completed_one_stays_silent`
(the mark and its counts; the model reference and blind record survive it; a completed run's set stays
silent).

#### SB-MLA-018 — The non-interruptible phase is declared, not hidden          [P2] [status: PRESENT-OK]

**Requirement.** Where a phase of a run cannot be cancelled, the progress model MUST show it as
such rather than presenting a cancel control that will not take effect until the phase ends.

**Rationale.** `SB-CORE-036`. The honesty here is already in the source and the requirement exists
to prevent its regression, since the natural "improvement" — making the cancel button always
enabled — would be a lie.

**As-built.** `PRESENT-OK` — `ml.rs:601`–`:603` sets an indeterminate phase for the fit and
`ml.rs:636`–`:638` states the constraint explicitly: "the sklearn fit upstream is a blocking child
process and is not interruptible, but the write-back loop is, so a late Cancel at least stops the
remaining wells getting curves they should not."

**Verified by.** SB-MLA-T18 (`CHARACTERIZATION`)

#### SB-MLA-019 — A cross-validation protocol that degraded MUST NOT report a score as if it had not          [P1] [status: PRESENT-OK]

**Requirement.** Where blind-well cross-validation cannot be performed because fewer than two
groups contributed samples, the run MUST refuse to report a cross-validated score. It MUST NOT
substitute a within-well protocol and report the result under the same field.

**Rationale.** `SB-CORE-002`. The substituted protocol is optimistic in a known direction and by an
unbounded amount — a within-well random split on depth-adjacent samples measures interpolation, not
prediction, and the dossier's cautionary fixture puts the gap at 0.99 against 0.31. A warning
attached to a number that is already in the leaderboard's `score` column will be read as a caveat
on a valid score rather than as an invalidation.

**As-built.** `PRESENT-OK` (2026-08-07) — when only one well group is available the run falls back to random `KFold` within that well, and it now says so as DATA rather than only as prose: `cv_degraded` and `<key>_degraded` are set, and `score_protocols[<key>]` states that the model was scored on rock centimetres from rock it was fitted on. Flagged as data because a renderer that can print the score must be able to find the qualification; the degraded number reads HIGH, which is the wrong direction for a caveat to fail in.

**Verified by.** SB-MLA-T19

#### SB-MLA-020 — A metric computed on a subsample says so          [P2] [status: PRESENT-OK]

**Requirement.** Any quality metric computed on a subsample of the data MUST be reported with its
sample count and the fact of subsampling. A subsampled metric MUST NOT be reported under the same
name as the full-population metric.

**Rationale.** `SB-CORE-007` — one definition per constant and transform. Two runs on the same well
can report different silhouettes for reasons that are entirely about the subsample, and nothing in
the output distinguishes that from a real change in cluster quality.

**As-built.** `PRESENT-OK` (2026-08-07) — the silhouette carries `silhouette_basis`, which states either that every clustered sample was scored or that a seeded random `SILHOUETTE_CAP` of N were, in the same object as counts that are not sampled. A previously swallowed exception is now reported as `silhouette_error` rather than leaving the metric silently absent.

**Verified by.** SB-MLA-T20

#### SB-MLA-021 — Density-based noise is a reported class, not a missing value          [P1] [status: PRESENT-OK]

**Requirement.** Where an algorithm assigns samples to a noise or reject class, those samples MUST
be distinguishable in the output from samples that were not evaluated because an input was missing.

**Rationale.** `SB-CORE-002`. "This sample is an outlier the model refuses to classify" and "this
sample had no `RHOB`" are different statements about the rock, and a single missing value in the
output curve conflates them. Geolog's STM reject concept — accept below one confidence, reject
above another, ambiguous between — is the corpus's model for this and is the best extrapolation
guard the dossier finds in any of the three tools.

**As-built.** `PRESENT-OK` (2026-08-07) — the clustering runner writes a rejected sample as
`CLUSTER_REJECT` instead of leaving it missing, so NaN in a class curve now means one thing only:
never evaluated. `noise_pct` is kept and joined by `n_rejected` and `reject_code`.

**The code is negative, and that is the decision.** Cluster ids run `0..K-1` ordered by ascending
first-feature mean, so a reject class appended after them would sit at the shaly end of an ordering
it is not part of — anyone averaging a curve by facies code would read it as the shaliest rock in the
well. A negative sorts below every cluster and belongs to no part of the ramp. The value is emitted
into the runner from one Rust constant by `ml_shared_constants_py`, the same mechanism and the same
argument as the k-means constants (`SB-MLA-023`): a literal written in Python would run and look
right.

**The display half is where this would have shipped wrong.** Both palette lookups fold an index back
into range with `((i % n) + n) % n`, so `-1` would have painted as a real cluster's colour — an
outlier drawn as a legitimate facies on the log view and in the printed deliverable, which is worse
than the gap it replaced. `plotCanvas.ts::faciesColor` and `composite.rs::facies_color` now return a
neutral grey outside the qualitative palette for **any** negative, not only this code, so an
unrecognised class is never painted as rock it is not. `faciesLabel` names it *"Rejected"* rather
than `F-1`, which reads as a facies with a strange id. `looksDiscrete` admits −1 (and only −1), or a
curve carrying a single rejected sample would silently drop back to a continuous colour ramp — the
one presentation that makes class codes meaningless.

Still open, and deliberately: this covers **reject**, not Geolog's three-way accept / ambiguous /
reject band, which needs a confidence threshold this product does not yet ask for.

**Verified by.** SB-MLA-T21 — `ml.rs::a_rejected_sample_is_a_class_of_its_own_and_is_never_coloured_as_a_cluster`
(the code is emitted and written, the old conflating behaviour is gone, and no cluster index 0..23
shares the reject colour).

#### SB-MLA-022 — The ordered-feature refusal is verified on the default test gate          [P1] [status: PRESENT-OK]

**Requirement.** The refusal to apply a model to a feature matrix whose columns differ in name or
order from the fitted set, and the round trip of a saved model applied to an unseen well without
refitting, MUST both be verified by tests that run on the project's default gate.

**Rationale.** §3.2 identifies these as two of the four things this tree does better than any
incumbent, and §3.6 finds that neither is checked by `cargo test`. A contract whose test is
`#[ignore]`d is a contract enforced by good intentions. The requirement is a testing obligation,
not a behaviour change: the behaviour is already correct.

**As-built.** `PRESENT-OK` (2026-08-07) — closed on the DEFAULT gate by taking the structural route rather than the behavioural one. `an_apply_request_cannot_state_a_feature_order_for_the_model_to_refuse` builds `MlApplyRequest` with an **exhaustive struct literal**, so adding a `feature_curves` field stops the suite COMPILING with "missing field" — earlier and stronger than any runtime assertion — and checks that a feature list offered over IPC is ignored rather than honoured. A refusal catches a bad order; having nowhere to express one means it cannot arise from this product at all. The behavioural test remains `#[ignore]`d and legitimately so.

**Verified by.** SB-MLA-T22

### Group C — One name, one method

#### SB-MLA-023 — One k-means, one definition          [P0] [status: PRESENT-OK]

**Requirement.** SandiBumi MUST expose exactly one k-means definition, with one restart count, one
iteration cap, one initialisation rule and one convergence tolerance, whatever engine executes it.
Where two engines exist for platform reasons, a conformance test MUST assert that they produce the
same labelling on a shared fixture, and a divergence MUST fail the build.

**Rationale.** `SB-CORE-006` — one name, one equation. This is the domain's clearest violation and
the vendors are not the source of it: the two implementations are both SandiBumi's. Restart count
and iteration cap are precisely the two knobs that select which local optimum a k-means lands in,
which is why every vendor in §2.9 exposes a restart control and Techlog ships a fall-off diagnostic
to reason about it.

**As-built.** `PRESENT-OK` (closed 2026-08-07) — the definition is now four named constants in
`facies.rs` (`KMEANS_RESTARTS = 10`, `KMEANS_MAX_ITERS = 300`, `KMEANS_TOL = 1e-4`, `SEED_DEFAULT`),
and `ml::ml_shared_constants_py` **emits them into the Python runner preamble** so the scikit-learn
side is configured FROM the same values rather than restating them. `KMeans(...)` reads
`n_init=KMEANS_N_INIT, max_iter=KMEANS_MAX_ITER, tol=KMEANS_TOL`; no literal remains in either
runner. The values are scikit-learn's documented defaults, adopted rather than invented, and both
moves are in the safe direction — restarts are best-of-N by inertia so 10 dominates 8, and Lloyd is
monotone in inertia so a higher cap only affects runs that had not converged at 100.

The third divergence the original survey did not name was the **stopping rule**: the native engine
ran to exact label stability while scikit-learn stopped on `tol`. `facies::kmeans_once` now
implements scikit-learn's rule — centre shift against `KMEANS_TOL` scaled by the mean feature
variance — with the no-label-changed break kept as a fast path, which can only fire where the shift
is exactly zero. A reseeded empty cluster forces another pass rather than being counted as a step.

Unchanged and already conformant: k-means++ seeding, population-sd z-scoring, and label ordering by
the mean of the first curve.

**Two tests, deliberately.** `the_two_kmeans_engines_are_configured_from_one_definition` needs no
Python and so **fails the build** on divergence, as the requirement demands — and it checks the
values reach Python *from* Rust, which is the property that stops the fork recurring; a test merely
asserting both say 10 would pass two literals in two files.
`the_two_kmeans_engines_label_the_same_data_the_same_way` runs both engines on a three-blob fixture
and requires identical labelling, skipping where scikit-learn is absent. The fixture is deliberately
unambiguous: the two engines draw from different generators (SplitMix64 against NumPy's Mersenne
Twister), so identical labelling on overlapping groups is not on offer and pinning it would be a
false claim.

**Verified by.** SB-MLA-T23

#### SB-MLA-024 — One seed concept, one default          [P1] [status: PRESENT-OK]

**Requirement.** The random seed MUST be a single named concept with a single documented default
across every module and engine in the product.

**Rationale.** `SB-CORE-007` — one definition per constant. Neither value is wrong; two values for
one concept is the defect, and it is the kind that survives indefinitely because both behaviours
are individually correct.

**As-built.** `PRESENT-OK` (closed 2026-08-07) — one constant, `facies::SEED_DEFAULT = 42`, read by
both Electrofacies and GMM Facies module specs, by the native fallback, by the ML suite's Rust-side
defaults (`req.seed`, `req.split_seed`) and by both Python runners through the emitted preamble. The
dossier's §5.2 records the split as SandiBumi's own and notes that **no vendor in the corpus ships a
seed control at all**, so there was no external value to defer to; 42 wins because it was already
the number in `ml.rs`, in the ML dialog and in the leaderboard header, leaving the two facies specs
as the only sites to move.

**This changes results.** Electrofacies and GMM Facies previously defaulted to 7. A run made before
this recorded its seed and remains reproducible by typing 7 back in; what moves is the clustering
produced by pressing Run without touching the field. Logged in `REVIEW.md`.

**Verified by.** SB-MLA-T24

#### SB-MLA-025 — One within-cluster-sum-of-squares partition, three declared applications          [P1] [status: PRESENT-DIVERGENT]

**Requirement.** The Ward criterion MUST have one implementation. Where it is applied under
different ordering constraints — free ordering, sorted-value contiguity, depth contiguity — each
application MUST be a named variant of that one implementation, and the variant name MUST appear
in the provenance of every curve it produces.

**Rationale.** `SB-CORE-006`. Three code paths computing the same criterion is three places for it
to drift, and the *differences* between them are real and load-bearing: a depth-contiguous
segmentation and a free agglomeration answer different geological questions and a user must be
able to tell which they ran.

**As-built.** `PRESENT-DIVERGENT` — `hfu.rs:103` (`ward_partition`, exact O(K·m²) DP over sorted
`FZI`), `lorenz.rs:152` (`segment_dp`, the same exact DP over the depth-ordered profile — the
module doc at `lorenz.rs:17`–`:20` correctly states it is "the same Ward criterion as `hfu.rs`, but
here the natural depth order is preserved"), and `ml.rs:170`
(`AgglomerativeClustering(n_clusters=k, linkage="ward")`). The first two are genuinely different
questions correctly separated; the third is a third implementation of the criterion itself.

**Verified by.** SB-MLA-T25

#### SB-MLA-026 — The leaderboard evaluates the model the run will fit          [P0] [status: PRESENT-OK]

**Requirement.** The algorithm-comparison leaderboard MUST construct each candidate estimator from
the same specification the training run will use, including every user-supplied hyperparameter.
Where a hyperparameter cannot be honoured in evaluation, the leaderboard MUST say so for that row
rather than silently evaluating a different estimator. A test MUST assert that the evaluation and
training paths construct identical estimators for every supported algorithm.

**Rationale.** `SB-CORE-006` and `SB-CORE-002` together, and it is the highest-consequence instance
of either in this chapter, because the leaderboard's entire purpose is to be *trusted for a
choice*. A ranking that describes different models from the ones the user will fit is not a
degraded ranking — it is a ranking of the wrong things, presented cleanly.

**As-built.** ~~`PRESENT-DIVERGENT`~~ → **`PRESENT-OK` (closed 2026-08-07).** There is now ONE
declaration of every supported supervised estimator — `ML_BUILD_MODEL` — and both runners are
composed from it (`ml_runner()`, `ml_eval_runner()`), so an estimator cannot be declared in one and
not the other. Syncing the two copies would have fixed the three divergences below and left the
mechanism that produced them. `MlEvalRequest` now carries the run's own `params`, scoped by
`params_for` to the algorithm the dialog is showing: applying one algorithm's `C` to every row would
re-rank estimators against a value nobody chose for them, and every other row is scored at the
defaults the run would fit for it. The leaderboard states this per row in a **Settings** column
(`yours` / `defaults`), which is the requirement's "MUST say so for that row" half.

Was: the hyperparameters were declared twice and independently:
`ML_RUNNER` at `ml.rs:84`–`:169` reading `p.get(...)` with defaults, and `ML_EVAL_RUNNER`'s
`make_model` at `ml.rs:1132`–`:1169` with every value hard-coded. `MlEvalRequest` (`ml.rs:1233`)
carries no parameter map at all, and the frontend sends only the algorithm id list
(`src/ui/mlDialog.ts:466`). Three concrete divergences today: the polynomial `degree` branch
(`ml.rs:113`–`:119`) has no eval counterpart, so `degree = 3` is ranked as linear; the
`HistGradientBoosting` fallback is bare in eval (`ml.rs:1143`, `max_iter = 100`,
`max_depth = None`) against `max_iter = 300`, `max_depth = 4` in the run (`ml.rs:98`–`:101`); and
`SVC` is constructed without `probability=True` in eval (`ml.rs:1156`) against `ml.rs:135` — not a
cosmetic difference, since that flag makes scikit-learn fit internal Platt scaling and changes the
estimator. No test compared the two. All three are gone by construction.

**Verified by.** SB-MLA-T27 — implemented as
`the_leaderboard_builds_the_same_estimators_the_run_will_fit` (structural; names every estimator and
asserts each appears in the shared fragment and in **neither** runner body, so a runner that embedded
the fragment and then shadowed it — the shape the defect actually had — still fails) and
`a_polynomial_degree_is_ranked_as_a_polynomial_not_as_a_line` (behavioural, skips without
scikit-learn: on `y = x²`, `degree = 3` must outscore the straight line it used to be ranked as).

#### SB-MLA-027 — Every reported score names its protocol          [P1] [status: PRESENT-OK]

**Requirement.** Every reported performance metric MUST carry the evaluation protocol that produced
it — training-set, within-well cross-validation with its splitter and shuffle state, or blind-well
cross-validation with its group count. Two metrics produced by different protocols MUST NOT be
displayed under names that differ only by the metric.

**Rationale.** `SB-CORE-007`. An interpreter comparing `r2_cv5 = 0.88` from a run against
`score = 0.44` from the leaderboard has no way to know these are different questions, and the
natural reading — that the leaderboard is pessimistic or buggy — is exactly backwards.

**As-built.** `PRESENT-OK` (2026-08-07) — `name_protocol` puts a sentence beside every reported score in `metrics.score_protocols`: in-sample, cross-validated over whole wells, cross-validated within one well, or blind. The blind sentence distinguishes whole-held-out wells from rows drawn out of wells the model was also fitted on, decided by whether the two well sets are DISJOINT rather than by the requested split mode — the disjointness is the property that makes the rows rock the model has never been near. R-squared in-sample, over folds of the same wells, and over unseen wells are three different claims that are routinely quoted as one.

**Verified by.** SB-MLA-T28

#### SB-MLA-028 — Every fitted transform is fitted inside the fold          [P0] [status: PRESENT-OK]

**Requirement.** In any cross-validated evaluation, every transform fitted from data —
standardisation, imputation, dimensionality reduction, target encoding — MUST be fitted on the
training partition of each fold only, and applied to the held-out partition. A transform fitted on
the full dataset before splitting MUST NOT be used in an evaluation whose result is reported as a
held-out score.

**Rationale.** `SB-CORE-002`. A blind-well score is the honest number this product offers against
three vendors that offer none, and a leaked scaler makes it quietly optimistic in a known
direction. The magnitude is bounded by the held-out group's share of the transform: with `nsplits`
capped at the number of contributing wells (`ml.rs:1173`–`:1174`), a three-well run gives the
blind well roughly a **third** of the weight in the centring and scaling it is supposed to be
blind to.

**As-built.** ~~`PRESENT-DIVERGENT`~~ → **`PRESENT-OK` (closed 2026-08-07).** Was:
`ml.rs:1130` computed `Xs = StandardScaler().fit_transform(X)` over the entire pooled matrix, with
the splitter not constructed until `ml.rs:1175`–`:1176`; the comment at `ml.rs:1129` was correct
about a different property (per-column scaling commutes with column subselection) and did not
address the fit-before-split. Now one scaler per fold is fitted on that fold's training rows and on
nothing else, and the splitter partitions the raw matrix. The column-subselection property is still
used — it is what lets one fit per fold serve every feature subset instead of one per
(fold × subset) — but it is no longer standing in for an argument it never made.

The fit path never had this defect for the supervised case: `ml.rs:68` fits the scaler on `X`
(train) and transforms `A` (apply) with it, and `ml.rs:229`–`:247` persists that scaler with the
model precisely so the apply wells never refit it.

**Carried with it.** The same change moved permutation importance onto the held-out partition —
see the correction recorded under `SB-MLA-050`, whose as-built note asserted a group-level
cross-validation that did not exist.

**Verified by.** SB-MLA-T29 — implemented as
`no_transform_is_fitted_outside_the_folds_training_rows` (structural, runs on the green gate) and
`a_shifted_well_is_standardized_by_the_wells_that_trained_on_it` (behavioural, skips without
scikit-learn).

#### SB-MLA-029 — A facies mnemonic names the engine that produced it          [P1] [status: PRESENT-DIVERGENT]

**Requirement.** Every class-label curve MUST carry a mnemonic that identifies the method that
produced it, and two different methods MUST NOT be able to write the same mnemonic in one well.

**Rationale.** `FINDINGS` rule 8, applied inward. The dossier records the same defect in IP, where
`Cfit` means two different quantities (G-6.4), and prescribes `CFIT_BINS` and `CFIT_ABS`; the same
discipline is owed to `FACIES`. Two facies tracks in one well from two engines, one of them named
after neither, is not a naming preference — it makes the confusion matrix in `facies_tie.rs`
ambiguous about what it just compared.

**As-built.** `PRESENT-DIVERGENT` — `facies.rs:160` writes `FACIES`, `facies.rs:186` writes
`FACIES_GMM`, and the Python clustering path writes the frontend default `FACIES_ML`
(`src/ui/mlDialog.ts:113`). Two of the three name their engine; the k-means native module, which is
the most likely to be run, does not.

**Verified by.** SB-MLA-T30

#### SB-MLA-030 — Probability outputs are typed          [P2] [status: PRESENT-OK]

**Requirement.** A curve carrying a probability MUST declare what the probability is over and how
it is normalised. A relative or maximum-of-normalised score MUST NOT share a mnemonic convention
with a calibrated posterior.

**Rationale.** `FINDINGS` rule 8. The dossier records that **both IP and Geolog emit relative-only
probabilities and say so**, which makes the distinction a cross-tool interoperability question as
well as an internal one: a `PROB` curve imported from either vendor is not the same quantity as a
mixture posterior.

**As-built.** `PRESENT-OK` (2026-08-07) — `PROB_MEANING` declares per estimator what the `_PROB` curve IS, and the run records `prob_definition` and `prob_normalisation`. They are not interchangeable and a reader cannot tell them apart from a track: a random-forest vote share, a k-NN agreement fraction that can only take k+1 values, a naive-Bayes posterior resting on an independence assumption log curves do not satisfy, and an SVM's Platt-scaled distance are four different quantities. GMM's responsibility — the one genuinely calibrated posterior — is declared separately with its own interpretive scale, matching `facies.rs`'s `FPROB` doc. The mnemonic itself is still shared; splitting it would rename curves in existing projects, so the distinction is carried as a declaration rather than a rename.

**Verified by.** SB-MLA-T31

### Group D — The disagreement is the product

#### SB-MLA-031 — Shipped vendor defaults are surfaced at the point of choice          [P2] [status: ABSENT]

**Requirement.** Where the corpus holds more than one vendor's shipped default for a parameter
SandiBumi exposes, the product MUST present the competing values with their sources at the point
where the user sets that parameter, and MUST record the user's choice as a decision with its own
provenance.

**Rationale.** `SB-CORE-013` and `03_EVIDENCE_BASE.md` §14.2. This domain has the corpus's densest
example: for the number of clusters alone, IP advises **15 to 20** as a first-stage count and
**4 to 5** consolidated (T2), Techlog ships a hard default of **5** in two independent modules
(T3, corroborated), and Geolog states none at all across its entire Facimage suite (T3, ML-8).
None of the three tells the interpreter the other two exist, and none of them can — they cannot
credibly publish a competitor's defaults.

**As-built.** `ABSENT` — `facies.rs:40` declares `K` default 5.0 with range 2…12 and
`src/ui/mlDialog.ts` offers a plain numeric field; no source or alternative is shown anywhere.

**Verified by.** SB-MLA-T32

#### SB-MLA-032 — The normalisation basis is a recorded choice, not an implicit one          [P1] [status: PRESENT-OK]

**Requirement.** The normalisation scheme applied before any distance-based method MUST be an
explicit, recorded parameter with a named set of options, and MUST appear in the provenance of
every curve the method produces.

**Rationale.** Dossier §3.5 — the widest silent divergence in the domain. Geolog offers four
schemes and three metrics; IP forces z-score with no alternative and does not document its scheme
for SOM or neural training at all (G-9.5); Techlog HRA normalises into PCA space at a 0.95
variance cut-off. A model whose normalisation is not recorded cannot be compared with a model from
any other tool, or with itself after a data change.

**As-built.** `PRESENT-OK` (2026-08-07) — the standardisation basis is recorded, not implied: `pre_transform` names which rows it was fitted on (fit-rows-only under a blind split, so the blind wells' mean and scale never reach the model), and `standardize_basis_mean` / `_scale` carry the numbers. A saved model already carried its scaler; what was missing was any statement of what the scaler was fitted against.

**Verified by.** SB-MLA-T33

#### SB-MLA-033 — A fixed normalisation basis is available, so adding a well does not move existing boundaries          [P1] [status: ABSENT]

**Requirement.** SandiBumi MUST offer a normalisation basis that is independent of the current
model-build set — fixed limits per curve, stored with the model — in addition to a
data-derived basis. Where a data-derived basis is used and the model-build set changes, the product
MUST report that the feature space was rescaled.

**Rationale.** This is the one place in the domain where a vendor has an answer the others lack and
SandiBumi should simply take it. Geolog's `Plot Limits` normalisation ties the feature space to the
display scale the analyst chose, which is stable across wells; `Data Range` ties it to the training
data, which is not. **IP has no equivalent choice at all**, so adding one well to an IP model-build
set silently rescales the entire feature space and moves every cluster boundary in the wells that
were already there. The dossier calls this "the add-a-well trap" and ships `T-ML-NORM-1` for it.

**As-built.** `ABSENT` — both engines compute their statistics from the samples in hand
(`facies.rs:100`–`:129`, `ml.rs:68`). There is no fixed-limits option.

**Verified by.** SB-MLA-T34

#### SB-MLA-034 — Every automatic pre-transform is announced          [P1] [status: PRESENT-OK]

**Requirement.** Any transform applied to an input curve without the user requesting it for that
curve — an automatic logarithm by family, a clip, an outlier removal — MUST be announced per curve
in the run output and recorded in the curve's provenance.

**Rationale.** Dossier §3.5 trap 1, and the cleanest available demonstration that announcing is
cheap: **Techlog announces its automatic `log10` in the Output window and IP announces nothing**,
for the same behaviour, in the same domain. Running the same GR/RHOB/NPHI/RT cluster job at
default settings in the two products gives different clusters for this reason alone. Neither answer
is wrong; a tool that does not state which it did produces a deliverable nobody can reproduce.

**As-built.** `PRESENT-OK` (2026-08-07) — `pre_transform` is emitted on both branches, including the one where nothing was transformed. Standardisation is not cosmetic: it is what makes a DBSCAN `eps` meaningful and what stops a resistivity in ohm-m dominating a porosity in v/v on any distance-based method, so a user reading `eps = 0.5` has to know that 0.5 is in standard deviations of a particular basis. The un-standardised branch says which curve will dominate instead of leaving an absence.

**Verified by.** SB-MLA-T35

#### SB-MLA-035 — A transformed quantity is a distinct quantity with its own name and unit          [P0] [status: PRESENT-OK]

**Requirement.** A log-transformed or otherwise re-scaled quantity MUST be a separate entry in the
curve registry, with its own mnemonic and its own unit. It MUST NOT be represented as the original
quantity with a transform flag. Statistics MUST be reported in the units of the entry they were
computed on, and any back-transform to display units MUST be an explicit, logged step.

**Rationale.** Dossier §3.5 trap 3, and the sharpest instance in this domain of `FINDINGS` rule 3
and rule 8 acting together. IP 2025 states that its `log10` flag "changes the reported statistics,
not just the internals — reported minima/maxima/means become logarithmic values", visible as a
**negative `PERMCORE` mean** in the vendor's own screenshot (T2). A cluster-statistics table
reading `PERMCORE mean = −0.4` is not an error state: it is `10^−0.4 = 0.398 mD` reported in log
units under a header that says mD. It renders, it prints, it reaches a client deck, and the only
reader who can catch it is one who already knows the flag was set — because the neighbouring rows
read `−0.4`, `1.2`, `2.8` and the eye takes them for a plausible spread. This is a **P0** because
it is a wrong number in a deliverable with no visible symptom, which is the failure class this
whole document exists to prevent.

**As-built.** `PRESENT-OK` (2026-08-07). The capability now exists — ML regression takes
`target_transform: "log10"` — and it was built to this requirement rather than retrofitted to it,
which is the only reason it is not the vendor's defect with a different logo.

*Two curves, never one with a flag.* A transformed run writes `<base>_LOG10` (the model's own
prediction, in log units) **and** `<base>` (its back-transform, in the target's units). The suffix
is part of the mnemonic rather than a column on the row, because a flag can be dropped by any
reader that does not know to look for it, while a name travels into the log view, the LAS export,
the workbook and the deck. `ml.rs::LOG10_SUFFIX`, and the naming in `run_ml` immediately before the
write loop. The back-transform is the "explicit, logged step" the requirement asks for — an
in-place back-transform would satisfy "the user gets mD" while destroying the record of what the
model actually predicted, and the reported R²/RMSE would still be log-space numbers describing a
curve that no longer exists.

*The registry gained somewhere to disagree.* `computed_curves` had no unit column at all, so a
computed curve's unit could only ever come from an `equations.output_units` row of the same name —
which is precisely what made this trap possible. `db::curve_unit` (PRIMARY KEY `(well_id,
curve_name)`) is declared by whatever WROTE the curve, on the [`declare_class_curves`] argument:
the writer is the only place the answer is known rather than guessed. A blank unit stores NULL, not
`""` — "dimensionless" and "we do not know" are different statements and only the second should
let a reader fall back to a guess. `equations::list_curve_catalog` and `export_las` both prefer the
declared unit over the inferred one; the LAS header is where the failure is most expensive, because
that number leaves the building attached to a unit and the reader has no way to check it.

*Scores are labelled with the space they were computed in.* `metrics.metric_space` carries
`log10(mD)` and the run panel prints it above the score table. An R² in log space is not the same
claim as an R² in mD — it is usually the lower of the two, because the log fit is not rewarded for
getting the few largest values roughly right.

*The leaderboard was transformed too, or SB-MLA-026 would have been broken by this change.* That
requirement says the leaderboard must rank the model the run will fit; a model fitted on log10(k)
is a different model from one fitted on k, and in linear space an R² over four decades is dominated
by the handful of highest values, so the winner there is routinely not the winner in log space. The
leaderboard note says which space it ranked in, and the note is JOINED with the combo-cap note
rather than replacing it.

*A zero is dropped and counted, never floored.* A permeability of exactly 0 is a real reading and
has no logarithm. Flooring it to some small number would be an invented parameter anchoring the low
end of the fit; the count is the honest thing to show. The rows are dropped from the feature matrix,
the target and the well-index vector TOGETHER — drop from the target alone and every feature row
after the first drop belongs to a different depth, the model fits confidently on scrambled pairs,
and nothing downstream can catch it because the row count still agrees with itself.

`hfu.rs` and `lorenz.rs` already worked in `log10` space internally (`lorenz.rs:17`–`:20`) and
correctly did not report log-space statistics under linear names; this generalises that behaviour
and gives it a registry to record it in.

**Verified by.** SB-MLA-T36 —
`ml::tests::a_log_transform_drops_a_row_from_every_column_or_from_none` (needs no Python, so a
regression fails the build) and
`ml::tests::a_log_fitted_prediction_and_its_back_transform_are_two_curves_with_two_units` (needs
scikit-learn, self-skips), which asserts the two mnemonics, the two units, the `10^log = linear`
relation, the metric-space label, and — in the requirement's own terms — that the exported LAS
header over the mD column does not carry log-space numbers.

#### SB-MLA-036 — Enumerated methods are addressed by id, never by display string          [P1] [status: PRESENT-OK]

**Requirement.** Linkage, distance metric, normalisation scheme, map geometry and every other
enumerated method choice MUST be addressed by a canonical identifier with a separate display
label. Vendor spellings MUST be accepted as input aliases only. An unrecognised identifier on load
MUST be an error, never a silent fall-through to a default.

**Rationale.** `FINDINGS` rule 7, with two independent instances in this domain. Geolog's Facimage
help spells the metric **`Euclidian`** verbatim while IP, Techlog and the literature spell it
*Euclidean* — an importer matching on the display string falls through to a default silently. IP
names linkage method #1 `Minimum` on one page and `Minimise` on a sibling (G-6.10). The
consequence in both cases is a model that loads and computes with a method the user did not choose.

**As-built.** `PRESENT-OK` (2026-08-07) — the two enumerations that could silently absorb an unknown value now refuse it by name. `facies.rs`'s `OPT_STANDARDIZE` was `!= "NONE"`, so every typo, stale chain step and hand-edited workflow silently standardised — the branch that changes the answer most, selected by an option nobody validated; it is now matched against `ZSCORE` / `NONE` and anything else is an error naming the value. The runner's `linkage` is validated against its enumeration rather than passed through to scikit-learn to raise. `task` and `algorithm` already failed on an unknown id.

**Verified by.** SB-MLA-T37

### Group E — Method obligations from the evidence

#### SB-MLA-037 — Fuzzy combination across curves is the reciprocal sum          [P1] [status: ABSENT]

**Requirement.** Where SandiBumi implements the Cuddy fuzzy curve or facies predictor, the
combination of per-curve bin probabilities MUST be the parallel (reciprocal) sum
`P(b) = 1 / Σ_j (1 / P_j(b))`. A product rule MUST NOT be substituted. The implementation MUST be
pinned by a regression test that fails if the rule is switched.

**Rationale.** Dossier §3.1 and §5.1 F2, IP 2025 `statisticalcurveprediction.htm` (T2). The two
rules **select different bins**: on the dossier's counter-example the reciprocal sum gives
`P_A = 0.04541`, `P_B = 0.07333` and picks B, while the product gives `0.049005` against
`0.010648` and picks A. At that depth the two implementations emit a different facies code or a
permeability from a different bin, and both look entirely plausible on a log plot. The evidence is
a **single T2 raster with no external arbiter**, which is stated here because it is the weakest
support any load-bearing equation in this chapter rests on — Geolog implements the same Cuddy
method but is a second implementation, not a second source for this equation.

**As-built.** `ABSENT` — no fuzzy implementation exists.

**Verified by.** SB-MLA-T38

#### SB-MLA-038 — Equal-population binning reports its actual populations          [P2] [status: ABSENT]

**Requirement.** A binning scheme that cannot achieve its requested bin count or its requested
equal populations — because of ties or a concentrated value — MUST report the populations it
actually produced. It MUST NOT claim the requested structure.

**Rationale.** IP documents this failure explicitly for its own fuzzy binning (T2), which makes it
a known-mode rather than a hypothetical. It matters more than it looks: under the `√n_b` weighting
of F1, the bin populations enter the probability directly, so a misreported binning silently
changes every probability downstream.

**As-built.** `ABSENT` — no binning scheme exists.

**Verified by.** SB-MLA-T39

#### SB-MLA-039 — The fuzzy uncertainty band has a defined edge behaviour          [P2] [status: ABSENT]

**Requirement.** Where the requested percentile band falls outside the cumulative distribution, the
band edge MUST be computed by the stated fallback rule rather than clipped or returned as missing,
and the fallback MUST be recorded as having fired.

**Rationale.** IP states the rule: outside `[0, 1]`, the result is the first or last bin mean
∓ two standard deviations of that bin's spread (T2). Edge rules are where reimplementations
diverge invisibly, because the edge case is rare in test data and common in real wells.

**As-built.** `ABSENT` — no fuzzy implementation exists.

**Verified by.** SB-MLA-T40

#### SB-MLA-040 — The bin-count weighting is explicit, with no hidden default          [P2] [status: ABSENT]

**Requirement.** The `√n_b` weighting inside the per-curve bin probability MUST be an explicit,
always-visible parameter with no default, and the product MUST state that disabling it is a
deviation from Cuddy as printed rather than a neutral option.

**Rationale.** G-6.5 / G-9.10 is an open IP contradiction — the prose says the option defaults
selected, the panel shows it cleared — so there is no defensible default to inherit. The
substantive point is that the `√n_b` term is **inside** the printed `P(C_b)`, so switching it off
changes the published equation rather than configuring it. It also interacts with the open
sub-question in `SB-MLA-037`'s rationale: since `P` carries `√counts`, whether the reciprocal sum
is applied to raw or per-curve-normalised probabilities changes the answer whenever bin
populations differ (ML-11), which under variable-size binning is always.

**As-built.** `ABSENT` — no fuzzy implementation exists.

**Verified by.** SB-MLA-T38. Escalation E-4.

#### SB-MLA-041 — SOM decay is parameterised by total iterations, and the degenerate form is refused          [P3] [status: ABSENT]

**Requirement.** Where SandiBumi implements a self-organising map, the neighbourhood and
learning-rate decay MUST be parameterised by a **required** total-iteration count, with no default.
A configuration expressing the decay constant in terms of the current iteration MUST be refused
with a message naming the degeneracy. The deviation from IP's printed form MUST be carried in the
parameter's source string.

**Rationale.** Dossier §3.3 and §5.1 F9. IP's printed `λ = t / log σ₀` with `t` the current
iteration makes the learning rate independent of `t` and collapses `σ` to exactly 1.0 from the
first iteration for a natural log — the map degenerates into a nearest-prototype vector quantiser
with no topology preservation. **A reimplementer who transcribes faithfully ships a SOM that is not
a SOM.** There is no arbiter: Techlog ships no SOM training math anywhere in its documentation tree
(ML-4, a verified negative result) and Geolog exposes `Iterations` + `Shakings` with no decay law.
The requirement is deliberately not "guess IP's intent" — it is to refuse the degenerate
parameterisation and carry SandiBumi's own with a source string that says why.

**As-built.** `ABSENT` — no SOM implementation exists. P3 because the capability itself is not a
near-term target; the requirement is recorded now so that the trap is closed before anyone starts.

**Verified by.** SB-MLA-T45. Escalation E-1.

#### SB-MLA-042 — Map quality is reported by a defined distortion measure          [P3] [status: ABSENT]

**Requirement.** A SOM implementation MUST report a map-quality measure computed by the stated
distortion form, with its convention (lower is better) and its neighbourhood radius stated
alongside the value.

**Rationale.** F10, IP 2025 `som.htm`, which cites Wu & Takatsuka, *Neural Networks* 19 (2006)
(T2). A map with no quality readout is a black box whose failure mode — a map that never ordered —
is invisible, and this is the one SOM quantity IP prints with a primary citation attached.

**As-built.** `ABSENT` — no SOM implementation exists.

**Verified by.** SB-MLA-T46

#### SB-MLA-043 — The cluster randomness index ships          [P2] [status: ABSENT]

**Requirement.** SandiBumi MUST provide a stratigraphic cluster-quality index measuring vertical
bed coherence, computed as the ratio of mean cluster-layer thickness to the thickness expected
under a random arrangement of the same cluster proportions, reported alongside the geometric
cluster-quality measure.

**Rationale.** Dossier §2.3 / §3.6, F11. It is printed by IP as **ASCII, not a raster**,
identically in two places and unchanged from IP 2018 (T2) — so it is an equation, not vendor
lookup data, and transcribing arithmetic from a printed formula is not the transcription
`CONTRACT.md` §2.1 prohibits. It measures something **no general-purpose ML library provides** and
something silhouette structurally cannot: silhouette is computed with no knowledge that the samples
are ordered in depth. A `K` that is good on both criteria is a genuinely different claim from a `K`
that is good on either. This is the cheapest genuine capability gain in the chapter.

**As-built.** `ABSENT` — no implementation anywhere in the tree.

**Verified by.** SB-MLA-T41, SB-MLA-T42

#### SB-MLA-044 — The native clustering path reports cluster quality          [P1] [status: PRESENT-OK]

**Requirement.** Every clustering run MUST report a per-cluster and an overall geometric quality
measure, whichever engine executed it, with its sample count.

**Rationale.** `SB-CORE-002` and parity between the two engines. A facies run that offers no
quality readout gives the interpreter no basis to reject the result, which is the same
"proceeds and returns a plausible number" pattern the corpus catalogues in the vendors — here in
SandiBumi's own most-used clustering path.

**As-built.** `PRESENT-OK` (2026-08-07) — both native engines now emit `FACIES_SIL`, a **per-sample** silhouette, alongside their class curve. Per sample rather than as one figure, and that is the better answer rather than a workaround for the module framework having no scalar channel: a single number says the clustering is "0.42 good" and cannot say the sands are clean while the interbedded section is guesswork. Depth-resolved, it says exactly that, and it plots beside the facies it qualifies. Negative values are the case that matters — a sample sitting closer to another cluster than its own — which a class code can never show, because a facies track looks equally confident everywhere.

The cap matches the Python path's (`SILHOUETTE_CAP = 5000`) so the two engines' numbers stay comparable; a quality measure computed over different sample counts on the two sides would be exactly the engine disagreement `SB-MLA-023` exists to prevent, arriving through the diagnostic. On the GMM engine it sits beside `FPROB` deliberately: they answer different questions, and a confidently-fitted but badly-separated mixture reports a high `FPROB` and a low silhouette.

**Verified by.** `the_native_clustering_says_how_well_separated_it_actually_is` — pinned from both sides, because a statistic that returned 0.9 for separated blobs AND for one undifferentiated cloud would pass any single-sided check and still be quoted.

**Verified by.** SB-MLA-T43

#### SB-MLA-045 — Restart spread is reported as a convergence diagnostic          [P2] [status: ABSENT]

**Requirement.** Where a clustering run performs multiple restarts, the run MUST report the
distribution of the objective across restarts and how often the retained solution was reached, and
MUST state that this is a convergence diagnostic and not a cluster-count criterion.

**Rationale.** Techlog's fall-off diagnostic: sort the cumulative Euclidean distance across the 50
runs, and a solution found about 10 % of the time is the happy medium — a flat left side means the
classes are too broad, and one that "just keeps decreasing" means the global minimum was never
found and `K` is too high (T3). The vendor also states its own limit, that the measure
"necessarily always decreases with increasing number of classes", and that caveat is part of the
requirement rather than a footnote to it — shipping the diagnostic without it invites exactly the
misreading the vendor warns about.

**As-built.** `ABSENT` — `facies.rs:144`–`:151` runs eight restarts and keeps only the best
inertia; the other seven values are discarded. The information is already computed.

**Verified by.** SB-MLA-T44

#### SB-MLA-046 — Hierarchical linkage is a named enumeration with a sourced default          [P2] [status: PRESENT-OK]

**Requirement.** Hierarchical clustering MUST expose the five linkage rules as canonical
identifiers with stated update rules, defaulting to Ward, and the default MUST carry its
three-vendor corroboration in its source string.

**Rationale.** F12 and §5.2. Ward is the one parameter in this domain with a genuine three-vendor
agreement — IP `cluster_analysis.htm` (T2), Geolog `facimage_03_generate_hc.3.6.html` "WARD
(default)" (T3), and Techlog's TechCore Petrophysical groups parameter table, `Default value`
column, for both `HC > Aggregation method` and `SOM > Aggregation method` (T3). **The scoping
caveat is part of the requirement**: Techlog's *Ipsom* HC page states no default at all, only that
"Ward method: This is the most used method", which is popularity, not a default — that page must
never be cited for this value.

**As-built.** `PRESENT-OK` (2026-08-07) — linkage is validated against the enumeration and an unknown value is refused by name rather than passed to scikit-learn to raise. The `ward` default is sourced in the comment by the criterion rather than by preference: it is the only linkage that minimises within-cluster variance, which is the same criterion `facies.rs`'s k-means and `hfu.rs`'s Ward partition already use, so it is the choice that keeps the three consistent.

**Verified by.** SB-MLA-T37

#### SB-MLA-047 — PCA reports loadings and correlation-circle coordinates          [P2] [status: PRESENT-OK]

**Requirement.** A principal-component analysis MUST report, in addition to explained variance, the
eigenvector loadings and the correlation-circle coordinate for each input curve on each retained
component, computed as the loading scaled by the square root of that component's eigenvalue.

**Rationale.** F13. This is one of only two quantities in the domain that **IP and Techlog state
identically and independently** — IP `principal_component_analysis.htm` (T2, whose worked-example
arithmetic was independently verified) and Techlog
`utility-techstat-principal-components-analysis-pca.html` (T3) — which makes it the best-supported
equation in the chapter. Without loadings, a PCA tells an interpreter how much variance was
captured and nothing about *which curves* drove it, which is the only petrophysically useful part.

**As-built.** `PRESENT-OK` (2026-08-07) — `metrics.loadings` carries `{component: {curve: weight}}`. The variance ratio says how much a component carries; the loadings say what it is made of, which is the half a petrophysicist reads — "PC1 is mostly density against neutron" is now answerable without re-deriving it. Correlation-circle COORDINATES (loading scaled by sqrt of the eigenvalue) are still absent; the loadings are the input to them.

**Verified by.** SB-MLA-T26

#### SB-MLA-048 — Component sign is fixed by a stated convention          [P1] [status: PRESENT-OK]

**Requirement.** The sign of every principal component MUST be fixed by a documented convention, so
that repeated runs on identical inputs produce identical component signs, loadings and scores.

**Rationale.** `SB-CORE-011`. An eigenvector's sign is arbitrary and library-version-dependent; a
PC score curve that flips sign between runs is byte-identical in nothing while being
mathematically identical, and an interpreter reading a flipped `PC1` track will read the
petrophysical trend backwards. This is the one place in the domain where a reproducibility failure
survives a correct seed.

**As-built.** `PRESENT-OK` (2026-08-07) — each component is oriented so its loading on the FIRST feature curve is non-negative, and `sign_convention` states it. A principal component is only defined up to its sign and the solver may return either, so left alone the same wells re-run give a PC1 that is the mirror of last month's: every crossplot reversed, every "high PC1 is the clean sand" inverted, and nothing to show anything changed. Anchored to the user's own first input rather than to the largest loading, because the largest loading can itself move between runs and a rule that moves is not a convention.

**Verified by.** SB-MLA-T08

#### SB-MLA-049 — Nearest-neighbour prediction is a normalised weighted average, and its weight function is SandiBumi's          [P2] [status: ABSENT]

**Requirement.** A `k`-nearest-neighbour curve prediction MUST be computed as `Σ w·y / Σ w`. The
distance-weighting function and its length scale MUST be SandiBumi's own, named as such, with the
length scale a first-class parameter, and MUST NOT be attributed to a vendor.

**Rationale.** F14, ML-12. Geolog's two describing pages disagree: `5.05` says the prediction is
"the **summation** of the weighted associated log values" (stated twice, for KNN and Barycenter)
while `6.8` says "an exponential distance weighted **average**" (T3). They agree if and only if
`Σ w = 1`, which is stated nowhere. The normalised form is the only one that is unit-correct — an
unnormalised sum of `k` weighted porosities is not a porosity and scales with `k` — and the failure
it prevents is concrete: **a porosity that doubles when you ask for one more neighbour.** Separately,
the weight function itself is never printed: `exp(−d/h)` and `exp(−d²/h²)` are both consistent with
Geolog's words, `h` is exposed nowhere in the help set, and the dossier is explicit that this "must
not be presented as Geolog's". Note the structural identity with ML-1: two vendors, two continents,
the same missing denominator, because **"summation of the weighted values" is a vendor idiom for
"weighted average" and is unsafe to transcribe literally.**

**As-built.** `ABSENT` for curve prediction — `ml.rs:136`–`:138` offers
`KNeighborsClassifier(n_neighbors=7)` for classification only, with scikit-learn's uniform weights.
There is no KNN regression path and no distance weighting.

**Verified by.** SB-MLA-T47, SB-MLA-T48. Escalation E-2.

#### SB-MLA-050 — Feature scoring by leave-one-out excludes the held-out frame          [P2] [status: ABSENT]

**Requirement.** A leave-one-out feature-subset score MUST exclude the held-out frame from its own
neighbour search. A scoring run whose `k = 1` error is exactly zero on the training set MUST fail.

**Rationale.** F15, Geolog `facimage_06_reference_hc.6.8.html` (T3), which names this exact trap:
without the exclusion, `k = 1` reconstructs the training data with zero error and every feature
subset scores perfectly. It is the rare vendor page that documents its own failure mode, and the
fixture it implies is a hard-fail test rather than a tolerance check.

**As-built.** `ABSENT` — no feature-subset scoring exists.

**Correction, 2026-08-07.** This note previously read that the leaderboard's permutation importance
"answers a different question and is correctly cross-validated at the group level". The first half
was right and **the second half was false.** `ml.rs:1217`–`:1219` fitted a second model over the
entire matrix and permuted against that same entire matrix: no splitter, no groups, no held-out
partition anywhere near it — and on a matrix that was itself globally standardised, so the figure
was in-sample twice over. Verified at source before the fix.

It mattered because the importance and the blind score are printed in **one leaderboard row**. A
reader taking "R² 0.81, and GR is the strongest predictor" as one finding was reading a held-out
number beside an in-sample one with nothing marking the difference. Importance is now measured on
each fold's held-out rows by that fold's own model and averaged, and the across-fold spread is
reported beside it — a feature that carried in one well and nowhere else has a large spread and is
not a predictor however high its mean. `n_repeats` is unchanged at **5** (§5, `ml.rs`); only the
population it is measured over changed.

**Verified by.** SB-MLA-T49

#### SB-MLA-051 — A contingency table carries both normalisations, each labelled with its axis          [P1] [status: PARTIAL]

**Requirement.** A contingency or confusion tabulation MUST emit raw counts, row-normalised
percentages and column-normalised percentages, each cell labelled with the axis it was normalised
on. A single unlabelled "percentage" MUST NOT be emitted.

**Rationale.** Dossier §2.8.d. Geolog's Facimage Comparison reports "recognition rates" normalised
**by column**; Techlog's Ancor reports "row frequency" normalised **by row** (both T3). A bare
"72 %" therefore means two different things depending on which product produced it, and nothing in
the number says which. The consequence is bounded only by class imbalance: a reference class
holding 10 % of samples mapped onto a model class holding 60 % reads as a high recognition rate on
one axis and a low one on the other. This looks like a display requirement and is not — the
ambiguity is in the number.

**As-built.** `PARTIAL` — `facies_tie.rs:100`–`:141` builds the raw count matrix correctly with
sorted label axes, and reports per-reference-class dominant purity (`facies_tie.rs:121`–`:126`),
which is a **row-wise** fraction. The column-wise recognition rate is absent and the axis is not
named in the payload.

**Verified by.** SB-MLA-T50

#### SB-MLA-052 — The tie-in acceptance threshold ships absent and visible          [P2] [status: PARTIAL]

**Requirement.** The dominant-class purity above which a facies mapping is accepted MUST ship with
no default, MUST be presented as a required user decision, and the chosen value MUST be recorded
with the result.

**Rationale.** `SB-CORE-004` and §12.2's standing decision. The method note the module implements
says "accept the mapping if dominant-class purity is above a threshold" and **states no value**; no
source in the corpus states one either. Shipping absent is correct — but an absence that is merely
implicit is indistinguishable from an oversight, and the user needs to see that the choice is
theirs.

**As-built.** `PARTIAL` — `facies_tie.rs` computes `overall_purity` (`facies_tie.rs:128`) and
returns it with no threshold, which is the right behaviour; there is no parameter, no prompt and no
record of a decision.

**Verified by.** SB-MLA-T51

#### SB-MLA-053 — A tolerance expressed in standard deviations is named for its unit          [P3] [status: PRESENT-OK]

**Requirement.** Where an outlier or quality rule is expressed as a multiple of a spread statistic,
the parameter MUST be named for the statistic it multiplies. A bare `tolerance` MUST NOT be used.

**Rationale.** ML-9 and F16. Techlog's own four pages disagree: two name the multiplicand the
standard deviation, one the variance, one does not say (T3). SD is the only reading consistent with
the vendor's stated `a = 2 → N ≈ 5 %` pairing — ±2σ leaves ≈ 4.6 % outside — and the only
**dimensionally stable** one, since under the variance reading `a·s` would carry curve units
squared and be compared against a quantity in curve units. Presented as the strongly corroborated
reading, not adjudicated. The requirement is therefore a naming rule rather than a value: a bare
"tolerance = 2" imported from Techlog is ambiguous across the vendor's own documentation.

**As-built.** `PRESENT-OK` (2026-08-07) — the only spread-multiple parameter here is DBSCAN's `eps`, and the name stays `eps` because that is scikit-learn's own and renaming it would fork the vocabulary. What it multiplies is DECLARED instead: `eps_unit` says "standard deviations of the standardisation basis" or "the RAW mixed units of the selected curves". The second case now also raises `eps_warning`, because an un-standardised `eps` is the failure this requirement is really about — a resistivity in ohm-m and a porosity in v/v are orders of magnitude apart, so the resistivity alone decides every neighbourhood, and the result is not an error but one huge cluster or noise everywhere.

**Verified by.** SB-MLA-T52

#### SB-MLA-054 — The depth-resampling decision is logged for every ML input          [P1] [status: ABSENT]

**Requirement.** Where an ML input is resampled, interpolated or depth-snapped to reach a common
frame, the decision MUST be recorded per curve — the source sampling, the target frame, the method
and the tolerance — and MUST appear in the provenance of the output.

**Rationale.** `FINDINGS` rule 15, made load-bearing by Geolog's own guidance: core data is
aperiodic **point** data and the permeability curve must control sampling, while facies is
aperiodic **tops** data (T3). An ML training frame is by construction a join across curves of
different sampling, so this is not an edge case in this domain — it is every run.

**As-built.** `ABSENT` — the frame fetch (`fetch_curve_frame_from_set`, used at `ml.rs:1342` and
`facies_tie.rs:184`) resolves curves onto a common frame and records nothing about how.
`facies_tie.rs:144`–`:162` is the exception that shows the pattern: `CORE_MATCH_TOL_M = 1.0`
matches a core plug to the nearest log sample within a stated tolerance, which is a good rule that
is not reported in the result.

**Verified by.** SB-MLA-T54

#### SB-MLA-055 — A class label is never interpolated          [P0] [status: PRESENT-OK]

**Requirement.** A curve whose values are class identifiers MUST NOT be linearly interpolated,
averaged, or resampled by any method that can produce a value not in the original label set. Class
curves MUST be resampled by nearest-value or step-interval methods only, and the curve registry
MUST carry the categorical flag that enforces this.

**Rationale.** Geolog's Facimage documentation makes the point directly for its own product: facies
is aperiodic **tops** data and interpolation must be `TOPS`, "or facies codes get numerically
interpolated into meaningless intermediate values" (T3). A facies curve holding 2.5 between a class
2 and a class 3 bed is not a rounding artefact — it is a class that does not exist, and it
propagates: it will be rounded somewhere downstream into whichever neighbour the rounding rule
favours, silently reassigning the bed boundary. This is P0 because the product already emits three
class curves (`FACIES`, `FACIES_GMM`, `FACIES_ML`) and there is nothing in the type system stopping
any resampling path from doing this.

**As-built.** ~~`ABSENT`~~ → **`PRESENT-OK` (closed 2026-08-07).** Was: the three class curves were
written as `f32` through `write_computed_curves_versioned` with no categorical marking, and nothing
distinguished them from a continuous curve at any downstream consumer.

**The registry.** New `curve_class` table (`db.rs`, picked up by `create_schema` — no migration
needed), holding `(well_id, curve_name, source)`. Its own table rather than a `curve_meta` column,
because `curve_meta` describes the IMPORT store and these curves live in `computed_curves`, which
has no metadata row at all — that absence, not the resampling heuristic, was the real gap.
`workflow.rs` declares a run's class outputs after the write succeeds, resolved through the same
rename and `OUT_PREFIX` the write used, so a renamed output is still protected.

Declaration is **per declared output key**, not per module (`modules::class_outputs`). `gmm_facies`
is the case that makes it load-bearing: it writes `FACIES_GMM` beside `FPROB`, and `FPROB` is an
ordinary continuous probability that must stay averageable. A per-module flag or a name prefix would
wrongly protect it.

**The enforcement, in four places.** `reframe::class_safe_method` coerces Interpolate → Nearest and
every averaging method → Mode, reporting the coercion per curve by name — a re-frame reports its
resolved method, so a substitution there is visible. The three modules that would otherwise average
codes cannot report, so they REFUSE: `frame::block` refuses MEAN / GEOMETRIC / HARMONIC / MEDIAN
(MEDIAN included deliberately — it routes through the R-type-7 percentile, so an even-count bed of
{1, 2} returns 1.5) and steers to MODE, while `condition::smooth` and `condition::despike` refuse
outright, having no safe form: smoothing produces values *between* those measured, and on a class
log a lone code between two others is a thin bed rather than a spike.

**Latent bug found and fixed here.** `block` had no MODE arm at all — while `coreimage.rs` has been
printing *"use Frame > Block with OPT_STAT = MODE, the one upscale that carries a class code whole"*
since it shipped. Following the application's own printed advice fell through the match to the
arithmetic mean.

**The rule that keeps it from becoming a heuristic:** a DECLARATION overrides an explicit choice; a
GUESS (`reframe::looks_discrete`) may only pick the default. `reframe`'s own contract promises that
a caliper logged in whole inches stays averageable when the user says so.

**Verified by.** SB-MLA-T53 — implemented as
`a_class_output_is_declared_under_the_name_the_run_wrote_and_a_probability_output_is_not`,
`a_declared_class_curve_is_never_averaged_and_an_undeclared_one_keeps_the_method_asked_for`,
`a_class_curve_is_carried_by_its_commonest_value_rather_than_averaged`,
`a_class_curve_is_blocked_by_its_commonest_code_and_refuses_every_average` and
`a_class_curve_is_refused_by_smooth_and_despike_and_an_undeclared_one_is_not`. All pin from both
sides — an undeclared curve with class-looking values is untouched.

#### SB-MLA-056 — Null discipline holds through the ML path with no opt-out          [P1] [status: PRESENT-OK]

**Requirement.** A missing value MUST remain missing through every ML input path. There MUST be no
option, default or otherwise, by which a null becomes a numeric sentinel that enters arithmetic.

**Rationale.** Dossier §3.9 item 2 and `FINDINGS` rule 6. IP ships the opposite as its unchecked
default: with `Check intermediate results for null data` cleared, "any intermediate null values are
treated as numeric values of −999 and used in calculations" (T2). A −999 flowing into a training
matrix does not merely corrupt one sample — it relocates a cluster centroid and shifts every
boundary in the model.

**As-built.** `PRESENT-OK` — both engines drop incomplete rows rather than filling them:
`facies.rs:87`–`:97` skips any sample with a NaN in any present slot, and the Python path pools
only rows where the target and all features are finite (`ml.rs:1351` in the leaderboard, and the
equivalent test in the training assembly). The requirement is recorded to prevent an imputation
option being added without this constraint.

**Verified by.** SB-MLA-T55

#### SB-MLA-057 — A threshold value can never be confused with a missing value          [P1] [status: PRESENT-OK]

**Requirement.** A user-settable threshold or limit MUST NOT default to, or be storable as, any
value used as a missing-data sentinel. "No threshold set" MUST be a distinct state from any
numeric value.

**Rationale.** ML-16, proven from two independent sources: Techlog's `Min and Max threshold`
**default value is −9999**, which is also Techlog's own `MissingValue` sentinel (T3 for the
default, T1 for the sentinel from the shipped Python package). A user-set threshold of exactly
−9999, a curve legitimately carrying −9999, and "no threshold set" are three states that are
indistinguishable. The dossier records this as the strongest single argument in the corpus for
SandiBumi's separate-null-flag design, and it is a vendor defect proven rather than inferred.

**As-built.** `PRESENT-OK` (2026-08-07) — enforced in `P()`, which is the one door every parameter comes through, so it cannot be forgotten by the next parameter somebody adds. "No value" was already a distinct state (it returns the declared default and is recorded as defaulted), so a missing-data sentinel arriving as a value can only be a mistake: `NULL_SENTINELS` and NaN are refused by name. Worth refusing rather than tolerating because these COMPUTE — `-999.25` as a DBSCAN `eps` returns one enormous cluster and no error at all.

**Verified by.** SB-MLA-T56

### Group F — The Tier-C boundary

#### SB-MLA-058 — Tier-C capabilities are named, never approximated          [P1] [status: PRESENT-OK]

**Requirement.** SandiBumi MUST NOT implement, approximate, reverse-engineer or reconstruct any
capability on the Tier-C register, and MUST NOT specify a requirement that is a Tier-C capability
under another name. Where a Tier-C capability serves a real user need, SandiBumi MAY ship a
**design-around** derived from primary sources SandiBumi holds, and such a feature MUST be labelled
as a design-around with its own citations.

**Rationale.** `CONTRACT.md` §2.2. The in-domain register items are Experienced Eye / EEFS, Domain
Transfer Analysis, the shipped neural-network weight files, and the Textural Facies tile encoding.
Describing what these are for is competitive intelligence and is in the vendor's own marketing;
describing how they work is not available to this project on any framing.

**As-built.** `PRESENT-OK` — nothing in the tree implements or approaches any register item. The
requirement exists so that the boundary is a stated product constraint rather than an accident of
what has been built so far.

**Verified by.** SB-MLA-T57

#### SB-MLA-059 — The user need behind a Tier-C capability may be served by an independently derived feature          [P3] [status: ABSENT]

**Requirement.** Where the register records a user need served by a Tier-C capability, SandiBumi
MAY ship a feature meeting that need provided it is derived entirely from primary sources
SandiBumi holds, is labelled a design-around, and carries those sources in its documentation and
in the provenance of anything it produces.

**Rationale.** `CONTRACT.md` §2.2 explicitly permits this and the dossier records that the
**user-needs** for every in-domain register item are already catalogued with named Tier-B
design-arounds. This chapter deliberately does not specify any of those design-arounds: the ones
in this domain would need a primary-source acquisition that has not happened (§7), and specifying
a design-around before its sources are in hand is how a reconstruction gets written by accident.

**As-built.** `ABSENT` — no design-around is specified or built.

**Verified by.** SB-MLA-T57. Refusals R-1, R-2, R-5.

#### SB-MLA-060 — No vendor model or weight file is read, converted or imported          [P0] [status: PRESENT-OK]

**Requirement.** SandiBumi MUST NOT read, parse, convert, or import any vendor-trained model file,
neural-network weight file, or tile-encoding table. Model interchange with an incumbent, where
offered at all, MUST be limited to the *outputs* the vendor exports as ordinary curves.

**Rationale.** `CONTRACT.md` §2.1 and §2.2. Reading a weight file to apply it is using the
capability; reading it to understand it is reconstruction. Neither is available. This is P0 because
it is a boundary that would be crossed for an entirely reasonable-sounding product reason — "let
the customer keep using the model they already trained" — and the cost of crossing it is not
recoverable.

**As-built.** `PRESENT-OK`, and LOCKED as of 2026-08-07. The only model format read anywhere is
SandiBumi's own joblib blob from its own `ml_models` table; no vendor format is parsed. Since the
requirement was already satisfied, what it needed was not a fix but a test that fails the build if
the boundary is ever crossed — this is a P0 precisely because it would be crossed for a
reasonable-sounding reason, and a boundary held only by good intentions is held until the first
customer asks.

The lock checks three doors. A **dependency** that can parse a model artifact is the widest one, and
the requirement's own test text says so ("a new dependency … fails the check") — a crate added for
some other reason brings the capability with it whether or not anything calls it. A **Python import**
is the same door on the far side of the subprocess boundary, where `cargo` cannot see it. And the
**source of the bytes** is the invariant itself: `joblib.load(_io.BytesIO(blob))` deserializes a
buffer handed to the runner on stdin, and no runner calls `open(` at all, so there is no path for a
file to enter. A fourth check refuses a vendor model extension named anywhere in the source, because
a file-dialog filter is how the first reader arrives — the parser gets written because the picker
already offers it.

Interchange with an incumbent stays available exactly where the requirement allows: the vendor's
*outputs*, exported as ordinary curves, come in through LAS and DLIS like any other log.

**Verified by.** SB-MLA-T57 — `ml::tests::no_code_path_reads_a_vendor_model_or_weight_file`
(needs no Python and no network, so it fails the green gate).

### Group G — Platform and scale

#### SB-MLA-061 — A missing interpreter is a named, actionable failure          [P1] [status: PRESENT-OK]

**Requirement.** Where an ML capability is unavailable because a required runtime component is
missing, the product MUST state which component, name the interpreter it inspected, and give the
exact command that would fix it. The capability MUST NOT be silently hidden, and the capabilities
that do not need that component MUST remain available.

**Rationale.** `SB-CORE-002`, applied to a deployment condition. A customer machine without a
suitable interpreter loses regression, classification, PCA, t-SNE, the leaderboard and model
persistence, while keeping the native facies, HFU and Lorenz paths — a partial product, which is
the right outcome, provided the partition is explained rather than experienced.

**As-built.** `PRESENT-OK`, and notably well done. `python_engine.rs:177` (`find_python`) searches
`SANDIBUMI_PYTHON`, then a legacy variable honoured silently but "never named in a message", then
recent per-user installs, then `PATH`, taking the first interpreter that can `import numpy` and
caching it for the session. The failure message names the fix
(`python_engine.rs:47`–`:48`), `ml.rs` adds "scikit-learn is not installed for this Python - run:
pip install scikit-learn", and the joblib absence is reported as a model-save failure with its
cause rather than as a failed run (`ml.rs:713`–`:718`), because the predictions are already
correct and on disk. The requirement is recorded to protect this behaviour.

**Verified by.** SB-MLA-T58

#### SB-MLA-062 — A long fit does not hold the global write lock          [P1] [status: PRESENT-DIVERGENT]

**Requirement.** No ML operation may hold the global database lock across a subprocess call or any
other unbounded wait. Where results must be written under the lock, the lock MUST be acquired after
the computation and released between wells.

**Rationale.** `SB-CORE-032` (global-lock hold time) and `SB-CORE-034` (interactive
responsiveness). A blocking subprocess is exactly the unbounded wait that requirement exists for,
and an ML fit over a portfolio is among the longest operations the product performs.

**As-built.** `PRESENT-DIVERGENT` — the fit itself is correctly outside the lock: `exec_ml_full` is
called at `ml.rs:614` and the connection is not acquired until `ml.rs:630`. The write-back loop
then holds that single lock across **every apply well** (`ml.rs:630`–`:706`), including the
per-well `create_log_set` and versioned write at `ml.rs:676`–`:677`, so the lock is held for the
whole write phase rather than per well. The leaderboard is clean — its lock scope closes at
`ml.rs:1360` before the Python call.

**Verified by.** SB-MLA-T59

#### SB-MLA-063 — Every capacity cap is a declared limit, not a silent truncation          [P2] [status: PRESENT-OK]

**Requirement.** Every internal cap on work — evaluated combinations, sample counts, cluster
counts, input curve counts — MUST be reported when it binds, naming the requested quantity, the cap
and what was dropped.

**Rationale.** `SB-CORE-002`. A leaderboard that quietly evaluates the first 80 of 300 combinations
ranks a subset while presenting as a ranking, and the dropped combinations are ordered by the
algorithm list rather than by anything meaningful.

**As-built.** `PRESENT-OK` (2026-08-07) — the outstanding silent clamp was `facies.rs`'s K, and it now REFUSES above 12 rather than clamping. A module returns curves and has no channel to carry a warning, so a clamp there could only ever be silent — and silently returning 12 classes to somebody who asked for 20 hands them a facies scheme with a different number of classes than the one they designed, after which every count, legend and crossplot downstream describes a scheme nobody chose. This is the same call this requirement already praises in the t-SNE limit. The cap itself stands: 12 is the palette length shared with `plotCanvas.ts`, past which two facies print the same colour. `MAX_COMBOS` and the t-SNE limit were already reported; the silhouette subsample now declares its cap too (SB-MLA-020).

**Verified by.** SB-MLA-T60

#### SB-MLA-064 — The model registry lists without materialising artifacts          [P2] [status: PRESENT-OK]

**Requirement.** Listing stored models MUST NOT load their serialised bytes.

**Rationale.** `SB-CORE-030` (portfolio scale). A model blob is a compressed pickle of a fitted
ensemble and can be large; a registry that materialises every one to draw a list scales with total
model size rather than model count.

**As-built.** `PRESENT-OK` — `db.rs:2667` (`list_ml_models`) is documented as never selecting the
blob column, and `listing_models_never_carries_their_bytes` (`ml.rs:1764`) pins it as an executing
test.

**Verified by.** SB-MLA-T61

#### SB-MLA-065 — A portfolio-scale ML run is bounded, cancellable and honestly reported          [P2] [status: PARTIAL]

**Requirement.** An ML run over a well portfolio MUST report per-well progress, MUST remain
cancellable during every phase that can be cancelled, MUST state the phases that cannot, and MUST
report a per-well outcome that distinguishes success, no-usable-samples, and failure.

**Rationale.** `SB-CORE-030`, `SB-CORE-034`, `SB-CORE-036`. This is where the domain meets the
portfolio-scale requirements, and the distinction in the last clause matters: a well that produced
no predictions because it had no complete samples is a data finding, not a failure, and conflating
them makes a run over eighty wells unreadable.

**As-built.** `PARTIAL` — the per-well result carries `rows_predicted` and an optional error
(`ml.rs:688`–`:692`), the zero-row case is distinguished as `Warned` with "no complete samples in
this well" (`ml.rs:681`–`:686`), and cancellation is handled per well (`ml.rs:639`–`:649`). What is
missing is the log-set-level record of an incomplete run (`SB-MLA-017`) and any bound on the fit
phase itself.

**Verified by.** SB-MLA-T17, SB-MLA-T18

---

## 5. Parameters

**How to read this table.** Three kinds of row appear and they are not interchangeable.

- **SandiBumi's own shipped constants**, read at source in this pass and cited `file.rs:line`
  (T1). These are what the product does today. Several of them are the subject of a §4 requirement
  precisely *because* they ship — `RESTARTS` and `n_init` are both cited, and `SB-MLA-023` exists
  because two cited values for one method is one too many.
- **Vendor defaults for capabilities SandiBumi has not built.** These are individual documented
  defaults with named page-level sources, not tabulated chart data — carrying them is what
  `CONTRACT.md` §2 requires and is unrelated to the §2.1 transcription prohibition. Each is
  adoptable **on build** and must travel with its source string into the configuration file.
- **Absences.** Fifteen rows read `ABSENT` in one of its three contract forms. That is the correct
  outcome, not a research shortfall. Three distinct causes are
  distinguished in the Source column: (i) the vendors disagree and no adjudication is defensible
  (§12.2's standing decision); (ii) **Geolog states no default at all** across most of its Facimage
  suite — about eighteen parameters behind one live-session acquisition (ML-8); (iii) the only
  value available is an **IP documentation screenshot**, which is not a factory default (G-9.2).
  Cause (iii) rows are marked `NON-ADOPTABLE — cited for verification` rather than absent, because
  a value does exist and a reviewer needs to be able to check it.

`SHR`-owned constants appear here for cross-reference only and are marked as such; this chapter
does not source them.

| Parameter | Symbol | Value | Unit | Source | Tier |
|---|---|---|---|---|---|
| **SandiBumi as-built — clustering engines** |
| Electrofacies k-means restarts (native) | `RESTARTS` | **8** | count | `facies.rs:23` — SandiBumi's own; **conflicts with the Python engine's 10, see `SB-MLA-023`** | T1 |
| Electrofacies Lloyd iteration cap (native) | `MAX_ITERS` | **100** | count | `facies.rs:24` — SandiBumi's own; **conflicts with scikit-learn's 300, see `SB-MLA-023`** | T1 |
| k-means restarts (Python engine) | `n_init` | **10** | count | `ml.rs:163` — SandiBumi's own | T1 |
| k-means iteration cap (Python engine) | `max_iter` | **300** | count | `ml.rs:163` — **not set by SandiBumi**; scikit-learn's `KMeans` default, inherited | T1 |
| Facies count, native modules | `K` | **5** | count | `facies.rs:40`, `facies.rs:178` — SandiBumi's own; independently corroborated by Techlog's stated default of 5 in two modules (T3), which SandiBumi did not cite when choosing it | T1 |
| Facies count range, native modules | `K` | **2 … 12** | count | `facies.rs:40`; enforced at `facies.rs:77` | T1 |
| GMM variance floor | `VAR_FLOOR` | **1e-4** | (standardised curve units)² | `facies.rs:215` — SandiBumi's own numerical guard; see `SB-MLA-015` | T1 |
| GMM EM convergence tolerance | — | **1e-6 × m** | log-likelihood | `facies.rs:289`, `m` = sample count | T1 |
| Feature scaling, native modules | `OPT_STANDARDIZE` | **`ZSCORE`** (alt. `NONE`) | — | `facies.rs:42`, `facies.rs:180`; population SD, `facies.rs:119` | T1 |
| Feature scaling, Python engine | `standardize` | **on** | bool | `ml.rs:67`; scaler fitted on train for supervised, on apply for unsupervised (`ml.rs:68`) | T1 |
| Random seed, Python engine | `seed` | **42** | — | `ml.rs:64`; frontend supplies it explicitly at `src/ui/mlDialog.ts:280` | T1 |
| Random seed, native modules | `SEED` | **7** | — | `facies.rs:41`, `facies.rs:179`, fallback at `facies.rs:80` — **two values for one concept, see `SB-MLA-024`** | T1 |
| Mask exclusion convention | — | value **exactly 1.0** excludes | — | `ml.rs:1348` and the training assembly; house convention, corroborated by Techlog HRA's optional bad-hole flag input (T3) | T1 |
| Cluster id ordering | — | ascending mean of the first supplied curve | — | `facies.rs:409`–`:430`, `ml.rs:181`–`:185`; corroborated by IP 2018's `Sort index` control (T2) | T1 |
| **SandiBumi as-built — supervised estimators** (the `SB-MLA-026` divergence set) |
| Random forest trees | `n_estimators` | **200** | count | `ml.rs:87` (run) and `ml.rs:1136` (leaderboard) — **agree** | T1 |
| Random forest depth | `max_depth` | **0 → unbounded** | count | `ml.rs:88` (run); leaderboard sets none, same effect | T1 |
| Gradient boosting trees | `n_estimators` | **300** | count | `ml.rs:93` (run, XGBoost branch); `ml.rs:1140` (leaderboard) — agree on the XGBoost branch only | T1 |
| Gradient boosting learning rate | `learning_rate` | **0.1** | — | `ml.rs:94`, `ml.rs:1140` | T1 |
| Gradient boosting depth | `max_depth` | **4** | count | `ml.rs:95`, `ml.rs:1140` | T1 |
| Gradient boosting fallback iterations | `max_iter` | **300** (run) / **100** (leaderboard) | count | `ml.rs:98` against `ml.rs:1143`, which constructs `HistGradientBoostingRegressor(random_state=seed)` bare — **divergent, see `SB-MLA-026`** | T1 |
| Gradient boosting fallback depth | `max_depth` | **4** (run) / **None** (leaderboard) | count | `ml.rs:100` against `ml.rs:1143` — **divergent** | T1 |
| Support-vector regularisation | `C` | **10.0** | — | `ml.rs:105`, `ml.rs:135`, `ml.rs:1146`, `ml.rs:1156` | T1 |
| Support-vector epsilon | `epsilon` | **0.1** | — | `ml.rs:105`, `ml.rs:1146` | T1 |
| MLP hidden geometry | `hidden` | **(64, 32)** | nodes | `ml.rs:108`, `ml.rs:1149`; SandiBumi's own, **not** IP's | T1 |
| MLP training iterations | `max_iter` | **500** | count | `ml.rs:110`, `ml.rs:1149`; SandiBumi's own — IP's own value is contested (G-6.1) | T1 |
| Polynomial degree | `degree` | **1** | — | `ml.rs:113`; **no leaderboard counterpart — see `SB-MLA-026`** | T1 |
| k-NN neighbours (classification) | `n_neighbors` | **7** | count | `ml.rs:138`, `ml.rs:1159` | T1 |
| Logistic regularisation / iterations | `C`, `max_iter` | **1.0**, **1000** | — | `ml.rs:147`, `ml.rs:1168` | T1 |
| DBSCAN radius / minimum points | `eps`, `min_samples` | **0.5**, **10** | standardised units, count | `ml.rs:174` | T1 |
| PCA components | `n_components` | **3** | count | `ml.rs:205` | T1 |
| t-SNE perplexity | `perplexity` | **30.0**, capped at `(n−1)/3` | — | `ml.rs:213` | T1 |
| **SandiBumi as-built — gates, caps and diagnostics** |
| Minimum labelled training samples | — | **10** | count | `ml.rs:571` | T1 |
| Minimum samples for run-path CV | — | **30** | count | `ml.rs:76` | T1 |
| Minimum samples for the leaderboard | — | **20** | count | `ml.rs:1362` | T1 |
| Leaderboard folds | `folds` | **5**, clamped **2 … 10** | count | `ml.rs:1403` | T1 |
| Leaderboard combination cap | `MAX_COMBOS` | **80** | count | `ml.rs:1299`; reported when it binds (`ml.rs:1394`) | T1 |
| Permutation-importance repeats | `n_repeats` | **5** | count | `ml.rs:1219` | T1 |
| Silhouette subsample cap | — | **5000** | count | `ml.rs:193`; **not reported, see `SB-MLA-020`** | T1 |
| t-SNE sample cap | — | **20000** | count | `ml.rs:210`; refuses rather than truncates | T1 |
| Core-plug to log-sample match tolerance | `CORE_MATCH_TOL_M` | **1.0** | m | `facies_tie.rs:144` — SandiBumi's own | T1 |
| Lorenz auto-`K` marginal-gain threshold | `AUTO_K_TOL` | **0.02** | fraction of single-segment SSE | `lorenz.rs:33` — SandiBumi's own, rationale stated in the source comment | T1 |
| Lorenz auto-`K` upper bound | `AUTO_K_MAX` | **12** | count | `lorenz.rs:35` — SandiBumi's own; matches the HFU cap | T1 |
| Facies tie-in acceptance purity | — | **`ABSENT — ships with no default`** | fraction | The method note this module implements states "accept … above a threshold" and gives no value; no source in the corpus states one. See `SB-MLA-052` | — |
| **Fuzzy (Cuddy) — not built; adopt on build** |
| Bin count | `fuzzy.n_bins` | **10** | count | Geolog V14 `PT15_Facimage/fuzzy_hc.2.08.html` — `NBINS` default 10, "recommended for most applications" | T3 |
| Bin count range | `fuzzy.n_bins` | **2 … 100** | count | IP 2025 `statisticalcurveprediction.htm`; unchanged from IP 2018 | T2 |
| Minimum calibration samples per bin | `fuzzy.min_samples_per_bin` | **30** | count | Geolog V14 `PT15_Facimage/fuzzy_hc.2.08.html` — "at least 30 core plug samples" per bin per interval | T3 |
| Maximum input curves | `fuzzy.max_input_curves` | **20** | count | IP 2025 `statisticalcurveprediction.htm`; IP 2018 was 8; Geolog caps at 6 (`fuzzy_hc.2.06.html`) | T2 |
| Maximum facies codes | `fuzzy.max_facies_codes` | **10** | count | Geolog V14 `PT15_Facimage/fuzzy_hc.2.04.html`; matches IP 2025's classification-NN cap of 10 | T3 |
| Percentile error band | `fuzzy.percentile_error Er` | **`ABSENT — ships with no default`** | percentile | IP 2025 `_flclip0010.png` shows 25 — **documentation screenshot, not a verified factory default** (G-9.2 open) | T2 |
| Bin-count weighting | `fuzzy.weight_bin_by_count` | **`ABSENT — ships with no default`** | bool | IP 2025 prose says default selected, panel shows cleared — G-6.5 / G-9.10 open. The `√n_b` term is **inside** the printed `P(C_b)`, so "off" deviates from Cuddy as printed. See `SB-MLA-040` | T2 |
| Within-bin regression refinement | `fuzzy.within_bin_regression` | **off** | bool | Geolog V14 `fuzzy_hc.2.08.html` describes it as unconditional; **SandiBumi ships it off** because it privileges `CURVE1` and IP's Cuddy has no such step | T3 |
| Fuzzy c-means exponent | `QQ` | **`ABSENT — ships with no default`** | — | Techlog states no default anywhere, and its prose contradicts the printed exponent's direction (ML-3). The method is quarantined; see Refusal R-3 | T3 |
| **Self-organising map — not built; adopt on build** |
| Initial neighbourhood radius | `som.sigma_0` | **grid_width / 2** | nodes | IP 2025 `som.htm` — "initialised so that it begins as half of the map grid width" | T2 |
| Initial learning rate | `som.learning_rate_0` | **`ABSENT — ships with no default`**; range `(0, 1)` | — | IP 2025 `som.htm` states the range; the value 0.1 is `somclip0016.png`, a **screenshot** (G-9.2 open) | T2 |
| Total training iterations | `som.total_iterations` | **`ABSENT — required, no default`** | count | **SandiBumi's own parameterisation.** IP's printed `λ = t / log σ₀` is degenerate (§2.2, G-6.2 open); the total-iteration form is the only non-degenerate reading. IP's panel value 60000 is a screenshot. See `SB-MLA-041` | T2 |
| Maximum map width | `som.map_width_max` | **200** | nodes | IP 2025 `som.htm`; unchanged from IP 2018 | T2 |
| Maximum input curves | `som.max_input_curves` | **8** | count | IP 2025 `som.htm` — explicitly **not** raised to 20 in 2025, unlike Fuzzy / NN / PCA | T2 |
| Map geometry | `som.geometry` | `square \| hexagonal \| spherical` | enum | IP 2025 `som.htm` — border effect: square worst, hexagonal reduced, spherical none | T2 |
| Map dimensionality | `som.dims` | `1D \| 2D` | enum | Geolog V14 `facimage_03_generate_hc.3.7.html` — "Specifying 1 for the X or Y Neurons builds a SOM 1D model" | T3 |
| Calibration weighting | `som.calibration_weight` | **1 / d²** | — | IP 2025 `som.htm` — "a weighting equal to the inverse of the square of the Euclidean distance"; same rule stated for Cluster Analysis calibration | T2 |
| Map size, Techlog | — | **`NON-ADOPTABLE — cited for verification`** | nodes | Techlog states **two** different defaults in two of its own modules — Ipsom "a 10\*10 default size (100 nodes)" against TechCore Petrophysical groups "Height and Width, Default value 7" (49 nodes) — neither page acknowledging the other (ML-15). Recorded so neither is ever cited bare | T3 |
| Neuron counts, Geolog | `X`/`Y Neurons`, `Shakings`, `Iterations` | **`ABSENT — ships with no default`** | count | Geolog Facimage states none (ML-8); closing this needs a live session, escalation E-5 | T3 |
| **Partitional and hierarchical clustering — vendor values** |
| First-stage cluster count | `cluster.k_stage1` | **15 … 20** | count | IP 2025 `cluster_analysis.htm` — "15 to 20 clusters would appear to be a reasonable number for most data sets"; unchanged from IP 2018 | T2 |
| Consolidated cluster count | `cluster.k_consolidated` | **4 … 5** | count | IP 2025 `cluster_analysis.htm`. Delivered-work precedent independently reached 7 facies groups and 4 rock-quality classes on two projects — precedent, **not** a default | T2 (precedent T4/PKB) |
| Cluster count, Techlog | `cluster.k_default` | **5** | count | Techlog 2018.2 `concept/outputs-displays-multiple-realizations-hra.html` — "Number of clusters", Default 5, "the main user parameter"; **corroborated by a second independent module**, `concept/techcore-petrophysical-petrophysical-groups.html`, "Groups count", Default value 5 | T3 |
| Restart count | `cluster.n_runs` | **50** | count | Techlog 2018.2 `concept/outputs-displays-multiple-realizations-hra.html` — "Number of runs", Default 50 | T3 |
| Seeding subset fraction | `cluster.seed_subset_fraction` | **0.10** | fraction | Techlog 2018.2 `concept/outputs-displays-multiple-realizations-hra.html` — "a random 10 % subset of the data is seeded…" | T3 |
| PCA variance retained before clustering | `cluster.pca_variance_cutoff` | **0.95** | cumulative fraction | Techlog 2018.2 `concept/outputs-displays-multiple-realizations-hra.html` — "PCA Variance", Default 0.95 | T3 |
| Linkage rule | `cluster.linkage` | **ward** | enum | **Three-vendor default.** IP 2025 `cluster_analysis.htm` "The default method Minimize the within-cluster sum of squares distance" (T2); Geolog V14 `facimage_03_generate_hc.3.6.html` "WARD (default)" (T3); Techlog 2018.2 `concept/techcore-petrophysical-petrophysical-groups.html` Default value column, `HC > Aggregation method` **and** `SOM > Aggregation method` (T3). **Scoping caveat: Techlog's Ipsom HC page states NO default** — only "Ward method: This is the most used method", which is popularity. Do not cite the Ipsom page for this value | T2 + T3 |
| Distance metric | `cluster.metric` | **euclidean** | enum {euclidean, variance, mahalanobis} | Geolog V14 `facimage_02_setup_hc.2.8.html` — "Euclidean (default)". **Vendor spells it `Euclidian`**; that spelling is an input alias, never a key (`SB-MLA-036`) | T3 |
| Normalisation basis | `cluster.normalize_using` | **data_range** | enum {data_range, plot_limits, standard_deviation, histogram} | Geolog V14 `facimage_02_setup_hc.2.8.html` — "Data Range (default)". IP's z-score is `standard_deviation` in this vocabulary and is IP's **only** option | T3 |
| Maximum input curves | `cluster.max_input_curves` | **8** | count | IP 2025 `cluster_analysis.htm`; unchanged 2018 → 2025 | T2 |
| Maximum output cluster sets | `cluster.max_output_sets` | **7** | count | IP 2025 `cluster_analysis.htm` | T2 |
| DYNCLUST kernel counts | `NBCR`, `NBCM` | **`ABSENT — ships with no default`** | count | Geolog Facimage states none (ML-8) | T3 |
| MRGC electrofacies bounds | — | **`ABSENT — ships with no default`** | count | Geolog Facimage states none for `Minimum`/`Maximum Number of Electrofacies`, `Number of Optimal Models`, `Initial Neurons for CFSOM` (ML-8). MRGC internals are not held (ML-6) | T3 |
| **Propagation and prediction — vendor values** |
| Neighbours for log prediction | `knn.k_log_prediction` | **10** | count | Geolog V14 `facimage_05_using_hc.5.05.html` — "Nearest Neighbors: Default 10… Up to 50". Delivered-work precedent chose 2 twice, once explicitly "by trial and error" — precedent, **not** a default | T3 (precedent T4/PKB) |
| Maximum neighbours for facies propagation | `knn.k_facies_propagation_max` | **10** | count | Geolog V14 `facimage_05_using_hc.5.04.html` — "Up to 10 nearest neighbors can be specified" | T3 |
| Most-probable-facies logs emitted | `knn.n_most_probable_facies_logs` | **1** | count | Geolog V14 `facimage_05_using_hc.5.04.html` — "By default, this parameter is set to 1" | T3 |
| Barycenter class weighting | `barycenter.use_class_weight` | **No** | bool | Geolog V14 `facimage_05_using_hc.5.05.html` — "The default is No" | T3 |
| KNN distance-weight function and length scale | `w(d)`, `h` | **`ABSENT — ships with no default`** | — | **Geolog never prints the function.** "An exponential distance weighting" only; `exp(−d/h)` and `exp(−d²/h²)` are both consistent with the words and give materially different predictions, and `h` is exposed nowhere in the help set (ML-12). Must be SandiBumi's own and labelled as such — `SB-MLA-049` | T3 |
| STM accept confidence | `stm.accept_confidence` | **90** | % | Geolog V14 `facimage_03_generate_hc.3.9.html` — "The default 90 means 90 % confidence for membership in accept" | T3 |
| STM reject confidence | `stm.reject_confidence` | **95** | % | Geolog V14 `facimage_03_generate_hc.3.9.html` — "95 means 95 % confidence for membership in reject… the 90-95 % range is Ambiguous" | T3 |
| Sammon projection iterations | `sammon.iterations` | **2000** | count | Techlog 2018.2 `concept/geology-2d-3d-sammon-projection.html` — "Enter the number of iterations (2000 by default)" | T3 |
| Outlier tolerance | `outlier.tolerance_sd` | **2** | multiples of SD (dimensionless) | Techlog 2018.2 `concept/petrophysics-kmod-properties.html` — "a tolerance of 2, default value, gives 5 % of outliers"; **same default verbatim** on `concept/petrophysics-quality-log-outliers.html` (K.mod) and `concept/geology-outliers-quality-log-appearance.html` (Ipsom) — 3 pages, 2 modules. **Units trap:** `s` is the standard deviation per 2 of the 3 pages that name it; `concept/geology-quality-log-outliers.html` says "variance" and is the single dissenting page (ML-9). Field is named `tolerance_sd`, never `tolerance` — `SB-MLA-053` | T3 |
| Expected outlier fraction at `a` = 2 | `outlier.expected_fraction` | **~0.05** | fraction | Techlog 2018.2, all four outlier pages — "if `a` = 2, `N` is in general equal to 5 %". Stated by the vendor as an **expectation**, not an exact Gaussian identity; the Gaussian value at 2 SD is 0.0455. Recorded as the vendor's stated pairing, **not** as a derived constant | T3 |
| K.mod training hyperparameters | — | **`ABSENT — ships with no default`** | — | Techlog states no default for network geometry, learning cycles, learning rate or convergence criterion. The `6-6-6-1` notation is **illustrative, not a default** (ML-10, corrected 2026-08-06) | T3 |
| **Neural / supervised — vendor values** |
| Hidden layers | `nn.hidden_layers` | **1** | count | IP 2018 neural-networks page: "The number of Hidden layers = 1". **Tier-C-adjacent provenance:** carried only because it is a generic architectural fact, independently re-derivable; the vendor's neural engine and every shipped weight file are Tier C and are never used (Refusal R-1) | T2 |
| Training epochs | `nn.epochs` | **`ABSENT — ships with no default`** | count | G-6.1 open: IP prose says 1000, the shipped panel shows 100 — the vendor contradicts itself. SandiBumi ships its own 500 (`ml.rs:110`, listed above) and does not claim IP compatibility | T2 |
| Training restarts | `nn.training_passes` | **3** | count | IP 2025 `neural_networks.htm` — "The default value of 3 works well to stop" the net getting stuck; unchanged from IP 2018. Adopt as a restart count, **not** as an IP-compatibility claim | T2 |
| Cross-validation percentage | `nn.cross_validation_pct` | **`ABSENT — ships with no default`**; **0 disables** | % | IP 2025 `neural_networks.htm` for the 0-disables rule; the value 5 is `_nnclip00018.png`, a **screenshot** (G-9.2 open). IP **silently disables** CV under zonal averaging — SandiBumi refuses the combination loudly instead (`SB-MLA-034` rationale, §2.10) | T2 |
| Training zone count | `nn.training_zones` | **4 … 8** narrow zones | count | IP 2025 `neural_networks.htm` — "For most purposes a small number (4-8) of narrow zones is enough" | T2 |
| Classification category cap | `nn.classification_max_categories` | **10** | count | IP 2025 `neural_networks.htm`; unchanged from IP 2018 | T2 |
| Maximum input curves | `nn.max_input_curves` | **20** | count | IP 2025 `neural_networks.htm`; IP 2018 was 8 | T2 |
| Sensitivity dither | `nn.sensitivity_dither` | **`NON-ADOPTABLE — cited for verification`** (vendor value: 10 % of normalised range) | % | IP 2025 `_nnclip00018.png` — "Raw Sensitivity (dithered at 10 % of normalised data range)", a **screenshot-only** disclosure. SandiBumi ships permutation importance instead (`ml.rs:1219`); this is recorded for cross-tool comparison, not adopted | T2 |
| Normalisation scheme, IP neural | — | **`ABSENT — not documented by the vendor`** | — | G-9.5 open: IP's sensitivity readout references a "normalised data range" but the scheme is never stated. SandiBumi's answer is its own — `StandardScaler` fitted on the training matrix and persisted with the model (`ml.rs:68`, `ml.rs:229`–`:247`) | T2 |
| **Panel values that are not defaults** |
| IP SOM map width / spherical nodes / iterations / `L₀` | — | **`NON-ADOPTABLE — cited for verification`** (20 / 642 / 60000 / 0.1) | mixed | IP 2025 documentation screenshots — **not verified as factory defaults** (G-9.2 open). Any use must carry that exact caveat in the source string | T2 |
| IP cluster count panel value | — | **`NON-ADOPTABLE — cited for verification`** (15) | count | IP 2025 documentation screenshot (G-9.2 open). Distinct from the prose guidance of 15–20 above, which **is** a stated recommendation | T2 |
| IP fuzzy panel values | — | **`NON-ADOPTABLE — cited for verification`** (10 bins, `Er` = 25) | mixed | IP 2025 documentation screenshots (G-9.2 open). The bin count of 10 is separately supported by Geolog's stated `NBINS` default above, which is the row to cite | T2 |
| Spherical SOM valid node counts | — | **`ABSENT — ships with no default`** | count | Only 642 has been observed, from a screenshot; valid tessellation counts must be derived from the tessellation itself, never from IP (G-9.8 open) | T2 |
| **Engine-wide conventions** |
| Missing-value sentinel on write | — | **−999.25**, with an explicit `NULL.` line | — | `FINDINGS.md` §6 rule 6. On read, honour the declared null then screen −999 / −9999 / −99 as suspected undeclared nulls. IP uses −999; Techlog's `MissingValue` is −9999 (T1, shipped Python package); Geolog uses `MISSING` | T1 + T2 |
| Threshold sentinel collision | — | **prohibited** | — | Techlog's `Min and Max threshold` **default is −9999**, which is also its own `MissingValue` — three states made indistinguishable (ML-16). Recorded as a design-negative exemplar; see `SB-MLA-057` | T1 + T3 |
| Trigonometric units | — | radians internally; degrees only at a formula-language boundary, named explicitly | — | IP 2025 `user-definedformula.htm`: `SIN`/`COS`/`TAN` take **degrees** and `ASIN`/`ACOS`/`ATAN` return degrees, while FORTRAN user apps in the same product take **radians** (`compiler-information.htm`) — a factor of 57.2958, silent | T2 |
| **Cross-reference — owned by `SHR`, not sourced here** |
| RQI constant | `RQI_C` | **0.0314** | µm (for `k` in mD, `φ` in v/v) | `hfu.rs:22`. **`SHR` owns the source.** Listed because `SB-MLA-025` touches `hfu.rs` | T1 |
| Inverse permeability-transform constant | `PERM_C` | **1014.24** | — | `hfu.rs:24` (= 1/`RQI_C`²). **`SHR` owns the source.** | T1 |
| HFU cluster count cap | — | **12** | count | `hfu.rs:210`. **`SHR` owns the source.** | T1 |

**Row count: 105** (counted by parsing the table, not by estimate). Of those, **15** read `ABSENT`
in one of its three forms — `ships with no default`, `required, no default`, or
`not documented by the vendor` — **5** read `NON-ADOPTABLE — cited for verification`, and **3** are
`SHR` cross-references that this chapter does not source. The remaining 82 carry a value with a
checkable source: 44 of them are SandiBumi's own constants at `file.rs:line` (T1) and 38 are vendor
defaults at page-level citations (T2/T3).

---

## 6. Acceptance tests

Sixty-one tests. Each states its input, operation, expected result with tolerance, and the source
of the expectation. **Four are labelled `CHARACTERIZATION`** — they pin current behaviour whose
expected value has no external source, and under CONTRACT §6 they are kept as snapshots and
labelled as such rather than dressed up as correctness tests.

Where a test derives its expected value arithmetically, the arithmetic is shown so a reviewer can
check it without re-deriving it.

**A guard that applies to two of these tests and must not be lost.** `SB-MLA-T25` and
`SB-MLA-T26` come from **different IP worked examples and their numbers must never be crossed.**
`T25`'s eigenvector table implies `λ₁ = 4 × 0.4888 = 1.9552` and a GR loading on PC1 of
**0.39083**; `T26` uses `λ₁ = 2.24` and a GR loading of **0.592**. Both are internally consistent
and both verify, but they describe different data. A reviewer who feeds `T25`'s eigenvector into
`T26`'s `coord = e·√λ` rule gets wrong expected values and will "fix" correct code. **Keep them in
separate files with separate input blocks.**

### Group A tests — provenance and reproducibility

#### SB-MLA-T01 — the effective seed is recorded even when it was defaulted
**Input.** A regression request with `seed` omitted from the parameter map.
**Operation.** Run, then read the stored run record and the model row.
**Expected.** The recorded parameter set contains `seed = 42` and a flag marking it as defaulted,
plus the identifier of the default's source. Exact equality; no tolerance.
**Source.** `SB-MLA-001`; the effective value is `ml.rs:64`.

#### SB-MLA-T02 — the training log set is on the model
**Input.** Train a model with `input_set` set to a named, versioned log set.
**Operation.** Persist, then read the model row back.
**Expected.** The row names that log set and its version. Applying the model after that set is
superseded emits a warning naming the set. Exact equality.
**Source.** `SB-MLA-002`; the field's own doc comment on `MlRequest::input_set`.

#### SB-MLA-T03 — the training-row hash is stable and discriminating
**Input.** Two fits with identical configuration on identical data; then a third after one
training sample's target is changed by 1e−6.
**Operation.** Compare recorded training-matrix hashes.
**Expected.** Fits 1 and 2 hash identically; fit 3 differs. The well list, `n_train` and the
recorded parameter set are unchanged across all three — so the hash is the only field that
distinguishes them, which is the point.
**Source.** `SB-MLA-003`.

#### SB-MLA-T04 — the mask and its effect are recorded per well
**Input.** Three training wells, a mask curve set to exactly 1.0 over a known 12 % of samples in
one of them.
**Operation.** Train and persist.
**Expected.** The record names the mask curve and reports the excluded count per well, matching the
constructed count exactly. A run with no mask records the absence explicitly, not as a null field.
**Source.** `SB-MLA-004`; the mask convention at `ml.rs:1348`.

**Reading, stated (2026-08-07).** "Not as a null field" is taken to mean *not as a field a reader
cannot tell from one that was never written* — the as-built writes `"mask_curve": null` into the JSON
rather than omitting the key. The test asserts the literal text is present, so an omission would fail
it. A sentinel string such as `"none"` was rejected: it cannot be distinguished from a mask curve
somebody actually named `NONE`.

#### SB-MLA-T05 — the runtime record is complete
**Input.** Any successful supervised fit.
**Operation.** Read the model row.
**Expected.** Interpreter version, `numpy`, `scikit-learn`, `joblib` and — where the algorithm can
use it — `xgboost` are all present and non-empty. A missing component is recorded as absent, not
omitted.
**Source.** `SB-MLA-005`.

#### SB-MLA-T06 — a curve from a fitting run names its model
**Input.** A supervised run with `save_model_as` set.
**Operation.** Read the provenance of a curve the run wrote.
**Expected.** The provenance carries the persisted model's identifier. It matches the identifier a
subsequent apply run writes for the same model, so the two paths are indistinguishable in what they
record.
**Source.** `SB-MLA-006`; the apply-path form at `ml.rs:949`.

#### SB-MLA-T07 — a cited model cannot be deleted silently
**Input.** A model, a curve written by applying it, then a delete request.
**Operation.** Attempt the delete.
**Expected.** Refused, with a message naming the wells and curves that cite the model. A forced
delete records the event and marks the citing curves as having an unresolvable model reference.
**Source.** `SB-MLA-007`; `db.rs:2740` is the current unconditional behaviour.

#### SB-MLA-T08 — same inputs, same seed, byte-identical outputs, every algorithm
**Input.** A fixed multi-well fixture; every supported algorithm; a fixed seed.
**Operation.** Run twice from a clean state and compare outputs byte for byte.
**Expected.** Identical output curves, cluster identifiers, probability curves, principal-component
signs and every reported metric. **Byte equality, not a tolerance** — a tolerance here would hide
exactly the drift the test exists to catch.
**Source.** Dossier §5.3 `T-ML-SEED-1`; the differentiator argued in §3.7.

**Two divergences, both deliberate (2026-08-07).** The fixture is a *pooled matrix*, not a multi-well
DB fixture: the test drives `exec_ml` directly, because what is being pinned is the runner's
determinism and a DB round trip between the two runs would add a second thing that could differ.
`autoencoder` and `dbscan` are excluded and the reasons are recorded in the test — the first refuses
outright (PyTorch is not wired up), and the second's parameters are data-scaled, so on this fixture
it returns one cluster and would pass while proving nothing. Fifteen configurations remain.

#### SB-MLA-T09 — blind and training metrics are reported as a pair
**Input.** A three-well supervised fixture constructed so training and blind performance diverge.
**Operation.** Train, then read what the curve carries.
**Expected.** The curve carries a blind-well metric, its protocol and the held-out well count. **A
training-only report is a fail.** Where no blind evaluation ran, the curve says so explicitly and
carries no metric at all.
**Source.** Dossier §5.3 `T-ML-BLIND-1`, whose cautionary case is a delivered project at
correlation 0.99 training against 0.31–0.70 blind (PKB, T4).

#### SB-MLA-T10 — the report carries the ML provenance block
**Input.** A project whose reported curve set includes one model-derived curve.
**Operation.** Generate the report.
**Expected.** The report names the model, its algorithm, its ordered feature list, its
training well count, its training log set, its blind metric and the run date. A report generated
with the ML curve removed does not contain the block.
**Source.** `SB-MLA-010`; `report.rs` currently contains no ML reference.
**Divergence, deliberate.** This test was written expecting the block *inside* the methodology
section. As built it is its own section immediately after it, because the methodology table
describes the METHOD and this describes a specific fitted artifact — the same algorithm over two
sets of rock is two models, and a methodology row cannot say which one made this well's curve. The
substance of the check is unchanged.

#### SB-MLA-T11 — well roles are recorded, including the empty contributor
**Input.** Four training wells, one of which lacks the target curve; two apply-only wells.
**Operation.** Train and read the run record.
**Expected.** Three wells recorded as training with their sample counts, one recorded as
selected-but-contributing-nothing with the reason, two recorded as apply-only. The run-time warning
at `ml.rs:580`–`:587` is additionally present and its counts agree with the record.
**Source.** `SB-MLA-011`.

#### SB-MLA-T12 — a substituted algorithm is never recorded under the requested name
**Input.** A `gbdt` regression request on an interpreter without `xgboost`.
**Operation.** Run and read the model row.
**Expected.** The `algorithm` field records the estimator actually used, not `gbdt`; the requested
algorithm and the reason for substitution are recorded separately; the user is told before the run
completes. A second assertion: loading an artifact under a differing recorded runtime fails with a
message naming the differing component.
**Source.** `SB-MLA-012`; the current substitution is at `ml.rs:91`–`:102`.

### Group B tests — fail loud inward

#### SB-MLA-T13 — an unclusterable well fails rather than emitting an empty curve
**Input.** Two cases: (a) a well where every supplied slot curve is entirely missing; (b) a well
with 4 complete samples and `K = 5`.
**Operation.** Run the native electrofacies and GMM modules, and the Python clustering path.
**Expected.** All three refuse the well with a message naming the cause. **No all-missing output
curve is written and no run reports success.** Case (b)'s message states the sample count and the
requested cluster count.
**Source.** Dossier §5.3 `T-ML-EMPTY-1`; IP's own message for the same condition is "One or more of
the clusters had zero data points!" (T2). Current behaviour is `facies.rs:95`–`:97` returning
through `facies.rs:137`–`:139`.

#### SB-MLA-T14 — a reduced cluster count is reported
**Input.** A dataset with 6 distinct feature vectors and `K = 10`.
**Operation.** Run every clustering path.
**Expected.** Each reports the effective count (6), the requested count (10) and the reason.
Cluster identifiers are contiguous from 0 with no gap.
**Source.** `SB-MLA-014`; the behaviour to match is `hfu.rs:273`, `:289`–`:300`, `:314`–`:326`,
already pinned by `run_hfu_skips_invalid_and_notes_capped_k` (`hfu.rs:489`).

#### SB-MLA-T15 — a floored mixture component is reported
**Input.** A dataset containing a tight three-point cluster that forces the variance floor, with
`K` set high enough to isolate it.
**Operation.** Run the GMM path.
**Expected.** The result reports that the floor fired, on which component, and how many times. The
`FPROB` values for that component's members are flagged as unreliable.
**Source.** `SB-MLA-015`; `VAR_FLOOR = 1e-4` at `facies.rs:215`.

#### SB-MLA-T16 — convergence and exhaustion are distinguished
**Input.** Two fixtures: one that converges in fewer than ten EM iterations, one constructed to
exhaust the cap.
**Operation.** Run the GMM path on both.
**Expected.** Each result states which terminal condition it hit, the iteration count reached and
the final convergence measure. The two results are distinguishable from their reports alone.
**Source.** `SB-MLA-016`; the tolerance is `facies.rs:289`, the cap `facies.rs:24`.

#### SB-MLA-T17 — a cancelled run marks the log set
**Input.** An ML run over eight apply wells, cancelled after the third write-back.
**Operation.** Inspect the output log set and the per-well results.
**Expected.** Wells 1–3 carry curves; wells 4–8 are reported cancelled; **the log set itself
records that it is incomplete**, and is distinguishable from a complete run over three wells by
that record alone.
**Source.** `SB-MLA-017`; the per-well half already works at `ml.rs:639`–`:649`.

#### SB-MLA-T18 — the fit phase is presented as non-cancellable          `CHARACTERIZATION`
**Input.** A supervised run large enough for the fit to dominate.
**Operation.** Observe the progress model during the fit phase.
**Expected.** The fit is shown as an indeterminate, non-cancellable phase; the write-back phase is
shown as cancellable and per-well. **Labelled `CHARACTERIZATION`**: the expected behaviour is the
current implementation's own stated design (`ml.rs:601`–`:603`, `:636`–`:638`) and has no external
source. It is kept because the natural "improvement" — always enabling the cancel control — would
be a regression into dishonesty.
**Source.** Current behaviour; no external source.

#### SB-MLA-T19 — a collapsed cross-validation reports no score
**Input.** A leaderboard run over three training wells with a mask that empties two of them.
**Operation.** Run.
**Expected.** No `score` or `score_std` is populated for any row; the result states that blind-well
cross-validation was not possible and why. **A populated score accompanied by a warning is a fail.**
**Source.** `SB-MLA-019`; the current fallback is `ml.rs:1171`–`:1176` with the note at
`ml.rs:1412`–`:1429`.

#### SB-MLA-T20 — a subsampled metric carries its sample count
**Input.** A clustering run over 12,000 complete samples.
**Operation.** Read the reported cluster-quality metric.
**Expected.** The metric carries its sample count (5,000) and a flag that it was subsampled, under
a name distinct from the full-population metric. Two runs with the same seed report the same value.
**Source.** `SB-MLA-020`; the cap is `ml.rs:193`.

#### SB-MLA-T21 — noise is distinguishable from missing
**Input.** A DBSCAN run over a well with (a) samples the algorithm assigns to noise and (b)
samples excluded because an input was missing.
**Operation.** Read the output curve.
**Expected.** The two sets are distinguishable in the output. The aggregate noise percentage
already reported at `ml.rs:187`–`:188` agrees with the per-sample count.
**Source.** `SB-MLA-021`.

#### SB-MLA-T22 — the ordered-feature refusal runs on the default gate
**Input.** A saved model fitted on `[GR, RHOB, NPHI]`; an apply request supplying
`[RHOB, GR, NPHI]`; and separately, a clean apply to an unseen well.
**Operation.** Run both on the project's default test gate.
**Expected.** The reorder is refused with a message naming both orders; the clean apply succeeds
without refitting and reproduces the fitted model's predictions exactly. **Both must execute on the
default gate, not under `#[ignore]`.**
**Source.** `SB-MLA-022`; the behaviour is `ml.rs:294`–`:297`, the currently ignored tests are
`ml.rs:1782` and `ml.rs:1832`.

### Group C tests — one name, one method

#### SB-MLA-T23 — the two k-means engines agree on a shared fixture
**Input.** A fixed 2,000-sample, three-feature fixture with `K = 6` and a fixed seed, structured so
the optimum is not trivially separable.
**Operation.** Cluster with the native engine and with the Python engine; compare labellings after
applying the shared ordering rule.
**Expected.** Identical labels for every sample. **A divergence fails the build**, and the failure
message names the differing constants.
**Source.** `SB-MLA-023`; `facies.rs:23`–`:24` against `ml.rs:163`.

#### SB-MLA-T24 — one seed default, and identifier stability across seeds
**Input.** (a) Every module and engine, parameters unspecified. (b) The same clustering fixture run
under three different seeds.
**Operation.** Read the effective seed in (a); compare cluster identifiers in (b).
**Expected.** (a) One value everywhere. (b) Cluster **identifiers** remain stable under the
ordering rule even where memberships differ, and the run reports the membership disagreement rate.
**Source.** `SB-MLA-024`; dossier §5.3 `T-ML-SEED-2`; the current split is `ml.rs:64` against
`facies.rs:80`.

#### SB-MLA-T25 — IP's PCA worked example reproduces
**Input.** IP's published means and standard deviations — ρb 2.63535 / 0.15722, Δt 64.53714 /
9.54118, φN 0.1321 / 0.07116, GR 79.13452 / 43.40833 — with eigenvector 1
`(−0.29412, 0.56258, 0.66652, 0.39083)`.
**Operation.** Standardise and form `PC1 = Σ_j e_{1,j}·z_j`.
**Expected.** `PC1` matches the printed linear form to **1e−6**. The variability percentages
48.88 / 37.51 / 11.06 / 2.54 sum to **99.99**, and the first two sum to **86.39**.
**Source.** IP 2025 `principal_component_analysis.htm` (T2), arithmetic independently verified in
the dossier. **Do not cross with `SB-MLA-T26`** — see the guard above.

#### SB-MLA-T26 — the correlation-circle coordinates reproduce
**Input.** Eigenvalues `λ₁ = 2.24`, `λ₂ = 1.474`; loadings GR `(e₁, e₂) = (0.592, 0.285)`, DT
`(e₁, e₂) = (−0.364, 0.665)`. The fixture carries its own loadings and is runnable standalone.
**Operation.** Apply `coord = e × √λ`.
**Expected.** GR `[0.886, 0.346]`, DT `[−0.545, 0.807]`, to **3 decimal places**. Intermediates for
debugging: `√2.24 = 1.49666`, `√1.474 = 1.21408`.
**Source.** IP 2025 `principal_component_analysis.htm` correlation-circle worked example (T2), all
four products independently re-derived; the `√eigenvalue` rule is **independently stated by
Techlog** in `utility-techstat-principal-components-analysis-pca.html` (T3).

#### SB-MLA-T27 — the leaderboard and the run construct identical estimators
**Input.** Every supported algorithm, at defaults and with a non-default hyperparameter set —
including `degree = 3` for the linear family — on an interpreter with `xgboost` and again on one
without.
**Operation.** Construct the estimator through both code paths and compare the full parameter
dictionaries.
**Expected.** Identical for every algorithm in every configuration. Where a hyperparameter cannot be
honoured in evaluation, that row is marked and excluded from the ranking rather than evaluated
differently.
**Source.** `SB-MLA-026`; the current divergences are `ml.rs:113`–`:119` against `ml.rs:1150`,
`ml.rs:98`–`:101` against `ml.rs:1143`, and `ml.rs:135` against `ml.rs:1156`.

#### SB-MLA-T28 — every score names its protocol
**Input.** A supervised run and a leaderboard run over the same wells.
**Operation.** Read every reported metric from both.
**Expected.** Each carries its protocol: training-set, or cross-validation with its splitter,
shuffle state and fold count, or blind-well with its group count. Two metrics from different
protocols do not share a display name that differs only by the metric.
**Source.** `SB-MLA-027`; the two current protocols are `ml.rs:75`–`:81` and `ml.rs:1175`–`:1176`.

#### SB-MLA-T29 — the scaler is fitted inside the fold
**Input.** A three-well fixture in which one well's feature distribution is shifted far from the
other two.
**Operation.** Run the leaderboard, and separately compute the same evaluation with the scaler
fitted per fold.
**Expected.** The two blind scores differ, and the product reports the **per-fold** value. A
direct structural assertion accompanies it: no transform is fitted on data outside the fold's
training partition. The magnitude of the difference is fixture-dependent and is not asserted;
what is asserted is that the pipeline order is fit-inside-fold.
**Source.** `SB-MLA-028`; the current fit-before-split is `ml.rs:1130` against the splitter at
`ml.rs:1175`.

#### SB-MLA-T30 — a class curve names its engine
**Input.** One well, clustered by all three engines.
**Operation.** List the resulting curve mnemonics.
**Expected.** Three distinct mnemonics, each identifying its engine. No two engines can write the
same mnemonic in one well. The confusion tool can name which engine produced each of the two
curves it is comparing.
**Source.** `SB-MLA-029`; the current names are `facies.rs:160`, `facies.rs:186` and
`src/ui/mlDialog.ts:113`.

#### SB-MLA-T31 — probability outputs are typed, and closeness-of-fit is not one symbol
**Input.** A classification run, a GMM run, and (on build) a fuzzy run.
**Operation.** Read the probability-bearing outputs and their declared types.
**Expected.** Each declares what it is a probability over and how it is normalised; a relative
score and a calibrated posterior do not share a naming convention. Separately, closeness-of-fit is
emitted as two distinct mnemonics — one an integer count in bins, one in curve units — and
**never as one symbol**.
**Source.** `SB-MLA-030`; dossier §5.3 `T-ML-CFIT-1` for the closeness-of-fit half, from G-6.4;
current outputs are `facies.rs:187`, `ml.rs:156`, `ml.rs:168`.

### Group D tests — vendor divergence

#### SB-MLA-T32 — competing vendor defaults are shown at the point of choice
**Input.** The cluster-count parameter.
**Operation.** Open the parameter for editing.
**Expected.** The competing shipped values are presented with their sources — IP's 15–20 first-stage
and 4–5 consolidated (T2), Techlog's stated 5 corroborated across two modules (T3), and Geolog's
explicit absence (T3) — and the user's choice is recorded as a decision with its own provenance.
**Source.** `SB-MLA-031`; `SB-CORE-013`; the values are §5's cluster rows.

#### SB-MLA-T33 — the normalisation basis is recorded on the curve
**Input.** Two clustering runs differing only in normalisation scheme.
**Operation.** Read the provenance of each output curve.
**Expected.** Each names its scheme and the statistics basis the scheme was computed over. The two
provenance records differ; the two curves differ.
**Source.** `SB-MLA-032`.

#### SB-MLA-T34 — the add-a-well trap
**Input.** Cluster three wells; then cluster the same three plus a fourth, under a data-derived
basis and again under a fixed-limits basis.
**Operation.** Compare the first three wells' cluster boundaries across the two pairs.
**Expected.** Under the data-derived basis the boundaries **shift**; under fixed limits they do
**not**. Both outcomes are reported. **A silent shift is a fail.**
**Source.** Dossier §5.3 `T-ML-NORM-1`, from Geolog `facimage_02_setup_hc.2.8.html` (T3).

#### SB-MLA-T35 — every automatic pre-transform is announced per curve
**Input.** A clustering job on GR / RHOB / NPHI / RT with an automatic logarithm enabled for
log-scale families, and the same job with it disabled.
**Operation.** Run both and read the run log.
**Expected.** The clusters differ; the run log states which transform was applied to which curve,
per curve, in both runs — including stating that none was applied in the second.
**Source.** Dossier §5.3 `T-ML-NORM-2`, from Techlog `principal-components-use-hra-clustering.html`
(T3), whose Output window is the behaviour being matched.

#### SB-MLA-T36 — a transformed quantity is a separate registry entry
**Input.** A permeability curve in mD and its base-10 logarithm.
**Operation.** Compute cluster statistics on the transformed quantity and report them.
**Expected.** The two are separate registry entries with distinct mnemonics and distinct units. The
reported mean of the transformed quantity carries the log unit; **a negative mean under a header
reading mD is a fail.** Any back-transform to display units appears in the log as an explicit step.
**Source.** `SB-MLA-035`; IP's documented opposite behaviour and its negative `PERMCORE` mean (T2).

#### SB-MLA-T37 — enumerated methods resolve by id, and the vendor spelling is an alias
**Input.** A model record whose metric is spelled `Euclidian`, one spelled `Euclidean`, one spelled
`Minimise`, one spelled `Minimum`, and one holding an unrecognised string.
**Operation.** Load each.
**Expected.** The first four resolve to the correct canonical identifiers through the alias table.
The fifth is an **error naming the unrecognised value** — never a silent fall-through to a default.
The five linkage rules are enumerated with their update rules, and Ward is the default carrying its
three-vendor source string.
**Source.** `SB-MLA-036`, `SB-MLA-046`; Geolog's verbatim `Euclidian` in
`facimage_03_generate_hc.3.6.html` (T3) and IP's `Minimum`/`Minimise` split (G-6.10, T2).

### Group E tests — method obligations

#### SB-MLA-T38 — the fuzzy combination-rule discriminator
**Input.** Three input curves, two competing bins. Bin A per-curve probabilities
`(0.99, 0.99, 0.05)`; bin B `(0.22, 0.22, 0.22)`.
**Operation.** Combine across curves and select the winning bin.
**Expected.** Reciprocal sum gives `P_A = 1/22.0202 = 0.04541` and `P_B = 1/13.63636 = 0.07333`, so
**bin B wins**. If the implementation returns A it has silently switched to a product rule
(`0.049005` against `0.010648`) — **hard fail**, not a tolerance breach. Additionally, the `√n_b`
weighting is asserted to be an explicit parameter with no default.
**Source.** Dossier §3.1 and §5.1 F2, derived from IP 2025 `[img-read: embim633.png]` (T2).

#### SB-MLA-T39 — equal-population binning reports what it achieved
**Input.** 100 calibration samples, 20 of them sharing one exact value, with 10 bins requested.
**Operation.** Bin and report.
**Expected.** The run does not claim 10 equal bins; it reports the populations actually produced.
**Source.** Dossier §5.3 `T-ML-FUZZY-2`; IP 2025 `statisticalcurveprediction.htm` states the
failure explicitly (T2).

#### SB-MLA-T40 — the uncertainty-band edge rule
**Input.** A configuration forcing `ResPC ± Er` outside `[0, 1]`.
**Operation.** Compute the band edge.
**Expected.** The result is the first or last bin mean **∓ 2 × that bin's standard deviation**, and
the run records that the fallback fired.
**Source.** Dossier §5.3 `T-ML-FUZZY-3`; IP 2025 `statisticalcurveprediction.htm` (T2).

#### SB-MLA-T41 — the cluster randomness index reproduces
**Input.** A synthetic label sequence: 1000 depth levels, 5 clusters with proportions
`p = (0.4, 0.3, 0.15, 0.1, 0.05)`, forming 50 cluster layers.
**Operation.** Compute the index.
**Expected.** `Av_thickness = 1000/50 = 20`;
`Random_thickness = 0.4/0.6 + 0.3/0.7 + 0.15/0.85 + 0.1/0.9 + 0.05/0.95
= 0.66667 + 0.42857 + 0.17647 + 0.11111 + 0.05263 = 1.43545`; `RI = 20 / 1.43545 = 13.933`.
Tolerance 1e−3 on the final value.
**Source.** Dossier §5.1 F11 and §5.3 `T-ML-RI-1`; IP 2025 `cluster_analysis.htm`, printed as
**ASCII, not a raster**, identically in `som.htm`, unchanged from IP 2018 (T2).

#### SB-MLA-T42 — the index falls to the vendor's random reference under shuffling
**Input.** The same label sequence as `SB-MLA-T41`, randomly shuffled with a fixed seed.
**Operation.** Recompute the index.
**Expected.** The value approaches **1**, the vendor's stated "totally random" reference. Asserted
as a bound rather than a point value: the shuffled index is at least an order of magnitude below
the structured one and within a stated band of 1.
**Source.** Dossier §5.3 `T-ML-RI-2`; IP 2025's interpretation statement (T2).

#### SB-MLA-T43 — the native path reports cluster quality
**Input.** A well-separated three-cluster fixture and a deliberately overlapping one.
**Operation.** Run the native electrofacies module on both.
**Expected.** Both report a per-cluster and an overall geometric quality measure with its sample
count. The separated fixture scores materially higher than the overlapping one; the direction is
asserted, the magnitude is not.
**Source.** `SB-MLA-044`; the inertia already computed and discarded at `facies.rs:143`–`:150`.

#### SB-MLA-T44 — restart spread is reported with its caveat
**Input.** A fixture with a known local optimum reachable from some seeds and not others.
**Operation.** Run with multiple restarts.
**Expected.** The distribution of the objective across restarts is reported, together with how
often the retained solution was reached. The output states that this is a **convergence**
diagnostic and not a cluster-count criterion.
**Source.** `SB-MLA-045`; Techlog's fall-off rule and its own stated caveat that the measure
"necessarily always decreases with increasing number of classes" (T3).

#### SB-MLA-T45 — the degenerate SOM decay is refused
**Input.** A SOM configuration expressing the decay constant in terms of the *current* iteration.
**Operation.** Attempt to configure and run.
**Expected.** Refused before training, with a message naming the degeneracy and citing the open
ledger item. A configuration supplying a total-iteration count is accepted; omitting it is refused
as a missing required parameter, not defaulted.
**Source.** Dossier §3.3 and §5.3 `T-ML-SOM-1`; the arithmetic is in §2.2 above.

#### SB-MLA-T46 — SOM distortion reproduces and decreases
**Input.** A hand-built 2×2 map with known neighbourhood weights and known squared distances.
**Operation.** Compute the distortion measure; then train on separable data and track it.
**Expected.** Matches the stated form to **1e−9** on the hand-built case, and **strictly decreases**
over training on separable data.
**Source.** Dossier §5.1 F10 and §5.3 `T-ML-SOM-2`; IP 2025 `som.htm` citing Wu & Takatsuka,
*Neural Networks* 19 (2006) (T2).

#### SB-MLA-T47 — the weight-normalisation guard
**Input.** A `k = 2` prediction where both neighbours carry the **same** associated value `y` and
sit at **equal** distance from the query point.
**Operation.** Predict.
**Expected.** The prediction equals **`y`**, not `2y`. **Hard fail** on `2y` — this is the
unnormalised-summation reading, and the failure it represents is a porosity that doubles when you
ask for one more neighbour.
**Source.** Dossier §5.1 F14, §5.3 `T-ML-KNN-4`, ML-12; Geolog `facimage_05_using_hc.5.05.html`
against `facimage_06_reference_hc.6.8.html` (T3).

#### SB-MLA-T48 — `k = 1` returns an actual training value
**Input.** A `k = 1` prediction at a query point.
**Operation.** Predict.
**Expected.** Exact equality with a training sample's value — not an average, not an interpolation.
**Source.** Dossier §5.3 `T-ML-KNN-2`; Geolog `facimage_06_reference_hc.6.8.html` and `.5.05.html`
(T3).

#### SB-MLA-T49 — leave-one-out excludes the held-out frame
**Input.** The training set itself, scored at `k = 1`.
**Operation.** Run the feature-subset scoring.
**Expected.** The error is **non-zero**. A zero error means the frame was not excluded and every
subset will score perfectly — **hard fail**.
**Source.** Dossier §5.1 F15 and §5.3 `T-ML-KNN-1`; Geolog `facimage_06_reference_hc.6.8.html`
names this exact trap (T3).

#### SB-MLA-T50 — the confusion-matrix axis guard
**Input.** A deliberately **non-square, non-symmetric** contingency table — 3 reference classes ×
4 model classes, unbalanced counts.
**Operation.** Emit both normalisations.
**Expected.** Row-% and column-% are **different numbers**; each sums to 100 % **along its own
axis**; each cell is labelled with the axis it was normalised on. A build that emits one
"percentage" column, or that passes only because the fixture was square or symmetric, is a **fail**.
**Source.** Dossier §2.8.d and §5.3 `T-ML-CONF-1`; Geolog's column-normalised "recognition rates"
against Techlog's row-normalised "row frequency" (T3).

#### SB-MLA-T51 — the acceptance threshold is a visible absence
**Input.** A facies tie-in run with no threshold configured.
**Operation.** Run and read the result.
**Expected.** The purity is reported; no accept/reject verdict is produced; the output states that
the threshold is a required user decision with no default. Once set, the value is recorded with the
result.
**Source.** `SB-MLA-052`; `SB-CORE-004`; `facies_tie.rs:128` currently returns purity with no
threshold and no prompt.

#### SB-MLA-T52 — a spread-multiple parameter is named for its statistic
**Input.** An imported configuration carrying a bare `tolerance = 2`.
**Operation.** Load it.
**Expected.** Refused or flagged as ambiguous, naming both candidate interpretations. The native
parameter is `tolerance_sd` and its unit is declared as multiples of the standard deviation.
**Source.** `SB-MLA-053`; ML-9 and dossier §5.1 F16, from four Techlog pages that disagree (T3).

#### SB-MLA-T53 — a class label is never interpolated
**Input.** A facies curve with class 2 above class 3 across a bed boundary, resampled to a finer
frame.
**Operation.** Resample by every path the product offers.
**Expected.** **No value outside the original label set is ever produced** — no 2.5, no 2.7. The
categorical flag on the registry entry makes any linear-interpolation path refuse rather than
round.
**Source.** `SB-MLA-055`; Geolog's own statement that facies is aperiodic tops data and
interpolation must be `TOPS` (T3).

#### SB-MLA-T54 — the resampling decision is logged per curve
**Input.** An ML run whose inputs are at three different sample intervals, plus point-sampled core
data.
**Operation.** Run and read the provenance.
**Expected.** Each input records its source sampling, the target frame, the method and the
tolerance. The core-plug match tolerance appears in the record with its value.
**Source.** `SB-MLA-054`; `FINDINGS` rule 15; the existing tolerance is `facies_tie.rs:144`.

#### SB-MLA-T55 — null discipline, with no opt-out
**Input.** A curve carrying −999, −9999 and −99 in an intermediate position, fed into an ML input
path and into the equation engine.
**Operation.** Run.
**Expected.** The affected samples are excluded, never arithmetically consumed. **There is no
setting that changes this.** A search of the settings surface for an opt-out is part of the test.
**Source.** Dossier §5.3 `T-ML-NULL-1`; IP 2025 `multi_line_user_formula.htm` documents the
opposite as its unchecked default (T2); `FINDINGS` rule 6.

#### SB-MLA-T56 — a threshold cannot be a sentinel
**Input.** An attempt to set a threshold to −9999, and an import carrying that value.
**Operation.** Set and load.
**Expected.** Both refused, naming the collision. "No threshold set" is representable and is
distinct from every numeric value.
**Source.** `SB-MLA-057`; ML-16, proven from Techlog's own default (T3) against its own
`MissingValue` (T1).

### Group F and G tests — boundary and platform

#### SB-MLA-T57 — no vendor model artifact is read, and no register item is approximated
**Input.** The full source tree and the shipped dependency set.
**Operation.** Static check: no code path reads a vendor model, weight or tile-encoding file; no
module implements or approximates a Tier-C register item; any feature serving a register item's
user need is labelled a design-around and carries its own primary sources.
**Expected.** Zero findings. A new dependency or file-format handler that would read a vendor model
artifact fails the check.
**Source.** `SB-MLA-058`, `SB-MLA-059`, `SB-MLA-060`; `CONTRACT.md` §2.1 and §2.2.

#### SB-MLA-T58 — a missing runtime component is named and actionable
**Input.** Three environments: no interpreter with `numpy`; an interpreter with `numpy` but no
`scikit-learn`; one with both but no `joblib`.
**Operation.** Attempt a supervised run with model saving in each.
**Expected.** Each names the missing component, names the interpreter inspected, and gives the exact
command that fixes it. In the third case the **predictions still complete and are written**, and
only the model save is reported as failed. The native facies, HFU and Lorenz paths remain available
in all three.
**Source.** `SB-MLA-061`; current behaviour at `python_engine.rs:47`–`:48`, `python_engine.rs:177`
and `ml.rs:713`–`:718`.

#### SB-MLA-T59 — the global lock is not held across a subprocess or a whole write phase
**Input.** An ML run over 40 apply wells, with a concurrent read from another part of the product.
**Operation.** Measure the longest contiguous hold of the global write lock.
**Expected.** The lock is not held during the fit at all, and is released between wells during
write-back. The measured hold is bounded by a single well's write, not by the run.
**Source.** `SB-MLA-062`; `SB-CORE-032`. The current write-back holds one lock across
`ml.rs:630`–`:706`.

#### SB-MLA-T60 — every binding cap is reported
**Input.** A leaderboard request generating 300 combinations; a t-SNE request above the sample cap;
a native facies request with `K = 20`.
**Operation.** Run each.
**Expected.** The first reports that 80 of 300 were evaluated and what was dropped; the second
refuses, naming the actual sample count; the third reports that `K` was clamped to 12 from 20.
The third currently clamps silently.
**Source.** `SB-MLA-063`; the caps are `ml.rs:1299`, `ml.rs:210` and `facies.rs:77`.

#### SB-MLA-T61 — listing models never loads their bytes          `CHARACTERIZATION`
**Input.** A project holding several models with large artifacts.
**Operation.** List the models and observe the query and the bytes transferred.
**Expected.** The blob column is not selected and the transferred size is independent of total
artifact size. **Labelled `CHARACTERIZATION`**: the expectation is the current documented
behaviour at `db.rs:2667`, already pinned by `listing_models_never_carries_their_bytes`
(`ml.rs:1764`), and has no external source. It is carried here so the requirement it satisfies is
traceable to a named test.
**Source.** Current behaviour; `SB-CORE-030`.

**Two further `CHARACTERIZATION` labels** apply to assertions embedded above rather than to whole
tests, and are named here so the count is honest: the "predictions still complete" clause of
`SB-MLA-T58` and the "no all-missing curve is written" clause of `SB-MLA-T13` both pin behaviour
that the Python path already has and the native path does not — the expected value is the better of
two in-tree behaviours, not an external source. Total labelled: **four**.

---

## 7. Open items, escalations and refusals

### 7.1 Open — needed, not yet answerable

**O-1 — Whether the two k-means engines currently agree on real data, and by how much.**
`SB-MLA-023` asserts they must; nothing establishes whether they do today. The constants differ
(8 restarts at 100 iterations against 10 at 300) and the divergence only bites where the objective
surface has competing local optima, which is data-dependent. **Settled by** writing `SB-MLA-T23`
and running it on a pooled multi-well fixture at `K = 15` before deciding whether the fix is to
unify the constants or to remove one engine.

**O-2 — The magnitude of the leaderboard's scaler optimism on real data.** §3.3(e) establishes the
mechanism and bounds the *weight* of the leak (roughly one group's share of the transform). It does
not establish how much the reported blind score moves on data with realistic between-well
variation, which is the number that says whether this is a correctness bug or a correctness bug
that also mattered. **Settled by** `SB-MLA-T29` run in both configurations on a multi-well fixture
with a deliberately shifted well.

**O-3 — Whether scikit-learn convergence warnings are reachable through the sidecar at all.**
`SB-MLA-016` requires convergence to be reported. The Python path writes warnings to `stderr`, and
`python_engine.rs` treats the last `stderr` line as an error message only on a non-zero exit — so a
`ConvergenceWarning` on a successful run may be discarded before anything can act on it. **Settled
by** reading the sidecar's stream handling for the `ml.rs` runners specifically and determining
whether a structured warning channel already exists or must be added.

**O-4 — Whether any current resampling path can interpolate a class curve.** `SB-MLA-055` is P0 on
the strength of the hazard, not on a demonstrated instance: the three class curves are written as
`f32` with no categorical marking, so the hazard is structural. Whether it is *live* depends on
which resampling and reframing paths a facies curve can currently reach. **Settled by** a sweep of
the reframe and decimation paths for their interpolation method, which is a `DIO`/`DBM` question as
much as this chapter's.

**O-5 — What, if anything, SandiBumi may take from Techlog's bundled Python ML stack.** The dossier
records it as a T1 install-tree finding. Reading a vendor's bundled third-party libraries tells you
what they build *on*, which is competitive intelligence; it says nothing about their algorithms and
nothing here proposes using it. The open question is narrower and is a licensing one, not a
technical one. **Settled by** a licensing read, not by more research.

**O-6 — Cluster-count guidance for the delivered-work regime.** The corpus gives IP's 15–20 → 4–5
staging, Techlog's default 5, and Geolog's silence. Delivered projects independently reached 7
facies groups and 4 rock-quality classes. None of this is a default and this chapter does not make
it one. What is not yet answerable is whether the staged approach (cluster high, consolidate down)
should be the product's recommended workflow, which is a product decision resting on `SB-MLA-043`
and `SB-MLA-044` shipping first so the consolidation has a criterion.

### 7.2 Escalations — each is a question with a checkable answer

**E-1 — Kohonen (1990), *Proc. IEEE* 78(9), 1464–1480.**
**Question:** what is the standard definition of the decay time constant `λ` in terms of the total
iteration count, as published? **Why it matters:** IP's printed form is provably degenerate (§2.2)
and IP is the only printed SOM source in the corpus with no external arbiter (ML-4). SandiBumi's
total-iteration parameterisation is currently *its own*, carried with a source string that says so;
the paper would let it be carried as the published form instead. **Cost:** one named paper. This is
the exact edition Geolog cites. **Blocks:** `SB-MLA-041` shipping as anything other than a
SandiBumi deviation.

**E-2 — Ye, S.-J. & Rabiller, P., "A New Tool for Electrofacies Analysis: Multi-Resolution
Graph-Based Clustering", SPWLA 41st Annual Logging Symposium, paper PP.**
**Question:** what are the definitions of the MRGC outputs Geolog names but does not define —
`KERNELS`, `KORDER`, `KRI`, `NI`, `NNI` — and what is the cluster-count optimisation criterion?
**Why it matters:** **MRGC is the method used in delivered work.** Geolog cites the paper and refers
the reader to it for every definition. It is also the single highest-value acquisition in the domain
because it closes ML-12 as well: the KNN/Barycenter weight function and its normalisation
(`SB-MLA-049`). **Search term:** the vendor prints **"41st Annual Logging Symposium" + "paper PP"**
— use that. **The year 2000 is the dossier's inference from the symposium number, not a vendor
statement**, and searching on it may fail. **Cost:** one named paper, double value.

**E-3 — Bezdek (1981), fuzzy c-means.**
**Question:** what is the published FCM barycenter and membership update, so ML-1 and ML-2 can be
closed against a primary source rather than against a defective vendor page? **Why it matters:**
only if the c-means family is ever built. **Cost:** one named text. **Priority:** low — the method
is quarantined (R-3) and nothing depends on it.

**E-4 — Cuddy (1997), SPWLA paper S; and Cuddy (2000), SPE 65411.**
**Question:** is the reciprocal-sum combination applied to the raw `√n_b`-weighted per-curve
probability, or to a normalised one (ML-11)? **Why it matters:** §2.1's combination rule is the most
consequential equation in the domain and currently rests on a **single T2 raster with no external
arbiter**. Since the probability carries `√counts`, the two readings differ whenever bin populations
differ, which under variable-size binning is always. The paper closes the sub-question *and* gives
an independent check on the transcription itself. **Cost:** two named papers. **Blocks:** the
confidence level `SB-MLA-037` can be stated with, not the requirement itself.

**E-5 — One live Geolog session, approximately one hour.**
**Question:** what pre-filled values do the Facimage dialogs actually carry for MRGC
`Minimum`/`Maximum Number of Electrofacies`, `Number of Optimal Models`, `Initial Neurons for
CFSOM`; DYNCLUST `NBCR`, `NBCM`, `Iterations`, `Minimum Interclass Stability Variation`; SOM
`X`/`Y Neurons`, `Shakings`, `Iterations`; AHC `Number of Classes`; ANN `Maximum Number of Training
Epochs`, `Neurons in Hidden Layer`; STM `Maximum K-Nearest Neighbors`, `Maximum K'-Strongest
Membership` and both a-priori reassignment rates; and fuzzy `QQ`? **Why it matters:** roughly
**eighteen** of §5's absences close at once (ML-8) — the best ratio of parameters resolved to effort
spent anywhere in this chapter. **Cost:** one live session on software already installed.

**E-6 — One live Techlog session.**
**Two questions, both decisive.** (a) Set `QQ = 2` and `QQ = 10` on the same data and observe which
direction the barycenter moves — this settles ML-3, where the prose and the printed exponent
contradict each other. (b) Read the emitted probability curve and test whether it sums to 1 across
classes at a depth — this settles ML-2, and the dossier notes the test is now "cheap and decisive"
after the factoring analysis. A third, lower-value question: the four K.mod *training*
hyperparameters that ML-10 still leaves open (network geometry, learning cycles, learning rate,
convergence criterion). **Priority:** low, gated on E-3's relevance.

**E-7 — One live IP 2025 session.**
**Question:** set two very different seed grids in Cluster Analysis and compare the outputs — do
they differ? **Why it matters:** ML-5 is a real cross-edition discrepancy. IP 2018 documents
PCA-based seeding as functional and tells the user to change the seed points on failure; IP 2025
states twice that seed values are ignored. Either the behaviour regressed or the 2018 documentation
was always wrong, and this chapter presents both readings without adopting either. **Priority:**
low — nothing in SandiBumi depends on the answer, and it is recorded because a vendor regression
found from documentation alone is worth confirming once.

**E-8 — Re-ingest the 2006 Rabiller Facimage guide at full depth.**
**Question:** what do pages 56–131 contain? The existing petro-kb note records that **only the
overview and the first pages of the MRGC section were read**, and explicitly lists what was skipped:
"the detailed MRGC clustering parameters, KNN prediction workflow, STM similarity workflow, and
histogram up-scaling". **Why it matters:** the dossier calls this "the cheapest large win
available" — it partially closes ML-6 and ML-7 **without acquiring anything new**, using a document
already on this machine. **Cost:** an ingest run, no acquisition. **This should be done before E-2**,
because it may narrow what E-2 needs to answer.

**E-9 — For Jauhar: does `SB-CORE-010` extend into the deliverable, or stop at the UI?**
`SB-CORE-010` as written requires the ancestry to be *recorded* and requires "the UI MUST show this
ancestry for any curve on demand". It does not require the ancestry to reach a report or an export.
`03_EVIDENCE_BASE.md` §14.4 states the ambition differently — "a parameter that carries the paper it
came from, through the computation, **into the deliverable**" — and that is the version this chapter
needs, because §3.5 finds the lineage stops dead at `report.rs` and `export.rs`. **The exact
question:** is the deliverable-side obligation inside `SB-CORE-010`'s scope, or does it need its own
core id? `SB-MLA-010` is written as a domain requirement either way and does not depend on the
answer; the escalation is about whether other chapters are silently assuming a core requirement
that is not there. **I have not minted an id.**

**E-10 — For Jauhar: `SB-CORE-010`'s enumeration is complete for deterministic modules and
incomplete for learned ones.** It requires "the module and its version; every input curve and its
log set; every parameter value and that value's source string; the zone definition; the operator;
and the timestamp". For a fitted model that list is **not sufficient to reconstruct the number** —
the training rows, the fitted scaler, the model artifact and the library set are not parameters and
are not inputs to the curve's own well. Group A of §4 specifies all of it as domain requirements, so
this chapter is covered. **The exact question:** should `SB-CORE-010` be amended to say "and, where
the module is a fitted model, its artifact identity and training-set identity", or is the domain
allocation sufficient? This matters beyond this chapter because any future chapter that fits
anything inherits the same gap. **I have not minted an id.**

**E-11 — No chapter in the eighteen owns the user-programming / formula layer.** The dossier's §2.9
covers it and ships two fixtures for it — operator precedence in a nested `MIN()` without inner
parentheses, which IP documents as returning incorrect results; and the degrees/radians round trip
across the formula boundary, a silent factor of 57.2958. Both are real, both are testable, and
**neither belongs to machine learning.** `SB-MLA-056` adopts the null-discipline fixture because a
null entering a training matrix is squarely this domain's problem, but the parser and trigonometry
fixtures are not. **The exact question:** which chapter owns the user formula language, or does the
set need a nineteenth? They are dispositioned `ESCALATED` in §8 rather than orphaned.

**E-12 — For Jauhar: sequencing within Group A.** Group A has twelve requirements and only two are
P0. The rest are P1 and they are not independent — `SB-MLA-003` (training-row hash) is what makes
`SB-MLA-008` (byte-identical re-run) checkable rather than asserted, and `SB-MLA-010` (the
deliverable block) is the one a customer sees. **The exact question:** is the first increment
`001 + 003 + 006 + 008` (make it true), or `001 + 006 + 009 + 010` (make it visible)? This chapter
does not choose, because the answer depends on whether the near-term goal is a defensible
deliverable or a demonstrable claim.

**Lower-priority acquisitions**, recorded so they are not lost: Thevoux-Chabuel, Veillerette &
Rabiller (1997) SPWLA paper BB, for the STM reject concept — the best extrapolation guard the
dossier finds in any of the three tools; Rabiller, Boles & Dewhirst, URTeC 2013 (control ID
1580723) for Input Back Modeling; Wu & Takatsuka (2006), *Neural Networks* 19, which would close
G-9.8's spherical-tessellation node counts from first principles rather than from an observed
screenshot value; Diday (1971) and Mourot (1993), Geolog's own DYNCLUST citations, low priority
because the method is described adequately in the help set.

### 7.3 Refusals

**Every Tier-C refusal in this domain is listed here, with the rule cited.**

**R-1 — Experienced Eye / EEFS.** `CONTRACT.md` §2.2. SandiBumi will not implement, approximate,
reverse-engineer or reconstruct it, and will not specify a requirement that is this capability under
another name. Capability-level description is permitted and is in the vendor's own marketing;
nothing beyond that appears in this chapter. **No design-around is specified**, because specifying
one before its primary sources are in hand is how a reconstruction gets written by accident
(`SB-MLA-059`).

**R-2 — Domain Transfer Analysis, including its key file.** `CONTRACT.md` §2.2. Same terms as R-1.
The file is not read, parsed, or characterised.

**R-3 — The shipped neural-network weight files.** `CONTRACT.md` §2.1 and §2.2. They are not read,
imported, converted, or inspected, and no SandiBumi feature consumes a vendor-trained model in any
format (`SB-MLA-060`, P0). **One boundary call is recorded rather than made silently:** §5 carries
`nn.hidden_layers = 1` from an IP page. That value is carried **only** because it is a generic
architectural fact, independently re-derivable from any neural-network text, and it is marked in the
table as Tier-C-adjacent provenance with the vendor engine and weight files named as never used. If
that reading is judged too fine, the row should be struck; it is load-bearing for nothing.

**R-4 — The Textural Facies `Freq_Tiles` tile encoding.** `CONTRACT.md` §2.2. The encoding is not
transcribed, inferred, or reconstructed from observed behaviour. Textural Facies is not a v1 target
and its SOM-input thresholds remain an open vendor gap (G-9.9) that this chapter does not attempt to
close.

**R-5 — Entropy image speed-correction and frequency-domain dispersion fits.** `CONTRACT.md` §2.2.
Named here for completeness because they are on the register; neither is in this domain's scope and
neither is approached.

**R-6 — Inferring a vendor algorithm from observed behaviour.** `CONTRACT.md` §2.2 prohibits
reconstruction "under any framing", and behavioural inference is a framing. Where this chapter
reports a vendor behaviour it reports what the vendor *documents*, and where the documentation is
absent it records the absence (ML-4's verified negative result on Techlog's SOM math is the model:
the finding is "nothing is printed", not a reconstruction of what must be happening).

**R-7 — Techlog's fuzzy c-means as printed.** Dossier §5.1 F18, marked **DO NOT IMPLEMENT**. This is
a quarantine of a defective printed equation, not a Tier-C refusal, and the distinction matters
because the terms of the two are different: the c-means family *may* be built later from a primary
source (E-3), whereas a Tier-C item may not be built at all. The quarantine stands specifically on
**ML-1 and ML-3, not on ML-2** — the membership factors to a form whose `argmax` is already correct
and whose row-normalisation recovers a proper membership exactly, while the barycenter printed as an
unnormalised sum lands outside the data cloud and corrupts every distance fed to it. If the method
is ever built, ML-1 closes first.

**R-8 — IP's printed SOM decay law.** §2.2. SandiBumi will not transcribe it, and will refuse a
configuration expressing it (`SB-MLA-041`, `SB-MLA-T45`). This is a refusal to reproduce a vendor
defect, and it is the clearest case in the chapter of `03_EVIDENCE_BASE.md` §14.1 — SandiBumi does
not copy a vendor's number to be compatible with a vendor's error.

**R-9 — The unnormalised-summation reading of Geolog's KNN prediction.** §2.6. SandiBumi implements
the normalised weighted average, which is the only unit-correct form. The deviation from one of
Geolog's two contradicting pages is declared rather than silent (`SB-MLA-049`, `SB-MLA-T47`).

**R-10 — IP's silent cross-validation disable under zonal averaging.** §2.10. Where a combination of
options cannot be honoured, SandiBumi refuses it with a named reason. It does not accept the
options and quietly drop one.

**R-11 — Any opt-out from null discipline.** §2.14 and dossier §3.9. IP ships
"treat intermediate nulls as −999" as its *unchecked default*. SandiBumi ships no such setting at
any default (`SB-MLA-056`, `SB-MLA-T55`).

**R-12 — Vendor lookup-table data and vendor parameter files.** `CONTRACT.md` §2.1. None is
transcribed into this chapter. **This chapter does not believe it has a second case of the recorded
Matthews & Kelly exception, and has not reasoned from that exception at any point.** The one
adjacent judgement is stated openly so it can be overruled: §5 and `SB-MLA-043` treat IP's cluster
randomness index as an **equation**, on the grounds that it is printed as ASCII rather than as a
raster, identically in two places, is four lines of arithmetic over quantities the user supplies,
and produces no tabulated values — which is a different object from the `.obg` overburden tables or
`Poisson_Ratio_Lithologies.par`, whose *content is their values*. Implementing an arithmetic
identity is not transcribing a lookup table. **If that distinction is judged wrong, `SB-MLA-043` and
`SB-MLA-T41`/`T42` fall and nothing else in the chapter depends on them.**

**R-13 — Reproducing MRGC.** Recorded as a refusal rather than an open item because the temptation
is real and specific: MRGC is the method used in delivered work, Geolog names its five advanced
outputs, and the behaviour is observable in a product on this machine. **This chapter does not
propose a reconstruction, an approximation, or an equivalent formulation, and it does not infer the
algorithm from the output names.** MRGC is not on the Tier-C register — it is a published method
whose paper simply is not held — so it is legitimately buildable **after E-2 delivers the primary
source**, and not before. The distinction from R-1 and R-2 is exactly that: a missing citation, not
a prohibited one.

---

## 8. Traceability — dossier disposition

### 8.0 The counting basis, and the discrepancy

`CONTRACT.md` §8 requires a row for every item in the dossier and requires the row count to be
reconciled against the dossier's own count. **The dossier does not state a single total**, so the
basis has to be declared before the count means anything. Items were enumerated by parsing the
file, not by estimate:

| Dossier section | Countable items | Basis |
|---|---|---|
| §1 Method inventory | **6** | §1.1, §1.2, §1.2.a, §1.3, §1.4, §1.5 |
| §2 Definitions and equations compared | **23** | every numbered subsection and lettered sub-subsection |
| §3 Differences that matter | **9** | §3.1 … §3.9 |
| §4.1 – §4.6 Optimal choice | **6** | one per method family |
| §4.7 Ledger disposition | **20** | table rows, covering **24 G-ids + R-15 = 25 ledger ids** |
| §5.1 Canonical equation forms | **18** | F1 … F18 |
| §5.2 Parameter table | **50** | data rows, excluding the 6 group-header rows |
| §5.3 Fixtures | **25** | 24 `T-ML-*` ids + 1 ⚠ guard row |
| §5.3 Field-data acceptance | **3** | bullets |
| §5.4 FINDINGS rules | **10** | rules 1, 3, 6, 7, 8, 9, 10, 11, 14, 15 |
| §6 New open items | **17** | ML-1 … ML-17 |
| §6 Acquisitions | **9** | numbered list |
| §8 Critique dispositions | **20** | B-1, B-2, M-1 … M-6, m-1 … m-12 |
| §8 Rebutted claims | **2** | findings that did not survive verification |
| **Total** | **218** | |

**§8 below carries 218 rows.** Three counting decisions are stated rather than smoothed, because
each one could have been made differently and a reader checking the arithmetic needs to know which
was taken:

1. **§6's "8 inherited open items" are not counted again.** They are `G-6.1/G-9.6`, `G-6.2/G-9.1`,
   `G-6.5/G-9.10`, `G-9.2`, `G-9.5`, `G-9.7`, `G-9.8`, `G-9.9` — the same objects already
   dispositioned as rows of §4.7, and §6 says so explicitly ("Dispositions in §4.7"). Counting them
   twice would inflate the total by 8 and would create two rows that could drift apart, which is
   the defect `SB-CORE-007` exists to prevent. **Total under the double-counting basis would be
   226.**
2. **§7's source register (10 subsections) is accounted for but not rowed.** It is a provenance
   manifest — where each tier of evidence was read — not a set of findings. Its contents are
   dispositioned through the findings that cite them. Rowing it would put ten rows in the table
   whose disposition column could only ever read "evidence provenance", which is noise in a
   completeness gate. **Total including it would be 228.**
3. **§4.7's table is counted at 20 rows, not 25 ids.** Three rows carry more than one ledger id
   (`G-6.1/G-9.6`, `G-6.2/G-9.1`, `G-6.5/G-9.10`) and one row carries three
   (`G-9.3, G-9.4, G-9.11`). The rows are the dossier's own unit of disposition. **The 25 ids are
   individually named inside the rows below**, so nothing is lost either way.

**One dossier count is itself wrong, and this chapter does not propagate it.** §6 states
"**Open items carried forward from the IP2025 ledger (8)**" and lists eight. §4.7 dispositions
**nine** distinct inherited items (the eight, plus `R-15`) and additionally dispositions
`G-6.3`, `G-6.4`, `G-6.6` … `G-6.13`, `G-9.3`, `G-9.4`, `G-9.11` as closed or not-applicable. The
"(8)" is a count of items **still open**, not of items carried forward, and the dossier's own §4.7
header says "every ledger / OPEN item touching this domain". The two counts measure different
things and are both correct under their own reading; the ambiguity is in the word "carried". Noted
so that a future reader reconciling the two does not conclude one is an error.

Disposition vocabulary, per `CONTRACT.md` §8: `ADOPTED` (became a requirement), `DEFERRED`
(sound, not v1), `REJECTED` (with a reason), `EVIDENCE-ONLY` (informs, generates no requirement),
`ESCALATED` (§7).

### 8.1 §1 Method inventory — 6 rows

| Dossier item | Disposition | Where |
|---|---|---|
| §1.1 IP 2025 inventory (28/28 pages, T2) | `EVIDENCE-ONLY` | Sources §2.1–§2.8; capacity caps → `SB-MLA-063` |
| §1.2 Techlog 2018.2 inventory (T3 + T1) | `EVIDENCE-ONLY` | Sources §2.3.b, §2.4, §2.5.b, §2.6, §2.8.b |
| §1.2.a Bundled Python ML stack (T1, install tree) | `EVIDENCE-ONLY` | §7.1 O-5 — a licensing question, not a technical one |
| §1.3 Geolog V14 inventory (T3 + T1 + T4) | `EVIDENCE-ONLY` | Sources §2.1, §2.6, §2.7, §2.8.c |
| §1.4 SandiBumi current state | `ADOPTED` | The whole of §3; every `file.rs:line` re-verified at source |
| §1.5 Explicit "no evidence held" | `ADOPTED` | `SB-MLA-058`, `SB-MLA-060`; §7.3 R-1, R-2, R-3, R-4 |

### 8.2 §2 Definitions, equations and assumptions compared — 23 rows

| Dossier item | Disposition | Where |
|---|---|---|
| §2.1 Fuzzy — the "same name, three equations" trap | `ADOPTED` | `SB-MLA-036`, `SB-MLA-037` |
| §2.1.a IP 2025 Cuddy (T2, raster) | `ADOPTED` | `SB-MLA-037`, `SB-MLA-040`; F1–F4 |
| §2.1.b Geolog V14 Cuddy (T3) | `ADOPTED` | `SB-MLA-038`, `SB-MLA-039`; §5 `fuzzy.n_bins`, `fuzzy.min_samples_per_bin` |
| §2.1.c Techlog Ipsom fuzzy c-means (T3, images at source) | `REJECTED` — quarantined as printed | §7.3 R-7; ML-1, ML-2, ML-3 escalated as E-3/E-6 |
| §2.1.d Fuzzy comparison table | `ADOPTED` | `SB-MLA-036` — the table *is* the case for id-not-string addressing |
| §2.2 Self-Organising Maps | `ADOPTED` | `SB-MLA-041`, `SB-MLA-042` |
| §2.3 Partitional clustering (K-means family) | `ADOPTED` | `SB-MLA-023`, `SB-MLA-024`, `SB-MLA-045` |
| §2.3.a IP Cluster Randomness Index (T2, ASCII) | `ADOPTED` | `SB-MLA-043`; boundary call recorded at §7.3 R-12 |
| §2.3.b Techlog TechCore Petrophysical groups (T3) | `ADOPTED` | `SB-MLA-046` (Ward), `SB-MLA-057` (the −9999 collision) |
| §2.4 Hierarchical / agglomerative clustering | `ADOPTED` | `SB-MLA-025`, `SB-MLA-046`; F12 |
| §2.5 Neural networks | `EVIDENCE-ONLY` | Method only; §7.3 R-3 governs the weight files |
| §2.5.a IP's four NN scenario recipes (T2) | `EVIDENCE-ONLY` | §5 `nn.training_zones`, `nn.training_passes` — cited, not adopted as SandiBumi defaults |
| §2.5.b Techlog Decision tree (T3) | `DEFERRED` | The only fully-printed supervised classifier in the corpus; an interpretable single tree is a strong v2 candidate, and ML-17's threshold transposition is recorded |
| §2.6 PCA / dimensionality reduction | `ADOPTED` | `SB-MLA-047`, `SB-MLA-048` |
| §2.6.a Techlog MCA — qualitative variables (T3) | `DEFERRED` | No SandiBumi path handles qualitative inputs; the ML-13(c) percent-sign defect is recorded |
| §2.6.b ML-13 — PCA worked example fails to close | `ADOPTED` as a negative | Quarantined: `SB-MLA-T25`/`T26` use **IP's** examples, which verify. §7.2 E-6 note |
| §2.7 Model propagation / prediction from a clustering | `ADOPTED` | `SB-MLA-049`, `SB-MLA-050` |
| §2.8 Contingency / confusion — all three tools ship one | `ADOPTED` | `SB-MLA-051` |
| §2.8.a IP standalone Contingency Table (T2) | `ADOPTED` | `SB-MLA-051` |
| §2.8.b Techlog TechStat Ancor (T3) | `DEFERRED` in part | Both normalisations `ADOPTED` at `SB-MLA-051`; the association statistics (χ², Cramér's V, C) are `DEFERRED` — no SandiBumi requirement claims them |
| §2.8.c Geolog Facimage Models → Comparison (T3) | `ADOPTED` | `SB-MLA-051`; the recognition/reconstruction naming is the source of the axis-labelling obligation |
| §2.8.d Comparison and the consequence for SandiBumi | `ADOPTED` | `SB-MLA-051`, `SB-MLA-052` |
| §2.9 User-programming / formula layer | `ESCALATED` | §7.2 E-11 — no chapter in the eighteen owns it. Null discipline alone is retained here at `SB-MLA-056` |

### 8.3 §3 Differences that matter — 9 rows

| Dossier item | Disposition | Where |
|---|---|---|
| §3.1 Fuzzy combination — reciprocal sum vs product | `ADOPTED` | `SB-MLA-037`; `SB-MLA-T33` is the discriminator |
| §3.2 One word "fuzzy", two algorithm families | `ADOPTED` | `SB-MLA-036`, `SB-MLA-029` |
| §3.3 IP's SOM decay is provably degenerate | `ADOPTED` as a refusal | `SB-MLA-041`; §7.3 R-8; E-1 would close the vendor-intent half |
| §3.4 K-means seeding — three tools, three stories | `ADOPTED` | `SB-MLA-024`, `SB-MLA-045`; ML-5 escalated as E-7 |
| §3.5 Normalisation and pre-transform — the widest silent divergence | `ADOPTED` | `SB-MLA-032`, `SB-MLA-033`, `SB-MLA-034`, `SB-MLA-035` — four requirements from one dossier section, the highest yield in the chapter |
| §3.6 Cluster-count selection — three genuine criteria | `ADOPTED` in part | `SB-MLA-044`; the staging workflow is `O-6`, not a default |
| §3.7 Determinism and auditability — every vendor loses | `ADOPTED` | Group A entire (`SB-MLA-001` … `SB-MLA-012`) |
| §3.8 Capacity and input limits (2018 → 2025 drift) | `ADOPTED` | `SB-MLA-063` |
| §3.9 Degrees vs radians, and the null-into-arithmetic trap | `ADOPTED` in part; `ESCALATED` in part | Null discipline → `SB-MLA-056`. The trigonometry half is E-11's, not this domain's |

### 8.4 §4.1 – §4.6 Optimal choice per item — 6 rows

| Dossier item | Disposition | Where |
|---|---|---|
| §4.1 Fuzzy → IP's equations, Geolog's engineering | `ADOPTED` | `SB-MLA-037` … `SB-MLA-040` |
| §4.2 SOM → IP's skeleton, Geolog's parameters, neither's decay | `ADOPTED` | `SB-MLA-041`, `SB-MLA-042` |
| §4.3 Partitional → Techlog HRA architecture, Geolog seeding vocabulary | `ADOPTED` in part | `SB-MLA-043`, `SB-MLA-044`, `SB-MLA-045`. The HRA architecture itself (PCA → K-means → silhouette) is `DEFERRED`: SandiBumi's two k-means engines must be reconciled (`SB-MLA-023`) before a third staged path is added |
| §4.4 Hierarchical → five linkages, Ward default | `ADOPTED` | `SB-MLA-046`; the M-1 scoping caveat is carried in the `SB-MLA-046` citation |
| §4.5 Neural / supervised → method only, no vendor weights | `ADOPTED` | `SB-MLA-060`; §7.3 R-3 |
| §4.6 Model propagation → Geolog Facimage is the reference | `ADOPTED` in part; `ESCALATED` in part | `SB-MLA-049`, `SB-MLA-050`. MRGC itself is E-2 and §7.3 R-13 — not reconstructed |

### 8.5 §4.7 Ledger disposition — 20 rows, covering 25 ledger ids

| Dossier item | Disposition | Where |
|---|---|---|
| `G-6.1 / G-9.6` NN epochs 1000 (prose) vs 100 (panel) | `REJECTED` — adopt neither | §5 `nn.epochs` reads `NON-ADOPTABLE — cited for verification`; SandiBumi's `max_iter=500` is its own, cited to `ml.rs:109` |
| `G-6.2 / G-9.1` SOM `λ = t / log σ₀`, `t` current | `ADOPTED` as a refusal | `SB-MLA-041`, `SB-MLA-T45`; §7.3 R-8; E-1 |
| `G-6.3` SOM raster prints `+` where `=` belongs | `ADOPTED` as a structural argument | `SB-MLA-036` and §5.4 rule 1 — one machine-readable form, rendered views generated |
| `G-6.4` `Cfit` means bin distance and curve units | `ADOPTED` | `SB-MLA-030` (typed outputs), and the **three-state** model — present-as-bins, present-as-units, legitimately absent — is why `SB-MLA-030` does not mandate a closeness-of-fit on every prediction |
| `G-6.5 / G-9.10` "Weight bin by sample count" default contested | `ADOPTED` | `SB-MLA-040` — explicit, always-visible, no hidden default; §5 row reads `ABSENT` |
| `G-6.6` User-app crossplot "five interactive lines" | `REJECTED` — not applicable | SandiBumi does not reproduce IP's user-app dialog |
| `G-6.7` Menu reorganisation inconsistent; PCA page self-contradicting | `EVIDENCE-ONLY` | §5.4 rule 10; a docs-generation obligation, not an ML one |
| `G-6.8` `mpmaths` vs `ipmaths` | `REJECTED` — not applicable | IP-internal dependency list |
| `G-6.9` `@` called an "ampersand" | `REJECTED` — cosmetic | — |
| `G-6.10` Linkage `Minimum` vs `Minimise` across sibling pages | `ADOPTED` | `SB-MLA-036` — canonical id `linkage.single`, display label separate, never matched on a string |
| `G-6.11` AppData folder cited three ways | `REJECTED` — not applicable | — |
| `G-6.12` Fuzzy tab name prose vs UI | `REJECTED` — not applicable | — |
| `G-6.13` SOM Input-tab text garbled mid-sentence | `EVIDENCE-ONLY` | Obscures "Use Well for Model Run"; recorded, nothing depends on it |
| `R-15` NeuroSolutions attribution scrubbed between 2018 and 2025 | `ADOPTED` as a refusal | `SB-MLA-060`; §7.3 R-3, including the recorded boundary call on `nn.hidden_layers = 1` |
| `G-9.2` Which panel values are factory defaults | `ADOPTED` | Every affected §5 row carries the screenshot caveat verbatim or reads `ABSENT`; §5.4 rule 9 |
| `G-9.5` NN normalisation scheme undocumented | `ADOPTED` | `SB-MLA-028`, `SB-MLA-032`; SandiBumi's answer is at `ml.rs:68` and `ml.rs:229`–`:247` |
| `G-9.7` SOM "Average Closest N nodes" values and default not listed | `ADOPTED` | §5 row ships `ABSENT` with a stated source; `SB-MLA-042` |
| `G-9.8` Spherical SOM tessellation-valid node counts | `DEFERRED` | Spherical SOM is not a v1 target; acquisition 8 (Wu & Takatsuka) would close it from first principles, not from IP |
| `G-9.9` Textural Facies SOM-input thresholds never enumerated | `REJECTED` — Tier C boundary | §7.3 R-4; the `Freq_Tiles` encoding is not approached |
| `G-9.3, G-9.4, G-9.11` SandPit `S1`/`S2`; scrolled grid; denormal Best Cost | `REJECTED` — out of domain / example values / observation | `G-9.3` belongs to `18_geomech-ppfg.md` |

### 8.6 §5.1 Canonical equation forms — 18 rows

| Dossier item | Disposition | Where |
|---|---|---|
| **F1** Fuzzy per-curve bin probability | `ADOPTED` | `SB-MLA-037`, `SB-MLA-040`; `SB-MLA-T33` |
| **F2** Fuzzy combination across curves | `ADOPTED` | `SB-MLA-037`; `SB-MLA-T33` is the reciprocal-sum-vs-product discriminator, and E-4 would close ML-11 |
| **F3** Fuzzy weighted result of the two most likely bins | `ADOPTED` | `SB-MLA-039` |
| **F4** Fuzzy uncertainty band | `ADOPTED` | `SB-MLA-039`, `SB-MLA-T35` — the edge rule at `ResPC ± Er` outside [0, 1] |
| **F5** Fuzzy within-bin refinement (Geolog) | `REJECTED` for v1 | Ships **off**: it privileges the first curve and IP's Cuddy has no such step. Recorded in §5, not defaulted on |
| **F6** Fuzzy prediction error bar (Geolog) | `ADOPTED` | `SB-MLA-039` |
| **F7** SOM BMU selection | `ADOPTED` | `SB-MLA-041` |
| **F8** SOM weight update (BMU and neighbours) | `ADOPTED` | `SB-MLA-041`; the `=`-not-`+` reading is forced by structure (G-6.3), never by inference about IP |
| **F9** SOM decay — SandiBumi's own parameterisation | `ADOPTED` | `SB-MLA-041`, `SB-MLA-T45`; §5 `som.total_iterations` reads `ABSENT`, `REQUIRED` |
| **F10** SOM distortion | `ADOPTED` | `SB-MLA-042`, `SB-MLA-T46` |
| **F11** Cluster Randomness Index | `ADOPTED` | `SB-MLA-043`, `SB-MLA-T41`/`T42`; the equation-vs-lookup-table boundary call is at §7.3 R-12 |
| **F12** Hierarchical linkage update rules | `ADOPTED` | `SB-MLA-046`; five named ids, Ward default cited to the TechCore page, **not** the Ipsom page |
| **F13** PCA construction and correlation-circle coordinate | `ADOPTED` | `SB-MLA-047`, `SB-MLA-048`; `SB-MLA-T25`/`T26` |
| **F14** KNN log prediction with uncertainty (Geolog) | `ADOPTED` with a declared deviation | `SB-MLA-049`, `SB-MLA-T47`; §7.3 R-9. The weight function itself is E-2/ML-12 and ships `ABSENT` |
| **F15** Best-predictors / leave-one-out feature scoring | `ADOPTED` | `SB-MLA-050`, `SB-MLA-T48` |
| **F16** Ipsom / K.mod outlier quality log | `ADOPTED` | `SB-MLA-053` — the field is `tolerance_sd`, never `tolerance`; ML-9's dimensional argument is the reason |
| **F17** K.mod RMSE | `ADOPTED` | `SB-MLA-027` — a score names its protocol; RMSE without a protocol is not a score |
| **F18** Techlog Ipsom FCM — transcribed, not adopted | `REJECTED` — quarantined | §7.3 R-7. The quarantine binds ML-1 and ML-3, **not** ML-2, and the distinction is stated there |

### 8.7 §5.2 Parameter table — 50 rows

Each dossier row was carried into §5 of this chapter, ships `ABSENT`, or is marked
`NON-ADOPTABLE — cited for verification`. **No dossier parameter was dropped, and none was
promoted to a SandiBumi default that the dossier did not source.**

| Dossier parameter | Disposition | Where |
|---|---|---|
| `fuzzy.n_bins` = 10 | `ADOPTED` | §5, cited to the Geolog NBINS page (T3) |
| `fuzzy.n_bins` range 2 … 100 | `ADOPTED` as a bound | §5; `SB-MLA-063` makes the cap a declared limit |
| `fuzzy.min_samples_per_bin` = 30 | `ADOPTED` | §5; `SB-MLA-038` reports actual populations against it |
| `fuzzy.max_input_curves` = 20 | `ADOPTED` as a bound | §5; `SB-MLA-063`. The 2018 → 2025 drift (8 → 20) is why it is a declared limit, not a constant |
| `fuzzy.max_facies_codes` = 10 | `ADOPTED` as a bound | §5; `SB-MLA-063` |
| `fuzzy.percentile_error Er` | `ABSENT` | §5 — IP's 25 is a screenshot (G-9.2), never a default |
| `fuzzy.weight_bin_by_count` | `ABSENT` | §5; `SB-MLA-040`. Contested between prose and panel (G-6.5) |
| `fuzzy.within_bin_regression` = off | `ADOPTED` as off | §5; F5 |
| `som.sigma_0` = grid_width / 2 | `ADOPTED` | §5, cited to `som.htm` (T2) |
| `som.learning_rate_0` | `ABSENT`, range (0, 1) adopted | §5 — the 0.1 is a screenshot (G-9.2) |
| `som.total_iterations` | `ABSENT`, `REQUIRED` | §5; `SB-MLA-041`. The 60000 is a screenshot |
| `som.map_width_max` = 200 | `ADOPTED` as a bound | §5; `SB-MLA-063` |
| `som.max_input_curves` = 8 | `ADOPTED` as a bound | §5; `SB-MLA-063` |
| `som.geometry` (square / hex / spherical) | `ADOPTED` as an enumeration | §5; `SB-MLA-036` — id, not display string. Spherical `DEFERRED` (G-9.8) |
| `som.dims` (1D / 2D) | `ADOPTED` as an enumeration | §5; `SB-MLA-036` |
| `som.calibration_weight` = 1 / d² | `ADOPTED` | §5 |
| `cluster.k_stage1` = 15 … 20 | `EVIDENCE-ONLY` | §5 — a workflow recommendation, not a default; `O-6` |
| `cluster.k_consolidated` = 4 … 5 | `EVIDENCE-ONLY` | §5; `O-6` |
| `cluster.k_default` = 5 | `NON-ADOPTABLE — cited for verification` | §5. SandiBumi's own `K` default is **5** at `facies.rs:40` from an independent decision; the coincidence is recorded so it is not read as adoption |
| `cluster.n_runs` = 50 | `NON-ADOPTABLE — cited for verification` | §5. SandiBumi runs **8** restarts at `facies.rs:23` and **10** `n_init` at `ml.rs:163` — the divergence `SB-MLA-023` exists to close |
| `cluster.seed_subset_fraction` = 0.10 | `EVIDENCE-ONLY` | §5 — seeding vocabulary; ML-5 (E-7) leaves IP's behaviour contested |
| `cluster.pca_variance_cutoff` = 0.95 | `EVIDENCE-ONLY` | §5. SandiBumi's PCA is fixed at 3 components (`ml.rs:205`), not variance-driven |
| `cluster.linkage` = ward | `ADOPTED` | §5; `SB-MLA-046`. Cited to the TechCore `Default value` table; the Ipsom page must never be cited for this |
| `cluster.metric` = euclidean | `ADOPTED` | §5; `SB-MLA-036` — Geolog's `Euclidian` spelling is an input alias, never a key |
| `cluster.normalize_using` = data_range | `ADOPTED` as an enumeration | §5; `SB-MLA-032`, `SB-MLA-033`. SandiBumi's own default is z-score (`facies.rs:42`, `ml.rs:67`) — a declared divergence, not a silent one |
| `cluster.max_input_curves` = 8 | `ADOPTED` as a bound | §5; `SB-MLA-063` |
| `cluster.max_output_sets` = 7 | `ADOPTED` as a bound | §5; `SB-MLA-063` |
| `cluster.id_ordering` = ascending mean of a chosen curve | `ADOPTED` | §5; already shipped at `facies.rs:409`–`:430` and `ml.rs:181`–`:185` |
| `cluster.badhole_flag` | `ADOPTED` | §5; `SB-MLA-004` records the mask and its effect |
| `knn.k_log_prediction` = 10 | `ADOPTED` | §5. SandiBumi's own KNN classifier ships **7** (`ml.rs:138`) — different method, different parameter, both cited |
| `knn.k_facies_propagation_max` = 10 | `ADOPTED` as a bound | §5; `SB-MLA-063` |
| `knn.n_most_probable_facies_logs` = 1 | `ADOPTED` | §5 |
| `barycenter.use_class_weight` = No | `ADOPTED` | §5 |
| `stm.accept_confidence` = 90 | `DEFERRED` | §5 — STM is not v1; acquisition 4 (Thevoux-Chabuel) would ground the reject concept |
| `stm.reject_confidence` = 95 | `DEFERRED` | §5, as above |
| `sammon.iterations` = 2000 | `DEFERRED` | §5 — Sammon projection is not a v1 target |
| `outlier.tolerance a` = 2 | `ADOPTED` | §5; `SB-MLA-053`. Named `tolerance_sd`; ML-9's 2 : 1 SD reading is carried with the dimensional argument |
| `outlier.expected_fraction N` ≈ 0.05 | `ADOPTED` as the vendor's stated pairing | §5 — recorded as the vendor pairs it, **not** re-derived as a Gaussian constant |
| `nn.hidden_layers` = 1 | `ADOPTED` with a recorded boundary call | §5; §7.3 R-3 — carried only as a generic, independently re-derivable architectural fact |
| `nn.epochs` | `NON-ADOPTABLE — cited for verification` | §5 — G-6.1 contested; SandiBumi's `max_iter=500` at `ml.rs:109` is its own |
| `nn.training_passes` = 3 | `EVIDENCE-ONLY` | §5 — an IP workflow recipe, not a constant |
| `nn.cross_validation_pct` | `ABSENT`; the "0 disables" semantics `ADOPTED` as a refusal | §5; `SB-MLA-019`. A disabled protocol must not report a score as if it had run |
| `nn.training_zones` = 4 … 8 narrow zones | `EVIDENCE-ONLY` | §5 — workflow guidance |
| `nn.classification_max_categories` = 10 | `ADOPTED` as a bound | §5; `SB-MLA-063` |
| `nn.max_input_curves` = 20 | `ADOPTED` as a bound | §5; `SB-MLA-063` |
| `nn.sensitivity_dither` = 10 % of normalised range | `EVIDENCE-ONLY` | §5. SandiBumi uses permutation importance (`ml.rs:1219`), a different and independently sourced method |
| `seed` = 42 (ML) / 7 (facies) | `ADOPTED` as a defect | §5; `SB-MLA-024` — two seed defaults in one product is the divergence, not the values |
| `standardize` = on | `ADOPTED` | §5, at `ml.rs:67`; `SB-MLA-032` |
| `null` = −999.25 with an explicit `NULL.` | `ADOPTED` | §5; `SB-MLA-056`, `SB-MLA-057` |
| `trig_units` = radians internally, degrees named at the boundary | `ESCALATED` | §7.2 E-11 — the formula layer has no owning chapter |

### 8.8 §5.3 Fixtures — 25 rows, and field-data acceptance — 3 rows

**Every one of the dossier's 24 fixture ids became an acceptance test in §6.** None was dropped and
none was weakened. Where a test's expected value has no external source it is labelled
`CHARACTERIZATION` in §6, per `CONTRACT.md` §6.

| Dossier fixture | Disposition | Where |
|---|---|---|
| `T-ML-PCA-1` IP worked example, four-curve | `ADOPTED` | `SB-MLA-T25` — numbers carried exactly |
| `T-ML-PCA-2` Correlation-circle coordinates | `ADOPTED` | `SB-MLA-T26` — self-contained loadings |
| ⚠ guard: PCA-1 and PCA-2 numbers must never be crossed | `ADOPTED` | Carried verbatim into §6 as a standing note on `SB-MLA-T25`/`T26` |
| `T-ML-FUZZY-1` combination-rule discriminator | `ADOPTED` | `SB-MLA-T33` — bin B wins under reciprocal sum, bin A under product |
| `T-ML-FUZZY-2` equal-bin degeneracy | `ADOPTED` | `SB-MLA-T34` |
| `T-ML-FUZZY-3` `ResPC ± Er` outside [0, 1] | `ADOPTED` | `SB-MLA-T35` |
| `T-ML-RI-1` randomness index on known bed structure | `ADOPTED` | `SB-MLA-T41` — Av 20, Random 1.43545, RI 13.933 |
| `T-ML-RI-2` RI on a shuffled label sequence | `ADOPTED` | `SB-MLA-T42` |
| `T-ML-SOM-1` degenerate-decay guard | `ADOPTED` | `SB-MLA-T45` |
| `T-ML-SOM-2` SOM distortion on a hand-built 2×2 map | `ADOPTED` | `SB-MLA-T46` |
| `T-ML-KNN-1` best-predictors self-reconstruction guard | `ADOPTED` | `SB-MLA-T48` |
| `T-ML-KNN-2` `k = 1` returns an actual training value | `ADOPTED` | `SB-MLA-T49` |
| `T-ML-KNN-3` collapsed-dimension prediction | `ADOPTED` | `SB-MLA-T50` |
| `T-ML-KNN-4` weight-normalisation guard (ML-12) | `ADOPTED` | `SB-MLA-T47` — mean, not twice the mean |
| `T-ML-CONF-1` confusion-matrix axis guard | `ADOPTED` | `SB-MLA-T51` — non-square, non-symmetric, both normalisations labelled |
| `T-ML-SEED-1` same seed, two runs, every algorithm | `ADOPTED` | `SB-MLA-T08` |
| `T-ML-SEED-2` different seeds, k-means | `ADOPTED` | `SB-MLA-T24` |
| `T-ML-NORM-1` the add-a-well trap | `ADOPTED` | `SB-MLA-T31` — the highest-value fixture in the dossier for `SB-MLA-033` |
| `T-ML-NORM-2` the log-transform trap | `ADOPTED` | `SB-MLA-T32` |
| `T-ML-NULL-1` −999 into an intermediate | `ADOPTED` | `SB-MLA-T55` |
| `T-ML-PARSE-1` `MIN()` without inner parentheses | `ESCALATED` | §7.2 E-11 — a formula-layer test with no owning chapter; **not orphaned, not silently dropped** |
| `T-ML-TRIG-1` `ASIN(SIN(x))` round trip | `ESCALATED` | §7.2 E-11, as above |
| `T-ML-CFIT-1` both closeness-of-fit curves on a mixed run | `ADOPTED` | `SB-MLA-T30` — including the legitimately-absent third state (G-6.4) |
| `T-ML-EMPTY-1` force an empty cluster (K > distinct) | `ADOPTED` | `SB-MLA-T13`, `SB-MLA-T14` — already partly shipped at `hfu.rs:489` |
| `T-ML-BLIND-1` blind-well leaderboard on a 3-well set | `ADOPTED` | `SB-MLA-T27`, `SB-MLA-T29` |

| Dossier field-data acceptance | Disposition | Where |
|---|---|---|
| Class 0 must be the lowest-mean-GR class, ordering monotone to shaliest | `ADOPTED` | `SB-MLA-T22` — the invariant is shipped at `facies.rs:409`–`:430` and `ml.rs:181`–`:185`; the test pins it on real data |
| Coal must fall in its own class at `k ≥ 5`; the delivered workflow excludes coal and tight streaks **before** training | `ADOPTED` | `SB-MLA-T23`; `SB-MLA-004` is what makes the exclusion recordable rather than a habit |
| Synthetic-log substitution is conditional on the badhole flag **and** physical direction | `EVIDENCE-ONLY` | Three independent project records agree, but the requirement belongs to the log-conditioning domain, not to ML. Recorded here so the agreement is not lost |

### 8.9 §5.4 FINDINGS rules — 10 rows

| Dossier rule | Disposition | Where |
|---|---|---|
| **1 — No raster-only truth** | `ADOPTED` | `SB-MLA-036`; G-6.3 is the catalogue's own example and F18 is the quarantine case |
| **3 — Unit-typed quantities, no magic constants** | `ADOPTED` | `SB-MLA-035`, `SB-MLA-053`. `PERM` [mD] and `LOG10_PERM` [log10(mD)] are two registry entries, never one with a flag |
| **6 — Null discipline** | `ADOPTED` | `SB-MLA-056`, `SB-MLA-057`; no opt-out at any default |
| **7 — Ordinal + semantic-name addressing** | `ADOPTED` | `SB-MLA-036`; the `Euclidian` spelling is the second independent instance |
| **8 — No bare reused symbol** | `ADOPTED` | `SB-MLA-029`, `SB-MLA-030` — `CFIT_BINS`/`CFIT_ABS`, `PROB_REL`/`PROB_ABS`, and a facies mnemonic that names its engine |
| **9 — Defaults are cited or absent** | `ADOPTED` | The whole of §5; 15 rows ship `ABSENT` |
| **10 — Docs generated from code** | `EVIDENCE-ONLY` | A build-system obligation; §5's table is emitted from the parameter registry, which is `SB-CORE`'s ground, not this chapter's |
| **11 — Worked examples must reproduce** | `ADOPTED` | `SB-MLA-T25`/`T26` verify arithmetically, which is exactly why they qualify and the Techlog example (ML-13) does not |
| **14 — Silent failures are bugs** | `ADOPTED` | `SB-MLA-013` … `SB-MLA-021` — nine requirements from one rule, the densest mapping in the chapter |
| **15 — Curve resolution and depth snapping are logged decisions** | `ADOPTED` | `SB-MLA-054`, `SB-MLA-055`. Geolog's TOPS/POINT distinction is the source: interpolate a facies code and you get a meaningless intermediate value |

### 8.10 §6 New open items — 17 rows — and acquisitions — 9 rows

| Dossier item | Disposition | Where |
|---|---|---|
| **ML-1** Ipsom FCM barycenter has no normalising denominator | `ESCALATED` | §7.2 E-3; §7.3 R-7 quarantines it meanwhile |
| **ML-2** Ipsom FCM membership has no outer reciprocal, ratio inverted | `ESCALATED` | §7.2 E-3 / E-6(b). The quarantine explicitly does **not** bind here — row-normalising recovers a proper membership exactly |
| **ML-3** Techlog `QQ` prose contradicts the printed exponent; no default | `ESCALATED` | §7.2 E-6(a); §5 `fuzzy QQ` ships `ABSENT` |
| **ML-4** Techlog SOM training math absent — verified negative result | `EVIDENCE-ONLY` | The asymmetry it creates is the reason `SB-MLA-041` cites Kohonen and not IP; E-1 |
| **ML-5** IP 2018 documents Seed Clusters as functional; 2025 says ignored | `ESCALATED` | §7.2 E-7 — low priority, nothing depends on it |
| **ML-6** Geolog MRGC internals not held | `ESCALATED` | §7.2 E-2, and E-8 first because it may narrow what E-2 must answer; §7.3 R-13 refuses reconstruction meanwhile |
| **ML-7** "Facimage offers only Unsupervised Classifications" vs supervised use | `EVIDENCE-ONLY` | The terminology-split reading is better supported; presented, not adjudicated. Partially closed by E-8 |
| **ML-8** Geolog Facimage ships almost no stated defaults (~18) | `ESCALATED` | §7.2 E-5 — the best ratio of parameters closed to effort spent in the chapter |
| **ML-9** Techlog outlier tolerance: SD vs variance, 2 : 1 | `ADOPTED` as the SD reading | `SB-MLA-053`; §5 `outlier.tolerance a`. The dimensional argument is what settles it, not the tally |
| **ML-10** K.mod training hyperparameters undefaulted (corrected scope) | `ESCALATED` | §7.2 E-6, third question; `outlier.tolerance` and `Apply mode` are `ADOPTED` in §5 |
| **ML-11** Is the reciprocal sum applied to raw or normalised per-curve `P`? | `ESCALATED` | §7.2 E-4. Affects the confidence `SB-MLA-037` can be stated with, not the requirement |
| **ML-12** Geolog KNN weight function unprinted; two pages disagree on the denominator | `ESCALATED` + `ADOPTED` as a declared deviation | §7.2 E-2; `SB-MLA-049`, `SB-MLA-T47`; §7.3 R-9 |
| **ML-13** Techlog PCA/MCA worked examples fail to close, three places | `ADOPTED` as a quarantine | §2.6 note; the examples must never become fixtures — `SB-MLA-T25`/`T26` use IP's, which verify |
| **ML-14** Techlog characterises the same four linkages differently in two modules | `EVIDENCE-ONLY` | The linkage rules themselves are unambiguous; only the editorial guidance differs. `SB-MLA-046` cites the page with the `Default value` column |
| **ML-15** Two different default SOM map sizes in two Techlog modules | `EVIDENCE-ONLY` | **Neither adopted.** Map size is a capacity choice, not a physical constant; recorded so neither value is ever cited bare |
| **ML-16** `Min and Max threshold` default −9999 collides with the `MissingValue` sentinel | `ADOPTED` | `SB-MLA-057` — the strongest argument in the corpus for a separate null flag, and it is a **vendor** instance of `SB-CORE`'s own rule |
| **ML-17** Decision tree prints one binary split with two thresholds | `EVIDENCE-ONLY` | Illustrative, enters no parameter row; recorded as the third instance of a Techlog worked example failing self-consistency |

| Dossier acquisition | Disposition | Where |
|---|---|---|
| 1. Ye & Rabiller, SPWLA 41st, paper PP (MRGC) | `ESCALATED` | §7.2 E-2 — cited by symposium number and paper letter, as the vendor prints it; the year 2000 is not repeated as fact |
| 2. Cuddy (1997) SPWLA paper S; Cuddy (2000) SPE 65411 | `ESCALATED` | §7.2 E-4 |
| 3. Kohonen (1990) *Proc. IEEE* 78(9) 1464–1480 | `ESCALATED` | §7.2 E-1 |
| 4. Thevoux-Chabuel, Veillerette & Rabiller (1997) SPWLA paper BB (STM) | `ESCALATED` — low priority | §7.2 closing paragraph; the reject concept is the best extrapolation guard found in any tool |
| 5. Rabiller, Boles & Dewhirst, URTeC 2013 (control ID 1580723) | `ESCALATED` — low priority | §7.2 closing paragraph |
| 6. Bezdek (1981) | `ESCALATED` | §7.2 E-3 |
| 7. Diday (1971), Mourot (1993) | `ESCALATED` — low priority | §7.2 closing paragraph |
| 8. Wu & Takatsuka (2006) *Neural Networks* 19 (spherical SOM) | `ESCALATED` — low priority | §7.2 closing paragraph; would close G-9.8 from first principles rather than from an observed value |
| 9. Re-ingest the 2006 Rabiller Facimage guide, pp. 56–131 | `ESCALATED` — **highest ratio** | §7.2 E-8. Costs no acquisition; the document is already on this machine |

### 8.11 §7 Source register — 10 subsections, accounted for, not rowed

Per §8.0 decision 2, the register is a provenance manifest and not a set of findings. Its ten
subsections — the two IP CHM ingests, the IP install-tree ingest, the Techlog ingest, the shipped
Techlog and Geolog trees, petro-kb, project-kb, the SandiBumi source and test plan, and the memory
notes — are dispositioned through every finding that cites them, and every tier marking in §2 and
§5 traces back to one of them. **All were read read-only**; the only file this work writes is this
chapter.

### 8.12 §8 Critique disposition — 20 rows — and rebutted claims — 2 rows

| Dossier item | Disposition | Where |
|---|---|---|
| **B-1** False cross-tool negative on contingency modules | `ADOPTED` | `SB-MLA-051`; §2.8 sources. The row-% vs column-% naming trap is the requirement's core |
| **B-2** Decision tree, MCA, Ancor missing from the §1.2 inventory | `ADOPTED` | §2 sources; `SB-MLA-051`. Decision tree `DEFERRED`, MCA `DEFERRED` |
| **M-1** "All three tools default to Ward" false for Techlog | `ADOPTED` | `SB-MLA-046` cites the TechCore `Default value` table and carries the explicit "do not cite the Ipsom page" warning |
| **M-2** ML-10 contradicted by the outlier tolerance default | `ADOPTED` | §5 `outlier.tolerance a`, `outlier.expected_fraction N`; `SB-MLA-053` |
| **M-3** ML-9 tally wrong — a page had never been read | `ADOPTED` | `SB-MLA-053`; the dimensional argument is carried, not just the 2 : 1 count |
| **M-4** F14 cites ML-12, which did not exist | `ADOPTED` | `SB-MLA-049`, `SB-MLA-T47`; §7.2 E-2 |
| **M-5** IP's log10 flag changes the *reported* statistics | `ADOPTED` | `SB-MLA-034`, `SB-MLA-035` — a negative mean under a header saying mD is the exemplar |
| **M-6** Internal contradiction on the Techlog PCA page unreported | `ADOPTED` | ML-13; the example is quarantined and `SB-MLA-T25`/`T26` use IP's instead |
| **m-1** ML-1/ML-2 pixel census does not reproduce; no threshold or index base stated | `EVIDENCE-ONLY` | A dossier-internal method defect. This chapter cites no pixel census and rests no claim on one |
| **m-2** ML-2's stated consequence overshoots its evidence | `ADOPTED` as narrowed | §7.3 R-7 carries the narrowed form: the quarantine binds ML-1 and ML-3, **not** ML-2 |
| **m-3** Geolog's Cuddy citation carries a year contradiction, silently normalised | `ADOPTED` | §7.2 E-4 cites Cuddy (1997) SPWLA paper S and Cuddy (2000) SPE 65411 as two distinct works, never a normalised single year |
| **m-4** "Ye & Rabiller SPWLA-2000" attributes a year the source does not print | `ADOPTED` | §7.2 E-2 cites by symposium number and paper letter, and states in terms that the year is an inference |
| **m-5** `HRA_PROBABILITIES*` omitted from both the ingest and the HRA output list | `EVIDENCE-ONLY` | `SB-MLA-030` types probability outputs generally; SandiBumi ships no HRA path, so there is nothing to name |
| **m-6** "538 shipped `.py` files" is a `PythonScripts\` count, not a shipped count | `ADOPTED` | This chapter makes no file-count claim about any vendor tree, and §8.12's second rebuttal states why exhaustiveness claims are not made at all |
| **m-7** `T-ML-PCA-2` is not runnable and invites a wrong pairing with `T-ML-PCA-1` | `ADOPTED` | `SB-MLA-T26` carries its own loadings and the ⚠ guard is reproduced verbatim in §6 |
| **m-8** §2.9 drops the Multi-Line grid semantics | `ESCALATED` | §7.2 E-11 — part of the formula layer that no chapter owns |
| **m-9** IP's SOM normalisation statement dropped; no IP-SOM row in §3.5 | `ADOPTED` | `SB-MLA-032` covers the normalisation basis for every algorithm, SOM included, rather than per-method |
| **m-10** "No `Cfit` for the weighted average" rule dropped | `ADOPTED` | `SB-MLA-030` and the G-6.4 row above — legitimately-absent is a modelled third state |
| **m-11** Geolog's KNN "summation" (5.05) vs "weighted average" (6.8) contradiction not raised | `ADOPTED` | Became ML-12; `SB-MLA-049`, `SB-MLA-T47`, §7.3 R-9 |
| **m-12** §4.7 miscounts the Part-4 blocker table | `ADOPTED` | §8.5 rows the corrected set; §8.0 states the one remaining count ambiguity rather than smoothing it |
| **Rebutted 1** — M-1's replacement claim "Techlog states no default HC method anywhere it ships" | `REJECTED` — correctly rebutted | The TechCore page names Ward twice in a `Default value` column. `SB-MLA-046` rests on the rebuttal, not on the critique |
| **Rebutted 2** — ML-4's inherited "the shipped Techlog doc tree is exhausted" | `REJECTED` — correctly withdrawn | A content grep answers a content question and never licensed a claim about the tree. This chapter makes no exhaustiveness claim about any vendor tree |

**Note on the twelve minors.** An earlier draft of this section rowed them as a single entry, on
the grounds that the dossier records them as uniformly applied. That was wrong for a completeness
gate: it made the physical row count 207 against an item count of 218, reconciled only by a
footnote. **They are now rowed individually, and §8 carries exactly 218 rows against 218 items.**
Four of them (m-2, m-4, m-10, m-11) turned out to bear directly on requirements in this chapter,
which is the argument against collapsing rows in the first place.

### 8.13 Requirements with no dossier antecedent — 24

These come from reading the shipped source, not from the cross-tool corpus. The dossier could not
have raised them: it examined three vendors, not SandiBumi. **They are listed separately so the
chapter cannot be mistaken for a restatement of its dossier**, and because `03_EVIDENCE_BASE.md`
§14.3 requires the inward-facing side to be treated as hard as the outward-facing one.

| Requirement | Origin |
|---|---|
| ~~`SB-MLA-002`~~ | ~~`ml.rs:733`–`:748` — `insert_ml_model` records no input log set~~ **CLOSED 2026-08-07** — `training_json` records the resolved set PER WELL, because `input_set` resolves per well; a well reading from the live store says so, and `training_set_drift` names a deleted or superseded set on apply |
| `SB-MLA-003` | `ml.rs:720`–`:732` — well names are recorded; the rows are not |
| ~~`SB-MLA-004`~~ | ~~`ml.rs:733`–`:748` — no mask curve is persisted with the model~~ **CLOSED 2026-08-07** — the same per-well record carries `masked` and `incomplete` SEPARATELY, since they call for opposite fixes; the run reports the total with the worst well named |
| ~~`SB-MLA-005`~~ | ~~`ml.rs:745` — the sklearn version is recorded; nothing else about the runtime is~~ **CLOSED 2026-08-07** — one shared probe (interpreter, numpy, scipy, sklearn, joblib, xgboost) written by both runners, compared at PICK time via the cached `ml_runtime` command so the warning lands before the apply. The `xgboost` *substitution* stays open under `SB-MLA-012` — that is a lie about the algorithm, not the runtime |
| `SB-MLA-007` | `db.rs:2740` — `delete_ml_model` is unconditional |
| ~~`SB-MLA-010`~~ | ~~`report.rs` and `export.rs` contain no ML reference at all — the chapter's spine~~ **CLOSED 2026-08-07** — the PDF and the Word twin both print the provenance block, driven from `computed_curves.set_id` so it describes the live curves rather than every run. LAS export still carries nothing; that realisation is `DIO`'s per §2 |
| `SB-MLA-011` | `ml.rs:580`–`:587`, `:720`–`:732` — empty-training wells are filtered out of `trained_on` |
| ~~`SB-MLA-013`~~ | ~~`facies.rs:137`–`:139`, `:192`–`:198` — an all-NaN return is reported as success~~ **CLOSED 2026-08-07** — both native engines return `Result`; the python path refuses before writing. The row understated it: `ml.rs` had the same defect and the as-built had certified that path as correct |
| `SB-MLA-014` | `hfu.rs:273` — `eff_k` silently reduces; `facies.rs:77` silently clamps |
| `SB-MLA-015` | `facies.rs:215` — `VAR_FLOOR` fires without a report |
| `SB-MLA-016` | `facies.rs:289` — iteration exhaustion is indistinguishable from convergence |
| `SB-MLA-017` | `ml.rs:639`–`:649` — per-well cancellation after curves have been written |
| `SB-MLA-018` | `ml.rs:636`–`:638` — the source comment states the fit is not interruptible; the UI does not |
| `SB-MLA-020` | `ml.rs:189`–`:196` — silhouette on a 5000-point subsample, reported unqualified |
| `SB-MLA-021` | `ml.rs:177`–`:188` — DBSCAN noise becomes NaN, indistinguishable from missing input |
| `SB-MLA-022` | `ml.rs:1782`, `ml.rs:1832` — both contract tests carry `#[ignore]` |
| `SB-MLA-023` | `facies.rs:23`/`:24` against `ml.rs:163` — two k-means engines, different constants |
| `SB-MLA-024` | `facies.rs:41` (7) against `ml.rs:64` (42) — two seed defaults |
| `SB-MLA-025` | `hfu.rs:103`, `lorenz.rs:152`, `facies.rs:318` — three within-cluster-sum-of-squares partitioners |
| `SB-MLA-026` | ~~`ml.rs:1132`–`:1176` against `ml.rs:87`–`:147` — the leaderboard fits a different model from the run~~ **CLOSED 2026-08-07.** One `ML_BUILD_MODEL`, both runners composed from it. Pinned by `the_leaderboard_builds_the_same_estimators_the_run_will_fit` |
| `SB-MLA-028` | ~~`ml.rs:1129`–`:1130` before `ml.rs:1171`–`:1176` — the scaler is fitted before the split~~ **CLOSED 2026-08-07.** One scaler per fold, fitted on `X[tr]` only, built after the splitter. Pinned by `no_transform_is_fitted_outside_the_folds_training_rows` |
| `SB-MLA-061` | `python_engine.rs:47`–`:48` — the missing-interpreter message |
| `SB-MLA-062` | `ml.rs:630` — the single-writer lock is taken across the fit |
| `SB-MLA-064` | `db.rs:2667` — an already-shipped strength, stated so a later change cannot silently remove it |

**`SB-MLA-023`, `SB-MLA-024`, `SB-MLA-025`, `SB-MLA-026` and `SB-MLA-028` are the five that matter
most**, and none of them could have come from the dossier. Four are `PRESENT-DIVERGENT` —
`CONTRACT.md` §3 calls that "the most valuable status in the document" — and the fifth
(`SB-MLA-028`) is a correctness defect that makes a reported blind-well score optimistic, which is
exactly the number a customer would rely on.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.

