# Kimi K3 as a delegation tier

Jauhar holds a **Kimi Code Allegretto** subscription ($39/mo flat) and asked whether it can carry
the daily coding on SandiBumi — adding and refining features — as a subagent tier beside Claude.

It can, for a specific and well-defined half of the work. This note records the mechanism, the
split, and the three hazards that are specific to this repo. It is written to be portable: nothing
here depends on machine-local memory.

## 1. The architectural fact that decides the shape

**You cannot have Claude as the main agent and Kimi as subagents inside one Claude Code process.**

`ANTHROPIC_BASE_URL` is process-wide. Subagents inherit the session's endpoint, so the moment the
session points at Moonshot, *everything* — main agent, subagents, background titles, summarization —
is Kimi. There is no supported per-subagent provider switch.

That leaves three real shapes, and the recommendation is a mix of the first two:

| Shape | What it is | Verdict |
|---|---|---|
| **A. Whole session on Kimi** | Set the env vars, run `claude` as normal. Every agent in that session is K3. | **The main win.** Zero moving parts, flat fee, no Anthropic spend. Use it for whole increments of mechanical work. |
| **B. Kimi as a shelled-out worker** | Stay in a Claude session; hand a scoped brief to `claude -p` running headless under Kimi env vars, in a git worktree. | **The precision tool.** Explicit, no silent fallback, the strong model keeps the verify gate. |
| **C. `claude-code-router`** | A local proxy routing per request; per-subagent via a `<CCR-SUBAGENT-MODEL>provider,model</CCR-SUBAGENT-MODEL>` prompt prefix. | **Not recommended here.** It adds a moving part whose failure is *silent* — Claude Code's own subagent model routing already has a standing bug report where every mechanism resolves back to the parent model. A routing miss means either an Anthropic bill you thought was free, or a Kimi run you thought was Opus. In this repo the second one is the dangerous direction. |

## 2. Setup — and the one thing that goes wrong

There are **two different Kimi endpoints**, and picking the wrong one is the single most common
failure. A subscription is worthless against the pay-per-token endpoint.

| | Subscription (what Jauhar has) | Pay-per-token |
|---|---|---|
| Key created at | `kimi.com/code/console` | `platform.kimi.ai` |
| `ANTHROPIC_BASE_URL` | `https://api.kimi.com/coding/` | `https://api.moonshot.ai/anthropic` |
| Billing | Flat fee, rides the plan | Per token |

Allegretto unlocks `k3`, `kimi-for-coding` and `kimi-for-coding-highspeed`, with **1M context** via
the `k3[1m]` form (that bracket syntax is a Claude Code env-var convention only — everywhere else it
is plain `k3` at 256K).

`tools/kimi.ps1` sets the whole block. Two details in it are load-bearing:

- **Every model-tier variable must be set.** A missing `ANTHROPIC_DEFAULT_HAIKU_MODEL` does not
  error — it makes subagents and background tasks fail *silently* on an unknown model.
- **`ANTHROPIC_API_KEY` is cleared.** A leftover Anthropic key silently conflicts with
  `ANTHROPIC_AUTH_TOKEN` and breaks auth in a way that reads like a network fault.

Verify with `/status`, never with `/model` — the model menu never lists Kimi, so chasing a menu
entry is a dead end. `tools/kimi.ps1 -Verify` does the headless version.

## 3. What Kimi K3 is genuinely good at, and where it is not

Stated without favouring either vendor, because getting this wrong costs either money or a client
report.

**Good, on published evidence.** K3 (2.8T MoE, July 2026) placed top-three across six coding
benchmarks, led SWE Marathon and Program Bench, and was built for long-horizon agentic work. On
straightforward coding it sits close to Opus 4.8. For mechanical, well-specified, compiler-gated
edits that is entirely sufficient, and at a flat $39/mo it is a large cost lever against the same
work billed per token.

**Where it falls off, and why each one bites *this* repo.**

- **Adversarial / trap tasks: ~36% failure against Opus's ~8%.** SandiBumi's verify loop is minutes
  (`cargo test` through vcvars), so the repo's own cost rule already applies — a cheap edit that
  fails twice costs more wall-clock than one correct expensive pass. Mitigation: give Kimi only work
  whose gate is cheap (`npx tsc --noEmit`, `cargo check`), never work whose only real gate is the
  full `tools\check.ps1`.
- **Degrades on large, interconnected repositories** (~6% F1 against frontier ~20% on the largest
  repo in one published evaluation). SandiBumi is exactly that shape. Mitigation: scope the brief to
  named files. Never hand it "find where this belongs".
- **"May proactively decide details on ambiguous instructions"** — Moonshot's own guidance says
  bounds must be enforced explicitly in the prompt. This is the sharpest one here, because
  SandiBumi's `CLAUDE.md` is ~30k tokens of mostly *prohibitions*, and a model that fills gaps by
  deciding is a model that will invent a default. The repo already bans exactly that
  (`gr_normalize`'s generic-reference rule, `param_open`'s no-default rule, the provenance
  discipline). Mitigation in §5.
- **Benchmark leaderboards overstate real agentic reliability** relative to what shows up in
  plan → implement → validate loops. Treat the numbers above as a ceiling, not an expectation.

None of this makes it a worse *tool*. It makes it a tool for the compiler-gated half of the work,
which is precisely where the repo's existing delegation table already puts its cheap tier.

## 4. The split

Kimi does not change the delegation rule — it changes the price of one row. The rule stands:
**cheap model + cheap verification = good; cheap model + expensive verification = bad; never
delegate when a wrong answer would be SILENT.**

**Give to Kimi** (compiler-gated, convention-light):

- TS / dockview / panel plumbing, `workspace.ts` wiring, a `＋` menu entry, a new pane skeleton
- Tauri command wrappers, IPC struct plumbing, `#[serde(default)]` field additions
- Renames and call-site sweeps
- i18n dictionary entries
- Test scaffolding around an already-decided contract
- Docs and comment passes
- Read-only inventory sweeps ("which modules lack tests", "every call site of `phie`")

**Never give to Kimi** — the same list the repo already marks session-model-only, because every
item fails *silently*:

- `equations.rs`, `multimin.rs` / `multimin2.rs`, `ssc.rs`, `lrlc.rs`, `satheight.rs`, `thomeer.rs`,
  `hfu.rs`, `montecarlo.rs`, `petrography.rs`, `distribution.rs`
- **Any physics default or published coefficient.** Rule 5 requires a cited source, and the Pittman
  episode is the standing proof of how a plausible wrong number survives: two of nine rows carried
  the wrong table's coefficients and only a monotonicity break exposed them.
- **Anything touching the DuckDB write discipline** — the PK-less `computed_curves` contract, the
  active-set SQL fragments, `with_txn` boundaries.
- **Provenance-sensitive edits.** No client identifier, no client-fitted default. A model that fills
  gaps by deciding must not be near this.

## 5. Working rules for a Kimi run

1. **Restate the binding rules in the brief.** Do not rely on it obeying all of `CLAUDE.md` — name
   the three to five rules that bind *this* task, verbatim. This is the direct mitigation for
   "proactively decides details".
2. **Name the files.** Scope beats search on a repo this size.
3. **Name the gate, and require it green** before the work is reported done. `npx tsc --noEmit` and
   `cargo check` are cheap; the full gate is not.
4. **Kimi writes, Claude or Jauhar reviews, Jauhar merges.** Collaboration rule 5 already routes
   every change through a PR, which is exactly the review boundary this needs. Never let a Kimi
   session push to `master`.
5. **Run it in a worktree** when handing off from a live Claude session, so two agents are not
   editing one tree.
6. **A Kimi result is not verified on its own say-so** — the same rule that already applies to every
   subagent tier.

## Sources

- [Moonshot releases Kimi K3 (VentureBeat)](https://venturebeat.com/technology/chinas-moonshot-ai-releases-kimi-k3-the-largest-open-source-model-ever-rivaling-top-u-s-systems)
- [Kimi K3 pushes Chinese AI into Fable-level territory (Fortune)](https://fortune.com/2026/07/16/moonshots-kimi-k3-pushes-chinese-ai-into-fable-level-territory/)
- [Using with third-party coding agents (Kimi Help Center)](https://www.kimi.com/help/kimi-code/third-party-agents)
- [Use Kimi in Claude Code (Kimi API Platform)](https://platform.kimi.ai/docs/guide/claude-code-kimi)
- [Kimi Everywhere — one harness, one subscription (gist)](https://gist.github.com/Maciejdziuba/038c0c822d2c799cffcfdec805975e66)
- [Kimi K3 with Claude Code: setup, env vars and real limits](https://www.codeagentswarm.com/en/guides/kimi-k3-with-claude-code)
- [Kimi K3 benchmarks: strong on paper, weak on precision (Semgrep)](https://semgrep.dev/blog/2026/kimi-k3s-code-security-results-lack-precision/)
- [Kimi K3 real-world coding review (MindStudio)](https://www.mindstudio.ai/blog/kimi-k3-real-world-coding-review)
- [claude-code-router](https://github.com/musistudio/claude-code-router)
- [Subagent model routing bug report (anthropics/claude-code#43869)](https://github.com/anthropics/claude-code/issues/43869)
