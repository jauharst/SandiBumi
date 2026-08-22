# Observability — what a user can send when something goes wrong

**2026-08-22.** Pass 2 of the brief *"make it safe to run on a client's machine, and diagnosable
when it is not."* Pass 1 is `SECURITY-REVIEW-2026-08-22.md`.

The question put was narrow and worth restating exactly, because the shape of the answer follows
from it: **if a user reports "it was slow" or "the numbers look wrong", what can they send me, and
is it enough?** With a hard rule attached — *whatever is collected must contain NO client
identifier and NO curve values; diagnostics that leak a delivery are worse than no diagnostics* —
and an explicit instruction not to build a telemetry system.

---

## 1. What existed, measured

| Signal | Where | What it answers |
|---|---|---|
| `db::boot_note` → `boot_report` | status line + process history | One-time events: migration backups, the memory cap, compaction, a ≥10 s open |
| `processLog.ts` `recordProcess` | `documents` table, `history`/`log` | What was done, in order, per well |
| `.corrupt-backup-*` | `src-tauri/` | The WAL recovery already ran |
| `CurveAncestry` | `ancestry.rs` | How a specific number was made — module, version, inputs, parameters, zone scope, depth frame |

Counted rather than estimated. These describe the tree **before** this increment, so each command
is given in the `git show` form that still reproduces it:

- `project.rs` measured **13** opening steps and only **4** reached the user. The other nine went
  to a console a built exe does not have.
  `git show HEAD~1:src-tauri/src/project.rs | grep -c 'eprintln!("\[boot\]'` → 13;
  same with `grep -c 'boot_note('` → 4.
- **Zero** duration measurements anywhere in the run path — for each of `workflow.rs`,
  `modules.rs`, `chain.rs`, `jobs.rs`:
  `git show HEAD~1:src-tauri/src/<file> | grep -c "Instant::now()"` → 0.
- **25** `run_simple_job` call sites, all delegating to `jobs::run_job`
  (`git show HEAD~1:src-tauri/src/lib.rs | grep -c "run_simple_job("` → 25) — one choke point
  covering every long operation.

## 2. The gap, stated precisely

It is not that the diagnosis is missing. **It is that the diagnosis is welded to the data.**

`CurveAncestry` answers "the numbers look wrong" richly and completely — and only from inside a
project file containing the client's curves. `recordProcess` records what was done and names the
well while doing it. So the two best diagnostic instruments in the application are both unsendable
under a confidentiality agreement, which is the same as not having them.

"It was slow" was a different problem: not welded, simply never recorded. Nothing timed a module
run, a chain, an import or an export, so a chain that ran for three hours left no trace of which
step took them.

## 3. What was decided

Put to Jauhar as an explicit choice, because it is the one judgement in the whole pass:

> **Do parameter *values* travel?** (a) Include them — best diagnosis; the report becomes
> commercially sensitive and must say so on its face. (b) Names only — safe to send without
> thinking; often not enough to explain a wrong number.

**His answer: (a).** Without `m`, `n`, `a`, `Rw` and the cut-offs there is usually no way to say
why a number looks wrong, and that is half of what the report is for.

The consequence is accepted rather than hidden: the report carries the client's own calibration,
which is analytical work product. So it states that **on its own face, above everything else** —
in the file, and again in the pane above the Save button. A person about to attach it to an email
reads the caution without having gone looking for it.

## 4. What shipped

One artefact: **Project ▸ Monitor ▸ Diagnostics…**, a pane, writing one plain-text file.

Deliberately not a telemetry system, and each of these is a decision rather than an omission:
nothing is transmitted, nothing is collected in the background, there is no daemon, no phone-home,
no identifier, no new runtime dependency. The user presses a button, **reads what it produced**,
and decides whether it goes anywhere.

| Section | Answers |
|---|---|
| Machine | Is this an 8 GB field laptop or a workstation? Is Python/scipy there? |
| Project shape | Counts only — the denominator for "slow". 60 s over 5 wells and over 2000 are different statements. |
| Opening this project | All 13 steps, so a slow open names the migration responsible. |
| Operations this session | Every run, chain, import, export and render, with its duration, item count and outcome — `ok`, `cancelled` or `FAILED`. |
| Session errors | The panic hook's record — see §5. |
| How these curves were made | One well's full `CurveAncestry`, **including parameter values**. |

### The redaction rule

**A name from the shipped vocabulary travels. A name the user invented is masked.**

Module ids, curve mnemonics, job kinds and error text are ours and are diagnostic. Well names,
field names, the project name and every file path are the client's and are replaced — wells become
`WELL-1`, `WELL-2`, consistently *within one report only*, with the mapping never stored, so two
reports cannot be lined up against each other.

Three properties of the implementation matter more than the rule itself:

- **The mapping is built from the project's OWN well and field list**, not from a pattern. A
  pattern would eventually miss a naming convention nobody anticipated, and a redactor that misses
  once has failed completely — the entire value of the report is that it can be sent.
- **Longest name first.** With `SANDI-1` and `SANDI-10` in one project, replacing the short name
  first leaves `WELL-1` followed by a stray `0`: wrong, and a partial leak of the original.
- **Redaction happens once, at RENDER.** `record_op` stores the label verbatim because the
  Processing panel shows the same label and needs it intact.

An operation's outcome is **three states, not a boolean**. A run the user gave up on is not a
completed one, and a four-minute abandoned chain reported as `ok` would mislead in exactly the case
a support call is most likely to be about. `run_job` decides the state first and records it, rather
than recording whether the worker thread returned.

Pinned from both sides by
`diagnostics::tests::a_report_carries_the_shape_of_the_problem_and_none_of_the_delivery` — a
redactor that blanked the whole file would satisfy "nothing leaks" perfectly and be worthless, so
the test asserts the module name, the duration and the counts SURVIVE as well.

### One measurement, printed and recorded

`diagnostics::boot_step` does the `eprintln!` and the record from a single `t.elapsed()`. A second
call would print one number and record another — too small to matter and exactly the drift this
repo does not accept between two renderings of one measurement. The one step that does not use it
is `init_db_resilient`, whose console line names the project PATH; it takes one measurement and
uses it twice, deliberately, so the path reaches the console and never the report.

## 5. F2, half-closed and said so

`SECURITY-REVIEW-2026-08-22.md` finding F2 — *one panic anywhere makes the project unusable until
restart* — is **not fixed here**. A panic poisons whatever mutex it held and every later
`lock().unwrap()` panics in turn, so the user sees an application that has stopped working with no
first cause visible anywhere.

What landed is the observability half: `diagnostics::install_panic_hook` records where the first
panic happened, so the report can name it. One hook rather than a guard at each of the **182**
`db.0.lock().unwrap()` sites in `lib.rs`
(`git show HEAD~1:src-tauri/src/lib.rs | grep -c 'db\.0\.lock()\.unwrap()'` → 182) — and not merely
because it is smaller. **`.unwrap()` panics before any code at that site could record anything**,
so a per-site guard structurally cannot catch the first one. The hook can, wherever it happens.
Recovering from the poisoning is a separate change and remains open.

## 6. Found by measuring, fixed here

Auditing the pane's contrast returned backgrounds of `rgba(0,0,0,0)`, which turned out not to be a
measurement artefact: **`--panel` was never a token in this stylesheet** — the surface token is
`--bg-panel` — so `background: var(--panel)` produced an invalid declaration that CSS silently
dropped. Four uses, three of them pre-existing (`.module-validity-item`,
`.module-validity-un_evaluable` / `-check_failed`, and `.module-contamination`, the caution card
this pane puts its warning in). All four now name the real token. Measured after the fix, light
and dark: caution title 15.37 / 15.88, caution body 6.10 / 6.53, report text 16.60 / 12.60 — all
above the 4.5:1 floor.

## 7. What this does not do

- It does not transmit, schedule, or collect anything in the background.
- It does not fix F2, only report it.
- It does not make a hostile project safe — that was pass 1's F1, and that is a notice too.
- It carries **no** curve values, and it carries parameter values on purpose. Those are different
  decisions and the report says which is which.
