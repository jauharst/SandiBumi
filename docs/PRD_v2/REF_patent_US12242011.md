# Reference — US 12,242,011 B2 claim analysis (Tier-C class C-1)

**Subject.** *Method for estimating water saturation in gas reservoirs using acoustic log P-wave and
S-wave velocities.* Assignee **Goshey Energy Services LLC**, inventor **Sheyore John Omovie**.
Granted **2025-03-04**, filed 2024-05-03, priority 2023-08-03. **Status: active, anticipated
expiry 2044-05-03.**

**Why this document exists.** `CONTRACT.md` §2.2 classifies this item **C-1 — patent-claimed**, the
one Tier-C class where independent re-derivation does *not* clear the risk, because a granted patent
claims the method itself regardless of how it was arrived at. Jauhar directed on **2026-08-07** that
the granted claims be read before any decision. This is that reading.

**What this document is not.** It is **engineering analysis, not a freedom-to-operate opinion, and
not legal advice.** Claim construction is a legal exercise, and the doctrine of equivalents can
reach subject matter outside the literal claim language. Nothing here authorises implementation.
The recommendation at the end is what to put in front of a patent attorney, not a substitute for
one.

**Provenance note.** A patent is a published government document; reading and citing it is not a
`CONTRACT.md` §2.1 vendor-file transcription and is explicitly the endorsed route under §2.2.
Claims were retrieved from Google Patents' structured markup, cross-checked against the USPTO PDF
for bibliographic data. The USPTO PDF is a CCITT-fax scan with no text layer and was not used for
transcription. **Only the claims, abstract, and the Field/Background opening were read** — the
description runs to roughly 265,000 characters and was not read in full. That is a stated limit on
everything below.

---

## 1. Claim structure — three independent claims, three different exposures

| Claim | Type | Does SandiBumi practise it? |
|---|---|---|
| **1** | System | **No.** Requires physical hardware |
| **12** | Method | **No.** Requires wellbore creation and device movement |
| **17** | CRM | **This is the one that matters** |

**Claim 1 requires an apparatus SandiBumi is not.** It recites "an acoustic logging system,
comprising: a motor and a cable, or a drill bit and at least one drilling pipe; and an acoustic
measuring device positioned horizontally adjacent to the hydrocarbon reservoir within the first
wellbore." A software package ships no motor, cable, drill bit or drilling pipe. It also recites
"positioned **horizontally**", a limitation apparently directed at horizontal wells.

**Claim 12 requires acts SandiBumi does not perform.** Its first two steps are "creating a wellbore
in a hydrocarbon reservoir" and "moving an acoustic measuring device to a position within the
hydrocarbon reservoir", followed by "measuring" the waves. Software that computes from
already-acquired logs performs none of these. An operator running the full field workflow might
practise the method; the software vendor alone does not.

**Claim 17 is the real question.** It is a pure non-transitory-computer-readable-medium claim with
no hardware and no wellbore-creation limitation — the classic software-reaching claim. Any
non-infringement position must clear claim 17 on its own terms.

---

## 2. The claimed equation, and what is wrong with it

All three independent claims recite the same equation:

```
Swt = ( |a₀·x + b₀·y + c₀| / √(a₀² + b₀²) )  ÷  ( |a₁₀₀·x + b₁₀₀·y + c₁₀₀| / √(a₁₀₀² + b₁₀₀²) )
```

Subscript `0` is the **fully hydrocarbon-saturated** trend; subscript `100` is the **fully
brine-saturated** trend. Each bracket is the standard point-to-line distance `|ax+by+c|/√(a²+b²)`.
So the claimed quantity is:

```
Swt  =  d(point → hydrocarbon trend)  /  d(point → brine trend)
```

**Evaluated at the three points where the answer is known:**

| Location of the sample point | Claimed formula returns | Physically correct value |
|---|---|---|
| On the fully hydrocarbon-saturated trend | `0 / d` = **0** | 0 ✓ |
| Exactly midway between the two trends | `d / d` = **1** | 0.5 ✗ |
| On the fully brine-saturated trend | `d / 0` = **∞** | 1 ✗ |

**It is correct at one of three, and it is unbounded** — even though the claim defines the output as
"a water saturation **fraction** of a total pore space at each lateral depth", which must lie in
[0, 1].

The normalised form of the same geometric idea is correct at all three:

```
Swt  =  d₀ / (d₀ + d₁₀₀)
```

| Location | `d₀/(d₀+d₁₀₀)` |
|---|---|
| On the hydrocarbon trend | `0/(0+d)` = 0 ✓ |
| Midway | `d/(d+d)` = 0.5 ✓ |
| On the brine trend | `d/(d+0)` = 1 ✓ |

This is ordinary fractional-distance interpolation between two lines, bounded in [0, 1] by
construction, and it is **a different expression from the one claimed**.

**Caveat, stated plainly.** The 265,000-character description was not read. It is possible — not
demonstrated — that the specification normalises, clips, or constrains the coefficients somewhere
such that the recited ratio behaves. The claims as granted contain no such limitation, and claim
scope is set by the claims. But the description must be read before anyone relies on the defect
being real rather than apparent.

---

## 3. Two further internal inconsistencies in the claims

Recorded because they bear on claim construction, not because SandiBumi should rely on them.

**3.1 Claim 17 recites a second reservoir with no antecedent basis.** Claims 1 and 12 measure the
shear wave "of the hydrocarbon-bearing interval across **the first** subsurface reservoir". Claim 17
measures it "across **a second** subsurface reservoir" — while its compressional measurement remains
in "a first subsurface reservoir". Read literally, claim 17 requires the shear and compressional
measurements to come from *different* reservoirs. A workflow measuring both across the same interval
arguably does not meet claim 17 literally. This looks like a drafting error; a court might construe
it as such, or might not.

**3.2 The trend-space limitation contradicts two of the four alternatives.** All three independent
claims require "the fully brine-saturated trend and the fully hydrocarbon-saturated trend are
determined in terms of velocity ratio and shear sonic log **or** velocity ratio and compressional
sonic log" — i.e. (Vp/Vs, DTS) or (Vp/Vs, DTC) space. Yet the four following `wherein` alternatives
include a **(K_B, G) elastic-modulus** space and a **(Vp, Vs)** space, neither of which is a velocity
ratio against a sonic log. Two of the four alternatives appear to fall outside the claim's own
governing limitation.

---

## 4. The underlying concept is not owned

The patent claims a **specific formula**, not the idea of estimating saturation from acoustic
response. Deriving `Sw` from the deviation of a measured point between a wet trend and a
hydrocarbon trend in elastic space is standard rock physics — it is what fluid substitution and
Gassmann-based analysis have done for decades, and the patent's own Background opens by discussing
Archie rather than by distinguishing prior acoustic-saturation work.

**No non-patent literature is cited anywhere in the record.** The six references are all patents
(four of them Chinese-office publications on unrelated fluid-identification and wave-matching
topics). For a method asserting novelty in a heavily-published field, the absence of a single
journal or SPWLA/SPE reference is conspicuous, and it is a fact worth putting in front of counsel:
it bears on both validity and on what the examiner actually considered.

---

## 5. The live risk: the family is still prosecuting

This is the part that most affects timing, and it cuts against complacency.

| Filing | Status |
|---|---|
| US 12,242,011 B2 (18/654,884) | **Granted** 2025-03-04 |
| US 19/068,613 → **US 2025/0306228 A1** | **Continuation, pending** (filed 2025-03-03) |
| PCT/US2025/019339 → **WO 2025/193677 A1** | **PCT filed** 2025-03-11 |

A pending continuation can pursue **new claims of different scope**, and continuations are routinely
used to write claims that read on what competitors are seen to be doing. A design-around that clears
the granted claims today does not necessarily clear what issues from 19/068,613 later. The PCT means
non-US jurisdictions are in play, which matters for a product intended for Indonesian operators and
international clients.

**Consequence for sequencing:** any decision here has a shelf life, and "we checked in 2026" will not
be a durable answer.

---

## 6. Assessment and recommendation

**A non-infringement position exists and it is not weak.** It rests on three independent legs, and
they do not all have to hold:

1. **Claims 1 and 12 are not reachable by a software product** — no motor, cable, drill bit or
   drilling pipe; no wellbore creation; no device movement.
2. **Claim 17 requires the specific recited ratio.** A normalised estimator `d₀/(d₀+d₁₀₀)` is a
   different expression, and it is the expression a competent implementer would choose anyway,
   because the claimed ratio is unbounded and wrong at two of three known points. **The design-around
   is not a workaround here — it is the correct engineering, arrived at independently.** That is the
   strongest possible posture for a design-around, and it is worth documenting *how* it was arrived
   at, contemporaneously, under `SB-CORE-004` and `SB-CORE-010`.
3. **The concept is old and the record cites no literature**, which bears on the breadth any claim
   can fairly be given.

**Against that:** the doctrine of equivalents is exactly the mechanism for reaching a formula that
performs substantially the same function in substantially the same way to achieve substantially the
same result — and a normalised distance ratio is, on its face, a candidate. **I cannot assess that.
It is a legal judgement and it needs counsel.**

### Recommended disposition

1. **Do not implement anything yet.** No requirement is allocated in any chapter on the strength of
   this document.
2. **Take three specific questions to a patent attorney**, not a general FTO request — the specific
   questions are cheaper and sharper:
   - Does a **normalised** estimator `d₀/(d₀+d₁₀₀)` fall outside claim 17 literally, and can it be
     reached under the doctrine of equivalents given that the claimed ratio is unbounded and the
     normalised form is not?
   - Does a **software-only product** that consumes already-acquired logs practise claims 1 or 12 at
     all?
   - What is the exposure from pending continuation **19/068,613**, and is it worth monitoring on a
     schedule?
3. **Read the description before counsel is engaged** if the capability is wanted — specifically to
   confirm the unbounded-ratio finding survives the full specification, and to see whether the
   trends are picked in a way that is itself claimed elsewhere in the family.
4. **Meanwhile, treat the capability as unallocated.** The owning chapter — plausibly
   `17_thinbed-laminated.md` or `25_fluidsub-rockphysics.md` — records it as C-1, cites this
   document, and specifies nothing.

### Why this matters more than a routine Tier-C item

The abstract states the method addresses **"a low resistivity low contrast shaly sand reservoir
where previous methods would indicate the reservoir was wet."** That is precisely the problem
`05_STRATEGY.md` §18.3 makes **Axis 3**, and precisely the deltaic low-contrast case the product is
positioned around. A resistivity-independent saturation route is directly competitive with the
product's strongest differentiator — which raises the value of clearing it, and raises the cost of
getting it wrong.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
