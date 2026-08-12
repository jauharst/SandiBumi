# SB-GEO next-version deferral receipt

Date: 2026-08-12
Branch: `codex/g1-sb-geo-deferral`
Decision authority: `DEC-011`
Prior live-adjudication plan: `e5e86b8` / `docs/superpowers/plans/2026-08-11-sb-geo-live-adjudication.md`
Accepted implementation anchor: `b332026cb498c105f36eade0bf7899bc0c1309f0` (reachable)
Serial parent: `b2d74030d32b12a53c8800a5957702044a76e400`
Origin/merge-base anchor: `29833735816d9e5be954afafd9ceb71fd856e3f0`

## Exact decision

Jauhar explicitly directed that geomechanics and PPFG be held for the next product version while the current paid offline Windows pilot focuses on open-hole petrophysics. `docs/takeover/DECISIONS.md` records that direction as DEC-011. It blocks no current-pilot capability and is revisited only after the petrophysics pilot scope is accepted.

This receipt applies that product timing decision to all 52 contiguous SB-GEO rows, SB-GEO-001 through SB-GEO-052. It is not a live source adjudication and does not execute the previously approved 52-row GEO evidence plan.

## What changes

- `release_disposition` changes from `UNDECIDED` to `DEFERRED` for all 52 SB-GEO rows.
- `risk_class` changes from `UNCLASSIFIED` to `LATER`, because this increment classifies product timing only.
- `dependencies`, `blocking_decision`, and `next_action` name DEC-011 and the bounded next-version live-adjudication action.
- Aggregate release dispositions change from 584 pilot blockers / 198 undecided / 149 deferred to 584 pilot blockers / 146 undecided / 201 deferred.

## What deliberately does not change

- Every `as_built_status` remains `UNADJUDICATED`.
- Every `test_class` remains `MISSING-OR-UNCLASSIFIED`.
- Every `commit_state` remains `UNVERIFIED` and every `last_reverified` remains blank.
- `implementation_paths`, `expected_value_source`, and `manual_evidence` remain blank because this receipt did not inspect and classify their row-level evidence.
- All source-owned requirement IDs, chapter paths, titles, priorities, historical statuses, and owned-test mappings remain byte-stable.
- No GEO method, equation, parameter, tolerance, unit limit, breakpoint, cutoff, endpoint, vendor table, test, UI surface, production file, PRD text, or manual evidence is added or changed.

## Row set and source custody

- Count: 52 exact rows.
- IDs: SB-GEO-001 through SB-GEO-052 with no gap or duplicate.
- Priority mix retained: 33 P0, 17 P1, 2 P2.
- Historical chapter state retained: 50 ABSENT, 2 PARTIAL.
- Chapter test intentions retained: 73, SB-GEO-T01 through SB-GEO-T73, including the original shared ownership mappings.
- Frozen six source-owned ledger columns SHA-256: `73089ac09c833c2cc6563161310ee6a388bf335e7ee49ffbda002a632496f198`.
- Deliberately absent/non-adoptable GEO parameters remain untouched. This deferral never authorizes a water density, pressure gradient, Biot alpha, Eaton exponent, Bowers coefficient, Matthews-Kelly tolerance/table, Daines table, Poisson breakpoint, stress ratio, strain, dynamic-to-static transform, failure coefficient, or any neighboring value.

## Gate 1 boundary

The takeover design's Gate 1 exit criteria require every row to distinguish historical chapter status from a reverified as-built status and require all 931 requirements to be accounted for. DEC-011 accounts for the 52 GEO rows' release timing, but it does not reverify their as-built state.

Therefore the post-deferral ledger is truthfully:

- 931 total requirements;
- 879 live-adjudicated requirements;
- 52 SB-GEO requirements explicitly scope-deferred but still as-built `UNADJUDICATED`;
- all 931 rows assigned a release disposition;
- 584 pilot blockers, 146 undecided, and 201 deferred.

The final Gate 1 audit must not call this 931/931 live adjudication. Formal all-931 as-built closure would require either executing the existing SB-GEO documentation-only adjudication plan or an explicit amendment to the Gate 1 exit criterion. DEC-011 by itself is a product-timing decision, not that evidence amendment.

## Manual and field evidence

No manual scenario is marked complete by this receipt. There is no GEO capability row in the generated verification matrix, and the adjacent unconventional capability remains 0/4. Those facts are not promoted into a GEO as-built verdict.

## Next-version action

When DEC-011 is revisited, start from the preserved `2026-08-11-sb-geo-live-adjudication.md` plan, re-anchor it to the then-accepted implementation tree, re-read the complete governing sources, and execute all 52 row-level evidence classifications before any GEO production implementation. Keep every uncited scientific value absent and every protected vendor/raster/binary source non-adoptable.
