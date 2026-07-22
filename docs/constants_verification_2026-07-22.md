# Rock-typing / SHF constants verification vs papers (2026-07-22)

**Scope:** every hardcoded literature constant in `rocktyping.rs`, `shf_fit.rs`, `thomeer.rs`,
`hfu.rs` (+ the forward `satheight.rs` J-function constants), cross-checked against
`docs/research_2026-07/ref_rocktyping_shf.md` and the published sources. This is the
"verify-before-field-release" gate the reference doc and the code comments repeatedly flag.

**Method:** static read of the constants → compare to the reference doc → confirm against the
primary/secondary literature via web search where the doc itself said "verify against the paper"
(no local paper copies exist for FZI/Winland/Pittman/Lucia/PGS/Swanson).

**Result: 3 discrepancies found, all number-changing. Jauhar approved → #1 (GHE bins) and #2 (PGS)
were APPLIED; #3 (Pittman r75) stays HELD pending a primary-source glance. Everything else confirmed
correct.** Post-fix: `cargo test --lib` = 247 passed / 0 failed / 7 ignored.

> **STATUS (2026-07-22):** #1 GHE ✅ applied · #2 PGS ✅ applied · #3 Pittman r75 ⏸ held.

---

## Verdict table

| # | Constant / formula | File | Code has | Literature says | Verdict |
|---|---|---|---|---|---|
| 1 | **GHE FZI bins** | rocktyping.rs:25 | ~~…1.5, 2.5, 4, 6, 8~~ → …1.5, **3, 6, 12, 24** | …1.5, **3, 6, 12, 24** (×2 series) | ✅ **FIXED** |
| 2 | **PGS pore-structure exponent** | rocktyping.rs:57 | ~~3.5~~ → PS = k/φ**³** | PS = k/φ**³** | ✅ **FIXED** |
| 2b | **PGS pore-geometry** | rocktyping.rs:104 | ~~k/φ~~ → PG = **√(k/φ)** | PG = **√(k/φ)** | ✅ **FIXED** |
| 3 | **Pittman r75 row** | rocktyping.rs:299 | (1.243, 0.674, −1.517) | ≈(0.778, 0.626, −1.205) | ⏸ **HELD (likely typo)** |
| — | Amaefule RQI 0.0314 / FZI / perm 1014.24 | rocktyping.rs, hfu.rs | 0.0314, 1014.24 | 0.0314, 1014.24 | ✅ correct |
| — | Kolodzie-Winland R35 | rocktyping.rs:99 | 0.732, 0.588, −0.864 | 0.732, 0.588, −0.864 | ✅ correct |
| — | Lucia RFN A/B/C/D | rocktyping.rs:157-160 | 9.7982/12.0838/8.6711/8.2965 | same (+forward-check sane) | ✅ correct |
| — | Pittman r10–r50 rows + r35 anchor | rocktyping.rs:291-298 | see file | match widely-cited table | ✅ correct |
| — | Thomeer Bv = Bv∞·exp(−G/**log₁₀**(Pc/Pd)) | thomeer.rs:33 | log₁₀ form | log₁₀ (Thomeer 1960 convention) | ✅ correct |
| — | Swanson k = 399·(Bv**%**/Pc)ᴬ^1.691 | thomeer.rs:264 | 399, 1.691, Bv×100 | 399, 1.691, Sb in **percent** | ✅ correct |
| — | Brooks-Corey / Skelt-Harrison / FOIL forms | shf_fit.rs | — (all fitted) | match reference doc | ✅ correct |
| — | Port classes (Hartmann-Beaumont) | rocktyping.rs:29 | 0.1/0.5/2.5/10 | nano/micro/meso/macro/mega | ✅ correct |
| — | Leverett J 0.21645 / 0.433 psi·ft⁻¹·SG⁻¹ / IFT_RES 26 | satheight.rs:14,15,109 | 0.21645, 0.433, 26 | canonical oilfield-unit values | ✅ correct |
| — | Hg-air σcosθ standardization target | thomeer.rs:163 | 367.0 | 367.7 (=480·\|cos140°\|) | ✅ ~ok (0.2% round; see note) |

---

## Discrepancy 1 — GHE FZI bin boundaries (CONFIRMED WRONG)

**Where:** `rocktyping.rs:25`
```rust
const GHE_BOUNDS: [f64; 9] = [0.0938, 0.1875, 0.375, 0.75, 1.5, 2.5, 4.0, 6.0, 8.0];
```

**Problem:** Corbett & Potter (2004) define the 10 Global Hydraulic Elements by a **regular
geometric (×2) progression of FZI**: `0.0938, 0.1875, 0.375, 0.75, 1.5, 3, 6, 12, 24`
(GHE10 open-ended above 24). The code's first five boundaries double correctly, then the tail
breaks the pattern (`2.5, 4, 6, 8` instead of `3, 6, 12, 24`). So every rock with FZI ≥ 2.5 —
i.e. the **best-quality reservoir rock (GHE6–GHE10)** — is mis-binned. Two independent web
sources returned the ×2 series; the code's tail matches neither.

**Recommended fix (HELD — reclassifies rocks, so needs sign-off):**
```rust
const GHE_BOUNDS: [f64; 9] = [0.0938, 0.1875, 0.375, 0.75, 1.5, 3.0, 6.0, 12.0, 24.0];
```
**Test impact:** `fzi_and_rqi_match_amaefule_formula` (rocktyping.rs:409) asserts FZI≈2.808 →
class 7; with the corrected bins it falls between 1.5 and 3 → **class 6**. Update the assertion
to `6.0`.

**Blast radius:** `RT` output when `METHOD=ghe` (the default), and the class-grouped `PERM_RT`
predictor that keys off it. Winland-port method unaffected.

---

## Discrepancy 2 — PGS pore-geometry & pore-structure definitions (STRONG)

**Where:** `rocktyping.rs:57` (PS_EXP default) and `:104` (PGEOM formula)
```rust
param("PS_EXP", "...", 3.5, 1.0, 6.0),   // default 3.5
pgeom[i] = (k / phi) as f32;             // PG = k/φ
pstruc[i] = (k / phi.powf(ps_exp)) as f32; // PS = k/φ^3.5
```

**Problem:** The definitive modern treatment of the Permadi-Susilo method — *"On the Pore
Geometry and Structure Rock Typing"* (ACS Omega 2024, open access) — derives from Kozeny-Carman:
- **Pore geometry PG = √(k/φ)** (∝ mean hydraulic radius) — the code uses `k/φ`, which is the
  **square** of the proper variable.
- **Pore structure PS = k/φ³** (exponent **3**, from `k ∝ φ³`). The article explicitly states it
  finds **no 3.5 exponent** "neither in discussions of the original Permadi & Susilo (2009) work
  nor in any modified approaches."

The `3.5` originates from the reference doc, which itself says it was *"stated from memory, no
local copy … MUST verify exact exponent (3 vs 3.5), and whether PG uses sqrt(k/φ)."* The evidence
now points to **3.0** and **√(k/φ)**.

**Recommended fix (HELD — changes the PGEOM/PSTRUC curve values):**
```rust
param("PS_EXP", "PGS pore-structure exponent (k/φ^PS_EXP)", "-", 3.0, 1.0, 6.0), // 3.5 → 3.0
...
pgeom[i] = (k / phi).sqrt() as f32;   // √(k/φ), not k/φ
```
…plus update the doc-strings/comments (rocktyping.rs:9, :47, :64-65) that assert 3.5 = Permadi-Susilo.

**Caveat (honesty):** verified against the ACS Omega 2024 *review* of Permadi-Susilo, **not** the
original SPE 125350 (1.9 MB PDF, image-only — couldn't extract). If you have SPE access, confirm
against SPE 125350 / Wibowo & Permadi 2013 before flipping. `PS_EXP` is already a user-editable
param, so the change is "make the default correct," not a hard-code — low risk to flip once confirmed.

**Blast radius:** `PGEOM` and `PSTRUC` output curves only — **diagnostic curves, not used for RT
classification** (RT comes from GHE/Winland). So no interpretation-number impact beyond those two
display curves.

---

## Discrepancy 3 — Pittman r75 coefficients (SUSPECT — likely transcription error)

**Where:** `rocktyping.rs:299`
```rust
("PR75", 1.243, 0.674, -1.517),
```

**Problem:** The code's r10–r50 rows all match the widely-cited Pittman (1992) coefficients
exactly (and r35 = 0.255/0.565/−0.523 cross-checks the reference doc). Only **r75 diverges**: the
code has `(1.243, 0.674, −1.517)` where the commonly-quoted Pittman r75 is `≈(0.778, 0.626,
−1.205)`. An isolated divergence in the last row, with every other row correct, is the classic
signature of a single mistyped table row.

**Could not fully adjudicate:** Pittman's Table 1 is an image (`tab01.JPG`) on every source I
could reach; academia.edu / AAPG-wiki returned 403; no clean text reproduction surfaced. So this
is flagged **SUSPECT**, not CONFIRMED — verify against **AAPG Bulletin v76 (1992) p.191-198,
Table 1** (or the `tab01.JPG` image) and correct if it reads `(0.778, 0.626, −1.205)`.

**Blast radius:** `PR75` curve, and `RAPEX`/`RT_PITT` **only when APEX=r75** (default is r35, so
the common path is unaffected).

**Also note (not an error):** the code offers 9 of Pittman's 14 published radii
(skips r45, r55, r60, r65, r70 — 5% increments). Fine as a subset; mentioned for completeness.

---

## Confirmed correct (no action)

- **Amaefule 1993** RQI = 0.0314·√(k/φ), φz = φ/(1−φ), FZI = RQI/φz, and the inverse perm
  transform k = 1014.24·FZI²·φ³/(1−φ)² — canonical (1014.24 ≈ 1/0.0314²; the ~1e-7 round-trip
  slack is already documented in `hfu.rs` tests). Consistent across `rocktyping.rs` and `hfu.rs`.
- **Kolodzie-1980 / Winland R35** = 10^(0.732 + 0.588·log₁₀k − 0.864·log₁₀φ%) — matches the
  reference doc and the canonical Winland equation.
- **Lucia RFN** (Jennings & Lucia 2003) A/B/C/D = 9.7982/12.0838/8.6711/8.2965. Matches the
  reference doc; the analytical inversion `r=(A+C·log φip − log k)/(B+D·log φip)` is algebraically
  exact; a forward-model sanity check (RFN 1–4 at φip 0.10–0.20) yields physically sensible
  carbonate k (≈0.07–13 mD). *Verified vs doc + physical sanity, not the primary paper's table.*
- **Thomeer** Bv = Bv∞·exp(−G/**log₁₀**(Pc/Pd)) — the code's **log₁₀** form is the published
  Thomeer (1960) convention, and the G range 0.1–1 is calibrated to it. (The reference doc's
  loose "ln" would need G rescaled by ln10≈2.303 — the code is the correct one, not the doc.)
- **Swanson 1981** k_air = 399·[(Sb/Pc)apex]^1.691 with **Sb in percent** — the code computes
  `399·(apex·100)^1.691` (apex = max Bv_frac/Pc), i.e. correctly puts Bv in percent. Confirmed the
  air-perm constants (399/1.691) and that it's suppressed on unstandardized (non-Hg) data.
- **Brooks-Corey**, **Skelt-Harrison**, **Cuddy FOIL** forms — match the reference doc; all are
  *fitted* (no hardcoded literature constants to drift).
- **Hartmann-Beaumont port classes** 0.1/0.5/2.5/10 µm (nano/micro/meso/macro/mega) — canonical.
- **Leverett-J forward module** (`satheight.rs`): J = 0.21645·(Pc/IFT)·√(k/φ), 0.433 psi·ft⁻¹ per
  unit SG, default reservoir IFT 26 dyn/cm — all canonical oilfield-unit values.

**Minor note (optional):** `HG_AIR_IFT = 367.0` (thomeer.rs:163) vs the exact 480·|cos140°| =
367.7. A 0.2% rounding; it enters only as a ratio (367/ift) in Pc standardization, so the effect
is sub-0.2% and G is invariant regardless. Not worth a change; noted for completeness.

---

## Action taken (2026-07-22, Jauhar approved)

1. **GHE bins — FIXED.** `GHE_BOUNDS` tail `2.5, 4, 6, 8` → `3, 6, 12, 24`; test
   `fzi_and_rqi_match_amaefule_formula` assertion updated (FZI 2.808 → GHE class 6, was 7);
   doc-comments updated.
2. **PGS — FIXED.** `PGEOM` `k/φ` → `√(k/φ)`; `PS_EXP` default (and NaN fallback) `3.5` → `3.0`;
   module/spec doc-strings updated. `PS_EXP` remains a user-editable param for override.
3. **Pittman r75 — HELD.** Not changed — I won't write an unconfirmed coefficient into the tool.
   Confirm vs **AAPG Bull. v76 (1992) p191-198, Table 1** (or the `tab01.JPG` image), then tell me
   the r75 triple and I'll fix the one row.

Verified: `cargo test --lib` = **247 passed / 0 failed / 7 ignored**. No TS change. Not committed
(pending Jauhar's go).

## Sources
- Corbett & Potter (2004) *Petrotyping* (SCA2004-30) — GHE ×2 FZI series.
- ACS Omega (2024) *On the Pore Geometry and Structure Rock Typing* (PMC11325524) — PG=√(k/φ), PS=k/φ³.
- Pittman, E.D. (1992) AAPG Bull. v76, 191-198 — r-table (r75 pending primary check).
- Swanson (1981) JPT 33, 2498-2504 — 399/1.691 apex correlation.
- Amaefule et al. (1993) SPE 26436; Kolodzie (1980) SPE 9382; Jennings & Lucia (2003) SPE 78740.
