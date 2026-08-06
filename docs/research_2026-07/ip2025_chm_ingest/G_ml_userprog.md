# G — Machine-Learning Suite + User Programming (IP 2025 CHM ingest)

Agent G of the 14-agent Interactive Petrophysics 2025 vendor-manual ingest.
Consumer: SandiBumi. Provenance discipline is absolute — every fact below carries
`(pagename.htm)` for prose or `[img-read: file.png]` for a raster transcription.

**Tier-C boundary observed.** Experienced Eye (EE), EEFS feature selection, and Domain
Transfer Analysis (DTA) are described at capability level only. Nothing in this document
functions as an implementation specification of their internals.

---

## 1. Scope & page inventory

All 28 assigned pages read and accounted for. Source: `C:\Users\ARUNIKA\AppData\Local\Temp\c25\<stem>_text.txt`.

### 1a. Pre-harvested (verified against the EE dossier, not re-extracted)

| Page | Status |
|---|---|
| `experiencedeye` | Tier-C. Capability-only. Cross-checked vs `ip_ingest/EE_capability_dossier.md` — **0 numeric facts missed**. |
| `curvepredictionusingdta` | Tier-C (registered item #6). Capability-only. Cross-checked — **0 numeric facts missed**. |
| `machinelearning` (hub) | Menu inventory only. Cross-checked — **0 numeric facts missed**. One structural fact worth carrying: IP 2025 has a **dedicated "Machine Learning" menu** listing 10 modules (EE, Fuzzy Logic, MLR, Neural Networks, Cluster Analysis, SOM, PCA, Contingency Table, Textural Facies Analysis, DTA). |

Method for the cross-check: every distinct numeric token on the three pages (20 / 11 / 4
respectively) was tested for presence in the dossier text. **All present. No new numbers.**
See §7 for the one genuinely new *structural* finding (the NN engine disclosure diff), which
is a diff finding rather than a dossier gap.

### 1b. Full-extraction targets

| Page | Chars | What it yielded |
|---|---|---|
| `interp-demo` | 72,349 | Worked user-app walkthrough; same routine in 7 languages. All parameter values are EXAMPLES (§2.8). |
| `som` | 42,374 | **Full SOM training math** — 11 equation rasters (§2.3). |
| `discrete_depth_analysis` | 32,409 | SandPit 3D geomechanics (not ML) — TWC/perforation sweep constraints (§3.6). |
| `cluster_analysis` | 31,905 | K-means + 5 hierarchical linkage methods + randomness index (§2.4). |
| `multi_depth_analysis_workflow` | 27,304 | **Full sanding failure equation set** — 8 rasters (§2.7), with defaults. |
| `user-app-properties` | 22,323 | User-app limits and runtime contract (§3.5). |
| `statisticalcurveprediction` | 20,063 | **Cuddy fuzzy-logic math** — 8 rasters (§2.1). |
| `neural_networks` | 19,240 | Architecture-as-documented, hyperparameters, stopping (§2.2). |
| `multi_line_user_formula` | 18,704 | Formula language + IF/AND/OR + array semantics (§4). |
| `compiler-information` | 14,169 | Toolchain/runtime facts (§4.6). |
| `principal_component_analysis` | 11,099 | PCA method + verifiable worked example (§2.5). |
| `user-definedformula` | 11,205 | Single-line formula function library (§4). |
| `textural_facies_analysis` | 10,616 | Frequency-tile binning algorithm (§2.6). |
| `graphical_workflow_manager` | 7,938 | Workflow curve-matching rules (§3.7). |
| `multiplelinearregression` | 6,860 | Least-squares MLR (§2.9). |
| `standalone_contingency_table` | 4,752 | Calibrated/uncalibrated modes (§2.10). |
| `specialinterpretation` (hub) | 2,900 | Advanced Interpretation menu inventory. |
| `managing-user-apps` | 2,607 | App storage/sharing rules (§3.5). |
| `create-new-user-app` | 2,179 | Creation workflow, filename constraints. |
| `running-user-apps` | 1,198 | Runtime curve filtering (SM button). |
| `userapps` (hub) | 936 | Language list. |
| `create-user-app-help` | 629 | Help-file mechanism. |
| `example-user-apps` | 601 | 4 shipped example apps. |
| `create-and-edit-user-apps` | 295 | Navigation only. |
| `edit-user-app` | 258 | Navigation only. |

**Rule 8 sweep:** `smectite` / `montmorillonite` — **zero hits across all 28 pages.** No clay
endpoints appear anywhere in this page set.

**Keyboard shortcuts** (all `_text.txt`): `Ctrl+Alt+F` Fuzzy Logic, `Ctrl+Alt+N` Neural Network,
`Ctrl+Alt+A` PCA, `Ctrl+Alt+S` SOM, `Ctrl+Alt+T` Cluster Analysis, `Ctrl+Alt+R` Multiple Linear
Regression.

---

## 2. Equations & methods per module

### 2.1 Fuzzy Logic Curve Prediction — the Cuddy method (OPEN, published)

Vendor cites: Cuddy, S. (1997), "The Application of the Mathematics of Fuzzy Logic to
Petrophysics", Paper S, 38th Annual SPWLA Symposium (statisticalcurveprediction.htm).

**Bin probability for one curve** — transcribed from the raster:

```
P(C_b) = sqrt(n_b) × exp( −(C − μ_b)² / (2 × σ_b²) )
```
`[img-read: embim630.png]`

Where (statisticalcurveprediction.htm):
- `P(C_b)` = probability that curve C is in bin b
- `n_b` = number of samples in bin b
- `C` = input value for curve C
- `μ_b` = mean value of curve C for bin b `[img-read: embim631.png — glyph is μ]`
- `σ_b` = standard deviation of curve C for bin b `[img-read: embim632.png — glyph is σ]`

The bare glyphs used in the narrative are confirmed independently:
`[img-read: embim628.png — μ]`, `[img-read: embim629.png — σ]`.

**Combination across curves — HARMONIC, not multiplicative:**

```
1/P_b = 1/P(C1_b) + 1/P(C2_b) + 1/P(C3_b) + ⋯
```
`[img-read: embim633.png]`

> **This is the single most important equation on the page for SandiBumi.** The combination
> rule is a *reciprocal sum*, i.e. `P_b` is the harmonic-style combination of the per-curve
> probabilities — **not** the naive-Bayes product that a reimplementer would assume by
> default. Getting this wrong is silent: it still computes, still plots, still ships.

**Weighted average of the two most likely results** — printed twice, in two notations, and the
two forms agree exactly (cross-check passed):

```
R_av = (R_ml × P_ml + R_sl × P_sl) / (P_ml + P_sl)
```
`[img-read: embim634.png]` (Equations & Methodology section)

```
Result = (Res_ml × Prob_ml + Res_2l × Prob_2l) / (Prob_ml + Prob_2l)
```
`[img-read: embim627.png]` (Run Model tab section)

**Binning** (statisticalcurveprediction.htm):
- The Curve-to-Predict values are sorted by value then divided into bins so all bins hold the
  same number of data. All input curves are divided into the same bins using input values at
  the same depths as the Curve-to-Predict depths.
- Two bin modes: **Variable size bins** (for discrete data, e.g. facies numbers — specify
  starting bin number and bin width) and **Equal sampled bins** (preliminary pass computes
  maxima/minima, then spacings set so an equal proportion falls in each bin).
- Equal-sampled binning is explicitly **not guaranteed exact**: "if one has a dataset with
  100 samples and 20 of the samples have the same value, then, if the number of bins is set
  to 10 … IP can not distribute the data equally".

**Most Likely Result Range (the fuzziness band)** (statisticalcurveprediction.htm):
1. At each level the bin probabilities are converted to a normalised (0–1) cumulative
   frequency distribution.
2. The Result Bin Percentile `ResPC` is found.
3. Low result = the bin at percentile `ResPC − Er`; High result = the bin at `ResPC + Er`,
   where `Er` is the percentile error entered on the Run Model tab.
4. High/low results are obtained by **extrapolating between the bins' Mean result values**.
5. **Edge rule:** if the shifted cumulative frequency falls outside 0–1, the result value is
   the Mean of the first or last bin **plus or minus two standard deviations** of the spread
   of data in that bin.

**Closeness of fit** = bin distance, always positive; e.g. original in bin 4, result in bin 6
→ Cfit = 2. Null (−999) where the original curve has no data. **No Cfit curve is produced for
the weighted-average result** — the vendor states this "makes no sense" (statisticalcurveprediction.htm).

**Probability is relative, not absolute** — and explicitly depends on the number of input
curves, so "one cannot compare the absolute values of two models created using different
numbers of input curves" (statisticalcurveprediction.htm).

### 2.2 Neural Networks (as documented — see Tier-C note)

**Tier-C note.** The IP2018 counterpart page disclosed the engine (NeuroSolutions 5.5) and
"number of Hidden layers = 1"; **the IP 2025 page has removed both statements** (see §7).
The registered Tier-C item stands on the IP2018 source. SandiBumi must not port the engine or
any shipped weights. Everything below is the vendor's own IP-2025 documented behaviour.

Architecture disclosed in IP 2025: **none** beyond "highly interconnected processing elements
(neurons)" and the existence of a classification mode (neural_networks.htm).

**Hyperparameters** (neural_networks.htm; screenshot values `[img-read: _nnclip00018.png]`):

| Parameter | Prose default | Screenshot value | Semantics |
|---|---|---|---|
| Training Passes | **3** | 3 | Retrains N times; each pass starts from randomised initial settings. "The default value of 3 works well to stop" the network getting stuck. |
| Epochs per pass | **1000** | **100** ← conflict, see §6.1 | Times the training data is presented per training run. |
| Cross-validation % | not stated | **5** | Fraction of training data held back to guard against over-training. **0 % disables cross-validation.** |
| Use classification network | off | unchecked | Categorised prediction (facies). |

**Constraint found only in the screenshot, not in the prose:** the Cross-validation field is
labelled "% of data will be used for cross-validation, **not used with zonal averages**"
`[img-read: _nnclip00018.png]`. So enabling "Use a single average value from each zone"
silently disables cross-validation.

**Stopping criterion:** training may terminate before the requested epoch count — "Epochs
Trained … may not be the same as the epochs per pass requested as the training may have
decided to stop because cross-validation showed it was beginning to over train"
(neural_networks.htm). So: early stopping on cross-validation degradation.

**Reported training metrics** (neural_networks.htm):
- *Epochs Trained* — epochs actually performed in that pass.
- *Epoch of Best Cost* — epoch with best result; absent means the pass did not improve on the
  previous one. Vendor's own words: "for information only and isn't really of any practical use."
- *Best Cost* — minimum error achieved (lower is better).
- *Raw Sensitivity* — per-input influence on the output. Vendor's worked reading: if
  sensitivities are `(5, 6, 0.1, 5.5)` the third input can probably be excluded, and dropping
  it "may in fact improve the results".

**Sensitivity is measured by dithering** — disclosed only in the screenshot: "Raw Sensitivity
(**dithered at 10 % of normalised data range**)" `[img-read: _nnclip00018.png]`. This is a
real algorithmic disclosure absent from the prose: the sensitivity metric is a finite-
difference perturbation at 10 % of each input's normalised range.

**Normalization:** the NN page states no normalization scheme. The only normalization
statement is the optional per-curve **base-10 log** flag, recommended for logarithmic data
such as core permeability (neural_networks.htm). → OPEN ITEM §9.

**Training data selection:** training is performed on **zones**, not the whole log, using a
dedicated "Neural Training Zones" Set. Default zone size is specified **as a number of data
points, not a depth** (neural_networks.htm). Guidance: "For most purposes a small number
(4–8) of narrow zones is enough to generate good results."

**Non-determinism is documented and expected:** "the results can be different each time the
neural network is trained, even if the input data and training zones have not been changed …
this is normal behaviour" (neural_networks.htm). An Undo Training button exists.

**Limits:** up to **20 input curves per well**; classification network predicts a **maximum
of 10 categories** of facies (neural_networks.htm).

**Scenario recipes** (neural_networks.htm) — vendor's own prescriptions:

| Target | Classification net | Round output | Single average per zone | Zones |
|---|---|---|---|---|
| Continuous log | No | No | "may be applicable if a large number of small zones are created" | one small zone per distinct lithology; 4–8 usually enough |
| Core porosity | No | No | No | **one zone covering all core data** (levels without core are auto-ignored) |
| Facies (route 1) | **Yes** | No | No | — |
| Facies (route 2) | No | **Yes** (round to nearest integer) | No | — |

**Closeness of Fit** here = absolute value of the difference between original data and the
`_NN` result curve (neural_networks.htm). Note this is a *different definition* from the
Fuzzy Logic Cfit (which is bin distance) — see §6.4.

### 2.3 Self-Organising Maps — full training math (OPEN)

Vendor reference: Kohonen, T., *Self-Organizing Maps, 3rd Edition*, Springer (som.htm).

**Map size:**
```
total nodes = map width²
```
`[img-read: somclip0001.png]`

**Best Matching Unit — Euclidean distance:**
```
Distance = sqrt( Σ(i=0 … i=n) (V_i − W_i)² )
```
`[img-read: somclip0002.png]` — V = current input vector, W = node weight vector (som.htm).
The node with the lowest Distance is the BMU.

**BMU weight update:**
```
W_(t+1) = W_t + L_t (V_t − W_t)
```
`[img-read: somclip0003.png]`

**Learning-rate bounds:**
```
0 < L < 1
```
`[img-read: somclip0004.png]`

**Learning-rate decay:**
```
L_t = L_0 exp( −t / λ )
```
`[img-read: somclip0005.png]`

**Time constant:**
```
λ = t / log σ_0
```
`[img-read: somclip0006.png]` — verified at 6× upscale. See §6.2: `t` here is ambiguous.

**Neighbourhood-radius decay** (vendor states it is "the same equation" as the learning rate):
```
σ_t = σ_0 exp( −t / λ )
```
`[img-read: somclip0015.png]`

**Neighbour weight update (with influence term):**
```
W_(t+1) + W_t + Θ_t L_t (V_t − W_t)
```
`[img-read: somclip0008.png]` — **transcribed exactly as printed.** The raster genuinely
prints `+` where `=` belongs; confirmed by 6× upscale. See §6.3.

**Influence of distance from BMU:**
```
Θ_t = exp( −dist² / (2 σ_t²) )
```
`[img-read: somclip0009.png]` — `dist` is the node's distance from the BMU, obtained by
Pythagoras on the grid (som.htm).

**Neighbourhood initialisation:** "The neighbourhood radius is initialised so that it begins
as **half of the map grid width**", shrinking with time to a single node (som.htm).

**Weight initialisation:** "the weights in each node are initialised to a random value" —
hence a different trained map each run (som.htm).

**SOM Distortion** — vendor-modified metric, source cited as Wu, Y. & Takatsuka, M.,
'Spherical self-organizing map using efficient indexed geodesic data structure', *Neural
Networks* 19 (2006). The modification: "alters the input's distortion to be the average
distortion over its Best Matching Unit's (BMU) neighbourhood" (som.htm).

```
E_d = (1/n) Σ(i=1…n) [ Σ(j=1…w) h_(b_i,j) ‖x_i − w_j‖² ] / [ Σ(j=1…w) h_(b_i,j) ]
```
`[img-read: somclip0023.png]`

```
h_(b_i,j) = exp( −dist(b_i, j)² / (2 r²) )
```
`[img-read: somclip0024.png]`

Where (som.htm): `E_d` = SOM total distortion; `n` = number of inputs; `w` = number of
neurons; `b_i` = BMU of input `x_i`; `h_(b_i,j)` = neighbourhood function;
`dist(b_i, j)` = distance between BMU and neuron j on the grid; `r` = neighbourhood radius.
**Lower distortion = better-trained map.** Vendor claim: "the Spherical SOM geometry
consistently results in a more accurately trained SOM."

**Geometries** (som.htm) — the motivation is the *border effect*, where border nodes train
more poorly than centre nodes:
- **Square** — original geometry; border nodes have limited connections.
- **Hexagonal** — more neighbours per node; **reduces but does not remove** the border effect.
- **Spherical** — tessellated sphere; **no border effect at all**, since all nodes have the
  same number of connections. Map size is chosen from a dropdown of node counts that satisfy
  the tessellation (not a free width).

**Node Distance display:** shading on internode borders; darker = bigger distance in
n-dimensional space. Vendor's rule: "If significant regions of the map have high node distance
then it may be useful to increase the size (Map Width) of the map and retrain" (som.htm).

**Calibration — three routes** (som.htm):
1. **Group Nodes by Hierarchical Clustering** — no calibration curve needed; uses the same
   five linkage methods as Cluster Analysis (§2.4).
2. **Calibration to a continuous curve** (e.g. permeability).
3. **Calibration to a discrete curve** (an existing facies curve).

**Calibration weighting rule** (som.htm) — this is the core of routes 2 and 3:
> The calibration takes input data one level at a time, computes the Euclidean distance from
> the input vector to **each** node, then assigns the calibration value to **every** node with
> a weighting equal to **the inverse of the square of the Euclidean distance**.

- *Discrete data*: one weighting group per facies; the node's result is the facies with the
  highest average weighting; second-most-likely is the next highest.
- *Continuous data*: one weight group; the node value is the **weighted average**; a
  **weighted standard deviation** per node is also computed and output as ±1 SD curves.
- *Probability curve* = "the average of the inverse of the square of the distances between all
  the calibration points and the node in question". Explicitly **relative only** — must not be
  used to compare one model to another.

**Node Distance output curve** is the **Mahalanobis distance** — "the normalized distance
between the input data vector and the node(s) selected", expressed as the number of normalised
standard deviations the input is from the selected node (som.htm). (All input curves are
normalised before use.)

**Advanced prediction options** (som.htm):
- *Use closest node* — single BMU.
- *Average Node Data* — average the N closest nodes (N from a dropdown).
- *Average using weighted difference* — "The weighting is **the inverse of the square of the
  distance**."
- *Average using nodes calibration weighting* — better-calibrated nodes get more weight.

### 2.4 Cluster Analysis for Rock Typing (OPEN)

Vendor reference: Doveton, J.H., *Multivariate Pattern Recognition and Classification Methods,
Geological Log Analysis Using Computer Methods* (cluster_analysis.htm).

**Two-stage design** (cluster_analysis.htm): Stage 1 K-means into many clusters
("**15 to 20 clusters** would appear to be a reasonable number for most data sets"), Stage 2
consolidation into geological facies ("This may involve reducing the data to **4 to 5**").

**Stage-1 K-means:**
- Assign each point to the cluster minimising the sum of squares difference to the cluster
  mean; recompute means; repeat "until the mean values do not change between loops."
- **Normalization (standardization) is mandatory and precedes clustering:** subtract the mean,
  divide by the standard deviation, "so that each input log has the same dynamic range. Hence
  a normalized log data value of 1.0 or −1.0 will be one standard deviation."
- **Seeding is dead code.** The manual states twice, emphatically: "The current implementation
  of the K-mean clustering **does not use the seed values** at their starting point. The
  clustering self-seeds the clusters based on the data. The Seed values are currently not
  used." The *Seed Clusters* button (which runs a PCA, sorts, and splits into equal groups)
  therefore has no effect on results.
- Failure mode documented: "One or more of the clusters had zero data points!" — remedy is to
  re-run.

**Cluster Means grid statistics** (cluster_analysis.htm):
- *# Points* — number of depth levels in the cluster.
- *Cluster Spread* — standard deviation of the distance of each point from the cluster mean,
  **in units of standard deviation of the original data**; lower = tighter.
- *Mean* — in units of the **input log** (un-normalised).
- *Std Dev* — per-log within-cluster SD. Diagnostic: "Large standard deviations for all
  clusters would indicate that this log bears little influence … and could probably be
  excluded from the input."

**Stage-2 hierarchical consolidation — the five linkage methods.** Given clusters A and B just
merged into Z, and a third cluster C (cluster_analysis.htm):

| # | Method | Update rule for d(Z,C) |
|---|---|---|
| 1 | Minimum distance between all objects in clusters | `min(d(A,C), d(B,C))` — single linkage |
| 2 | Maximum distance between all objects in clusters | `max(d(A,C), d(B,C))` — complete linkage |
| 3 | Average distance between merged clusters | average distance of all objects within the cluster formed by merging, and C |
| 4 | Average distance between all objects in clusters | average distance of objects in Z to objects in C — average linkage |
| 5 | **Minimize the within-cluster sum of squares distance** (**DEFAULT**) | the increase in within-cluster sums of squares if the two clusters were merged — Ward |

Vendor's shape guidance: method 1 "will yield long thin clusters"; method 2 "will yield
clusters that are more spherical"; methods 3 and 5 "tend to yield clusters that are similar to
those obtained with" method 4. Default method 5 "gives good results for separating out the
different log lithologies into different clusters."

**Cluster Randomness Index** — printed as ASCII, not a raster (cluster_analysis.htm, and
identically in som.htm):

```
Av. Thickness     = Number of depth levels / Number of cluster layers
Random Thickness  = Σ p_i / (1 − p_i)          [p_i = proportion of depth levels in cluster i]
Randomness index  = Av. Thickness / Random Thickness
```

Interpretation: "A value of 1 would be totally random, higher values less random." Pick the
number of clusters at the **highest peaks**. Vendor's worked example: "a cluster grouping of
6 or perhaps 10 would seem to give the most likely information."

**Calibration to an external facies curve** (cluster_analysis.htm) — same weighting law as SOM:
"the value of the calibration curve is then stored at each cluster with a weighting factor
which is **the inverse of the square of the distance** of the calibration point to this cluster
point"; the facies with the highest weighted average wins. The calibration curve **cannot be a
continuously variable curve** like core permeability (it may be a Text curve of facies names).

**Fit flag curve:** 1.0 where the input calibration curve equals the output facies curve,
0.0 where different; named `<base>_1`, `_2`, `_3` … per output set (cluster_analysis.htm).

### 2.5 Principal Component Analysis (OPEN)

**Normalization** (principal_component_analysis.htm): "The input data is normalized for each
curve … by subtracting the curve mean value and dividing by the curve standard deviation."
(Z-score, same as Cluster Analysis.)

**PC curve construction** — the vendor's own worked example, which I re-derived and verified:

```
PC1 = −0.294 (Rhob − 2.635)/0.157
    + 0.563 (Dt   − 64.53)/9.54
    + 0.667 (Nphi − 0.132)/0.071
    + 0.391 (Gr   − 79.13)/43.41
```
(principal_component_analysis.htm)

Example eigenvector / variability table (principal_component_analysis.htm):

| PC | % Variability | rhob | dt | nphi | gr |
|---|---|---|---|---|---|
| 1st | 48.88 | −0.29412 | 0.56258 | 0.66652 | 0.39083 |
| 2nd | 37.51 | 0.68196 | −0.27708 | 0.14744 | 0.66062 |
| 3rd | 11.06 | 0.50637 | 0.77419 | −0.36116 | −0.11741 |
| 4th | 2.54 | 0.43819 | −0.08581 | 0.63527 | −0.63012 |

Normalisation constants for the same example: rhob mean 2.63535 / SD 0.15722; dt 64.53714 /
9.54118; nphi 0.1321 / 0.07116; gr 79.13452 / 43.40833.

*Arithmetic verified:* 48.88 + 37.51 + 11.06 + 2.54 = 99.99; and 48.88 + 37.51 = 86.39,
matching the vendor's "the first 2 curves will contain 86.4 of the variability."

**PCA Correlation crossplot coordinates** — loading scaled by the square root of the
eigenvalue (principal_component_analysis.htm):

```
coordinate = eigenvector_component × sqrt(eigenvalue)
```

Vendor's example (eigenvalues 2.24 for PC1, 1.474 for PC2):
```
GR: [ 0.592 × √2.24 ,  0.285 × √1.474 ] = [  0.886, 0.346 ]
DT: [ −0.364 × √2.24 , 0.665 × √1.474 ] = [ −0.545, 0.807 ]
```
*Arithmetic verified independently:* √2.24 = 1.49666 → 0.592 × 1.49666 = 0.8860 ✓;
−0.364 × 1.49666 = −0.5448 ✓; √1.474 = 1.21408 → 0.285 × 1.21408 = 0.3460 ✓;
0.665 × 1.21408 = 0.8074 ✓.

**Two run modes** (principal_component_analysis.htm): per-well (analysis run independently in
each well) or pooled across all Model-Build wells. Warning worth carrying: in per-well mode
"the PC analysis runs immediately so there will be **no indication that the analysis has run**."

Also reported: **Pearson's r correlation matrix** between input curves, and a "PCA Quality of
Representation" table showing each input curve's contribution to each PC.

### 2.6 Textural Facies Analysis (OPEN algorithm, one Tier-C-adjacent flag)

Pipeline (textural_facies_analysis.htm): Image log → tile → **Frequency Transform** per tile
(low frequency = coarse texture, high frequency = fine) → bin the frequency tiles horizontally
and vertically → per-bin statistics → rank bins → threshold → feed survivors into a **SOM** →
group nodes by hierarchical clustering or calibration to a facies curve.

**Binning schemes** — bins are deliberately unequal, "designed to give greater resolution to
the lower frequencies at the expense of the higher frequencies" (textural_facies_analysis.htm):
- **Doubling:** bin sizes 1, 2, 4, 8, 16 … elements, both horizontally and vertically.
- **Incrementing:** bin sizes 1, 2, 3, 4 … elements, both directions.

*Vendor's worked example, arithmetic verified:* a frequency tile 24 wide × 120 tall under
Doubling gives 5 bins across (1+2+4+8+16 = 31 ≥ 24) and 7 bins down (1+2+4+8+16+32+64 = 127
≥ 120) = 35 bins per tile; × 8 tiles = **280 bins** across the image. ✓

**Bin Statistics options:** Sum of the Absolute Values, Average of the Absolute Values,
Maximum Value, or Standard Deviation (textural_facies_analysis.htm).

**Binning filters** (remove constant artefacts):
- *Mean Filter* — removes Bin 1, "which basically represents zero frequency (DC, a simple offset)".
- *Horizontal / Vertical filters* — remove all of Row 1 or Column 1. The Vertical filter
  removes the pad/flap discontinuity; the Horizontal filter removes residual depth offsets
  between button rows.

**Ranking:** by Bin Statistics (Standard Deviation or Maximum) or by Principal Components.
Ranked bins are written as the `Bin` curve, element 1 having the largest value. Thresholding
selects the top-N into `SOM_Bins`. Initially the threshold is zero (no thresholding) — a
second Run is required after setting it (textural_facies_analysis.htm).

**SOM stage:** "The underlying SOM engine is the same one used in the standalone SOM module …
Here, we are using a **Spherical** map, and the number of nodes available is the same as for
the stand-alone SOM" (textural_facies_analysis.htm).

**Tier-C-adjacent flag (NEW — not in the register):** the `Freq_Tiles` output curve holds data
"in a **proprietary format**, and cannot be interpreted by the end user" — so it is not
plotted (textural_facies_analysis.htm). The frequency-transform *encoding* is undisclosed;
the surrounding binning/ranking/SOM algorithm is fully open. Recommend adding to the Tier-C
register as a scoped item (encoding only). See §8.

Parameter set auto-saves to the well folder as a `*.TFset` file on calibration completion.

### 2.7 SandPit 3D sanding failure equations (Multi Depth / Discrete Depth)

Not machine learning, but assigned to G and fully extracted. All eight equations transcribed
from rasters on multi_depth_analysis_workflow.htm.

```
CBHP  = (3×S1 − S2 − U)/(2 − A)  −  P_p × A/(2 − A)          [img-read: geom_clip0001.png]
P_p   = P_i − P_depletion                                     [img-read: geom_clip0002.png]
U     = TWC × Boost                                           [img-read: embim488.png]
A     = ((1 − 2ν) × α) / (1 − ν)                              [img-read: geom_clip0004.png]
CDP   = P_p − CBHP                                            [img-read: geom_clip0005.png]
LF    = (Sθmax − BHFP) / U                                    [img-read: geom_clip0006.png]
Sθmax = 3×S1 − S2 − BHFP×(1 − A) − A×P_p                      [img-read: geom_clip0007.png]
BHFP  = P_p − P_drawdown                                      [img-read: geom_clip0008.png]
```
The `Sθmax` glyph is confirmed twice: `[img-read: embim489.png]`, `[img-read: embim490.png]`.

Symbols (multi_depth_analysis_workflow.htm): `S1`, `S2` = maximum and minimum normal stresses
on the cavity wall, computed from ShMin, SHMax and vertical stress; `U` = Equivalent Formation
Strength; `A` = poroelastic stress coefficient; `ν` = Poisson's ratio; `α` = Biot's
coefficient; `P_p` = cavity (pore) pressure; `BHFP` = bottom hole flowing pressure.

**Internal consistency check — PASSED (I derived this, it is not stated in the manual).**
Setting `LF = 1` (the vendor's stated sanding threshold) gives `Sθmax − BHFP = U`:
```
3S1 − S2 − BHFP(1−A) − A·P_p − BHFP = U
3S1 − S2 − A·P_p − BHFP(2−A)        = U
BHFP = (3S1 − S2 − U)/(2−A) − P_p·A/(2−A)  ≡  CBHP  ✓
```
So CBHP is exactly the BHFP at which LF = 1. The eight equations are mutually consistent —
a strong signal that the raster transcriptions are correct.

**Physical meaning** (multi_depth_analysis_workflow.htm): CBHP = well pressure at which sand
failure occurs (predictive, for new wells); CDP = pressure change needed to cause failure
(diagnostic, for producing wells); LF = ratio of maximum effective tangential stress to
formation strength — **LF ≥ 1 indicates sanding is happening.** The workflow is to adjust the
Boost Factor until LF reaches 1 where sanding is known to occur.

Boost factor is candidly described as "a **'fudge factor'** which is used to scale up
laboratory results of rock strength (the 'TWC' test) to full scale in a reservoir."

### 2.8 Interp-Demo — worked walkthrough (ALL VALUES ARE EXAMPLES)

> **Marking per tasking (e): every number in this subsection is an EXAMPLE from a demonstration
> user app shipped as a teaching aid. None of it is an IP default, an endorsed parameter, or a
> recommendation. Do not propagate any of it into SandiBumi as a default.**

The same routine is presented in 7 languages (GNU Fortran, GNU C/C++, C# .NET, VB .NET,
MATLAB, IronPython, Full Python) — interp-demo.htm.

**Workflow sequence revealed** (interp-demo.htm, Fortran reference implementation):
1. `CalculatePickett` — derive m and Rw from the interactive Pickett-plot line endpoints.
2. Loop over depth levels.
3. **Porosity** — sonic or density branch, selected by a text parameter.
4. **Clay volume** from GR, clamped to [0, 1].
5. **Clay-corrected porosity**: `Por = Por − Vclay × PorClay`, floored at 0.0001.
6. **Water saturation** — Archie or Indonesian branch.
7. **Rwa**, **BVW**.
8. **Net reservoir / net pay flags** and zonal summation.

**Example equations as coded** (interp-demo.htm):
```
Sonic porosity : Por     = (S − SonMat) × SonCp / (SonFluid − SonMat)
Sonic clay por.: PorClay = (SonClay − SonMat) × SonCp / (SonFluid − SonMat)
Density porosity: Por     = (D − DenMat) / (DenFluid − DenMat)
Density clay por.: PorClay = (DenClay − DenMat) / (DenFluid − DenMat)
Vclay          = (GR − GRclean) / (GRclay − GRclean)       , clamped [0,1]
Shell m        = 1.87 + 0.019 / Por                        (logic-flag option)
Archie         : F = a / Por^m ;  Sw = (F × Rw / Rt)^(1/n)
Indonesian     : Sw = [ Rt^0.5 × ( Vcl^(1−Vcl/2)/Rclay^0.5
                                 + Por^(m/2)/(a×Rw)^0.5 ) ]^(−2/n)
Rwa            = Rt × Por^m / a
BVW            = Sw × Por  if Sw < 1, else Por
```
The Indonesian form matches standard Poupon-Leveaux exactly (I re-derived it from
`1/√Rt = Sw^(n/2)·[Vcl^(1−Vcl/2)/√Rcl + φ^(m/2)/√(aRw)]`).

**Clamps and null conventions revealed in passing** (these *are* generalisable engine facts,
not example parameters):
- Null sentinel throughout: **−999.0**.
- `Vclay` clamped to [0, 1].
- Porosity floored at **0.0001** after clay correction.
- `Sw` hard-capped at **1.2** for the output curve, but separately clamped to **1.0** before
  net-pay flagging — two different ceilings for two different purposes.
- Net reservoir: `Por ≥ PhiCut AND Vclay ≤ VclCut`. Net pay adds `Sw ≤ SwCut`.

**Pickett endpoint solve** (interp-demo.htm):
```
m  = ( log10(max(Ro1,0.01)) − log10(max(Ro2,0.01)) )
   / ( log10(max(Phi2,0.001)) − log10(max(Phi1,0.001)) )
Rw = Ro1 × Phi1^m / a ,  clamped to [0.0001, 100.0]
```
Guard values 0.01 / 0.001 prevent log of zero. If the recomputed values match the stored
Pickett defaults (within `|ΔRw| < 0.001` and `|Δm| < 0.01`) the app assumes the Pickett plot is
not open and falls back to the parameter values.

**Example parameter table** `[img-read: _upclip0063.png]` — EXAMPLES ONLY:

| Code | Label | Default | Min | Max | Tab |
|---|---|---|---|---|---|
| xa | 'a' | 1 | 0 | 5 | Water Saturation |
| xm | 'm' | 2 | .5 | 10 | Water Saturation |
| xn | 'n' | 2 | .5 | 10 | Water Saturation |
| Rw | Rw | .1 | .01 | 10 | Water Saturation |
| SonMat | Son Matrix | 55.7 | 5 | 100 | Porosity |
| SonFluid | Son Fluid | 189 | 50 | 300 | Porosity |
| SonCp | Son CP | 1 | 0.1 | 10 | Porosity |
| SonClay | Son Clay | 100 | 30 | 220 | Porosity |
| DenFluid | Den Fluid | 1.0 | 0.01 | 1.9 | Porosity |
| DenClay | Den Clay | 2.70 | 1.5 | 3.9 | Porosity |
| DenMat | Den Mat | 2.65 | 1.5 | 3.9 | Porosity |
| GrClean | GR Clean | 10 | 0 | 300 | Clay Volume |
| GrClay | GR Clay | 100 | 0 | 300 | Clay Volume |
| Rclay | Res Clay | 1 | .01 | 5000 | Water Saturation |

**The grid is scrolled** — `PhiCut`, `VclCut`, `SwCut`, `PPphi1/2`, `PPres1/2`, `RwPick`,
`MPick` are referenced in the code but lie below the visible rows. See OPEN ITEM §9.4.

**Example text parameters** `[img-read: _upclip0064.png]` — EXAMPLES ONLY:
- `PorEq` / "Phi Eq" — values `Sonic,Density`, default **Density**.
- `SwEq` / "Sw Eq" — values `Archie,Indonesian`, default **Archie**.

**Logic flag:** `Shellm` — switches the Shell formula for m on/off (interp-demo.htm).

**API idiom worth carrying** (interp-demo.htm): input parameters and logic flags are exposed
to user code **as functions**, called with `()`; depth-indexed access uses the level index
(`RW(INDEX)`); **index `-1` denotes zone-level (non-depth-indexed) access**, as used throughout
`CalculatePickett`. Output is written via `SAVE_<curve>(INDEX, VALUE)`. The `DEPTH` curve is
always available without being declared.

### 2.9 Multiple Linear Regression (OPEN)

Least-squares fit of the Curve-to-Predict against N input curves
(multiplelinearregression.htm). No equation raster on the page — the method is stated in prose
only.

Reported outputs: per-curve **Coefficients**; **Norm Coefficients** ("The closer the normalized
coefficient value is to zero for an input curve, the lower its effect on the model build.
Conversely, the closer the value is to one, the more important"); total data points; and **R²**.

Useful integration fact: right-click the coefficients grid → **Copy as Formula** puts the
fitted model on the clipboard as a formula string, pasteable into the User Formula module.

Optional per-curve base-10 log transform; output can be clipped to a min/max.

### 2.10 Contingency Table (OPEN)

Compares two discrete curves (standalone_contingency_table.htm; also machinelearning.htm).

- Inputs must contain a small set of distinct values — **maximum 100**, "more typically 4–8
  values are normal."
- **Calibrated mode** (default): assumes facies numbers mean the same thing in both curves, so
  facies 3 matches facies 3. Match criteria are reported; table is square.
- **Uncalibrated mode**: the two curves may have different facies counts and values. **No match
  criteria are reported**, histogram graphics are dropped, pie charts are always on, and the
  table is not necessarily square.
- An all-zero row or column is **not shown** on the table.
- Percentages may be referenced to either the calibration input or the calculated results —
  the vendor warns "Changing this option can affect the table and graphics by a considerable
  amount."
- Export: printer, clipboard, graphics file, `.txt` or `.csv`.

---

## 3. Parameters, defaults & constraints

### 3.1 Fuzzy Logic

| Item | Value | Source |
|---|---|---|
| Number of bins | **must be between 2 and 100** | statisticalcurveprediction.htm |
| Number of bins (shipped panel value) | 10 | `[img-read: _flclip0007.png]` |
| Starting bin Value / Bin width (variable-size mode) | 1 / 1 | `[img-read: _flclip0007.png]` |
| Bin sorting mode selected | Equal sampled bins | `[img-read: _flclip0007.png]` |
| Weight bin by number of samples in bin | prose: "**The default is to have this box selected**"; panel shows it **cleared** | conflict — §6.5 |
| Input curves per well | **up to 20** | statisticalcurveprediction.htm |
| Percentile error `Er` | **25** (both ML and Wt-av rows) | `[img-read: _flclip0010.png]` |
| Null value | −999 | statisticalcurveprediction.htm |
| Default output set | `Fuzzy (Fuzzy Logic)` | `[img-read: _flclip0010.png]` |
| Output results checked by default | Most Likely, Wt av. 2 most likely, Probabilities all bins, Most Likely high/low | `[img-read: _flclip0010.png]` |
| Output results unchecked by default | 2nd Most Likely, Wt av. 2 most likely high/low | `[img-read: _flclip0010.png]` |
| Report file | `Fuzzy.Txt` in the loaded database folder | statisticalcurveprediction.htm |

Output curve naming convention `[img-read: _flclip0010.png]`: `<root>_ml`, `_2l`, `_av`;
probabilities `Prob_ml/_2l/_av`; closeness of fit `Cfit_ml/_2l`; result bin `BinN_ml/_2l/_av`;
low/high `_mlL`/`_mlH`, `_avL`/`_avH`; all-bins probability array `ProbB`.

### 3.2 Neural Networks

See §2.2 table. Additional: input curves per well **up to 20**; classification network
**max 10 facies categories**; recommended training zones **4–8**; report covers Well details,
Training Settings, Training Results, Model Run Settings (neural_networks.htm).

### 3.3 SOM

| Item | Prose | Shipped panel | Source |
|---|---|---|---|
| Geometry | Square / Hexagonal / Spherical | **Spherical selected** | som.htm; `[img-read: somclip0016.png]` |
| Map Width (Square, Hexagonal) | width² = node count; **max 200** | **20** (→ 400 nodes) | som.htm; `[img-read: somclip0016.png]` |
| Spherical map size | chosen from a dropdown of tessellation-valid node counts | **642** | som.htm; `[img-read: somclip0016.png]` |
| Number of Training Iterations | user-set | **60000** | `[img-read: somclip0016.png]` |
| Initial Learning Rate | range stated **0–1** | **0.1** | som.htm; `[img-read: somclip0016.png]` |
| Neighbourhood radius σ₀ | **half the map grid width** | — | som.htm |
| Default Zone Size | **20** ("can be rather small … setting it to eg 50 can make adjustment easier") | — | som.htm |
| Input curves per well | **up to eight** | — | som.htm |
| SOM Total Distortion (example) | lower is better | 0.904 | `[img-read: somclip0016.png]` |
| Star plot SD range | selectable **1 to 10** standard deviations | — | som.htm |

> Caveat on the panel column: a screenshot captures the dialog state at documentation time.
> These are *as-shipped-in-the-manual* values, strong evidence of factory defaults but not
> proof. Treated as such throughout. Flagged in §9.2.

### 3.4 Cluster Analysis

| Item | Value | Source |
|---|---|---|
| Recommended initial K | **15 to 20** | cluster_analysis.htm |
| Number of Clusters (shipped panel) | **15** | `[img-read: _caclip0006.png]` |
| Recommended consolidated groups | **4 to 5** | cluster_analysis.htm |
| Default linkage method | **Minimize the within-cluster sum of squares** (Ward) | cluster_analysis.htm |
| Number of linkage methods | **5** | cluster_analysis.htm |
| Max User Sets (output) | **seven** | cluster_analysis.htm |
| Input curves per well | **up to eight** | cluster_analysis.htm |
| Text-curve name extension default | `_T` | cluster_analysis.htm |
| Report file | `ClusterAnalysis.txt` | cluster_analysis.htm |
| Example consolidation ladder | 15 clusters → 11 → 8 → 5 groups | cluster_analysis.htm |

Worth carrying: "IP attempts to group similar colors and values for the cluster groups in
order to keep the log plots looking broadly similar" across different group counts.

### 3.5 User Apps — hard limits

| Limit | Value | Source |
|---|---|---|
| Max **input curves** per user app | **70** | user-app-properties.htm |
| Max **input parameters** per user app (total) | **70** | user-app-properties.htm |
| Max parameters **per tab** | **20** | user-app-properties.htm |
| Max **parameter tabs** | **14** | user-app-properties.htm |
| Max Window Label length | **15 characters** | user-app-properties.htm |
| Interactive parameter lines per track | **up to 5** | user-app-properties.htm |
| Interactive lines per crossplot | prose says "up to five", then describes only Lines 1–3 | conflict — §6.6 |
| Default curve storage precision | **Single-precision 32-bit float** | user-app-properties.htm |
| Optional precision | 64-bit doubles via the Options-tab flag | user-app-properties.htm |

Other contract facts (user-app-properties.htm):
- The special tab named **`Hide`** holds parameters that are hidden from the UI — used to pin
  constants, and hidden parameters also do not appear at the base of a crossplot.
- Window Label layout syntax: words joined by `_` stay on one line; words separated by a space
  split into separate title cells.
- Changing an existing Window Label **requires deleting the app's Parameter Set** first
  (`Well → Delete Parameter Set → Other Sets → UPxxx`).
- Output curves may be written back to input curves; input curves must exist, output curves are
  created if absent.
- "Do **not** leave empty lines between lines of code as it can cause problems with the
  indexing of the code instructions."
- Zone processing order defaults **top zone → bottom zone**; reversible via Options. Reversing
  the direction *within* a zone requires editing the `For` loop in user code — the two are
  independent controls.
- `Trim Output Arrays on Run` — declare the maximum array X-dimension at compile time, then
  trim to the maximum non-null X used at run time.
- Native DLLs reload every run; **.NET assemblies are not unloaded until IP exits** (so a
  rebuilt assembly needs an IP restart).
- DLL/assembly names must be globally unique across user apps, "even though the path is
  different" — otherwise Windows resolves the wrong one.
- Curve Groups: reading from a group returns the **first curve in the group**; other members
  are reachable via the enumerable `<Curvename>_Curves` object.
- User-app names **cannot end with a numeric character** (create-new-user-app.htm).
- Code filenames are fixed: `UsersCode.pas` (Pascal), `UsersCode.f` (Fortran)
  (create-new-user-app.htm).
- Apps on a network share must live in a subfolder named **`UserPrograms`** to be detected;
  network-hosted apps can be **run but not edited or deleted** — "This is a safety feature"
  (managing-user-apps.htm).
- Compiler **Warnings may be ignored; Errors must be fixed** (create-new-user-app.htm).

### 3.6 SandPit 3D (Discrete + Multi Depth)

| Parameter | Default | Valid range | Source |
|---|---|---|---|
| Poisson Ratio ν | **0.25** | 0 – 0.5 | multi_depth_analysis_workflow.htm |
| Biot Factor α | **1** | 0 – 1 | multi_depth_analysis_workflow.htm |
| Stress Path Factor | **0.7** (field data "shows an average value of 0.7") | 0 – 1 | multi_depth_analysis_workflow.htm |
| Boost Factor (cased hole) | **3.1** | 1 – 10 | multi_depth_analysis_workflow.htm |
| Boost Factor (open hole) | **no default** | must be > 1 | multi_depth_analysis_workflow.htm |
| Boost Factor practical range | 2.5 (open hole) → 3.3 (cased hole) | — | multi_depth_analysis_workflow.htm |
| Stress Azimuth | **no default**, must be entered | 0 – 360° | multi_depth_analysis_workflow.htm |
| Completion type | **Open Hole** | Open / Cased-and-perforated | multi_depth_analysis_workflow.htm |
| Drawdown | normally 0 | may be negative "on limited occasion" | multi_depth_analysis_workflow.htm |
| Depletion | — | must be 0 or positive | multi_depth_analysis_workflow.htm |

Structural constraints:
- Cased-and-perforated creates **31 output curves**: 10 CBHP + 10 CDP + 10 LF at perforation
  angles **0°–90° in 10° steps** (fixed, not user-changeable) + 1 Valid-Output-Data flag
  (multi_depth_analysis_workflow.htm). Naming is `<prefix>0`, `<prefix>10` … `<prefix>90`.
- Open hole creates **4 curves**: CBHP, CDP, LF, Valid Output Data. At least one of CBHP/CDP/LF
  must be selected; all three are selected by default.
- Plots show only **0°, 30°, 60°, 90°** — not all 10 orientations.
- Zero-degree perforation is aligned to **top dead centre**; for vertical sections
  (deviation = 0) it falls back to the azimuth curve value "even though a vertical well section
  does not have an azimuth as such."
- Zone depths for the parameter set use **measured depth (MD), not TVD**.
- Discrete Depth automatic mode: **10 TWC values** — user gives min and max, 8 evenly spaced
  in between; min must be > 0 and < max. Manual mode: up to 10 user values
  (discrete_depth_analysis.htm).
- Discrete Depth cased-and-perforated: fixed sweep 0°–90° in 10° steps at a **single** TWC > 0.
- Reservoir pressure sweep: final pressure must be > 0 and < initial pressure.
- Operating Envelope plot: control line is **`y = x`**; curves truncated at it. `Clip Curves`
  hides curves wholly below **`y = 0`**.
- Results grid sorting: up to **three** columns (discrete_depth_analysis.htm).
- Depth value in Discrete Depth "is only used as a reference … **it is not used as an input to
  the calculation itself**."
- Interactivity breaks silently if the TVD curve contains any null: "If the input TVD curve
  contains any null data anywhere in it, then the interactivity of this line will no longer
  work" (multi_depth_analysis_workflow.htm).

### 3.7 Graphical Workflow Manager

Upstream/downstream **curve matching rules, in strict priority order**
(graphical_workflow_manager.htm) — directly reusable for a SandiBumi pipeline binder:

1. Use the exact curve set and curve name if it exists in an upstream output.
2. Use the last output curve of the same **type** as the current input curve type, if specified.
3. If there is no curve default type, use the last output curve of the same **name** (ignoring
   set header name).
4. If there is a curve default type, use the last output curve of the same **type**.
5. Use whatever curve is already specified for the module.

Only **selected** upstream items are considered. Two linking modes: "No automatic linking with
upstream curves" (**default**) and "Merge upstream curves with defaults". Critically: "this
curve matching only applies **during design** of the workflow … At runtime, the parameter sets
specified in the workflow file will be loaded."

Execution order: selected items run **sequentially top to bottom by column**. Layout files use
the `.gwl` suffix and embed the parameter sets. Columns can be flagged "Single Select".
External apps support `%LocalAppData%`-style folder tokens, space-separated runtime parameters
with `\"quoted values\"` for embedded spaces, and an optional "wait for app to finish" gate.

---

## 4. User-formula language reference

Two modules share one language: **User Formula** (single line, `.frm`) and **Multi Line User
Formula** (grid, `.mlf`). Sources: user-definedformula.htm, multi_line_user_formula.htm.

### 4.1 Complete function library

The function set is **identical in both modules** (verified line-by-line across the two pages).
22 entries: 6 operators + 16 functions.

| Token | Signature | Semantics |
|---|---|---|
| `*` | `x * y` | Multiply |
| `+` | `x + y` | Add |
| `-` | `x - y` | Subtract |
| `/` | `x / y` | Divide |
| `**` | `x ** y` | Raise to power |
| `^` | `x ^ y` | Raise to power (identical to `**`) |
| `LOG` | `LOG(number or curve name)` | Logarithm **base 10** |
| `ALOG` | `ALOG(number or curve name)` | Antilogarithm base 10 |
| `LN` | `LN(number or curve name)` | Natural logarithm |
| `EXP` | `EXP(number or curve name)` | e raised to the given power |
| `TAN` | `TAN(number or curve name)` | Tangent — **input in degrees** |
| `SIN` | `SIN(number or curve name)` | Sine — **input in degrees** |
| `COS` | `COS(number or curve name)` | Cosine — **input in degrees** |
| `ATAN` | `ATAN(number or curve name)` | Arctangent — **output in degrees** |
| `ASIN` | `ASIN(number or curve name)` | Arcsine — **output in degrees** |
| `ACOS` | `ACOS(number or curve name)` | Arccosine — **output in degrees** |
| `SQRT` | `SQRT(number or curve name)` | Square root |
| `ABS` | `ABS(number or curve name)` | Absolute value |
| `MIN` | `MIN(number1 or curve1, number2 or curve2)` | Smallest of **exactly two** parameters |
| `MAX` | `MAX(number1 or curve1, number2 or curve2)` | Largest of **exactly two** parameters |
| `TRUNC` | `TRUNC(number or curve name)` | Truncate — remove all digits after the decimal point |
| `RANDOM` | `RANDOM` | Random number **between 0 and 1 at each depth level**; "useful for adding noise to a curve" |

**Trigonometric units are degrees on both input and output** — a genuine divergence from
almost every host language's radians default, and a classic silent-wrongness trap.

### 4.2 Conditional / logical syntax (Multi Line module only)

Excel-spreadsheet style, "in brackets with comma separators" (multi_line_user_formula.htm):

```
IF  : IF( logical_test, value_if_true, value_if_false )
AND : AND( logical1 )
OR  : OR( logical1 )
```

**Critical constraint, stated explicitly:** "The `AND`, `OR` statement **only applies to one
logical condition**. If more than one logical condition is required then the `AND` statement
has to be **nested**." Documented usage patterns:
```
IF(AND(x > y, x < z), value_if_true, value_if_false)
IF(OR(x < y, x > z), value_if_true, value_if_false)
```
Nested `IF` statements are supported. The single-line User Formula module has **no** IF/AND/OR;
it instead exposes a FORTRAN-style `If … and/or … then … else …` **discriminator row** in the
dialog, where blank discriminators mean only the `then` formula line is computed
(user-definedformula.htm). The discriminator boxes accept curve names or numeric values, and
can test for the **existence or non-existence of a curve**.

### 4.3 Null handling — the highest-risk area for SandiBumi

- Null sentinel is **−999** (user-definedformula.htm, multi_line_user_formula.htm).
- **`Check for null data`** (both modules): when selected, validates all input curves for null
  and sets the output to null at the same depth. When cleared, "the null data issues within the
  equations are handled manually" / "null values will be counted in the computations and it is
  likely that there will be some unpredictable values in the output curves."
- **`Check intermediate results for null data`** (Multi Line only): when checked, intermediate
  nulls propagate to dependent output curves only; independent outputs are unaffected. **When
  unchecked, "any intermediate null values are treated as numeric values of −999 and used in
  calculations."** This is the single most dangerous default-behaviour statement in the
  formula documentation — −999 silently entering arithmetic.
- **Arithmetic errors** set the output to null and log a message; **a maximum of 20 errors are
  logged**, then messages are suppressed and processing continues to subsequent depths.
  ("*Previously*, the formula run would terminate on an arithmetic error.") Named examples:
  divide by zero, square root of a negative number, `ASIN` of a value < −1 or > 1
  (multi_line_user_formula.htm).

### 4.4 Naming, parsing, and evaluation rules

- **Reserved-character escape:** a curve name containing a character that could be parsed as an
  operator (e.g. `PHI-T`, where `-` reads as minus) **must be double-quoted**: `"PHI-T"`
  (both pages).
- **Avoid single-letter curve names** such as `e`, and avoid mixing letters and digits like
  `e42` — "This can cause syntax errors" (both pages).
- **`MIN`/`MAX` argument parenthesisation** — documented wrong/right pair:
  ```
  Min(RAW:SGR * 0.5 + 20, RAW:SGR)     - Wrong
  Min((RAW:SGR * 0.5 + 20), RAW:SGR)   - Correct
  ```
  "In the case that the Min and Max functions are passed **an expression** rather than a curve
  or number, then the expression **must be placed in parenthesis, or incorrect results will be
  returned**" (user-definedformula.htm). This is a silent-wrongness parser defect, not an
  error — it returns a wrong number.
- `MIN`/`MAX` may be **nested**, the documented idiom for clamping (e.g. limiting a GR-derived
  VCL to 0–1).
- **Negative base with non-integer exponent raises an error** — `**` and `^` "will return an
  error if you attempt to raise a negative number to a non-integer power … the User Formula
  module does not support complex numbers."
- Evaluation order is "traditional"; braces/brackets nest expressions.
- **Curve Type reference:** prefix `@` to a generic curve type — `@GammaRay`, `@density` —
  to select by type rather than name (both pages).
- Set-qualified names use `SET:CURVE` (e.g. `RAW:SGR`).
- Syntax validation runs on `Run All`; errors report a **character position** (e.g. "position
  19" for a missing `)`).
- Continuation lines are recombined into a single statement **before** syntax validation
  (multi_line_user_formula.htm).

### 4.5 Multi-line grid semantics and array rules

Grid columns (multi_line_user_formula.htm): `Line`, `Use`, `Cont.`, `Out`, `Name`, `Type`,
`Array`, `Unit`, `Formula`.
- `Use` cleared → line ignored; this is also the mechanism for **comments**.
- `Out` cleared → the line's result is a **temporary local variable** within the formula.
- `Type` — **Numeric (default)** or Text curve.
- `Array` is only sensitive when `Use` and `Out` are true **and the curve does not already
  exist**.
- The **output/result curve must be named in the first row** of the grid.
- Rows can be reordered by dragging the line-number cell, which changes execution order.

**Array semantics** (both pages):
- **Array indices start at 1**; element selected with square brackets, e.g. `CURVE[3]`.
- If `Array` is false, a single-sampled output is created from the **[n]th element**; if `[n]`
  is unspecified it **defaults to element [1]**.
- Applying a scalar function (e.g. `SQRT`) to an array returns an array of the same dimensions.
- Dimension compatibility rules, with the vendor's own worked examples using
  `Cap(X=100,Z=2)`, `Por(X=1,Z=2)`, `Phi(X=1,Z=1)`, `Cp(X=2,Z=2)`, `Result(X=100,Z=2)`:
  - **Element-wise where both exist**; where the smaller operand has no element, the result is
    **NullValue** (e.g. `Result(3,1) = NullValue` because `Cp(3,1)` does not exist).
  - **X-broadcast:** an operand with X = 1 is reused across all X (the `Por` case).
  - **Full scalar broadcast:** a non-array curve (X = 1, Z = 1) is reused everywhere (the `Phi`
    case).
  - Z-dimension mismatch → **do not run**. But "if the **only** mis-match is that the
    X-dimension of the Output is less than that of the input arrays then **do** run."
- Reducing a 2-D Z-array to a single sample loses the high-resolution depth: "Only the first
  depth-slot of the original Z-array is preserved."

Depth range: blank Top/Bottom defaults to the **entire well depth range**. Zone *names* (rather
than depths) may be stored, so the same formula resolves to different depths per well — the
vendor highlights this for Multi-Well Batch use (both pages).

### 4.6 Compiler / runtime facts (compiler-information.htm)

**Supported user-app languages** (userapps.htm): FORTRAN, C/C++, VB.NET, C#.NET, MATLAB,
IronPython, Full Python. (Delphi Pascal appears in the app dialog but greyed out
`[img-read: _upclip0063.png]`.)

| Toolchain | Version / detail |
|---|---|
| GCC 32-bit | **GCC 2.95.2**, path entry `C:\gcc-2.95.2\bin;` |
| GCC 64-bit | **GCC MinGW-64 ver 4.4.1**, path entry `C:\GCC\MinGW-64\bin;` |
| IronPython | **IronPython 2.7.4** (bundled) |
| Full Python | Python **2 and 3** supported; **3.12 recommended** "for optimal security and compatibility" |
| .NET | assemblies loadable at versions **2, 3.5 and 4** (user-app-properties.htm) |

- Compilers are FSF/GNU, downloadable free; the wrong-bitness install produces a
  "compiler was not found" error.
- **"The GNU compiler does not like single Letter code names like A, B, C. Output curve names
  need to be renamed to at least a 2 character mnemonic e.g. AA, BB, CC."**
- **No console window exists in IP** — `print` output is invisible. The documented workaround
  posts to the IP Message Board:
  ```python
  def ipprint(text):
      from PGL.IP.API import IntPetroAPI
      messageBoard = IntPetroAPI().GetService('PGL.IP.Services.IMessageBoard, PGL.IP.Services')
      messageBoard.Add(1, text)
  ```
- Python apps still require a "Compile" step because dependent Python files must be generated
  and copied to the user-app folder.
- **`ip2py`** — the IP Python library wrapping the IP API. Modules: `calculations`, `curves`,
  `debugger` (PTVSD + VS Code), `general`, `ipmaths`, `jupyter`, `parametersets`, `userapp`
  (includes converting all app input curves into a **pandas DataFrame**), `wells` ("Wells need
  to be active to be picked up"), `zones`.
- `ip2py` dependencies: `pandas`, `numpy`, `ptvsd`, `jupyter`, `jupyterlab`, `pywin32`,
  `mpmaths` (see §6.8 — `mpmaths` is almost certainly a typo).
- Jupyter Notebook / JupyterLab integration; default startup directory is the **`IntPetro47`**
  folder under AppData.
- Examples ship at `C:\Program Files\IPxxxx\ApiDocumentation\Examples\UserApps\ip2py`.
- Known trap: installing `ip2py` into a `C:\Program Files` Python install fails with access
  denied even for admins — relaunch IP via "Run as Administrator".
- **FORTRAN fixed-form layout is enforced**: comments start with `C` in column 1; label numbers
  in columns 1–5; statements in columns 7–72; any character in column 6 marks a continuation.
- FORTRAN trig functions in user code take **radians** (compiler-information.htm), whereas the
  User Formula language takes **degrees** (§4.1). Both are correct in their own context; the
  mismatch is a live porting hazard.
- MATLAB apps spawn a new MATLAB instance per run unless
  `enableservice('AutomationServer',true)` is issued.

---

## 5. Assumptions & validity limits

1. **Fuzzy probabilities are relative, not absolute.** They depend on the number of input
   curves; models with different input counts are not comparable
   (statisticalcurveprediction.htm).
2. **SOM probabilities are relative only.** "Their absolute values have no meaning and the
   curve should only be used to compare one depth level to another. It should also **not be
   used to compare one model to another**" (som.htm).
3. **SOM and NN are non-deterministic by construction** — random weight initialisation means a
   different result every training run, even on identical data (som.htm, neural_networks.htm).
   Neither module documents a seed control.
4. **Equal-sampled binning is approximate** where many identical values exist
   (statisticalcurveprediction.htm).
5. **K-means seeding is inert** — the Seed Clusters button and any manually entered seed values
   are ignored by the current implementation (cluster_analysis.htm, stated twice).
6. **Cluster calibration curves must be discrete.** "the calibration curve cannot be a
   continuously variable curve like core permeability" (cluster_analysis.htm). SOM has no such
   restriction — it accepts continuous calibration curves.
7. **Contingency table inputs must be low-cardinality** — max 100 distinct values
   (standalone_contingency_table.htm).
8. **Uncalibrated contingency mode reports no match criteria** — by design, since the two
   facies schemes are unrelated (standalone_contingency_table.htm).
9. **Textural facies frequency transform is sensitive to image discontinuities.**
   "Discontinuities in a full width image … such as the 'join' between a pad and flap, will be
   picked up by the frequency transform and could affect the results"; mitigate with
   'one per Pad/Flap' or the Vertical Filter (textural_facies_analysis.htm).
10. **Border effect is a real SOM bias** — square-grid border nodes are "more poorly trained
    compared to nodes in the centre"; hexagonal reduces it, only spherical removes it (som.htm).
11. **Stress path factor assumes** "a homogeneous and isotropic poroelastic formation, and a
    passive tectonic basin" when set to the uniaxial value
    (multi_depth_analysis_workflow.htm).
12. **Boost factor is an admitted fudge factor** requiring iterative field calibration; "Initial
    values should be used with caution" (multi_depth_analysis_workflow.htm).
13. **Discrete Depth's depth entry is cosmetic** — a label, not an input to the calculation
    (discrete_depth_analysis.htm).
14. **Formula module cannot do complex numbers** — negative base to a non-integer power is an
    error, with the vendor's own advice to use a user app in a language that supports complex
    numbers (both formula pages).
15. **Log10 flagging changes the reported statistics**, not just the internals — reported
    minima/maxima/means become logarithmic values (statisticalcurveprediction.htm; visible as
    the negative PERMCORE mean in `[img-read: _flclip0007.png]`).
16. **NN cross-validation is silently disabled** when zonal averaging is used
    `[img-read: _nnclip00018.png]`.

---

## 6. Internal discrepancies

Every item here is a real conflict inside the IP 2025 documentation set. Rules 3 and 4 apply —
I report the page's own values and flag the tension rather than resolving it.

**6.1 — Neural Networks "Epoch per pass" default: 1000 vs 100.**
Prose: "The default value of **1000** is a good value for the neural networks supplied with IP"
(neural_networks.htm). Shipped Training Settings panel shows **100**, and the Training Results
listing in the same image confirms "Epochs Trained : 100" for both passes
`[img-read: _nnclip00018.png]`. A factor-of-10 conflict on the single most consequential NN
hyperparameter. Unresolved — do not adopt either number without testing against the product.

**6.2 — SOM time constant λ: which `t`?**
The raster prints `λ = t / log σ₀` `[img-read: somclip0006.png]`, and the surrounding prose
defines `t` as "the current training pass iteration" (som.htm). If `t` really is the *current*
iteration, λ changes every iteration and the decay `exp(−t/λ)` collapses to the constant
`exp(−log σ₀)` — which cannot be the intent. The standard Kohonen formulation uses the *total*
number of iterations. The manual's symbol definition and its equation are mutually
inconsistent. Reported as printed; see OPEN ITEM §9.1.

**6.3 — SOM neighbour update equation prints `+` instead of `=`.**
`[img-read: somclip0008.png]` reads `W_(t+1) + W_t + Θ_t L_t (V_t − W_t)` — verified at 6×
upscale, so this is not an OCR artefact but a defect in the vendor's own raster. The companion
BMU equation `[img-read: somclip0003.png]` correctly prints
`W_(t+1) = W_t + L_t (V_t − W_t)`. Read structurally, the intended form is
`W_(t+1) = W_t + Θ_t L_t (V_t − W_t)` — but I have **not** substituted that; the transcription
in §2.3 is exactly what is printed.

**6.4 — "Closeness of fit" means two different things.**
Fuzzy Logic: **bin distance** (integer, always positive; bin 4 vs bin 6 → 2)
(statisticalcurveprediction.htm). Neural Networks: **absolute value of the difference** between
original data and the result curve, in curve units (neural_networks.htm). Same curve name
family, incomparable semantics.

**6.5 — Fuzzy "Weight bin by number of samples in bin": prose vs panel.**
Prose: "The default is to have this box **selected**" (statisticalcurveprediction.htm). The
shipped panel shows it **cleared** `[img-read: _flclip0007.png]`. Also note the prose says this
option "is used in Prediction mode when the **Variable size bins** option is chosen", yet the
panel has *Equal sampled bins* selected while the checkbox sits enabled — the scoping of the
option is not clearly documented.

**6.6 — User-app interactive crossplot lines: 5 vs 3.**
"Up to **five** interactive lines can be set-up per crossplot", but the following paragraph
documents only Line 1, then "**Lines 2 and 3** have the same functionality as Line 1"
(user-app-properties.htm). Lines 4–5 are never described.

**6.7 — Menu location conflicts (stale text from the pre-2025 reorganisation).**
The IP 2025 `machinelearning.htm` hub lists SOM, PCA and Cluster Analysis under the new
**Machine Learning** menu. But `som.htm` still instructs "From the **Advanced Interpretation**
menu, select Self Organising Maps", and every one of SOM/PCA/Cluster/Fuzzy/NN/MLR still carries
"Related Topics → Advanced Interpretation". Worse, `principal_component_analysis.htm`
contradicts *itself*: it opens with "From the Machine Learning Menu, Select Principal Component
Analysis" and later states "The module is accessed under the main menu **Advanced
Interpretation** menu list." Meanwhile `specialinterpretation.htm` (the actual Advanced
Interpretation hub) no longer lists any of them.

**6.8 — Library name `mpmaths`.**
The `ip2py` dependency list gives **`mpmaths`** (compiler-information.htm), which is not a
known PyPI package; the `ip2py` *module* list on the same page gives `ipmaths`. Almost
certainly a typo for `mpmath` (or `ipmaths`). Flagged, not corrected.

**6.9 — `@` described as an "ampersand".**
`multi_line_user_formula.htm` says "Prefix an **ampersand (@)** to the curve type"; `@` is an
at-sign. `user-definedformula.htm` correctly says "Prefix an **@ character**". The symbol
itself is unambiguous in both.

**6.10 — Linkage method #1 renamed between sibling pages.**
`cluster_analysis.htm`: "**Minimum** distance between all objects in clusters".
`som.htm`: "**Minimise** distance between all objects in clusters". Same method, two labels —
matters only if SandiBumi matches on strings.

**6.11 — AppData version folder is inconsistent across pages.**
`IntPetro41` (user-app-properties.htm, managing-user-apps.htm, and the title bars of
`[img-read: _upclip0063.png]` / `[img-read: _upclip0064.png]`), `IntPetro47`
(compiler-information.htm, Jupyter default directory), and `intpetro36` (interp-demo.htm
example output path). Documentation carried forward across releases without updating paths.

**6.12 — Fuzzy Logic tab name.**
Prose calls it the "**Create Model**" tab; the UI tab reads "**Create Fuzzy Model**"
`[img-read: _flclip0007.png]`. Also `_flclip0007.png` is referenced twice on the page — once
for "Create Prediction Model" and once for "Prediction Model Statistics" — although it is a
single screenshot showing both.

**6.13 — SOM Input tab text is garbled.**
The "Use Well for Model Run" bullet in som.htm has two sentences spliced mid-word: "if this is
selected then the ' Show Plot - this button opens a log plot window … data from that well will
be used in the Self Organising Maps model run." A copy-editing failure, not a technical
conflict, but it obscures the actual behaviour of the control.

---

## 7. IP2018 numeric diff

Method: located each assigned page's counterpart in `C:\Users\ARUNIKA\AppData\Local\Temp\c18`,
stripped markup, and compared the numeric and structural claims.

**Page presence.** 13 of 28 pages have IP2018 counterparts. **Absent from IP2018:**
`experiencedeye`, `machinelearning`, `interp-demo`, `user-app-properties`,
`compiler-information`, `user-definedformula`, `textural_facies_analysis`,
`managing-user-apps`, `create-new-user-app`, `running-user-apps`, `create-user-app-help`,
`example-user-apps`, `create-and-edit-user-apps`, `edit-user-app`.

This independently **confirms the Tier-C register entry** stating Experienced Eye is absent
from IP2018 — EE and the Machine Learning menu are genuinely new in this release, as is
Textural Facies Analysis.

### Material differences

| # | Fact | IP2018 | IP2025 | Significance |
|---|---|---|---|---|
| 1 | **NN engine disclosure** | "The neural network that IP uses is a commercial product by **Neuro Solutions** … built with **Neuro Solutions 5.5**. The number of **Hidden layers = 1**." (neural_networks) | **Statement entirely removed** (0 hits for `neurosolutions` or `hidden layer`) | The vendor scrubbed its third-party engine attribution and the only architectural disclosure. The Tier-C register entry now rests solely on the IP2018 source — record that provenance. |
| 2 | **Input curves per well — Fuzzy Logic** | up to **eight** | up to **20** | Real capacity increase (2.5×). |
| 3 | **Input curves per well — Neural Networks** | up to **eight** | up to **20** | Same. |
| 4 | **Input curves per well — PCA** | up to **eight** | up to **20** | Same. |
| 5 | **Input curves per well — SOM** | up to **eight** | up to **eight** | **Unchanged** — SOM did not get the increase. |
| 6 | **Input curves per well — Cluster Analysis** | up to **eight** | up to **eight** | **Unchanged.** |
| 7 | **SOM Default Zone Size** | default value of **5** | default value of **20** | Default quadrupled. Both releases advise raising it to ~50. |
| 8 | **NN menu location** | "From the **Advanced Interpretation** Menu" | "From the **Machine Learning** Menu" | The menu reorganisation is real and IP2025-new; §6.7 shows it was applied inconsistently. |

### Verified unchanged

- **Neural Networks:** Training Passes default **3**; Epoch per pass prose default **1000**;
  cross-validation **0 % disables**; classification network **max 10 facies categories**.
  (So discrepancy §6.1 is *not* a 2025 regression — the prose has said 1000 since 2018.)
- **Fuzzy Logic:** number of bins **between 2 and 100**.
- **Cluster Analysis:** **15 to 20** initial clusters; **4 to 5** consolidated; **five**
  linkage methods; **seven** max User Sets; Ward default.
- **SOM:** maximum Map Width **200**; square/hexagonal/spherical geometries; SOM Distortion
  metric and the Wu & Takatsuka (2006) citation; all training equations.
- **SandPit 3D — every geomechanics default is byte-identical:** Poisson 0.25 (0–0.5);
  Biot 1 (0–1); Stress Path Factor 0.7 (0–1); Boost 3.1 cased (1–10), no open-hole default
  (> 1); practical boost range 2.5–3.3; Stress Azimuth 0–360 with no default. **No drift.**
- **Multi Line User Formula:** function library and IF/AND/OR syntax.
- **Contingency Table:** max 100 distinct values, typically 4–8.

---

## 8. SandiBumi notes

**Adopt (Tier A — conventions and architecture):**
1. **The two-stage clustering architecture** (many K-means clusters → hierarchical
   consolidation into facies) is a genuinely good pattern and is fully open. Ship all five
   linkage rules; default to Ward.
2. **The Cluster Randomness Index** (§2.4) is a cheap, defensible answer to "how many facies?"
   — better than an unexplained elbow plot, and fully specified in ASCII.
3. **The GWM curve-matching priority list** (§3.7) is a ready-made spec for a pipeline binder.
4. **Star plots and contingency tables** as facies QC surfaces, including the uncalibrated mode.
5. **Zone-name-rather-than-depth persistence** in saved formulas — the right primitive for
   multi-well batch work in the Mahakam datasets.

**Reimplement from open sources (Tier B — cite the primary reference):**
6. **Fuzzy Logic → cite Cuddy 1997 (SPWLA Paper S).** Implement the harmonic combination of
   §2.1 exactly; it is the non-obvious part. Add what IP lacks: a seed control and an absolute
   (comparable) probability.
7. **SOM → cite Kohonen (3rd ed.) and Wu & Takatsuka (2006)** for the distortion metric.
   Resolve §6.2 (λ) from Kohonen, not from the manual.
8. **PCA** is textbook; the manual's worked example (§2.5) makes an excellent regression test —
   the numbers verify arithmetically, so use them as fixtures.

**Never implement (Tier C):**
9. Experienced Eye, EEFS, DTA — per the register and the EE dossier.
10. The neural-network engine. IP's is NeuroSolutions 5.5 (per IP2018); any shipped weights are
    NeuroSolutions artefacts. SandiBumi must train its own network with an independently
    licensed stack. The only adoptable NN facts are generic and independently re-derivable
    (single hidden layer, early stopping on cross-validation, 10 % dither sensitivity).
11. **NEW — recommend adding to the Tier-C register:** the **Textural Facies Analysis
    `Freq_Tiles` encoding**, which the vendor explicitly calls "a proprietary format …
    cannot be interpreted by the end user" (textural_facies_analysis.htm). Scope the entry
    tightly: the *encoding* is Tier C; the tiling/binning/ranking/thresholding/SOM pipeline
    around it is fully documented and Tier A/B.

**Where SandiBumi can exceed IP:**
12. **Determinism.** Both SOM and NN are documented as irreproducible run-to-run with no seed
    control. A seeded, reproducible implementation is a straightforward and genuinely
    differentiating win for auditable client deliverables.
13. **Fix the null trap.** IP's "intermediate nulls become numeric −999" behaviour (§4.3) is
    exactly the silent-wrongness class this project exists to eliminate. SandiBumi's formula
    engine should make null propagation the only behaviour, with no opt-out.
14. **Fix the MIN/MAX parenthesisation defect** (§4.4) — a parser that returns a wrong number
    for an unparenthesised expression argument is a bug, not a documented feature. Parse
    arguments properly.
15. **Degrees-vs-radians.** If SandiBumi's formula language targets IP compatibility, trig must
    be **degrees** (§4.1) — but the FORTRAN user-app path is radians (§4.6). Whatever is chosen,
    make it explicit at the call site.
16. **Seeding.** IP's K-means seeding controls are dead code (§2.4). Either implement seeding
    properly (k-means++) or do not present the control.
17. **Comparable probabilities.** Both Fuzzy and SOM emit relative-only probabilities that
    cannot be compared across models. Normalising these properly would remove a documented
    limitation users are told to work around.

**Direct reuse for the Mahakam work:** the SandPit 3D equation set (§2.7) is complete,
internally consistent (I verified LF = 1 ⟺ BHFP = CBHP), and carries a full default set that has
not drifted since 2018. If SandiBumi ever adds sanding analysis, this is a citable,
implementable specification. Note the CBHP form assumes the vendor's `S1`/`S2` cavity-wall
convention — the mapping from ShMin/SHMax/Sv to `S1`/`S2` is **not** given (§9.3).

---

## 9. OPEN ITEMS

**9.1 — SOM time constant λ (blocking for any SOM reimplementation).**
`λ = t / log σ₀` `[img-read: somclip0006.png]` with `t` defined as the *current* iteration is
self-defeating (§6.2). Needed: whether `t` is the total training-iteration count. Resolve from
Kohonen or by testing the product — **not** by assuming.

**9.2 — Which panel values are factory defaults?**
The SOM Train tab (Map Width 20, spherical 642 nodes, 60000 iterations, learning rate 0.1),
Cluster (K = 15), and Fuzzy (10 bins, Er = 25) values come from documentation screenshots. They
are strong evidence but not proof of factory defaults — a screenshot may capture an author's
working session. Confirm against a live IP install before adopting any as a SandiBumi default.

**9.3 — SandPit 3D `S1` / `S2` derivation is not given.**
The manual states only that "S1 and S2 are calculated from the values of ShMin, SHMax and
Vertical Stress" (multi_depth_analysis_workflow.htm) — the actual transformation to cavity-wall
maximum/minimum normal stresses (which must involve well deviation, azimuth, stress azimuth,
and perforation angle) is never printed. Without it the equation set of §2.7 is not
implementable end-to-end.

**9.4 — Interp-Demo parameter grid is scrolled.**
`[img-read: _upclip0063.png]` shows 14 of an unknown number of rows. `PhiCut`, `VclCut`,
`SwCut`, `PPphi1`, `PPphi2`, `PPres1`, `PPres2`, `RwPick`, `MPick` are referenced in the code
but their example defaults and limits are below the visible area. Low priority — these are
EXAMPLE values, not adoptable defaults.

**9.5 — Neural network normalization scheme undocumented.**
The NN page states no input normalization beyond the optional base-10 log flag, yet the
sensitivity readout refers to a "normalised data range" `[img-read: _nnclip00018.png]`, which
implies normalization definitely occurs. The scheme (min-max? z-score? over what window?) is
not disclosed on any assigned page.

**9.6 — Epoch-per-pass default (§6.1) unresolved.** 1000 (prose, unchanged since 2018) vs 100
(panel). Needs a live-product check.

**9.7 — SOM "Average Closest N nodes" — available values not listed.**
The dropdown's options and default are not given (som.htm).

**9.8 — Spherical SOM valid node counts.**
Only 642 is observed `[img-read: somclip0016.png]`; the full dropdown list of
tessellation-valid node counts is not printed anywhere.

**9.9 — Textural Facies "pre-defined choices" for the SOM input threshold** are referenced but
never enumerated (textural_facies_analysis.htm).

**9.10 — Fuzzy `Weight bin by number of samples` scope** (§6.5): prose scopes it to Variable
size bins; the panel shows it enabled alongside Equal sampled bins. Actual behaviour under
equal-sampled binning is undetermined.

**9.11 — Best Cost values in the NN example are denormal** (1.2223e−314, 2.0430e−314)
`[img-read: _nnclip00018.png]`. Values in the 1e−314 range sit in IEEE-754 subnormal territory
and are far more consistent with uninitialised memory than with a real training cost. Possibly
a display defect in the shipped product. Noted as an observation only — it is example output,
not a specification.

---

*Agent G. 28/28 pages read. All equation rasters on the assigned pages transcribed or
explicitly flagged. No vendor file copied; nothing written outside this report.*
