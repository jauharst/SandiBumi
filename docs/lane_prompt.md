# The lane prompt — one agent, one branch, one scope

**Use this for every code lane in `ROADMAP.md` §B0.** Fill in two lines, paste the block, and do
nothing else until the agent stops. The queue, the branch names and the STOP line live in §B0; this
file is only the prompt and the check that follows it.

Written 2026-08-08 after three lanes proved the method (PRs #21, #25, #29). The reasoning behind the
boundaries is `docs/record_parallel_lanes.md`; read that before changing anything here. **Every rule
below exists because its absence cost something**, and the two most expensive are the last two in
the HALT list.

---

## 1. Before you paste

```sh
cd "/d/XX. SandiBumi-check"
git fetch origin
git checkout -b <BRANCH> origin/master
git branch --show-current
```

`<BRANCH>` comes from §B0's table. The last line must print it back. If it prints `master`, the
checkout did not happen — read the error rather than continuing.

---

## 2. The prompt

Change only the two `SCOPE` and `SPEC` lines. Everything else is fixed.

```
Continue in this worktree, on the branch already checked out.

SCOPE: the open P0 requirements of <DOMAIN> in docs/PRD_v2/91_REQUIREMENTS_INDEX.md.
SPEC:  docs/PRD_v2/<CHAPTER>.md — implement what it specifies. Do not re-derive it and do
       not re-litigate it. The requirements are already written with rationale, as-built
       status and named tests.

READ FIRST, IN THIS ORDER
1. AGENTS.md, then CLAUDE.md it points to. Rules 1-11 are binding, in particular:
   missing data is f32::NAN and never Option<f32>; arrays cross IPC as bytemuck bytes and
   never as JSON; the frontend never sends SQL for writes; Python is a SUBPROCESS and never
   embedded; data edits are undoable.
2. The docs/record_*.md covering the files you will touch. It states what previous
   increments settled and why, which is the part that says which alternative is wrong.
3. Your SPEC chapter: §4 requirements, §5 parameters, §6 tests, §8 traceability.

GOAL, AND THE ONLY DEFINITION OF DONE
Every requirement in SCOPE either implemented with its named test, or reported BLOCKED with
the reason. The full gate green. One commit per requirement. Then stop and report.

You are NOT done if the gate is red. You are NOT done if a requirement is silently skipped.
Reporting a blocker is a successful outcome; guessing is not.

HALT IMMEDIATELY — do not work around any of these, do not continue, report and stop

1. A requirement needs a petrophysical value the chapter does not cite. A parameter is
   CITED or it ships ABSENT (CONTRACT.md §2). Never pick a plausible number, never round
   one, never carry one over from a neighbouring vendor, never take one from your own
   training. A named gap is the correct output. This is the single most important rule here:
   a wrong endpoint computes, plots, and ships into a client report without ever failing
   loudly, and no test will catch it.
2. The work requires editing a file on the NEVER TOUCH list below.
3. Your change breaks a test in a file you do not own. Report which test and why. Do NOT
   edit that file to make it pass.
4. The gate is red and the fix is outside your own files.
5. You believe the specification is wrong. Say so and stop. Do not implement your reading
   of what it should have said.

NEVER, UNDER ANY CIRCUMSTANCE

- Never mark a failing test #[ignore] to get a green gate. #[ignore] is for a test whose
  subject needs an optional package, nothing else. A gate made green by silencing a test is
  a lie told to every future session.
- Never delete, weaken, or loosen an assertion in an existing test. If an existing test is
  genuinely wrong, HALT and say why.
- Never delete or narrow a refusal, a guard, or a validation that already ships, to make
  your own work pass.
- Never add ON CONFLICT, upsert, or any duplicate-tolerant write path to computed_curves.
  That table is deliberately PK-less and uniqueness is upheld by the write discipline
  (CLAUDE.md).
- Never widen SCOPE. If you finish early, STOP. Do not start the next domain, do not
  refactor something you noticed, do not tidy unrelated code. An improvement nobody asked
  for is an unreviewed change.
- Never switch, create or check out another branch, including master.
- Never push, never merge, never open a pull request.
- Never git add docs/research_2026-08/ if it is present. It is an untracked local copy and
  keeping it out of git is a decision already made.

NEVER TOUCH THESE FILES
  db.rs — the DuckDB write discipline is never delegated
  equations.rs, multimin.rs, multimin2.rs, ssc.rs, lrlc.rs, satheight.rs, thomeer.rs,
    hfu.rs, montecarlo.rs, distribution.rs, petrography.rs — a wrong answer in any of these
    compiles, plots and reaches a client report with no gate to catch it
  ml.rs, mlDialog.ts — another session's domain
  src/ui/chartOverlays.ts — GENERATED; regenerate with tools/chartdig, never hand-edit
  docs/PRD_v2/** — the specification is not yours to amend
  ROADMAP.md, CLAUDE.md, AGENTS.md, .cursorrules
  THIRD-PARTY-LICENSES.md — generated by tools/gen-third-party-licenses.mjs

ALWAYS
- parsers::read_text_file for every text import. Never read_to_string, never
  BufReader<File> — one stray byte must not reject a whole delivery.
- sys.stdin.buffer in every Python runner. Never sys.stdin: a piped child decodes text
  stdin with the Windows ANSI codepage while serde_json emits UTF-8, so any non-ASCII
  character arrives as mojibake.
- No client, field, block, basin, operator, well or project name anywhere in code, tests,
  test data or comments. Name the physical condition instead.

TESTS
One named test per contract, and the NAME IS THE SENTENCE IT PINS — for example
a_declared_null_is_honoured_even_when_it_is_not_minus_999. Where a lazier implementation
would also pass, pin the contract from BOTH sides so neither alone would satisfy it. Cite
the source of every expected value; a test whose expected value has no source is the
uncited-parameter failure wearing a different hat. A test whose subject needs an optional
package is #[ignore]d so the green gate can never depend on it.

Do not pad. A test that only restates the code it calls proves nothing and still has to be
maintained.

REVIEW.md
One entry per increment, at the top of the file, in the existing style. You are the only
lane writing that file — do not reformat or reorder what is already there.

VERIFY, IN THIS ORDER, BEFORE YOU REPORT
  npx tsc --noEmit
  cd src-tauri && cargo check
  powershell -ExecutionPolicy Bypass -File tools\check.ps1

The gate was green on 2026-08-08 at 828 passed / 0 failed / 36 ignored. It must still be
green, with your new tests ADDED to that total. A total that has not grown means the
requirements were implemented and nothing pins them.

COMMIT AND STOP
Committing is REQUIRED. Pushing is NOT. One commit per requirement, the message naming the
requirement id and what changed. Leave the work committed and unpushed.

REPORT IN EXACTLY THIS SHAPE, so it can be read in one minute:
  - one line per requirement: id, DONE or BLOCKED, and what you did or what blocked it
  - the gate numbers: passed / failed / ignored
  - files touched, as a list
  - anything you stopped on for want of a cited source
  - anything you noticed and deliberately did NOT do
```

---

## 3. The check, before you push — five minutes, four commands

```sh
cd "/d/XX. SandiBumi-check"
git status --short
git log --oneline origin/master..HEAD
git diff --stat origin/master...HEAD
```

| Check | Pass | Fail means |
|---|---|---|
| `git status --short` | clean, or only untracked files you recognise | the agent left work uncommitted — this has happened; commit it before pushing |
| `git log …origin/master..HEAD` | one commit per requirement | **empty means nothing was committed.** Do not push an empty branch |
| `git diff --stat` | no file from NEVER TOUCH | the boundary leaked. Do not merge. Untangle first |
| the agent's report | `0 failed`, and the passed total **higher** than last lane's | red is not done, whatever it says. Same total = no new tests |

All four pass:

```sh
git push -u origin <BRANCH>
```

Then on GitHub: **Create pull request** → read the *Files changed* tab → **Create a merge
commit** → **Delete branch**. Three lines of body: which ids, which files, the gate numbers.

---

## 5. Which batch to run, and what to paste

**`docs/P0_TRACKER.md`.** It holds all eight batches with their literal `SCOPE`/`SPEC` lines, the
progress ledger, and the three paragraphs to append to §2's prompt for a batch.

Deliberately not duplicated here: this file is the prompt and the check, the tracker is the
sequence and the fill values. One home each.

## 6. The step no agent can do

**Field-verify each lane against real wells before starting the next**, and write the
`REVIEW.md` mark yourself. PR #29 changed how LAS files are read — null recognition, index
units, export provenance — and no gate on earth can tell you whether a real delivery still
reads correctly. `[x]` means you clicked through it and it works; `[ ]` means not yet checked.

Ten lanes is ten PRs to read and ten field checks. **The queue is not the constraint.**

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
