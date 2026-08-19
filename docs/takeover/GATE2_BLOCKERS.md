# Gate 2 blocker decision packet

This is the human-readable companion to the machine-owned blocker set in
`gate2-program.json`. The requirement evidence remains authoritative in
`requirements.csv`; product-owner choices remain authoritative in `DECISIONS.md`.

## Live snapshot

- Gate: `G2 - SILENT-WRONGNESS CLOSURE` - **FORMALLY CLOSED 2026-08-20 (DEC-083)**
- Scope: `222` Gate 2 requirements plus `20` later-gate-only requirements
- Handled: `222 / 222`
- Done: `222`
- Blocked: `0`
- Remaining unhandled: `0`

---

## Formal closure, 2026-08-20 (DEC-083) - and the first-sale deferred register

Jauhar could not obtain counsel in the pre-ship timeframe and ruled the gate formally
finalized (DEC-083, verbatim in `DECISIONS.md`). The last two rows - `SB-PLT-024` and
`SB-CORE-044` - closed with every engineering arm complete: the six-of-six derivation
program (DEC-078/079/080/081), the thirteen-id delete (DEC-082), per-value endpoint
custody (DEC-078), the theme rename (DEC-074), and the owed fail-closed release-inventory
proof, landed as `tools/release-inventory.test.mjs` in the green gate.

**DEC-083 is a pre-ship posture, not legal clearance.** The counsel items are DEFERRED BY
NAME to first commercial sale, and the release-inventory test fails if any of them silently
vanishes from `CLAIMS.md`, `DECISIONS.md` or this register:

- **CLAIM-013 - the shipped chart set**: the six derived overlay definitions (published
  equations + printed page values), their keep-listed digitized remnants (apparent-position
  mineral points and annotation lines), and the Por-4/Por-5 neutron equivalence tables in
  `src-tauri/src/neutron_charts.rs` - a registered digitized asset that was never inside the
  thirteen-id ruling and still ships because the neutron-porosity module math depends on it.
- **CLAIM-012 - the endpoint library**: engineering custody is complete (DEC-078 per-value
  provenance); whether the curated selection carries a claim beyond its rows is counsel's.
- **Dependency attention items**: `THIRD-PARTY-LICENSES.md` is generated and gate-current -
  inventoried, not legally cleared.

Before first commercial sale: obtain the counsel dispositions above, then record them here
and in `CLAIMS.md`; nothing else reopens at Gate 2.

---

## What was solved on 2026-08-17, and what it changed

A working session closed three rows outright, materially advanced three more, and corrected the
register against its own chapters **seven times**. Read this before trusting a blocker below.

### Rows closed

- **`SB-DIO-056`** — the LAS writer took `depth[1] - depth[0]` as the `STEP`. On a merged or
  depth-shifted well that is not the step, and a conforming reader is entitled to rebuild depths
  from `STRT`/`STEP`, silently re-gridding the data. Now verified across the whole index, declaring
  `STEP 0` when it varies (LAS 2.0's own provision) and naming the depth where it changes.
  **The comparison is made on the emitted fixed-decimal-4 TEXT, not the stored `f32`s** — at ~1000 m
  an `f32` resolves to ~0.00006, so a perfect 0.1524 m frame is not bit-identical under subtraction
  and would be called irregular. Proven by mutation, not argued.
- **`SB-DIO-062`** — the declared text-encoding inventory is published as a contract and pinned.
  No ambiguity rule was invented, because the decode order is TOTAL. The cp1252 fallback's
  inability to fail is pinned as a **property over every byte**, not one example.
- **`SB-POR-054`** — density and sonic wrote the same identity two ways.
  `modules::two_endpoint_fraction` is now the one subtraction order and **no call site writes a
  subtraction**, so the half-flipped form — which returns a NEGATIVE porosity from an expression
  that reads like both textbooks — is unwritable rather than discouraged. Bit-identical: negating
  both halves of a quotient is exact in IEEE-754, and all backend tests passed unchanged.

### Rows advanced, with the remaining question stated exactly

- **`SB-SAT-027`** — the register claimed two un-cross-asserted solvers. **There is one**: `sw_sim`
  delegates to `sandimin`, and 15 `modules.rs` sites route there. The `n = 2` fast path is now
  proven equal to the general root-finder. **One clause left, and it is a method ruling:** the
  chapter specifies Geolog's guards (seed 0.5, 20 iterations, `|delta| < 1e-5`) while the solver
  uses 60-step bisection — arguably better, since bisection on a monotone function is
  unconditionally convergent, but swapping a specified numerical method is Jauhar's call.
- **`SB-CUT-001`** — the net-pay clip rule existed **three times**. All three consumers now route
  through `workflow::sample_incl_thickness`; proven number-neutral. **One clause left, and it moves
  reserves:** the requirement wants a model parameter defaulting to CENTRED while every shipped path
  uses FORWARD, and the two differ by half a sample step at each zone boundary.
- **`SB-DBM-032`** — DEC-028 ruled both one-handle parameter forms are refused, and a test now pins
  it. Honest limit: defeating the structural ordinal guard did NOT turn the test red, because schema
  validation refuses too. That is defence in depth, and it means the test proves the CONTRACT rather
  than any single guard.

### The register was wrong seven times, and in one direction

Every correction made the work SMALLER or different, never larger: a fixed arity mistaken for a cap;
a landed dependency recorded as outstanding; a chapter permitting a fast path recorded as forbidding
it; two engines that were one; a stale `SB-POR-005`/`SB-POR-040` link in neither requirement text.
**Read the chapter before believing a blocker.**

### The stale class column

Roughly **31 of the 59** blocked rows cite a `DEC-` number in their Blocker class. Those decisions
were RULED on 2026-08-17. Those rows are ordinary engineering waiting on implementation, not
questions waiting on Jauhar — but the class column still reads as though the decision were open, and
that is what made an earlier summary mis-state the shape of this gate.

## What I actually need from you (plain language)

There are 58 blocked rows, but they are not 58 separate problems. They collapse into four kinds of
thing. **As of 2026-08-17 the decision bucket is empty** - 38 of them are now ordinary engineering
work; SB-DIO-056, SB-DIO-062 and SB-POR-054 have since shipped. What still needs you is 8 documents
and 1 legal question.

### 1. Decisions only you can make — all cleared 2026-08-17

**Every decision that blocked a Gate 2 row is now ruled.** You answered twenty-five of them in one
sitting on 2026-08-17, with a twenty-sixth closed by reference (DEC-026 asked the PHIE-floor
precedence question DEC-043 had already answered), and **41 of the 62 blocked rows are
decision-cleared** — one of them (SB-DIO-056) has since shipped and the rest are ordinary
engineering work waiting to be done, not questions waiting on you.

Twice you rejected the question's premise rather than picking one of my options, and both times you
were right:

- **SB-POR-054 → DEC-049.** I asked which of two spellings of the porosity identity was canonical.
  You said *"both true, its different type of porosity."* Density and sonic porosity are different
  measurements with different endpoints, each carrying its own conventional form. The MUST is
  satisfied by declaring each type's convention, not by forcing one spelling onto both. What is
  actually dangerous here is neither published form — it is the MIXED ordering, which returns a
  negative porosity from an expression that looks like both textbooks at a glance.
- **SB-CORE-007 → DEC-051.** I asked how strict the output-name collision rule should be. You said
  modules already carry distinct defaults (`PHIE_DEN`, `PHIE_SON`) and a user typing `PHIE` is
  deliberately replacing a result. So a default collision is a registry bug and a typed one is a
  supported workflow — which dissolves the row's difficulty instead of answering it: with unique
  defaults, no rule has to tell canonical results from working aliases from categorical flags.

Five rulings each cleared several rows at once. **DEC-031** — one coded correction-state curve, the
applied-step manifest in the log-set archive, partial correction over the intervals a caliper
actually covers — settled three storage requirements on one answer and, through the chapter's own
OI-7, settled **DEC-032** as well. **DEC-021** (a per-module source digest, replacing a package
version that does not move when a module's arithmetic does) with **DEC-023/024** closed all four
arms of the re-run manifest. **DEC-025** authorized the neutron matrix-basis seam. **DEC-036**
authorized a narrow versioned CLY provenance registry, because unlike DEC-035's deferred cull these
deferred rows own infrastructure that already-approved rows cannot be built without.

Two rulings carry a stated risk rather than a buried one. **DEC-022** — legacy timestamps are WIB —
is only correct if every legacy record came off a UTC+7 machine, and a timestamp carries no evidence
of the clock that made it. **DEC-056** deliberately does *not* take that route for depth: legacy
depths are typed MD and stay untied to any datum, because the KB-to-GL difference is commonly 5–15 m
and a wrongly declared reference offsets a whole well with nothing in the numbers to show it.

**Six decisions remain open — DEC-004 through DEC-009 — and none of them blocks a Gate 2 row.** They
are G5 commercial questions: licence unit, commercial model, support hours, update window, benchmark
hardware profile, lineage granularity.

### 2. Documents I need you to find — 7 rows

These are not decisions. They are stopped because the source that fixes the number is not in the
repo and I will not invent it. This is the same shape as SB-POR-021 earlier: you sent the 1977
Bateman & Konen paper and the block dissolved in one message.

- **`SB-CORE-015`** — **RESOLVED 2026-08-17. The spec is found.** You supplied
  `energistics.org/sites/default/files/RP66/V1/Toc/main.html`, which is the full normative RP66 V1
  document — sections 1-7 plus Appendix B (Representation Codes) and Appendix E (Checksum), and
  section 2 carries the Storage Unit Label, Visible Record and Logical Record Segment framing. This
  row is no longer waiting on you. It is now a **substantial engineering row**, not a quick one: a
  conforming writer means storage-unit framing, record segmentation, EFLR/IFLR, representation codes
  and the checksum. Nothing partial ships in the meantime.
- **`SB-ENV-004`** - **RESOLVED 2026-08-19 by DEC-077.** All 29 rows ruled (blanket Mine with the T_REF and SALW adoptions, BHT/TD_BHT ABSENT); the row is DONE and off the blocked inventory below.
- **`SB-DBM-005`** — the complete registered-module derivation-source map. **`SB-DBM-010` unblocks
  itself the moment this lands**, so it is two rows on one document.
- **`SB-DIO-011`** — a named source for every accepted deviation alias.
- **`SB-DIO-057`** — a versioned ENV-reviewed family registry with a source for every member.
- **`SB-CORE-005`** and **`SB-CORE-044`** — sources *and* counsel dispositions; these two sit
  partly in the legal bucket below.

### 3. Rows where the tracking file is simply wrong

Worth knowing before you spend time on any of these. **Four Group F rows in a row** carried blockers
that the chapter itself does not support — SB-POR-026, 028, 043 and 044 — and one of them
(**SB-POR-043**) turned out to be fully implementable once I read `11_porosity.md` instead of the
summary. It shipped this session. So if a row here says "needs a decision", it is worth one minute
checking the chapter before you spend an hour deciding.

### 4. Rows waiting on other rows — no action from you

These unblock themselves as their prerequisite lands. Example: **SB-POR-025** needs somewhere to get
salinity-dependent endpoints from, and both of its sources are shut — one is deferred outside the
manifest, the other is SB-POR-021, whose paper you already sent but whose evaluator I have not
written yet. Build that, and 025 opens on its own.

### One legal item

**SB-PLT-024** is `BLOCKED-LEGAL`. That is a lawyer question, not a petrophysics one.

### Honest caveat

I read the POR rows closely this session. The SAT and CUT rows still carry the original audit's
wording, and given what I found in Group F, some of those blockers are probably stale too. Do not
treat a blocker here as proven until it has been re-read against its chapter.

---

- Evidence boundary: Automated, Visual, Manual and Field evidence are separate.
- Scientific boundary: a value, limit, tolerance, endpoint or family classification is cited or
  remains absent. Current code is never its own authority.

`BLOCKED` does not mean one thing. A row can require a product-owner decision, an authoritative
source, legal disposition, or later engineering follow-through. Treating all four as ordinary
coding gaps would invite silent wrongness or transfer engineering responsibility to Jauhar.

## Exact blocked-requirement inventory

The first column is mechanically compared with `gate2-program.json`. Every live blocked
requirement appears exactly once.

None. The set is empty as of 2026-08-20: the final two rows (`SB-PLT-024`,
`SB-CORE-044`) closed under DEC-083 with their counsel items moved to the first-sale
deferred register above. Their full closure narratives live in `requirements.csv`; the
superseded blocker rows live in git history.

## Product-owner decision packet

The detailed alternatives and affected tests for `DEC-021` through `DEC-037` live in
`DECISIONS.md`. The recommended architecture bundle is:

1. Use build-derived per-module semantic identities rather than package version or whole-executable
   identity.
2. Label legacy timestamps `ZONE_UNKNOWN`; store new timestamps as UTC instants and convert only at
   display.
3. Authorize narrow identity and metadata seams required by approved requirements without silently
   enabling deferred capabilities.
4. Keep the PHIE floor absent until the conflicting binding contract and sources are adjudicated.
5. Preserve queryable `REQUIRED_UNSET`; refuse unsafe legacy one-handle parameter packs.
6. Split exact mnemonic requests from semantic-family requests.
7. Keep correction and reason channels typed; do not invent categorical `f32` codes.
8. Exempt only explicitly enumerated repair module/output/mode paths from masking.
9. Retain the under-five Hampel refusal and keep culling deferred unless the pilot manifest is
   explicitly revised.
10. Authorize narrow categorical CLY provenance infrastructure; require source-scoped sentinel
    handling in which explicit `NoNull` wins.
11. Classify SSC/SSPW under the POR twin contract and define exactly what their unlimited lineage
    bypasses before authorizing any protected-file change.
12. Approve the singular POR branch/limit representation and its complete initial registry; a
    binary bit and an undocumented integer code are both insufficient.

Approval of a decision authorizes the engineering contract; it does not itself mark any affected
requirement done. Each requirement still needs its named test, full gate, evidence update and
separate commit.

## Source and legal intake packet

The following evidence can be supplied without choosing a software design:

- Exact per-value endpoint-library construction custody or primary sources.
- API RP66 V1 writer material and the first-pilot DLIS-export scope decision.
- One primary citation or approved first-principles derivation document per shipping module.
- The source for every accepted deviation-survey index alias.
- A cited whole-index `STEP` tolerance, or an explicit exact-equality rule.
- A versioned source-cited logarithmic curve-family registry.
- A complete Windows single-byte code-page inventory and ambiguity rule.
- Sources for the 19 source-required environmental parameters.
- Counsel dispositions for the endpoint library, chart payloads (themes closed by DEC-074) and dependency
  attention items.

Providing a number without its source does not unblock a row. Providing a legal opinion without
the asset and distribution route it covers does not unblock a legal row.

## Maintenance contract

- `gate2-program.json` owns the blocked ID set and counts.
- `requirements.csv` owns the detailed current evidence, blocker and next action per requirement.
- `DECISIONS.md` owns Jauhar's product choices.
- This file owns the one-place human explanation and intake checklist.
- `STATUS.md` links here and carries only the one-minute roll-up.
- The Gate 2 program test fails when this file omits, duplicates or invents a blocked ID, or when
  the dashboard stops linking here.

When a blocker closes, update all four stores in the same requirement increment. Manual and Field
acceptance remain Jauhar-owned and cannot be inferred from a green automated gate.
