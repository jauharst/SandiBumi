# DRAFT — SB-ENV-005 the one authoritative correction manifest (content for sign-off)

**Status: DRAFT, delivered 2026-08-19 under DEC-073 item 8** ("engineering proposes the one
authoritative correction manifest"). Per the same ruling the proposed content below goes to
Jauhar for sign-off; nothing is implemented on unsigned content.

## The ruled ground (recorded, not re-asked)

- **OI-4 is ANSWERED.** DEC-031(b), confirmed unchanged by DEC-060: *"the applied-step
  manifest lives in the LOG-SET ARCHIVE, riding with the versioned interpretation output it
  describes; per 20_envcorr-qc.md:2538-2541 that one answer also settles SB-ENV-028's mask
  record and SB-ENV-042's interactive-edit provenance."* The requirement row's "Jauhar
  selects OI-4's single persistence owner" predates that ruling; this draft builds on it
  instead of re-opening it.
- **The per-sample state channel** is the DEC-060(a) one-hot boolean group
  (`<OUT>_FULL/_PARTIAL/_NONE/_REFUSED`), emitted on every run.
- **The bit-exact recovery record** (SB-ENV-037/DEC-035/DEC-061(b)) already ships for the
  conditioning pilot as the `OUT_ORIG` curve — original 32 bits at changed samples, built
  before the flag is consumed, both written or neither.

The manifest is the missing layer ABOVE those: the ordered list of steps actually applied,
retrievable without re-running.

## Proposed manifest (the content DEC-073 delegated)

**Where.** A nullable `applied_steps_json` column on `log_sets`, added LAST (positional
rule; the DBM-031/DIO-007 additive-column precedent). `NULL` = a pre-contract version whose
step history cannot be recovered — preserved as unknown, never backfilled. Written in the
SAME transaction that allocates the log-set version: the manifest and the version it
describes exist atomically or not at all ("one typed atomic manifest").

**Shape.** Versioned JSON, `{"v": 1, "steps": [...]}`, each step:

```
{
  "seq": 1,
  "kind": "module" | "correction" | "mask" | "edit",
  "module": "nphi_env_corr",            // module/equation identity, kind != "edit"
  "params_digest": "<sha256>",           // digest of resolved params incl. zone overrides
  "inputs": ["NPHI@RAW", "FTEMP"],      // resolved input mnemonics with set qualification
  "outcome": {"full": n, "partial": n, "none": n, "refused": n},  // counted from the
                                         // DEC-060 flag group, never re-derived later
  "mask": "BADHOLE" | null,              // rule-11 mask consumed by this step
  "recovery": "GR_DSP_ORIG" | null       // the SB-ENV-037 record curve, where one exists
}
```

- `params_digest` references the run's `params_json` (already on the same row) rather than
  duplicating it; the digest makes "same step re-applied?" decidable without parsing.
- **No correction coefficient, measured input, chain identity or status is invented**: every
  field above is copied from what the run already resolved, at the moment it resolved it.
  A step the runner cannot fully describe writes the fields it has and omits the rest —
  omission is representable (`null`), fabrication is not.

**One vocabulary answers all three questions** (the DEC-031(b) coupling):
a rule-11 mask application is a step of `kind: "mask"` naming the flag curve; an
interactive edit (SB-ENV-042) is a step of `kind: "edit"` naming its recovery record; an
environmental correction is `kind: "correction"` with its outcome counts. SB-ENV-028's mask
record and SB-ENV-042's provenance are therefore entries in THIS manifest, not parallel
structures.

**Retrieval.** A read command returns the manifest for any log-set version; the Processing
history item links to it. Retrieval never re-runs anything — the manifest is the record,
not a recipe re-executed.

## Scope honesty — what lands when

- **On signature**: the column + versioned schema, the atomic writer, the retrieval
  command, and the T10-shaped pin (T10's storage choice is dissolved by DEC-031(b); the
  pin asserts manifest-rides-version atomicity from both sides — a version without a
  manifest on a manifest-era write, and a manifest without its version, both refuse).
- **With their owning rows, not before**: T08's chain arm needs SB-ENV-010/011's
  source-complete correction chains (not pilot-complete — their coefficients are ENV-004
  tier-1 material, held for last per Jauhar); T09 needs the deferred SB-ENV-019/OI-3
  uncertainty machinery. Their manifest entries are already representable in the schema
  above, so neither dependency reshapes the contract — they populate it.
- **SB-ENV-006/007** separately refuse or exclude currently unmanifested outputs — their
  rows, unchanged here.

## Verification (after sign-off)

Pinned from both sides: (1) a module/conditioning run on a manifest-era project writes the
manifest atomically with the version, and the retrieval command returns the steps actually
applied (counted against the run's own flag-group outputs, never re-derived); (2) a
pre-contract version reads back `NULL` and the reader says "unknown", never an empty
step-list (an empty list claims "nothing was applied", which is an answer, not an
absence); (3) an unknown manifest version refuses interpretation while the curves still
read.
