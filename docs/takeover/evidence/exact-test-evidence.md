# Gate 1 exact executable-test evidence

This receipt closes only the executable-test-resolution limb of the Gate 1 evidence audit. It does
not close Gate 1, convert an automated assertion into field evidence, or claim that a named test
proves more than the sentence it pins.

## Scope and result

- The current source tree exposes 1,073 discoverable Rust and JavaScript/TypeScript tests: 1,037 in
  the default catalog and 36 marked ignored.
- The 931-row takeover ledger contains 201 rows that claim automated proof: 77 `CORRECTNESS`, 120
  `CHARACTERIZATION`, and 4 `OPTIONAL-PACKAGE-IGNORED`.
- `docs/takeover/test-evidence.csv` resolves those 201 claimed rows to 251 exact
  `test_path::test_name` references: 106 correctness references, 141 characterization references,
  and 4 optional-package ignored references.
- Every exact reference exists in the current source catalog. Ordinary correctness and
  characterization evidence resolves only to default tests; optional-package evidence resolves
  only to tests that are actually ignored.
- Duplicate evidence rows for the same requirement, class, path, and test name are rejected.
- A test may support more than one requirement only where the per-domain receipt records the shared
  cross-cutting contract. Reuse does not increase the number of proved requirements.

The executable catalog and map are enforced by `node tools/takeover-ledger.mjs --check`. The focused
tool suite names the failure each validation catches, including missing maps, nonexistent test names,
class mismatches, ignored/default-gate mismatches, duplicate rows, and stale evidence retained after
a proof is downgraded.

## Claims downgraded during exact resolution

Twelve prior classifications did not survive exact-test resolution:

| Requirement | Previous class | Current class | Why the previous claim did not qualify |
|---|---:|---:|---|
| `SB-CUT-018` | `CHARACTERIZATION` | `MISSING` | Source inventory did not execute a registry contract. |
| `SB-CUT-028` | `CHARACTERIZATION` | `MISSING` | Manifest parity could pass while a prohibited bare `SW` identity remained. |
| `SB-CUT-043` | `CHARACTERIZATION` | `MISSING` | No executable case distinguishes changing gross thickness from changing interpreted pay. |
| `SB-CUT-046` | `CHARACTERIZATION` | `MISSING` | Helper inspection did not assert the recorded output contract. |
| `SB-MIN-034` | `CHARACTERIZATION` | `MISSING` | No executable test reaches the once-only equality re-solve path. |
| `SB-MIN-036` | `CHARACTERIZATION` | `MISSING` | Generic output parity does not prove the complete positive and negative output inventory. |
| `SB-MLA-018` | `CHARACTERIZATION` | `MISSING` | Comments, strings, and a visible control do not execute phase-specific cancellation behavior. |
| `SB-MLA-034` | `CORRECTNESS` | `MISSING` | Existing transform tests do not assert both enabled and disabled announcements on the run surface. |
| `SB-MLA-058` | `CORRECTNESS` | `CHARACTERIZATION` | The governance test characterizes the shipped Tier-C register; it does not independently prove every route. |
| `SB-TBD-007` | `CHARACTERIZATION` | `MISSING` | No executable test submits out-of-model geometry and observes the derived clamp. |
| `SB-TBD-013` | `CHARACTERIZATION` | `MISSING` | Source inspection finds two range authorities, but no test compares their behavior. |
| `SB-TBD-066` | `CHARACTERIZATION` | `MISSING` | No executable test proves that withdrawn defaults still enter a fresh run. |

These are evidence corrections only. No production behavior, refusal, parameter, or manual checkbox
was changed.

## Boundary that remains open

Exact resolution proves that a referenced test exists and has the declared default/ignored state.
It does not mechanically prove that its assertions semantically cover the entire requirement. That
judgment remains explicit in each domain receipt and was conservatively downgraded wherever the
whole-contract match could not be defended.

Gate 1 therefore remains open for its final seven-criterion audit, the explicit SB-GEO scope
boundary, Jauhar's first-pilot capability-manifest approval, and a fresh full repository gate.
