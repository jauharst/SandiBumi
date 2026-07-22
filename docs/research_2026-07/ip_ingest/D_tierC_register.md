# IP 2025.3 — Consolidated TIER-C REGISTER (patented / proprietary capabilities)

**Source install (read-only):** `C:\Program Files\IP2025\` (publisher PGL / Lloyd's Register / Geoactive)
**Date:** 2026-07-22
**For:** SandiBumi legal review + roadmap gating.

**Rule applied:** For every item below, evidence (file/module) + user-need were recorded, and a **Tier-B, patent-free design-around** SandiBumi could build instead is named. **No Tier-C algorithm was read, decompiled, approximated, or reverse-engineered.** Where tier is ambiguous, the MORE restrictive tier was taken. `UserProgram.dll` binaries were never decompiled; the shipped neural-net weight DLLs were never read.

| # | Capability | Evidence (paths) | Tier | LEGAL-REVIEW |
|---|---|---|---|---|
| 1 | **Sonic Saturation** (Omovie) | `Modules\SonicSaturation\` | **C — patented** | **YES** |
| 2 | **Domain Transfer Analysis (DTA)** | `DomainTransferAnalysis.exe`, `DomainTransferAnalysisLog.exe`, `DomainTransferAnalysisKeys.txt` | **C — proprietary** | **YES** |
| 3 | **Experienced Eye** | `ExperiencedEye.exe`, `Extensions\PGL.IP.Extensions.ExperiencedEye.dll` | **C — proprietary** | **YES** |
| 4 | **Entropy-based image speed correction** | inside `Image Tools\` / `Optional Image Tools\` image engine (stated patented) | **C — patented (algorithm only)** | **YES** |
| 5 | **Pre-trained Neural-Net weights** | `Neural Networks\Default\ns *.1.dll`, `ns class *.10.dll` (40 DLLs) | **C for the shipped weights; method is Tier-B** | **YES (weights)** |
| 6 | **MLNET** (disambiguation) | `Modules\MLNET\` | **B — open ML.NET** | no |
| 7 | **Recall** (export descriptors only; engine absent) | `Image Tools\*Recall Export.itt/.itp` | **A — interchange descriptors** | no |

---

## 1. Sonic Saturation (Omovie) — **US Patent 12,242,011 B2**

- **Evidence:** `Modules\SonicSaturation\UserProgram.dll` (141 KB, compiled), `Parameters` (manifest, 331 KB), `UserProgram.config` → menu **"Advanced Interpretation" → caption "Sonic Saturation"**, `Compiler=CSharp`, `useZones=true`. Manifest header declares inputs including `RHOB` (Density) and `DTC` (compressional slowness).
- **User-need it serves:** estimate water / hydrocarbon **saturation from acoustic (sonic) data**, for cases where resistivity-based Sw is unreliable or absent — invaded/low-contrast zones, fresh-water or LRLC pay (Mahakam-delta relevance), or as an independent Sw cross-check to Archie/Simandoux.
- **Patent:** US 12,242,011 B2 (Omovie / assignee). Treat as an active, enforceable patent.
- **Tier-B design-around SandiBumi could build instead (do NOT copy Omovie):** if an acoustic-Sw capability is ever needed, construct it from open literature on a *different* architecture — Biot-Gassmann fluid substitution + Wyllie/Raymer-Hunt-Gardner time-average to predict a water-wet sonic baseline and invert for fluid, or a published rock-physics fluid indicator (Vp/Vs, Poisson, AI-based). All are patent-free and materially different from the patented method. **Even the design-around must be run past counsel given the granted patent.**
- **Action taken:** existence + user-need recorded ONLY. `UserProgram.dll` not decompiled; `Parameters` internals not mined for the algorithm.

## 2. Domain Transfer Analysis (DTA) — proprietary (Geoactive/PGL)

- **Evidence:** `DomainTransferAnalysis.exe` (1.16 MB), `DomainTransferAnalysisLog.exe` (1.15 MB), `DomainTransferAnalysisKeys.txt` (4001 lines of **opaque base64 strings** — appears to be an encoded key/model/feature store; NOT decoded, NOT an algorithm). Menu entry confirmed via `Modules\MLNET\UserProgram.config` which lists `<siblingnode>Domain Transfer Analysis</siblingnode>` under "Advanced Interpretation".
- **User-need it serves:** **domain adaptation / transfer learning** — take a model or log-relationship learned where training data exist (a cored/logged well or field) and transfer it to wells/domains lacking that data; predict missing logs and propagate interpretation across a field.
- **Tier-B design-around:** ordinary, published supervised ML — multivariate regression, k-NN, random forest / gradient boosting (LightGBM/XGBoost), or classic MLP log prediction (Rogers et al. 1992; Huang et al. 1996). IP itself ships the **open** `MLNET` module (item 6) for exactly this class of task. SandiBumi builds log-prediction from open frameworks, never from DTA.
- **Action taken:** existence + user-need recorded. Executables not run/inspected for logic; keys file not decoded.

## 3. Experienced Eye — proprietary (Geoactive/PGL)

- **Evidence:** `ExperiencedEye.exe` (837 KB), `Extensions\PGL.IP.Extensions.ExperiencedEye.dll`.
- **User-need it serves:** automated **feature selection / expert-mimicking** analytics — surface the most informative log combinations and interpretation cues an experienced petrophysicist would notice, to guide/accelerate analysis.
- **Tier-B design-around:** standard published feature-selection & dimensionality methods — mutual information, LASSO/elastic-net, recursive feature elimination, random-forest permutation importance, PCA/ICA, SHAP. All open, none tied to the Experienced Eye implementation.
- **Action taken:** existence + user-need recorded. Binary/DLL not decompiled.

## 4. Entropy-based borehole-image speed correction — stated patented (algorithm only)

- **Evidence:** the speed-correction is embedded in IP's **Image Tools** processing engine (compiled). The `Image Tools\` (204 files) and `Optional Image Tools\` folders themselves contain only **`.itt`/`.itp` vendor tool-descriptor files** — those are Tier-A taxonomy (Baker/Halliburton/SLB image-tool geometry/response descriptors), NOT the algorithm. No standalone "speed"/"acceleration" source surfaced; the patented entropy method lives inside the compiled image engine.
- **User-need it serves:** correct wireline/LWD **borehole-image logs for tool speed variation / stick-slip** (depth/velocity artifacts) by optimizing image entropy.
- **Tier-B design-around:** accelerometer-based speed correction (industry standard) or image cross-correlation depth-shifting between pads/passes. The **entropy-optimization** step is the patented novelty — avoid it specifically; the accelerometer/cross-correlation approaches are open.
- **Scope:** `later` (image logs are not SandiBumi v1). The `.itt/.itp` descriptor *catalog* is safe Tier-A reference (see Target A/B), only the speed-correction *algorithm* is Tier-C.
- **Action taken:** existence + user-need recorded; image engine not inspected. `.itt/.itp` are safe to catalog but were not needed here.

## 5. Neural Networks — pre-trained weights proprietary; method Tier-B (ambiguous → restrictive)

- **Evidence:** `Neural Networks\Default\` ships **40 pre-built weight DLLs**: `ns 1.1.dll … ns 20.1.dll` (prediction nets) and `ns class 1.10.dll … ns class 20.10.dll` (classifier nets), each ~319 KB.
- **Ambiguity:** whether IP's neural-net feature is DTA/Experienced-Eye-based or a standalone classic back-prop MLP is not determinable from the readable install. **Taking the more restrictive tier:** treat the **shipped pre-trained weight DLLs as proprietary artifacts** — do not extract or reuse them.
- **User-need it serves:** neural-net **log prediction** (synthetic missing curves, e.g. DT/RHOB reconstruction) and **classification** (electrofacies) — some of which (synthetic-log fill) is v1-relevant.
- **Tier-B design-around:** the *method* (feed-forward NN for regression/classification) is general and published — SandiBumi trains its own nets (or GBMs) from open frameworks. Only the **IP-shipped weights** are off-limits.
- **Action taken:** weight DLLs listed as evidence only; none read.

## 6. MLNET — **NOT Tier-C** (open ML.NET) — registered only to disambiguate

- **Evidence:** `Modules\MLNET\` (UserProgram.dll + `Parameters` + config; menu "Advanced Interpretation", caption "ML.NET", sibling of DTA).
- **Finding:** built on **Microsoft ML.NET (MIT-licensed, open source)**. Not patented, not proprietary to Geoactive. This is the **free ML path** sitting right next to the proprietary DTA — a reviewer scanning the menu could conflate them, so it is recorded explicitly as clear.
- **SandiBumi:** may freely implement an equivalent (scikit-learn / LightGBM). **LEGAL-REVIEW: no.**

## 7. Recall — engine NOT present; only export descriptors (Tier-A)

- **Evidence:** no Recall database/engine in the install. Only image **export descriptors** to Recall's format: `Image Tools\Baker STAR - Recall Export.itt`, `…Baker StarTrak - Recall Export.itt`, `Halliburton XRMI - RECALL Export.itt`, `Schlumberger FMI - Recall Export.itp`, `Schlumberger geoVISION (RAB) - Recall Export.itt`, etc.
- **Finding:** these are interchange-format *descriptors* (Tier-A taxonomy), not the Recall (Baker Hughes) system or any Recall algorithm. Nothing proprietary-algorithmic to extract. **LEGAL-REVIEW: no** (recorded for completeness per the register checklist).

---

## Bundled third-party utilities (for completeness — none are petro Tier-C)

`putty.exe / pscp.exe / plink.exe` (PuTTY SSH suite), `ExamDiff.exe` (file compare), `ipy.exe/ipy64.exe` + `pyc.py` (IronPython, Apache-2.0), `AForge.*` (LGPL imaging). These are OSS/freeware infrastructure, unrelated to protected petrophysics; listed so counsel is not surprised by unfamiliar binaries in the tree.

---

## Summary for counsel

- **Hard patent, must-avoid:** #1 Sonic Saturation (US 12,242,011 B2). Any acoustic-Sw feature needs a clean-room, literature-only design and a patent clearance.
- **Proprietary methods, build the open equivalent instead:** #2 DTA and #3 Experienced Eye — both map to standard, published ML / feature-selection that SandiBumi can implement freely.
- **Patented sub-step inside a broader feature:** #4 image entropy speed correction — avoid the entropy step specifically; open alternatives exist. `later` scope.
- **Proprietary artifacts (not method):** #5 shipped NN weights — retrain from scratch, do not reuse the DLLs.
- **Confirmed clear:** #6 MLNET (open ML.NET), #7 Recall (only export descriptors present).
- **Confirmation vs the task brief:** all "known items to confirm" were located in this install. Every readable algorithm actually extracted (Target D `D_readable_algorithms.json`) is Tier-B/A only (HFU=Amaefule, Interp_Demo=Archie/Indonesia/Wyllie/Pickett, plus utilities). No Tier-C code was read.
