# Security review — 2026-08-22

Scope as briefed: the surfaces that take untrusted input or run code. Delivered as findings first;
**F1 was then fixed on Jauhar's decision and is marked so below, and nothing else was touched**.
Method follows the `security-review` skill (categories, false-positive filter,
confidence ≥ 8 to report), applied to the shipped surfaces rather than to a branch diff.

**Threat model used.** This is a desktop application processing the operator's own data, running
with their privileges. So "the user can read their own files" is not a vulnerability. The two
boundaries that *are* real:

1. **A delivered file** — a LAS, DLIS, CSV, SCAL, deviation survey or photograph that arrived
   from a laboratory, a partner or an operator. Its contents are not under our control.
2. **A project file** — a `.duckdb` that arrived from someone else. This is the boundary that
   produced every finding below, and the one I think is currently under-defended.

---

## 1. What was confirmed safe

This half matters as much as the findings, because it is what you can tell a client.

### 1.1 Python subprocesses — no command injection, and every runner reads bytes

**Every** Python runner reads `sys.stdin.buffer`, never `sys.stdin`. Enumerated across all 18
runner constants in 8 modules (`grep -rn "sys\.stdin" --include=*.rs`, then a negative sweep for
`sys.stdin.read()` / `readline()` / `input()` outside `.buffer`, which returned one doc comment and
one unrelated test name). This is broader than CLAUDE.md claims — it also holds for `coreimage.rs`
and `petrography.rs`, which that file does not name.

**No user text reaches a command line or a script body.** Every spawn is one of two shapes:

- `Command::new(python).args(["-c", CONST_RUNNER])` — the script is a Rust `const`;
- `Command::new(python).arg(script.path())` — the script is written to a temp file from a `const`.

Arguments are passed as a vector, never through a shell, so there is no quoting surface. Two
spawns build their script dynamically and both are benign: `ml.rs:2795` interpolates
`ML_RUNTIME_PY` (a constant), and `images.rs:1427` is inside a `#[test]`.

Rule 7 holds throughout: the interpreter is discovered, never required, and a missing one fails
only the feature that needed it.

### 1.2 The write whitelist — no SQL injection

Twenty-one SQL-building `format!` sites carry an interpolated token
(`grep -rnE 'format!\(' | grep -E '(INSERT|UPDATE|…) ' | grep -c '{'` → **21**). Every one was
inspected. Each interpolated identifier is one of:

| Guard | Sites |
|---|---|
| A compile-time constant or a literal array being iterated | `db.rs:2187,2202,9269,9467,9558`, `equations.rs:2157`, `ingest.rs:3349`, `ancestry.rs:3632` |
| **Type-enforced** `&'static str` — a user `String` cannot become one | `curve_edit.rs:236` (`CurveStore::Standard(&'static str)`) |
| Checked against an allowlist before use, error otherwise | `db.rs:4108` (`set_datum_store`), `9286/9294/9302` (match arms), `9344` (`EDITABLE`), `5246/5247` (`TABLE_SPECS` find-or-error) |
| Generated placeholders (`?, ?, ?`) or a `usize` | `ancestry.rs:1703,2588`, `db.rs:5246/5247` offset/limit |
| Identifier-quoted with doubled quotes | `project.rs:271` |
| The read-only wrapper, analysed below | `db.rs:5799` |

`TABLE_SPECS` is a genuine whitelist: `get_table_page` refuses any table not in it, and takes its
columns and sort order from the spec rather than from the caller.

The strongest of these is worth naming: `CurveStore::Standard` holds a `&'static str`, and its only
construction site reads `column.storage_column` out of a constant table. The **type system** makes
injection impossible there, not a runtime check.

### 1.3 The SQL panel is read-only, on four independent layers

`db::run_readonly_query` requires the first non-comment token to be `select` or `with`, refuses any
`;` in the body, and wraps the query as `SELECT * FROM (<user sql>) __sandibumi_q LIMIT n`. The
wrap is the real boundary — the code says so already — because a bare `DELETE`/`INSERT`/`COPY` is
not a valid table expression.

The existing suite pins the CTE-**prefixed** write (`WITH x AS (SELECT 1) DELETE FROM wells`) and
asserts the row count is unchanged. It does **not** cover the other shape — the write *inside* the
CTE body, `WITH x AS (INSERT … RETURNING *) SELECT * FROM x`, which starts with `with`, carries no
semicolon and *is* a valid table expression.

I settled that by experiment rather than assertion. A throwaway test against an in-memory project:

```
WITH x AS (INSERT INTO wells … RETURNING *) SELECT * FROM x  -> Parser Error: A CTE needs a SELECT
WITH x AS (DELETE FROM wells RETURNING *) SELECT * FROM x    -> Parser Error: A CTE needs a SELECT
WITH x AS (UPDATE wells SET … RETURNING *) SELECT * FROM x   -> Parser Error: A CTE needs a SELECT
row count after all three attempts                           -> unchanged
```

**DuckDB does not implement data-modifying CTEs**, so this shape is refused by the engine even
before the wrap would catch it. The probe was reverted; it is not in the tree.

### 1.4 A malformed file fails the import, not the app

Every production panic site reachable from an import was inspected and is guarded:

- `parsers.rs:2409` — `partial_cmp(…).unwrap()` runs on a vector explicitly built with
  `.filter(|v| v.is_finite())` and guarded by an empty early-return, so `None` is impossible.
- `petrography.rs:1420` — `values[*idx.last().unwrap()]` is preceded by `if values.is_empty()
  { return NaN }`, so `idx` is never empty.
- `ingest.rs:341` — the `unreachable!()` follows `for i in 1..`, which always returns.
- `parsers.rs:1823` — a documented invariant; refuse is handled before resolution.

`dlis.rs`, `coreimage.rs` and `images.rs` carry **no** production `unwrap`/`panic!` at all.
Parsers return `Result` and the import rolls back as a unit — pinned by an existing test that
asserts `wells`, `standard_curves`, `curve_meta` and `curve_samples` are all empty after a failed
delivery.

### 1.5 Export paths cannot escape the chosen folder

Only two paths build a filename from a well name — `report.rs:835` and `office.rs:1306` — and both
map every character that is not alphanumeric, `-` or `_` to `_`, which kills `..`, `/`, `\` and
`:`. `report.rs` additionally falls back to the well id when a name sanitises to nothing, and
de-duplicates stems so two wells cannot overwrite each other. `export.rs` builds no user-named
path at all; LAS export writes to the destination the file dialog returned.

This one is worth keeping: a hostile LAS carrying a well name of `..\..\Windows\System32\x` is the
obvious attack, and it does not work.

---

## 2. Findings

### F1 — A project file is executable content, and nothing says so

* **Status: FIXED 2026-08-22** — option (a), Jauhar's call. See the closing note below.
* **Severity: MEDIUM** (impact HIGH, requires a deliberate user action) · **Confidence 9**
* **Category:** `deserialization_rce` / `code_execution`
* **Locations:** `ml.rs:1772` (`joblib.load(_io.BytesIO(blob))`), and the `"equation"` `doc_type`
  persisted in the `documents` table.

A `.duckdb` project carries two kinds of executable payload:

1. **Saved ML models.** `ml_models.data` is a joblib dump — a Python pickle. `apply_ml_model`
   streams it to the runner, which calls `joblib.load` on it. Unpickling executes arbitrary code
   **before** any of the feature-order or shape checks below it run; those checks defend against a
   *wrong* model, not a *hostile* one.
2. **Saved equations.** `"equation"` is a persisted document type, and running a saved equation
   executes its Python by design.

**Exploit scenario.** An analyst receives a project file from a partner, a client or a colleague —
entirely normal in this business, and the whole point of `Save Project As`. They open it, see a
saved model in the Saved models list, and press Apply to a scope of wells. Arbitrary code runs with
their privileges, on the machine holding the confidential deliveries. The equation route is worse
in one respect: opening a saved equation and pressing Run is a routine daily action, and the user
believes they are running their own code.

**What already limits this**, and it is not nothing: `list_ml_models` never selects `data` (pinned
by a test), so browsing models does not unpickle anything, and there is no model-import feature —
the only route in is a whole project file. Nothing executes merely on opening a project.

**Recommendation — this is a judgement call and I am not making it.** The options, cheapest first:

- **(a) Say so.** Record the project's origin on open (a path outside the app's own project folder,
  or simply first-open) and show a one-line notice the first time a model or a saved equation from
  that project is run: *this project came from elsewhere and contains code that will run on your
  machine*. Costs nothing, changes no behaviour, and is what every office suite does.
- **(b) Sign what we wrote.** Store a keyed digest of the blob when the model is saved and refuse a
  blob whose digest does not match. Stops a tampered project, but a key that ships in the binary is
  not a real signature, so this is weaker than it looks.
- **(c) Do nothing and document it.** Defensible if you decide project files are only ever
  self-produced — but that is not how `Save Project As` is used.

I lean to **(a)**. It is honest, it is one dialog, and it does not pretend to a security property
we cannot deliver.

**Decision and outcome (2026-08-22).** Jauhar chose **(a) warn once, then let it run**, and it
is implemented. A project this machine has not seen before, which actually carries saved equations
or saved models, announces that once after opening; the user acknowledges and carries on. Nothing
is gated, nothing is refused, and a project holding only curves says nothing at all. A project
CREATED here is trusted at creation, so your own work never nags you.

The acknowledgement is recorded in `%APPDATA%\SandiBumi\trusted-code.json`, beside the recents
list, and **deliberately not inside the project**. A marker written into the `.duckdb` would travel
with the file, so a project passed between two operators would carry a trace of every machine that
had opened it — the repository's client-identifier rule, pointed at file metadata. Pinned from both
sides by `project::tests::a_project_from_elsewhere_is_announced_once_and_nothing_is_written_into_it`,
which compares the project's bytes before and after: an implementation that stamped the
acknowledgement inside the file would satisfy the behaviour half just as well.

What this does NOT claim: it is a notice, not a defence. Someone who acknowledges it and runs a
hostile model still runs it. That is the honest limit of option (a), and it is why option (b) was
described as looking stronger than it is.

### F2 — One panic anywhere makes the project unusable until restart

**CORRECTED 2026-08-22 (pass 2). This finding was wrong about the shipped product, and the
correction matters more than the finding did.**

`src-tauri/Cargo.toml` sets `panic = "abort"` under `[profile.release]`. A shipped
`sandibumi.exe` therefore does not unwind: a panic runs the panic hook and then **terminates the
process**. No stack is unwound, no destructor runs, and **no mutex is ever poisoned**. The failure
described below - a session in which every later database operation fails - can happen under
`cargo test` and `tauri dev`, which use the unwinding dev profile, and cannot happen in a build a
client runs.

Measured rather than reasoned, with a standalone probe compiled both ways
(`rustc -C panic=unwind` / `-C panic=abort`, panic on a thread holding a `Mutex`):

| | hook runs | mutex poisoned | code after the panic |
|---|---|---|---|
| `panic = "unwind"` (dev, test) | yes | **yes** | runs |
| `panic = "abort"` (release) | yes | n/a | **nothing runs** |

`project::open_and_migrate` had documented this all along - *"with `panic = "abort"` plus
`windows_subsystem = "windows"` it kills the process with no window, no dialog and no console"* -
and this review contradicted the codebase without noticing. That is the more useful lesson here:
the finding was assembled from a `grep -c` of `lock().unwrap()` and the standard poisoning rule,
without checking which panic strategy the product ships with.

**So the recommendation below - replace the lock sites with a recovering helper - is withdrawn.**
It would have touched 182 call sites and changed the behaviour of no shipped build whatsoever.

**What the real shipped failure is, and what was done about it.** A panic closes the window
instantly, with no console and no dialog. The user reports *"it just closed"*, which is the least
diagnosable sentence there is. `diagnostics::install_panic_hook` now writes the panic's location and
message to `crash-log.txt` in the per-user config directory **before the process dies**, and the next
launch's diagnostic report reads it back, redacted and dated. That converts the one failure mode a
client can actually hit from silent into reportable. See `docs/OBSERVABILITY-2026-08-22.md` section 5.

**Still open, and deliberately not done:** recovering from a poisoned lock, which would only ever
help a developer running `tauri dev`. It is worth doing if the dev loop ever suffers for it; it is
not worth 182 call sites for a shipped-build benefit of zero.


* **Severity: MEDIUM** (robustness, not classic security) · **Confidence 9**
* **Category:** `availability` / `error_handling`
* **Location:** `lib.rs` — **218** occurrences of `lock().unwrap()` (`grep -c`).

The DuckDB connection is a `Mutex<Connection>`. Rust poisons a mutex when a thread panics while
holding it, and every subsequent `lock().unwrap()` then panics too. So a single panic inside any
command that holds the lock does not just fail that command — it makes **every later database
operation** fail for the rest of the session.

I want to be precise about the severity, because the security skill would exclude this as DoS and
I am reporting it under your framing instead ("a malformed file must fail the import, never the
app"). I found **no reachable panic** in the import parsers (§1.4), so I am not claiming a file can
trigger this today. The finding is that the *blast radius* of any future panic is the whole
session, and that a user experiencing it sees an app where nothing works any more with no
explanation — which is exactly the unsupportable failure the observability pass is about.

**Recommendation.** Replace `lock().unwrap()` with a helper that recovers the guard
(`PoisonError::into_inner`) and records one boot-note-style entry saying the session hit an
internal error. The data is not corrupted by a panic — DuckDB transactions are atomic — so
recovering the guard is sound and turns a bricked session into one failed command. This is
mechanical, gated by the compiler, and I would want it in the observability increment rather than
here.

**RESOLVED 2026-08-22 (pass 2).** It is provably unreachable today, which this review could not establish: `classify_las_section` returns only `VersionBlock`, `WellBlock`, `CurveBlock`, `AsciiData` or `None`, the `None` arm returns `Header` earlier, and `AsciiData` has already returned above. So no LAS can reach it. It was still changed to a parse error, because an `unreachable!()` under `panic = "abort"` is not a failed import - it is a closed window, and adding a `Header` arm to `classify_las_section` would have armed that silently. One line, no behaviour change for any file that parses today.

### F3 — An `unreachable!()` on a LAS section I could not prove unreachable

* **Severity: LOW** · **Confidence 7** — *below the reporting bar, listed for completeness only*
* **Location:** `parsers.rs:1122` — `LasSection::Header | LasSection::AsciiData => unreachable!()`

`AsciiData` provably cannot reach it (the branch above returns early). `Header` I could not rule
out by reading, and I did not construct a LAS that reaches it. If it is reachable, a crafted LAS
panics the import thread — which, per F2, would poison the connection.

**Recommendation.** Turn it into a parse error rather than a panic. One line, no behaviour change
for any valid file. I flag it at confidence 7 because I could not demonstrate reachability, so
treat it as cheap insurance, not a known hole.

### F4 — Predictable temp-script filename

* **Severity: LOW** · **Confidence 8** — *not exploitable on the shipping platform*
* **Location:** `ml.rs:4110` — `sandibumi-{tag}-{pid}-{seq}.py` in `std::env::temp_dir()`

The name is predictable, and `fs::write` follows an existing symlink. On a shared `/tmp` another
local user could pre-create that path and have the app execute their Python.

On Windows — the shipping platform — `%TEMP%` is per-user and ACL-restricted, so no other user can
place that file, and same-user malware does not need this route. **Not exploitable as shipped.**
Worth a note only if a macOS or Linux build is ever contemplated.

---

## 3. One observation, deliberately not a finding

**The SQL panel can read arbitrary local files.** Verified: `read_text()`, `read_blob()` and
`read_csv_auto()` all succeed through `run_readonly_query`.

This is **not a vulnerability** — it is the operator, running their own SQL, reading their own
files, with their own privileges. No boundary is crossed. I record it only so nobody later assumes
the SQL panel is a sandbox that could be safely exposed to a less-trusted user; it is a
read-only-to-the-*project* console, not a confined one.

---

## 4. What this review did not cover

Stated so the gaps are known rather than assumed:

- **Dependency vulnerabilities** (`cargo audit`, `npm audit`) — excluded by the skill and managed
  separately.
- **The DuckDB file format itself.** Opening a hostile `.duckdb` relies on DuckDB's own parser
  being robust. That is a third-party trust decision, not something this codebase controls.
- **The Tauri IPC allowlist / CSP configuration** — not in the briefed scope.
- **`dlisio`, `Pillow`, `scikit-learn` parsing hostile input.** These are third-party parsers
  running in a subprocess. That subprocess boundary is real and valuable: a crash in `dlisio`
  fails the import, not the app. But the subprocess is **not a sandbox** — it runs with full user
  privileges, so a code-execution bug in one of those libraries would be a code-execution bug here.
  Rule 7 buys isolation of *failure*, not of *privilege*.
