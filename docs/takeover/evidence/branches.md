# Gate 1 branch and commit inventory

The initial census was measured after `git fetch origin` on 2026-08-10. The checked-out branch
remained `codex/sandibumi-takeover-gate1`; no branch was switched, moved, rebased, merged or deleted
during that inventory. The later GitHub-master baseline integration is recorded separately below.

## Accepted baseline

- Local `master`: `fa6b5ba24975437ed9d9260e280cec2f3608d496`.
- Fetched `origin/master`: `29833735816d9e5be954afafd9ceb71fd856e3f0`.
- Takeover HEAD inspected: `32115dac3f6b20cc396c9a21c32c3731972c0326`.
- Accepted tested source baseline: `706fe59d50fa673066a1dbfb87f0948074656f4e`.
- Local `master` is one commit behind `origin/master`. The remote commit was classified below as an
  `ACCEPTED-CANDIDATE` and was not silently integrated during the inventory.

## Ref census

- Refs inspected: `62` after excluding symbolic `origin/HEAD`.
- Refs fully contained in local `master`: `29`.
- Refs with at least one commit not contained in local `master`: `33`.
- Distinct `git cherry` patch records across those refs: `66`.
- Non-contained refs: `backup/pre-vs-strip`, `claude/agitated-mcclintock-ab7e23`,
  `claude/elastic-jemison-86d664`, `codex/sampling_prob`, `codex/sampling-problem`,
  `codex/sandibumi-takeover-gate1`, `docs/claude-md-progressive-disclosure`,
  `docs/context-audit-fixes`, `docs/prd-and-security-hardening`, `feat/core-040-matrix`,
  `feat/ignored-tests`, `feat/ins-p0`, `feat/missing-tests`, `feat/p0-core`, `feat/plt-p0`,
  `fix/undeclared-surface-tokens`, `old-work-2026`, `origin`,
  `origin/chore/gitignore-local-artifacts`, `origin/chore/gitignore-model-router`,
  `origin/claude/kimi-k3-sandibumi-subagents-7grc3b`,
  `origin/claude/sandibumi-worktree-codex-bwzhjk`, `origin/codex/sampling_prob`,
  `origin/docs/claude-md-progressive-disclosure`, `origin/docs/context-audit-fixes`,
  `origin/docs/prd-and-security-hardening`, `origin/feat/core-040-matrix`,
  `origin/feat/ignored-tests`, `origin/feat/ins-p0`, `origin/feat/missing-tests`,
  `origin/feat/p0-core`, `origin/feat/plt-p0`, `origin/master`.
- Fully-contained refs: `chore/fresh-clone-check`, `claude/gracious-kowalevski-cbfbfb`,
  `docs/prd-v2-gap-analysis`, `docs/prd-v2-requirements-index`, `feat/dio-p0`, `feat/dio-p1`,
  `feat/ins-p1`, `feat/ml-feature-transforms`, `feat/ml-pane-round3`, `feat/p0-env-cut`,
  `fix/ml-blind-score-leak`, `fix/ml-cv-score-estimator`, `master`,
  `origin/chore/rename-and-release-hardening`, `origin/docs/prd-v2-amendment-sweep`,
  `origin/docs/prd-v2-chapters`, `origin/docs/prd-v2-requirements-index`,
  `origin/feat/dio-p0`, `origin/feat/dio-p1`, `origin/feat/ml-feature-transforms`,
  `origin/feat/ml-hdbscan`, `origin/feat/ml-knn-propagation`, `origin/feat/ml-pane-round3`,
  `origin/feat/ui-design-tokens`, `origin/fix/ml-blind-score-leak`,
  `origin/fix/ml-cv-score-estimator`, `origin/fix/ml-input-one-well-picker`,
  `origin/fix/ml-input-slots`, `origin/fix/r28-tops-wrong-well`.

## Patch-equivalent commits

`git cherry -v master <ref>` reports `-` for the first 52 rows. The final row is `+` against
local `master` but is patch-equivalent to fetched `origin/master`: both commits have tree
`d5dc6acf98cd807b3cc611844126bf4f2a560e51`, and `git cherry` reports `-` in both directions.

| Commit | Refs | Subject | Classification | Evidence |
|---|---|---|---|---|
| `01feeba` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-PLT-028 characterize plot invalidation layers | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `037795d` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-011 disclose Pickett identifiable product | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `041ece5` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-INS-007 pin interpreter-specific remediation | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `108993e` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-031 export plot reduction manifests | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `10bd251` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-INS-006 pin package preflight ordering | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `16470b0` | `feat/ins-p0`, `origin/feat/ins-p0` | SB-INS-010 materialize immutable settings templates | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `19ccf64` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-003 type overlay quantity and unit binding | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `26c1a3e` | `feat/ignored-tests`, `origin/feat/ignored-tests` | Fix ignored field workflow contracts | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `27039d0` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-020 add provenance to plot parameter writes | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `2e38d6a` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-023 block charts without complete provenance | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `3399a4b` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-PLT-025 pin unknown template field preservation | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `34ee79e` | `feat/core-040-matrix`, `origin/feat/core-040-matrix` | SB-CORE-040 index verification by capability | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `3a77024` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-008 distinguish percentile and range position | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `3abf150` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-CORE-044 characterize Tier-C policy register | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `3c7adaa` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-INS-002 pin native operation without Python | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `4c436c9` | `feat/ins-p0`, `origin/feat/ins-p0` | SB-INS-014 key parameter-pack rows semantically | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `5197cc7` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-013 apply channel-specific range policies | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `535b43c` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-INS-021 characterize support report fragments | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `56d39cd` | `backup/pre-vs-strip`, `claude/agitated-mcclintock-ab7e23`, `old-work-2026` | Finish packaged-mode checks | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `57f5e10` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-001 persist semantic and concrete plot bindings | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `582cec2` | `backup/pre-vs-strip`, `claude/agitated-mcclintock-ab7e23`, `old-work-2026` | Rename prep | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `591cff0` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-INS-017 characterize raw unit and encoding retention | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `5d1e0d9` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-PLT-010 characterize regression result payload | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `5e0b52c` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-PLT-030 pin keyboard canvas accessibility | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `62c705c` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-015 preserve decimation identity and manifest | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `7a1c399` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-PLT-026 characterize vector and raster export labels | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `86c9066` | four local/remote docs refs | One home for the rules | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `8b99de9` | `feat/ins-p0`, `origin/feat/ins-p0` | SB-INS-001 qualify the per-machine Windows MSI | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `8d5d0ea` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-005 gate unit limits on dimensional audit | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `94301aa` | `feat/ins-p0`, `origin/feat/ins-p0` | SB-INS-004 centralize capability dependency detection | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `972e954` | `origin/chore/gitignore-model-router` | Ignore model-router plugin folder | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `9c29265` | `feat/ins-p0`, `origin/feat/ins-p0` | SB-INS-005 explain session Python resolution | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `a39ae5e` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-014 allocate after finite-pair screening | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `a55de2b` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-002 expose axis range precedence | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `a7c8844` | `fix/undeclared-surface-tokens` | Give sticky header an opaque background | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `abfb626` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-004 separate validity filters from display clipping | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `af7af10` | local/remote progressive-disclosure refs | Move reasoning to docs | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `b85e152` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-016 reconcile depth steps conservatively | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `bb0c488` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-CORE-042 characterize manual green gate | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `bcd875f` | `feat/plt-p0`, `origin/feat/plt-p0` | SB-PLT-006 canonicalize histogram binning | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `ca16b65` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-PLT-009 characterize statistics disclosure gap | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `ca6c9e9` | `feat/ins-p0`, `origin/feat/ins-p0` | SB-INS-003 derive truthful prerequisite surfaces | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `d13b91e` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-PLT-018 characterize linked brush state | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `db77ef9` | `feat/ins-p0`, `origin/feat/ins-p0` | SB-INS-023 gate serviced Windows matrix | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `dd26cb3` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-PLT-029 pin stale generation disposal | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `de10062` | `feat/ins-p0`, `origin/feat/ins-p0` | SB-INS-016 type unit conversions by quantity | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `f14371c` | `fix/undeclared-surface-tokens` | Record three dark-theme surfaces | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `f25ff8b` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-INS-018 characterize missing unit mappings | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `f333888` | `feat/missing-tests`, `origin/feat/missing-tests` | SB-PLT-035 characterize clay overlay equation parity | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `f36433f` | `feat/ins-p0`, `origin/feat/ins-p0` | SB-INS-015 refuse ambiguous parameter packs | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `fb4b5f2` | `feat/ins-p0`, `origin/feat/ins-p0` | SB-INS-008 gate offline managed deployment | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `febd27f` | `origin/chore/gitignore-local-artifacts` | Ignore vendor corpus and local artifacts | `PATCH-EQUIVALENT` | `git cherry` `-` |
| `b8792f4` | `codex/sampling_prob`, `origin/codex/sampling_prob` | Preserve imported LAS sets on native grids | `PATCH-EQUIVALENT` | same tree as `2983373`; `git cherry` `-` both directions |

## Unique candidate commits

| Commit | Refs | Subject | Owned paths | Requirement/evidence | Classification | Reason |
|---|---|---|---|---|---|---|
| `b272d19` | takeover branch | Define five-gate takeover | takeover design | approved governance design | `ACCEPTED-CANDIDATE` | contained by takeover HEAD |
| `16d0e14` | takeover branch | Plan Gate 1 foundation | Gate 1 plan | reviewed execution plan | `ACCEPTED-CANDIDATE` | contained by takeover HEAD |
| `706fe59` | takeover branch | Establish tracker | tracker, ledger, gate | 9 tracker tests; full gate green | `ACCEPTED-CANDIDATE` | accepted tested baseline |
| `32115da` | takeover branch | Record baseline receipt | receipt and dashboard | post-commit tracker check green | `ACCEPTED-CANDIDATE` | contained by takeover HEAD |
| `bb807ca` | CORE branches and takeover | SB-CORE-001 depth units | CORE production/tests/docs | named requirement and tests; current full gate green | `ACCEPTED-CANDIDATE` | contained by takeover HEAD |
| `78bd21d` | CORE branches and takeover | Adjudicate SB-CORE-002 reporting | PRD adjudication | explicit seven-surface adjudication | `ACCEPTED-CANDIDATE` | contained by takeover HEAD |
| `d25c274` | CORE branches and takeover | Lock SB-CORE-002 reporting regressions | reporting production/tests/docs | named reporting regressions; current full gate green | `ACCEPTED-CANDIDATE` | contained by takeover HEAD |
| `2983373` | `origin`, `origin/master` | Preserve imported LAS sets on native grids | 27 data/store/view/export paths | named Rust/frontend tests plus nine unchecked field scenarios | `ACCEPTED-CANDIDATE` | canonical remote commit; integrated by the follow-up below, while its field scenarios remain unchecked |
| `18da8b0` | old PRD/security refs | V1 scope, architecture and release suite | three legacy docs | `PRD_v2/00_INDEX.md:27-31` explicitly names this branch and says PRD v2 supersedes it | `SUPERSEDED` | authoritative PRD v2 absorbed the documents |
| `82e56c9` | remote Kimi branch | Add Kimi delegation tier and launcher | `.gitignore`, `CLAUDE.md`, Kimi docs/tool | current `CLAUDE.md:1137-1162` delegates the tier ladder to machine-level policy | `SUPERSEDED` | provider-specific launcher is not the current collaboration contract |
| `d1f0c1e` | old pre-strip refs | PRD and Architecture | machine `.vs` artifacts, AGENTS, SSC, design doc | no single requirement, owned test or clean scope | `REJECTED` | mixes large machine-local binaries with a protected numeric-module change |
| `fb01bc0` | remote worktree ref | Remove numeric NEVER TOUCH block | `docs/lane_prompt.md` | current lines 91-100 retain that guard explicitly | `REJECTED` | weakens a safety boundary without replacement evidence |
| `0d5389e` | old Claude ref | Remove raw Tops colour interpolation | Tops UI, safe DOM, REVIEW/progress | current `topsPanel.ts` still interpolates stored `tops.color` into an `innerHTML` style attribute; old 19/19 browser evidence is not a maintained regression | `ACCEPTED-CANDIDATE` | port only the narrow DOM-construction fix in Gate 2 after a current observable regression fails; do not cherry-pick the stale whole commit |

## Explicit exclusions

- Symbolic `origin/HEAD` was excluded because it is an alias, not an independent ref.
- Refs with zero commits ahead of local `master` were recorded in the ref census but produced no
  non-contained commit row.
- Patch-equivalent commits are not integration candidates merely because their hashes differ.
- No classification asserts that an unchecked `REVIEW.md` scenario is complete.

## Baseline integration follow-up — 2026-08-10

- Active development root: `D:\XX. SandiBumi`.
- Takeover parent: `54bc93839dd24b54fa6646145073144c55783f5e`.
- GitHub default-branch parent: `29833735816d9e5be954afafd9ceb71fd856e3f0` from
  `origin/master` / `origin/HEAD`.
- Integration method: one non-fast-forward merge without rebase or a checkout of `master`; Git's
  three-way merge reported no textual conflicts.
- The combined gate exposed a semantic conflict rather than a Git conflict: the earlier
  `SB-CORE-T04` partial-success contract contradicted `2983373`'s deliberately atomic LAS delivery.
  Jauhar authorized retaining atomicity. `04_CORE_REQUIREMENTS.md` and the named reporting-surface
  test now require a named per-file error and no partial well; the production transaction was not
  weakened or split.
- The canonical Windows checkout also exposed CRLF conversion in the byte-current generated PRD
  audit. The artifact is pinned to LF and the existing named test asserts checkout-stable bytes.
- The exact adjudicated merge tree passed `tools/check.ps1`: 16 tracker, 13 frontend and 917 Rust
  tests passed; 0 failed and 36 Rust tests remained ignored for their recorded reasons.

## Integration actions and remaining follow-ups

1. **CLOSED in the baseline integration follow-up:** `2983373` is a parent of the merge commit that
   carries this record. Its unchecked field scenarios remain open evidence; integration is not field
   acceptance.
2. **CLOSED as a Gate 1 classification, not as a production fix:** current source still contains
   `style="background:${t.color ?? "#8b8f96"}"` inside `topsPanel.ts`'s row `innerHTML`; `tops.color`
   remains writable as text through the Inspector, and no maintained frontend acceptance test
   exercises the DOM-parser boundary. `0d5389e` is therefore an `ACCEPTED-CANDIDATE` for Gate 2.
   Port only the narrow safe-DOM construction after a current observable regression fails. Do not
   cherry-pick the whole commit: its `REVIEW.md`, progress narrative and null-CSP premise are stale,
   and the old browser transcript is evidence history rather than an executable gate.
3. Do not revive `18da8b0`, `82e56c9`, `d1f0c1e` or `fb01bc0` as whole commits.
4. Source integration is no longer the Gate 1 blocker. G1-I005 must re-measure the field-evidence
   baseline after this merge, and live domain adjudication remains separate work.

## Initial-census classification totals

- `PATCH-EQUIVALENT`: `53`.
- `ACCEPTED-CANDIDATE`: `9`.
- `SUPERSEDED`: `2`.
- `REJECTED`: `2`.
- `UNRESOLVED`: `0`.
- Total distinct patch records: `66`.
