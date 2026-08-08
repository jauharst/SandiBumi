# Running a second agent on this repo — worktree lanes

**2026-08-08.** Jauhar put a second assistant on SandiBumi (the Codex desktop app, local,
against a folder on the same machine) while Claude Code held the main tree. This is what that
session settled. It is written for the next person who tries it, and the reasons are the part
worth reading — most of the failure modes here are *silent*, and several are unrecoverable.

The first delivery under it: the seven remaining PRD v2 chapters, 7,355 lines, one PR, zero
files touched outside the agreed list.

---

## 1. A lane is a folder, a branch, and one agent

```
D:\XX. SandiBumi          master or a feature branch   the tree you work in
D:\XX. SandiBumi-sweep    docs/prd-v2-amendment-sweep  one agent, one job
```

`git worktree add -b <branch> <path> origin/master`. The agent is pointed at the folder and can
reach nothing else, which is the entire mechanism: a boundary the tool cannot cross beats a
boundary it is asked to respect.

In the Codex desktop app that means **one source folder per project, and never the main tree
alongside it.** A second source folder pointing at `D:\XX. SandiBumi` would let the lane edit
`master` directly, and the split would stop meaning anything.

**One job per folder.** A docs lane and a code lane on one branch puts code changes into a PR
being read as prose, and code touches `modules.rs`, `lib.rs`'s single `generate_handler!` list,
`ribbon.ts` and `workspace.ts::createComponent` — the four files every capability touches.

---

## 2. What travels through git, and what does not

| | How it reaches a lane |
|---|---|
| Tracked files — all code, all committed docs | git: commit → push → merge → the other branch pulls `master` |
| **Untracked files** | by hand, per worktree, because git does not know they exist |

**Never copy code between worktreets.** A worktree is a checkout, not a copy; a frontend edit in
the main tree is *supposed* to be invisible to a lane.

The one exception is the cross-tool evidence corpus (`docs/research_2026-08/`, 42,936 lines,
cited by thirteen PRD files, deliberately untracked). It needs one `cp -r` into every **docs**
lane. Code lanes do not need it: they implement from the committed chapter.

**A lane must never `git add` it.** `CONTRACT.md` §2.1 forbids transcribing vendor material into
this tree, and the decision to keep the corpus out of git is recorded. The asymmetry decides it:
keeping it local costs ten seconds per worktree and reverses freely, while committing it is
**not cleanly reversible** — git keeps it in history, and getting it out means rewriting history
and breaking every clone. An agent that sees 36 uncommitted files feels pressure to tidy them
into a commit. Say so in the prompt.

---

## 3. File ownership is the mechanism, and it is stated as two lists

Every lane prompt carries an explicit **may edit** list and an explicit **may not, under any
circumstance** list. Not a description of scope — the paths.

The may-not list always includes `ROADMAP.md`, `REVIEW.md`, `CLAUDE.md`, `AGENTS.md`, everything
under `src/` and `src-tauri/`, and any file another lane is holding.

**`REVIEW.md` is the standing conflict magnet.** Every increment appends an entry, so two lanes
guarantee a conflict. They resolve trivially and they never stop coming. **While one lane is
open, only that lane writes `REVIEW.md`.**

Also state the branch: *stay on this branch, do not check out, create or switch to another,
including `master`*. The folder boundary stops a lane reaching another **directory**; it does
nothing about another **branch** in the same folder, and every branch is one click away in the
picker.

---

## 4. A lane never edits the spine — it logs to `_SPINE_PENDING.md`

A chapter lane that finds the cross-cutting documents contradicting the evidence writes an entry
naming the spine file and section, the current claim, the verified source with `file:line`, and
**which direction was wrong**. It does not edit the spine.

This is not deference. Across the batch-2 chapters the tally was **the spine stale four times and
the chapter right; the chapter wrong twice and the source right.** Both directions happen, so each
disagreement needs a judged decision, not an automatic edit.

The mechanical reason is worse than the editorial one. Two agents amending
`04_CORE_REQUIREMENTS.md` in parallel produce a merge git accepts happily and that leaves the
documents contradicting each other — a clean merge and a wrong result, with nothing to signal it.
The same argument bars parallel work on `SB-CORE-006` and `-007` themselves.

It works. The first lane returned two entries, both real, both evidenced, both correct: a stale
Tier-C prohibition in `00_INDEX.md` §0.4, and a product-statement claim of areal gas-in-place
against `unconventional.rs`, which computes intensive gas content in scf/ton and takes no
thickness or area.

---

## 5. Who pushes

For a lane touching only new files, the lane may push and open the PR — the diff is additive and
a bad one is obvious.

**For a lane touching a spine file, the push is Jauhar's.** A spine change should reach GitHub
only after he has looked at it, and the cheapest way to guarantee that is for the push to be his
keystroke. The lane stops at commit and reports.

Either way the lane never merges. And `git push -u origin <branch>` in full, never a bare
`git push`: a worktree created from `origin/master` tracks `origin/master`, and the explicit form
removes any chance of the one push that must never happen.

Merge method follows `CLAUDE.md`: a merge commit where the branch has an arc worth keeping — one
commit per chapter is such an arc — and rebase for a single commit. A branch that will keep
receiving commits after the merge must use a merge commit, or the local branch is left holding
commits whose content is already on `master` under different hashes.

---

## 6. What a second agent may not be given

`CLAUDE.md`'s never-delegate rule is unchanged and it is the whole safety argument: physics
defaults, which must trace to `docs/` or a cited source, and anything touching the DuckDB write
discipline.

The workable distinction is narrow: **a lane implements a specification that already carries its
citations. It never derives one.** A PRD v2 chapter with cited parameters is safe to implement
from. Choosing a value is not delegable, because `CONTRACT.md` §2 says a parameter is cited or it
ships `ABSENT`, and the cheapest move for any agent under pressure to finish is to supply a
literature-plausible number. That number computes, plots, prints in a client deliverable, and no
test catches it — there is no reference value to compare against.

So a lane's correct output, where it cannot source something, is a **named gap**. Say that in the
prompt, and ask for the list of refusals as a deliverable: it is more useful than the edits.

One legal case deserves its own sentence. `CONTRACT.md` §2.2's **C-1** class means
patent-claimed, and a lane cannot settle C-1 versus C-2. Every such call is a **draft opinion**,
labelled as one, and every C-1 candidate goes on the list already with a lawyer.

---

## 7. The physical limits, which decide how many lanes fit

- **One running app.** `vite.config.ts` sets `strictPort: true` on 1420, so exactly one
  `npm run tauri dev` exists at a time. UI verification is serial however many agents run, which
  makes "UI" the worst possible third lane.
- **A docs lane costs nothing.** No `npm install`, no `target/`, no port. Several can run.
- **A code lane is expensive.** Its own `node_modules` and its own `target/` with DuckDB compiled
  from source. Verify `tools\check.ps1` passes in a fresh worktree *before* opening code lanes —
  that is `SB-CORE-041`, and a fresh checkout has broken before: `.gitignore` swallowed a fixture
  `include_bytes!` needs at compile time, so the whole test target failed with "cannot find the
  file specified".
- **The real ceiling is the human.** Every lane produces a PR he must read and a `REVIEW.md` entry
  he must field-verify against real wells. Two queues into one person is the limit, not tokens.
- **Windows holds folders open.** `git worktree remove` fails while the agent's app, an Explorer
  window, or any terminal sits in the folder. Close them; then `rm -rf` and `git worktree prune`,
  which is the proper cleanup, not a workaround.

---

## 8. What the arrangement caught in one day

Recorded because it is the evidence that the discipline pays for itself, and because each item is
a class of error to expect again:

- A **product-statement overclaim** — areal gas-in-place against a module that computes gas
  *content*. The code's own doc string was the honest one.
- A **misleading output mnemonic** — `GIP_ADS` / `GIP_FREE` / `GIP_TOTAL` carrying scf/ton. A
  reader takes `GIP` for a volume. That is `SB-CORE-007`'s output-mnemonic ownership, so fixing
  only the document leaves the wrong name shipping.
- A **stale spine claim**: `00_INDEX.md` §0.4 still barred a capability the amended contract
  requires be independently derived.
- **733 lines of stranded work.** PRD v2 states PRD v1's four follow-on documents were never
  written, "verified 2026-08-07". Three of them exist — `V1_SCOPE.md`, `RELEASE.md`,
  `TARGET_ARCHITECTURE.md`, unmerged on a branch 206 commits behind. The verification checked
  `master`, where it was true. Found by a branch sweep, not by reading the documents.
- **A wrong interpreter.** `SANDIBUMI_PYTHON` pointed at a Python 3.8 with no `scikit-learn`,
  below the 3.10 floor, while every package sat in the Python 3.12 environment. The ML suite,
  DLIS import and all three office exports were failing for that one reason.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.
