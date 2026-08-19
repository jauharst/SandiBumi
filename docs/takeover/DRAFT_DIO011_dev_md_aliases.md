# DRAFT — SB-DIO-011 deviation-survey MD alias registry (sources for sign-off)

**Status: DRAFT, delivered 2026-08-19 under DEC-073 item 6** (corpus-drafted alias registry;
Jauhar adjudicates rows the corpus cannot source). The row's blocker: `DEV_MD_ALIASES`
(`parsers.rs:2878`, values `MD | DEPTH | DEPT | MEASURED_DEPTH`) is the fourth index-bearing
alias list and the only one with no documented source. This draft attributes each shipped
value from the ingested vendor corpus and the chapter's own adjudications, names what remains
analogical, and reports the corpus's negative result. Nothing was invented; every citation
below names its file and evidence tier.

## The corpus's negative result — stated first because it frames everything

**No vendor ships a deviation-survey header-alias vocabulary the registry could adopt.**
IP 2025 loads survey data through PRE-DEFINED position curves (East Distance, North Distance,
TVD, "already defined in Manage Well Header Info → Position, all three within the same curve
set" — `research_2026-07/ip2025_chm_ingest/N_data_io.md` §5.11, T2) and its ASCII loader
mandates the literal name `DEPTH` rather than aliasing (below). So the deviation alias list
is SandiBumi's own wizard vocabulary, adjudicated here — not a vendor table transcribed.

## Per-value attribution

| Value | Source | Tier |
|---|---|---|
| `DEPTH` | IP 2025 ASCII loader: *"The depth curve must be named `DEPTH` in the Curve Name row"* (`load_ascii_data.htm`, via `N_data_io.md` ~:83-87) — the one literal a major vendor MANDATES. Also the head symbol of real Geolog's reference-alias declaration (`alias.alias:14`, T1, as recorded at `21_data-io.md:199-203`). | T2 + T1 |
| `DEPT` | The LAS 2.0 index convention `DEPTH_ALIASES` already relies on — `21_data-io.md:669` records the list and its rationale (the chapter's `parsers.rs` line pointers are stale against current code, where the list now sits at `parsers.rs:858`; the rationale is unchanged). A survey delivered as LAS-adjacent text carries the LAS spelling. | Chapter-adjudicated |
| `MEASURED_DEPTH` | IP 2025's tops fixed format carries the spelled-out column literal *"REAL Measured Depth"* (`N_data_io.md:605`); the underscore form arrives through SandiBumi's own header normalization (`header_matches`). | T2 |
| `MD` | ANALOGICAL — no vendor citation. The chapter's own adjudication of `CORE_DEPTH_ALIASES` (*"admitting `MD` is correct for its context"*, `21_data-io.md:686`) argues from context: a table that is not an LAS file and has no ambient index convention, which a deviation CSV also is. The row is presented for Jauhar's explicit sign-off as the one member resting on analogy rather than a source. | Analogy (sign-off) |

## Candidate additions — presented, not adopted

Real Geolog's reference-depth aliases (`alias.alias:14`, T1): `SCD`, `IDWD`, `DVP1`,
`PDEP_XPT`, `DEPM`, `TDEP` — declared under the file's own `# aliases for references`
section. He may adopt any of these; the draft deliberately does NOT, because the chapter's
own finding is the cautionary tale: `TOPS_DEPTH_ALIASES` admitting `TVD` is recorded as a
DEFECT (`21_data-io.md:690` — a tops table carrying both MD and TVD resolves the wrong one),
and over-inclusion in an index list converts a wrong answer into a silent one. `TVD` is
NOT proposed for the deviation list for the same reason — a survey file routinely carries
an MD column AND a TVD column, and the alias list must never match the latter.

## What follows the signature

Cite the signed source beside the `DEV_MD_ALIASES` declaration; then rewrite T17 to discover
every index-bearing alias list MECHANICALLY (so a fifth list cannot ship uncited — the
current test enumerates three by name and claimed "every") without weakening the TVD
negative controls. SB-DIO-014 separately owns reference semantics and is untouched.
