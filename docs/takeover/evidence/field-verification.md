# Gate 1 manual and field-verification baseline

Measured on 2026-08-10 from the committed GitHub-master baseline `02b59ea`. The generated
capability matrix passed `node tools/generate-verification-matrix.mjs --check` before these counts
were recorded.

## Evidence boundary

- Manual scenario source: [`REVIEW.md`](../../../REVIEW.md).
- Capability definitions: [`verification/capabilities.json`](../../../verification/capabilities.json).
- Generated mapping: [`docs/VERIFICATION_MATRIX.md`](../../VERIFICATION_MATRIX.md).
- A checked scenario means that a human recorded exercising that scenario against real well data.
- Automated tests, a green repository gate and desktop-harness evidence do **not** close an
  unchecked manual scenario.
- These counts describe recorded evidence. They do not establish that an unexercised capability is
  absent, that an exercised capability is scientifically correct, or that the product is pilot-ready.

## Scenario counts

| State | Scenarios |
|---|---:|
| Checked | 78 |
| Unchecked | 1,401 |
| Total | 1,479 |
| Checked ratio | 5.3% |

The accepted GitHub-master increment added nine unchecked native-grid scenarios relative to the
earlier 1,470-scenario receipt. Adding an unchecked scenario expands the evidence obligation; it is
not evidence that the behavior has been field-verified.

## Capability matrix

- Total mapped capabilities: `54`.
- Capabilities with at least one checked scenario: `14 / 54`.
- Fully exercised capabilities: `1 / 54`.

| Generated state | Capabilities |
|---|---:|
| `Exercised` | 1 |
| `Partially exercised` | 13 |
| `Not exercised` | 38 |
| `Not recorded` | 1 |
| `Not listed` | 1 |

The generated matrix is the row-level record; this report does not duplicate all 54 rows.

## Capabilities with recorded exercise

- Fully exercised: `curve-editing` (`5 / 5`).
- Partially exercised: `delimited-intake` (`3 / 27`), `saturation` (`2 / 97`),
  `electrofacies` (`2 / 26`), `machine-learning` (`7 / 189`), `monte-carlo` (`2 / 14`),
  `well-scope` (`3 / 9`), `log-view` (`5 / 37`), `histogram` (`5 / 22`), `crossplot`
  (`6 / 13`), `chart-overlays` (`16 / 53`), `report` (`6 / 53`), `project-lifecycle`
  (`3 / 24`) and `themes-language-accessibility` (`2 / 52`).

The scenario fractions can overlap across capabilities because one review scenario may be relevant
to more than one mapped capability. They must not be summed into an independent scenario total.

## Capabilities not fully exercised

`53 / 54` capabilities are not fully exercised in the recorded matrix: 13 are partial, 38 have
mapped scenarios but no checked exercise, one has a matching review section with no recorded
scenario, and one is deliberately not listed in the review map. A zero in this matrix means no
qualifying checked evidence was found; it does not prove that the capability cannot run.

## Mapping gaps

- `formation-temperature` is `Not recorded`: its capability pattern matches one `REVIEW.md`
  section, but that section contains no checkbox scenario to exercise.
- `thomas-stieber` is `Not listed`: `verification/capabilities.json` explicitly marks it that way,
  so there is no mapped review section or scenario.

No scenario was invented to close either gap. Defining or adding those scenarios is separate work.

## Gate 4 consequence

Pilot field evidence remains `OPEN`. Gate 4 cannot close until Jauhar defines and confirms the
representative pilot workflow and its required real-data scenarios are actually exercised and
recorded. Neither the 946-test automated gate nor the single fully exercised capability substitutes
for that pilot decision and evidence.
