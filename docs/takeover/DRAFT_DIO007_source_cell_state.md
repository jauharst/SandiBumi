# DRAFT — SB-DIO-007 source-cell-state mask contract (content for sign-off)

**Status: DRAFT, delivered 2026-08-19 under DEC-073 item 3.** The SHAPE is already RULED and
is not re-opened here: *"a versioned source-cell-state mask beside the numeric f32 array
(empty vs explicitly-nulled distinguishable), per-deliverable preservation/sidecar/refusal
defined in the draft."* What DEC-073 delegated to this draft — the mask contract's details
and the per-deliverable table — is the content below, and per the same ruling it goes to
Jauhar for sign-off before anything is implemented.

The requirement (21_data-io.md §4.1 SB-DIO-007, witness T11): a delimited reader MUST record
whether a cell was EMPTY (nothing between the delimiters) or EXPLICITLY NULLED (the file's
declared null token), both stay absent in arithmetic, and the distinction survives to the
deliverable. `intake.rs:76` already types source cells as `"number" | "text" | "empty"` —
the read side sees the distinction today; it dies at commit.

## The mask contract (proposed details of the ruled shape)

1. **Encoding.** One byte per sample beside the f32 array: `0 = measured value`,
   `1 = empty`, `2 = explicitly nulled`. A byte, not 2 bits — the mask is small relative to
   the f32 array (¼ its size), byte offsets keep every reader trivial, and six spare values
   remain for states a later version defines (never reused to mean something else).
2. **Versioned.** The mask blob is prefixed with a one-byte version (currently `1`). A
   reader seeing an unknown version REFUSES to interpret states (values still read — the
   mask is auxiliary custody, never a gate on the measurements themselves).
3. **Rule 2 untouched by construction.** Both states 1 and 2 store `f32::NAN` in the
   numeric array; every consumer of the matrix sees NaN and no `Option<f32>` appears
   anywhere. The mask is consulted ONLY by exporters, the Database Inspector and audit
   surfaces.
4. **Where it lives.** For imported curves: a nullable `state_mask BLOB` column added LAST
   on the store (positional-Appender rule; the DBM-031 `depth_datum` migration is the
   precedent). `NULL` mask = pre-contract import — legacy meaning is preserved as unknown,
   never backfilled, exactly the DEC-073 item 5 datum policy. For the typed point stores
   (`aux_data`, core extras): a last-position state column with the same three tokens,
   same NULL-legacy rule.
5. **Source curves only.** A COMPUTED curve's absence belongs to the computation (its
   inputs, its mask, its refusals) — computed outputs carry no source-cell mask, and no
   code path fabricates one.
6. **Custody through edits.** An edit that moves or removes samples moves or removes the
   corresponding mask bytes in the same transaction; a curve write that cannot supply its
   mask writes the whole-curve NULL mask, never a partial one (a mask that misaligns with
   its array is worse than none — it attributes states to the wrong depths).

## Per-deliverable preservation / sidecar / refusal (the table DEC-073 asked for)

| Deliverable | Carriage | Class |
|---|---|---|
| Delimited export (CSV/TXT) | Native round-trip: state 1 exports an EMPTY cell, state 2 exports the file's declared null token. This is T11's witness. | PRESERVED |
| LAS | One NULL token exists in-band, so both states export as the declared NULL; the distinction is written as a `~Other`-section summary naming each affected mnemonic with its empty/nulled counts. Never a second in-band null value — other tools would read it as a measurement. | SIDECAR (in-file) |
| DLIS (when the SB-CORE-015 writer ships) | Representation-code absent values carry no state; same summary pattern as LAS in a comment/parameter object, decided inside the CORE-015 design. | SIDECAR (deferred to CORE-015) |
| Workbook (Excel) | The blank-is-not-zero rule already renders both as an EMPTY cell; a per-sheet audit note is NOT added (a workbook is arithmetic surface, not custody surface). Inspector is the custody surface. | RENDERED-ABSENT |
| Word / PDF / Deck | Dash / blank per the existing absence conventions; no state markup. | RENDERED-ABSENT |
| Database Inspector | Shows the state column / mask class read-only beside the value. | PRESERVED |
| Project copy / Save As / Compact | `engine_copy_to` copies live rows verbatim — masks ride along bit-identically. | PRESERVED |

No deliverable REFUSES on state grounds: absence is representable everywhere, so the
refusal arm of "preservation/sidecar/refusal" is deliberately empty — stated here so its
emptiness is a signed decision rather than an omission.

## Verification (after sign-off)

Pin SB-DIO-T11 exactly as written: a delimited file with `a,,b` and `a,-999.25,b` on
consecutive rows imports with both absent in arithmetic (same NaN, same module behaviour)
and exports distinguishably (empty cell vs null token) — pinned from both sides so an
implementation that collapses either direction fails. Plus: unknown mask version refuses
interpretation while values still read; legacy NULL mask stays NULL through migration and
re-export says so; a mask always matches its array's length or is absent whole.
