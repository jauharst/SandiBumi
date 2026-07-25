# SandiBumi long-term prompts — maintain, debug, expand

Reusable prompts for running SandiBumi with Claude Code **after** the build-out phase: keeping it
healthy, fixing what the field reports, and growing it without the codebase turning into something
nobody can safely change.

Three modes. Each is a copy-paste block. Pick the one that matches what you actually want today —
mixing them is how a "quick fix" becomes a four-hour refactor.

| You want to… | Use | Typical length |
|---|---|---|
| Fix something that is wrong / crashing / giving a bad number | **Mode B — DEBUG** | one session |
| Add a method, panel, import format, or output | **Mode A — EXPAND** | one to three sessions |
| Burn down the backlog, bump dependencies, keep the docs true | **Mode C — MAINTAIN** | one session per increment |

**Related prompts, different jobs:** `engineering_review_prompt.md` *finds* problems across the
whole app (sweep tiers F1–F5). `qc_audit_prompt_template.md` audits **one tool** end to end. This
file is what you run to *do the work* those two produce, plus the everyday running of the app.

---

## 0. The rule that outranks everything else

**A finding written in a document is a hypothesis, not a work order.**

Everything in `docs/review_sweep/*`, `AUDIT-*.md` and `ROADMAP.md §4b` was true when written and
may not be true now. Three ways a finding goes stale, all three seen for real in rounds R24–R29:

1. **Already closed** — an earlier round fixed it as a side effect (F5 #14, closed by R15).
2. **Real, but the diagnosis is wrong** — the symptom exists, the stated cause does not, so the
   proposed fix fixes nothing (F5 #10 / R28: the report blamed a race between two async calls, but
   the Tauri command is synchronous and cannot race; the actual cause was stale rows on screen).
3. **Real, but the code moved on** — the fix as written is no longer sufficient because the file
   grew a new mechanism since the sweep (F5 #16 / R29: an async editor mount appeared afterwards,
   so the proposed fix left a second hole open).

So: **never open with "implement finding N."** Always open with "**is finding N still open, and is
its stated cause the real one?**" That single change of wording is worth more than any other
instruction in this file.

Corollary: where a finding carries both a reporter's **Fix** and a **Verifier correction**, the
Verifier correction wins. It was written by someone who tried to disprove the finding.

---

## Mode A — EXPAND (add a capability)

```
Add {{WHAT}} to SandiBumi at D:\XX. Arshilla.

## Step 1 — Find the cheapest shape before writing any code

SandiBumi has machinery that makes some additions nearly free and others expensive. Before
designing anything, decide which shape this is, and say so:

- A petrophysics calculation -> a Rust fn + a manifest entry in modules.rs. The parameter dialog
  auto-generates. WRITE NO UI CODE. This is by far the cheapest shape; prefer it whenever the
  feature can be expressed as curves in -> curves out.
- A heavy solver -> its own .rs file, referenced from modules.rs::list_modules / run_module
  (precedent: multimin2.rs, satheight.rs). Still no UI code.
- A new way of LOOKING at existing data -> a panel in src/ui/, registered in workspace.ts.
  This is the expensive shape. Check first whether an existing panel plus a new option gets
  there instead.
- A new file format -> a parser in src-tauri/, plus the Python-subprocess pattern if it needs a
  library (dlis.rs is the precedent). Never embed an interpreter; never let a missing optional
  dependency stop the app from launching.

If the request only fits the expensive shape, say that explicitly and give the cheaper
approximation as an alternative before starting.

## Step 2 — Read the neighbours, then restate

Read the two or three closest existing implementations of the shape you picked, and CLAUDE.md.
Then restate, in your own words and BEFORE writing code:
  (a) what the feature does, in petrophysical terms, including what it does when the inputs are
      bad (all-NaN curve, missing curve, zone with no samples, one well of 200 failing);
  (b) which existing file each piece will live in, and why there;
  (c) what will NOT be built, so scope is visible up front.
Wait for my go-ahead on that restatement if the feature is more than a single module.

## Step 3 — Physics defaults must be sourced

Any constant, endpoint, cutoff or default parameter must come from a documented source: the
specs in docs/, the reference-suite exports, the chartbook, or my own studies. Cite the source
in a code comment. If you cannot source a number, do not invent a plausible one -- say the
number is missing and ask. A wrong constant that looks reasonable is the worst possible
outcome, because it will silently produce wrong reserves for years.

## Step 4 — Build it in increments

One increment = one coherent, shippable step. Implement -> verify -> record -> commit, then
propose the next. Do not batch three increments into one commit.

## Step 5 — Verify (see the shared gate table below)

## Step 6 — Record and commit (see the shared closing ritual below)
```

**Why Step 1 is the whole game.** The module manifest is the reason SandiBumi has ~40 petrophysics
methods and not ~40 hand-built dialogs. Every time a new capability gets forced into that shape,
the app grows without the UI growing. Every time one becomes a bespoke panel instead, the
maintenance surface grows permanently. Ask for the cheap shape first, every time.

---

## Mode B — DEBUG (something is wrong)

```
Something is wrong in SandiBumi at D:\XX. Arshilla.

Symptom: {{WHAT I SAW}}
Where: {{panel / module / well / zone}}
What I expected instead: {{...}}
Reproduce: {{steps, or "not sure"}}

## Step 1 — Do not fix anything yet

Reproduce or locate the fault first. State plainly which of these you achieved:
  (a) reproduced it,
  (b) found code that would produce exactly this symptom but could not run it,
  (c) neither -- in which case say so and ask for more detail rather than guessing.

Never patch a symptom you have not located. A fix aimed at a guessed cause usually leaves the
real fault in place AND adds a second thing to maintain.

## Step 2 — Check whether it is already known

Grep REVIEW.md (look for [x] marks -- those are MY field-verified failures), ROADMAP.md,
AUDIT-*.md, docs/review_sweep/*.md and docs/playbook_build_progress.md for the panel, module
and curve names involved. If it is a known deferral, say so and ask whether to promote it
rather than silently re-deciding something I already decided.

## Step 3 — Classify the fault, because the class picks the proof

  - WRONG NUMBER (physics, units, cutoff, zone bounds)     -> the most serious class here
  - WRONG DATA SHOWN (right value, wrong well/zone/curve)  -> the R26/R28 family
  - PRESENTED-AS-CLEAN (a degraded or failed result shown as a success) -> CARDINAL RULE, fix now
  - CRASH / PANIC / HANG
  - LEAK or lifecycle (grows with use, no wrong output)
  - COSMETIC

Data-honesty faults are never deferred. This app's cardinal rule: a degraded or failed result
must never be presented as a clean one. A silent NaN, a swallowed error, a partial batch
reported as complete, or a zero shown where a computation failed -- all of these are fix-now
regardless of how small the code change looks.

## Step 4 — Fix the cause, not the neighbourhood

Change the smallest thing that removes the cause. If you notice adjacent problems while in
there, DO NOT widen the fix -- record them separately (a task chip, or a ROADMAP line) and say
you did. A fix that touches four files is four times harder for me to field-verify.

## Step 5 — Prove it, with an instrument that can actually see this fault class

See the shared gate table below. Type-checking cannot see a wrong number, a wrong well, or a
leak. Name the instrument that CAN, and use it.

## Step 6 — Record and commit (see the shared closing ritual below)
```

**The classification step earns its keep** because it silently selects the proof. "The Tops pane
showed another well's depths" and "the equation editor is slow after a while" both look like bugs,
but one is proven by tracing which well id reaches the plot and the other by counting what stays in
memory across open/close cycles. Skipping the classification is how a fix gets shipped with a green
`tsc` and no actual evidence.

---

## Mode C — MAINTAIN (keep it healthy)

```
Maintenance increment for SandiBumi at D:\XX. Arshilla.

Source of work: {{ROADMAP §4b | docs/review_sweep/{{TIER}}.md | a [x] in REVIEW.md | dependency
bump | doc drift}}

## Step 1 — Pick ONE item and prove it is still open

Apply §0 of docs/maintenance_scaling_prompt.md: the finding is a hypothesis. Read the live code
before reading the proposed fix. Report which of the three staleness modes applies -- including
"none, it is exactly as described" -- before writing a line of code. If it is already closed,
say so, strike it from the source document, and pick the next item rather than inventing work.

Prefer items that can be proven WITHOUT a running app (pure logic, lifecycle, data plumbing)
when the desktop app or browser is unavailable. Say which constraint drove the pick.

## Step 2 — Read the Verifier correction, not the reporter's Fix

Where both exist, the Verifier correction wins.

## Step 3 — Fix, verify, record, commit (shared sections below), then STOP

One item per increment. Propose the next item and wait. Do not chain.

## Step 4 — Keep the documents true

Any increment that changes behaviour must also update whichever of these it invalidates:
CLAUDE.md (conventions and current state), ROADMAP.md (the backlog), REVIEW.md (a new numbered
Round entry with a Try: line), docs/playbook_build_progress.md (the chain record).
A document that has drifted out of date is worse than no document, because it is trusted.
```

**Health checks worth running periodically**, none of which need a new finding to justify:

- `npm run build` and note the main `index` bundle against the baseline in `engineering_review_prompt.md`
  §2 F4 — a jump means something heavy stopped being lazily loaded.
- `npm audit`, assessed as a *desktop Tauri* threat model, not as a public web server.
- `cargo test` (via the vcvars line below) — the pipeline tests are the closest thing to a
  regression net for the physics.
- Grep for `[x]` in REVIEW.md — those are field-verified failures and they outrank every sweep item.

---

## Shared: the verification gate table

**Pick the gate that can see the fault class.** The commonest mistake is a green `tsc` presented as
proof of something `tsc` structurally cannot observe.

| What changed | Gate | What it can NOT prove |
|---|---|---|
| Any TypeScript | `npm run build` (= `tsc` + `vite build`) | anything about runtime *values* or *lifetime* |
| Any Rust | `cargo check`, then `cargo test` | that the physics is right, only that it compiles/passes existing tests |
| A physics constant or formula | a `cargo test` case with a hand-worked expected value, plus a cited source | nothing else can prove this — a review cannot |
| Wrong-data-shown / plumbing | a headless Node harness in the scratchpad that models the lifecycle and encodes the **consumer's** acceptance rule | that the real UI wires it the same way — read the consumer to confirm |
| A leak / lifecycle | a harness counting what stays rooted across N cycles, plus a codebase invariant (e.g. "every X construction site has a matching destroy") | — |
| Anything visible on screen | the running app | — |

**Two honest-abstention rules**, both of which have applied for real:

- **If the change cannot be exercised by the instrument, say so and skip it** rather than running it
  for appearances. Starting a Vite dev server to "verify" a panel whose data arrives over Tauri IPC
  proves nothing — the panel just renders its error state.
- **If no test surface exists, build a throwaway harness in the scratchpad** rather than declaring it
  unverifiable. A harness that models the lifecycle and counts the leak is real evidence; it is not
  as good as the running app, and the report should say which one it was.

**Rust compile commands on the reference machine** must go through the pinned toolset (MSVC 14.50 is
broken there — missing `clui.dll`):

```bash
cmd.exe /c "call \"C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat\" -vcvars_ver=14.29 && cd /d \"D:\XX. Arshilla\src-tauri\" && cargo test"
```

Run scratchpad `cargo`/`.bat` invocations through **PowerShell**, not Git Bash — Bash mangles the
`cmd /c` quoting.

---

## Shared: the closing ritual (every increment, no exceptions)

1. **REVIEW.md** — prepend a new numbered `Round` entry, newest first. It must contain a **`Try:`**
   line: the exact clicks that would expose the fault if it came back. Written for me at a
   workstation with real wells, not for someone reading the diff.
2. **docs/playbook_build_progress.md** — extend the chain record; backfill the previous round's
   commit hash.
3. **Commit** — stage **specific files only**. `git add -A` is forbidden; it has swept in unrelated
   untracked work before. Message: plain descriptive, **no embedded double quotes** (PowerShell 5.1
   quoting), ending with the Co-Authored-By line.
4. **Never commit `docs/sandibumi_dev_playbook.md`** — that one is mine.
5. **Report**, leading with the outcome in petrophysical terms, then what was proven and how, then
   a proposed next increment. I reply "go ahead" to accept; anything else redirects.
6. Never force-kill `npm run tauri dev` — an unclean kill mid-write corrupts the DuckDB WAL. Plain
   `npm run dev` (Vite only) is safe to kill; stop it afterwards so port 1420 is free.

---

## Scaling — the two different questions

### Scaling the software (more wells, more data)

The bottlenecks found so far, so they are not re-derived:

- **Writes, not compute.** The field-scale bottleneck was an index on `computed_curves`, not
  parallelism. DuckDB is single-writer by design (`Mutex<Connection>`) — that is fundamental, not a
  defect to fix. Wins come from *fewer, bigger* writes (delete-then-append one well's whole output),
  not from more threads.
- **The write-discipline contract.** `computed_curves` is deliberately PK-less. Uniqueness is upheld
  by how it is written. **Never add an upsert/ON CONFLICT path that assumes a primary key.**
- **Anything that runs per-well across the field should be able to stay in memory.** Monte Carlo does
  this deliberately — it returns vectors and writes nothing. Copy that pattern for new batch work
  before reaching for the database.
- **The GPU is render-only.** It is not a compute resource here; do not propose moving solvers to it.

### Scaling the collaboration (a repo bigger than one conversation)

This matters more than it sounds, and it is the reason for the documents:

- **Claude's own memory is machine-local and does not survive.** Everything durable must be in the
  repo: `CLAUDE.md`, `docs/`, `ROADMAP.md`, `REVIEW.md`, `AUDIT-*.md`. When a session and a document
  disagree, **the document wins** — and if the document is the one that is wrong, fixing it is part
  of the increment.
- **Write down conventions the moment they are decided**, especially the ones that look like bugs to
  a newcomer (module runs deliberately not undoable; PK-less table; backend not enforcing well-group
  scoping). Every undocumented deliberate choice will eventually be "fixed" by someone helpful.
- **One increment per commit** is what makes a 2000-well app reviewable by one petrophysicist. A
  commit you cannot field-verify in one sitting is too big.
- **Use the heavy instruments for finding, not fixing.** `/code-review ultra` (you trigger it; Claude
  cannot) reviews a *diff*. The F-tier sweeps cover *whole-app properties* a diff review structurally
  cannot see. Both produce hypotheses, which then come back through Mode C one at a time.

---

## Notes (outside the prompts)

- **The modes are deliberately separate.** "While you're in there, could you also…" is how a
  one-file fix becomes a five-file change I cannot field-verify. If a debug session uncovers an
  expansion, it becomes a separate Mode A session.
- **"Prove it is still open" costs ten minutes and has paid for itself three times** in the R-chain.
  Keep §0 at the top of this file.
- **The `Try:` line in REVIEW.md is the highest-leverage habit in the whole process.** It is what
  lets field verification happen without reading code, which is what keeps me able to check the work
  at all.
