# PRD v2 spine corrections pending

This file records verified corrections for a separate spine-maintenance session. Chapter-writing
lanes do not edit the spine directly.

## SP-001 — Tier-C implementation policy

- **Status:** **RESOLVED 2026-08-08** by commit `1ae005d`.
- **Spine location:** `docs/PRD_v2/00_INDEX.md` §0.4, current line 102.
- **Former claim:** Tier C is never implemented, approximated or reverse-engineered.
- **Verified source:** `docs/PRD_v2/CONTRACT.md:146-169`, amended 2026-08-07, says that
  reconstruction is prohibited but a real Tier-C user need requires independently derived
  capability from public literature, primary sources and first principles.
- **Resolution:** §0.4 now bars transcription, reconstruction, approximation from vendor outputs and
  reverse engineering while requiring independently derived capability where a real user need is
  established, with SandiBumi's own method, defaults, limitations, tests and supported `Betters:`
  statement.

## SP-002 — Unconventional gas-content scope

- **Spine location:** `docs/PRD_v2/01_PRODUCT.md` §4.3, current line 190.
- **Current claim:** the shipped unconventional suite includes gas-in-place.
- **Verified source:** `src-tauri/src/unconventional.rs:247-281` explicitly defines per-sample gas
  content in scf/ton and names those intensive outputs `GIP_*`; lines 286-364 take no thickness or
  area and produce no Bcf volume.
- **Direction wrong:** the spine overstates the shipped physical scope. Intensive gas content ships;
  extensive areal gas-in-place does not.

## SP-003 — Omovie Sonic Saturation independent-derivation evidence

- **Chapter location:** `docs/PRD_v2/12_saturation.md` §7.4 and ESC-10.
- **Draft class:** **C-1**, because CONTRACT §2.2 names Omovie Sonic Saturation under US 12,242,011
  B2. This remains a draft legal classification until the claims are read.
- **Gap:** neither the patent claims nor the relevant published sonic-saturation literature is held
  in the saturation evidence package. The real user need, a lawful independent route, supported
  `Betters:`, defaults, limitations and an owning `SB-SAT-*` requirement/test package therefore
  cannot yet be stated.
- **What closes it:** acquire and review the claims and primary literature, establish the user need,
  then either write the independently derived package or state why no qualifying need exists.

## SP-004 — Mineral-solver sonic bridge independent-derivation evidence

- **Chapter location:** `docs/PRD_v2/13_mineral-solver.md` §7.5 and ACQ-11.
- **Draft class:** **C-2**, because dossier G-6 identifies IP's Wyllie ↔ Hunt-Raymer `Cp` bridge as a
  proprietary vendor-fitted four-term regression and identifies no patent claim.
- **Gap:** the lawful candidate route — the full nonlinear sonic response, without the fitted bridge
  or its coefficients — is supported at capability level, but the primary Wyllie (1956/1958) and
  Raymer–Hunt–Gardner (SPWLA 1980) papers are not held. Method limitations, defaults, a fully
  supported `Betters:` statement and an owning `SB-MIN-*` requirement/test package remain unwritten.
- **What closes it:** acquire the named papers, derive and document SandiBumi's own method and bounds,
  then add analytic, boundary, continuity and regression tests before implementation is required.

## SP-005 — Missing record of the superseded PRD-v1 follow-on documents

- **Spine location:** `docs/PRD_v2/06_SEQUENCING_AND_GATES.md` §26, outside this lane's ownership.
- **Verified fact:** `docs/V1_SCOPE.md`, `docs/RELEASE.md` and `docs/TARGET_ARCHITECTURE.md` exist on
  unmerged branch `docs/prd-and-security-hardening` at commit `18da8b0`; `docs/ARCHITECTURE.md` does
  not. `docs/PRD_v2/00_INDEX.md` §0.1 now records Jauhar's stated 2026-08-08 decision to discard the
  three existing documents as superseded by PRD v2.
- **Gap:** §26's “Decisions already made” table does not currently contain that discard decision,
  although the task direction says it is recorded there.
- **What closes it:** in a spine-maintenance lane, add the 2026-08-08 discard/supersession decision
  to §26 so the index citation and consolidated decision register agree.

## SP-006 — Requirements total is 931, not 932

- **Index location:** `docs/PRD_v2/91_REQUIREMENTS_INDEX.md` Reading rules and Roll-ups.
- **Counted source:** requirement definitions in `04_CORE_REQUIREMENTS.md` §15 and every chapter's
  §4, `10_clay-volume.md` through `27_ip-install-blockers.md`.
- **Discrepancy:** the commissioning brief states 932 distinct requirement IDs. The mechanical,
  de-duplicated count is 931: 25 `SB-CORE` definitions and 906 domain definitions. No duplicate
  requirement definition accounts for the difference, so the index records 931 and does not
  manufacture a row.

## SP-007 — `SB-CORE` requirement-number gaps

- **Chapter location:** `docs/PRD_v2/04_CORE_REQUIREMENTS.md` §15.
- **Discrepancy:** the defined requirement sequence omits `SB-CORE-008`–`009`, `SB-CORE-016`–`029`,
  and `SB-CORE-037`–`039`.
- **Disposition:** unassigned only; no number is filled, reused, or renumbered in this lane.

## SP-008 — `SB-CORE` test-number gaps — CLOSED 2026-08-09

- **Chapter location:** `docs/PRD_v2/04_CORE_REQUIREMENTS.md` §15, the per-requirement **Verified
  by** clauses; also recorded in `docs/PRD_v2/RESUME.md` §5.
- **Discrepancy:** `SB-CORE-T04` through `SB-CORE-T08` were unassigned between defined `T03` and
  `T09`, and the shorthand index entry obscured the seven-test ownership.
- **Disposition:** closed by the `SB-CORE-002` adjudication and implementation. `SB-CORE-T03`
  through `T09` each own one explicit, non-overlapping reporting-surface contract for one of the
  seven recorded R4/R18/R19/R21 violations, and all seven named regressions now pass. `T07` also
  closes the remaining production defect by carrying a Pay Summary degradation into the batch/run
  result beside the still-written PDF.

## SP-009 — Porosity requirements omit status and tests omit T26–T27

- **Chapter location:** `docs/PRD_v2/11_porosity.md` §4 and §6.
- **Discrepancy:** all 62 requirements state priority but no per-requirement status, although
  `CONTRACT.md` §3 requires the status vocabulary on each requirement. Section 6 defines 41 test
  IDs but leaves numeric IDs `SB-POR-T26` and `SB-POR-T27` unassigned, using `T14b` and `T18b`
  within the stated `T01 … T27` carried set instead.
- **Disposition:** status cells remain empty and the test gaps remain unfilled in the index.

## SP-010 — Fifteen installer requirements omit priority

- **Chapter location:** `docs/PRD_v2/27_ip-install-blockers.md` §4.
- **Discrepancy:** `SB-INS-006`, `-007`, `-009`, `-011`–`013`, `-017`–`022`, and `-024`–`026`
  state a contract-defined status but no priority. The chapter's eleven explicit `P0` tags match
  its front-matter P0 count; that does not supply priorities for the remaining fifteen.
- **Disposition:** priority cells remain empty; no `P1`–`P4` value is inferred.

## SP-011 — Two `SB-CORE` statuses are outside the contract vocabulary

- **Chapter location:** `docs/PRD_v2/04_CORE_REQUIREMENTS.md` §15.3.
- **Discrepancy:** `SB-CORE-030` uses `UNMEASURED`; `SB-CORE-033` uses
  `ABSENT — designed, parked`. Neither is one of `ABSENT`, `PARTIAL`, `PRESENT-OK`,
  `PRESENT-DIVERGENT`, or `PRESENT-UNVERIFIED` from `CONTRACT.md` §3.
- **Disposition:** both values are carried verbatim in the index and are not mapped to a legal
  status.

## SP-012 — The lack-of-compaction correction runs BACKWARDS on its own shipped default

- **Code location:** `src-tauri/src/modules.rs` `phi_son`, the `cp` binding (~`:995`) and the
  `DT_SH` parameter declaration in `phi_son_spec` (~`:968`).
- **Shipped behaviour:** `Cp = DT_SH / 100`, and the Wyllie porosity is divided by it. `DT_SH`
  ships at **90 µs/ft** with a declared range of 60–150, so `Cp` spans **0.6 – 1.5**.
- **Verified source:** Raymer, Hunt & Gardner, SPWLA Twenty-First Annual Logging Symposium,
  July 8–11 1980, quoted in `docs/method_sonic_porosity.md` §2 — *"Cp is always greater than
  unity. Values ranging from 1 to 1.3 are common, with values as high as 1.8 occasionally
  observed."*
- **Consequence:** with `OPT_CP=ON` and the shipped `DT_SH` default, `Cp = 0.90`, so dividing
  **raises** porosity by 11 %. A lack-of-compaction correction exists to bring an over-high
  porosity DOWN. The entire lower half of the declared `DT_SH` range is physically impossible
  per the source, and the only guard is `dt_sh > 0.0`.
- **Why it qualifies as Tier −1 rather than an ordinary P0:** it is silent. The result computes,
  plots, sums into a pay summary and prints in a deliverable with nothing flagged, and it moves
  porosity in the direction that over-reports pay.
- **Candidate fix, not applied here:** clamp `cp = max(1.0, dt_sh / 100.0)`, or refuse a
  `DT_SH < 100` by name under `SB-CORE-002`. Which of the two is Jauhar's call; a clamp is silent
  where a refusal is not.

## SP-013 — The `RHG` option is a one-segment approximation under a three-segment name

- **Code location:** `src-tauri/src/modules.rs` `phi_son`, the `rhg` branch (~`:1001`), and the
  `OPT_SON` option in `phi_son_spec`.
- **Shipped behaviour:** `PHIT = 0.625 · (DT − DT_MA) / DT`.
- **Verified source:** the coefficient `0.625` and this algebraic form appear **nowhere** in
  Raymer, Hunt & Gardner 1980. That paper's φ < 37 % form is `V = (1 − φ)² · Vma + φ · Vf`,
  quadratic in φ, and the paper states directly that no single algorithm covers the porosity
  range — *"The response curve was, therefore, divided into three segments."* Breakpoints are
  cited at 0.37 and 0.47. See `docs/method_sonic_porosity.md` §3.
- **Consequence:** a widely used field approximation ships under the name of the transform it
  approximates, and `0.625` is `SHIPPED-UNCITED` against the source it is named for. This is
  `SB-CORE-006` — one name, one equation.
- **Two dispositions, both legitimate, neither chosen here:** rename the option to what it is
  (`RHG_APPROX`) and cite the approximation's own source, **or** implement RHG80's three
  segments. The second makes `13_mineral-solver.md` §7.5's candidate `MIN-C2-1` reachable, since
  the published transform is the lawful route away from IP's proprietary `Cp` bridge.

## SP-014 — A basin name ships in module dialog text

- **Code location:** `src-tauri/src/modules.rs` `phi_son_spec` `doc` string (~`:960`).
- **Text:** *"undercompacted shaly sands (e.g. shallow Mahakam delta) read porosity high…"*
- **Rule:** `CONTRACT.md` §2.3 and `CLAUDE.md`'s provenance discipline — no client, field, block,
  basin or operator name anywhere in the tree. Name the physical condition instead.
- **Why it matters more than an ordinary comment:** a manifest `doc` string is rendered in the
  auto-generated parameter dialog, so this is **shipped user-visible text**, not an internal note.
- **Disposition:** *"undercompacted shaly sands"* already carries the meaning; the parenthesis is
  the whole violation. A one-line deletion, no judgement required.

## SP-015 — Two `phi_son` behaviours were correct but uncited, and are now citable

Recorded because a correct-and-uncited value is still a `CONTRACT.md` §2 gap, and closing one is
worth as much as finding a defect.

- **`Cp` never applies to RHG.** The code comment reasoned this out unaided. RHG80 states it:
  *"Use of the new transform eliminates the need for the 'lack of compaction' correction factor.
  Using the proposed transform, sonic transit time yields porosity directly."*
- **`Cp = DT_SH / 100`.** Credited to Hilchie in the code comment; RHG80 gives the same estimator
  independently — *"The simplest is to use the sonic transit time observed in nearby shales
  divided by 100."*
- **Disposition:** cite `docs/method_sonic_porosity.md` at both sites when either is next touched.
  No behaviour change.
