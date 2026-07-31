# SandiBumi manual field-test plan (UAT)

**Generated 2026-07-22** from the live UI source (real button labels, real dialog fields), cross-referenced
against REVIEW.md's click-through ledger and the 65 confirmed findings in `AUDIT-2026-07-21-full-qc.md`.
This is the HUMAN test companion to that code audit: the audit proved what the code does; this plan
verifies what the running app does in your hands, with real field data.

## How to use this plan

1. **Never test against your live project database.** Make a copy of a real project `.duckdb`
   (while the app is CLOSED) or build a fresh UAT project from the Balam South LAS set. Several
   tests import/edit/delete data.
2. Work through sections in order — they're sequenced so earlier tests create the data later
   tests need (imports before modules, modules before pay summary, SCAL before Pc/SHF fits).
   Within a section, tests are ordered cheap-smoke first.
3. Each test ends in a **Pass / Fail / Blocked** checklist — tick exactly one, and jot anything
   odd in Notes, including rough timings on the PERF tests (ROADMAP's Performance tier #128–132
   is explicitly waiting on those numbers). These are standard markdown task boxes
   (`- [ ]` → `- [x]`).

   **How to tick them on this machine — use the keyboard, not the mouse.** Install the
   **Markdown All in One** extension (`yzhang.markdown-all-in-one`) in Cursor, then put the
   caret on a checkbox line and press **Alt+C** to toggle it. That works in the editor itself,
   so there is no preview to round-trip through.

   Do **not** expect clicking to work in a preview pane. GitHub and Obsidian toggle task boxes
   and write the change back, but most built-in editor previews — VS Code and Cursor included —
   render them **read-only**, so a click does nothing at all. That is not a problem with this
   file. Failing everything else, edit the source and put an `x` between the brackets by hand;
   `tools\testplan-tally.ps1` reads the source text, so every route scores identically.

   Two things to know if your editor **formats markdown on save** (Prettier and the built-in
   formatter both do this): it will re-pad every table and re-write `*emphasis*` as `_emphasis_`,
   producing an enormous diff that buries your actual ticks. It also rewrites a bare `______`
   blank into `**\_\_**`, which the tally script counts as a real note. Either turn formatting
   off for this file, or accept the reformat once and commit it separately from your results.
4. Tests carrying a **Known issue** line are _expected_ to fail that specific way — the cause is
   already confirmed in `AUDIT-2026-07-21-full-qc.md` and queued for fixing. Log them as
   "known, confirmed in app" rather than as new bugs. If one fails in a _different_ way than
   described, that IS a new finding — note it.
5. A test that can't run because its precondition failed is **Blocked**, not Fail.
6. When done, run `tools\testplan-tally.ps1` (see under the Tally table) — it counts the ticks
   for you and prints the Fail/Blocked rows with their Notes, ready to hand back to a Claude
   Code session in this repo for serial fixing.

### "Automated coverage" lines (added 2026-07-31)

Some tests now carry an **Automated coverage** line just above their Result block. It says whether a
Rust test on the green gate (`tools\check.ps1`) already checks that test's arithmetic, and names it.
Three forms:

- **pinned** — the numbers are checked on every gate run. Your tick still adds something: the gate
  proves the arithmetic, not that the running app puts it on screen where you can see it.
- **pinned, with a residual** — most of the claim is checked; the line names the part that is not.
- **none** — and, where it says so, why there will not be any.

Where a test **pins a known defect AS-IS**, the line says so. That is not a vote that the behaviour
is right; it stops it drifting further while the decision is open, and the test fails the moment
someone fixes it, which is the alarm.

**Nothing automated has touched, or will ever touch, your Pass / Fail / Blocked boxes.** Those are
yours alone, and `tools\testplan-tally.ps1` scores only those — it does not read the coverage lines.
A `[x]` in `docs/review_triage.md` means "a Rust test checks this"; a `[x]` here means "Jauhar ran it
and it worked." The two are deliberately different things.

## Tally

| Code  | Section                                                            | Tests   | Pre-flagged known issues |
| ----- | ------------------------------------------------------------------ | ------- | ------------------------ |
| SHELL | Shell & project lifecycle                                          | 18      | 1                        |
| IMP   | Data import & export                                               | 17      | 2                        |
| WELL  | Wells, groups, tops & zones                                        | 17      | 1                        |
| PREP  | Prep & conditioning modules                                        | 19      | 2                        |
| PETRO | Core petrophysics modules                                          | 19      | 2                        |
| ADV   | Advance tab (SSC/SSPW, RtC/IMTS, SandiMin, Sw-height)              | 19      | 6                        |
| RT    | Rock typing, HFU, Pc fit, SHF fit, facies tie-in                   | 18      | 2                        |
| BATCH | Batch & field-scale tools                                          | 19      | 3                        |
| MLEQ  | ML, equations & curve management                                   | 18      | 6                        |
| PLOT  | Plots, viewers & curve editing                                     | 20      | 2                        |
| REP   | Reporting & database access                                        | 19      | 2                        |
| AUX   | Cross-cutting & auxiliary features                                 | 20      | 2                        |
| INT   | End-to-end integration & performance                               | 20      | 2                        |
| SHIP  | Session 2026-07-29 shipping checks (CSP, R30, R-A, R-B, R-C, gate) | 7       | 0                        |
|       | **Total**                                                          | **250** | **33**                   |

**Result summary — don't count by hand.** Tick the boxes as you go, then run:

```bash
powershell -ExecutionPolicy Bypass -File tools\testplan-tally.ps1
```

It reads the marks back and prints this table filled in (per-section Pass / Fail / Blocked /
Untested), then the **Fail / Blocked list with each test's Notes** — which is exactly what
step 6 above asks you to hand back for fixing. A test with more than one box ticked is listed
separately and counted in no column, so a contradictory mark can never be scored as a pass.

---

# Section SHELL — Shell & project lifecycle

### Cluster SHELL — app shell & project lifecycle

Shared preconditions: reference machine (ARUNIKA), repo at `D:\XX. SandiBumi`, port 1420 free. A working project with **at least 3 wells imported** (GR+RHOB+NPHI, e.g. Balam South LAS set) and at least one spare `.duckdb` path to create test projects in. Never force-kill `npm run tauri dev` except where T-SHELL-18 explicitly says so — an unclean kill mid-write can corrupt the project WAL. Tests are ordered cheap-smoke → deep/negative; run in order where preconditions chain (T-05/06 create projects reused later).

### T-SHELL-01 — App launch (dev run)

**Tool/panel:** app shell (CLAUDE.md §Dev commands, `index.html`, `src/main.ts`)
**Preconditions:** clean boot, no SandiBumi instance running, port 1420 free.
**Steps:**

1. Open `cmd.exe` and run the pinned dev command (MSVC 14.50 is broken on this machine):
   `cmd.exe /c "call \"C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat\" -vcvars_ver=14.29 && set PATH=C:\Program Files\nodejs;%USERPROFILE%\.cargo\bin;%PATH% && cd /d \"D:\XX. SandiBumi\" && npm run tauri dev"`
2. Wait for the Rust compile to finish and the desktop window to appear.
   **Expected:** Window opens titled **SandiBumi — {project name}**. Ribbon shows tabs **Project / Data / Petrophysics / Advance / Plot / View** with **Petrophysics** active; there is **no icon strip left of the tabs** (removed 2026-07-30). Click **Project**: its groups read **Project** (Open Project… / New Project… / Save Project As… / Recent ▾), **Session** (Save Session… / Open Session…), **Edit** (Undo / Redo, both greyed), **Monitor** (History / Processing / Performance), **Appearance**, **Language**, **Help** — every one a labelled button, no bare icons. Status bar at the bottom reads **Ready**. Sidebar anchor panes **Wells**, **Tops**, **Processing**, **Performance** are present plus **Log View** and **Inspector**. No error dialogs. If the app panics on startup instead: check `src-tauri\` for `.corrupt-backup-*` files (WAL recovery already ran — note it, relaunch once).
   **Automated coverage - end-to-end (pile C, 2026-08-01):** `shell.e2e.mjs` "renders the ribbon,
   the status bar and the workspace" drives the built app and asserts every declared tab has a
   panel and no panel is orphaned, that the status bar exists, and that the dockview workspace was
   created. What stays yours: the window TITLE, the specific group contents of the Project tab, the
   sidebar pane set, and "no error dialogs" — a modal the harness does not know to look for.

   **Result — T-SHELL-01:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-02 — Ribbon tab walk + overflow chevrons

**Tool/panel:** ribbon (`index.html`, `src/ui/ribbon.ts`)
**Preconditions:** app launched (T-SHELL-01).
**Steps:**

1. Click each ribbon tab in turn: **Project**, **Data**, **Petrophysics**, **Advance**, **Plot**, **View**.
2. On **Data**, open **Import Logs ▾**, then click **Import Data ▾** — then click elsewhere.
3. Narrow the app window until a tab's groups no longer fit (≈720 px); click the **›** box; widen the window again.
   **Expected:** Each tab shows its groups: Project = Open/New Project + Recent ▾ + Theme + Language; Data = Import Logs ▾ / Import Data ▾ / **Export LAS…** / Tools ▾ + Wells & Tops / Curve Catalog / DB Inspector / SQL Query; Petrophysics = module dropdowns + Zones… + Cutoffs & Summary… + batch group; Advance = SSC/SSPW/RtC/IMTS buttons + **SandiMin…** + **ML Models…**; Plot = Log Views / Parameter Selection / Correlation / Deliverables; View = New Window / Reset Workspace. Only one dropdown menu is open at a time; picking an item or clicking outside closes it (covers REVIEW.md §"Highlight tool + ribbon overflow…" and §Wave A-2). When narrow, no raw scrollbar — a boxed **›** appears at the overflowing edge, clicking scrolls the row and **‹** appears at the left; chevrons disappear when the window is wide again.
   **Automated coverage - end-to-end (pile C, 2026-08-01):** `shell.e2e.mjs` "shows exactly one
   ribbon panel at a time, and each one has captioned groups" walks all six tabs and asserts
   exactly one panel visible, exactly one `.active` tab, and no blank group caption. It reads
   `checkVisibility()` rather than the `hidden` attribute on purpose — a CSS `display` rule has
   overridden `hidden` on these panels twice, and in both of those bugs the attribute was correct.
   **Not covered:** step 2's dropdown behaviour (only one menu open at a time) and step 3's
   overflow chevrons, which need a real window resize.

   **Result — T-SHELL-02:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-03 — UI language switch EN → ID → SU → JV → EN

**Tool/panel:** Project tab → Language select (`src/i18n.ts`, `index.html`)
**Preconditions:** app launched; a well with a real name visible in the Wells pane.
**Steps:**

1. Project tab → **Language** → **Bahasa Indonesia**.
2. Walk the ribbon tabs; open Data tab.
3. Switch to **Basa Sunda**, then **Basa Jawa**, checking a few labels each time.
4. Switch back to **English**.
   **Expected:** Status line shows **Language: Bahasa Indonesia**. Labels translate live without a restart: Petrophysics→**Petrofisika**, View→**Tampilan**, Import Logs→**Impor Log**, Tools→**Alat**, Save→**Simpan**; in Basa Jawa: Save→**Simpen**, Depth→**Jero**, Reload→**Muat manèh**. Technical terms stay English by design (Monte Carlo, Pickett, SandiMin, curve mnemonics, LAS/DLIS). Well names and layout names are never translated; the Language dropdown's own option labels stay native names in every language. Back to English restores every label exactly. Choice survives a relaunch. Covers REVIEW.md §"Held-item resolutions" (Bahasa Jawa item) and §Wave A-2 (translated import labels).
   **Automated coverage - end-to-end (pile C, 2026-08-01):** `shell.e2e.mjs` "switches UI language
   EN to ID to SU to JV and back" drives the real select through all four locales and back. It
   keys on **Project**, whose four forms differ only by diacritics (Project / Proyek / Proyék /
   Proyèk), so the assertion proves the RIGHT dictionary was selected — **Petrophysics would have
   proved nothing**, since it is "Petrofisika" in all three translations, and a test built on it
   would pass while serving Sundanese to an Indonesian user. It also asserts an untranslated term
   stays English. **Not covered:** the status line, the deeper label set inside dialogs, that well
   and layout names are never translated, and survival across a relaunch.

   **Result — T-SHELL-03:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-04 — Theme switching, all 8 themes, live repaint

**Tool/panel:** Project tab → Theme select (`src/theme.ts`, `src/ui/ribbon.ts`, styles.css)
**Preconditions:** a well selected; a **Log View** and a **Histogram** (of GR) open side by side. Optional: a **Monte Carlo** pane with a finished run (for the known issue below).
**Steps:**

1. Project tab → **Theme**: switch through every entry in order — **Default, Dark, System, White / Grey** (Standard group) then **Pertamina, Halliburton, Schlumberger, LAPI ITB** (Client group).
2. At each switch, look at: ribbon + pane chrome (dockview tabs/headers), the Log View canvas, the Histogram canvas, and the log-view cursor readout pill (hover a track).
3. Leave a client theme active and restart the app once (close window, relaunch).
   **Expected:** Every switch repaints **immediately, without reopening any panel**: dockview chrome, ribbon, and all canvas plots recolor (theme change bumps `themeVersion`). Status line shows **Theme: {value}** (e.g. `Theme: pertamina`). Client themes are all light/professional in their brand colors; Dark inverts the cursor readout pill legibly; histogram/crossplot pick swatches follow the theme accents (Pertamina = blue/lime) — covers REVIEW.md §Wave A-1 "Theme check". After relaunch the chosen theme is still active.
   **Known issue:** AUDIT-2026-07-21 §Monte Carlo — "Monte Carlo's HPV histogram canvas never repaints on a live theme swap or panel resize, unlike every sibling Canvas-2D dock pane": if a Monte Carlo pane with results is open, expect its HPV histogram to keep the old colors until re-run — log as known, not new.
   **Result — T-SHELL-04:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:** Remove brand name, only color theme, widen the option of color theme

### T-SHELL-05 — New Project + data isolation

**Tool/panel:** Project tab (`src/ui/ribbon.ts` handleNewProject/switchProject)
**Preconditions:** main project open with wells; 1–2 spare LAS files at hand.
**Steps:**

1. Project tab → **New Project…** → in the native "New Project" dialog save as `uat-test.duckdb` (default suggested name is `new-project.duckdb`).
2. Data tab → **Import Logs ▾ → Import LAS…** → import one LAS.
3. Project tab → **Recent ▾** → switch back to the main project.
   **Expected:** After step 1: status **Switching project…** then **Project: uat-test**; window title becomes **SandiBumi — uat-test**; the Project group caption shows **uat-test** (hover = full path); Wells pane shows **No wells ingested yet**; **Project ▸ Edit ▸ Undo/Redo** grey out (undo stacks cleared). After step 2 the well appears only here. After step 3 the main project's wells return and the imported test well is NOT in the list. History panel (**Project ▸ Monitor ▸ History**) has a **Project — Opened project …** entry in each project's own history. Covers REVIEW.md §Wave A-3 items 1 and 3.
   **Result — T-SHELL-05:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-06 — Open Project + Recent list (incl. missing file)

**Tool/panel:** Project tab (`src/ui/ribbon.ts` handleOpenProject/refreshRecentMenu)
**Preconditions:** T-SHELL-05 done (≥2 projects in recents); a Histogram open on the current project.
**Steps:**

1. Project tab → **Recent ▾** — inspect the list without clicking.
2. Click the `uat-test` entry.
3. Switch back via **Open Project…** (native dialog, `.duckdb` filter) picking the main project file.
4. With the app closed later (or from Explorer now), rename `uat-test.duckdb` away, then reopen **Recent ▾**.
   **Expected:** Recent lists up to 12 projects, the current one prefixed **●** and disabled (stored outside any project in `%APPDATA%\SandiBumi\projects.json`). Switching reloads everything: title + caption, wells, and the open Histogram re-reads (empty/new data — no stale plot of the old project); well selection and undo history clear. After step 4 the renamed project shows greyed with suffix **(missing)** and cannot be clicked. Covers REVIEW.md §Wave A-3 item 2.
   **Result — T-SHELL-06:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-07 — Save Project As = backup copy

**Tool/panel:** **Project ▸ Project ▸ Save Project As…** (`src/ui/ribbon.ts` handleSaveProject)
**Preconditions:** main project open with wells.
**Steps:**

1. On the Project tab, find the button labelled **Save Project As…** in the **Project** group — hover it; the tooltip explains it writes a compacted copy to a new file.
2. Click it; save as `backup-uat.duckdb` in the "Save Project As" dialog.
3. Make a small change (e.g. pin a well ★, or add a top) and note it.
4. Close and reopen the app.
   **Expected:** Status **Project saved to {path}**; History entry **Project — Saved project to {path}**; the file exists on disk at the chosen path. The app KEEPS working on the original project (backup-copy semantics, not IP-style switch-to-copy): the step-3 change is in the original project on relaunch, and opening `backup-uat.duckdb` via Open Project shows the pre-change state. Covers REVIEW.md §Wave A-3 note item ("Save Project As stays a backup copy").

   **Automated coverage - pinned (pile B, 2026-07-31):** `save_as_writes_a_backup_copy_and_leaves_the_app_on_the_original` (project.rs) drives the same sequence against a real file and checks the claim from both sides — the copy opens as a valid project holding the state at the moment it was taken, and the well added afterwards is in the ORIGINAL only, read back from disk rather than from the connection that wrote it.

   **Worth knowing before you click:** the backup will be **noticeably smaller than the original**, and that is correct, not data loss. Save As is an engine copy (`ATTACH` + `COPY FROM DATABASE`), which writes live rows only — so it is also a compaction, and a project bloated by months of module re-runs exports at its true data size. In the test a project with 200k deleted rows behind it copies smaller while every one of the 1000 live rows crosses. If you want to be sure, compare well and curve counts rather than file size.
   **Result — T-SHELL-07:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-08 — Clean relaunch restores last project + workspace

**Tool/panel:** app shell (`src/autosave.ts` applyAutosaveExtras, recents)
**Preconditions:** main project open; arrange a distinctive workspace (Log View with a customized layout, a Crossplot, well B active — not the first well).
**Steps:**

1. Wait ≥10 s after arranging (autosave interval).
2. Close the window normally (✕), let `npm run tauri dev` exit on its own.
3. Relaunch with the T-SHELL-01 command.
   **Expected:** No recovery dialog (clean exit). The **last project you had open** reopens (title confirms). The pane arrangement is back, the **active well is still well B**, and the Log View shows its customized layout/track state (autosave carries what dockview's JSON can't). Covers REVIEW.md §Wave A-3 item 4.
   **Result — T-SHELL-08:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-09 — NEGATIVE: project switch refused while a chain runs

**Tool/panel:** Workflow Builder + Project tab (`src/ui/ribbon.ts`, chain registry)
**Preconditions:** main project with all wells; a saved workflow chain of ≥2 modules (e.g. VSH from Gamma Ray → Porosity from Density).
**Steps:**

1. Petrophysics tab → **Workflow…** → run the chain with scope **All** so it takes at least several seconds.
2. While the Processing panel shows live progress, go Project tab → **Open Project…** and pick another project.
   **Expected:** A clear refusal (status/error à la **Project switch failed: …** naming the running chain) — NOT a switch. The chain keeps running to completion in the Processing panel; the current project stays open and uncorrupted. Afterwards (chain finished) the same switch succeeds. Covers REVIEW.md §Wave A-3 item 5.

   **Automated coverage - pinned (pile B, 2026-07-31):** `a_registered_chain_holds_the_project_switch_shut_until_it_is_really_finished` (chain.rs) walks a real chain's whole life against the same predicate the three commands gate on — shut the instant the job is registered, still shut while running, released on Completed, released on Cancelled, and one queued job among finished ones still holds it.

   **Worth knowing before you click:** you do not need to be quick. The job is registered BEFORE the worker thread starts, so the switch is already refused the instant Run returns — there is no window where the chain is running but unregistered. Cancelling releases it, so "cancel the chain, then switch" works and is the intended way out. The same guard covers **New Project** and **Compact Project**, which are worth a try each while the chain runs.

   **KNOWN ISSUE (2026-07-31, finding 17) — OPEN, your call.** Nothing ever removes an entry from the chain registry, so a chain whose worker thread dies without reporting a terminal status stays "running" forever and the guard never releases. Open Project, New Project and Compact Project are then all refused for the rest of the session, each telling you to wait for a job that will never finish; only restarting the app clears it. If during a click-through you get "A background job is still running" with the Processing panel showing nothing in flight, that is this, not a mis-click.
   **Result — T-SHELL-09:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-10 — Sessions: Save Session As / Open Session / delete

**Tool/panel:** **Project ▸ Session** group (`src/ui/ribbon.ts` handleSaveSession/handleOpenSession, `src/ui/workspace.ts` snapshotSession)
**Preconditions:** distinctive workspace: 2 Log Views (one with a custom layout), a Histogram, well B active.
**Steps:**

1. Click **Project ▸ Session ▸ Save Session…** (tooltip: "…the current panes, wells & visualizations as a named workspace"). In the **Save Session As** dialog, clear the default name **My Session**, leave it EMPTY, press **Save** (negative).
2. Type `UAT Layout A`, click **Save**.
3. Wreck the workspace: close the plots, switch to well A, View → **Reset Workspace**.
4. Click **Project ▸ Session ▸ Open Session…** → in the **Open Session** dialog click **UAT Layout A**.
5. Reopen **Open Session** and click the row's **🗑** button.
   **Expected:** Step 1: nothing happens on empty name (dialog stays). Step 2: status **Session "UAT Layout A" saved**; History entry **Session — Saved session "UAT Layout A"**. Step 4: panes, arrangement and the **active well (B)** come back; Log Views restore their per-view layouts; plot panes reopen in place but their internal curve selections may reset (known limitation — not carried by the snapshot). Status **Opened session "UAT Layout A"**. Step 5: status **Deleted session "UAT Layout A"** and the list updates (empty list shows "No saved sessions yet. Use Save Session to create one."). Sessions live in the project DB — they do not appear in other projects.
   **Automated coverage - end-to-end (pile C, 2026-08-01):** `sessions.e2e.mjs` covers save, the
   stored snapshot's shape, the Open Session listing, and delete. The snapshot is asserted field by
   field rather than merely parsing: one that lost `layout` still parses, still restores without
   error, and rebuilds nothing - the workspace comes back empty and it reads as a save that never
   worked. `well` must be present even when null, since its absence is how a session silently stops
   restoring the well it was taken on. **Not covered:** actually APPLYING a session (restoring a
   workspace mid-run would tear down the panes the other specs are driving), and the log-view
   Layout reattachment that `applySession` does by id.

   **Result — T-SHELL-10:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-11 — Quiet Ctrl+S re-save + Escape closes ribbon menus

**Tool/panel:** shell hotkeys (`src/ui/ribbon.ts` quickSaveSession)
**Preconditions:** a session already named this app-run (redo step 2 of T-SHELL-10 if the app restarted since).
**Steps:**

1. Move/resize a pane, then press **Ctrl+S**.
2. Click into any text input (e.g. SQL Query editor or a dialog name field) and press **Ctrl+S** again.
3. Open **Recent ▾** on the Project tab; press **Escape**.
   **Expected:** Step 1: status **Session "…" saved** with NO dialog (quiet in-place re-save of the last-named session) and the unsaved dot on the Save-Session button clears. Step 2: the app-level save does NOT fire while typing in an input/CodeMirror (editors keep their own Ctrl+S). Step 3: the ribbon menu closes; nothing else (no dialog dismissed). Covers REVIEW.md §"Held-item resolutions" (Quiet Ctrl+S save + Escape closes ribbon menus).
   **Automated coverage - end-to-end (pile C, 2026-08-01):** the Ctrl+S half only.
   `sessions.e2e.mjs` checks that once the session has a name, Ctrl+S re-saves it QUIETLY - no
   dialog reopens - and that the status line names the session it wrote. A save that re-prompts
   every time is one people stop using, and the unsaved-state dot then stops meaning anything.
   **Not covered:** Escape closing ribbon menus.

   **Result — T-SHELL-11:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-12 — Dirty-state ● indicators

**Tool/panel:** panel tabs + **Project ▸ Session** (`src/dirty.ts`, `src/ui/workspace.ts`)
**Preconditions:** one Log View open on a well; a session has been saved (nothing currently dirty — the **Project** ribbon tab has no dot).
**Steps:**

1. Edit the Log View: drag a track wider, or toggle a curve via **Plot → Properties…**.
2. Look at the Log View's tab and at the **Project** ribbon tab — WITHOUT switching to it. Then open Project and look at **Save Session…**.
3. Plot tab → **Save Layout…**, save under a name.
4. Rearrange the panes (drag a tab), then click **Project ▸ Session ▸ Save Session…** and save.
   **Expected:** Step 2: the Log View tab shows **●**; the **Project ribbon tab** carries a small amber dot while you are still on another tab (hover it: "Unsaved changes — Project ▸ Session ▸ Save Session…"), and the **Save Session…** button itself has a red dot with "— unsaved changes" in its tooltip. The tab must NOT change width when the dot appears. Step 3: that panel's ● clears (layout is now in a named save) but the workspace-arrangement dot may remain if panes moved. Step 4: **everything** clears — no ● on any panel tab, no dot on the Project ribbon tab, no red dot on the button. The dot means "not in a named save yet" only; the 10-s crash autosave runs regardless. Covers REVIEW.md §P1-b "Unsaved markers" (unchecked item).
   **Result — T-SHELL-12:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-13 — Undo/Redo with live labels (+ History cross-check)

**Tool/panel:** **Project ▸ Edit ▸ Undo / Redo** (`src/undo.ts`, `src/ui/topsEditor.ts`)
**Preconditions:** fresh launch (Undo/Redo greyed); a Log View open on a well.
**Steps:**

1. On the **Project** tab, confirm both **Undo** and **Redo** in the **Edit** group are greyed and their tooltips read plain **Undo (Ctrl+Z)** / **Redo (Ctrl+Y)**.
2. In the Log View's toolbar click **🏷** ("Edit tops: click to add…"), click at any depth, type name `UAT_TOP`, click **Add top**.
3. Back on the Project tab, hover **Undo**; click it.
4. Hover **Redo**; click it.
5. Clean up: Undo once more (leave the project without `UAT_TOP`).
   **Expected:** After step 2: Undo enables, tooltip **Undo add top UAT_TOP (Ctrl+Z)**; History panel gains **Tops — {well}: Added top UAT_TOP at {depth}**. Step 3: status **Undo: add top UAT_TOP**; the top vanishes from the Log View AND the Tops pane; Redo enables with tooltip **Redo add top UAT_TOP (Ctrl+Y)**. Step 4: the top returns at the same depth. Ctrl+Z / Ctrl+Y do the same as the buttons.
   **Result — T-SHELL-13:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-14 — Processing History panel: entries, export, clear

**Tool/panel:** Processing History pane (`src/ui/historyPanel.ts`, `src/processLog.ts`)
**Preconditions:** a project on which several operations were just done (imports, session saves, top add — T-05…T-13 provide them).
**Steps:**

1. Click **Project ▸ Monitor ▸ History** ("Processing history — everything done in this project").
2. Read the list and the count in the toolbar.
3. Click **⭳ Export…**, save as `processing-history.txt` (default name), open the file in Notepad.
4. Restart the app; reopen the panel.
5. Click **🗑 Clear** → in the confirm ("Clear the processing history for this project? This cannot be undone.") choose **Cancel** (negative). Click **🗑 Clear** again → **OK**.
   **Expected:** A pane titled **Processing History** opens (singleton — clicking again refocuses). Newest entries first, each row = time + colored kind chip (**Project / Import / Module / Tops / Session / Export…**) + detail, well-scoped entries prefixed with the well name; toolbar shows **N operations**. Export: status **Processing history exported to {path}**; the file starts `SandiBumi processing history (N entries)` with one `YYYY-MM-DD hh:mm:ss  [Kind] Well: detail` line per row matching the panel. After restart the history is still there (persisted in the project DB — it also travels with Save Project As). Cancel keeps everything; OK empties the list to "No operations recorded yet…" and **0 operations**. Covers REVIEW.md §"Polish — UX" item "Processing history now covers every operation".
   **Result — T-SHELL-14:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-15 — History attribution: single-well vs batch module run

**Tool/panel:** module pane + Processing History (`src/ui/workspace.ts` onRunComplete, wellScope)
**Preconditions:** ≥3 wells with GR; History panel open.
**Steps:**

1. Petrophysics tab → **VSH ▾ → VSH from Gamma Ray**. In the pane's **Wells** scope bar pick **Custom…** and tick exactly ONE well (not the currently selected one). Click **Run**.
2. In the Wells pane Ctrl-click two OTHER wells; back in the module pane pick scope **Selection** (should show 2). Click **Run**.
3. Read the two new **Module** entries in the History panel.
   **Expected:** Petrophysically: VSH_GR lands 0–1, high in shales, low in clean sand (spot-check in a log view). History cross-check: the step-1 entry names the well that was ACTUALLY run (not the globally selected one); the step-2 batch entry carries **no well name** (field/batch convention). Covers REVIEW.md §Round 4 "History attribution" (fix pending click-through — if the entry still names the wrong/selected well, the fix regressed: log as Fail with the well names seen).
   **Result — T-SHELL-15:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHELL-16 — Global well pin 📌 (workspace-follow vs working-pane)

**Tool/panel:** Wells pane group bar (`src/ui/objectTree.ts`)
**Preconditions:** ≥3 wells; two Log Views open (both following the active well A).
**Steps:**

1. Hover the **📌** button in the Wells pane's group bar — note its tooltip; it should be highlighted (Pin ON is the default).
2. Click well B in the tree.
3. Click **📌** (Pin OFF); click inside Log View 1 to make it the working pane, then click well C in the tree.
4. Click inside Log View 2, click well A in the tree, then click **📌** again (Pin ON) and click well B.
   **Expected:** Step 2 (Pin ON): BOTH Log Views and any plots switch to well B; status **Pin ON — every view and plot follows the selected well** was shown when toggling. Step 3 (Pin OFF): status **Pin OFF — only the active panel follows; other views keep their wells**; only Log View 1 switches to C, Log View 2 stays on B; browsing panes (Tops, Inspector) still track the selection. Step 4: Log View 2 (the pane you clicked into last) takes C→A while Log View 1 keeps C; then with the pin back ON everything follows to B. "Active panel" means **the viewer you last clicked into** — clicking a well in the tree does not hand the role to the tree, so with the pin off exactly one viewer always follows. Do not confuse 📌 with the per-well **★** star — that is the pinned-favourites run scope, unrelated to following.
   **Result — T-SHELL-16:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:** Pin off, never follow well even for active panel, and other visual pane such histo, xplot, etc (except log view) cant display multiple groups together, better have option for well selections like modules

### T-SHELL-17 — Interaction guards: right-click, reload, armed number fields

**Tool/panel:** app-wide guards (`src/interactionGuard.ts`, `src/ui/contextMenu.ts`)
**Preconditions:** a Log View, a Crossplot, and any module pane with a number field open.
**Steps:**

1. Right-click the Crossplot canvas, then the empty grey background of a pane.
2. Right-click a ribbon button, a well row in the tree, and a pane toolbar.
3. Right-click inside a text input (e.g. the session-name field or SQL editor).
4. Press **F5**; dismiss the dialog with **Escape**. Press **Ctrl+R**; dismiss it by clicking **Cancel**. Press **F5** and then, with the dialog still up, press **Ctrl+R** again. Finally press the mouse Back (side) button.
5. Single-click the module pane's number field, then double-click it.
   **Expected:** Step 1: the custom app context menu appears (panel-specific items + window actions like Split right / Split down) — on a plot canvas that menu leads with **Properties…**, so the plot's own settings and the window actions are both one click away. Step 2: NO menu at all (native WebView menu suppressed — a stray "Refresh" there would wipe the workspace). Step 3: the native EDIT menu appears (undo/cut/copy/paste — no Refresh/Back). Step 4: **each** of F5 and Ctrl+R raises the same blocking confirm "Reload SandiBumi? The workspace re-opens from its last saved state…" with **Cancel** / red **Reload**; Escape and Cancel both dismiss without reloading; a second reload key pressed while the dialog is already up does NOT open a second dialog — it briefly pulses the open one, so the key is visibly acknowledged; mouse Back/Forward do nothing. Step 5: first click only arms the field (status tip "Number fields arm on click — double-click to edit", no caret); double-click enters edit with the value selected — a stray click+scroll can never change a parameter.
   **Result — T-SHELL-17:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:** right click in xplot showed properties instead of option like in log view, ctrl+R does nothing, beside that good

### T-SHELL-18 — Crash resilience: autosave + recovery dialog (run LAST)

**Tool/panel:** crash recovery (`src/autosave.ts`, `src/main.ts`)
**Preconditions:** distinctive workspace arranged (2+ panes, non-default well); **app fully idle — no import/module/chain running** (killing mid-write risks WAL corruption; that path recovers automatically but muddies this test).
**Steps:**

1. Arrange the workspace, wait ≥15 s (autosave every 10 s).
2. Task Manager → end the **sandibumi.exe** task (this simulates a crash/power loss).
3. Relaunch (T-SHELL-01 command). In the dialog choose **Restore autosaved workspace**.
4. Repeat steps 1–2, relaunch, and this time choose **Start in Safe Mode**.
5. Click **Project ▸ Session ▸ Open Session…**.
   **Expected:** Step 3: BEFORE anything loads, a blocking dialog titled **"SandiBumi did not close properly last time."** offers **Start in Safe Mode** / **Restore autosaved workspace** (the latter focused). Restore brings back panes, arrangement, active well and log-view layouts as of ≤10 s before the kill; status **Workspace restored from the crash autosave**. Step 4: Safe Mode boots the clean default layout; status **Safe Mode — previous workspace kept as session "Recovered {date time}"**. Step 5: the **Recovered …** session is listed and opening it restores the pre-crash workspace — nothing silently lost. If the app instead panics on launch: look for `.corrupt-backup-*` in `src-tauri\` (DuckDB WAL recovery ran) — record as a note, relaunch again.
   **Result — T-SHELL-18:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

# Section IMP — Data import & export

### Cluster IMP — Data import/export (ribbon **Data** tab)

Shared preconditions: app running via `npm run tauri dev` with a project open; a set of real field LAS files (Balam South / reference-suite) plus one real .dlis, core RCAL CSV, tops CSV, deviation CSV, SCAL Pc CSVs, and a well-locations CSV on disk. All import entry points live on the **Data** tab: **Import Logs ▾** (Import LAS…, Import DLIS…), **Import Data ▾** (Import Core…, Import SCAL…, Import Tops…, Import Aux…, Import Deviation…, Import Well Locations…), the flat **Export LAS…** button, and **Tools ▾** (Autocorrelate Tops…, Shift Core…, Well Header…) — covers REVIEW.md §Wave A-2 (compact import ribbon menu check). "Status line" = the message strip at the bottom of the window; "History" = the **Processing History** pane; "Curve Catalog" = the **Inspector** pane opened by the **Curve Catalog** button.

### T-IMP-01 — LAS batch import, multiple files at once

**Tool/panel:** Import LAS… (src/ui/ribbon.ts `handleImport`, src-tauri/src/ingest.rs, parsers.rs)
**Preconditions:** fresh or near-empty project; 3+ clean field LAS files in one folder.
**Steps:**

1. Data tab → **Import Logs ▾** → **Import LAS…**.
2. In the file dialog multi-select 3 LAS files → Open.
3. Watch the status line; then open the **Wells** pane, the **Processing History** pane, and **Curve Catalog**.
   **Expected:** Status shows `Importing 3 LAS file(s)...` then `Imported 3/3 well(s).` All 3 wells appear in the Wells pane without a manual refresh. History gains an `Import — Imported 3/3 LAS well(s)` entry. Curve Catalog lists every curve from each file (standard GR/RES_DEEP/NPHI/RHOB/DT/SP plus extras like PEF/CALI as RAW-set rows with the LAS file's units). Null values (−999.25 or the file's own `~W NULL` declaration — covers REVIEW.md §Chartbook overlay library + audit quick fixes, LAS NULL item) render as gaps in a Log View, not as spikes. GR should read ~10–120 gapi with shale/sand character intact.
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-IMP-01:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-IMP-02 — Re-import a same-named LAS → duplicate-name warning, separate record

**Tool/panel:** Import LAS… (ribbon.ts `handleImport`, ingest.rs ~104–118)
**Preconditions:** T-IMP-01 done (well already in project).
**Steps:**

1. Data tab → **Import Logs ▾** → **Import LAS…** → pick one of the SAME files imported in T-IMP-01 → Open.
   **Expected:** Import completes (`Imported 1/1 well(s). 1 well(s) had depth issues.` — the generic warning note), and History gains a per-well entry containing `a well named '<name>' already exists — imported as a separate record`. The Wells pane now shows two rows with the same name (merge is deliberately NOT automatic). Covers REVIEW.md §Round 4 — AUDIT safe-bucket ("LAS duplicate-name warning").
   **Automated coverage - pinned, with a residual (pile A):** that the duplicate warns and stays a separate record IS asserted. NOT asserted: the display surface - the status line and the History row. That part is still yours.

   **Result — T-IMP-02:**

- [x] Pass
- [ ] Fail
- [ ] Blocked

**Notes:** i need to state that for every curve set or any data set that imported together, it should have defined "set name", refer how geolog or IP managed this. So later user can trace which curve set he wanna use, even its duplicate. And it better be accessed either in well panes (each well can be expanded to see curve set and curve as well that it has), or in database.

**Update 2026-07-30 — BUILT (import sets).** Import LAS now opens a **curve set** dialog first:
the set name is suggested from what the filenames share (`blso*_lapi2023_fprooh.las` → `FPROOH`),
and **Attach to existing wells** (default ON) makes a re-delivery of a well already in the
project land as a NEW SET on that one record instead of a duplicate well row. A set name
already used on a well is auto-suffixed (`FPROOH` → `FPROOH_1`) — an import never overwrites
an earlier delivery. The Wells pane row now has a **▸ twisty** that expands into
well → sets → curves (the Geolog tree), lazily loaded per well. Curve resolution is unchanged
for existing projects: **set RAW keeps absolute priority**, and only a mnemonic RAW does not
carry is looked up in the attached sets. Re-test this case with `01. Final Log`'s RAW +
FPROOH + MULTIMIN folders for the same well.

### T-IMP-03 — Malformed LAS: duplicated depth section imports with a dropped-rows warning

**Tool/panel:** Import LAS… (parsers.rs `sanitize_curve_columns`/`sanitize_las_frame`, ingest.rs)
**Preconditions:** copy a good LAS to `dup_depth.las`; in a text editor duplicate a block of ~20 data lines in the `~A` section (repeat depths).
**Steps:**

1. Data tab → **Import Logs ▾** → **Import LAS…** → pick `dup_depth.las`.
2. Check status, History, then open the well in a Log View.
   **Expected:** Import SUCCEEDS (`Imported 1/1 well(s). 1 well(s) had depth issues.`) — never a silent partial well or a raw PK-constraint error. History carries the per-well warning `dropped 0 row(s) with missing depth and 20 with duplicate depth` (first occurrence kept). Log View shows continuous curves with no doubled interval. Covers REVIEW.md §P0 senior-audit backlog ("LAS import survives duplicate/odd-depth files on BOTH stores").
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-IMP-03:**

- [ ] Pass
- [ ] Fail
- [x] Blocked

**Notes:** i dont understand this part, where do u provide dup_depth.las?

**Update 2026-07-30 — FIXED.** Fair complaint: the step asked you to doctor a file yourself
and never said so. The file now EXISTS: **`dataset for test/examples/bad_dup_depth.las`**
(40 rows, of which rows 10–14 repeat row 9's depth). Import it and expect 35 rows plus a
dropped-duplicates warning. A cargo test asserts exactly that, so the exemplar and the app
can't drift apart.

### T-IMP-04 — Malformed LAS: all-null depth column and truncated last row → clean error, no orphan well

**Tool/panel:** Import LAS… (parsers.rs, ingest.rs ~83)
**Preconditions:** two doctored copies of a good LAS: (a) `null_depth.las` — every depth value in `~A` replaced with −999.25; (b) `truncated.las` — the last data line cut mid-row (delete the trailing half of the line).
**Steps:**

1. Note the well count in the Wells pane.
2. Data tab → **Import Logs ▾** → **Import LAS…** → pick `null_depth.las`.
3. Repeat with `truncated.las`.
   **Expected:** Each import ends with status `Imported 0/1 well(s).` and the Wells pane count is UNCHANGED — no empty orphan well, no partial curves in the Curve Catalog. (Backend errors: "no importable rows: N had missing depth…" and "ASCII data ended with N leftover token(s)… truncated or corrupt LAS?".) Covers REVIEW.md §P0 senior-audit backlog (all-null depth errors cleanly) and §Low-tier correctness & data-integrity sweep (truncated-row loud failure).
   **Automated coverage - pinned (pile B, 2026-07-31):** `malformed_las_exemplars_fail_the_documented_way` (already existed) plus `a_truncated_las_refuses_rather_than_importing_what_survived` (example_data_test.rs). Your Blocked mark is now answerable - there was no truncated exemplar to import, so `dataset for test/examples/bad_truncated.las` was added to the generator.

   **Result — T-IMP-04:**

- [ ] Pass
- [ ] Fail
- [x] Blocked

**Notes:** i dont understand this part, where do u provide null_depth.las?

**Update 2026-07-30 — FIXED.** Same fix: **`dataset for test/examples/bad_null_depth.las`**
(every depth cell is −999.25). Import it and expect a clean error with NO well row created —
check the Wells pane and the DB Inspector for a stray `SANDI-BAD-NULL`. Asserted by cargo test.

**Update 2026-07-31 — the OTHER half now has a file too.** Your Blocked note was right about
both files, and only one of them had been supplied. The truncated case now ships as
**`dataset for test/examples/bad_truncated.las`** (last data row cut mid-line: depth and GR
present, RHOB missing, no trailing newline — what a half-written file actually looks like).
Import it and expect a clean error naming *"leftover token(s) … truncated or corrupt LAS?"*,
with **nothing imported — not even the 39 intact rows** and no stray `SANDI-BAD-TRUNC`. Pinned
by `a_truncated_las_refuses_rather_than_importing_what_survived`. Both halves of this test are
now doable from the shipped examples folder; nothing needs doctoring by hand.

### T-IMP-05 — No-well-selected guards and cancel mid-dialog leave no side-effects

**Tool/panel:** all Data-tab importers (ribbon.ts guard clauses)
**Preconditions:** a project with wells; note the History entry count.
**Steps:**

1. In the **Wells** pane click empty space / ensure NO well is selected (restart selection by switching project tab if needed).
2. Try **Import DLIS…**, **Import Core…**, **Import SCAL…**, **Import Aux…**, **Import Deviation…**, **Export LAS…**, and Tools ▾ → **Shift Core…** — each without a selected well.
3. Select a well, open **Import LAS…**, then press **Cancel** in the file dialog. Repeat Cancel for **Import Tops…** and **Import Deviation…**.
   **Expected:** Steps 2: every tool refuses with status `Select a well first (Wells & Tops panel)` — no dialog opens, no History entry. Step 3: cancelling the file picker returns silently — no status change, no History entry, no data change (open plots do not refresh).
   **Result — T-IMP-05:**

- [ ] Pass
- [x] Fail
- [ ] Blocked

**Notes:** Import core, scal does nothing when i didnt select any wells, but for tops its opened

**Update 2026-07-30 — EXPLAINED, watch the status bar.** Core/SCAL without a well selected
do refuse with a message — but it goes to the STATUS BAR (bottom left), which is easy to
miss; nothing "does nothing" silently. Re-test watching the status bar: it should read
`Select a well first (Wells & Tops panel)`. **Tops opening with no well selected is by
design**: a tops file with a WELL column routes every row to its own well by name (your
multi-well Petrel exports), so no selection is needed — a single-well file without that
column falls back to the selected well and refuses only then. If you'd rather the guard were
louder than a status-bar line (e.g. a dialog), say so and it becomes a small UX increment.

### T-IMP-06 — DLIS import: sentinels screened, re-import replaced-count, LAS-mnemonic collision

**Tool/panel:** Import DLIS… (ribbon.ts `handleImportDlis`, src-tauri/src/dlis.rs)
**Preconditions:** a well imported from LAS (has GR etc.) selected in the Wells pane; a real .dlis for the same logging suite.
**Steps:**

1. Data tab → **Import Logs ▾** → **Import DLIS…** → pick the .dlis (status shows `Importing DLIS into <well>… (dlisio may take a moment)`).
2. Open **Curve Catalog**: check the new RAW rows (each frame gets its own run number) and their min/max.
3. Re-import the SAME .dlis into the same well.
4. Open a module dialog (e.g. Petrophysics → VSH — Gamma Ray) and check which GR the input dropdown resolves.
   **Expected:** Step 1: `Imported N curve(s), M samples into <well>.` + History entry. Step 2: no curve min/max shows −999.25/−9999 or |v|>1e30 — sentinels are screened to missing; curves read physically (RHOB ~1.9–2.9 g/cc, NPHI ~0–0.6). Step 3: status now appends `(replaced N existing curve(s))` — covers REVIEW.md §P0 senior-audit backlog ("DLIS null sentinels + no silent overwrite"). Step 4: modules still resolve the original LAS curve, not the DLIS run.
   **Known issue:** AUDIT-2026-07-21 §DLIS import #2 — "The 'no silent overwrite' collision check only catches a re-import of the identical DLIS file — the far more common case (a DLIS curve reusing a mnemonic already present in the well from LAS/standard_curves at run_no NULL) is never flagged, and the shadowed DLIS curve becomes permanently invisible to every module/equation with zero indication to the user." Expect step 1 to report a clean unqualified success even when the DLIS carries GR/NPHI/RHOB names the well already has, and the DLIS copies (visible as `run N` rows in the Curve Catalog) to be unreachable in step 4. Log as known, not new.
   **Automated coverage - none, and there will not be any (regraded to pile D, 2026-07-31):** DLIS is a binary vendor format and there is no honest way to synthesise a fixture that exercises sentinel screening and mnemonic collision. `dlis.rs::import_real_dlis` exists but is ignored behind the `SANDIBUMI_TEST_DLIS` environment variable. Point that at one of your own .dlis files and `cargo test -- --ignored` runs it. Nothing automated can retire this one.

   **Result — T-IMP-06:**

- [ ] Pass
- [x] Fail
- [ ] Blocked

**Notes:** dlis imported (processing history showed it) but well not showing, and duplicate logs should be also imported, we dont always know what inside, refer to T-IMP-02 to discuss how curve set or any data set managed.

**Update 2026-07-30 — PARTLY BUILT.** The duplicate half is done: Import DLIS now asks for a
**set name** first. Give the second tape its own name and both are KEPT (auto-suffixed
`WIRE` → `WIRE_1`); `replaced` can only be non-zero when you leave it as RAW, which preserves
the old replace-by-(mnemonic, run) behaviour. Both sets are then visible under the well's
▸ twisty in the Wells pane, so "we don't always know what's inside" is now inspectable after
import rather than guessed before it. **Still open:** "well not showing" — retest with the
set tree in place and tell me exactly what you see; if the well row itself is missing that is
a separate bug from the curves, and I'll need the well name to chase it.

### T-IMP-07 — Core CSV import: plugs off the log grid overlay at native depths

**Tool/panel:** Import Core… (ribbon.ts `handleImportCore`, parsers.rs `parse_core_csv`)
**Preconditions:** a well with RHOB/NPHI + a computed PHIT/PHIE; an RCAL CSV with headers like `DEPTH,CPOR,CPERM,CGD,CSW` (aliases PORO/PERM/KAIR etc. accepted) whose plug depths do NOT coincide with the 0.1524 m log grid, porosity in % is fine (auto-converts to v/v).
**Steps:**

1. Select the well → Data tab → **Import Data ▾** → **Import Core…** → pick the CSV.
2. Open a **Log View** on the well with a PHIT/PHIE (or RHOB) track displayed.
3. Open a **Crossplot** pane, set X/Y to NPHI vs RHOB (or PHIE vs PERM) and switch ON the **Core data** overlay toggle in plot properties.
4. Data → **DB Inspector** → table **Core Data**.
   **Expected:** Status: `Imported N core sample(s) for <well>.` + History entry `Imported N core sample(s) ← <path>`. Log View draws core points as markers over the matching track (CPOR over PHIT/PHIE/NPHI, CGD over RHOB, CPERM over PERM) at their NATIVE plug depths — between log samples, not snapped to the grid. Crossplot shows the core diamonds; CPOR should sit near log PHIT in clean sand (within ~2–3 p.u.), CGD ~2.6–2.7 g/cc for quartzose Mahakam sands. DB Inspector shows the rows with cpor/csw as fractions (0–1).
   **Result — T-IMP-07:**

- [ ] Pass
- [x] Fail
- [ ] Blocked

**Notes:** Core imported, but it should detect well name of core imported from the data inside, and for other properties / point curve it should be confirmed first (what unit it is, is it float, alpha, real, etc type of data), and name of each properties / point curve should be confirmed as well in the beginning. Imagine managing hundred wells that have cores. And should be worket either it comes from 1 csv or multiple csv, or even .txt or tab delimited data

**Update 2026-07-30 — BUILT (core import v2 wizard).** Import Core is now probe → CONFIRM →
commit. The dialog shows what was detected and lets you fix every piece before anything is
written: the **well-name column** (WN / WELL NAME / WELL…, detected from the data — rows
route to project wells by name, no well selection needed; a numeric pad-number column loses
to a textual name column when both exist), the **depth column and its unit** (units row
`FEET/M` and header suffixes like `MD (ft.)` are read; converted to the project unit — the
silent 3.28× trap), the **property columns** (CPOR/CPERM/CGD/CSW with each column's sniffed
type shown: number/text/empty), and **percent detection** ("CPOR reads as percent → divided
to v/v"). Multi-select works (one CSV per well, the BLSO shape) — the mapping is confirmed
once by header NAME and re-applied per file; `.txt`/tab/semicolon/whitespace delimiters are
auto-sniffed. Unmatched and ambiguous well names are REPORTED and skipped, never guessed.
Try it on `dataset for test/examples/core_rcal_multiwell.csv` (whole field in one file),
your real `03. Core Logs` folder (multi-select all 321), and the parent folder's Duri
`Core.csv` (WELL NAME beside a numeric WELL column). **Per Jauhar's note: BLSO is only an
exemplar, not the spec — the importer must take ANY delimited text with mixed column types
(alpha/integer/real/…). Delimiter sniffing + per-column type detection are in.**

**Update 2026-07-30 (b) — the EXTRA columns import too.** The requirement above is now
complete. `core_data` has four measurement slots; everything else in the file (LITH text,
So, Kv/Kh, sample IDs, tape names) can be carried from the same dialog as **point data at
the plug depths**: tick "Extra columns", untick what you don't want, name the dataset
(default `CORE`). Each cell is typed on its own — numeric cells become numbers, everything
else stays text — so a column that mixes `12.5` with `below detection` survives intact.
Values are stored VERBATIM (no percent or unit conversion is applied to extras; the depth
they hang on IS converted). A column claimed by a core role can never also be an extra.
Re-import replaces per (well, dataset), same discipline as the plugs. Check the result in
DB Inspector → `aux_data`.

### T-IMP-08 — Core CSV with a duplicated plug depth imports (first kept), never aborts

**Tool/panel:** Import Core… (parsers.rs `parse_core_csv` dedup, db.rs `insert_core_data`)
**Preconditions:** copy the T-IMP-07 CSV, duplicate one data row (same depth twice).
**Steps:**

1. Select the well → **Import Data ▾** → **Import Core…** → pick the doctored CSV.
2. Check DB Inspector → **Core Data** row count.
   **Expected:** Import SUCCEEDS with `Imported N core sample(s)` where N = file rows − 1 (first occurrence of the repeated depth wins) — NOT a raw `PRIMARY KEY or UNIQUE constraint violation` error with 0 rows. Re-import replaces (row count stays N, not doubled). Covers REVIEW.md §Round 4 — import-robustness batch 2, fix (1).
   **Automated coverage - pinned (pile B, 2026-07-31):** `a_repeated_plug_depth_is_dropped_not_a_failed_import` (parsers.rs) - first occurrence wins, companion columns follow the kept row, and the import never aborts.

   **Result — T-IMP-08:**

- [ ] Pass
- [x] Fail
- [ ] Blocked

**Notes:** refer T-IMP-06 and T-IMP-02 about how duplicated data managed

**Update 2026-07-30 — REBUILT as core SETS (your note is the spec).** Duplicated data is
now managed the way T-IMP-02 manages curves: **one delivery = one named core set**, and an
import NEVER overwrites an earlier one. Import Core asks for a **Core set** name (suggested
from the filename — `blso00025_lapi2023_rcal.csv` → `RCAL`); a well that already carries
that name gets the new delivery suffixed (`RCAL` → `RCAL_1`), reported in the status line.
The imported set becomes that well's **active** core.

Unlike curve sets, core sets do NOT merge: two deliveries measure the same plugs, so
**exactly one set is active per well** and every reader follows it — log overlay, φ-k
crossplots, HFU, SandiMin core calibration, Shift Core, DB Inspector edits. Switch or delete
deliveries in **Data → Tools ▾ → Data Sets…** (● marks the live one, with plug
count, source file and import date).

Duplicated depths WITHIN one delivery still drop first-kept — that is a broken row inside a
single file, not a second delivery. Re-test both: (1) doctored CSV with a repeated depth →
imports with the dropped-row note; (2) import the same real file twice → two sets, both
kept, the newest live, and the plug count in any plot does NOT double.

**Update 2026-07-30 (b) — the rule is UNIVERSAL, not core-only.** Per Jauhar: *any kind of
point data should behave like core — XRD, CEC, oil show, etc.* Every `aux_data` dataset
(petrography, XRD, CEC, oil show, perforations, core extras, any custom name) now versions
the same way: **Import Aux… takes a Set name**, a re-delivery is auto-suffixed and becomes
live, and **one set per (well, dataset)** is read — panel counts follow the active delivery,
never the sum. Datasets are independent: switching XRD leaves CEC and oil show alone. Core
EXTRAS are stored under the core set's own name, so a core switch carries them. **SCAL Pc
follows the same rule** — the files selected together in one Import SCAL are one named
delivery, and only the live one feeds Pc QC, the Leverett-J fit and Thomeer. Browse and
switch everything from the **Wells pane ▸ twisty** (Core / SCAL / Surveys / Point data,
● = live, double-click to switch) or **Data → Tools ▾ → Data Sets…** (four sections).

### T-IMP-09 — Shift Core: constant core-to-log shift, undo, invalid input rejected

**Tool/panel:** Tools ▾ → Shift Core… (ribbon.ts `handleShiftCore`)
**Preconditions:** T-IMP-07 done; Log View open showing core overlay.
**Steps:**

1. Data tab → **Tools ▾** → **Shift Core…**.
2. In the `Shift Core — <well>` dialog leave **Shift (m)** empty → click **Apply Shift**.
3. Enter `2.5` → **Apply Shift**.
4. Press **Ctrl+Z**.
   **Expected:** Step 2: status `Enter a non-zero shift in metres`, dialog stays open, nothing written. Step 3: status `Shifted N core plug(s) of <well> by +2.5 m`, History entry `Edit — Core shift +2.5 m (N plugs)`, and the open Log View's core points visibly move 2.5 m deeper immediately. Step 4: points move back (undo re-shifts −2.5 m, with its own status/History trace).
   **Result — T-IMP-09:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:** we should have resolve previous problem to do this

**Update 2026-07-30 — UNBLOCKED.** T-IMP-07 is resolved (core import v2 wizard), so this
test can run as written. Note the shift is per WELL: after a multi-well core import,
Shift Core still applies to the selected well's plugs only.

### T-IMP-10 — Tops CSV: multi-well WELL column, single-well file, unmatched + blank WELL cells

**Tool/panel:** Import Tops… (ribbon.ts `handleImportTops`, ingest.rs `import_tops_file`)
**Preconditions:** ≥3 wells in project. Prepare: (a) multi-well CSV `WELL,TOP,DEPTH` covering 2 project wells plus one bogus well name and ONE row with a blank WELL cell; (b) single-well CSV `TOP,DEPTH` (no WELL column).
**Steps:**

1. Select well A → Data tab → **Import Data ▾** → **Import Tops…** → pick file (a).
2. Check the **Tops** pane / Wells & Tops tree for each well; check well A did NOT receive the blank-cell top.
3. With well B selected → **Import Tops…** → file (b).
4. Open a Log View on well B.
   **Expected:** Step 1: status `Tops: N marker(s) across 2 well(s) — unmatched well name(s): <bogus>`; History entry; tops appear immediately under the matched wells and as lines in open log views (dataVersion refresh). The blank-WELL row is SKIPPED, not routed to the selected well (covers REVIEW.md §Round 4 — import-robustness batch 2, fix (5)). Step 3: all tops land in well B only. Step 4: tops lines drawn at the right depths. Covers REVIEW.md §P2-a — Tops-style imports.
   **Automated coverage - pinned (pile B, 2026-07-31):** `tops_import_multiwell_and_default` (already existed) plus `a_blank_well_cell_is_skipped_rather_than_charged_to_the_selected_well` (ingest.rs).

   **Result — T-IMP-10:**

- [ ] Pass
- [x] Fail
- [ ] Blocked

**Notes:** same for core import, it should auto detect well names, either it comes from 1 csv or multiple csv

**Update 2026-07-30 — DONE for core.** Core import now auto-detects the well-name column
and routes rows exactly the way tops always did (this test's own routing rules), including
multi-select of many files and .txt/tab delimiters. See the T-IMP-07 update.

### T-IMP-11 — Aux data import: PERFORATION and XRD land per-well, replace on re-import

**Tool/panel:** Import Aux… (ribbon.ts `handleImportAux`)
**Preconditions:** a selected well; a tops-style CSV with `TOP` (or DEPTH) + `BASE` columns and value columns (e.g. perf interval status, or XRD mineral %).
**Steps:**

1. Data tab → **Import Data ▾** → **Import Aux…**.
2. In `Import Aux Data — <well>`: Dataset = **PERFORATION** → **Choose file & import…** → pick the perf CSV.
3. Repeat with Dataset = **XRD** and the XRD CSV. Also try Dataset = **Custom…** with an empty name → Choose file.
4. Data → **DB Inspector** → table **Aux Data**.
   **Expected:** Step 2/3: result box `Imported N value(s) across M column(s): <names>`; status + History entry per import. Step 3 empty-Custom: refused with `Enter a dataset name.` (no file dialog). Step 4: rows visible per well/dataset (read-only). Re-importing the same dataset replaces that dataset's rows only (count stays, other datasets untouched). XRD quartz+clay+carbonate percentages should sum to ~100%. Covers REVIEW.md §P2-a — Tops-style imports (Aux item).
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-IMP-11:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:** same for core import, it should auto detect well names, either it comes from 1 csv or multiple csv

**Update 2026-07-30 — BUILT (aux routing).** Import Aux now honors a WELL column: rows
route to each named project well (unmatched/ambiguous names and blank cells reported,
never guessed); a file without one binds to the selected well as before. The result box
says where the rows went. Try `dataset for test/examples/xrd_multiwell.txt` (tab-delimited,
3 wells in one file). Mixed value types per column (numbers AND text) were already stored
correctly (value_num vs value_text per cell).

### T-IMP-12 — Deviation survey import: TVD/TVDSS computed; duplicate-MD survives; TVD not yet consumable

**Tool/panel:** Import Deviation… (ribbon.ts `handleImportDeviation`, parsers.rs `parse_deviation_csv`, deviation.rs)
**Preconditions:** a deviated well selected; survey CSV with `MD,INC,AZI` headers (aliases INCL/AZIM fine); well KB known.
**Steps:**

1. Data tab → **Import Data ▾** → **Import Deviation…** → pick the CSV.
2. In `Import Deviation — <well>`: enter Datum / KB (m) = your KB (e.g. `25`) → **Import Survey**.
3. Data → **SQL Query** → run `SELECT md, tvd, tvdss FROM well_path WHERE well_id = (SELECT well_id FROM wells WHERE well_name = '<well>') ORDER BY md`.
4. Duplicate one station row in the CSV and re-import.
5. Cross-check consumption: open the **SW — Saturation-Height** module dialog and look at its TVD input.
   **Expected:** Step 2: status `Imported N survey station(s); TVD/TVDSS computed for <well>.` + History entry; dialog closes. Step 3: TVD ≤ MD everywhere (equal only in the vertical section), monotonically increasing, TVDSS = TVD − datum; at 30° inclination TVD grows ~0.866 m per m MD — sanity-check one station by hand. Step 4: import still succeeds with the duplicate MD dropped (first kept) — covers REVIEW.md §Round 4 — import-robustness batch 2, fix (2).
   **Known issue:** AUDIT-2026-07-21 §Importers B #1 — "Deviation-survey TVD/TVDSS is computed and stored, but no code path ever exposes it as a fetchable curve — sw_height's 'TVD' input (the P0 fix's whole point) is permanently unreachable for any well relying on Import Deviation, and the module dialog silently pre-selects it as if it worked." Expect step 5 to show "TVD" pre-selected even though no TVD curve exists; a run will silently fall back to MD per sample (SWH unchanged vs MD-based). `getWellPath` has no UI consumer yet. Log as known, not new.
   **Automated coverage - pinned (pile B, 2026-07-31):** `deviation_import_materializes_tvd_tvdss_curves` and `deviation_import_versions_surveys_and_switching_rebuilds_tvd` (both already existed) plus `a_repeated_survey_station_is_dropped_not_a_failed_survey` (parsers.rs).

   **Result — T-IMP-12:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:** refer T-IMP-02

**Update 2026-07-30 — surveys are VERSIONED too.** Same model as core sets: Import
Deviation asks for a **Survey name** (default `SURVEY`, auto-suffixed if taken), a second
import lands BESIDE the first instead of replacing it, and the new survey becomes active.
Only the active survey is read anywhere, and **switching one immediately recomputes
TVD/TVDSS from it** — a stale TVD would otherwise keep feeding every height calculation the
geometry you just switched away from. Manage them in **Data → Tools ▾ → Core Sets &
Surveys…**; the row shows station count, datum, source file and import date. Deleting the
active survey hands over to the next newest and rebuilds TVD from it. Worth testing with a
preliminary vs definitive survey on the same well: TVD at TD should visibly change when you
switch, and change back.

### T-IMP-13 — Well locations import → wells post on the Field Map at UTM coordinates

**Tool/panel:** Import Well Locations… (ribbon.ts `handleImportWellLocations`, mapPanel.ts)
**Preconditions:** ≥3 wells; CSV `WELL,EASTING,NORTHING` with real UTM 50S coordinates (Mahakam: easting ~4–5×10⁵, northing ~9.9×10⁶), one row with a blank WELL cell; **Field Map** pane already open.
**Steps:**

1. Data tab → **Import Data ▾** → **Import Well Locations…**.
2. In the dialog: **Default UTM zone** = `UTM 50S` (the default) → **Choose file & import…** → pick the CSV.
3. Check the Field Map.
4. Tools ▾ → **Well Header…** on one located well.
   **Expected:** Step 2: result box `Located N well(s)… Open Field Map to view.` (blank-WELL row skipped and reported, no unrelated well relocated); status + History entry. Step 3: the already-open map auto-fits and posts the wells at the correct relative geometry (inter-well distances/bearing match the field layout). Step 4: Surface X / Surface Y / UTM zone fields show the imported values (not blank); changing only TD and clicking **Save Header** preserves the coordinates. Covers REVIEW.md §Field Map — well surface coordinates (first four checklist items).
   **Result — T-IMP-13:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-IMP-14 — SCAL Pc/Sw import: multi-file, auto-detect, Leverett-J fit reported

**Tool/panel:** Import SCAL… (ribbon.ts `handleImportScal`)
**Preconditions:** a well with core perm/poro context; your W-MND-1-style SCAL exports — a porous-plate wide table and/or per-plug centrifuge CSVs.
**Steps:**

1. Select the well → Data tab → **Import Data ▾** → **Import SCAL…** → multi-select the files (ONE lab fluid system per import).
2. In `Import SCAL — <well>`: File format = **Auto-detect per file**; Fluid system = **Air-brine (72)** (sigma·cosθ auto-fills 72) → **Import & Fit**.
3. Negative: reopen, set Fluid system = **Other / custom** (sigma field clears), leave it blank → **Import & Fit**.
4. Re-import the same files and compare point counts.
   **Expected:** Step 2: result box `Imported N Pc point(s). J-fit: A = …, B = …, R² = … (n points). Enter these as SWH_A/SWH_B in SW — Saturation-Height.` (or the honest `Too few valid points…` message if plugs lack perm/poro); status + History entry (`Imported SCAL Pc data (auto) ← <path>`). B should be negative (Sw falls as J rises) and R² > ~0.7 for a consistent rock family. Step 3: refused with `Lab sigma·cosθ must be a positive number.` — nothing imported. Step 4: points REPLACE (count unchanged), never append; a zero-point parse refuses the replace-write instead of wiping existing data. Covers REVIEW.md §Round 3 — (8) increment 2 — SCAL importers (incl. the post-review hardening items).
   **Automated coverage - pinned, with a residual (pile A):** multi-file import, auto-detect, the Leverett-J fit and the zero-row refusal ARE asserted. NOT asserted: the sigma guard, which is a frontend string rather than backend arithmetic.

   **Result — T-IMP-14:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-IMP-15 — LAS export: NaN→−999.25, computed curves included, mixed-case name exports real values

**Tool/panel:** Export LAS… (ribbon.ts `handleExport`, src-tauri/src/export.rs)
**Preconditions:** a well with standard curves + at least one computed curve; create one equation output with a mixed-case name (e.g. `Vsh_final`) via the Inspector's equation editor and run it.
**Steps:**

1. Select the well → Data tab → **Export LAS…** → accept the default filename `<well>.las` → Save.
2. Open the exported file in a text editor.
   **Expected:** Status `Exported <well> (N rows) to <path>`; History entry `Export — Exported LAS (N rows) → <path>`. In the file: `NULL. −999.25` declared in `~W`; every gap in the source curves written as −999.25; header lists GR/RES_DEEP/NPHI/RHOB/DT/SP plus each computed curve; the `Vsh_final` column carries REAL values (0–1, high in shale), NOT −999.25 at every depth. Covers REVIEW.md §Round 4 — backend robustness batch 1, fix (9) (mixed-case export column).
   **Automated coverage - pinned (pile B, 2026-07-31):** `export_writes_missing_as_null_and_carries_mixed_case_computed_curves` (export.rs).

   **Result — T-IMP-15:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-IMP-16 — Round-trip: export → fresh project → re-import → values identical

**Tool/panel:** Export LAS… + Import LAS… (export.rs ↔ parsers.rs/ingest.rs)
**Preconditions:** T-IMP-15's exported `<well>.las` on disk; note 3 spot values in the source well (e.g. GR, RHOB, VSH_final at one shale and one sand depth) via Log View readout or SQL Query.
**Steps:**

1. Project tab → **New Project…** → create a scratch project (workspace switches to it).
2. Data tab → **Import Logs ▾** → **Import LAS…** → pick the exported file.
3. Open a Log View; read the same 3 depths. Optionally Data → **SQL Query**: `SELECT depth, gr, rhob FROM standard_curves WHERE depth BETWEEN <d1> AND <d2>`.
4. Reopen the original project (Project → **Recent ▾**) and confirm it is untouched.
   **Expected:** Import succeeds `Imported 1/1 well(s).`; the well carries the full curve set including `VSH_FINAL`-named computed curve (re-imported as a RAW curve — provenance is now "imported", visible in the Curve Catalog set/run columns). Spot values match the source to LAS precision (4 decimals); −999.25 rows come back as gaps, not as spikes of −999. Depth range and sample count match the source well's export row count N.
   **Automated coverage - pinned (pile B, 2026-07-31):** `an_exported_las_reimports_with_the_same_values` (export.rs).

   **Result — T-IMP-16:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-IMP-17 — Cross-checks: open panes refresh on import (dataVersion) and repaint on theme switch

**Tool/panel:** whole Data tab + workspace (state.ts dataVersion, theme.ts)
**Preconditions:** project with wells; a spare LAS file not yet imported.
**Steps:**

1. Open and arrange: **Log View**, **Curve Catalog** (Inspector), **Field Map**, **Processing History**.
2. Data tab → **Import Logs ▾** → **Import LAS…** → import the spare LAS.
3. Without clicking any refresh: check all four panes.
4. Open the **Import SCAL…** or **Import Aux…** dialog, then Project tab → **Theme** → switch (e.g. Default → Dark → Pertamina) with the dialog and panes open.
   **Expected:** Step 3: the new well appears in Wells and the Curve Catalog well/curve lists without manual action; History shows the entry at the top; open plots stay consistent (no stale curve lists in their curve pickers). Step 4: ribbon, panes, modal dialog, and plot canvases all repaint immediately in the new palette — no white-on-white text or unstyled dialog remnants (theme contract: all 15 CSS vars).
   **Result — T-IMP-17:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

# Section WELL — Wells, groups, tops & zones

### Cluster WELL — wells, groups, map, tops, zones

Shared preconditions: SandiBumi running via `npm run tauri dev` on a project with at least 5 real wells imported (LAS with GR + RHOB + NPHI), formation tops present in at least 3 wells, and no pending edits. Tests 07–09 additionally need surface coordinates (**Data ▸ Import Data ▾ ▸ Import Well Locations…**, e.g. UTM 50S for Mahakam). Keep the **Processing History** pane (**Project ▸ Monitor ▸ History**, or right-click a pane ＋ ▸ Processing History) open throughout — several tests cross-check it.

### T-WELL-01 — Object tree: click activates, 📌 pin mode drives the workspace

**Tool/panel:** Wells pane object tree (src/ui/objectTree.ts)
**Preconditions:** ≥5 wells imported; a Log View and a Histogram open.
**Steps:**

1. In the **Wells** pane, plain-click a well.
2. Confirm the 📌 button in the group bar is ON (highlighted), then click a second well.
3. Click 📌 to turn it OFF (status: "Pin OFF — only the active panel follows…"), activate the Log View tab, then click a third well.
4. Click the ☆ star left of a well name.
   **Expected:** (1–2) clicked well highlights, and with 📌 ON every open view (Log View, Histogram) reloads to it. (3) With 📌 OFF only the active Log View follows; the Histogram keeps the previous well. (4) Star turns ★, status "Well pinned — available as a run scope", and it survives an app restart (persisted). Covers REVIEW.md §Panes ("★ pin a well").
   **Automated coverage - end-to-end (pile C, 2026-08-01):** `wells.e2e.mjs` covers the plain-click
   activation (exactly one row carries `tree-selected`) and the ★ **favourite** pin — asserting the
   pin reached the project via `list_pinned_wells`, not merely that a class toggled, because a star
   that looks set and was never written gives a run scope that silently empties on the next launch.
   **Not covered:** the 📌 global well LOCK (a different control from the ★), panel titles following
   the selection, and ★ persistence across a relaunch.

   **Result — T-WELL-01:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-02 — Multi-select: Ctrl-click, Shift-click range, ⇄ invert, plain-click clear

**Tool/panel:** Wells pane object tree (src/ui/objectTree.ts)
**Preconditions:** ≥5 wells visible; a Log View open on well A.
**Steps:**

1. Ctrl-click wells B and C.
2. Verify the Log View still shows well A (active well must NOT move).
3. Shift-click a well 3 rows below the last Ctrl-click.
4. Click the **⇄** button in the group bar.
5. Plain-click any well.
   **Expected:** (1) B and C get the accent multi-select edge; header reads "Wells (N) • 2 selected"; status "2 wells selected — batch dialogs will pre-tick them". (2) Active well unchanged — Ctrl-click never fires well activation. (3) The whole range highlights. (4) Selection inverts within the visible list (previously selected wells clear, the rest select). (5) Multi-selection clears, status "Multi-selection cleared", and the clicked well activates. Covers REVIEW.md §Well scope.
   **Automated coverage - end-to-end (pile C, 2026-08-01):** `wells.e2e.mjs` drives all four
   gestures against the real pane. Two things it checks that a naive version would miss: ctrl-click
   is a **toggle**, so a second one must REMOVE (a test that only ever adds passes on an
   implementation that cannot remove); and ctrl-click must **not move the active well**, which is
   the entire point of the gesture, since every open view follows the active well. The shift range
   is deliberately two of three, so an implementation that selected everything visible fails rather
   than passes. Note for anyone extending it: `el.click()` cannot carry modifier keys, so ctrl- and
   shift-click must be dispatched as real `MouseEvent`s — written the obvious way a ctrl-click
   silently becomes a plain click and the test then measures a different gesture.

   **Result — T-WELL-02:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-03 — Multi-selection feeds a batch dialog's "Selection" scope, live

**Tool/panel:** Well scope selector in module pane (src/ui/wellScope.ts, src/ui/moduleDialog.ts)
**Preconditions:** Active group set to **All wells** (group bar dropdown); no wells pinned.
**Steps:**

1. Ctrl-click 3 wells in the Wells pane.
2. Petrophysics ▸ **VSH ▾** ▸ **VSH from Gamma Ray** — the module pane opens with a **Wells** scope row (Group / ★ Pinned / Selection / All / Custom…).
3. Note which scope button is active and the "N wells" count; hover the count.
4. With the pane still open, Ctrl-click a 4th well in the Wells pane.
5. Click **Custom…**.
   **Expected:** (2–3) **Selection** is the active scope, count "3 wells", hover tooltip lists exactly the 3 well names, hint reads "Running on the wells selected in the Wells pane (Ctrl/Shift-click)". (4) Count updates live to "4 wells" without reopening. (5) Custom opens a searchable checklist seeded with those 4 wells ticked. Covers REVIEW.md §Well scope.
   **Result — T-WELL-03:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-04 — Well Groups manager: create, edit membership, rename, delete

**Tool/panel:** Well Groups modal (src/ui/wellGroups.ts), opened from the Wells pane **⚙** button
**Preconditions:** ≥5 wells; no group named "UAT-North" yet.
**Steps:**

1. Click **⚙** in the Wells pane group bar — the "Well Groups" dialog opens with an "All wells" row.
2. Type "UAT-North" in "New group name…", click **Create**.
3. In the membership panel ("Wells in “UAT-North”"), tick 3 wells (try "Filter wells…", **Select all (shown)**, **Clear (shown)** too), click **Save membership**.
4. Double-click the group name, rename to "UAT-N" in the prompt.
5. Click **Delete** on the group; cancel the confirm; click Delete again and accept.
   **Expected:** (2) Group row appears, membership editor jumps straight to it. (3) Status "Group “UAT-North” now has 3 wells"; the row's wells count reads 3; live "3 selected" label tracks ticking. (4) Row shows "UAT-N". (5) First confirm cancels harmlessly; second removes the row; wells themselves are untouched (count in "All wells" unchanged).
   **Result — T-WELL-04:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-05 — Active group scopes the tree and freshly opened batch dialogs

**Tool/panel:** Group bar dropdown (src/ui/objectTree.ts) + module pane scope
**Preconditions:** Two groups exist with different membership (e.g. "North" 3 wells, "South" 2 wells).
**Steps:**

1. In the Wells pane dropdown pick "North (3)".
2. Check the tree header and the well list.
3. Petrophysics ▸ **VSH ▾** ▸ **VSH from Gamma Ray** (close it first if already open, via the pane's ✕).
4. Switch the dropdown back to **All wells**.
   **Expected:** (1) Status "Active well group: North (3 wells)". (2) Header "Wells — North (3)"; only member wells listed; empty groups would show "No wells in this group — Edit wells to add some". (3) The scope row defaults to **Group** with "North" selected in its group dropdown, count "3 wells". (4) Status "Well group cleared — showing all wells"; full list returns.
   **Result — T-WELL-05:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-06 — NEGATIVE: already-open batch dialog does NOT re-scope on group switch

**Tool/panel:** Any batch pane left open across a group switch (src/ui/wellScope.ts + src/ui/workflowDialog.ts)
**Preconditions:** Groups "North" and "South" exist; "North" active.
**Steps:**

1. Open Petrophysics ▸ **Workflow…** — its Wells scope shows Group "North".
2. Leaving it open, switch the Wells pane dropdown to "South".
3. Look at the Workflow pane's scope row: which group is selected, and what is the well count?
4. Also create a brand-new group via ⚙ and check whether it appears in the open pane's Group dropdown.
   **Expected (spec):** the open dialog's Group scope should follow the new active group (or at minimum offer it).
   **Known issue:** AUDIT-2026-07-21-full-qc.md, Substrate — well-group scoping sweep #1: "No batch-run dialog re-scopes to a new active well group while it's already open — only the Wells sidebar tree and Map pane react live to a group switch." Expect the pane to keep "North" and its stale membership/count, and new groups to be missing until the pane is closed and reopened. A run launched now would silently compute over the WRONG group — log as known, note which panes you tried.
   **Result — T-WELL-06:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-07 — Field Map smoke: markers, pan/zoom, Fit, info line, group ring, theme, auto-fit

**Tool/panel:** Field Map pane (src/ui/mapPanel.ts), Petrophysics ▸ **Field Map…**
**Preconditions:** Well locations imported for most wells; one group active.
**Steps:**

1. Open **Field Map…**. Read the toolbar info span.
2. Drag to pan; wheel-zoom at a marker; click **Fit**.
3. Check the active group's wells vs the rest.
4. Project tab ▸ **Theme** — switch theme, watch the map.
5. On a second project with NO coordinates, open Field Map first (empty-state message), then run **Data ▸ Import Data ▾ ▸ Import Well Locations…**.
   **Expected:** (1) Info reads "N located · UTM <zone>" (or "mixed zones (…)" with a warning tint if zones differ); markers labeled when ≤80 wells; scale bar bottom-left reads a sensible distance (well spacing in a Mahakam field ~ hundreds of m–km). (2) Zoom anchors under the cursor; Fit frames every well. (3) Active-group wells are ring-highlighted. (4) Map repaints immediately in the new palette — no interaction needed. (5) Empty state says "No wells have surface coordinates yet… Data ▸ Import Well Locations…"; after import the map fits automatically, no manual Fit. Covers REVIEW.md §Field Map bullets 1 and 4.
   **Result — T-WELL-07:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-08 — Draw polygon → assign enclosed wells to a NEW group

**Tool/panel:** Field Map pane (src/ui/mapPanel.ts)
**Preconditions:** T-WELL-07 passed; ≥3 wells located close together.
**Steps:**

1. Click **✏ Draw polygon** (status explains draw mode); click 4+ vertices around a cluster of wells; close by clicking the first vertex (or Enter).
2. Read the status and info line.
3. Click **Assign to group…**; in the "Assign wells to group" dialog keep Target = "＋ New group…", type "MAP-UAT" in Name, click **Assign**.
4. Check the Wells pane group dropdown and the Processing History pane.
   **Expected:** (1) Rubber-band line follows the cursor; on close the polygon fills faintly, enclosed markers enlarge/recolor. (2) Status "Polygon closed — N well(s) enclosed…"; info "…polygon encloses N"; **Assign to group…** enabled only when N > 0. (3) Dialog lists the enclosed well names; status "Created group “MAP-UAT” with N well(s)." (4) "MAP-UAT (N)" appears in the group dropdown and ⚙ manager; History gains a **Group** entry "Created group \"MAP-UAT\" from map polygon (N wells)". Covers REVIEW.md §Field Map bullet 5 and §Polish — Processing history (map-polygon→group).
   **Result — T-WELL-08:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-09 — Polygon editing, union into EXISTING group, Esc/Clear negatives

**Tool/panel:** Field Map pane (src/ui/mapPanel.ts) + modal Escape scoping (src/ui/modal.ts)
**Preconditions:** Group "MAP-UAT" from T-WELL-08 exists; polygon still on screen (or redraw one).
**Steps:**

1. Drag a square vertex handle outward so one more well falls inside.
2. Click **Assign to group…**, pick Target = "MAP-UAT (N)", click **Assign**.
3. Start **✏ Draw polygon** again, drop 2 vertices, press **Esc**.
4. Start drawing again, drop 3 vertices, then open any dialog (e.g. Data ▸ Tools ▾ ▸ **Well Header…**) and press **Escape** once.
5. Draw a small polygon enclosing NO wells; check **Assign to group…**; click **Clear**.
   **Expected:** (1) Enclosed set re-highlights live while dragging. (2) Status "Group “MAP-UAT” now has M well(s) (+k)" — a union, no duplicates; History gains a **Group** entry. (3) Esc cancels the half-drawn polygon. (4) Escape closes only the dialog; the in-progress polygon is still there (covers REVIEW.md §P1 "Dialog Escape is scoped to the dialog" map-polygon test). (5) With 0 enclosed, **Assign to group…** is disabled ("No wells inside the polygon." if forced); **Clear** removes the polygon and disables itself.
   **Result — T-WELL-09:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-10 — Tops pane: click a top windows plots to its interval

**Tool/panel:** Tops pane (src/ui/topsPanel.ts) + plot zone follower (src/ui/plotCommon.ts)
**Preconditions:** Active well has ≥3 tops; a Log View and a Histogram open on that well.
**Steps:**

1. Confirm **Tops** is its own dock panel below **Wells** (drag/resize it once).
2. Click a mid-well top row (color chip + name + depth).
3. Check the Log View and the Histogram's zone dropdown.
4. Click the same top again.
5. Select a well with no tops.
   **Expected:** (1) Separate resizable "Tops" panel — covers REVIEW.md §Panes ("Tops is its own pane now"). (2) Row highlights; Log View scrolls so that top's depth is at view top; the interval runs down to the next top (or TD). (3) Plot zone selector shows/offers "Top <NAME> (<top>–<next|TD>)" and the histogram recomputes over only that interval (sample count drops accordingly). (4) Deselects; plots return to the full logged interval. (5) Pane shows "No tops for this well"; with no well selected, "Select a well".
   **Result — T-WELL-10:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-11 — Tops editor: 🏷 add a top, undo, History, cross-pane sync

**Tool/panel:** Tops overlay editor in the Log View toolbar (src/ui/topsEditor.ts, src/ui/logViewPanel.ts)
**Preconditions:** Log View open on a well with GR displayed; Tops pane visible.
**Steps:**

1. In the Log View toolbar click **🏷** ("Edit tops: click to add, drag to move, double-click to rename/delete").
2. Click an empty depth at a clear GR break — the "New top" dialog opens (Name / Depth / Color, **Add top**).
3. Enter a lowercase name (e.g. "top_uat_a"), adjust Depth, pick a color, press Enter or **Add top**.
4. Check Tops pane, a second Log View on the same well, and Processing History.
5. Press **Ctrl+Z**.
6. Try adding a top with a blank name, then with a non-numeric depth.
   **Expected:** (3) Name auto-uppercases to TOP_UAT_A; status "Added top TOP_UAT_A at <d>"; a labeled colored line appears across all tracks and tracks pan/zoom. (4) Top appears immediately in the Tops pane and every other view of that well; History gains a **Tops** entry. (5) Status "Undo: add top TOP_UAT_A"; line and pane row vanish everywhere. (6) Both rejected with status "Top needs a name and a numeric depth" — dialog stays open, nothing written. Covers REVIEW.md §P2-b ("Tops lines in the log view", "🏷 edit mode").
   **Result — T-WELL-11:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-12 — Tops editor: drag-move, edit/rename/recolor/delete, stratigraphic crossing warning

**Tool/panel:** Tops overlay editor (src/ui/topsEditor.ts), backend check_top_order
**Preconditions:** ≥3 wells share the same two tops (e.g. TOP_A above TOP_B everywhere); 🏷 edit mode ON.
**Steps:**

1. Drag a top line ~10 m (dashed preview while dragging), release.
2. Press Ctrl+Z, verify it returns, then Ctrl+Y.
3. Double-click a top line — "Edit top — <NAME>" dialog: rename it, change color, **Save**. Ctrl+Z.
4. Double-click again, click **Delete**. Ctrl+Z.
5. Now drag TOP_B ABOVE TOP_A in this well only, watch the status bar.
6. Verify wheel zoom still works with edit mode on; toggle **🖍** and confirm 🏷 switches off.
   **Expected:** (1) Status "<NAME>: <old> → <new>"; Tops pane depth updates. (2–4) Every operation undoable/redoable with named status messages; History logs **Tops** entries for edit and delete. (5) A ⚠ crossing warning appears in the status bar naming the reversed pair against the majority vote of the other wells (e.g. "below it in 4 of 5 other wells"). (6) 🏷 and 🖍 are mutually exclusive. Covers REVIEW.md §P2-b ("Crossing warnings").
   **Result — T-WELL-12:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-13 — Top autocorrelation: propagate by GR shape, untick, apply, batch undo

**Tool/panel:** Autocorrelate Tops pane (src/ui/autoCorrDialog.ts), Data ▸ **Tools ▾** ▸ **Autocorrelate Tops…**
**Preconditions:** Source well active with a confidently hand-picked marker (e.g. an MFS on GR); ≥3 target wells with GR in the active group.
**Steps:**

1. Open **Autocorrelate Tops…**. Verify "Source well" shows the active well.
2. Pick the marker in **Top**; leave **Log** = GR, **Window ± (m)** = 10, **Search ± (m)** = 25.
3. Click **Correlate N wells**.
4. Review the proposals table (Well / Current / Proposed / r): untick one strong match.
5. Click **Apply k picks**.
6. Check target wells' Tops panes + History, then press **Ctrl+Z** once.
   **Expected:** (3) Button reads "Correlating…" then restores. (4) Rows with r ≥ 0.70 come pre-ticked; weak matches are dimmed and unticked; failed wells show "—" and a per-row error, checkbox disabled; the Apply label tracks the tick count. (5) Status "<TOP> picked in k well(s) by autocorrelation"; the top appears at the proposed depth in each ticked well (within your ±25 m search window of your own hand pick — judge r against the deltaic GR character). History gains "Tops — Autocorrelated <TOP> into k well(s)". (6) ONE undo reverts the whole batch (restores prior depths, deletes where none existed). Covers REVIEW.md §P2-b ("Autocorrelate… (Data tab)").
   **Result — T-WELL-13:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-14 — Autocorrelation negatives: no tops, bad curve, no targets

**Tool/panel:** Autocorrelate Tops pane (src/ui/autoCorrDialog.ts)
**Preconditions:** One well with zero tops; one single-well group.
**Steps:**

1. Activate the topless well, open **Autocorrelate Tops…**.
2. Back on a well WITH tops, open the pane, set **Log** to a curve that exists nowhere (e.g. "XXNOPE"), click **Correlate N wells**.
3. Activate the single-well group (source is its only member), reopen the pane.
   **Expected:** (1) Message pane: "No tops picked in <well> yet — pick one in the log view first (🏷)" — no crash. (2) A backend error is surfaced in the results area and status ("Autocorrelate: …"); button re-enables; NO tops are written and History gains nothing. (3) Message "No other wells (in the active group) to correlate to".
   **Result — T-WELL-14:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-15 — Zones pane: From Tops, add/update/delete, invalid input, History

**Tool/panel:** Zones pane (src/ui/zonesDialog.ts), Petrophysics ▸ **Zones…** (Intervals group)
**Preconditions:** Active well has ≥3 tops; Processing History open.
**Steps:**

1. Open **Zones…**; click **From Tops**.
2. In the add row type name "UAT_TEST", Top 2100, Bottom 2050 (bottom < top), click **Add / Update Zone**.
3. Correct to Top 2050 / Bottom 2100, click **Add / Update Zone** again; then re-add "UAT_TEST" with Bottom 2150.
4. Click **✕** on "UAT_TEST".
5. Switch to another well and back.
6. (Cross-check) In a Log View use **🖍** to paint a band, double-click it, choose **Convert to zone** — check it lands in this pane.
   **Expected:** (1) Status "Built N zone(s) from tops for <well>"; table lists one zone per consecutive top pair with correct top/bottom depths; History gains a **Zone** entry. (2) Nothing is added — the row is silently rejected (note: no error message today; record what you see). (3) Zone appears; the re-add UPDATES the same row to 2050–2150 rather than duplicating. (4) Row removed; History "Deleted zone UAT_TEST". (5) Pane follows the well — each well keeps its own zone list. (6) Converted band appears as a zone (covers REVIEW.md §Highlight tool item (e)). Covers REVIEW.md §Polish — Processing history (zone add/edit/delete).
   **Automated coverage - end-to-end (pile C, 2026-08-01):** `zones.e2e.mjs` drives the real pane
   for steps 2, 3, 4 and the isolation half of 5: add, re-add updating IN PLACE rather than
   duplicating, the silent refusal, per-well isolation, and delete. Two are frontend-only claims
   that no Rust test could pin — `db::upsert_zone` has NO validation and would store an inverted
   zone quite happily, so `bottom <= top` is refused solely by `zonesDialog.ts`. The refusal being
   silent is why the assertion is on stored state: every zone must come back byte-identical, since
   an inverted write would keep the same name and row count while swapping the interval underneath
   it. The zero-thickness case (`bottom == top`) is checked too. **Not covered:** step 1 (From
   Tops), the History entries throughout, and step 6 (Convert to zone from a highlight).

   **Result — T-WELL-15:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-16 — Per-zone parameter override actually drives a module run

**Tool/panel:** Zones pane overrides (src/ui/zonesDialog.ts) + VSH from Gamma Ray (src/ui/moduleDialog.ts, src-tauri/src/modules.rs vsh_gr)
**Preconditions:** Well with GR and ≥2 zones (e.g. ZONE_A, ZONE_B); shale and clean-sand intervals present in both.
**Steps:**

1. In **Zones…** set-row enter Zone "ZONE_A", Parameter "GR_MA", Value 60, click **Set**.
2. Confirm the override row appears (Zone/Parameter/Value table) and History logs "Set GR_MA = 60 on zone ZONE_A".
3. Petrophysics ▸ **VSH ▾** ▸ **VSH from Gamma Ray**; leave GR_MA at its dialog default 20 and GR_SH 120; scope to just this well; click **Run**.
4. Display VSH in the Log View across the ZONE_A/ZONE_B boundary; open the Curve Catalog.
   **Expected:** (3) Result line "✓ <well>: N samples → VSH_GR, VSH"; History entry "Module — Ran VSH from Gamma Ray" attributed to this well (covers REVIEW.md §Round 4 "History attribution"). (4) VSH ∈ [0,1] everywhere, high in shale, low in clean sand; inside ZONE_A VSH is systematically LOWER than the same GR would give elsewhere (denominator GR_SH−GR_MA shrinks 100→60 but the numerator drops more: at GR=60, VSH=0 in ZONE_A vs 0.40 outside) with a visible step exactly at the zone boundary — proving the zone value beat the dialog value, as the pane's hint promises ("Overrides beat the value typed in a module dialog"). VSH/VSH_GR appear in the Curve Catalog with a new version. Remove the override (✕) and re-run: the step disappears.
   **Automated coverage - pinned (pile B, 2026-07-31):** `a_per_zone_gradient_override_reaches_exactly_its_own_samples` (workflow.rs) - the same test retires this and T-PREP-05, because it is the same claim.

   **Result — T-WELL-16:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-WELL-17 — NEGATIVE: degenerate zone override (TD_BHT = 0) reports honestly, no garbage

**Tool/panel:** Zones pane + Formation Temperature module (src-tauri/src/modules.rs ftemp_grad, BHT mode)
**Preconditions:** Well with ≥1 zone; T-WELL-15 done.
**Steps:**

1. In **Zones…** set Zone "\*" , Parameter "TD_BHT", Value 0, click **Set** (the zones pane accepts any number — module dialog ranges don't apply here).
2. Petrophysics ▸ **Data Prep ▾** ▸ **Formation Temperature**; set OPT_FT = BHT; **Run** on this well.
3. Inspect FTEMP in the Log View / Curve Catalog min-max.
4. Remove the override and re-run.
   **Expected:** (2–3) The BHT interpolation must NOT emit ±Infinity or a fake green success: FTEMP comes back MISSING where TD_BHT ≤ 0 applies, and an all-missing run is reported as an error/Warned in the Processing panel rather than "✓ N samples" (covers REVIEW.md §Round 4 "All-NaN module runs report honestly"; the TD_BHT guard from AUDIT finding "ftemp_grad's BHT mode divides by TD_BHT with no degenerate-value guard" is now in modules.rs with a unit test — this test field-verifies it). (4) With the override gone, FTEMP is a smooth monotonic ramp from TSURF (~27 °C) to BHT at TD — physically sensible for a Mahakam gradient (~0.03 °C/m).
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-WELL-17:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

# Section PREP — Prep & conditioning modules

## PREP cluster — data conditioning & prep modules

**Shared preconditions:** SandiBumi running via `npm run tauri dev`; a project open with at least 2 real Mahakam wells imported from LAS, together carrying GR, RHOB, NPHI, DT, CALI, DRHO, BS (or known bit size), RES_DEEP (RT), RXO, and TVDSS on at least one deviated well. All prep modules open from **Petrophysics tab ▸ "Data Prep" dropdown** (group caption "Data Cond & Prep"); each opens as a singleton dock pane with the auto-generated form (Wells scope row, parameter fields, "Mask (optional)", "Input cons", "Output cons" = INTERP, "Outputs: …" note, "Run" button). Per-well results appear in the **Processing** panel (**Project ▸ Monitor ▸ Processing**); the audit trail is **Processing History** (**Project ▸ Monitor ▸ History**); curves and versions are in **Data ▸ Curve Catalog**. NOTE: the working tree contains uncommitted Round-4 fixes for several AUDIT-2026-07-21 findings — where a test verifies one, it is cited so a failure is logged against the known finding, not as new.

### T-PREP-01 — Module dialog machinery smoke: dropdown, pane form, "(none)" optional input

**Tool/panel:** Ribbon Data Prep dropdown + auto module pane (src/ui/ribbon.ts `renderCategoryModules`, src/ui/moduleDialog.ts)
**Preconditions:** Project open, 2+ wells imported.
**Steps:**

1. Petrophysics tab → click **Data Prep** ▾. Verify the menu lists exactly these titles: Formation Temperature, Pre-Calculation (P / T / Rmf / Ct / Cxo), Bad-Hole QC Flag, Data Conditioning Flags, Neutron Matrix Conversion, Gas Correction (density, iterated), GR Hole-Size Correction, Neutron Environmental Correction, Density Hole-Size Correction, Depth Shift, Splice Curves, GR Normalization (Two-Point Percentile), Synthetic Log (KNN Predict).
2. Pick **Formation Temperature** — it opens as a dock pane (not a popup).
3. Check the form top-to-bottom: a **Wells** scope row with mode buttons (Group if groups exist) / **★ Pinned** / **Selection** / **All** / **Custom…** and a live "N wells" count; OPT_FT dropdown; TSURF/TGRAD/BHT/TD_BHT numeric fields with units in brackets; **Mask (optional)** dropdown offering BADHOLE and COND_FLAG even before they exist; **Input cons** (default "(latest values)"); **Output cons** (default "INTERP"); the note "Outputs: FTEMP"; a **Run** button.
4. Open **Data Prep ▸ Bad-Hole QC Flag**: its optional inputs (DRHO, CALI, BS) each offer a leading **"(none)"** entry in the dropdown even though curves of those names exist.
5. Click **Project ▸ Help ▸ Help** with the Formation Temperature pane focused — the method narration/formula for the module appears.
   **Expected:** All 13 titles present; pane form matches the manifest exactly; "(none)" selectable on optional inputs (covers REVIEW.md §Round 4 "Blank '(none)' for optional inputs"); pane docks/undocks like any other (covers REVIEW.md §All tools as dockview panes #24).
   **Automated coverage - end-to-end (pile C, 2026-08-01):** `moduledialog.e2e.mjs` opens a module
   pane the way a user does - Petrophysics tab, ribbon dropdown, menu item - finding the item by the
   module's own manifest TITLE read from `list_modules`, so a rename moves both sides together
   rather than leaving the test hunting a string that no longer exists. It asserts the scope
   control, the numeric parameter fields, the Outputs note (the only place a user is told what a run
   will write before pressing Run) and the leading **(none)** option on a curve picker - a picker
   that lost it would bind the first curve in the list instead, and the module would run on a curve
   nobody chose.

   **Result — T-PREP-01:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-02 — Formation Temperature: GRADIENT and BHT modes

**Tool/panel:** Formation Temperature pane (`ftemp_grad`, src-tauri/src/modules.rs)
**Preconditions:** One well selected (Wells scope = Selection, 1 well).
**Steps:**

1. Leave **OPT_FT = GRADIENT**, TSURF 26.7, TGRAD 0.03. Click **Run**.
2. Add FTEMP to a log view (Plot ▸ New Log View, then **Properties…** ▸ add curve FTEMP) or check Min/Max in the Curve Catalog.
3. Set **OPT_FT = BHT**, enter the well's real BHT and TD (e.g. BHT 100, TD_BHT 2000). Click **Run** again.
   **Expected:** GRADIENT: FTEMP is perfectly linear, ≈26.7 °C at 0 m and TSURF + 0.03·TD at TD (e.g. ≈86.7 °C at 2000 m) — a plausible Mahakam gradient. BHT: FTEMP interpolates linearly from TSURF at surface to exactly BHT at TD_BHT. Second run creates INTERP **v2** in the Curve Catalog "Constellations" list — v1 is not overwritten. Result line: "All 1 well(s) computed…"; Processing panel shows one ✓ line.
   **Automated coverage - pinned (pile B, 2026-07-31):** `formation_temperature_lands_on_both_of_its_anchors` (modules.rs) - both modes land on their anchors, and a control proves OPT_FT still switches between them.

   **Result — T-PREP-02:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-03 — Formation Temperature negative: TD_BHT ≤ 0 zone override yields MISSING, not ±Infinity

**Tool/panel:** Formation Temperature pane + Zones… (Petrophysics ▸ Zones…)
**Preconditions:** T-PREP-02 run; well has at least one zone defined.
**Steps:**

1. Petrophysics ▸ **Zones…**: for one zone add a parameter override **TD_BHT = -10** (zone overrides bypass the dialog's 100–10000 range check).
2. In the Formation Temperature pane set **OPT_FT = BHT**, Run.
3. Inspect FTEMP inside that zone (log view cursor readout, or Curve Catalog Min/Max).
   **Expected:** Samples in the overridden zone are MISSING (blank), never ±Infinity; FTEMP outside the zone is unchanged; Curve Catalog Min/Max for FTEMP stay finite and physically sensible. Covers REVIEW.md §Round 4 "backend robustness batch 1" item (2) — verifies the fix for the audit finding "ftemp_grad's BHT mode divides by TD_BHT with no degenerate-value guard".
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-PREP-03:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-04 — Pre-Calculation: FTEMP/FPRESS/RMF/CT/CXO plausible on a known well

**Tool/panel:** Pre-Calculation (P / T / Rmf / Ct / Cxo) pane (`precalc`, modules.rs)
**Preconditions:** A well with TVDSS, RES_DEEP, RXO; its mud report Rmf + temperature at hand.
**Steps:**

1. Open **Data Prep ▸ Pre-Calculation (P / T / Rmf / Ct / Cxo)**. Confirm the RT input pre-selects the RES_DEEP family and TVDSS pre-selects the TVDSS curve.
2. Keep OPT_TU = degF and the ONWJ ft-based defaults (SURF_TEMP 77, TEMP_GRAD 0.026, PSURF 0, PGRAD 0.433), OPT_RMF = ARPS with your measured RMF_MEAS/RMF_TEMP. **Run**.
3. Spot-check one depth by hand: FTEMP_F = 77 + 0.026·TVDSS; FTEMP = same in degC; FPRESS = 0.433·TVDSS psi (near-hydrostatic); RMF at depth ≈ surface Rmf × (T₁+6.77)/(T₂+6.77) (Arps).
4. Check CT = 1000/RT and CXO = 1000/RXO at a depth where you know RT; confirm CT/CXO are MISSING wherever RT/RXO are missing or ≤ 0.
   **Expected:** Six new curves (FTEMP degC, FTEMP_F degF, FPRESS, RMF, CT, CXO) in the Curve Catalog with module = precalc provenance. Values physically plausible for Mahakam: FTEMP monotonic-increasing, RMF decreasing with depth, FPRESS ≈ hydrostatic; no negative or infinite resistivity-derived values. Covers REVIEW.md §Wave E-17 pre-calculation module items 1–3 and 5.
   **Result — T-PREP-04:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-05 — Pre-Calculation: degC metric mode + per-zone gradient kink

**Tool/panel:** Pre-Calculation pane + Zones…
**Preconditions:** T-PREP-04 done; metric well; a zone defined.
**Steps:**

1. Set **OPT_TU = degC**, SURF_TEMP 25, TEMP_GRAD 0.03 (degC/m). **Run**. Verify FTEMP in degC, FTEMP_F in degF (×1.8+32), RMF still Arps-correct.
2. In **Zones…**, override TEMP_GRAD (e.g. 0.035) for one zone. Re-run.
3. Plot FTEMP vs depth in a log view.
   **Expected:** The FTEMP trend changes slope exactly at the zone boundary (per-zone params resolve per sample) and both segments are linear. Covers REVIEW.md §Wave E-17 items 4 and 6.
   **Known issue — CONFIRMED 2026-07-31, and this step's original wording was wrong.** It used to say the trend "kinks… no discontinuity artifacts". It does not kink: `precalc` computes every sample as `SURF_TEMP + gradient(sample) × depth(sample)`, applying the gradient **from surface** rather than integrating down through the zones above, so a per-zone override produces a **STEP at the boundary**. Measured: a 0.03 °C/m well with 0.035 below 1500 m gives 67.0 °C at 1400 m and **77.5 °C at 1500 m** — a 10.5 °C jump where the trend would have risen 3.0. Rock temperature is continuous, so this is not physical, and it propagates through the Arps Rw correction into Sw. Pinned as-is by `a_per_zone_gradient_override_reaches_exactly_its_own_samples` (`workflow.rs`) — **step 3 will show a step; that is the current code, log it against this finding.** Fixing it means deciding what temperature each zone starts at, which is a method decision awaiting your call.
   **Automated coverage - pinned (pile B, 2026-07-31):** `a_per_zone_gradient_override_reaches_exactly_its_own_samples` (workflow.rs). Writing it found the zone-boundary step now recorded in the Known issue below.

   **Result — T-PREP-05:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-06 — Bad-Hole QC Flag: washout flagging vs DRHO/CALI

**Tool/panel:** Bad-Hole QC Flag pane (`badhole`, modules.rs)
**Preconditions:** Well with DRHO + CALI; a known washed-out interval (from the caliper).
**Steps:**

1. Open **Data Prep ▸ Bad-Hole QC Flag**, defaults (DRHO_MAX 0.05, DCAL_MAX 1.0, BS_DEF 8.5 — set BS_DEF to the real bit size if no BS curve). **Run**.
2. Add BADHOLE next to CALI and DRHO in a log view.
3. Negative: re-open the pane, set DRHO, CALI and BS all to **"(none)"**, Run on the same well.
   **Expected:** BADHOLE = 1 exactly where |DRHO| > 0.05 or (CALI − bit) > 1"; 0 in gauge hole; MISSING where neither QC curve reads. The flagged set visually matches the washouts the caliper shows. Negative run: every sample MISSING — with the Round-4 honesty fix the Processing panel shows **⚠ "no finite output"**, not a green ✓ (covers REVIEW.md §Round 4 "All-NaN module runs report honestly").
   **Result — T-PREP-06:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-07 — Data Conditioning Flags: coal / tight / crossover / shoulder on the log view

**Tool/panel:** Data Conditioning Flags pane (`condflag`, modules.rs)
**Preconditions:** BADHOLE computed (T-PREP-06); well with known coal streaks, a tight streak, and a gas sand.
**Steps:**

1. Open **Data Prep ▸ Data Conditioning Flags**. Confirm BADHOLE pre-selects in its input slot. Leave **Mask (optional) empty** (the module doc: masking this run with BADHOLE would blank COND_FLAG exactly where it must read 1). Defaults; RHO_MA 2.645. **Run**.
2. Add COAL_FLAG, TIGHT_FLAG, XOVER_FLAG, SHOULDER_FLAG, COND_FLAG as a flag track in a log view (Plot ▸ Properties…).
3. Check: coals picked (RHOB < 1.9 ∧ NPHI > 0.35 ∧ DT > 100 where sonic exists); tight streak flagged; crossover in the known gas sand (if NPHI is limestone-units against sandstone RHO_MA, raise XOVER_MIN to ~0.08 or convert with nphimat first — see T-PREP-08).
4. Check SHOULDER_FLAG brackets each coal/tight bed by ~0.5 m; a lone one-sample spike is dropped (MIN_THICK 0.25); a washout interval reads into COND_FLAG but a one-sample BADHOLE blip does not dilate.
5. In **Zones…** override RHO_MA = 2.71 for a carbonate zone, re-run: TIGHT/XOVER shift there only.
   **Expected:** Flags land on the intervals you would hand-pick; washed-out intervals are never called coal; COND_FLAG = coal ∪ tight ∪ badhole ∪ shoulder (no crossover, OPT_XCOND = NO default). Covers REVIEW.md §Data Conditioning Flags module #20 (all five unchecked items).
   **Result — T-PREP-07:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-08 — Neutron Matrix Conversion: LS/SS/DOL + chart spot-check

**Tool/panel:** Neutron Matrix Conversion pane (`nphimat`, modules.rs + neutron_charts.rs)
**Preconditions:** Well with NPHI; you know the tool (TNPH/NPHI/APLC…) and recorded matrix from the LAS header.
**Steps:**

1. Open **Data Prep ▸ Neutron Matrix Conversion**. Set TOOL to match the delivery, MATRIX_IN per the header (usually LS or SS), SALINITY = FRESH (Mahakam). **Run**.
2. In a clean water sand, read NPHI_LS / NPHI_SS / NPHI_DOL at one depth.
3. Hand-check against the paper chart (Por-5 for CNL/TNPH, Por-4 for APS/SNP) at that depth.
4. Feed NPHI_SS + RHO_MA 2.65 into a condflag (or phi_dn) run: crossover in the known gas sand should now appear at the default XOVER_MIN 0.04, no limestone-offset fudge.
   **Expected:** NPHI_SS ≈ NPHI_LS + 0.03–0.04 in clean sand; NPHI_DOL well below both (thermal dolomite bow); the input convention passes through unchanged; hand-chart agreement within ~0.5 pu (the shipped worked example TNPH 18 pu @ 250 kppm → SS 24 pu reproduces to 0.04 pu). Covers REVIEW.md §Neutron Matrix Conversion module #21 items 1–3.
   **Result — T-PREP-08:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-09 — GR Hole-Size Correction

**Tool/panel:** GR Hole-Size Correction pane (`gr_hole_corr`, modules.rs)
**Preconditions:** Well with GR + CALI; a washout interval known from T-PREP-06.
**Steps:**

1. Open **Data Prep ▸ GR Hole-Size Correction**, defaults (K_GR 0.0075, BS_DEF = real bit size). **Run**.
2. Overlay GR_EC on GR in a log view.
3. Negative: set CALI to **"(none)"** and Run again.
   **Expected:** GR_EC > GR only where CALI > bit size (in the washout, GR_EC restored upward by ~0.75 %/inch of enlargement); GR_EC = GR in gauge and in undersize hole (no negative correction). With CALI = (none): GR_EC identical to GR (documented pass-through), still reported as a normal ✓ run.
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-PREP-09:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-10 — Density Hole-Size Correction

**Tool/panel:** Density Hole-Size Correction pane (`rhob_hole_corr`, modules.rs)
**Preconditions:** Well with RHOB + CALI including an interval with CALI > 10".
**Steps:**

1. Open **Data Prep ▸ Density Hole-Size Correction**, defaults (K_RHO 0.004, HD_REF 10). **Run**.
2. Overlay RHOB_EC on RHOB.
   **Expected:** RHOB_EC = RHOB wherever CALI ≤ 10"; above 10" RHOB_EC is shifted up by 0.004 g/cc per inch beyond 10 — a small (<~0.05 g/cc) correction; grossly washed-out intervals should instead be excluded via BADHOLE downstream (the module doc says no correction is trustworthy there).
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-PREP-10:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-11 — Neutron Environmental Correction: computed-only FTEMP contract

**Tool/panel:** Neutron Environmental Correction pane (`nphi_env_corr`, modules.rs)
**Preconditions:** One well WITH precalc run (T-PREP-04); ideally a second well whose LAS carries a raw FTEMP curve (degF) and no precalc.
**Steps:**

1. Open **Data Prep ▸ Neutron Environmental Correction**, defaults (K_TEMP 0.0001, T_REF 24, K_SAL −0.002, SALW 20000 — Mahakam-fresh). **Run** on the precalc'd well.
2. Compare NPHI_EC to NPHI at a hot deep level and a shallow level.
3. Negative: Run on the well with only the raw LAS FTEMP (no precalc/ftemp_grad output).
   **Expected:** Step 2: correction = 0.0001·(FTEMP−24) − 0.0004 — small, positive at depth, larger where hotter; NPHI_EC tracks NPHI within a few thousandths v/v. Step 3: the raw degF FTEMP must NOT be consumed (FTEMP is a computed-only input) — only the salinity term applies, so NPHI_EC = NPHI − 0.0004 everywhere. Covers REVIEW.md §Round 4 "backend robustness batch 1" item (5) — verifies the fix for the audit finding "nphi_env_corr's FTEMP input is a plain log_in, not computed_only".
   **Automated coverage - pinned (pile B, 2026-07-31):** `a_raw_ftemp_never_satisfies_the_computed_only_contract` (workflow.rs) - a raw degF FTEMP in the RAW set is correctly ignored, and the computed one is followed sample by sample.

   **Result — T-PREP-11:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-12 — Gas Correction WITH precalc + condflag: plausible de-gassed density

**Tool/panel:** Gas Correction (density, iterated) pane (`gascorr`, modules.rs)
**Preconditions:** A known gas well; precalc (T-PREP-04) and condflag (T-PREP-07) both run so FTEMP/FPRESS and XOVER_FLAG exist.
**Steps:**

1. Open **Data Prep ▸ Gas Correction (density, iterated)**. Confirm FTEMP/FPRESS slots resolve to the precalc outputs and GAS_FLAG defaults to XOVER_FLAG. Keep OPT_GATE = FLAGGED, defaults (RHO_MA 2.65, SG_GAS 0.65). **Run**.
2. Overlay RHOB_GC on RHOB; add GASDEN.
3. Check a coal streak (flagged by COAL_FLAG, excluded from XOVER_FLAG) stays untouched.
4. Feed RHOB_GC to **phi_den** (Porosity dropdown) — NOT phi_dn (doc: corrected RHOB + still-gas-affected NPHI biases porosity low).
   **Expected:** In the gas sand RHOB_GC > RHOB (gas replaced by liquid, density restored up); PHIT_GC slightly below the uncorrected density porosity; GASDEN ≈ 0.10–0.15 g/cc at typical Mahakam P/T (the KK example pins 0.1297 at 2743 psi / 93.9 °C); SWT_GC in [0,1]. Coal untouched. Outside flagged zones RHOB_GC = RHOB. Covers REVIEW.md §Gas Correction module #23 items 1, 2 and 5.
   **Result — T-PREP-12:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-13 — Gas Correction negatives: no condflag → hard error; no precalc → honest all-NaN report

**Tool/panel:** Gas Correction pane + Processing panel
**Preconditions:** A well with RHOB/RT but with NO condflag run; a well (or fresh cons) with condflag but NO precalc.
**Steps:**

1. Run Gas Correction (OPT_GATE = FLAGGED, defaults) on the well without condflag.
2. On the well with condflag but no precalc (FTEMP/FPRESS absent — they are computed-only, a raw LAS FTEMP must not satisfy them), Run again.
3. Open the Processing panel and Processing History; inspect how each run is reported.
   **Expected:** Step 1: explicit error — "OPT_GATE = FLAGGED but the gas flag '…' has no data — run condflag first or set OPT_GATE = EVERYWHERE" (covers REVIEW.md §Gas Correction #23 item 3). Step 2: every output sample is MISSING and the run must be reported as **⚠/error "no finite output — every sample is missing (check inputs, e.g. precalc not run)"** — NOT a green "✓ N samples" success (covers REVIEW.md §Round 4 "All-NaN module runs report honestly" and §Gas Correction #23 item 4).
   **Known issue — RESOLVED, corrected 2026-07-31:** this step used to say the Round-4 fix for AUDIT-2026-07-21 "Module-run status reports '✓ success' even when every output sample is MISSING" was still uncommitted, and told you to log a green ✓ against that finding rather than as new. **It is committed and pinned** by `all_nan_module_output_reports_error_not_success` (`workflow.rs`), which runs on every gate. So a green "✓ N samples" on an all-MISSING run is now a **genuine new failure** — log it as one. Note the one case where a green ✓ is CORRECT: if the flag curve covers only part of the well, the unflagged samples pass RHOB through with real values, so the run is not all-MISSING and should succeed. Step 2 only holds where the flag covers everything you ran.
   **Automated coverage - pinned (pile B, 2026-07-31):** `the_empty_flag_refusal_names_the_users_curve_and_its_remedy_works` (modules.rs), on top of the older `gascorr_guards_stay_missing_or_error` and `gascorr_flag_gate_and_missing_inputs`. What was genuinely uncovered and is now pinned: the refusal must NAME the curve you picked, and the remedy it recommends (EVERYWHERE) must actually work.

   **Result — T-PREP-13:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-14 — GR Normalization: P3/P97 alignment across two wells

**Tool/panel:** GR Normalization (Two-Point Percentile) pane (`gr_normalize`) + Histogram (Plot ▸ Histogram)
**Preconditions:** 2+ wells with GR whose raw histograms visibly differ.
**Steps:**

1. Plot ▸ **Histogram**, curve GR, on well A: ⚙ **Properties** ▸ user percentiles "3, 97" ▸ Apply. Note P3/P97. Repeat for well B — confirm they differ (the mis-calibration you are about to remove).
2. Open **Data Prep ▸ GR Normalization (Two-Point Percentile)**. Wells scope = **Selection** with both wells (Ctrl-click in the Wells pane). Keep defaults (P_LOW 3, P_HIGH 97, GR_LOW_REF 20, GR_HIGH_REF 120 — generic clean-sand and clay endpoints, matching vsh_gr's own defaults; substitute your field refs if you have them). **Run**.
3. Histogram of **GRN** on each well with the same "3, 97" percentiles.
   **Expected:** For every normalized well, GRN's P3 ≈ GR_LOW_REF and P97 ≈ GR_HIGH_REF (percentile pinning, whatever references you ran with) — the histograms now overlay; shale intervals read high, clean sands low, character preserved (a linear rescale, no shape change). Both wells get their own ✓ line in the Processing panel.

   > **Corrected 2026-07-31.** This step used to name 53.68 / 133.93 as the defaults. The
   > provenance sweep replaced those with the generic 20 / 120 — a two-decimal endpoint is
   > somebody's regression result, and it is silently wrong in another basin. Each well anchoring
   > on its OWN percentiles is now pinned by
   > `gr_normalization_anchors_each_well_on_its_own_percentiles` (`workflow.rs`), so step 3 is
   > checked automatically; what remains for you is whether the character survived on real logs.
   **Automated coverage - pinned (pile B, 2026-07-31):** `gr_normalization_anchors_each_well_on_its_own_percentiles` (workflow.rs).

   **Result — T-PREP-14:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-15 — MASK machinery: BADHOLE mask changes gr_normalize percentiles and blanks outputs

**Tool/panel:** GR Normalization pane + Mask dropdown (src/ui/moduleDialog.ts, workflow.rs masking)
**Preconditions:** T-PREP-06 (BADHOLE with a real flagged interval containing hot/washout GR) and T-PREP-14 run.
**Steps:**

1. Re-run GR Normalization on one well with **Mask (optional) = BADHOLE**, Output cons = "TEST" (type it — new constellation).
2. Compare GRN (TEST, masked) against GRN (INTERP, unmasked) in a log view (use Input cons to pick each).
   **Expected:** Two observable differences: (a) GRN is MISSING inside every BADHOLE = 1 interval (outputs blanked); (b) GRN values in GOOD hole shift too, because the well P3/P97 are now computed from unmasked samples only — the washout/hot-streak GR no longer anchors the two-point transform. If (b) shows no change at all, the input-side masking is broken. Both runs visible as separate constellations (INTERP vs TEST) in the Curve Catalog.
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-PREP-15:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-16 — Synthetic Log (KNN Predict): fill a gap, then the masked-washout case

**Tool/panel:** Synthetic Log (KNN Predict) pane (`log_predict`, modules.rs; workflow.rs output-masking)
**Preconditions:** A well where DT (or RHOB) is absent over an interval but GR + NPHI (+ RHOB) read; BADHOLE from T-PREP-06.
**Steps:**

1. Open **Data Prep ▸ Synthetic Log (KNN Predict)**. TARGET = DT, P1 = GR, P2 = RHOB, P3 = NPHI, OPT_COMBINE = FILL_MISSING, K = 5, no Mask. **Run**.
2. Overlay DT_SYN on DT: the gap interval is filled; where DT exists, DT_SYN = DT (FILL_MISSING keeps raw).
3. Now the washout-repair case: TARGET = RHOB, OPT_COMBINE = **MAX_RAW**, **Mask = BADHOLE**. **Run**. Inspect RHOB_SYN inside a BADHOLE = 1 washout.
4. Negative: run with TARGET = a curve with under 10 valid samples (or scope a nearly-empty well).
   **Expected:** Step 2: filled values are plausible DT (within the well's DT range, tracking lithology — high in shale/coal, low in tight streaks); no extrapolated nonsense outside predictor coverage. Step 4: all-MISSING output reported as ⚠ "no finite output", not green success. Step 3 SHOULD show a finite repaired RHOB inside the washout (that is the module's purpose), but see below.
   **Known issue:** AUDIT-2026-07-21 finding "log_predict's MAX_RAW/repaired-synthetic value is unconditionally re-blanked at masked (washout) depths by workflow.rs's output-masking step" (§Prep statistical #1, CONFIRMED; still unfixed — listed in REVIEW.md Round 4 under "6 findings that … await your sign-off"). Expect step 3 to FAIL: RHOB_SYN will be NaN exactly inside the masked washout it exists to fill. Log as known.
   **Automated coverage - pinned (pile B, 2026-07-31):** `a_synthetic_log_fills_gaps_keeps_raw_and_repairs_only_downward` (modules.rs) plus `a_masked_washout_defeats_the_very_module_meant_to_repair_it` (workflow.rs). The second one pins the audited defect AS-IS, not as correct behaviour.

   **Result — T-PREP-16:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-17 — Depth Shift: block shift + dialog range validation

**Tool/panel:** Depth Shift pane (`depth_shift`, modules.rs)
**Preconditions:** Well with GR and a sharp marker (coal top / tight streak) at a known depth.
**Steps:**

1. Open **Data Prep ▸ Depth Shift**. CURVE = GR, SHIFT = 2 (m, + = deeper). **Run**.
2. Overlay GR_DS on GR at the marker.
3. Negative: type SHIFT = 5000 and click Run.
4. Zone case: in **Zones…** give one zone SHIFT = −1 and re-run; check the shift flips only inside that zone.
   **Expected:** The marker on GR_DS sits exactly 2 m deeper; curve shape preserved (linear resample); ends that shift off the depth range go MISSING; the raw GR is untouched. Step 3: inline validation "SHIFT: value must be between -1000 and 1000." and no run occurs. Output named GR_DS in the Curve Catalog.
   **Result — T-PREP-17:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-18 — Splice Curves: run-to-run splice at depth

**Tool/panel:** Splice Curves pane (`splice`, modules.rs)
**Preconditions:** A well with two overlapping GR runs (or use GR and GR_EC as stand-ins).
**Steps:**

1. Open **Data Prep ▸ Splice Curves**. TOP_CURVE = run-1 GR, BOT_CURVE = run-2 GR, SPLICE_DEPTH = the casing-shoe/overlap depth. **Run**.
2. Overlay the output on both inputs around the splice depth.
   **Expected:** Output (named `<top input>_SPL`) equals TOP_CURVE strictly above SPLICE_DEPTH and BOT_CURVE at and below it — one clean handover, no averaging, no gap; inputs unmodified. Where the contributing curve is MISSING the output is MISSING (no fill from the other curve).

   **Automated coverage - pinned (pile B, 2026-07-31):** `a_gap_in_the_contributing_run_stays_a_gap` (modules.rs) checks all four quadrants — a gap in the run that IS contributing survives as a gap in both directions (with a real value sitting in the other run at that depth, so the assertion means something), and a gap in the run that is NOT contributing is irrelevant. `a_sample_with_no_depth_is_not_assigned_to_a_side` covers a sample with no depth. Both pass; the promise holds.

   **Worth knowing before you click:** the handover is **half-open** — the sample sitting exactly ON the splice depth belongs to the BOTTOM run. Overlay the two inputs and you should see the last top-run sample one step above the depth you typed. Also: if your two runs have different data coverage, expect gaps to show through rather than be papered over — that is the intended answer, since filling one would be a second splice at a depth you never chose and could not see on the log.
   **Result — T-PREP-18:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PREP-19 — Cross-checks: multi-well scope, Processing lines, History attribution, provenance, live refresh, theme

**Tool/panel:** Pre-Calculation pane + Processing panel + Processing History + Curve Catalog (workspace.ts, inspectorPanel.ts)
**Preconditions:** 2+ wells; precalc parameters valid for both; a log view open showing FTEMP; T-PREP-04 run at least once.
**Steps:**

1. In the Pre-Calculation pane set Wells scope = **All** (count shows "N wells"; hover the count to see the names). **Run**.
2. The result line reads "Running precalc on N well(s)… see the Processing panel for progress" and the Processing panel fronts: verify one line per well with ✓/⚠/✗ state and the count chips.
3. Open **Processing History** (**Project ▸ Monitor ▸ History**): the new entry reads "Ran Pre-Calculation (P / T / Rmf / Ct / Cxo) on N wells" — attributed to the wells actually run, not whichever well happens to be selected.
4. **Data ▸ Curve Catalog**: FTEMP/FPRESS/RMF/CT/CXO rows show cons/version/module ("precalc"); the "Constellations" section shows INTERP bumped to the next version with module · date, and hovering shows the params JSON (your SURF_TEMP/TEMP_GRAD etc.).
5. The already-open log view showing FTEMP refreshes on its own (dataVersion) — no manual reload, viewport kept.
6. With the module pane and Processing panel open, switch **Project tab ▸ Theme** to Dark, then Pertamina: both panes repaint immediately in the new palette (form fields, result text, per-well lines readable).
7. Import a new LAS (or compute any curve) while the precalc pane stays open: its curve dropdowns pick up the new names without losing your current selections.
   **Expected:** All seven observations hold. Covers REVIEW.md §Round 4 "History attribution" and "Race guards" (step 7 exercises the refresh race fix for the audit finding "moduleDialog.ts's persistent-pane data refresh has no race-guard generation counter"), and REVIEW.md §P1 "Plots refresh after a module run, keeping their viewport".
   **Result — T-PREP-19:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

# Section PETRO — Core petrophysics modules

I have everything I need from the source. Here is the PETRO cluster.

---

## PETRO cluster — core petrophysics modules (ribbon **Petrophysics** tab, auto-generated module panes)

Shared preconditions for all tests: SandiBumi running via `npm run tauri dev`; a project open with at least one real well carrying **GR, RHOB, NPHI, DT, RES_DEEP** (Mahakam LAS import done); the **Petrophysics** ribbon tab is the active tab on startup. Module panes are opened from the category dropdown buttons (**VSH**, **Porosity**, **Saturation**, **Permeability**, **Thin Beds**); every pane has the same auto-generated layout: **Wells** scope row (Group / ★ Pinned / Selection / All / Custom…), input-curve dropdowns, parameters, **Mask (optional)**, **Input cons**, **Output cons** (default `INTERP`), an "Outputs: …" note, and a **Run** button. Verify outputs in the Curve Catalog (**Data ▸ Curve Catalog**, table columns Mnemonic / Unit / Family / Set / Source / Samples), the **Processing** panel (auto-opens on Run), and **Processing History** (**Project ▸ Monitor ▸ History**). Tests 01–02 must run before 06–17 (VSH/PHIE/PHIT are chain inputs).

### T-PETRO-01 — vsh_gr linear smoke run

**Tool/panel:** "VSH from Gamma Ray" module pane — Petrophysics ▸ VSH ▸ VSH from Gamma Ray (src-tauri/src/modules.rs `vsh_gr_spec`, form src/ui/moduleDialog.ts)
**Preconditions:** One well with GR imported and selected. Pick GR_MA/GR_SH first from Plot ▸ Histogram of GR (your P3/P97 convention).
**Steps:**

1. Petrophysics ▸ **VSH** dropdown ▸ **VSH from Gamma Ray**.
2. In **Wells**, click **Selection** (or **All** if one well). Confirm the count chip reads "1 well".
3. Leave `OPT_GR` = LINEAR; set `GR_MA` / `GR_SH` to your histogram picks; `GR` input = GR; **Output cons** = `INTERP`.
4. Click **Run**.
   **Expected:** Result line "All 1 well(s) computed. Per-well details are in the Processing panel."; status bar reads "vsh_gr: 1/1 well(s) computed"; the **Processing** panel auto-opens with a ✓ for the well. Curve Catalog gains **VSH_GR** and **VSH** rows (Set = INTERP, Source = vsh_gr). Domain: VSH within 0–1, ≈1 in massive shale, <0.15 in clean sand; VSH_GR (unlimited) may run slightly outside 0–1 but never ±Infinity. Processing History shows "\<well>: Ran VSH from Gamma Ray".
   **Result — T-PETRO-01:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-02 — vsh_gr nonlinear options + version N+1

**Tool/panel:** VSH from Gamma Ray module pane (as T-01)
**Preconditions:** T-01 passed.
**Steps:**

1. In the same pane change `OPT_GR` to **LARINOV1** (Larionov Tertiary); Run.
2. Repeat for **LARINOV2**, **STIEBER1**, **CLAVIER** (four more runs, same Output cons `INTERP`).
3. Open Curve Catalog and inspect the versioned-run list under Constellations; hover a version row.
   **Expected:** Each re-run lands as version N+1 (never overwrites; catalog list shows "vsh_gr · \<timestamp>" per run, hover tooltip lists params so you can tell which OPT_GR each version used). Domain: at intermediate GR every nonlinear VSH < the LINEAR VSH (all corrections concave — e.g. linear 0.50 → Larionov-Tertiary ≈0.33, Larionov-older ≈0.22); endpoints 0 and 1 unchanged; all limited VSH stay 0–1.

   **THE LARIONOV LABELS ABOVE ARE REVERSED — READ THIS BEFORE PICKING ONE (2026-07-31, finding 21).** Step 1 calls `LARINOV1` "Larionov Tertiary" and the Expected line pairs Tertiary with ≈0.33 and older-rocks with ≈0.22. Both are backwards relative to what the code computes. `LARINOV1` is `0.33*(2^(2*IGR) - 1)` — Larionov for **older rocks, Mesozoic and older**, giving **0.330** at IGR 0.5. `LARINOV2` is `0.083*(2^(3.7*IGR) - 1)` — Larionov for **Tertiary / unconsolidated**, giving **0.216**. The code matches the published coefficient sets; the labels in this step do not. Mahakam Delta is Miocene, so the transform you almost certainly want is **LARINOV2**, and picking LARINOV1 on the label above would overstate shale volume by more than half through the whole intermediate-GR interval — where the VSH cutoff decides net pay, with nothing on the log to show for it. The dropdown itself gives no rock age either; it shows the bare option ids.

   **A second correction to the Expected line: "endpoints 0 and 1 unchanged" does not hold for the Larionov forms, and that is not a defect.** They are empirical fits that were never normalized to close at pure shale: at IGR 1, LARINOV1 stops at **0.99**, LARINOV2 at **0.9957**, and LARINOV3 **overshoots to 1.133**. LINEAR, all three Stieber forms and Clavier do land on exactly 1.0 (Clavier cancels exactly, 1.7 − sqrt(0.49) = 1.0). VSH clamps every one of them into 0–1; VSH_GR keeps the raw value. That difference is what the two outputs are for, so read VSH for the answer and VSH_GR when you want to see the transform's own behaviour.

   **Automated coverage - pinned (pile B, 2026-07-31):** `every_vsh_gr_transform_lands_on_its_published_coefficient` (modules.rs) evaluates ALL EIGHT options at IGR 0.5 against their published closed forms computed by hand — so it is a check, not a snapshot of a run — plus both endpoints, the clamping, the concavity claim (every nonlinear form reads below LINEAR at intermediate GR), and monotonicity in GR, which is where a sign or bracket slip hides when the endpoints still look right. `re_running_a_module_bumps_the_set_version_and_keeps_every_earlier_run` (equations.rs) covers step 3: five runs give five versions each carrying its own OPT_GR in the params record, a different set name versions independently, and a well that has never been run starts at 1 whatever its neighbours are on.
   **Result — T-PETRO-02:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-03 — vsh_gr invalid parameters (negative)

**Tool/panel:** VSH from Gamma Ray module pane (as T-01)
**Preconditions:** T-01 passed.
**Steps:**

1. Type `GR_SH` = 1500 (above its 1000 max); click **Run**.
2. Restore `GR_SH` = 120; set `GR_MA` = 150 (≥ GR_SH, but inside its own 0–200 range); click **Run**.
   **Expected:** Step 1: no run — the result line reads "GR_SH: value must be between 0 and 1000." and focus returns to the field. Step 2: the run executes but every sample is skipped by the GR_MA ≥ GR_SH guard → all-NaN output, and the Processing panel reports the well as **Warned/error ("no finite output")**, NOT a green success (covers REVIEW.md §"Round 4 — AUDIT-2026-07-21 safe-bucket follow-through" item "All-NaN module runs report honestly"). No crash; catalog stats stay finite.
   **Automated coverage - end-to-end (pile C, 2026-08-01):** step 1 only.
   `moduledialog.e2e.mjs` drives the range refusal through vsh_gr's own pane and checks the message
   NAMES the parameter and its bounds - a bare "invalid input" is true and useless on a form with
   several numeric fields - and that nothing was written. **Not covered:** step 2, the
   `GR_MA >= GR_SH` guard reaching the all-NaN honest-report path.

   **Result — T-PETRO-03:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-04 — vsh_dn crossplot VSH + VSH_DN_FLAG

**Tool/panel:** "VSH from Density-Neutron" module pane — Petrophysics ▸ VSH ▸ VSH from Density-Neutron (`vsh_dn_spec`)
**Preconditions:** Well with RHOB + NPHI + GR; a known gas-bearing interval helps.
**Steps:**

1. Petrophysics ▸ VSH ▸ **VSH from Density-Neutron**.
2. Set matrix/shale/fluid endpoints from your crossplot picks (`RHO_MA`, `RHO_SH`, `NPHI_SH` etc.); leave `GR` input = GR (it is optional — note the "(none)" entry exists); `FLAG_TOL` = 0.25.
3. **Run**; add VSH_DN_FLAG and both VSH curves to a log view.
   **Expected:** Outputs **VSH_DN**, **VSH**, **VSH_DN_FLAG** in the catalog. VSH 0–1, high in shale. VSH_DN_FLAG is strictly 0/1 and raises 1 exactly where (a) the point falls off the matrix–shale–fluid triangle (VSH_DN < −0.05 or > 1.05) or (b) N-D VSH diverges from GR VSH by > FLAG_TOL — expect flagged streaks across the gas interval (N-D reads low vs GR) and across kaolinite-rich vs illite intervals (clay-type sensitivity).
   **Result — T-PETRO-04:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-05 — vsh_dn degenerate-triangle regression (no ±Infinity)

**Tool/panel:** VSH from Density-Neutron module pane (as T-04)
**Preconditions:** T-04 passed.
**Steps:**

1. Leave all endpoints at defaults but type `RHO_SH` = **2.0482843** (makes matrix/shale/fluid collinear: (c−d) ≈ 0).
2. **Run**; then check the VSH_DN row's stats in the Curve Catalog and autoscale in a log view.
   **Expected:** No ±Infinity anywhere: the degenerate samples are skipped to MISSING, so VSH_DN is blank and the run is reported **Warned ("no finite output")** in the Processing panel; catalog min/max and plot autoscale stay finite. This verifies the fix for AUDIT-2026-07-21 finding "vsh_dn's density-neutron crossplot divides by (c − d) with no guard against a degenerate matrix/shale/fluid triangle" (covers REVIEW.md §"Round 4…" item "(1) vsh_dn now skips a degenerate matrix/shale/fluid triangle"). If the curve pins at ±Inf, log Fail citing that finding.
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-PETRO-05:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-06 — phi_den density porosity incl. shale branch

**Tool/panel:** "Porosity from Density" module pane — Petrophysics ▸ Porosity ▸ Porosity from Density (`phi_den_spec`)
**Preconditions:** VSH computed (T-01); well spans both clean sand and massive shale.
**Steps:**

1. Petrophysics ▸ **Porosity** ▸ **Porosity from Density**; inputs RHOB = RHOB, VSH = VSH.
2. Set `RHO_MA` 2.645 (Mahakam sand), `RHO_SH`, `RHO_FL`; leave `OPT_PHIEMAX` = SHALE_REDUCED, `PHIE_MAX` = 0.3. **Run**.
3. Re-run with `OPT_PHIEMAX` = MAXIMUM and compare versions.
   **Expected:** Outputs PHIE_DEN, PHIT_DEN, **PHIE**, **PHIT**. Domain: 0 ≤ PHIE ≤ PHIT ≤ ~0.35; clean Mahakam sand PHIE ≈ 0.20–0.33; in VSH ≥ 0.95 intervals PHIE = 0 exactly and PHIT = the shale porosity (RHO_DSH−RHO_SH)/(RHO_DSH−RHO_W) ≈ 0.09 at defaults. SHALE_REDUCED caps PHIE at PHIE_MAX·(1−VSH), so mid-shaly samples cap lower than the MAXIMUM version; the two versions differ only where the cap bites. (Note: phi_den/phi_dn had zero unit tests per the audit — your hand-check here is the coverage.)
   **Result — T-PETRO-06:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-07 — phi_dn crossplot porosity, AVERAGE vs GAS_RMS

**Tool/panel:** "Porosity from Density-Neutron" module pane — Petrophysics ▸ Porosity ▸ Porosity from Density-Neutron (`phi_dn_spec`)
**Preconditions:** VSH computed; a known gas interval with D-N crossover. If you have run Neutron Matrix Conversion, use NPHI_SS + `RHO_MA` 2.65 (covers the unchecked REVIEW.md §"Neutron Matrix Conversion module — NPHI LS/SS/DOL (#21)" item "Feed NPHI_SS + RHO_MA 2.65 into phi_dn").
**Steps:**

1. Petrophysics ▸ Porosity ▸ **Porosity from Density-Neutron**; `OPT_XPLOT` = AVERAGE; **Run**.
2. Change `OPT_XPLOT` = **GAS_RMS**; **Run** again into the same cons.
3. Compare the two PHIE versions across the gas interval in a log view.
   **Expected:** Both versions: 0 ≤ PHIE ≤ PHIT ≤ ~0.35, PHIE = 0 in VSH ≥ 0.95 shale. Domain: in the gas interval (density porosity ≫ neutron porosity) **GAS_RMS PHIE > AVERAGE PHIE** — the RMS restores gas-suppressed porosity; in water-bearing sand the two are nearly identical. No negative PHIE spikes at the shale-reduction clamps.
   **Result — T-PETRO-07:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-08 — phi_son Wyllie/RHG + OPT_CP compaction correction

**Tool/panel:** "Porosity from Sonic" module pane — Petrophysics ▸ Porosity ▸ Porosity from Sonic (`phi_son_spec`)
**Preconditions:** Well with DT + VSH; shallow undercompacted section ideally.
**Steps:**

1. Petrophysics ▸ Porosity ▸ **Porosity from Sonic**; `OPT_SON` = WYLLIE, `OPT_CP` = OFF (default); **Run** (baseline).
2. Set `OPT_CP` = **ON**, leave `DT_SH` at its default **90**; **Run**.
3. Set `OPT_SON` = **RHG**, `OPT_CP` still ON; **Run**. Compare the three PHIT_SON versions.
   **Expected:** Baseline: PHIT_SON = (DT−DT_MA)/(DT_FL−DT_MA), 0–1, tracking the D-N porosity in compacted sand. RHG version unaffected by OPT_CP (self-compacting). Domain expectation for step 2: with DT_SH = 90 (≤ 100 µs/ft, i.e. compacted shale) the Cp correction should be a **no-op** — instead the current code divides by Cp = 0.9 and **inflates PHIT_SON ≈ +11 %** over the whole well. Covers REVIEW.md §"Held-item resolutions" item "Wyllie lack-of-compaction (Cp) correction — shipped as opt-in".
   **Known issue:** AUDIT-2026-07-21 finding "phi_son OPT_CP lack-of-compaction correction is missing the DT_SH>100 us/ft gate — it inflates porosity instead of no-op below the threshold, including at the module's own default". Expect step 2 to fail the domain check (porosity rises ~11 % where it should be unchanged); awaiting your sign-off on the gate — log as known, not new.
   **Automated coverage - pinned, with a caveat (pile A):** the gate checks Wyllie/RHG and the opt-in compaction correction. The caveat: it pins the CURRENT un-gated behaviour, which is the audited defect. If the DT_SH gate is ever added, that test changes with it. It is not a vote that this behaviour is right.

   **Result — T-PETRO-08:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-09 — phimax constant + TVDSS-trend porosity ceiling

**Tool/panel:** "Porosity Ceiling (φmax)" module pane — Petrophysics ▸ Porosity ▸ Porosity Ceiling (φmax) (`phimax_spec`)
**Preconditions:** PHIE computed (T-06/07). Covers the unchecked try-items in REVIEW.md §"φmax porosity ceiling — phimax module (#26)".
**Steps:**

1. Petrophysics ▸ Porosity ▸ **Porosity Ceiling (φmax)**; `MODE` = **constant**, `PHIMAX0` = 0.25, input `PHI` = PHIE; **Run**.
2. Overlay **PHIE_CAP** and **PHIE_MAX** with PHIE in a log view.
3. `MODE` = **linear**, `TVDSS_REF` = a shallow depth, `PHIMAX_GRAD` = 0.03; **Run** (no TVDSS curve → it reads against measured DEPTH — fine for a near-vertical well).
   **Expected:** Constant mode: PHIE_CAP = PHIE wherever PHIE < 0.25 and flattens at exactly 0.25 above it; PHIE_MAX is a flat 0.25 line; no point pokes above the ceiling; input PHIE itself is untouched (same version/stats as before). Linear mode: PHIE_MAX declines with depth (deeper = lower ceiling); a deep zone's ceiling < a shallow zone's. Output curves are named **PHIE_CAP / PHIE_MAX** (named after the input curve).
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-PETRO-09:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-10 — sw_arch smoke + coal/tight zero-porosity guard

**Tool/panel:** "SW — Archie" module pane — Petrophysics ▸ Saturation ▸ SW — Archie (`sw_arch_spec`)
**Preconditions:** PHIT + PHIE computed; RES_DEEP present; Rw known (Pickett or client value). A coal/tight streak (PHIT = 0) in the well exercises the guard.
**Steps:**

1. Petrophysics ▸ **Saturation** ▸ **SW — Archie**; `OPT_RW` = CONSTANT, `RW` = your Rw at formation temperature; A/M/N = 1/2/2; `RT` input = RES_DEEP.
2. **Run**; display SWT, SWE, SWT_ARCH in a log view; check SWT_ARCH min/max in the Curve Catalog.
   **Expected:** SWT and SWE within 0–1; SWE ≤ SWT everywhere; ≈1 in known wet sands; low (≈0.2–0.5, fresh-water Mahakam caveat noted) in pay; VOL_UWAT = PHIE·SWE. Over coal/tight PHIT = 0 streaks SWT = SWE = 1 (all-water convention) and **SWT_ARCH stays finite** — catalog min/max never shows Infinity (covers REVIEW.md §"Low-tier correctness & data-integrity sweep" item "Archie SWT_ARCH no longer writes +Infinity").
   **Result — T-PETRO-10:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-11 — sw_indo vs sw_sim vs sw_arch on the same interval

**Tool/panel:** "SW — Indonesia (Poupon-Leveaux)" and "SW — Simandoux" module panes (`sw_indo_spec`, `sw_sim_spec`)
**Preconditions:** T-10 done; VSH computed; `RT_SH` picked from massive shale resistivity.
**Steps:**

1. Petrophysics ▸ Saturation ▸ **SW — Indonesia (Poupon-Leveaux)**; identical A/M/N/RW to T-10; set `RT_SH`; `OPT_INDO` = FULL; **Run**.
2. Petrophysics ▸ Saturation ▸ **SW — Simandoux**; same parameters; `OPT_SIM` = MODIFIED; **Run**.
3. Put SWT_ARCH, SWE_INDO, SWE_SIM in one log-view track over a shaly-sand interval.
4. Re-run Simandoux with `OPT_SIM` = **SCHLUMBERGER** and check a VSH = 1 massive-shale interval.
   **Expected:** All three within 0–1 after limiting. Domain: in shaly zones **Indonesia and Simandoux read LOWER Sw than Archie** (Archie ignores shale conductivity and overestimates Sw); in clean sand the three converge to within a few s.u.; Indonesia ≈ Simandoux over moderate VSH. Step 4: pure shale resolves to SWE = 1 (all-water), not a silent gap — covers REVIEW.md §"Low-tier…sweep" item "Simandoux (SCHLUMBERGER) no longer divides by zero at VSH=1".
   **Result — T-PETRO-11:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-12 — RT = 0 null-streak regression (no +Infinity in unlimited Sw)

**Tool/panel:** SW — Archie / SW — Indonesia module panes (as T-10/11)
**Preconditions:** A well whose RES_DEEP has a zero/null streak (or zero a few samples via Data ▸ DB Inspector first).
**Steps:**

1. Run **SW — Archie** and **SW — Indonesia** over that well (defaults from T-10/11).
2. Inspect SWT_ARCH and SWE_INDO in the Curve Catalog (min/max) and in a log view over the streak.
   **Expected:** The RT ≤ 0 streak reads as a **gap (MISSING)** in SWT_ARCH/SWE_INDO — not Sw = 1, and never +Infinity; curve autoscale is not pinned to a huge number; catalog min/max finite. This verifies the fix for AUDIT-2026-07-21 finding "sw_arch/sw_indo store +Infinity in their unlimited curves when RT is exactly 0 (or negative)" (covers REVIEW.md §"Round 4…" item "Correctness — RT ≤ 0 → +Infinity in the Sw modules"). If the streak pins the scale, log Fail citing that finding.
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-PETRO-12:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-13 — zone parameter override: RW in one zone only

**Tool/panel:** Zones pane (Petrophysics ▸ **Zones…**, src/ui/zonesDialog.ts) + SW — Archie pane
**Preconditions:** T-10 done; the well has formation tops.
**Steps:**

1. Petrophysics ▸ **Zones…**; click **From Tops** (or add one with Zone name/Top/Bottom + **Add / Update Zone**).
2. Under **Per-zone parameter overrides** enter Zone = one zone's exact name, Parameter = `RW`, Value = 0.02; click **Set**. The override appears in the Zone/Parameter/Value table.
3. Re-run **SW — Archie** with the dialog still showing `RW` = 0.1 (overrides beat the dialog value, per the pane's own hint).
4. Compare the new SWT version against the previous one in a log view.
   **Expected:** SWT changes **only inside the overridden zone**: with N = 2, SWT there drops by ×√(0.02/0.1) ≈ ×0.45; outside the zone the two versions are identical sample-for-sample. Processing History shows "\<well>: Set RW = 0.02 on zone \<name>" and the re-run entry.

   **Automated coverage - pinned (pile B, 2026-07-31):** `a_zone_parameter_override_moves_that_zone_and_leaves_the_rest_untouched` (workflow.rs) checks exactly the numbers this step names — the ×√(0.02/0.1) ratio inside the zone, and **sample-for-sample equality** outside it (`assert_eq!` on the raw f32, not a tolerance). The dialog still says RW = 0.1 on the re-run, so the override really is what wins.

   **Worth knowing before you click:** a zone interval is **half-open** — `[top, bottom)`. Two adjacent zones written the way anyone writes them (1000–1010 and 1010–1020) do not both claim the sample sitting exactly on 1010; it belongs to the deeper zone. So if you are checking the boundary in a log view, expect the change to stop one sample above where you might read it off the zone table. Also: a `*` override applies well-wide FIRST and a named zone overrides it, so the two stack rather than conflict — and where two NAMED zones overlap, the later one in the table wins silently.
   **Result — T-PETRO-13:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-14 — perm_wyllie_rose, all OPT_WR variants

**Tool/panel:** "Permeability — Wyllie-Rose" module pane — Petrophysics ▸ Permeability ▸ Permeability — Wyllie-Rose (`perm_wyllie_rose_spec`)
**Preconditions:** PHIE computed; `SWE_IRR` from your Buckles/SWE-in-pay estimate (default 0.15).
**Steps:**

1. Petrophysics ▸ **Permeability** ▸ **Permeability — Wyllie-Rose**; `OPT_WR` = TIMUR; **Run**.
2. Re-run with **MORRIS_BIGGS_OIL**, **MORRIS_BIGGS_GAS**, **TIXIER** (same cons → 4 versions).
3. Histogram PERM_WR (log scale) per version.
   **Expected:** PERM_WR/PERM ≥ 0, MISSING where PHIE is missing, 0 at PHIE = 0. Domain sanity at φ ≈ 0.25, Swirr 0.15: TIMUR ≈ 900 mD, MORRIS_BIGGS_OIL ≈ 700 mD, MORRIS_BIGGS_GAS ≈ 70 mD. **MORRIS_BIGGS_OIL and TIXIER are byte-identical** (same C=250/D=3/E=1 in this port — confirm, it is documented in the pane's doc); GAS is lowest by >~1 decade.
   **Automated coverage - pinned (pile B, 2026-07-31):** `the_wyllie_rose_variants_carry_their_own_constants_and_two_are_one_equation` (modules.rs) confirms all four numbers above, the TIXIER / MORRIS_BIGGS_OIL identity and the decade between oil and gas; the edge cases were already covered by `perm_wyllie_rose_edges` and `perm_wyllie_rose_negative_phie_missing_across_all_variants`. One thing to know that the plan does not say: an UNRECOGNISED OPT_WR silently falls back to TIMUR, so a typo in a saved chain becomes a different rock without complaint.

   **Result — T-PETRO-14:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-15 — perm_coates default constant

**Tool/panel:** "Permeability — Coates" module pane — Petrophysics ▸ Permeability ▸ Permeability — Coates (`perm_coates_spec`)
**Preconditions:** PHIE computed; a Geolog perm_coates run on the same well available for comparison (your reference suite).
**Steps:**

1. Petrophysics ▸ Permeability ▸ **Permeability — Coates**; leave `CONST_COATES` at its default **100**, `SWE_IRR` = 0.15; **Run**.
2. Compare PERM_COATES against your Geolog perm_coates output for the same interval (histogram or export).
3. Re-run with `CONST_COATES` = **70** and compare again.
   **Expected:** PERM = (C·PHIE²·(1−Swirr)/Swirr)², finite and ≥ 0, MISSING where PHIE missing. Step 3 (C = 70) should match Geolog.
   **Known issue:** AUDIT-2026-07-21 finding "perm_coates default CONST_COATES (100) doesn't match the reference suite source it claims to port (documented default is 70)". Expect step 2 to read **≈ 2.04× (=(100/70)²) higher than Geolog** at the default; awaiting your sign-off on 100→70 — log as known, not new, and note which constant you want as the shipped default.
   **Result — T-PETRO-15:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-16 — perm_transform vs core φ–k + overflow regression

**Tool/panel:** "Permeability — Por-Perm Transform" module pane — Petrophysics ▸ Permeability ▸ Permeability — Por-Perm Transform (`perm_transform_spec`)
**Preconditions:** RCAL core porosity-permeability data imported for the well; PT_A/PT_B fitted from your core log10(k)–φ regression (Crossplot pane fit on the core curves).
**Steps:**

1. Petrophysics ▸ Permeability ▸ **Permeability — Por-Perm Transform**; enter your fitted `PT_A` (slope) and `PT_B` (intercept); **Run**.
2. Crossplot PERM_XFM (log axis) vs PHIE and compare against the core φ–k points — same fit line.
3. Negative test: set `PT_A` = 100, `PT_B` = 5 (both in-range); **Run**; check the highest-porosity samples.
   **Expected:** Step 2: the transform curve tracks the core cloud (it IS the regression — deviations only from the difference between log-derived and core porosity). Step 3: samples where 10^(PT_A·φ+PT_B) overflows come out **MISSING, never +Infinity** — catalog min/max finite (covers REVIEW.md §"Round 4…" item "(4) perm_transform emits MISSING instead of +Infinity"; closes the matching AUDIT finding).
   **Result — T-PETRO-16:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-17 — thin_bed_ts Thomas-Stieber decomposition

**Tool/panel:** "Thin Beds — Thomas-Stieber" module pane — Petrophysics ▸ Thin Beds ▸ Thin Beds — Thomas-Stieber (`thin_bed_ts_spec`)
**Preconditions:** PHIT + VSH computed; a known laminated sand-shale (LRLC) interval and a clean massive sand for contrast. `PHI_SD_MAX` from clean-sand PHIT (histogram, e.g. 0.30); `PHI_SH` from massive-shale PHIT (e.g. 0.10–0.15).
**Steps:**

1. Petrophysics ▸ **Thin Beds** ▸ **Thin Beds — Thomas-Stieber**; set `PHI_SD_MAX`, `PHI_SH`; inputs PHIT/VSH; **Run**.
2. Display VLAM, VDISP, VSAND, PHIE_LAM with VSH and PHIT in a log view over both intervals.
   **Expected:** All fractions 0–1; VLAM + VDISP ≈ VSH; VSAND = 1 − VLAM. Domain: in the laminated interval **VLAM ≫ VDISP** (points near the laminated line) and **PHIE_LAM > PHIT** (sand porosity restored after stripping laminar shale, capped at PHI_SD_MAX); in clean sand VLAM ≈ VDISP ≈ 0 and PHIE_LAM ≈ PHIT; in massive shale VLAM → VSH. PHIE_LAM goes MISSING only where VSAND ≈ 0.
   **Result — T-PETRO-17:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-18 — missing input curve + Cancel mid-batch (negative)

**Tool/panel:** Porosity from Sonic + VSH from Gamma Ray panes; Processing panel (src/ui/processingPanel.ts)
**Preconditions:** A multi-well group where at least one well has **no DT** curve; ≥ 10 wells total for the cancel step.
**Steps:**

1. **Porosity from Sonic** ▸ Wells = **Group** (the mixed group) ▸ **Run**.
2. In the Processing panel open **▸ details** and read the per-well breakdown.
3. **VSH from Gamma Ray** ▸ Wells = **All** ▸ **Run**; while the bar fills, click **Cancel** in the Processing panel.
   **Expected:** Step 1–2: DT-less wells show ⚠/✗ with a message (missing input / no finite output), the rest ✓; result line reads "X/Y well(s) computed — Z need attention. Open Processing → details for the report." — never a green all-clear. Step 3: button flips to "Cancelling…", the job stops within a well or two and shows state **Cancelled**; wells already computed keep their new version; open plots do NOT show stale data afterwards (dataVersion bumps on cancel — covers REVIEW.md §"Round 4…" item "dataVersion refresh … on workflow-chain cancel/fail"). No frozen UI at any point (the run lives off the main thread — closes the AUDIT finding "vsh_gr / vsh_dn standalone module runs never leave the Tauri main thread").
   **Result — T-PETRO-18:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PETRO-19 — provenance, History attribution, live refresh, theme repaint

**Tool/panel:** Curve Catalog (Data ▸ Curve Catalog), Processing History (**Project ▸ Monitor ▸ History**), any module pane
**Preconditions:** Multi-well group available; a log view and a histogram of VSH left open.
**Steps:**

1. Select well A in the Wells pane, but run **VSH from Gamma Ray** with Wells = **Custom…** ticking only well B.
2. Open Processing History; read the newest entry.
3. Re-run with Wells = **Group** (N > 1 wells); read the new entry.
4. Watch the already-open log view and VSH histogram as each run completes.
5. Project tab ▸ **Theme** ▸ Dark (then back).
   **Expected:** Step 2: entry reads "\<well B>: Ran VSH from Gamma Ray" — attributed to the well actually run, NOT the selected well A (covers REVIEW.md §"Round 4…" item "History attribution"; closes the AUDIT finding "Batch module runs … attribute their History-panel entry to the globally 'selected' well"). Step 3: entry reads "Ran VSH from Gamma Ray on N wells" with no single-well name. Step 4: both open plots redraw with the new version automatically — no reopen needed; the module pane's own curve dropdowns now list the new outputs. Step 5: ribbon, module pane, catalog and plots all repaint in the new theme with no white/stale patches; switching back restores exactly.
   **Result — T-PETRO-19:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

Note for the runbook owner: four issues the orchestrating plan expected to be open (vsh_dn degenerate triangle, RT ≤ 0 → +Infinity, perm_wyllie_rose negative-PHIE, perm_transform overflow) are already fixed in current code (`D:\XX. SandiBumi\src-tauri\src\modules.rs`) per REVIEW.md §"Round 4 — AUDIT-2026-07-21 safe-bucket follow-through"; T-PETRO-05/12/16 are written as fix-verification regressions (expected Pass) rather than Known-issue failures. Only the two sign-off-pending findings (phi_son OPT_CP gate → T-PETRO-08; perm_coates 100 vs 70 → T-PETRO-15) carry **Known issue** lines.

---

# Section ADV — Advance tab (SSC/SSPW, RtC/IMTS, SandiMin, Sw-height)

### Cluster ADV — Advance ribbon tab (SSC, SSPW, RtC, IMTS, Saturation-Height, SandiMin)

Shared preconditions: project open in `npm run tauri dev` with at least one Mahakam-style well imported carrying GRN, RHOB, NPHI, DT and RES_DEEP (RXO if available); **precalc** already run on that well (FTEMP_F/RMF exist); zones defined in **Zones…**. A second well processed through **SSPW only** (no SSC curves) is needed for T-ADV-10/11, and a deviated well with an imported deviation survey for T-ADV-13. Note: several audit findings named below were fixed in the uncommitted Round-4 working tree (REVIEW.md §Round 4, all still `[ ]`) — those tests double as the click-through verification of that batch.

### T-ADV-01 — Advance tab smoke: all flagship buttons render

**Tool/panel:** Advance ribbon tab (index.html `data-panel="advance"`, buttons auto-generated by src/ui/ribbon.ts `renderAdvancedModules`)
**Preconditions:** app started, any project open.
**Steps:**

1. Click the **Advance** ribbon tab.
2. Confirm the **Advance Methods** group shows buttons **SSC**, **SSPW**, **RtC**, **IMTS**, **Thin Beds**; hover each and read the tooltip (full title + method description, e.g. "SSC — Sand-Silt-Clay (Kuttan/LQR) — …").
3. Confirm the **Mineral Solver** group shows **SandiMin…** and that no button named "Mineral Inv" / legacy "multimin" appears anywhere on the tab.
4. Narrow the window until the tab overflows; a **›** chevron appears at the right edge and scrolls the panel; **‹** appears after scrolling.
   **Expected:** all five method buttons plus SandiMin… present with correct tooltips; legacy multimin absent (superseded by SandiMin); overflow chevrons work.
   **Automated coverage - end-to-end (pile C, 2026-08-01):** `shell.e2e.mjs` "fills the Advance tab
   with the flagship methods and the calibration tools" asserts all five promoted buttons (SSC,
   SSPW, RtC, IMTS, Thin Beds) resolved from their manifests, plus SandiMin…, Calibrate RtC…,
   Calibrate S… and ML Models…. A companion test sweeps the WHOLE ribbon for a legacy
   "Mineral Inv" / "Multimin" button and asserts there is none — step 3's negative. **Not
   covered:** the tooltip text of each button, and step 4's overflow chevrons.

   **Result — T-ADV-01:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-02 — SSC run with LQR defaults: wiring + cross-checks

**Tool/panel:** SSC module pane (Advance ▸ SSC; form auto-built by src/ui/moduleDialog.ts from src-tauri/src/ssc.rs `ssc_spec`)
**Preconditions:** Mahakam well with GRN/RHOB/NPHI selected in the Wells pane.
**Steps:**

1. Advance ▸ **SSC**. In the **Wells** scope row click **Selection** (count shows "1 well").
2. Verify defaults: GR = GRN, RHOB = RHOB, NPHI = NPHI; OPT_VSHGR = LINEAR; GR_MA 10 / GR_SH 150 / RHOB_MA 2.65 / RHOB_WCL 2.3 / NPHI_WCL 0.6 / RHOB_DCL 2.71 / PHIT_CL 0.24; **Mask (optional)** = (none); **Output cons** = INTERP. The hint line lists all 23 outputs (VSAND … PHIT_GR).
3. Click **Run**.
   **Expected:** result line "Running ssc on 1 well(s)… see the Processing panel for progress", the **Processing** panel surfaces automatically and shows the well ✓; then "All 1 well(s) computed…"; status line "ssc: 1/1 well(s) computed". Quick-access clock button ▸ **Processing History** pane has a new "Module" entry naming this well (covers REVIEW.md §Round 4 "History attribution"). Data ▸ **Curve Catalog**: VSAND, VSILT, VDCL, VWCL, VSH_SSC, VSHGR, VSHND, PHIT_SSC, PHIE_SSC, PHIFF_SSC, CBW, CWSH, BW, SWIRR_T, SWIRR_EFF and the 8 `*_GR` curves all listed under cons INTERP v1.
   **Result — T-ADV-02:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-03 — SSC domain validation: closure, porosity ladder, bound-water split

**Tool/panel:** Log View (Plot ▸ New Log View) reading SSC outputs
**Preconditions:** T-ADV-02 passed.
**Steps:**

1. Open a New Log View on the well; add tracks for VSAND/VSILT/VDCL/PHIT_SSC, then PHIT_SSC/PHIE_SSC/PHIFF_SSC, then CBW/CWSH/BW, then VSH_SSC + VSHGR.
2. At 3 depths (one clean sand, one silty interval, one shale) use the cursor readout to read values and hand-sum VSAND+VSILT+VDCL+PHIT_SSC.
3. Check the ladder PHIT_SSC ≥ PHIE_SSC ≥ PHIFF_SSC ≥ 0 everywhere; check BW = CBW+CWSH ≤ PHIT_SSC and SWIRR_T = BW/PHIT_SSC ∈ [0,1].
   **Expected:** closure ≈ 1.00 (±0.01) at every spot-check; volumes each in [0,1]; VSH_SSC high (→ ~1) in shale, low (< ~0.2) in clean sand and broadly tracking VSHGR; PHIT_SSC in a plausible Mahakam range (~0.10–0.35 in sands); CBW grows with clay content; SWIRR_T low in clean sand, → 1 in shale; SWIRR_EFF = 1 (not 0) at zero-PHIE shale points (covers REVIEW.md §Low-tier sweep "SSC SWIRR_EFF no longer 0 at a 100 %-shale point").
   **Result — T-ADV-03:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-04 — SSC GR-equivalent family (\*\_GR): eyeball closely

**Tool/panel:** Log View on SSC `*_GR` outputs (ssc.rs GR-rescale block)
**Preconditions:** T-ADV-02 passed; GRN present so VSHGR is finite.
**Steps:**

1. Add tracks VSAND_GR/VSILT_GR/VDCL_GR/PHIT_GR and PHIT_GR/PHIE_GR/CBW_GR/CWSH_GR/PHIFF_GR.
2. Spot-check 3 depths: VSAND_GR+VSILT_GR+VDCL_GR+PHIT_GR ≈ 1; PHIT_GR = PHIE_GR+CBW_GR; PHIE_GR = PHIFF_GR+CWSH_GR.
3. Compare the `*_GR` track sums against VSHGR: shale-side volumes should honour VSHGR (higher where VSHGR is higher than the N-D shale estimate).
4. Find a pure-shale streak (VWSH → 1): the `*_GR` curves go blank there.
   **Expected:** closure and the two porosity identities hold at every spot-check; `*_GR` blank (MISSING) at pure-shale/degenerate-VWSH samples is BY DESIGN, not a failure; no negative volumes.
   **Known issue:** AUDIT-2026-07-21 "SSC's 8-curve GR-equivalent output family (\*\_GR) has zero unit-test coverage" — a closure regression test was added only in the uncommitted Round-4 batch (REVIEW.md §Round 4 "New test coverage", unchecked) and the family has never had domain sign-off (REVIEW.md §Phase 8.5 validation item). This is the least-proven output family in the app: eyeball it against your reference-suite run and log ANY discrepancy against the audit finding.
   **Result — T-ADV-04:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-05 — SSC negative tests: bad param, empty scope, no-GR well

**Tool/panel:** SSC module pane
**Preconditions:** any well selected; one well without a GRN curve available (or temporarily pick a non-existent GR mnemonic).
**Steps:**

1. Set RHOB_MA = 9 and click **Run** → inline message "RHOB_MA: value must be between 1 and 4." and focus returns to the field; nothing runs, no History entry.
2. Restore 2.65. Switch scope to **★ Pinned** with no wells pinned; **Run** → "No wells in scope — pick a group, pin/select wells, or choose All."
3. Scope back to **Selection**; select the no-GRN well (GR dropdown may still offer "GRN" even though absent). **Run**.
   **Expected:** steps 1–2 block cleanly with the quoted messages. Step 3: run completes; density-neutron outputs (VSAND…PHIT_SSC, bound-water) are finite but VSHGR and all 8 `*_GR` curves are blank — SSC degrades gracefully, no crash, no fabricated VSHGR.
   **Result — T-ADV-05:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-06 — SSC re-run: versioning + open-plot refresh (dataVersion)

**Tool/panel:** SSC module pane + Curve Catalog + open Log View
**Preconditions:** T-ADV-02 passed; Log View from T-ADV-03 still open showing PHIT_SSC.
**Steps:**

1. In the SSC pane change RHOB_DCL 2.71 → 2.75, keep **Output cons** = INTERP, **Run**.
2. Watch the open Log View without touching it.
3. Data ▸ **Curve Catalog**: check the log-set version history for the well.
   **Expected:** the open Log View's PHIT_SSC/VDCL tracks redraw with the new values within a moment of the run finishing (no manual reopen); Curve Catalog shows INTERP at version 2 with version 1 preserved (re-run = N+1, never overwrites); a second Processing History entry appears.
   **Result — T-ADV-06:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-07 — SSPW run: PHR-standard porosity ladder

**Tool/panel:** SSPW module pane (Advance ▸ SSPW; src-tauri/src/ssc.rs `sspw_spec`)
**Preconditions:** a well with RHOB, NPHI and a VSH curve (run a VSH module first if needed); note SSPW's NPHI must be sandstone units.
**Steps:**

1. Advance ▸ **SSPW**, scope **Selection**. Inputs: RHOB = RHOB, NPHI = NPHI, VSH = your VSH curve. Defaults RHOB_MAT 2.65 / RHOB_SH 2.4 / RHOB_DSH 2.71 / VOL_CBW_SH 0.1.
2. **Run**.
3. Log View: PHIT_SSPW/PHIE_SSPW/PHIFF_SSPW + CBW_SSPW/CAPBW_SSPW/BW_SSPW/SWIRR_SSPW.
   **Expected:** run reports 1/1 computed; PHIT_SSPW ≥ PHIE_SSPW ≥ PHIFF_SSPW; CBW_SSPW ≈ VSH·0.1 (rises with VSH); SWIRR_SSPW ∈ [0,1], high in shale; Curve Catalog gains the 7 `*_SSPW` rows under INTERP; History entry recorded.
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-ADV-07:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-08 — SW-RtC after SSC: corrected Rt, Sw vs Indonesia

**Tool/panel:** RtC module pane (Advance ▸ RtC; src-tauri/src/lrlc.rs `sw_rtc_spec`)
**Preconditions:** T-ADV-02 passed on an LRLC well with RES_DEEP; know one high-clay microporous pay interval and one clean water sand.
**Steps:**

1. Advance ▸ **RtC**, scope **Selection**. Verify defaults: RT = RES_DEEP, PHIT = PHIT_SSC, CAPBW = CWSH, CBW = CBW; RW 0.3 (set your zone Rw), M/N 2.0, A_CAP 0.45, B_QV 0.0057, C0 −0.0071, RSF 2.25. **Run**.
2. Petrophysics ▸ Saturation ▸ **SW — Indonesia (Poupon-Leveaux)** on the same well with PHIE = PHIE_SSC, VSH = VSH_SSC, same RW/M/N. **Run**.
3. Log View: RES_DEEP + RT_CORR (log scale), CEX_RTC, then SWT_RTC/SWE_RTC/SWE_INDO on one track.
   **Expected:** SWT_RTC and SWE_RTC ∈ [0,1] with SWE_RTC ≤ SWT_RTC; RT_CORR ≥ RES_DEEP everywhere (conductivity only removed, capped at 98%); CEX_RTC ≥ 0 and largest in clayey/silty intervals. Domain acceptance: in the high-clay microporous pay interval SWE_RTC reads visibly LOWER than SWE_INDO (that is the point of the method); in the clean water sand the two agree and both read ~1.
   **Result — T-ADV-08:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-09 — SW-IMTS after SSC: iterative Waxman-Smits-family Sw

**Tool/panel:** IMTS module pane (Advance ▸ IMTS; src-tauri/src/lrlc.rs `sw_imts_spec`)
**Preconditions:** T-ADV-02 and T-ADV-08 step 2 done.
**Steps:**

1. Advance ▸ **IMTS**, scope **Selection**. Defaults: RT = RES_DEEP, PHIT = PHIT_SSC, VKAOL = VDCL, SWIRR = SWIRR_T, CBW = CBW; set RW and TEMP_C to your zone values; MSTAR/NSTAR 1.9, S_FACTOR 0.5, CEC_KAOL 8 / CEC_ILL 25. **Run**.
2. Log View: SWT_IMTS/SWE_IMTS next to SWE_INDO and SWT_RTC; add QVEFF.
   **Expected:** SWT_IMTS ∈ [0,1]; QVEFF ≥ 0, rising with clay volume and shrinking porosity; in high-clay LRLC pay SWT_IMTS reads LOWER than SWE_INDO and broadly agrees with SWT_RTC (the two excess-conductivity methods should tell the same geological story); clean water sand ~1 for all. Curve Catalog + History entries as usual.
   **Result — T-ADV-09:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-10 — RtC/IMTS on an SSPW-only well: SSPW fallback

**Tool/panel:** RtC + IMTS panes (lrlc.rs `prefer()` fallback)
**Preconditions:** the SSPW-only well (T-ADV-07 well, never run through SSC).
**Steps:**

1. Select the SSPW-only well, RtC pane, scope **Selection**, leave every input at its default (PHIT still says PHIT_SSC — that curve does not exist on this well). **Run**.
2. Repeat in the IMTS pane.
3. Log View: SWT_RTC / SWT_IMTS on this well.
   **Expected:** both runs succeed with FINITE Sw curves — PHIT/CAPBW/CBW silently fall back per-sample to PHIT_SSPW/CAPBW_SSPW/CBW_SSPW (covers REVIEW.md §Round 4 "LRLC SSPW fallback", **Try** case verbatim). Sw values plausible per T-ADV-08/09 criteria.
   **Known issue:** AUDIT-2026-07-21 "sw_rtc/sw_imts default input wiring points only to SSC's curve names; running them against an SSPW-only well silently produces an all-NaN 'success'" — fixed in the uncommitted Round-4 batch. Expect PASS; an all-blank Sw curve here is that finding resurfacing — log as known, not new.
   **Automated coverage - pinned (pile B, 2026-07-31):** the sw_rtc half was already covered by `rtc_falls_back_to_sspw_curve_names`; `the_sspw_fallback_covers_imts_and_chooses_sample_by_sample` (lrlc.rs) adds sw_imts and pins that the fallback is per SAMPLE, not per curve - so a section reprocessed through SSPW mixes cleanly with SSC curves above and below it. It also checks the fallback lands on the SAME Sw as the SSC path, not merely on a finite one.

   **Result — T-ADV-10:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-11 — RtC on a well with NO porosity curve at all: honest failure

**Tool/panel:** RtC pane + Processing panel (workflow.rs all-NaN guard)
**Preconditions:** a raw well with RES_DEEP but neither PHIT_SSC nor PHIT_SSPW.
**Steps:**

1. Select the raw well, RtC pane, scope **Selection**, defaults, **Run**.
2. Read the pane result line and the Processing panel row for this well.
   **Expected:** NOT a green "N samples → …" success: the well is reported as an error / **Warned** with "no finite output — every sample is missing (check inputs…)" and the pane says "0/1 well(s) computed — 1 need attention" (covers REVIEW.md §Round 4 "All-NaN module runs report honestly").
   **Known issue:** AUDIT-2026-07-21 "Module-run status reports '✓ success' even when every output sample is MISSING" — fixed in the uncommitted Round-4 batch. Expect PASS; a green full-sample-count success here is that finding — log as known.
   **Automated coverage - pinned (pile B, 2026-07-31):** `rtc_without_porosity_under_either_name_is_reported_not_returned_as_success` (workflow.rs), with the well then given porosity under the FALLBACK name only as the control - so the refusal is provably about absent porosity, not about sw_rtc failing to look for the SSPW name. One thing to expect that Expected does not mention: per finding 10, the blank output curves ARE still written to the catalog.

   **Result — T-ADV-11:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-12 — Saturation-Height (Leverett) with SCAL-fitted A/B

**Tool/panel:** Import SCAL modal (Data ▸ Import Data ▾ ▸ Import SCAL…, src/ui/ribbon.ts `handleImportScal`) + SW — Saturation-Height pane (Petrophysics ▸ Saturation dropdown; src-tauri/src/satheight.rs)
**Preconditions:** well with PHIE and PERM curves (run a permeability module first); a core Pc CSV for that well; FWL known for the reservoir.
**Steps:**

1. Select the well, Data ▸ Import Data ▾ ▸ **Import SCAL…**, pick the Pc file(s); **File format** = Auto-detect, **Fluid system** = your lab system (e.g. Air-brine (72)); click **Import & Fit**.
2. Note the reported "J-fit: A = …, B = …, R² = … (n points)".
3. Petrophysics ▸ Saturation ▸ **SW — Saturation-Height**: OPT_SWH = LEVERETT, FWL = your contact (same depth reference as the well — negative for TVDSS), RHO_W/RHO_HC/IFT_RES per fluid, **SWH_A/SWH_B = the fitted A/B**, PHIE = PHIE, PERM = PERM. **Run**.
4. Log View: SWH + HAFWL, alongside your resistivity Sw (SWE_INDO or SWT_RTC).
   **Expected:** SCAL import reports the fit and leaves an "Import" History entry; SWH = 1 at and below the FWL (HAFWL ≤ 0), decreases with height above it, lowest in high-perm/high-φ intervals, never below SWT_IRR; HAFWL = FWL − depth. Domain acceptance: SWH broadly tracks the resistivity-based Sw in the transition zone. B ≈ negative (typically −0.2…−0.8); < 3 valid Pc points instead reports "Too few valid points to fit…".
   **Result — T-ADV-12:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-13 — Saturation-Height on a DEVIATED well: TVD input is a no-op

**Tool/panel:** SW — Saturation-Height pane, TVD input (satheight.rs; workflow.rs input resolution)
**Preconditions:** a deviated well with its deviation survey imported (Data ▸ Import Data ▾ ▸ deviation import) but no TVD channel in the source LAS; T-ADV-12 parameters known.
**Steps:**

1. Open **SW — Saturation-Height** on the deviated well. Note the TVD dropdown offers "TVD" (leave it selected).
2. **Run**, then read HAFWL at one depth in the strongly deviated section.
3. By hand compute FWL − MD and FWL − trueTVD (from the survey) at that depth and compare with HAFWL.
   **Expected:** (desired behaviour) HAFWL = FWL − trueTVD, i.e. the height honours the deviation survey.
   **Known issue:** AUDIT-2026-07-21 "sw_height's TVD input has no producer anywhere in the app — the deviated-well fix (marked DONE, unit-tested) is a no-op in real use" — NOT yet fixed. Expect HAFWL = FWL − MD exactly (the TVD dropdown is a false affordance: no curve named TVD exists, the module silently falls back to measured depth, overstating height ≈ 1/cos(inc) and understating SWH in the deviated section). Mark **Fail — known**, log against the audit finding.

   **THE KNOWN ISSUE ABOVE IS OUT OF DATE — IGNORE IT (2026-07-31).** The producer it says does not exist does: `ingest::materialize_tvd_curves` resamples the deviation survey onto the log depth grid and writes TVD/TVDSS as fetchable curves, on every deviation import. Expect the step to **PASS** — HAFWL = FWL − trueTVD, the desired behaviour. Do not mark Fail out of deference to the paragraph above.

   **Automated coverage - pinned (pile B, 2026-07-31):** `a_deviated_wells_height_is_measured_from_the_survey_not_along_hole` (workflow.rs) runs the whole path — imports a survey (vertical to 1000 m, building to 60° by 2000 m), runs sw_height through the real input resolution, reads HAFWL back from the database — and it lands on FWL − TVD at every sample, more than 500 m above the along-hole answer at TD. It also pins the fallback: a well with **no** survey still measures along hole, which is correct for a vertical well.

   **Worth knowing before you click:** both halves of this were already tested and both were green the whole time the feature was a no-op — `sw_height_uses_tvd_and_allows_tvdss_fwl` hands the module a TVD array by hand, `deviation_import_materializes_tvd_tvdss_curves` checks the survey reaches the log grid. Nothing tested the joint. If your deviated well still reads along hole, check the survey actually imported (the Wells pane ▸ tree shows it) before suspecting the module.
   **Result — T-ADV-13:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-14 — SandiMin pane smoke + fluid autofill from precalc (incl. well-switch race)

**Tool/panel:** SandiMin — Mineral Solver pane (Advance ▸ SandiMin…; src/ui/multiminDialog.ts)
**Preconditions:** precalc'd well selected; a second precalc'd well available; zones defined.
**Steps:**

1. Advance ▸ **SandiMin…**. Verify: components box grouped **Minerals / Clays / Fluids** (fluids badged flushed/unflushed) with Quartz, Illite, Water Sxo, Water Sw pre-ticked; tool rows with curve + σ fields (Formation Density 0.0264, Neutron 0.014, Sonic Transit Time 1.951, Total Gamma Ray 6, Unflushed Conductivity (from RT) → RES_DEEP with σ blank = "auto"); **Fluid properties (CT/CXO — resistivity → conductivity)** box with Rw/Rmf/temps/m/n/Mud and a live preview line (w=… Cw=… Cbw=…).
2. In the **Autofill from precalc** row leave "(whole well)" and click **Read** → Formation temp (°F) and Rmf sample fill; status reads "SandiMin autofill (WELL, whole well): FTEMP … °F, RMF … ohmm (n/n samples)". Repeat with a zone picked (covers REVIEW.md §"SandiMin: wet→dry clay converter + fluid autofill from precalc (#22)" autofill items).
3. Switch to the second well in the Wells pane: the autofill zone list refreshes to that well's zones (covers §#22 "Switch wells…"). Click **Read**, then IMMEDIATELY switch back to well 1 before it resolves.
4. On a well never precalc'd, click **Read**.
   **Expected:** steps 1–2 as described. Step 3: the form is NOT stamped with the wrong well's FTEMP/RMF after the switch (stale response discarded). Step 4: status "SandiMin autofill: no FTEMP_F/RMF samples on … — run the precalc module first"; nothing applied.
   **Known issue:** AUDIT-2026-07-21 "SandiMin's 'Autofill from precalc' Read button has no stale-response race guard, unlike refreshZones() in the same file" — fixed in the uncommitted Round-4 batch (REVIEW.md §Round 4 "Race guards", unchecked). Expect PASS on step 3; stale values appearing is that finding — log as known.
   **Result — T-ADV-14:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-15 — SandiMin wet→dry clay converter (KKT ONWJ workflow)

**Tool/panel:** SandiMin pane, "Wet clay → dry clay (PHIT-basis endpoints)" box
**Preconditions:** SandiMin pane open, CT tool on (fluid box visible).
**Steps:**

1. Enter Wet RHOB 2.18333, Wet NPHI 0.48958, Wet GR 110, Dry clay density 2.70; leave Wet DT "(none)"; **Apply to clay** = Illite (covers REVIEW.md §#22 "KK-1 Post Main check").
2. Read the live preview (φ_clay, dry RHOB/NPHI/GR, v_bw = k·v_dryclay ratio, CEC_eq).
3. Click **Apply to clay + include BoundWater**.
4. Change Formation temp by 20 °F and watch both the fluid preview AND the dry-clay preview update.
   **Expected:** preview shows a sensible φ_clay (~0.3–0.4 for these picks) and CEC_eq > 0; on Apply, Illite's RHOB/NPHI/GR endpoints in the table change to the dry values, its CEC cell = CEC_eq, and **BoundWater** becomes ticked automatically; status message ends "…(re-apply if fluid T/Rw/α or this clay's RHOB endpoint change)" — the pairing rule of §#22. Both previews refresh on the temperature edit.
   **Result — T-ADV-15:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-16 — SandiMin full run: closure, RECON, vs deterministic answers

**Tool/panel:** SandiMin pane + Log View + Curve Catalog
**Preconditions:** SSC (T-ADV-02) and RtC (T-ADV-08) done on the well for comparison; precalc autofill applied (T-ADV-14).
**Steps:**

1. Components: Quartz, Illite, **Water Sxo**, **Water Sw**, plus one HC pair for your fluid (e.g. **Gas Sxo** + **Gas Sw**). Tools on: Formation Density, Neutron, Sonic Transit Time, Total Gamma Ray, Unflushed Conductivity (from RT); Flushed Conductivity (from RXO) too if RXO exists.
2. Check the endpoints table shows a row per component with editable cells, "auto" in the CT/CXO columns for in-zone fluids, CEC editable only for Illite, **Max** column present.
3. **Apply to wells** scope = Selection; **Output prefix** = MM; **Hard unity (Σ minerals + unflushed fluids = 1)** ticked. Click **Run**.
4. Read the result table (Well | Samples solved | Mean recon (σ) | Note).
5. Log View: VOL*QUARTZ/VOL_ILLITE/VOL*_ + MM*PHIT/MM_SWT/MM_RECON; cursor-sum Σ(mineral + unflushed-fluid VOL*_) at 3 depths; compare MM*PHIT vs PHIT_SSC and MM_SWT vs SWT_RTC.
   **Expected:** status "SandiMin: running…" then "SandiMin: wrote N curves to 1 well(s)"; Samples solved ≈ the interval sample count, Mean recon low (order ~1σ, i.e. ≲ 2); Σ VOL*_ (minerals + unflushed fluids) ≈ 1.00 at every spot-check; MM*PHIT tracks PHIT_SSC within a few p.u.; MM_SWT tells the same story as SWT_RTC (low in pay, ~1 in water sand); MM_SXOT ≥ MM_SWT (WBM invasion). Curve Catalog: all VOL*_ + MM\_\* rows under set SANDIMIN; History entry "SandiMin (Quartz, Illite, …) → …" naming the well; any open plot showing an input refreshes.
   **Result — T-ADV-16:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-17 — SandiMin re-run with a lowercase prefix: no shadow rows

**Tool/panel:** SandiMin pane + Curve Catalog / DB Inspector (multimin2.rs prefix uppercasing)
**Preconditions:** T-ADV-16 done (MM\_\* curves exist).
**Steps:**

1. Same setup, change **Output prefix** to lowercase `mm`, **Run**.
2. Data ▸ **Curve Catalog**: search the MM\_ curves.
3. Optional deep check: Data ▸ **DB Inspector** ▸ computed*curves — filter curve_name LIKE 'MM*%' / 'mm*%'.
   **Expected:** the run writes to the SAME uppercase MM_PHIE/MM_PHIT/… names (prefix is canonicalized); the catalog shows ONE row per MM* curve at a bumped version — no duplicate lowercase "mm*\*" rows, and plots of MM_SWT show the fresh values.
   **Known issue:** AUDIT-2026-07-21 "SandiMin's free-text Output Prefix isn't case-normalized, giving the confirmed db-write-versioning-discipline bug a second live trigger" — fixed in the uncommitted batch-1 (REVIEW.md §Round 4 batch 1, items (6)+(7): prefix upper-cased + case-insensitive DELETE). Expect PASS; duplicate `mm*_`/`MM\__` rows or a plot showing stale values is that finding — log as known.

   **Automated coverage - pinned (pile B, 2026-07-31):** `a_re_run_under_a_lowercase_prefix_leaves_no_shadow_rows` (multimin2.rs) runs the real solver twice on a forward-modelled well, first with prefix `MM` and then with `mm`, and confirms both halves of the fix: the outputs are named MM_* (never mm_*), not one lowercase row survives in computed_curves, MM_PHIE has exactly one row per depth, and what a reader gets back is the SECOND run's answer. The second run deliberately changes an endpoint so the numbers really move — a re-run producing identical values could not tell a live row from a shadow.

   **Worth knowing before you click:** the fix is in two places and either alone would leave the bug, so check both symptoms. `run_multimin` upper-cases the prefix before naming anything, and the computed-curve write DELETEs on `upper(curve_name)` so a prior-casing row is reclaimed rather than left behind. The reason a shadow row is dangerous rather than merely untidy is that every curve reader resolves names case-insensitively: a stale `mm_PHIE` can win against a fresh `MM_PHIE`, so a plot or a downstream module would show the FIRST run's answer while the catalog shows a fresh run at a bumped version. Step 3's DB Inspector filter is the direct way to see it.
   **Result — T-ADV-17:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-18 — SandiMin negative tests: too few components / under-determined / empty scope

**Tool/panel:** SandiMin pane
**Preconditions:** SandiMin pane open.
**Steps:**

1. Untick every component except Quartz; **Run** → status "SandiMin: select at least two components".
2. Re-tick a 5–6-component model (Quartz, Illite, Kaolinite, Water Sxo, Water Sw, Gas Sw) but untick tools until only **Formation Density** and **Neutron** remain; **Run**.
3. Restore tools; set **Apply to wells** to ★ Pinned with nothing pinned; **Run** → "No wells in scope — pick a group, pin/select wells, or choose All".
   **Expected:** step 2 refuses with "need at least N input logs to constrain M components (have 2)" (covers REVIEW.md §P0 "SandiMin refuses under-determined models") — no curves written, no History entry for any refused run; the pane stays usable afterwards (Run re-enabled).
   **Result — T-ADV-18:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-ADV-19 — Theme switch repaints open Advance panes

**Tool/panel:** Project tab ▸ Appearance ▸ Theme; SSC pane + SandiMin pane
**Preconditions:** SSC module pane and SandiMin pane both open with results visible.
**Steps:**

1. Project tab ▸ **Theme** ▸ **Dark**.
2. Then **Pertamina**; then back to **Default**.
   **Expected:** on each switch, both panes repaint immediately in the new palette — SandiMin's endpoints matrix, group headers, result table and the module form controls all restyle with no white/stale patches and no reopen needed; any open Log View/Crossplot showing SSC/SandiMin curves repaints too.
   **Result — T-ADV-19:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

# Section RT — Rock typing, HFU, Pc fit, SHF fit, facies tie-in

## Cluster RT — Rock Typing feature set (Wave B item 8, increments 1–2)

Covers the four Rock Typing modules on the Petrophysics ribbon (`rocktyping`, `lucia_rfn`, `pittman_rx`, `rt_cutoff` — src-tauri/src/rocktyping.rs), the four workspace panes shipped with them (SHF Fit, Pc Fit (Thomeer), HFU Clustering, Facies Tie-in — shf_fit.rs, thomeer.rs, hfu.rs, facies_tie.rs), and the legacy 4-component Multimin (multimin.rs). Everything here is flagged in REVIEW.md Round 3 as "**Not yet clicked through in the real app with field data**", so these are first-contact tests: run them on a cored Mahakam well with computed PHIE, a permeability curve, imported routine core (Data ▸ Import Data ▸ Import Core…) and imported SCAL Pc (Data ▸ Import Data ▸ Import SCAL…).

---

### T-RT-01 — Smoke: Rock Typing ribbon group lists all four modules and opens their panes

**Tool/panel:** Petrophysics ribbon ▸ "Rock Typing" dropdown (src/ui/ribbon.ts `renderCategoryModules`, category "Rock Typing"; specs in src-tauri/src/rocktyping.rs)
**Preconditions:** Any project open (wells imported); app started with `npm run tauri dev`.
**Steps:**

1. Click the **Petrophysics** ribbon tab.
2. Find the group captioned **Rock Typing** and click its dropdown button (label **Rock Typing**).
3. Confirm the menu lists exactly: **Rock Typing (FZI / R35 / PGS)**, **Lucia Rock-Fabric Number (carbonate)**, **Pittman Pore-Throat Radii (r10–r75)**, **Rock Type from Cutoffs (electrofacies)**.
4. Click each entry in turn; a dock pane opens per module. In the **Rock Typing (FZI / R35 / PGS)** pane check: a Wells scope row (mode buttons **Group / ★ Pinned / Selection / All / Custom…**), a **METHOD** select with entries `ghe` and `winland_port`, a **PS_EXP [-]** numeric (default 3.5), **PHI** and **PERM** curve selects, **Mask (optional)**, **Input cons** (default "(latest values)"), **Output cons** (default "INTERP"), the note "Outputs: RQI, PHIZ, FZI, R35, PGEOM, PSTRUC, RT, PERM_RT", and a **Run** button.
5. Click the same ribbon entry again — the existing pane is focused, not duplicated (singleton).
   **Expected:** All four entries present with those exact titles; each opens a parameter pane without errors; re-clicking never duplicates a pane. Legacy "Multimin — Mineral Inversion" does NOT appear anywhere in the Petrophysics tab or the Advance tab.
   **Result — T-RT-01:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-02 — Smoke: the four rock-typing workspace panes exist in the ＋ add-panel menu

**Tool/panel:** Workspace tab-bar **＋** button menu (src/ui/workspace.ts `showAddPanelMenu`)
**Preconditions:** Any project open.
**Steps:**

1. Click the **＋** button on a window's tab bar (tooltip "Add a panel to this window").
2. Confirm the menu contains: **SHF Fit (Cuddy FOIL)**, **Pc Fit (Thomeer)**, **HFU Clustering (FZI)**, **Facies Tie-in (RT confusion)** (plus **Processing History** further down).
3. Open each of the four; check the tab titles read **SHF Fit (Cuddy FOIL)**, **Pc Fit (Thomeer)**, **HFU Clustering (FZI)**, **Facies Tie-in**.
4. Re-pick one from the ＋ menu of a second window — the singleton pane MOVES to that window instead of duplicating.
   **Expected:** All four entries present with those labels; each opens its pane (empty-state, no crash even with no core/SCAL yet); singletons move rather than duplicate.
   **Result — T-RT-02:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-03 — Rock Typing (FZI / R35 / PGS) run, GHE method: outputs, catalog provenance, History, plot refresh

**Tool/panel:** "Rock Typing (FZI / R35 / PGS)" module pane (src-tauri/src/rocktyping.rs `rocktyping`; pane host src/ui/moduleDialog.ts)
**Preconditions:** One cored well selected; a computed **PHIE** and a permeability curve (e.g. PERM/PERM_COATES) exist for it. A Log View displaying PERM is open.
**Steps:**

1. Petrophysics ▸ Rock Typing ▸ **Rock Typing (FZI / R35 / PGS)**. Scope: **Selection** (the one well).
2. Leave **METHOD** = `ghe`, **PS_EXP** = 3.5. Set **PHI** = PHIE and **PERM** = your permeability curve. Output cons **INTERP**.
3. Click **Run**. The result line reads "Running rocktyping on 1 well(s)… see the Processing panel for progress" and the Processing panel pops open.
4. Data ▸ **Curve Catalog** (button tooltip "Open the Inspector (equations + curve catalog)"). In the catalog table find new rows **RQI, PHIZ, FZI, R35, PGEOM, PSTRUC, RT, PERM_RT** — check the **Set** column shows `INTERP v1` and **Module / Source** names the module.
5. Check the open Log View / any open crossplot refreshed without a manual reload (dataVersion bump), and that FZI etc. are now pickable in plot curve dropdowns.
6. Open ＋ ▸ **Processing History**: a new entry "Ran Rock Typing (FZI / R35 / PGS)" attributed to the well actually run.
7. Domain sanity from the catalog Min/Max/Mean columns: **FZI Min > 0** (never negative/−Inf), **R35** within ~0.05–50 µm for Mahakam sands (meso/macro for good sand, micro in silts), **RT** Min ≥ 1 and Max ≤ 10 with only whole-number classes, **PERM_RT Min > 0**.
8. Crossplot PERM_RT vs PERM (log-log): points should scatter around 1:1 within roughly half a decade for samples inside a coherent GHE class.
   **Expected:** All 8 curves written and versioned into INTERP; provenance correct; open plots refresh automatically; History entry recorded; domain checks in steps 7–8 hold (RT ordinal — higher class = higher FZI = better rock).
   **Result — T-RT-03:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-04 — Re-run with METHOD = winland_port: port classes and version N+1 (no overwrite)

**Tool/panel:** Same pane as T-RT-03
**Preconditions:** T-RT-03 passed (INTERP v1 exists).
**Steps:**

1. In the same pane change **METHOD** to `winland_port`. Click **Run**.
2. In the Curve Catalog confirm the rock-typing rows now show **Set** `INTERP v2` — v1 is still listed in the log-set version history, not overwritten.
3. Check **RT** now spans 1..5 only (Winland port classes nano→mega), no longer 1..10.
4. Cross-check one depth by hand: R35 ≈ 6–7 µm must give RT = 4 (macro, 2.5–10 µm band).
   **Expected:** Re-run creates version 2 alongside version 1 (the pane's own hint: "a re-run becomes version N+1, never overwriting"); RT re-binned to the 5 port classes consistent with the R35 curve.
   **Result — T-RT-04:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-05 — Negative: run rocktyping on a well with no permeability curve

**Tool/panel:** Same pane; runner in src-tauri/src/modules.rs
**Preconditions:** A second well in the project that has PHIE but NO permeability curve.
**Steps:**

1. Scope the pane to only that well (Custom… ▸ tick it alone).
2. Keep **PERM** pointing at the permeability mnemonic that exists only on the cored well. Click **Run**.
3. Watch the Processing panel per-well breakdown and the pane's one-line outcome.
   **Expected:** A clean per-well failure (✗/⚠ with an error naming the input problem) — no crash, no freeze. The Curve Catalog for that well gains NO FZI/RT rows, and no half-written curves appear. Catalog Min/Max of existing curves unchanged (no ±Inf poisoning).
   **Known issue — CONFIRMED 2026-07-31, the catalog part of Expected will NOT hold.** The per-well
   failure IS clean and named, and no half-written values appear - but the Curve Catalog DOES gain
   rows. A run whose every output is MISSING still writes and versions the whole family (RQI, PHIZ,
   FZI, R35, PGEOM, PSTRUC, RT, PERM_RT), blank from top to bottom. Measured: rows for all eight,
   finite values in none. So expect eight new catalog entries with n = 0, not none at all. Nothing
   is corrupted and Min/Max are not poisoned; what is lost is the catalog's ability to tell "never
   run" from "run and could not answer". Log as known. See docs/review_triage.md finding 10.

   **Automated coverage - pinned (pile B, 2026-07-31):** `rocktyping_without_a_permeability_curve_fails_and_writes_no_curves` (workflow.rs) - it asserts the clean failure, and pins the empty-curve write AS-IS, with the same well plus a permeability curve as the control.

   **Result — T-RT-05:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-06 — Lucia Rock-Fabric Number on a well with carbonate stringers

**Tool/panel:** "Lucia Rock-Fabric Number (carbonate)" module pane (src-tauri/src/rocktyping.rs `lucia_rfn_module`)
**Preconditions:** A well with carbonate stringers, PHIE + permeability computed. (REVIEW.md item (8) increment 2 first chunk, unchecked: "**Try:** run it on a well with carbonate stringers.")
**Steps:**

1. Petrophysics ▸ Rock Typing ▸ **Lucia Rock-Fabric Number (carbonate)**.
2. Set **PHI** = PHIE (note: the doc says PHI should be INTERPARTICLE porosity — subtract vug porosity if you have it), **PERM** = permeability. **Run**.
3. Curve Catalog: new rows **RFN** and **RT_LUCIA**.
4. Domain sanity: RT_LUCIA takes only values 1, 2, 3 (grainstone → mud-dominated); catalog **n** (sample count) for RT_LUCIA is much smaller than for RFN's well coverage on a clastic-dominated well — samples with RFN outside the calibrated 0.5–4 band must be MISSING, not clamped to 1 or 3.
5. Against the stringer interval on a log view: the carbonate streaks should carry the populated RT_LUCIA values; the clastic background should be blank.
   **Expected:** RFN + RT_LUCIA written; class values strictly in {1,2,3}; out-of-band → MISSING; populated only where the rock is actually carbonate-like. History entry recorded.
   **Result — T-RT-06:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-07 — Rock Type from Cutoffs: RT_LOG ladder + inconsistent-cutoff behavior

**Tool/panel:** "Rock Type from Cutoffs (electrofacies)" module pane (src-tauri/src/rocktyping.rs `rt_cutoff`)
**Preconditions:** VSH and PHIE computed on the cored well. (Feeds T-RT-15; REVIEW.md electrofacies tie-in item, unchecked: "**Try:** run `rt_cutoff` to make RT_LOG, then Facies Tie-in ▸ RT_LOG vs your core RT.")
**Steps:**

1. Petrophysics ▸ Rock Typing ▸ **Rock Type from Cutoffs (electrofacies)**. Defaults: **VSH1** 0.15, **PHI1** 0.12, **VSH2** 0.35, **PHI2** 0.06; **VSH** = your Vsh curve, **PHIE** = PHIE. **Run**.
2. Curve Catalog: new **RT_LOG** row; Min = 1, Max = 3, integer classes only.
3. Display RT_LOG next to VSH/PHIE in a log view. Spot-check three depths: clean+porous → 1, moderate → 2, shaly or tight → 3; depths with missing Vsh or PHIE → blank.
4. Negative sub-check: set **VSH1** = 0.50 and **VSH2** = 0.20 (violates the doc's "Requires VSH1 ≤ VSH2") and **Run**. Record what happens — the pane validates only the 0–1 range per field, so the run is expected to proceed; note whether any warning appears and whether the resulting ladder is self-contradictory.
5. Restore defaults and re-run so a sane RT_LOG (new version) exists for T-RT-15.
   **Expected:** Steps 1–3: correct 1/2/3 ladder honoring the cutoffs, MISSING propagated. Step 4: no crash; document the (current) silent acceptance of an inconsistent ladder in Notes — candidate for a cross-field validation ticket.
   **Update 2026-07-31 — step 4 is worse than silent acceptance, so look closely.** The inverted
   ladder does not just shift classes, it SCATTERS the middle one. Because RT1 is tested first and
   its Vsh gate is now the looser one, moderately shaly rock splits: the porous half is PROMOTED to
   class 1 (best) and the tighter half is DEMOTED to class 3 (non-net), in the same run, with no
   warning. Worth noting in your Notes as a cross-field validation ticket, since RT_LOG feeds the
   facies tie-in in T-RT-15.

   **Automated coverage - pinned (pile B, 2026-07-31):** `an_inverted_cutoff_ladder_is_accepted_and_scatters_the_middle_class` (rocktyping.rs) - the sane 1/2/3 ladder and MISSING propagation, then the inverted case pinned AS-IS.

   **Result — T-RT-07:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-08 — Pittman Pore-Throat Radii: r10–r75 family + APEX selector

**Tool/panel:** "Pittman Pore-Throat Radii (r10–r75)" module pane (src-tauri/src/rocktyping.rs `pittman_rx`)
**Preconditions:** PHIE + permeability on the cored well. (REVIEW.md Pittman/HFU item, unchecked: "**Try:** run `pittman_rx` (pick APEX) for the radius family…")
**Steps:**

1. Petrophysics ▸ Rock Typing ▸ **Pittman Pore-Throat Radii (r10–r75)**. Leave **APEX** = `r35`. **Run**.
2. Curve Catalog: 11 new rows — **PR10, PR15, PR20, PR25, PR30, PR35, PR40, PR50, PR75, RAPEX, RT_PITT**.
3. Domain sanity at any good-sand depth: the radius family must decrease monotonically with mercury saturation — PR10 > PR25 > PR35 > PR50 > PR75 (larger throats invade first). RAPEX must exactly equal PR35. RT_PITT in 1..5.
4. Cross-check PR35 vs the Winland **R35** from T-RT-03 on a crossplot — same order of magnitude, correlated but not identical (different regressions).
5. Change **APEX** to `r50`, **Run** again (version N+1): RAPEX now tracks PR50 and RT_PITT re-bins accordingly (generally one class finer or equal).
   **Expected:** Full family written; monotone ordering holds everywhere both curves are populated; RAPEX follows the chosen APEX row; invalid samples (φ∉(0,1), k≤0) blank in ALL eleven outputs.
   **Known issue — CONFIRMED 2026-07-31, step 3's ordering does NOT hold at the r50/r75 end.**
   PR10 > PR15 > ... > PR50 holds as written. PR75 does not: above roughly **79 mD at 25 % porosity**
   the table returns a LARGER radius at 75 % mercury than at 50 %, which cannot happen in rock -
   mercury enters the widest throats first. Measured at that point: PR50 2.907 um against PR75
   2.953 um. At 1 mD the same pair is the right way round, so it is the coefficients, not a bad
   sample. The nine rows are independent regressions with nothing forcing them to agree, and the
   module doc already flags the full set as transcribed from Pittman 1992 and to be verified before
   field release - this is that verification failing. It reaches the outputs: choosing APEX = r75
   for fine rock, which the doc recommends, builds RAPEX and RT_PITT on the inverted value. Log as
   known. Fixing it needs the paper in hand. See docs/review_triage.md finding 9.

   **Automated coverage - pinned (pile B, 2026-07-31):** `the_pittman_radius_family_inverts_between_r50_and_r75_in_good_sand` (rocktyping.rs) pins the monotone head, the inversion above with its measured numbers, and that an invalid sample blanks all ELEVEN outputs. RAPEX, the APEX selector and the port class were already covered by `pittman_r35_matches_published_regression`, `pittman_apex_selector_switches_controlling_radius` and `pittman_missing_inputs_stay_missing`.

   **Result — T-RT-08:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-09 — HFU Clustering (Ward) on core φ-k: table, crossplot, histogram, highlight, theme repaint

**Tool/panel:** **HFU Clustering (FZI)** pane (src/ui/hfuDialog.ts; backend src-tauri/src/hfu.rs)
**Preconditions:** Routine core analysis imported for at least one scoped well (Data ▸ Import Data ▸ **Import Core…**). This pane reads the `core_data` φ-k, NOT log curves — the pane's own hint says "Import core data first." (REVIEW.md, unchecked: "**Try:** … HFU Clustering (FZI) ▸ pick Ward or Histogram + K ▸ Cluster — check the RQI–φz unit-slope lines and the FZI histogram breaks against your rock types.")
**Steps:**

1. ＋ ▸ **HFU Clustering (FZI)**. Scope to the cored well(s). Leave **HFUs (K)** = 5, **Method** = **Ward (min-variance)**.
2. Click **Cluster HFUs**. Status line reads like "5 HFU(s) from N plug(s) • X ms" (plus ", M plug(s) skipped" if some plugs had invalid φ/k).
3. Per-HFU table: HFU ids contiguous 1..K, **HFU 1 = lowest FZI** (poorest rock), FZI min/max bands non-overlapping and ascending, φ mean and the perm-transform R² populated.
4. RQI–φz crossplot: unit-slope FZI_gm lines, one per HFU, colour-matched to the points; lines stay inside the plot frame (must not paint over axis labels).
5. log₁₀ FZI histogram: K−1 cut lines, each sitting between populated bars, never inside an empty gap.
6. Click a table row: that HFU highlights and the others dim in BOTH plots; click again to clear. Resize the pane — canvases redraw sharp, not stretched.
7. Project tab ▸ **Theme** ▸ pick **Dark** (or back): after clicking a row (forcing a redraw) the two canvases render in the new palette, legible in both themes.
8. Curve Catalog: unchanged (this pane is read-only, writes no curves). Processing History: a "RockType" entry "HFU cluster (ward): …".
9. Domain acceptance: compare the FZI histogram breaks and the per-HFU FZI bands against your known rock types for this well — the K=5 partition should not split an obviously single population.
   **Expected:** All of steps 2–8 as described; clustering is reproducible (re-clicking Cluster HFUs with the same inputs gives identical boundaries).
   **Result — T-RT-09:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-10 — HFU Clustering: histogram method, K cap, and the no-core negative

**Tool/panel:** Same pane as T-RT-09
**Preconditions:** T-RT-09 done; also one well in the project with NO core data.
**Steps:**

1. Switch **Method** to **Histogram (antimodes)**, keep K = 5, **Cluster HFUs**. Compare boundaries with the Ward result — they may differ, but ids must again be contiguous 1..K with no empty interior HFU.
2. Set **HFUs (K)** = 12 on a small plug set and re-cluster: the result may return fewer units than requested (capped to distinct FZI levels / natural breaks) — the status line reports the actual count, and the table has no empty rows.
3. Set **HFUs (K)** = 1 and click **Cluster HFUs**: rejected in-app with "HFUs (K) must be at least 2" (status bar), no backend call.
4. Re-scope to only the core-less well and **Cluster HFUs**.
   **Expected:** Step 4 fails gracefully with the exact backend message: "no core plugs with valid φ (0–1) and k (>0) in the selected wells — import core data first" shown as "Failed: …" in the pane's status line; previous results are cleared, no crash.
   **Result — T-RT-10:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-11 — Pc Fit (Thomeer): per-plug fit, Hg-equivalent standardization, Swanson-k suppression

**Tool/panel:** **Pc Fit (Thomeer)** pane (src/ui/thomeerDialog.ts; backend src-tauri/src/thomeer.rs)
**Preconditions:** SCAL Pc imported for the scoped well(s) via Data ▸ Import Data ▸ **Import SCAL…** with the **Fluid system** select set per lab system (e.g. Air-mercury 367 for MICP, Air-brine 72 for porous plate); plug porosity present. Ideally import BOTH an air-brine set and a mercury set (separate imports — the dialog warns one lab fluid system per import). (REVIEW.md Thomeer item, unchecked: "**Try:** import MICP as Air-mercury ▸ Pc Fit (Thomeer) ▸ Fit — check the Pd–G clusters against your rock types.")
**Steps:**

1. ＋ ▸ **Pc Fit (Thomeer)**. Scope the SCAL wells. Click **Fit Thomeer**.
2. Status line: "N plug(s) fitted" plus "(M plug(s) skipped — no φ or too few points)" when applicable — skipped plugs are counted, never silently dropped.
3. Per-plug table columns: Well, Sample, Depth, φ, k mD, System, Pd psi, G, Bv∞, R², n, Swanson k. Click different rows — the row highlights and the "Selected plug: Bv vs Pc (log) with the fitted Thomeer hyperbola" plot re-draws for that plug; the "Pd–G plane (all plugs; selected highlighted) — the Thomeer rock-typing crossplot" highlights the same plug.
4. Domain acceptance per plug: G mostly 0.1–1 (lower = better sorted); Bv∞ ≤ φ (in the same units, approximately); the fitted hyperbola visually follows the Bv-Pc points; R² high for clean MICP curves; Swanson k within roughly an order of magnitude of the measured "k mD" column.
5. Standardization cross-check: a twin air-brine plug and mercury plug from the same rock should land at comparable **Pd psi** on the shared Pd–G plane (air-brine Pc is converted ×367/σcosθ to Hg-air equivalent BEFORE fitting).
6. Suppression path: for any rows whose imported points lack a recorded σcosθ (legacy imports, or "Other" system with the field cleared), the System column shows the "(raw)" suffix and **Swanson k is "—"** (suppressed, since it would be 16–88× wrong on raw Pc).
7. Artifact flag: if any plug shows "⚠" beside Pd, hover it — the tooltip explains Pd is pinned at a search bound (entry-truncated curve); treat that Pd as unresolved, not a real entry pressure.
8. Processing History: "RockType" entry "Thomeer fit: N plug(s)…". Curve Catalog unchanged (read-only pane).
   **Expected:** Fits, plots, row-selection, standardization (step 5), "(raw)"/Swanson suppression (step 6), and the Pd ⚠ flag behave exactly as described; Pd–G clusters group consistently with your known rock types.
   **Result — T-RT-11:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-12 — SHF Fit — Cuddy FOIL with FWL scan

**Tool/panel:** **SHF Fit (Cuddy FOIL)** pane (src/ui/shfDialog.ts; backend src-tauri/src/shf_fit.rs)
**Preconditions:** Computed PHIE and SW (SWE/SWT) and a TVDSS curve (Data ▸ Import Data ▸ **Import Deviation…**) on ≥1 well with a hydrocarbon column; an independent FWL estimate (DST/pressure gradient) to judge the scan against.
**Steps:**

1. ＋ ▸ **SHF Fit (Cuddy FOIL)**. **SHF form** = **Cuddy FOIL (BVW = a·H^b)**. Set **Porosity (φ)** = PHIE, **Water saturation (Sw)** = your Sw curve, **TVDSS** = TVDSS. Scope the wells.
2. Enter your best **Free-water level (TVDSS)** and a **Min φ (net cutoff)** (e.g. 0.06). Leave the scan OFF. Click **Fit FOIL**.
3. Results: a table with "a (BVW at H=1)", "b (slope)", "R² (log space)", "points fitted", "FWL used (TVDSS)", and the "BVW vs height above FWL (log–log) with the fitted FOIL line" crossplot — points should hug the line.
4. Domain acceptance: **b < 0** (BVW shrinks with height above FWL); R² respectable for a single rock-type pool; a in a physically sensible BVW range at H=1 m.
5. Tick **Scan for FWL (Cuddy Eq 19)**, set **FWL lo / FWL hi** bracketing your estimate and **step** 0.5. **Fit FOIL** again.
6. The extra "FWL scan — fit residual vs candidate free-water level (Cuddy Eq 19)" plot shows a clear residual minimum; "FWL (scan best)" appears in the table, the dashed marker sits at the minimum, and the **Free-water level (TVDSS)** input auto-fills with the best value.
7. Domain acceptance: the scanned FWL within a few metres of your DST/pressure-derived contact.
8. Processing History: "SHF" entry "Cuddy FOIL fit: BVW=…·H^… (R²=…)". Curve Catalog unchanged (writes no curves — the (a, b) law is for the forward sw_height apply).
   **Expected:** Fit and scan behave as described, with the domain checks in steps 4 and 7 holding.
   **Result — T-RT-12:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-13 — SHF Fit — Brooks-Corey and Skelt-Harrison forms

**Tool/panel:** Same pane (REVIEW.md SHF-forms item, unchecked: "**Try:** SHF Fit ▸ pick Brooks-Corey / Skelt-Harrison.")
**Preconditions:** T-RT-12 done (inputs + FWL known).
**Steps:**

1. Change **SHF form** to **Brooks-Corey**. The **FWL scan** row hides and the run button relabels to **Fit SHF**.
2. Click **Fit SHF**. Results: a parameter table (Swirr, λ-type params + R² + points fitted) and the "Sw vs height above FWL (log H) with the fitted brooks_corey curve" plot.
3. Domain acceptance: Swirr in [0, 1] and plausible for the rock (Mahakam shaly sand — expect it noticeably above zero); the fitted curve is monotone (Sw falls with height, flattening toward Swirr); the overlay tracks the cloud's trend.
4. Switch to **Skelt-Harrison**, **Fit SHF**: fitted curve monotone decreasing toward 1−A at large H, R² reported; overlay sensible.
5. Both runs write a "SHF" History entry naming the method and parameters.
   **Expected:** Form selector drives button label + scan-row visibility; both fits converge on real field data with monotone Sw-height curves overlaying the scatter; parameter values physically plausible.
   **Result — T-RT-13:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-14 — Negative: SHF Fit with a starved point cloud

**Tool/panel:** Same pane
**Preconditions:** T-RT-12 done.
**Steps:**

1. Set **Min φ (net cutoff)** = 0.40 (excludes essentially everything) and **Fit FOIL**.
2. Repeat with **Brooks-Corey**.
3. Then set the scope to a well with no TVDSS or no Sw curve and try once more.
   **Expected:** Each case fails cleanly with "Failed: …" in the pane's status line (the fitters reject too-few/degenerate input per their unit-tested guards); previous results are cleared, not left stale; no crash and no nonsense parameters (e.g. no NaN/Inf rows in the table).
   **Result — T-RT-14:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-15 — Facies Tie-in: RT_LOG vs reference RT confusion matrix + purity

**Tool/panel:** **Facies Tie-in (RT confusion)** pane (src/ui/faciesTieDialog.ts; backend src-tauri/src/facies_tie.rs)
**Preconditions:** RT_LOG exists (T-RT-07) and a reference rock-type curve exists on the same wells — your core-derived RT curve if imported, else the RT from T-RT-03. (REVIEW.md electrofacies tie-in item, unchecked, cited in T-RT-07.)
**Steps:**

1. ＋ ▸ **Facies Tie-in (RT confusion)**. **Predicted RT (log)** pre-selects RT_LOG; set **Reference RT (core)** to your core RT (the dropdown prefers RT / RT_LUCIA / FACIES / FACIES_ML). Scope the wells.
2. Click **Compare**. Status: "Overall purity XX.X% over N matched samples".
3. Results: the per-class table (columns "Ref class", "→ dominant pred", "purity", "n") and the "Confusion matrix (row = reference, col = predicted)" with the dominant cell per row emphasized.
4. Cross-check: each matrix row's cell counts sum to that row's "n"; the "n" values sum to the status line's N.
5. Domain acceptance: the best reference class should dominantly map to RT_LOG = 1 and the non-net class to RT_LOG = 3 — an inverted mapping means the ladder cutoffs (or the reference curve) are wrong, and RT_LOG must NOT be trusted for uncored intervals.
6. Negative: set both dropdowns to the same curve and click **Compare** — rejected in the status bar with "Pick two different curves (predicted vs reference)", no backend call.
7. Negative: scope a well where the two curves never coexist at a depth — expect "Failed: no matched samples where both curves are present".
8. Processing History: "RockType" entry "Facies tie-in: RT_LOG vs …, purity …%".
   **Expected:** Matrix + purity computed and internally consistent (step 4); both guards fire (steps 6–7); History recorded.
   **Result — T-RT-15:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-16 — Legacy Multimin: deprecated entry filtered from the Workflow step picker (audit-fix regression)

**Tool/panel:** Workflow Builder step picker (Petrophysics ▸ **Workflow…** button; src/ui/workflowDialog.ts `DEPRECATED_STEP_MODULES`)
**Preconditions:** Any project open.
**Steps:**

1. Petrophysics ▸ **Workflow…** to open the Workflow Builder.
2. Open the **Add module** dropdown and expand the **Saturation** optgroup.
3. Confirm the deprecated **Multimin — Mineral Inversion** is NOT listed (sw_arch/sw_indo/sw_sim/sw_rtc/sw_imts remain).
4. Also confirm SandiMin is not offered as a chain step anywhere in the picker (it remains a standalone pane — Advance tab / ＋ ▸ SandiMin Solver — by design, pending a separate chain-composability decision).
5. Cross-check the ribbon: neither the Petrophysics Saturation dropdown nor the Advance tab shows a "Multimin — Mineral Inversion" / "Mineral Inv" button.
   **Expected:** The deprecated solver is unreachable from any new-chain or ribbon path; the only remaining route is loading a pre-existing saved workflow that already references it (see T-RT-17). This verifies the fix REVIEW.md records as an unchecked click-through item: "the deprecated legacy `multimin` module is filtered out of the Workflow step picker (use SandiMin)."
   **Known issue:** AUDIT-2026-07-21-full-qc.md, Legacy multimin finding 1: "Workflow Builder's step picker exposes the deprecated multimin module unfiltered/unlabeled, while SandiMin has no path into chains at all … a user building a brand-new chain today can only silently add the deprecated fixed 4-component solver … with no UI signal". The picker half was fixed post-audit (workflowDialog.ts now filters `multimin`) — this test confirms the fix holds; the SandiMin-not-chainable half is unchanged by design.
   **Automated coverage - end-to-end (pile C, 2026-08-01):** **step 5 only.** `shell.e2e.mjs`
   "gives the legacy fixed multimin no button in any ribbon tab" sweeps every `.ribbon-label` in
   the whole ribbon. That is the right shape for this claim, because the retirement rests on TWO
   independent mechanisms — membership of `Ribbon.ADVANCED_MODULE_IDS`, which filters it out of the
   Petrophysics dropdowns, and a META caption outside `groupOrder`, which keeps it off the Advance
   tab — so breaking either one puts the button back in a different place, and checking one tab
   would catch only one. **Steps 1–4 (the Workflow Builder's own step picker) are NOT covered** and
   remain yours.

   **Result — T-RT-16:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-17 — Legacy Multimin v1 run (via a pre-existing saved chain): VOL\_\* + RECON_ERR outputs

**Tool/panel:** "Multimin — Mineral Inversion" module (src-tauri/src/multimin.rs), executed through a saved Workflow chain (src-tauri/src/workflow.rs)
**Preconditions:** A saved workflow document that already contains a **Multimin — Mineral Inversion** step (created before the 2026-07-22 picker filter). If the test project has none, mark **Blocked** — since the fix, there is deliberately no UI path to add it to a new chain (T-RT-16). Test well has RHOB, NPHI, DT and PEF.
**Steps:**

1. Petrophysics ▸ **Workflow…** ▸ **Saved** dropdown ▸ pick the legacy chain ▸ **Load**. The multimin step should render in the step list, resolving its title normally (saved chains still dispatch it — backward compatibility).
2. Scope one well with all four logs. Click **Run chain**; watch the Processing panel.
3. Curve Catalog: new rows **VOL_SAND, VOL_CLAY, VOL_WATER, VOL_HC, PHIT_MM, VSH_MM, SWT_MM, RECON_ERR**.
4. Domain acceptance at several depths: all four volumes ≥ 0; VOL_SAND+VOL_CLAY+VOL_WATER+VOL_HC ≈ 1.00 (soft unity constraint, W_UNITY 1000); PHIT_MM = VOL_WATER+VOL_HC; SWT_MM = VOL_WATER/PHIT_MM; in a known clean wet sand, VOL_CLAY small and SWT_MM ≈ 1.
5. RECON_ERR with all 4 tools live: near zero where the default endpoints fit, genuinely elevated across intervals the 4-component model can't explain (coals, heavy-mineral streaks) — this is the informative configuration.
6. Note in the Curve Catalog how easily these legacy mnemonics (PHIT_MM/VSH_MM/SWT_MM) sit beside SandiMin's MM_PHIT/MM_VSH/MM_SWT — record any mis-pick risk you notice for the follow-up ticket.
   **Expected:** The saved chain still runs the legacy solver end-to-end; all 8 outputs written with the step 4 identities holding; History entry "Ran chain (…)" recorded.
   **Result — T-RT-17:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-RT-18 — Legacy Multimin RECON_ERR at exactly 3 tools (known QC blindness)

**Tool/panel:** Same saved chain as T-RT-17; step parameter editor (gear icon, "Edit parameters for this step")
**Preconditions:** T-RT-17 runnable (else Blocked).
**Steps:**

1. In the loaded chain, open the multimin step's parameter editor and set the **PEF** input to **(none)** — the common no-PEF logging-string case in Mahakam wells.
2. Deliberately mis-set one endpoint (e.g. **RHOB_SAND** = 2.75) so the model is knowably wrong for your sand.
3. **Run chain** on the same well; inspect the new RECON_ERR version and the volume curves.
   **Expected (desired behavior):** RECON_ERR should flag the wrong-endpoint intervals (or the curve should be NaN/flagged as under-constrained at 3 tools) so the tester is never shown a silently-perfect QC on a mis-parameterized model.
   **Known issue:** AUDIT-2026-07-21-full-qc.md, Legacy multimin finding 2 (CONFIRMED, fix currently HELD per REVIEW.md "6 findings that WOULD change interpretation numbers await your sign-off (… legacy-multimin RECON_ERR at 3 tools …)"): "RECON_ERR is a near-guaranteed ~0 (uninformative) QC signal whenever exactly 3 of the 4 tools are live — the common one-log-missing case (e.g. no PEF)" — with 3 tool rows + the unity row the system is square, so "the NNLS solve is provably equal to the exact solution of that square system … so RECON_ERR … reads ~0 regardless of whether the chosen endpoints are physically right for the rock." Expect RECON_ERR ≈ 0 everywhere despite the wrong endpoint (while VOL_CLAY can be off by up to 100% relative per the verifier's numerical experiment). Mark Fail-as-predicted and record the observed RECON_ERR magnitude for the sign-off decision.
   **SUPERSEDED 2026-07-31 — do not run this one; mark it Blocked and move on.** The module it
   tests is RETIRED. `run_module` blocks legacy `multimin` and the solver body was deleted; the spec
   survives only so a saved chain still resolves by name and can show its stored parameters while
   you redo the step in SandiMin. Step 3 gets a loud "use SandiMin" refusal, not a RECON_ERR to
   read. The Known issue above is describing code that no longer exists.

   The concern itself was inherited by SandiMin and is already handled there, so **the REVIEW.md
   sign-off item for this can be dropped** - there is nothing to decide. The blindness is not a bug
   anyone can fix: with as many equations as components the solve reproduces the logs exactly
   whatever the endpoints are, so the residual cannot say anything about them. SandiMin detects the
   condition (dof = 0) and returns a note saying RECON is forced to ~0 and to add an input log.
   Measured on one well, one set of logs, CORRECT endpoints throughout: RHOB + NPHI + unity gives
   dof 0 and RECON ~0.00; adding DT and GR gives dof 2 and RECON 0.62. The square number is
   arithmetic, not fit quality.

   **What IS worth your click-through:** run SandiMin with only two tools and check the pane makes
   that note hard to miss. A warning nobody reads is the same as no warning - and that is a UI
   judgement, which is yours.

   **Automated coverage - pinned (pile B, 2026-07-31):** `multimin_is_retired_but_still_cataloged` (modules.rs) for the retirement, and `an_exactly_determined_model_hides_a_wrong_endpoint_and_only_the_dof_note_says_so` (multimin2.rs) for the inherited concern - it shows the wrong endpoint moving the clay volume while RECON does not budge, with the over-determined run as the control. See docs/review_triage.md finding 11.

   **Result — T-RT-18:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

# Section BATCH — Batch & field-scale tools

All source files verified. Composing the test plan now — grounding every label in the code just read, and citing only the still-unfixed audit findings as Known issues (several BATCH-cluster findings were already fixed in the uncommitted Round-4 tree per REVIEW.md and confirmed in code, so those tests verify the fix instead).

---

## BATCH cluster — batch/field-scale tools (Workflow Builder, Cutoffs & Pay Summary, Cutoff Sensitivity, Field Dashboard, Monte Carlo, Processing panel)

Shared preconditions: project open in `npm run tauri dev` with **≥5 wells** carrying GR + RHOB + NPHI + a deep resistivity (RT), **zones defined on ≥2 wells**, and **one well with a DST/PERF aux dataset** imported (Data ▸ Import aux data). Tests run in order — T-BATCH-02/04 create the chain and outputs that later tests consume.
Note: several audit findings in this cluster (chain-cancel dataVersion, legacy-multimin step picker, Pay-Summary History entry, cutoff-sweep NTG>1, per-well isolation) are already fixed in the current uncommitted tree (REVIEW.md §Round 4, verified in source) — the tests below double as their click-through verification; only the three still-open Monte Carlo findings carry **Known issue** lines.

### T-BATCH-01 — Workflow Builder smoke: pane opens, step picker clean

**Tool/panel:** Workflow Builder (src/ui/workflowDialog.ts; ribbon button in index.html `#workflow-btn`)
**Preconditions:** project open; any wells.
**Steps:**

1. Ribbon ▸ **Petrophysics** tab ▸ **Batch** group ▸ click **Workflow…**.
2. Confirm a docked pane titled **Workflow Builder** opens (movable/floatable tab, not a popup).
3. Click **Workflow…** again — no second pane; the existing one is focused (covers REVIEW.md §"Pane layout + MC/workflow polish" run-dialogs-are-singleton-panes item).
4. Open the **Add module** dropdown; scroll every category group (VSH, Porosity, Saturation, Permeability, …).
   **Expected:** Steps area shows "No steps yet — add modules above." Module picker lists modules grouped by category with full titles. **No "Multimin" entry appears anywhere in the picker** — the legacy solver is filtered out (use SandiMin from the Advance tab); covers REVIEW.md §Round 4 "the deprecated legacy `multimin` module is filtered out of the Workflow step picker". SandiMin itself is also absent from the picker (by design — it is not chainable yet).
   **Result — T-BATCH-01:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-02 — Compose and run the 4-step chain across 5+ wells with live progress

**Tool/panel:** Workflow Builder + Processing panel (workflowDialog.ts, processingPanel.ts)
**Preconditions:** ≥5 wells with GR/RHOB/NPHI/RT; zones on ≥2 wells.
**Steps:**

1. In **Add module**, pick **VSH from Gamma Ray** ▸ **+ Add step**. Repeat for **Porosity from Density-Neutron**, **SW — Indonesia (Poupon-Leveaux)**, **Permeability — Coates** (order matters — note the helper text "Steps run top-to-bottom; later steps use earlier outputs").
2. Click a step's **⚙** to expand; confirm sw_indo's PHIE input resolves to the phi_dn output (module outputs are selectable as inputs even in a fresh project).
3. In the **Wells** scope row pick **All** (or a Group covering ≥5 wells); leave **Input cons** = "(latest values)", **Output cons** = `INTERP`.
4. Click **Run chain**.
   **Expected:** The **Processing** panel pops open automatically (also reachable via **Project ▸ Monitor ▸ Processing**). It shows a "Workflow" job with a live progress bar, "Running", the current well name, and per-well **✓** outcomes accumulating; the builder's status line reads "Step k/4: … — see Processing panel". App stays responsive throughout (covers REVIEW.md §"Workflow chain runs without freezing the app + live progress works" and §"Processing panel"). On finish: status "Done: 4 steps, N curves across M wells"; Processing job phase **Done** with all wells ✓. Domain check in a Log View: VSH in [0,1], high in shales, low in clean sand; PHIE in [0,~0.35], anti-correlated with VSH; SWE in [0,1], low in the known pay; PERM positive, spanning ~0.1–1000s mD in sands and near-zero in shales (Coates needs the earlier PHIE — all-NaN PERM means chaining failed). History (**Project ▸ Monitor ▸ History**) shows "Workflow — Ran chain (4 step(s) × M well(s))".
   **Result — T-BATCH-02:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-03 — Chain outputs: Curve Catalog provenance, versioning, open-plot refresh

**Tool/panel:** Workflow Builder + Inspector ▸ Curve Catalog tab (workspace.ts, inspectorPanel.ts)
**Preconditions:** T-BATCH-02 passed; a Log View open displaying VSH or PHIE.
**Steps:**

1. Open **Inspector** ▸ **Curve Catalog** tab; find the **INTERP** constellation.
2. Check the entry's provenance: module names and the parameters used.
3. With the Log View still open and showing VSH, re-run the same chain (**Run chain**).
4. Re-check the Curve Catalog.
   **Expected:** First run created INTERP **version N**; the re-run adds **version N+1** — nothing overwritten (covers REVIEW.md §"One version per chain run"). The open Log View refreshes automatically when the run completes (dataVersion bump) — no manual reopen needed. Curve values of version N remain restorable from the Catalog.
   **Result — T-BATCH-03:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-04 — Save, reload, and delete the chain as a workflow document

**Tool/panel:** Workflow Builder (workflowDialog.ts)
**Preconditions:** the 4-step chain from T-BATCH-02 still composed.
**Steps:**

1. Edit one parameter (e.g. expand vsh_gr ▸ change GR_SH) so the saved doc has a non-default value.
2. In **Save as** at the bottom, type `UAT_CHAIN4` ▸ **Save**. Status: `Saved workflow "UAT_CHAIN4"`.
3. Close the pane, reopen **Workflow…**; the **Saved** dropdown now lists UAT_CHAIN4. Select it ▸ **Load**.
4. Toggle **List | Grid**; in Grid, confirm the edited parameter shows in its column; use the **Set all** row to write a shared parameter across steps (covers REVIEW.md §"Wave A-4: workflow grid inspector" save/reload item).
5. **Delete** removes it from the dropdown (re-save it afterwards — T-BATCH-16 needs it).
   **Expected:** Load restores all 4 steps with the edited parameter intact; status `Loaded workflow "UAT_CHAIN4" (4 steps)`; Grid and List views show identical steps; Delete confirms via status line.
   **Result — T-BATCH-04:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-05 — Cancel mid-chain: quick stop, honest status, plots still refresh

**Tool/panel:** Workflow Builder + Processing panel (workflowDialog.ts, processingPanel.ts)
**Preconditions:** UAT_CHAIN4 saved; all wells in scope (the more wells, the easier to catch mid-run); a Log View open on one of the FIRST wells in the list, showing VSH.
**Steps:**

1. **Run chain** across all wells; watch the Processing panel.
2. After 1–2 wells show ✓, click **Cancel** (either the builder's Cancel or the job's **Cancel** in the Processing panel — they share the same flag).
   **Expected:** Button flips to "Cancelling…"; the run drains within a well or two; builder status "Cancelled at step k"; Processing panel phase **Cancelled**; no stuck progress bar (covers REVIEW.md §"Cancel empties the progress bar"). Critically: wells completed **before** the cancel keep their newly written curves, and the open Log View **refreshes to show them** — a cancelled run must not leave open plots stale (covers REVIEW.md §Round 4 "dataVersion refresh … on workflow-chain cancel/fail"; this verifies the fix for audit finding "Cancelling a workflow chain never bumps dataVersion", now applied in workflowDialog.ts).
   **Result — T-BATCH-05:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-06 — Workflow negatives: no steps, empty scope, one broken well

**Tool/panel:** Workflow Builder + Processing panel
**Preconditions:** one well in the project lacking GR (or import a stub LAS with only DEPT+RHOB).
**Steps:**

1. Clear all steps (✕ each) ▸ **Run chain** → status "Add at least one step".
2. Add one step; set Wells scope to **Selection** with nothing selected ▸ **Run chain** → "No wells in scope — pick a group, pin/select wells, or choose All".
3. Scope **All** (including the GR-less well) ▸ run the 4-step chain.
   **Expected:** Steps 1–2 refuse to start, no job appears in the Processing panel. Step 3: the GR-less well shows **⚠/✗** in the Processing panel's notable-wells list with an advice line (e.g. "Check these wells have the input curves (Curve Catalog)…"), while every other well completes ✓ — one bad well must not kill the batch. Completion status counts the warnings ("— n well/step warnings").
   **Result — T-BATCH-06:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-07 — Cutoffs & Pay Summary: flags, table sanity, History, PAYFLAG version

**Tool/panel:** Cutoffs & Pay Summary pane (src/ui/summaryDialog.ts; ribbon Petrophysics ▸ Reporting ▸ **Cutoffs & Summary…**)
**Preconditions:** T-BATCH-02 outputs exist (VSH/PHIE/SWE); zones defined on ≥2 wells.
**Steps:**

1. Petrophysics ▸ Reporting ▸ **Cutoffs & Summary…** → docked pane **Cutoffs & Pay Summary**.
2. Set **VSH ≤** 0.5, **PHIE ≥** 0.1, **SWE ≤** 0.6, leave **PERM ≥ (optional)** blank ("(off)").
3. Wells scope: pick ≥3 wells ▸ **Compute Summary**.
4. Open **Inspector ▸ Curve Catalog**; then the History panel (**Project ▸ Monitor ▸ History**).
5. Re-run Compute Summary once.
   **Expected:** Table with columns Well | Zone | Flag | Top | Bottom | Gross | Net | N/G | Avg VSH | Avg PHIE | Avg SWE | HPV (m), rows for SAND/RESERVOIR/PAY per well-zone. Domain acceptance: **Net ≤ Gross** everywhere; N/G in [0,1]; per zone PAY-net ≤ RESERVOIR-net ≤ SAND-net; SAND rows' Avg VSH ≤ 0.5, RESERVOIR rows' Avg PHIE ≥ 0.1, PAY rows' Avg SWE ≤ 0.6; HPV ≤ Net × Avg PHIE. Status line "Pay summary: N rows; FLAG curves written". Cross-checks: Curve Catalog shows a **PAYFLAG** log set holding FLAG*SAND/FLAG_RESERVOIR/FLAG_PAY whose provenance records the cutoffs; the re-run makes it **version N+1** (covers REVIEW.md §"Pay-summary provenance — FLAG*\* versioned + cutoffs recorded"); a **"Pay Summary"** entry appears in Processing History listing the cutoffs and well count (covers REVIEW.md §Round 4 "Pay Summary → Processing History" — verifies the fixed audit finding "Compute Summary … never calls recordProcess"); an open Log View with a FLAG curve refreshes.
   **Result — T-BATCH-07:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-08 — Pay Summary negatives: PERM cutoff without PERM, bare well, per-well isolation

**Tool/panel:** Cutoffs & Pay Summary
**Preconditions:** one well WITHOUT a PERM curve in scope; one well with no VSH/PHIE/SWE computed.
**Steps:**

1. Set **PERM ≥** 10 and scope to a well that has VSH/PHIE/SWE but **no PERM** ▸ **Compute Summary**.
2. Set PERM back to blank; scope to only the well with **no computed curves** ▸ Compute Summary.
3. Scope a mix of good wells + the bare well ▸ Compute Summary.
   **Expected:** (1) PAY rows show Net = 0 / no PAY intervals — a sample with missing PERM must FAIL the cutoff, not silently pass (REVIEW.md, confirmed [x] item "with a PERM cutoff active, samples with missing PERM now FAIL the cutoff"); SAND/RESERVOIR rows are unaffected. (2) "No results — check that VSH/PHIE/SWE have been computed for the selected wells." — no crash, no misleading rows. (3) Good wells still return full rows; the bare well contributes nothing — one well's failure no longer zeroes the whole response (covers REVIEW.md §Round 4 "Per-well isolation").
   **Known issue — CONFIRMED 2026-07-31, step 1 will NOT behave as Expected says.** The confirmed
   REVIEW.md item is about a SAMPLE with missing PERM, and that part is true. But whether the cutoff
   runs at all is decided per WELL: `has_perm_cut = perm_min.is_some() && perm.iter().any(|v|
   !v.is_nan())`. A well carrying no permeability ANYWHERE makes that false and exempts itself, so
   expect **full pay, not Net = 0**. Measured on two wells of identical rock at PERM >= 1000: the
   well that measured 1 mD reported net 0, the well that measured nothing reported all of it. Log as
   known, not new. Steps 2 and 3 behave as written. See docs/review_triage.md finding 7 - whether an
   uncored well should be excluded or exempted is your call, and it changes reserves.

   **Automated coverage - pinned (pile B, 2026-07-31):** `a_well_with_no_perm_at_all_quietly_escapes_an_active_perm_cutoff` and `one_unusable_well_cannot_zero_the_whole_pay_summary` (workflow.rs). The first pins the exemption above AS-IS, not as correct behaviour - when it is fixed, that test fails, which is the alarm.

   **Result — T-BATCH-08:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-09 — Cutoff Sensitivity sweep: pick a VSH cutoff and reconcile with Pay Summary

**Tool/panel:** Cutoff Sensitivity pane (src/ui/cutoffDialog.ts; ribbon Petrophysics ▸ Reporting ▸ **Cutoff Sensitivity…**)
**Preconditions:** T-BATCH-02 outputs on ≥3 wells.
**Steps:**

1. Open **Cutoff Sensitivity…**; scope 3 wells; **Zone** "(whole well)", **DST / perf set** "(all samples)".
2. **Method** = Sweep; **Sweep** = VSH; **From → to** 0 → 1; **Steps** 60; **Metric** = Net; untick "Normalise each well to its own peak" ▸ **Compute**.
3. Click on the plot near the curve's elbow — a red cutoff line appears; readout shows `VSH = x.xxx → Net …` per well.
4. Click **Use pick as VSH cutoff** → the **VSH ≤** field updates to the pick.
5. Click **Save as pay-summary default**; then open **Cutoffs & Pay Summary**.
6. Cross-check: with identical VSH/PHIE/SWE, whole well, no zone/DST, compare the sweep's Net at the picked cutoff against the Pay Summary's Net for the same well.
   **Expected:** One sweep line per well (net pay monotonically non-decreasing as VSH cutoff loosens); pick/readout as described; the pay-summary pane opens **preloaded with the saved cutoffs**; sweep Net at the fixed cutoffs **matches the Pay Summary Net** for the same well (shared `classify_sample` math); History gains a "Cutoffs — Saved default cutoffs (…)" entry. Covers the unchecked click-throughs in REVIEW.md §"Cutoff Sensitivity pane (2026-07-20 #25)".
   **Result — T-BATCH-09:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-10 — Cutoff Sensitivity: NTG stays ≤ 1 with a mid-sample zone/DST boundary

**Tool/panel:** Cutoff Sensitivity (backend src-tauri/src/workflow.rs `run_cutoff_sweep`/`compute_sweep`)
**Preconditions:** a well whose zone tops and/or DST interval edges do NOT fall exactly on log sample depths (any real DST interval qualifies).
**Steps:**

1. Scope that well; pick the **Zone** and the **DST / perf set**; Method Sweep; Sweep VSH 0→1; **Metric = N:G** ▸ **Compute**.
2. Read the curve maximum; place the pick at the loosest cutoff (VSH = 1).
3. Cross-check the same well/zone/DST at fixed cutoffs against **Cutoffs & Pay Summary** Net and N/G.
   **Expected:** NTG never exceeds 1.0 anywhere on the sweep, including at fully-permissive cutoffs, and the sweep's numbers agree with the Pay Summary for the same slice — the boundary sample now contributes only its clamped overlap (covers REVIEW.md §Round 4 "Cutoff-sweep geometric clamp"; verifies the fixed audit finding "Cutoff-sweep NET/HPV/NTG isn't clamped to the zone/DST overlap — it re-introduces the exact 'step bleed past boundary' bug").
   **Automated coverage - pinned, with a residual (pile A):** NTG staying at or below 1 across a mid-sample zone base IS asserted. NOT asserted: step 3, the sweep-versus-Pay-Summary agreement.

   **Result — T-BATCH-10:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-11 — Cutoff Sensitivity crossplot + DST overlay + invalid-range negative

**Tool/panel:** Cutoff Sensitivity (cutoffDialog.ts)
**Preconditions:** well with a DST/PERF aux set and VOL_WETCLAY (or a Vclay curve) + PHIE.
**Steps:**

1. **Method** = **DST Crossplot**; preset **PHIE vs Vclay** (X curve `VOL_WETCLAY`, Y curve `PHIE`); pick the DST set in **DST / perf set** ▸ **Compute**.
2. Drag the red crosshair lines; readout shows "Crosshair: … Drag the red lines to adjust."
3. Click **Apply crosshair → cutoffs** → **PHIE ≥** and **VSH ≤** fields update. Try preset **PHIE vs Sw** ▸ Compute ▸ Apply → **SWE ≤** updates instead.
4. Negative: switch back to Sweep, set **From → to** = 1 → 0 ▸ Compute.
5. Theme check: Project tab ▸ **Theme** ▸ Dark — the crossplot repaints immediately in the new palette.
   **Expected:** Crossplot draws all samples dim with **DST-interval samples highlighted**; DST points should cluster at high PHIE / low Vclay (that is the defensible-pay argument); crosshair writes go to the correct fields. Step 4: readout "Sweep range invalid: 'to' must exceed 'from'." and no run. Step 5: instant repaint (this pane subscribes to themeVersion — note the contrast with T-BATCH-19).
   **Result — T-BATCH-11:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-12 — Field Dashboard: grid, sort, box plots, CSV export, read-only

**Tool/panel:** Field Dashboard (src/ui/dashboardPanel.ts; ribbon Petrophysics ▸ Batch ▸ **Field Dashboard…**)
**Preconditions:** T-BATCH-02 outputs across ≥5 wells; note the current PAYFLAG version count in the Curve Catalog.
**Steps:**

1. Open **Field Dashboard…**; set **VSH ≤** 0.5, **PHIE ≥** 0.1, **SWE ≤** 0.6, **PERM ≥** blank; **Flag** = PAY; **Metric** = HPV (m) ▸ **Compute**.
2. Time it roughly; then switch **Flag** to RESERVOIR and SAND (no recompute needed), and **Metric** through Avg PHIE / N/G / Net (m).
3. In "All PAY intervals (N)", click the **Net** column header, then again (▲/▼ toggles); click **Well** to sort alphabetically.
4. Click **Export CSV**; open the file.
5. Re-check the Curve Catalog's PAYFLAG version count.
   **Expected:** Compute finishes in **seconds, not minutes**, even across every well (covers REVIEW.md §"Field Dashboard is fast now"); status "N well(s) · M zone-rows across K flag level(s)…". Three sections render: **By zone — PAY** aggregation (Zone | Wells | Σ Net (m) | Σ HPV (m) | Mean N/G | Mean PHIE | Mean SWE), **HPV (m) distribution by zone** box plots (median line inside the box, whiskers, `median (n)` labels; per-zone medians should rank consistently with the By-zone table), and the sortable interval grid; empty aggregates render "—" with no crash (covers §"Field Dashboard no longer crashes on ~540 wells"). CSV headers match the grid columns; nulls are empty cells, not "null". PAYFLAG version count is **unchanged** — the dashboard is read-only, persisting flags stays with Cutoffs & Pay Summary. (The status line's trailing "FLAG curves written." is stale wording from before the read-only change — the Catalog check is the truth; note it in the ledger if observed.)
   **Result — T-BATCH-12:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-13 — Monte Carlo smoke: default chain, 2 uncertain parameters, percentile ordering

**Tool/panel:** Monte Carlo pane (src/ui/monteCarloDialog.ts; ribbon Petrophysics ▸ Batch ▸ **Monte Carlo…**)
**Preconditions:** ≥2 wells with GR/RHOB/NPHI/RT and zones.
**Steps:**

1. Open **Monte Carlo…**; **Chain** = "Default: VSH → Porosity → SW-Indo" (the note line shows the step titles).
2. Click **+ Add uncertain parameter** twice; in the two rows pick two different parameters from the dropdown (e.g. the vsh_gr shale point GR_SH and the sw_indo water resistivity — the dropdown lists every numeric parameter the chain exposes) with distribution **normal**; adjust mean/std dev to field-plausible values.
3. **Settings**: Iterations 200, Seed 42, HPV bins 12, Percentiles **P10 / P90**, VSH ≤ 0.5, PHIE ≥ 0.08, SWE ≤ 0.5, PERM ≥ blank. Scope 2+ wells ▸ **Run Monte Carlo**.
   **Expected:** Status "Running 200 realizations × M well(s)…" then "Done in X ms · N well-zone results"; a **Monte Carlo** job with per-well progress and Cancel appears in the **Processing** panel (MC now runs off-thread). Results: HPV histogram with dashed **P10/P50/P90** marker lines and a y-axis count; per well-zone table (Well | Zone | Gross | Net pay | NTG | Avg PHIE | Avg SWE | HPV) where every banded cell satisfies **lo ≤ P50 ≤ hi** (P10 ≤ P50 ≤ P90) and the P10–P90 band **brackets** the headline P50; clicking a table row switches the histogram to that zone. Domain: P50 Net/NTG/HPV should sit near the deterministic Pay Summary values at the same cutoffs, with spread widening as you widen the std devs. History gains "Monte Carlo — 200 realizations across M well(s) → N zone results".
   **Result — T-BATCH-13:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-14 — Monte Carlo seed reproducibility

**Tool/panel:** Monte Carlo pane
**Preconditions:** T-BATCH-13 configuration still in the pane.
**Steps:**

1. **Run Monte Carlo** with Seed 42; write down P10/P50/P90 Net pay and HPV for two well-zones (all displayed digits).
2. Run again, same Seed 42, nothing else changed.
3. Change Seed to 43 ▸ run again.
   **Expected:** Runs 1 and 2 produce **identical numbers to every displayed digit** for every cell (seeded per-realization RNG); run 3 differs slightly in the percentile bands but P50s stay close (200 realizations of the same distributions).
   **Automated coverage - pinned, with a residual (pile A):** seed 42 reproducibility and the zero-variance case ARE asserted. NOT asserted: the seed 43 step, where the run differs but P50 should stay close.

   **Result — T-BATCH-14:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-15 — Monte Carlo sensitivity: Spearman ranks + tornado sweep

**Tool/panel:** Monte Carlo pane (renderSensitivity/drawTornado)
**Preconditions:** T-BATCH-13 run configuration with 2 varied parameters.
**Steps:**

1. Confirm both **Sensitivity** checkboxes are ticked: "Rank sensitivity (Spearman)" and "Tornado sweep (P10 / P90)" ▸ **Run Monte Carlo**.
2. In **Parameter sensitivity**, switch **Zone** and **Metric** (HPV / Net pay / NTG / Avg PHIE / Avg SWE).
3. Switch **Percentiles** to P5 / P95 ▸ re-run.
   **Expected:** Tornado shows horizontal bars around a dashed "base" line, longest bar on top, split-coloured low/high sides, significant ρ annotations on the right; a parameter that cannot move the chosen metric is hidden (e.g. the Rw-type parameter must vanish for Avg PHIE — Sw does not feed porosity), with the caption explaining the gating; GR_SH should dominate VSH-driven metrics. After step 3 the histogram markers and table band read **P5/P95** and the tornado sweeps to the new percentiles. Covers REVIEW.md §"Monte Carlo parameter sensitivity + tornado (Wave B #13)".
   **Result — T-BATCH-15:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-16 — Monte Carlo PERM cutoff with chain-produced PERM

**Tool/panel:** Monte Carlo pane + montecarlo.rs
**Preconditions:** UAT_CHAIN4 saved (T-BATCH-04) — it ends in Permeability — Coates producing PERM.
**Steps:**

1. **Chain** = "Workflow: UAT_CHAIN4"; keep one uncertain parameter; Iterations 100, Seed 42.
2. Run once with **PERM ≥** blank; note P50 Net pay and HPV for a well-zone.
3. Run again with **PERM ≥** set high enough that it must bite (e.g. 50) — compare.
   **Expected:** The PERM-cutoff run should report **lower or equal** Net/HPV, mirroring the Pay Summary's behavior with the same cutoff.
   **Known issue:** AUDIT-2026-07-21 §Monte Carlo — "PERM cutoff is silently ignored whenever PERM is produced by the Monte Carlo chain itself (not read from the DB)": `has_perm_cut` only checks the DB-read input pool, and chain-produced PERM never enters it, so **expect both runs to return identical numbers**. Still unfixed (REVIEW.md holds it for sign-off as an interpretation-changing fix). Log as known, not new.
   **Update 2026-07-31 — the trigger is broader than the Known issue line says.** PERM reaches the
   cutoff check only if a step CONSUMES it and no step PRODUCES it. So the cutoff works on a chain
   that reads permeability from the project (e.g. one ending in Rock Typing), and goes silently dead
   the moment a permeability MODEL is inserted ahead of it - which is exactly the chain this test
   uses. Still expect both runs to return identical numbers. See docs/review_triage.md finding 8.

   **Automated coverage - pinned (pile B, 2026-07-31):** `adding_a_permeability_model_to_a_chain_switches_off_the_permeability_cutoff` (montecarlo.rs), which runs the working chain beside the broken one as its control. Pins the defect AS-IS.

   **Result — T-BATCH-16:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-17 — Cross-check: Monte Carlo vs Workflow chain with a bad-hole MASK

**Tool/panel:** Monte Carlo pane + Workflow Builder + Cutoffs & Pay Summary
**Preconditions:** a well with a bad-hole flag curve (e.g. from condflag/badhole) covering some pay; UAT_CHAIN4 saved with a **Mask** set on the phi_dn step (Workflow Builder ▸ step ⚙ ▸ Mask dropdown, or the Grid's Mask column).
**Steps:**

1. Run UAT_CHAIN4 through the **Workflow Builder** on that well; then **Compute Summary** at fixed cutoffs — note Net pay.
2. Run **Monte Carlo** on the same saved chain, same well, same cutoffs, **no** uncertain parameters, Iterations 10, Seed 42 — note P50 Net pay.
   **Expected:** The two should agree: the MC engine claims to run "the same chain", so masked (washout) samples must be excluded from pay in both.
   **Known issue:** AUDIT-2026-07-21 §Monte Carlo — "montecarlo.rs's own from-scratch chain executor misses two correctness behaviors the real chain runner enforces: MASK blanking and computed_only provenance resolution": MC ignores the step's MASK, so **expect MC Net/HPV ≥ the pay-summary value**, inflated by flagged intervals. Still unfixed (held for sign-off). Log as known.
   **Update 2026-07-31 — there are TWO causes, so a partial fix will not show up here.** Besides
   `run_realization` never blanking, the Monte Carlo planner never even FETCHES the flag curve: its
   external-input list is built from log inputs, and MASK is an option. The mask setting is carried
   all the way into the plan and then read by nobody. Still expect MC Net/HPV above the pay-summary
   value.

   **Automated coverage - pinned (pile B, 2026-07-31):** `the_monte_carlo_chain_ignores_a_step_mask_the_real_chain_honours` (montecarlo.rs) - it runs the real chain and the Monte Carlo chain over the same masked well and compares them, and asserts BOTH causes. Pins the defect AS-IS.

   **Result — T-BATCH-17:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-18 — Monte Carlo negatives: empty scope, dry well, cancel mid-run

**Tool/panel:** Monte Carlo pane + Processing panel
**Preconditions:** one well with no usable curves (the stub from T-BATCH-06); many wells available for the cancel case.
**Steps:**

1. Wells scope **Selection** with nothing selected ▸ **Run Monte Carlo** → "No wells in scope — pick a group, pin/select wells, or choose All"; no job starts.
2. Scope only the stub well ▸ run → "No results (no curve data or no zones matched)." — no crash; any dry metric cell renders **"—"**, never a fake hard 0 (covers REVIEW.md item "(10) Monte Carlo summarize() returns NaN (→ '—') for a dry/no-data metric").
3. Scope all wells, Iterations 5000 ▸ run ▸ in the **Processing** panel click the Monte Carlo job's **Cancel** mid-run.
   **Expected:** (3) The job stops at a well boundary (phase reflects the early stop); the pane renders whatever well-zone results completed, without hanging; the app stays responsive throughout.
   **Result — T-BATCH-18:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-BATCH-19 — Monte Carlo plots on pane resize and live theme swap

**Tool/panel:** Monte Carlo pane (monteCarloDialog.ts drawHistogram/drawTornado)
**Preconditions:** a completed MC run with histogram + tornado visible.
**Steps:**

1. Drag the pane wider/narrower.
2. Project tab ▸ **Theme** ▸ switch Default ↔ Dark (and to a brand theme if configured).
3. Click a different table row (forces a redraw) and compare.
   **Expected:** Step 1: histogram and tornado re-rasterize crisp at the new width — no blurry stretched bitmap (a ResizeObserver now handles this half). Step 2 (desired): both canvases repaint immediately in the new palette, exactly as the Cutoff Sensitivity plot did in T-BATCH-11.
   **Known issue:** AUDIT-2026-07-21 §Monte Carlo — "Monte Carlo's HPV histogram canvas never repaints on a live theme swap or panel resize, unlike every sibling Canvas-2D dock pane": the resize half has since been fixed, but there is still no themeVersion subscription — **expect the theme-swap repaint to fail** (stale colors until step 3's click). REVIEW.md lists "MC histogram theme-repaint" as deferred polish. Log as known.
   **Result — T-BATCH-19:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

# Section MLEQ — ML, equations & curve management

Everything verified against source. Composing the test plan now — this is my final output.

## MLEQ — ML, user equations & curve management

**Shared preconditions:** project open with at least 3 wells imported carrying GR, RHOB, NPHI and DT (DT absent or blanked in one well for T-MLEQ-10); PHIT computed on at least 2 wells (any porosity module or SSC/SSPW run); Python 3.10+ with numpy + scikit-learn on PATH (or `SANDIBUMI_PYTHON` set). Ribbon tabs referenced: **Data**, **Petrophysics**, **Advance**, **Project**. The Processing History pane opens from **Project ▸ Monitor ▸ History** or right-click workspace background → **Processing History**.

### T-MLEQ-01 — ML pane opens with the full form (smoke)

**Tool/panel:** Machine Learning pane (src/ui/mlDialog.ts, opened via Advance ribbon)
**Preconditions:** project open, wells imported.
**Steps:**

1. Ribbon → **Advance** tab → **ML Models…** (Machine Learning group).
2. Inspect the pane top to bottom without running anything.
   **Expected:** a dock pane titled **Machine Learning** opens (non-blocking). Controls present: **Task** = "Predict a continuous log (regression)", **Algorithm** = "Random Forest Regressor" with a one-line description under it; **Input curves** checkbox list (GR, NPHI, RHOB, RES_DEEP, DT pre-checked where they exist); **Target curve**; **Train wells** checklist; the shared **Wells** scope row (Group / ★ Pinned / Selection / All / Custom… with a live count); **Parameters** (trees = 200, max depth = 0); **Output curve** = ML_PRED; **Common** = "Standardize inputs (z-score)" checked, Seed = 42; **Run Model** button; **Compare** row with subset dropdown ("Full set only") and **Compare algorithms** button; hint line "Needs Python with numpy + scikit-learn". Switching Task swaps the algorithm list and the Output curve default (ML_CLASS / FACIES_ML / PC).
   **Result — T-MLEQ-01:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-02 — Inspector opens; Python engine status is shown

**Tool/panel:** Inspector — Equation Editor tab (src/ui/inspectorPanel.ts)
**Preconditions:** project open.
**Steps:**

1. Ribbon → **Data** tab → **Curve Catalog** button (Manage group).
2. In the Inspector pane confirm two tabs: **Equation Editor** (active) and **Curve Catalog**.
3. In Equation Editor set **Language** = "Python (numpy)". Read the note line at the top.
   **Expected:** the note reads "Python (numpy): input curves are float32 arrays (NaN = missing) plus `depth`…" and, after a moment, appends **"(engine: \<path to python\>)"** — the live worker path. If it instead appends "⚠ No Python with numpy found — install Python 3.10+ & numpy, or set SANDIBUMI_PYTHON", stop: Python-dependent tests (03, 05, 10–15) are **Blocked** until the environment is fixed.
   **Result — T-MLEQ-02:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-03 — Python equation PHIE_TEST = PHIT × 0.9 on 2 wells

**Tool/panel:** Inspector — Equation Editor (src/ui/inspectorPanel.ts → src-tauri/src/python_engine.rs)
**Preconditions:** PHIT exists on wells A and B; T-MLEQ-02 passed.
**Steps:**

1. Equation Editor → **Equation** picker = "— New equation —". **Name** = `PHIE_TEST`, **Input curves** = `PHIT`, **Output curve** = `PHIE_TEST`, **Units** = `v/v`, **Language** = Python (numpy).
2. Script: `phie_test = phit * 0.9` (variables are the lowercased mnemonics).
3. Click **Save** → expect status `Saved "PHIE_TEST".`
4. Select well A in the Wells tree, leave **Apply to all wells** unchecked, click **Run**.
5. Select well B, click **Run** again.
   **Expected:** each run reports "1/1 well(s) succeeded, N rows written." Curve Catalog tab (well A or B selected): row **PHIE_TEST** with **Set = EQUATION v1**; the Constellations section gains an **EQUATION v1** entry. Processing History shows two **Equation** entries `Ran "PHIE_TEST" on 1 well(s)`. Domain check: PHIE_TEST = exactly 0.9 × PHIT everywhere (spot-check in a log view or DB Inspector), range 0 – ~0.35 v/v, NaN where PHIT is NaN.
   **Result — T-MLEQ-03:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-04 — Rhai equation (legacy per-sample engine)

**Tool/panel:** Inspector — Equation Editor (src-tauri/src/equations.rs Rhai path)
**Preconditions:** GR present on the selected well.
**Steps:**

1. New equation: **Name** = `VSHR_TEST`, **Input curves** = `GR`, **Output curve** = `VSHR_TEST`, **Language** = "Rhai (legacy)".
2. Script (a single expression, lowercased variable): `gr / 150.0`
3. **Save**, then **Run** on one selected well.
   **Expected:** "1/1 well(s) succeeded, N rows written." VSHR_TEST appears in the Curve Catalog under set EQUATION. Domain check: VSHR_TEST ≈ GR/150 — high (→ ~0.7–1) in shale intervals, low (< 0.3) in clean sand, NaN wherever GR is NaN (per the Rhai note, any NaN input yields NaN).
   **Result — T-MLEQ-04:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-05 — Equation negatives: unsaved run, syntax error, unresolvable input

**Tool/panel:** Inspector — Equation Editor
**Preconditions:** T-MLEQ-03 done.
**Steps:**

1. Pick "— New equation —", type any script, click **Run** WITHOUT saving.
2. Load PHIE_TEST, break the script (e.g. `phie_test = phit * ` ), **Save**, **Run** on one well.
3. Fix the script but set **Input curves** = `PHIT_NOPE` (a curve that doesn't exist), **Save**, **Run**.
   **Expected:** (1) status "Save the equation before running it." — no run. (2) a readable per-well error naming the Python failure (status turns error-styled, "Errors: \<well\>: …") — the app does not crash and the worker stays alive (a following valid run still works). (3) an **error**, not a green success: the run must not report rows written for an input that resolves to nothing.
   **Known issue:** audit finding "An equation with an unresolvable input or output curve name 'succeeds' silently as all-NaN, indistinguishable from a legitimate result" (Equations engine §3). A Round-4 uncommitted fix ("All-NaN module runs report honestly… Same guard on Rhai + Python equations") claims step 3 now errors — if you still get a clean success with an all-NaN output, mark Fail and log it as this known finding, not a new one.
   **Result — T-MLEQ-05:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-06 — Equation re-run: open plots refresh, version v2 kept

**Tool/panel:** Inspector — Equation Editor + any open plot (dataVersion cross-check)
**Preconditions:** T-MLEQ-03 done; a log view showing PHIE_TEST (or a Histogram of PHIE_TEST) left open.
**Steps:**

1. With the plot open, edit PHIE_TEST's script to `phie_test = phit * 0.8`, **Save**, **Run** on the same well.
2. Watch the open plot without touching it.
3. Open Curve Catalog tab → Constellations section.
   **Expected:** the open log view/histogram re-reads and redraws the new (lower) PHIE_TEST in place — no reopen needed. Constellations shows **EQUATION v1 AND v2** (old values kept, v2 current). Covers REVIEW.md §Round 4 "dataVersion refresh after equation / ML / report runs" and §P1-c "Never overwrite".
   **Known issue:** audit finding "Equation runs never bump dataVersion — every other open panel goes stale after Run" (Equations engine §1). The fix is present in the current working tree (inspectorPanel.ts:322) but unverified — if the plot stays stale until something else refreshes it, mark Fail and log as this known finding.
   **Result — T-MLEQ-06:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-07 — Electrofacies (K-means), K=4, block track + theme repaint

**Tool/panel:** module pane "Electrofacies (K-means)" (src/ui/moduleDialog.ts + src-tauri/src/facies.rs)
**Preconditions:** wells with GR + RHOB + NPHI (+DT).
**Steps:**

1. Ribbon → **Petrophysics** tab → **Facies ▾** dropdown → **Electrofacies (K-means)**.
2. Wells scope = All (or your group). **CURVE1** = GR, **CURVE2** = RHOB, **CURVE3** = NPHI, **CURVE4** = DT (or "(none)" if absent), **CURVE5** = "(none)". **K** = 4, **SEED** = 7, **OPT_STANDARDIZE** = ZSCORE, **Mask (optional)** = "(none)", **Output cons** = INTERP.
3. Click **Run**. The Processing panel auto-opens — watch the per-well ✓ list.
4. Open a log view on a run well and pick the built-in **Facies** layout (or set FACIES's display to "Facies blocks").
5. Ribbon → **Project** tab → **Theme** dropdown → switch theme, then switch back.
   **Expected:** result line "All N well(s) computed. Per-well details are in the Processing panel." FACIES is integer 0–3 only. Domain check: **FACIES 0 is the cleanest class** (lowest mean GR — clean sand), 3 the shaliest; blocks follow the GR character bed-by-bed. History gains a **Module** entry "Ran Electrofacies (K-means) on N wells". On theme switch the facies block colors and pane chrome repaint immediately (covers REVIEW.md §"FACIES block track" and §"Theme switch repaints everything immediately"; folds in §"Electrofacies — k-means").
   **Result — T-MLEQ-07:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-08 — Electrofacies (GMM, soft): FACIES_GMM + FPROB

**Tool/panel:** module pane "Electrofacies (GMM, soft)" (src/ui/moduleDialog.ts + facies.rs)
**Preconditions:** T-MLEQ-07 run on the same wells (same curves, K=4).
**Steps:**

1. Petrophysics → **Facies ▾** → **Electrofacies (GMM, soft)**. Same inputs as T-MLEQ-07, **K** = 4, **SEED** = 7.
2. **Run**, then display FACIES_GMM as facies blocks next to FACIES, and FPROB as a 0–1 curve.
3. Color a crossplot (e.g. RHOB vs NPHI) by FACIES_GMM.
   **Expected:** outputs **FACIES_GMM** (0–3) and **FPROB** (0–1) both land in the catalog. Domain check: FACIES_GMM broadly agrees with the k-means FACIES in thick homogeneous beds; **FPROB ≈ 1 mid-bed and dips toward ~1/K (0.25) at bed boundaries/transitional silty intervals** — transitional beds visible instead of forced. Crossplot shows coherent, GR-ordered clusters with the categorical F0..F3 legend. Covers REVIEW.md §"GMM soft electrofacies" (both unchecked items).
   **Result — T-MLEQ-08:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-09 — Facies negative: well with no usable input curves

**Tool/panel:** module pane "Electrofacies (K-means)" + Processing panel (facies.rs guards)
**Preconditions:** one well in the project lacking all five input curves (e.g. tops-only or resistivity-only well); alternatively pick a tiny well and set K larger than its complete-sample count.
**Steps:**

1. Open **Electrofacies (K-means)**; Wells scope = **Custom…** → tick only the curve-less well.
2. **Run**, then open the Processing panel details for the run.
   **Expected:** the well is reported as an **error / ⚠ warned** ("no input curve present" class of failure) — NOT a green "✓ N samples" success; no plausible-looking all-NaN FACIES version should be silently added for that well.
   **Known issue:** audit finding "facies.rs's 'can't cluster this well' cases (no input curve present, or fewer complete samples than K) are silently reported as a full successful run with a plausible row count, not a warning or error" (Facies §2). Round 4's uncommitted "All-NaN module runs report honestly" fix claims this now warns — if you still see ✓ success with an all-NaN FACIES, mark Fail and log as this known finding.
   **Automated coverage - pinned, with a residual (pile A):** the honest error on a well with no usable curve IS asserted, with a live control. NOT asserted: that no FACIES version row was written.

   **Result — T-MLEQ-09:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-10 — ML regression: predict DT from GR/RHOB/NPHI (+ leaderboard)

**Tool/panel:** Machine Learning pane (mlDialog.ts + src-tauri/src/ml.rs)
**Preconditions:** ≥2 wells with complete GR/RHOB/NPHI/DT (train); one well with DT absent or blanked (apply target of interest).
**Steps:**

1. Advance → **ML Models…**. **Task** = "Predict a continuous log (regression)", **Algorithm** = Random Forest Regressor.
2. **Input curves**: check GR, RHOB, NPHI only (uncheck DT, RES_DEEP). **Target curve** = DT.
3. **Train wells**: tick the ≥2 complete wells. **Wells** scope: Custom… → all train wells + the DT-less well. **Output curve** = `DT_SYN`.
4. Click **Run Model**.
5. Then click **Compare algorithms** (subset "Full set only").
   **Expected:** status "Done in N ms → DT_SYN"; app status line "Random Forest Regressor: wrote DT_SYN to N well(s)". Metrics table shows **r2_train** (expect ≥ ~0.8 for these correlated logs), rmse_train, n_train; per-well table lists predicted sample counts including the DT-less well. Domain check: on a well WITH real DT, overlay DT_SYN vs DT in a log view — same track scale 40–140 µs/ft, they track within ~±10 µs/ft; on the DT-less well DT_SYN is petrophysically plausible (higher in shale/high-NPHI, lower in tight/high-RHOB streaks). Curve Catalog: DT_SYN in **set ML v1**; History gains an **ML** entry. The Compare click renders the blind-well CV **leaderboard** (best R² first, ± std) plus permutation-importance bars, and writes no curves (covers REVIEW.md §Round 3 item (3) "ML comparison leaderboard").
   **Result — T-MLEQ-10:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-11 — ML classification: ML_CLASS + ML_CLASS_PROB

**Tool/panel:** Machine Learning pane
**Preconditions:** FACIES exists on ≥2 wells (T-MLEQ-07).
**Steps:**

1. **Task** = "Predict a discrete log (classification)", **Algorithm** = Random Forest Classifier.
2. **Input curves** = GR, RHOB, NPHI (uncheck the rest). **Target curve** = FACIES (auto-selected when present).
3. **Train wells** = the wells carrying FACIES; **Wells** scope = All. **Output curve** = `ML_CLASS`. **Run Model**.
   **Expected:** status "Done in N ms → ML_CLASS, ML_CLASS_PROB" — TWO curves written (the \_PROB suffix is automatic). Metrics: accuracy_train (expect high, ≥0.9 — it learned these very wells), class_counts per facies id, n_train. Domain check: on a train well ML_CLASS ≈ FACIES; on other wells the predicted classes follow log character; **ML_CLASS_PROB ∈ [0.25, 1] dips where the log character is ambiguous** (silty transitions), ≈1 in unambiguous beds. Covers REVIEW.md §"ML suite" supervised item. Cross-check: both curves in Curve Catalog under set ML.
   **Result — T-MLEQ-11:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-12 — ML clustering: K-Means k=4, silhouette, plots refresh

**Tool/panel:** Machine Learning pane (field-wide pooled clustering)
**Preconditions:** ≥2 wells with GR/RHOB/NPHI; a log view left open on one of them.
**Steps:**

1. **Task** = "Electrofacies clustering (unsupervised)", **Algorithm** = K-Means.
2. **Input curves**: GR checked FIRST, plus RHOB, NPHI (the form hint says order matters — class 0 = lowest mean of the first checked curve). **K classes** = 4, Seed 42. **Wells** scope = All. **Output curve** = `FACIES_ML`. **Run Model**.
3. Watch the already-open log view; then add FACIES_ML as a facies-blocks track in two wells side by side.
   **Expected:** metrics table shows **cluster_sizes** (4 non-empty classes) and a **silhouette** row (expect ~0.2–0.6; < 0.1 means the 4 clusters barely separate — note it). Domain check: **FACIES_ML 0 = lowest-GR (cleanest) class**, monotone to 3 = shaliest; because clustering pools all wells, class ids are consistent across the two side-by-side wells. The open log view refreshes to show the new curve availability without reopening. Covers REVIEW.md §"ML suite" field-wide electrofacies.
   **Known issue:** audit finding "mlDialog.ts never bumps dataVersion after a successful run, unlike every sibling curve-writing dialog" (ML bridge §2). The bump now exists in the working tree (mlDialog.ts:482, REVIEW.md Round 4 unchecked) — if open plots/catalog do NOT refresh after the run, mark Fail and log as this known finding.
   **Result — T-MLEQ-12:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-13 — ML dimensionality reduction: PCA

**Tool/panel:** Machine Learning pane
**Preconditions:** ≥3 numeric input curves on the scope wells.
**Steps:**

1. **Task** = "Dimensionality reduction (PCA / t-SNE)", **Algorithm** = Principal Component Analysis, **components** = 3. Input curves GR, RHOB, NPHI, DT. **Output curve** stays `PC`. **Run Model**.
2. Crossplot PC1 vs PC2, colored by FACIES_ML.
   **Expected:** status "Done in N ms → PC1, PC2, PC3" — numbered components, all three in the Curve Catalog (set ML). Metrics show **explained_variance_pct** as a descending list; PC1 should dominate (typically > 50% for correlated porosity-lithology logs, sum of 3 near 90%+). Domain check: the PC1-PC2 crossplot separates the electrofacies classes into coherent point clouds (PCA and clustering see the same structure).
   **Result — T-MLEQ-13:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-14 — ML negatives + missing bad-hole Mask

**Tool/panel:** Machine Learning pane
**Preconditions:** a BADHOLE flag exists on at least one scope well (Bad-Hole QC module run) — for step 3's assessment only.
**Steps:**

1. Uncheck every Input curve → **Run Model**.
2. Task = regression, tick only 1 **Train well** → **Compare algorithms**.
3. Search the whole ML pane for any "Mask (optional)" control (compare: every module pane, e.g. Electrofacies, has one above "Input cons").
   **Expected:** (1) status "Check at least one input curve", nothing runs. (2) "Blind-well comparison needs at least 2 training wells". (3) desired behavior would be a mask picker so washout/casing samples can be excluded from training/pooling — it does not exist, so flagged bad-hole samples silently bias the scaler, cluster centers, trained models and PCs for every well in the run.
   **Known issue — MOSTLY RESOLVED, corrected 2026-07-31:** this used to say "run_ml has no bad-hole/flag MASK support at all" and told you to expect step 3 to Fail. **The backend does have it**, pinned by `run_ml_mask_excludes_apply_samples` and `run_ml_mask_excludes_training_outlier` (`ml.rs`), both on the gate — flagged samples are excluded from the apply set *and* from training. What is still missing is only the **Mask picker in `mlDialog.ts`**, so you cannot choose one from the dialog. Step 3 will therefore look like it fails from the UI, but the reason is a missing control, not a missing capability — log it against the dialog, and do not treat ML results over bad-hole intervals as untrustworthy on this basis.
   **Result — T-MLEQ-14:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-15 — ML pane list staleness while open

**Tool/panel:** Machine Learning pane (dataVersion subscription gap)
**Preconditions:** ML pane open; any module ready to run (e.g. VSH from GR).
**Steps:**

1. Leave the ML pane open. In another pane run a module that writes a NEW curve name (e.g. Petrophysics → Shale volume → vsh_gr → VSH), or import a new well.
2. Return to the still-open ML pane: look for the new curve in **Input curves**/**Target curve** and the new well in **Train wells**.
3. Close and reopen the ML pane; look again.
   **Expected:** desired: the lists refresh in place (the module panes do exactly this). Actual pass criterion for the reopen step: after reopening, the new curve/well IS listed.
   **Known issue:** audit finding "mlDialog.ts never subscribes to dataVersion, so its own wells/curve-catalog lists go stale while the pane stays open" (ML bridge §3) — explicitly deferred in REVIEW.md Round 4 ("Low-value polish left: … ml/wellScope dataVersion subscribe"). Expect step 2 to Fail (stale until reopen); log as known.
   **Result — T-MLEQ-15:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-16 — Curve Catalog: provenance, statistics, search/sort

**Tool/panel:** Inspector — Curve Catalog tab (inspectorPanel.ts)
**Preconditions:** tests 03–13 have produced EQUATION, ML and INTERP sets; a well from those runs selected in the Wells tree.
**Steps:**

1. Inspector → **Curve Catalog** tab.
2. Verify one row per curve produced so far (PHIE_TEST, VSHR_TEST, FACIES, FACIES_GMM, FPROB, DT_SYN, ML_CLASS, ML_CLASS_PROB, FACIES_ML, PC1–PC3) with columns Mnemonic / Unit / Family / **Set** (with vN) / **Module / Source** / When / n / Min / Max / Mean.
3. In the **Constellations** section hover a set row; then type `electrofacies` (then `EQUATION`) in the search box; click the **Mean** header twice.
   **Expected:** every computed curve shows its set + version (EQUATION / ML / INTERP …), producing module, timestamp and n/min/max/mean stats. The hover tooltip reveals the exact **params / inputs / curves** of that run (e.g. K=4, SEED=7, CURVE1=GR — answer to "where did this FACIES come from?" in one glance). Search filters live across mnemonic/set/module/unit/date; header clicks sort then reverse. Covers REVIEW.md §P1-c "Per-curve provenance" and "Catalog search/filter/sort".
   **Automated coverage - end-to-end (pile C, 2026-08-01):** `catalog.e2e.mjs` covers the rows, the
   live search and the header sorting. The filter is checked for NARROWING to matching rows rather
   than merely shrinking the table, and for being reversible - a filter that cannot be cleared
   leaves the panel stuck on a subset, which reads exactly like a well that lost its curves.
   Sorting is asserted on the SECOND click reversing the first exactly, since one click can
   coincide with the order already there and a header that only draws an arrow looks sorted
   without being sorted. **Not covered:** the hover provenance tooltip, and the statistics columns.
   Worth knowing: with no active well this panel renders a plausible static placeholder (GR,
   RES_DEEP, NPHI, RHOB, DT, SP) with no search box - if you are reading a catalog that looks
   oddly generic, no well is selected.

   **Result — T-MLEQ-16:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-17 — Restore an old constellation version

**Tool/panel:** Inspector — Curve Catalog, Constellations section
**Preconditions:** T-MLEQ-07 done; a log view open showing the FACIES block track.
**Steps:**

1. Re-run **Electrofacies (K-means)** with **K = 6** (same wells, Output cons INTERP) → creates v2.
2. Curve Catalog → Constellations: confirm **INTERP v1 AND v2** rows, v2 tagged "current".
3. Click **Restore** on v1. Watch the open log view.
4. Click **Restore** on v2 to return.
   **Expected:** status "Version restored (N samples back in the current curves)"; History gains a **Constellation** entry. The open log view's FACIES track flips back to the 4-class v1 blocks WITHOUT reopening (dataVersion bump), then to 6-class on step 4. The old run's values were never destroyed by the re-run. Covers REVIEW.md §P1-c "Never overwrite" and "Restore a version".
   **Result — T-MLEQ-17:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-MLEQ-18 — Delete a version: history pruned, current values untouched

**Tool/panel:** Inspector — Curve Catalog, Constellations section
**Preconditions:** T-MLEQ-06 left EQUATION v1 and v2 (v2 current); a plot showing PHIE_TEST open.
**Steps:**

1. In Constellations click **Delete** on **EQUATION v1** — the button must change to **"Confirm delete"**.
2. Wait ~3 s WITHOUT clicking → it reverts to "Delete" (accidental-click guard).
3. Click **Delete** then **Confirm delete**.
   **Expected:** status "Constellation version deleted (current curve values kept)". Only the v1 history row disappears; the open plot's PHIE_TEST values are byte-identical (still the v2 result, = 0.8×PHIT) — current curves are never touched by a prune. History gains a **Constellation** "Deleted a constellation version" entry. Covers REVIEW.md §P1-c "Prune old versions" (incl. the two-click confirm).
   **Result — T-MLEQ-18:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

# Section PLOT — Plots, viewers & curve editing

# UAT Cluster: PLOT — Plots & Viewers

Shared preconditions for the whole cluster: app running via `npm run tauri dev`; a project with **≥3 wells** imported (GR, RHOB, NPHI, RES_DEEP, DT); on at least one well the standard chain has been run so **VSH, PHIE, PHIT, SWE** exist; that well also has **tops**, **core data** (CPOR/CPERM), and a **FACIES/electrofacies curve**; at least one saved zone. Tests 01–09 use the Log View; 10–15 the parameter/multi-well plots; 16–20 are cross-cutting. Keep the **History** panel (**Project ▸ Monitor ▸ History**) reachable throughout.

### T-PLOT-01 — Open a Log View (smoke)

**Tool/panel:** Log View (WebGPU) — `src/ui/logViewPanel.ts`, ribbon `index.html`
**Preconditions:** Project open; a well selected in Wells & Tops.
**Steps:**

1. Ribbon **Plot** tab → click **New Log View**.
2. Confirm the pane tab title reads "_wellname_ — _layout name_".
3. Select a different well in Wells & Tops.
4. In a pane's ＋ add-panel menu, confirm **New Log View** is also listed there.
   **Expected:** Curves draw on the WebGPU canvas; status line shows "Loaded well _X_"; track headers show each curve with its color and min/max scale; report strip shows Well / Field / Depth Coverage. Switching wells reloads the same layout for the new well and resets the scroll to the top of the well. If WebGPU is unavailable the panel must say "WebGPU unavailable — viewer disabled", not hang.
   **Result — T-PLOT-01:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-02 — Depth scale, zoom, pan (true 1:N)

**Tool/panel:** Log View mini toolbar — `src/ui/logViewPanel.ts` (covers REVIEW.md §Held-item resolutions "Depth-scale dropdown now shows the TRUE scale" and §Low-tier sweep "Log-view smoothness")
**Preconditions:** T-PLOT-01 passed.
**Steps:**

1. In the log view's own toolbar, open the **1:N** selector — confirm it opens at **1:2000**.
2. Pick **1:200**, then **1:500** — visibly different zooms each time.
3. Click **−** / **＋** zoom buttons; then Ctrl+wheel over the canvas; drag-pan up/down through the well.
4. Watch the 1:N box after each zoom: between presets it must show a transient "**1:N ⟳**" with the live ratio, never stick on the last preset.
5. Click **⟳** (Reset view).
   **Expected:** 1:200 and 1:500 are honestly different scales (1:200 ≈ 5 mm of screen per metre); pan is smooth on a busy 15-curve layout with no stutter; depth axis ticks update continuously; ⟳ returns to top-of-well at the default 1:2000 and the selector re-reads "1:2000".
   **Result — T-PLOT-02:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-03 — Cursor readout with units + track scoping

**Tool/panel:** Log View readout — `src/ui/viewerChrome.ts` `renderReadout` (covers REVIEW.md §Polish — UX "Cursor readout: real units + no more mangled values")
**Preconditions:** Layout showing a resistivity track and a porosity or permeability track.
**Steps:**

1. Hover the canvas over the resistivity track — readout shows "Depth: _N_" plus only that track's curves.
2. Hover a permeability/porosity track.
3. Single-click (no drag) a track — status says "Track "_X_" selected — readout follows it"; move the cursor over other tracks.
4. Click the same track again to release.
   **Expected:** Values keep resolution and carry catalog units: RT reads like "2151 ohm.m" (not "2151.00"), PHIE like "0.18 v/v", a low perm keeps "0.003" (never "0.00"). While a track is selected the readout sticks to its curves regardless of cursor position; releasing returns to follow-the-cursor. Hovered track's header tints.
   **Result — T-PLOT-03:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-04 — Track resize, reorder, curve drag between tracks, scale edit

**Tool/panel:** Track headers — `src/ui/viewerChrome.ts` `renderTrackHeaders`
**Preconditions:** Log view open with ≥3 tracks.
**Steps:**

1. Drag a track header's right-edge **resizer** — the track widens live.
2. Drag a track header **title** onto another header — tracks reorder.
3. Drag a **curve row** (e.g. GR) from one header onto another track's header — the curve MOVES; press **Project ▸ Edit ▸ Undo** (Ctrl+Z) — it moves back; redo (Ctrl+Y).
4. Repeat the drag holding **Ctrl** — the curve is COPIED (stays in both tracks).
5. Click a scale number (min or max) under a curve — an inline edit box appears; type a new value, Enter.
6. Click a curve's color swatch or name — the curve toggles hidden/visible.
7. Toolbar **▤** — headers cycle full → compact → titles-only; **▦** opens **Track borders** (set Dashed, width 2, **Apply**).
   **Expected:** Every change repaints the canvas immediately; curve drawn against the new scale after the scale edit (e.g. GR 0–150 visibly stretches vs 0–200); curve-move is bit-exact undoable from **Project ▸ Edit ▸ Undo** (status "Undo: move GR → …"); a curve dropped on a track already showing it is refused silently; pane tab shows the unsaved dot (●) after edits; borders redraw dashed between tracks.
   **Result — T-PLOT-04:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-05 — Layout Properties dialog + Save Layout

**Tool/panel:** Layout Properties — `src/ui/layoutPropsDialog.ts`; ribbon **Plot ▸ Properties…** / **Save Layout…** (covers REVIEW.md §Low-tier sweep "Duplicate track titles prevented")
**Preconditions:** Log view active.
**Steps:**

1. Ribbon **Plot ▸ Properties…** (or toolbar ⚙). Confirm the dialog title "Layout Properties — _name_".
2. Track list header: **＋** (insert), **⧉** (duplicate), **↑/↓** (reorder), **✕** (delete) — do each once.
3. Try **✕** with only one track left — must be refused.
4. Rename a track to another track's exact title — it must auto-suffix ("_name_ 2"); re-typing a track's own name must NOT suffix.
5. In the curve table change a curve's **Color**, **Min/Max**, **Fill** (To left edge) + **Fill color** + **Opacity**; **＋ Add curve** (pick from the datalist); **✕** remove one.
6. Click **Apply** (dialog stays), then **OK**. Then Ctrl+Z.
7. Ribbon **Plot ▸ Save Layout…**, give a name; then switch the ribbon **Layout** dropdown between the saved layout and the original.
   **Expected:** Apply/OK repaint the view with new colors/fills/scales; Cancel discards; Ctrl+Z restores the pre-dialog layout in one step (status "Undo: layout properties (…)"); the saved layout appears in the **Layout** dropdown and switching it rebuilds the active view; on reopen the saved layout persists (stored in the project DB).
   **Result — T-PLOT-05:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-06 — FACIES block track

**Tool/panel:** Layout Properties fill = "Facies blocks" — `src/ui/layoutPropsDialog.ts` + `logViewPanel.ts`
**Preconditions:** Well with a FACIES/cluster curve (integer classes).
**Steps:**

1. Layout Properties → insert a track, add the FACIES curve, set **Fill** = **Facies blocks**, OK.
2. Inspect the track header and the drawn track; zoom in/out.
   **Expected:** The track fills with solid colored blocks, one color per class, spanning full track width; header shows a striped multi-color swatch and the scale line reads "**class blocks**" (no editable min/max); block boundaries land exactly at the depths where FACIES changes value (cross-check against the readout); blocks track pan/zoom without smearing. Right-click on this track must NOT offer "Edit FACIES…" (block curves are not editable).
   **Result — T-PLOT-06:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-07 — Tops overlay, tops editing, interval windowing everywhere

**Tool/panel:** Tops editor + zone windowing — `src/ui/topsEditor.ts`, `plotCommon.ts` `buildZoneSelect` (covers REVIEW.md §Highlight tool… "Highlight tool — colored depth bands in the Log View")
**Preconditions:** Well with tops; a Histogram and a Crossplot also open on the same well.
**Steps:**

1. In the log view, confirm top lines + labels draw at the correct depths and track pan/zoom.
2. Toolbar **🏷** — status "Tops editing ON". Click to add a top; drag one to move; double-click to rename, then delete it. Ctrl+Z after each.
3. Toolbar **🖍** — drag to paint a highlight band; double-click it → recolor/label; confirm 🏷 and 🖍 are mutually exclusive (turning one on turns the other off).
4. In the **Wells & Tops** pane click a top: the log view scrolls to it, AND in the open Histogram/Crossplot the **Zone** dropdown auto-selects "Top _X_ (_min–max_)" and the plot reloads windowed to that interval.
   **Expected:** Tops add/move/rename/delete all undoable and recorded in History ("[Tops]"-kind entries); the windowed histogram/crossplot show only samples in the top interval (n drops accordingly); highlight bands persist across a well switch and back, sit below tops lines.
   **Result — T-PLOT-07:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-08 — Core-point overlays in the log view

**Tool/panel:** Core overlay — `src/ui/logViewPanel.ts` `drawCoreOverlay`
**Preconditions:** Well with core data imported (CPOR, CPERM); layout with PHIE and PERM tracks.
**Steps:**

1. Load that well in the log view; scroll to the cored interval.
2. Zoom and pan across the core points.
3. Edit the PHIE track scale (e.g. max 0.25) via the header.
   **Expected:** Diamond markers appear over the PHIE track at plug depths (CPOR) and over the PERM track (CPERM), in the host curve's color, positioned on that curve's own scale — on a log-scaled PERM track the diamonds must sit log-correctly. Petro check: CPOR diamonds should track the PHIE curve within a few p.u. in good hole. Values outside the track scale are simply not drawn (never clamped to the track edge). Diamonds re-register perfectly through pan/zoom and after the scale edit.
   **Result — T-PLOT-08:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-09 — Pin / follow-well behavior (side-by-side wells)

**Tool/panel:** Well pin — `src/ui/logViewPanel.ts` (pin toggle in the workspace/toolbar), Wells & Tops
**Preconditions:** ≥2 log views open, ≥3 wells.
**Steps:**

1. With pin **ON** (default): select different wells in Wells & Tops — **every** open log view follows.
2. Turn the well pin **OFF** (📌). Make view A active, select well 2 — only view A reloads; view B keeps its well.
3. Make view B active, select well 3 — only B reloads.
4. Turn pin back ON, switch well — all views follow again.
   **Expected:** Pin OFF gives true side-by-side multi-well viewing; each pane's tab title shows its own well; no cross-contamination of series when rapidly switching wells (titles and curves always match — fast switches never paint a stale well's curves).
   **Result — T-PLOT-09:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-10 — Histogram: bins, overlays, percentiles, picks → zone parameters, templates

**Tool/panel:** Histogram — `src/ui/histogramPanel.ts` (covers REVIEW.md §Low-tier sweep "Histogram: constant curves render; the n never silently disagrees" and §Low-tier "Stats / regression reject ±Infinity")
**Preconditions:** Well selected; GR present; a FLAG/constant curve available for step 8.
**Steps:**

1. Ribbon **Plot ▸ Histogram**. Curve = GR, Zone = All depth.
2. Click stat chips: **P5 P50 P95 n** show values; **Mean**/**P50**/**P5**/**P95** chips draw marker lines.
3. **⚙ Properties**: Bins = 120; tick **Normalize (%)**, **Cumulative % overlay**, **Box plot (P5–P25–P50–P75–P95)**; Percentiles = "3, 97"; Statistics = **Both**; tick **Show parameter pickers (Pick A/B → zone parameter)**; **Apply**.
4. Click on the clean-sand mode → Pick A fills (param defaults **GR_MA**); click the Pick B row then click the shale mode (param **GR_SH**); press each row's **Set** button.
5. Ctrl+wheel to zoom the X axis, drag to pan, double-click to reset zoom.
6. **★ Save template** ("GR-QC"), change bins to 20, then recall "GR-QC" from the **— Template —** dropdown.
7. Change Zone to a real zone — data windows; n drops.
8. Negative: pick a constant FLAG curve.
   **Expected:** P3/P97 markers appear (Jauhar's GR normalization anchors); cumulative curve rises to 100% at right; box strip spans P5–P95 with P50 line at the bar mode; status confirms "GR_MA = … set on zone '…' " and the value lands in the zone's parameters (visible in Zones dialog / used by the next VSH run); axis label reads "n = X of Y" whenever the P2–P98 window clips tails — never contradicting the chips; template recall restores every option; the constant curve draws one central bar, NOT "No valid data". Petro check: GR histogram of a sand-shale interval is bimodal; GR_MA < GR_SH.
   **Result — T-PLOT-10:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-11 — Crossplot: log axes, regression (all models × methods), Z-color

**Tool/panel:** Crossplot — `src/ui/crossplotPanel.ts` (covers REVIEW.md §Performance "Crossplot: Z coloring memoized across pan/zoom/hover")
**Preconditions:** Well with PHIE + a permeability curve (or CPERM), GR, NPHI, RHOB.
**Steps:**

1. Ribbon **Plot ▸ Crossplot**. Default opens NPHI (X) vs RHOB (Y) colored by GR — RHOB axis must be **inverted** (density increases downward, D-N convention).
2. Set X = PHIE, Y = PERM. **⚙ Properties**: tick **Y log**; **Regression line** on; Model = **Exponential Y = 10^(a + b·X)**; try each Method (**Y on X**, **X on Y**, **RMA**); **Apply** each.
3. Switch Model through **Linear**, **Power**, **Log** — equation text and line shape change accordingly (a power fit is straight only when both axes are log).
4. Color = a permeability or SW curve; Properties → Colormap **Viridis (log-safe)** + **Log Z scale** — color bar relabels "(log)".
5. Color = FACIES — categorical swatch legend (F1, F2, …) replaces the color bar.
6. Pan/drag and Ctrl+wheel with a dense cloud colored by Z — motion must stay smooth (memoized colors).
   **Expected:** R², n, and the fitted equation display in real units; por-perm exponential/power fit slope is positive (perm rises with porosity); Z coloring identical before/after pan (only a Z/colormap change recolors); "— None —" Z gives single-color points.
   **Result — T-PLOT-11:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-12 — Crossplot overlays: N-D chartbook, matrix points, core, Thomas-Stieber

**Tool/panel:** Crossplot overlays — `src/ui/crossplotPanel.ts`, `chartOverlays.ts` (covers REVIEW.md §P2-f+ — D-N chartbook overlay: "A real Mahakam sand interval should plot on/left of the quartz sandstone line" and its "Gating" item)
**Preconditions:** NPHI-RHOB pair; core data; VSH + PHIT computed.
**Steps:**

1. X = NPHI, Y = RHOB. Properties → **Chart overlay** = **Por-11 CNL (fresh, rhof 1.0)** (listed under "For these axes"); also tick **Matrix points (Qtz/Cal/Dol on NPHI-RHOB)**; Apply.
2. Verify lithology: a clean water-bearing Mahakam sand interval (window via a zone) plots on/left of the quartz sandstone matrix curve; shale points trend toward high NPHI / moderate RHOB; the Qtz point sits at (−0.02, 2.65), Cal (0.00, 2.71), Dol (0.02, 2.87).
3. Negative gating: set X = GR — the chart overlay silently disappears (axes no longer match); set X back and tick **X log** — also suppressed.
4. Tick **Core data (diamonds)** with X = PHIE, Y = PERM — core plugs draw as diamonds; only depths with BOTH measurements plot.
5. Tick **T-S triangle**: axes auto-switch to VSH vs PHIT (status message). Drag the **PHI_SD_MAX** (VSH=0) and **PHI_SH** (VSH=1) circle handles vertically; release.
   **Expected:** Chart curves stay registered under zoom/pan (drawn in data space); T-S laminated line runs sand→shale, dispersed line dips to its porosity minimum at VSH = PHI_SD; on release the status confirms "PHI_SD_MAX = … set on zone …" / "PHI_SH = … set on zone …" (zone-parameter write). Petro check: laminated sand-shale data should scatter between the laminated and dispersed lines.
   **Result — T-PLOT-12:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-13 — Crossplot parameter handle, click-pick, zoom-to-cursor

**Tool/panel:** Crossplot picks — `src/ui/crossplotPanel.ts`
**Preconditions:** NPHI-RHOB crossplot; a zone selected in the Zone dropdown.
**Steps:**

1. With **Show parameter pickers** on, confirm the ringed handle sits at the cloud median; pick rows read "X pick → NPHI_SH", "Y pick → RHO_SH".
2. Drag the handle onto the shale cluster; release.
3. Click empty plot space — the handle/marker jumps there.
4. Ctrl+wheel at a cluster — zoom centers on the cursor; drag background pans (the handle grab must NOT pan); double-click resets zoom; double-click again (unzoomed) opens Properties.
5. Cancel test: press Esc / close the Properties dialog without Apply — no settings change.
   **Expected:** Release writes BOTH zone parameters (status "NPHI_SH = … set on zone '…' " then "RHO_SH = …"); values honor the drop position (check against the axes); a failed write must surface "Failed to set …", never silent success. Petro check: shale point in a Mahakam shale cluster ≈ NPHI 0.3–0.45, RHOB 2.3–2.6 g/cc.
   **Result — T-PLOT-13:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-14 — Pickett plot: two-point pick ⇄ typed M/Rw, Z-color, axis config

**Tool/panel:** Pickett — `src/ui/pickettPanel.ts` (covers REVIEW.md §Polish — UX "Pickett v2 — properties dialog, typed M/Rw, configurable axes, Z-color")
**Preconditions:** Well with RES_DEEP + computed PHIE; VSH or SW curve for Z-color.
**Steps:**

1. Ribbon **Plot ▸ Pickett**. RT = RES_DEEP, Porosity = PHIE, N = 2.
2. Click TWO points along the lowest-RT (water-bearing) trend at different porosities.
3. Type a different **Rw** in the toolbar (e.g. 0.20); then a different **M** (e.g. 1.9).
4. Negative: pick two points at the SAME porosity — status must say it can't fit ("pick points at different porosities"), no line.
5. **⚙** (or right-click) → RT axis 0.2 → 200, PHIE axis 0.05 → 0.5, **Color by** = VSH, Colormap Viridis; **Apply**.
6. Close the pane, reopen Pickett — axis/point/Z settings persist.
   **Expected:** After the two-point pick the **M and Rw fields fill** with the fit and the solid Sw=1 line plus dashed Sw=0.5/0.25 lines draw; annotation reads "Sw=1 line: M = …, Rw = … ohmm (N = 2)". Typing Rw/M moves the lines instantly — verify geometrically: the Sw=1 line must pass through RT = Rw at PHIE = 1.0. Petro checks: M ≈ 1.7–2.2 for Mahakam sands; points above/right of the water line are hydrocarbon-bearing (higher RT at same φ); low-VSH points should lie closest to the water trend when colored by VSH. Pick rows write **M** / **RW** to the zone via **Set**.
   **Result — T-PLOT-14:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-15 — Correlation: strips, tops connected, flatten, TVDSS, contacts, live well list

**Tool/panel:** Correlation — `src/ui/correlationPanel.ts` (covers REVIEW.md §Polish — UX "Correlation: fresh well list + Ctrl+wheel zoom" and §Round 3 "(9) Fluid contacts in Correlation")
**Preconditions:** ≥3 wells sharing at least one top name; one deviated well with a TVDSS curve.
**Steps:**

1. Ribbon **Plot ▸ Correlation**. All group wells appear as side-by-side GR strips; tops draw with dashed connectors between adjacent strips.
2. **Wells (n/m)…** — untick one well; it disappears; retick.
3. Curve dropdown → RHOB; set **min/max** fields (1.95 / 2.95).
4. Datum dropdown → **Flatten on _TopX_**; wells lacking the top label "(no datum)".
5. Depth mode **MD → TVDSS**.
6. **Contacts…** → **＋ Add contact**, type OWC, set a depth, tick **TVDSS**, close. Toggle MD ↔ TVDSS.
7. **Ctrl+wheel** over a strip — zooms about the cursor depth; plain wheel pans; **Fit** refits all wells.
8. Import a new LAS (Data ▸ Import Logs) with the panel open — the new well must appear as a strip without reopening (dataVersion).
9. Negative: untick ALL wells — panel must show "No wells included — pick some under Wells…", no crash.
   **Expected:** Flattening puts the datum top at display depth 0 as a dashed accent line and the connected top lines become horizontal; a TVDSS-stored contact drawn in TVDSS mode is **perfectly flat across every well** including the deviated one (in MD mode it shifts per well); geological sanity: tops should not cross between adjacent wells in a layer-cake section.
   **Known issue:** AUDIT finding "No batch-run dialog re-scopes to a new active well group while it's already open — only the Wells sidebar tree and Map pane react live to a group switch" — switching the active well group while Correlation is open will NOT refilter the strips/Wells menu until a data event (e.g. an import) fires. Log as known, not new.
   **Result — T-PLOT-15:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-16 — Synchronized hover crosshair across all open plots

**Tool/panel:** hoverDepth broadcast — `logViewPanel.ts`, `histogramPanel.ts`, `crossplotPanel.ts`, `pickettPanel.ts`, `correlationPanel.ts`
**Preconditions:** Same well open in: two Log Views, a Histogram (GR), a Crossplot, a Pickett; Correlation open with that well included.
**Steps:**

1. Slowly move the cursor down one log view.
2. Watch every other pane simultaneously.
3. Hover a strip in Correlation instead.
4. Move the cursor off the canvas.
   **Expected:** The second log view draws a horizontal crosshair line at the same depth; the histogram shows a marker at the GR value of that depth; crossplot and Pickett ring the sample nearest that depth (ring skips when the sample is NaN); hovering Correlation drives the same crosshairs in the log views — with Correlation in TVDSS mode the broadcast still lands at the correct **measured** depth in the log view (check against a top). Leaving the canvas hides all crosshairs/markers. Cross-check the depth number in the log-view readout equals the correlation hover depth.
   **Result — T-PLOT-16:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-17 — Plot export: Copy / Image / Print + per-plot right-click menu

**Tool/panel:** `src/ui/plotExport.ts` toolbar + workspace context menu
**Preconditions:** Histogram, Crossplot, Pickett, Correlation panes open with data.
**Steps:**

1. On each of the four plots click **⧉ Copy** — paste into Paint/Word to verify.
2. Click **⭳ Image** — save a PNG to disk; open the file.
3. Click **⎙ Print** — the print dialog shows ONLY the plot image, not the app chrome; cancel it.
4. Right-click each plot pane — menu shows the plot heading plus **Copy image / Save image… / Print…** and "New _kind_ window"; the crossplot/histogram canvas itself opens its own Properties on right-click instead (that is by design — use the pane tab/margin for the workspace menu).
5. Open the **History** panel.
6. Negative: cancel the save dialog — no file, no false success status.
   **Expected:** Status line confirms "_name_ copied to clipboard" / "_name_ image saved to _path_"; the PNG matches the on-screen plot including overlays and Z color bar; History gains "[Export]" entries for copy and save; the Log View has no image-export buttons (it exports via Composite — WebGPU canvas, by design).
   **Result — T-PLOT-17:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-18 — Curve Edit dialog: all five ops, bit-exact undo, History

**Tool/panel:** Curve Edit — `src/ui/curveEditDialog.ts` from log-view right-click
**Preconditions:** Log view on a well with GR; History panel open; note some GR values via the readout first.
**Steps:**

1. Right-click the GR track at a known depth → menu heading "Track _X_" → **Edit GR…**. Dialog "Edit GR — _well_".
2. Operation **Wireline shift**, Shift (m) = 2 → **Apply**. Verify the curve moved 2 m deeper; **Project ▸ Edit ▸ Undo** → verify the readout values return exactly.
3. **Set constant**: Top/Bottom span ~5 m, Value = 75 → Apply → interval is flat at 75 → Undo → original values back bit-exact.
4. **Blank (erase)** over an interval → gap appears (line pen lifts) → Undo.
5. **Interpolate across** a spike → the spike bridges linearly between the interval edges → Undo.
6. **Scale a·v + b** with a = 1.1, b = 5 → GR visibly recalibrated → Undo.
   **Expected:** Each Apply: status "_op_ GR (_well_) — N samples changed (Ctrl+Z undoes)", every open plot of that well refreshes (histogram re-bins, log view repaints — dataVersion), and a "[Edit]" History entry appears with the op, curve, well and sample count; each Undo restores values **bit-exactly** (readout matches pre-edit to the last digit) and also bumps the plots. "Nothing changed" status when the interval contains no samples.
   **Result — T-PLOT-18:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-19 — Curve Edit negative tests (invalid input, stale undo)

**Tool/panel:** Curve Edit — `src/ui/curveEditDialog.ts` + backend `curve_edit.rs`
**Preconditions:** T-PLOT-18 done; a VSH module you can re-run on the well.
**Steps:**

1. Open **Edit GR…** → **Set constant** → **clear the Value field empty** → Apply. Observe what is written over the interval. Undo afterwards.
2. Repeat with Value = "abc" (if the field allows typing it) and with "1e999".
3. Stale-undo probe: **Set constant** on a VSH curve interval → re-run the VSH module (recomputing VSH) → now press Ctrl+Z on the old edit. Compare the VSH curve against the fresh module output.
   **Expected:** Ideally Apply should refuse an empty/invalid Value with a hint.
   **Known issue:** AUDIT finding ""Set constant" (and other numeric fields) silently coerce invalid/empty input to 0.0, not an error" — expect step 1/2 to silently overwrite the interval with 0.0 and report "N samples changed" as success. Also AUDIT finding "restore_curve_values (the undo path) has no staleness/version check" — expect step 3's Ctrl+Z to silently splice pre-edit values over the freshly recomputed VSH with an unqualified "Undo: …" success. Log both as known, not new; recover with a module re-run.

   **Automated coverage - pinned (pile B, 2026-07-31):** `a_set_constant_refuses_a_value_that_is_not_a_number` (curve_edit.rs) confirms the BACKEND refuses a non-finite constant and writes nothing — the curve is re-read after each refusal, because "returns an error" and "changed nothing" are different claims and only the second protects the data. `an_undo_replayed_after_the_curve_was_rewritten_splices_stale_values` pins the stale-undo gap as-is, on a computed curve (VSH), which is the scenario step 3 describes.

   **Steps 1 and 2 are still a FAIL, but the reason has moved (2026-07-31, finding 19).** The backend guard is now correct and is unreachable: `curveEditDialog.ts:88` turns any unparseable field into its default, which for Value is **0** — a finite number that passes the guard and gets written. So expect the plan's outcome exactly as written, from a different cause. One correction to step 2: `1e999` no longer writes +Infinity, it writes **0.0** as well; the Infinity half of that audit finding was fixed and the empty/garbage half was not. The reason it went unnoticed is worth knowing — an empty `add` field falls back to 0 and an empty `mul` to 1, which are both no-ops, and "set a constant" is the one field where the fallback is a real reading rather than an identity.

   **Step 3 is a FAIL as written, and the damage is worth looking at directly.** The undo matches depths bit-exactly against whatever the curve holds NOW, with no version check, so it splices pre-edit values over the recomputed curve and reports success — one curve, two vintages, nothing on the log or in the provenance to say where the boundary falls. There is a second, opposite failure: if the module re-ran on a different sampling the old depths match nothing, so the undo writes nothing and ALSO reports success. Both are silent, which is why a module re-run is the recovery for either.
   **Result — T-PLOT-19:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PLOT-20 — dataVersion refresh preserving viewport; theme repaint; clean dispose

**Tool/panel:** All plot panels — `state.ts` dataVersion/themeVersion wiring (covers REVIEW.md §Round 4 "dataVersion refresh" and §Cutoff Sensitivity theme-switch item pattern)
**Preconditions:** Log View, Histogram (zoomed in X), Crossplot (zoomed + parameter handle placed), Pickett (water line fitted, zoomed) all open on one well. DevTools console open (Ctrl+Shift+I) for step 4.
**Steps:**

1. Run a module that rewrites a displayed curve (e.g. re-run VSH or an equation on GR-derived output).
2. Verify each open plot refreshed its data **without** losing state: histogram keeps its X zoom; crossplot keeps zoom AND the placed handle; Pickett keeps zoom, picks and the M/Rw line; the log view keeps its scroll position.
3. Project tab → **Theme** → switch Light → Dark → **Pertamina** with all panes visible.
4. Close each plot pane with its ✕; also close a log view mid-well-load (open, immediately close).
   **Expected:** (2) fresh curve values appear in every plot (no stale data), viewport/picks preserved. (3) every canvas repaints in the new palette immediately — WebGPU log view background + curves, plot frames/text, core-overlay diamond outlines — with no interaction needed and no white-flash panes left behind. (4) no console errors on dispose; after closing, hovering remaining views raises no errors (subscriptions cleaned up); reopening a closed plot works normally.
   **Result — T-PLOT-20:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

# Section REP — Reporting & database access

## REP — Reporting & database access

Shared preconditions for this cluster: app running via `npm run tauri dev`; a project with at least 3 LAS wells imported (GR + RES_DEEP + NPHI + RHOB), interpretation already run so VSH / PHIE / SWE computed curves exist on at least 2 wells; at least one zone defined (Petrophysics ▸ Zones…) with one parameter override (e.g. RW) so the zone-parameter table is non-trivial; one well in the project with **no** standard-curve rows (e.g. a DLIS-only well whose curves live only in the RAW store — verify via DB Inspector ▸ Standard Curves showing 0 rows) for the batch-failure test. Composite… and Report… live on the **Plot** ribbon tab (Deliverables group); DB Inspector and SQL Query on the **Data** tab (Manage group); Processing History opens from the clock icon in the Quick Access Toolbar.

### T-REP-01 — Composite & Report panes open, follow the selected well

**Tool/panel:** Composite Log + Report dock panes (src/ui/compositeDialog.ts, reportDialog.ts, workspace.ts)
**Preconditions:** Project open; **no** well selected yet (Ctrl-click to deselect in Wells & Tops if needed).
**Steps:**

1. Plot tab → click **Composite…**, then **Report…**.
2. With no well selected, read both panes' content.
3. In Wells & Tops, select a well with full curves.
4. Right-click inside an open Log View → click **Print / export layout…**.
   **Expected:** Both panes open as dockable panes (not popups). With no well: each shows "Select a well (Wells & Tops) — … will follow", tab titles plain "Composite Log" / "Report". After selecting a well: both panes fill in their forms and tab titles become "Composite Log — {well}" / "Report — {well}". Step 4 focuses the existing Composite pane (singleton, no duplicate). Covers REVIEW.md §All tools as dockview panes (2026-07-20 #24), unchecked items "Open the Zones / Composite / Report pane with no well selected" and "Docking sanity … 'Print / export layout…' opens the Composite pane".
   **Result — T-REP-01:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-02 — Composite render: layout, print scale, page size, pagination

**Tool/panel:** Composite Log pane (src/ui/compositeDialog.ts → src-tauri/src/composite.rs)
**Preconditions:** Well with GR+RHOB+NPHI selected; a layout exists (built-in is fine).
**Steps:**

1. In the Composite pane leave **Layout** on the active layout, **Print scale** = 1:500 (default), **Page size** = A4 (210×297), depth fields blank.
2. Click **Render**. Note the page count in the status line "{well}: N page(s) at 1:500."
3. Page with **◀ / ▶**; read the label "Page i / N · top–bot m".
4. Change **Print scale** to 1:200 → **Render**. Then 1:1000 → **Render**.
5. Change **Page size** to A3 (297×420) → **Render**.
   **Expected:** A vector preview appears (tracks, depth grid, curve traces matching the on-screen Log View for the same layout). Page 1 header shows well name, "Field: … TD: … KB: …", "Layout: {name} Scale 1:{n} Interval {top}–{bot} m", and the grey footer "Made in SandiBumi — composite log". Depth per page must be physically exact: at 1:500 an A4 track window covers ~2.5× the metres of 1:200 — so 1:200 gives ≈2.5× the pages of 1:500, and 1:1000 halves the 1:500 count; each page label's top–bottom range must tile the full logged interval with no gaps/overlap. A3 gives fewer pages than A4 at the same scale. ◀ is disabled on page 1, ▶ on the last.

   **Automated coverage - pinned (pile B, 2026-07-31):** `a_metre_of_formation_occupies_its_declared_millimetres_on_the_page` (composite.rs) measures the scale in the ARTWORK rather than asserting arithmetic — it reads every depth label off the emitted page and checks each adjacent pair spans exactly 1000/scale mm, at 1:200, 1:500 and 1:1000. Every pair, not just the ends, so a scale that drifted down the page could not pass. `the_page_count_follows_the_print_scale_and_the_page_size` covers the counts, the exact 2.5x page-height ratio between 1:500 and 1:200, and the tiling. What is NOT covered and is still yours: the on-screen preview matching the Log View, the page label text, and ◀ / ▶ enable states.

   **Worth knowing before you click: "A3 gives fewer pages" can come out EQUAL, and that is not a failure.** Page count is a step function — the extra height only costs a page when it crosses a boundary. On the test well (199.5 m) A3 and A4 both give 2 pages at 1:500, and A3 only wins at 1:200. If your well is short, compare at the finest scale. Related: the FIRST page holds fewer metres than the ones after it (its metadata header is 32 mm against 8 mm), which is why the plan says "≈2.5x" rather than exactly 2.5 — the exact ratio is between two pages of the same kind.
   **Result — T-REP-02:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-03 — Composite depth window + invalid window (negative)

**Tool/panel:** Composite Log pane (src/ui/compositeDialog.ts)
**Preconditions:** T-REP-02 rendered once.
**Steps:**

1. Enter a **Depth top / bottom (m)** window inside the logged interval (e.g. 1500 / 1700) → **Render**.
2. Clear both fields → **Render** (hint says "Blank = full logged interval").
3. Enter top **below** bottom (e.g. top 1700, bottom 1500) → **Render**.
4. Enter a window entirely outside the logged interval (e.g. 9000 / 9100) → **Render**.
   **Expected:** (1) Pages cover only 1500–1700 m; page labels confirm. (2) Full interval returns. (3)+(4) A clear failure in the status line ("Render failed: …" e.g. empty depth range) — no crash, no stale preview left claiming to be the new window; **Save SVG…/Save PDF…** become disabled after a failed render.

   **Automated coverage - pinned, with a residual (2026-07-31):** `a_depth_window_that_selects_no_rock_is_refused_rather_than_rendered` (`composite.rs`) asserts the backend refuses all four ways of selecting no rock - top below bottom, wholly under TD, wholly above the logged top, and zero thickness. NOT asserted: the display surface - that the status line shows the message, that the previous preview is not left on screen claiming to be the new window, and that Save SVG/PDF disable. Those three are the whole point of steps 3 and 4 and are still yours.

   **Worth knowing before you click:** a window that only PARTIALLY overlaps the logged interval does NOT fail - it renders the overlap. Ask for 1500 - 9000 on a well logged to 2000 and you get 1500 - 2000, with the page labels saying 2000. That is correct (you cannot render rock that was never logged) but it means step 4 only fails when the window misses the data ENTIRELY.
   **Result — T-REP-03:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-04 — Composite export SVG

**Tool/panel:** Composite Log pane (src/ui/compositeDialog.ts, exportCompositeSvg)
**Preconditions:** Successful render in the pane (multi-page, 1:500/A4).
**Steps:**

1. Click **Save SVG…**; accept the default name `{well}_composite.svg`; save to a scratch folder.
2. Read the status line; open the folder; open one SVG in a browser.
   **Expected:** Status "Wrote N file(s): …" — one SVG per preview page. Each SVG opens as vector graphics (text selectable/sharp at any zoom, not a bitmap) and matches the corresponding preview page: same tracks, curve shapes, depth annotations, header, "Made in SandiBumi — composite log" footer.
   **Result — T-REP-04:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-05 — Composite export PDF, verify against on-screen log view

**Tool/panel:** Composite Log pane (src/ui/compositeDialog.ts → src-tauri/src/composite.rs assemble_pdf)
**Preconditions:** Successful render; an open Log View on the same well with the same layout for comparison.
**Steps:**

1. Click **Save PDF…** → save `{well}_composite.pdf`.
2. Open the PDF in a reader at 100 %.
3. Compare page by page against the preview and the Log View: track order, curve scales printed in the track headers (min/max, log tags), tops/zone bands, depth numbers.
4. Measure one page: at 1:500 on A4, 10 cm of paper track = 50 m of depth (check against the depth grid).
5. (If a FACIES curve exists) switch Layout to the built-in **Facies** layout, re-render, re-export → the FACIES track prints as solid colored rectangles.
   **Expected:** One multi-page PDF; every preview page present in order; header block (well/field/TD/KB, Layout/Scale/Interval) and footer "Made in SandiBumi — composite log" on page 1; curve geometry and track scales identical to the on-screen view; the paper-scale check in step 4 holds within measurement error. Step 5 covers REVIEW.md §FACIES block track, unchecked item "Composite export shows the blocks".
   **Result — T-REP-05:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-06 — Report render: cover, methodology, zone params, pay summary, composite pages

**Tool/panel:** Report pane (src/ui/reportDialog.ts → src-tauri/src/report.rs)
**Preconditions:** Well with VSH/PHIE/SWE computed and ≥1 zone with a param override selected.
**Steps:**

1. Plot tab → **Report…**. Set **Study title** (default "Petrophysical Evaluation — {field}"), **Prepared by** = your name.
2. Leave **Cutoffs VSH/PHIE/SWE/PERM** at defaults 0.5 / 0.1 / 0.6 / blank; **Tables only** unchecked.
3. Click **Render**; page through with ◀ / ▶.
   **Expected:** Status "{well}: N report page(s)." Page order: (1) cover with title, well, interval, "Prepared by: {name}"; (2) Methodology table (Parameter | Method | Remarks — defaults if you typed nothing); (3) Zone Parameters table listing your zones (zones without params show "-"), your RW override visible; (4) Pay Summary table titled "Pay Summary (VSH ≤ 0.50, PHIE ≥ 0.10, SWE ≤ 0.60)" with per-zone SAND/RESERVOIR/PAY rows — domain check: Net ≤ Gross, 0 ≤ NTG ≤ 1, avg VSH low on SAND rows, avg PHIE plausible for Mahakam sands (~0.1–0.3), HPV ≥ 0 and PAY ⊆ RESERVOIR ⊆ SAND (each successive Net no larger); (5) the composite pages. Each table page footer: "Made in SandiBumi".

   **Automated coverage - pinned (pile B, 2026-07-31):** `a_rendered_report_carries_the_plans_page_order_and_a_self_consistent_pay_table` (report.rs) renders the real document and checks the page order (cover → Methodology → Zone Parameters → Pay Summary, located by first occurrence so a table that paginates does not break it), the cover's title/well/"Prepared by", the exact pay-section title string, the RW override listed by name AND value with an unoverridden zone still shown, and that `tables_only` genuinely stops there. The invariants are checked on the computed rows AND the printed nets are checked to match them, so they are pinned to what ships. The fixture stops every fourth sample at a different cutoff, so SAND/RESERVOIR/PAY are **strictly** decreasing rather than merely non-increasing.

   **KNOWN ISSUE found while writing that test (2026-07-31) — step 3's footer expectation is wrong:** the table pages carry **no footer at all**. "Made in SandiBumi" is emitted by the cover, by every composite page, by the Word document and by the PowerPoint deck — but not by `table_pages`, so the methodology, zone-parameter and pay-summary pages are the only unmarked surface in the deliverable set. Everything else in step 3 checks out. Log as known; whether the mark belongs on every page or only the cover is your call.

   **KNOWN ISSUE (2026-07-31) — "HPV ≥ 0" is not an invariant:** the pay summary sums PHIE·(1−SWE)·h with no floor, so a **negative PHIE inside net sand is subtracted**. A tight carbonate streak reads low GR, clears the VSH cutoff and is flagged SAND, while a density porosity on a sandstone matrix reads slightly negative there. Measured: 2.5 m of streak at PHIE = −0.05 through a 5 m zone understates the SAND row's HPV by over 20%. RESERVOIR and PAY are byte-identical either way (the streak fails the porosity cutoff), so the two rows you check first agree while the SAND row quietly does not. Pinned as-is by `a_dense_stringer_is_subtracted_from_the_sand_rows_hpv`.
   **Result — T-REP-06:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-07 — Methodology table edit persists as report_template document

**Tool/panel:** Report pane (src/ui/reportDialog.ts, TEMPLATE_DOC_TYPE "report_template")
**Preconditions:** Report pane open.
**Steps:**

1. In **Methodology table** type 3 rows, one per line, pipe-separated, e.g. `VSH | Clavier from GR | P3/P97 normalized`.
2. Click **Save Template** → expect status "Methodology template saved."
3. Click **Render** → methodology page shows your 3 rows.
4. Close the Report pane; reopen via **Report…** → the textarea is pre-filled with your rows.
5. Data tab → **SQL Query** → run: `SELECT doc_type, name, json FROM documents WHERE doc_type = 'report_template'`.
6. Restart the app (`npm run tauri dev` again), reopen Report.
   **Expected:** Steps 4 and 6: the edited rows survive pane reopen AND app restart. Step 5: exactly one row, doc_type `report_template`, name `default`, json a serialized array of your {parameter, method, remarks} rows. Clearing the textarea and rendering falls back to the built-in default methodology.
   **Result — T-REP-07:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-08 — Report export single PDF

**Tool/panel:** Report pane (src/ui/reportDialog.ts, exportReportPdf)
**Preconditions:** Successful render in T-REP-06.
**Steps:**

1. Click **Save PDF…** → save `{well}_report.pdf`.
2. Open in a reader; compare all pages against the preview.
3. Zoom the bottom of a table page.
   **Expected:** One multi-page PDF matching the preview page-for-page (cover → methodology → zone params → pay summary → composite); the same pay-summary numbers as on screen; centered grey footer "Made in SandiBumi" on report pages and "Made in SandiBumi — composite log" on the composite pages. Status line "Wrote {well}\_report.pdf" and the app status bar "Report PDF exported for {well}."
   **Result — T-REP-08:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-09 — "Tables only" mode

**Tool/panel:** Report pane (src/ui/reportDialog.ts → src-tauri/src/report.rs report_pages)
**Preconditions:** Same well as T-REP-06; note the wall-clock time of the full render.
**Steps:**

1. Tick **Tables only (no composite)** → **Render**. Time it roughly.
2. Page through; then **Save PDF…**.
   **Expected:** Output is correct: cover + methodology + zone params + pay summary only, **no** composite pages, and the cover still states the true logged interval. However the render should feel meaningfully faster than the full render — it currently will NOT be.
   **Known issue:** AUDIT-2026-07-21 (Viz/reporting #3) — "Report generator's 'Tables only' mode still does the full composite computation — it only skips appending the result." Expect tables-only render time ≈ full render time; log as known, not new. Output content itself should still be correct.

   **Automated coverage - pinned (pile B, 2026-07-31):** `tables_only_drops_the_composite_pages_and_still_dates_the_cover_to_real_rock` (report.rs) confirms the output half — exactly four pages in order, no composite, the cover stating the true logged interval, TD and KB — with a full render beside it as the control, so a tables-only mode that silently dropped a TABLE could not pass. Timing is not asserted (a test cannot honestly time a render on a build machine); the slowness is still yours to observe.

   **Worth knowing before you click: the known issue is NOT a missing `if`, and that changes what a fix costs.** `report_pages` renders the composite unconditionally and skips only the appending, because the cover's interval is read off the composite's own pagination — the expensive render supplies the cover's last remaining fact. Remove it naively and the cover prints "Interval: 0.0 – 0.0 m" on a client document. So expect the slowness to persist until the cover gets its own depth query; it is a coupling, not an oversight.

   **KNOWN ISSUE (2026-07-31, finding 18) — OPEN, your call.** Because the interval comes from the pagination, it follows the composite's **print window**. Set a depth window and the cover re-dates the whole report — including the pay table, which is computed per zone and ignores the window entirely. A report rendered over 1005–1010 m carries a table covering every zone in the well under a cover announcing a 5 m interval, and on a tables-only render there are no log pages left to show the reader that the window was only a print setting. If you want to see it: set a narrow window, tick Tables only, and compare the cover against the zones in the pay table.
   **Result — T-REP-09:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-10 — Cross-check: report render writes FLAG\_\* curves; catalog + open plots refresh

**Tool/panel:** Report pane × Curve Catalog (inspectorPanel.ts) × Log View (dataVersion)
**Preconditions:** A well with VSH/PHIE/SWE whose FLAG_SAND/FLAG_RESERVOIR/FLAG_PAY have never been computed (check Curve Catalog first — Data tab ▸ **Curve Catalog**); a Log View open on that well with a layout containing a FLAG_PAY track (Layout Properties ▸ add curve FLAG_PAY, fill "Facies blocks" works well).
**Steps:**

1. Confirm FLAG_PAY absent from the Curve Catalog for this well.
2. In the Report pane click **Render** (report render intentionally persists the pay flags).
3. Without touching anything else, look at the open Log View's FLAG_PAY track and re-open the Curve Catalog.
   **Expected:** Immediately after render: FLAG_SAND, FLAG_RESERVOIR, FLAG_PAY appear in the Curve Catalog for the well; the already-open Log View repaints its FLAG_PAY track (dataVersion bump — no manual refresh, no well re-select). Domain check: FLAG_PAY = 1 only where VSH ≤ 0.5, PHIE ≥ 0.1, SWE ≤ 0.6 — spot-check one flagged interval against the curves. Covers REVIEW.md §Round 4 — AUDIT-2026-07-21 safe-bucket follow-through, unchecked item "dataVersion refresh after equation / ML / report runs".
   **Result — T-REP-10:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-11 — Cross-check: composite/report exports in Processing History

**Tool/panel:** Processing History panel (src/ui/historyPanel.ts) × composite/report exports
**Preconditions:** T-REP-04, T-REP-05, T-REP-08 done in this session.
**Steps:**

1. Click the clock icon in the Quick Access Toolbar ("Processing history — everything done in this project").
2. Search the list for Export entries for the composite SVG, composite PDF, and report PDF you just wrote (compare: an **Export LAS…** run and plot-image exports DO log entries like "Exported LAS (N rows) → path").
   **Expected:** By the app's own convention every export should appear as an "Export" entry with well name and destination.
   **Known issue:** AUDIT-2026-07-21 (Viz/reporting #2) — "Report generator's Render/Save/Batch actions persist FLAG\_\* … but never … record to History": neither compositeDialog.ts nor reportDialog.ts calls recordProcess, so expect NO History entries for composite/report exports (the dataVersion half of this finding has since been fixed; the History half has not). Log as known.
   **Result — T-REP-11:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-12 — Batch export, one PDF per well, with a broken well in scope

**Tool/panel:** Report pane batch (src/ui/reportDialog.ts, wellScope.ts → src-tauri/src/report.rs export_report_batch; job via lib.rs "Report batch")
**Preconditions:** ≥2 good wells + the curve-less well (see cluster preconditions) all in one well group; that group active in the pane's scope selector.
**Steps:**

1. In the Report pane set the scope selector so the button reads **Batch (N wells)…** with N including the broken well.
2. Click **Batch (N wells)…** → pick an empty destination folder.
3. Watch the status line and the **Processing** panel while it runs.
4. When finished, open the destination folder; open one good well's PDF.
   **Expected:** Per-well failure isolation: every good well gets `{WELL}_report.pdf` (non-alphanumeric name chars become `_`), each a complete report for THAT well (check the cover well name differs per file). The broken well is skipped, not fatal: the status line reports the mixed outcome — "Batch export: wrote {N−1} file(s); failed: {well_id}: no curve data for this well" (note the failure is identified by well **UUID**, not name — worth logging as UX feedback). The Processing panel shows a "Report batch" job entry. No partial/corrupt PDF for the failed well in the folder.

   **Automated coverage - pinned, with a residual (2026-07-31):** `one_unrenderable_well_costs_only_itself_in_a_batch_export` (`report.rs`) asserts the isolation - the broken well is listed FIRST, both healthy wells still get their own complete PDF (byte-different from each other, so the cover well really did change), and the broken well leaves no file at all. The UUID-not-name failure message you flagged is pinned as current behaviour too. NOT asserted: the status line wording and the Processing panel entry. Those are still yours.

   **KNOWN ISSUE found while writing that test (2026-07-31) - worth adding a step:** if two wells in scope share a name, the second report **silently overwrites the first**, and the batch still reports both as written. A 3-well batch says "wrote 3 file(s)" with 2 files in the folder. The filename comes from the well name with every non-alphanumeric mapped to `_`, so `SANDI/1` and `SANDI 1` collide as well. Nothing warns. Pinned as-is by `two_wells_with_one_name_silently_overwrite_each_others_report`; the fix (suffix the duplicate, or fall back to the well id) changes delivered filenames, so it is logged in `ROADMAP.md` §B1 as **your call** rather than done. If you have duplicate well names in your project, add them to this test's scope and check the folder count against the status line.
   **Result — T-REP-12:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-13 — Export cancels and empty scope (negative)

**Tool/panel:** Composite + Report panes (compositeDialog.ts, reportDialog.ts)
**Preconditions:** A successful composite and report render on screen.
**Steps:**

1. Composite pane → **Save PDF…** → in the file dialog press **Cancel**.
2. Report pane → **Batch (N wells)…** → in the folder dialog press **Cancel**.
3. Set the report scope to a selection containing zero wells (e.g. an empty group) so the button reads **Batch (0 wells)…** → click it.
   **Expected:** (1)+(2) No file written, no error, buttons re-enable, previous status text intact or unchanged — the app never hangs on a cancelled dialog. (3) Status "No wells in scope — pick a group, pin/select wells, or choose All." and no folder dialog opens.
   **Result — T-REP-13:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-14 — DB Inspector: browse all 8 tables, page through

**Tool/panel:** Database Inspector pane (src/ui/dbInspectorPanel.ts → src-tauri/src/db.rs get_table_page / TABLE_SPECS)
**Preconditions:** Well with >200 curve samples selected.
**Steps:**

1. Data tab → **DB Inspector**.
2. Cycle the **Table** dropdown through all 8 entries: Wells, Standard Curves, Computed Curves, Tops, Zones, Zone Parameters, Core Data, Aux Data.
3. On Standard Curves (default) read the pager "1–200 of N"; click **▶** twice, then **◀**.
4. Note the scope caption per table.
5. Deselect the well (or pick none) and revisit Standard Curves.
   **Expected:** Every table renders its whitelisted columns (Wells: well_id/well_name/field_name/td/kb; Standard Curves: depth/gr/res_deep/nphi/rhob/dt/sp; Computed Curves: depth/curve_name/value; etc.). Wells shows "(whole project)" scope; all others show "Well: {name}". Paging is 200 rows a step, pager arithmetic correct, ◀ disabled on the first page, ▶ on the last. Depths strictly increasing within a page; GR values plausible (roughly 0–250 gAPI); NULL cells visibly distinct. Step 5: the grid shows "Select a well in Wells & Tops to browse Standard Curves." — no crash.

   **Automated coverage - pinned (pile B, 2026-07-31):** `every_inspector_table_returns_the_columns_it_declares` (db.rs) browses EVERY `TABLE_SPECS` entry and checks each returns exactly the columns it declares, in order, and that a well-scoped table refuses rather than quietly returning the whole project — a dropped filter would fill one well's grid with another well's samples, which looks like data, not an error. `the_inspector_pager_lands_exactly_on_the_last_partial_page` covers step 3's arithmetic on 250 rows paged 100 at a time, verifying every sample appears exactly once across the pages and in depth order. NOT covered and still yours: the scope captions, the ◀ / ▶ enable states, and the no-well placeholder in step 5.

   **Worth knowing before you click:** Core Data and Aux Data are the two tables the inspector shows WITHOUT the active-delivery filter every other reader uses — that is deliberate (`set_name` and `dataset` are in the column list so you can tell deliveries apart), but it means a well with two core deliveries shows both here while every plot and module sees only one. Do not read a doubled row count as a bug.
   **Result — T-REP-14:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-15 — DB Inspector: double-click edits in wells, standard_curves, zone_params — persist, refresh, undo

**Tool/panel:** Database Inspector pane (src/ui/dbInspectorPanel.ts, pushUndo)
**Preconditions:** As T-REP-14; a Log View open on the same well showing GR; a zone param row exists.
**Steps:**

1. Table = **Wells** → double-click the selected well's `field_name` cell → type a new value → **Enter**.
2. Table = **Standard Curves** → find a depth on screen in the Log View → double-click its `gr` cell → change e.g. 75.2 → 200 → **Enter**. Watch the Log View's GR track.
3. Table = **Zone Parameters** → double-click a `value_num` cell (e.g. your RW override) → change it → **Enter**.
4. Press **Ctrl+Z** three times, watching the grid and status bar.
5. Restart-check: re-do edit (1), restart the app, reopen DB Inspector ▸ Wells.
   **Expected:** Each commit: cell updates, status bar logs "edit {Table}.{column}: 'old' → 'new'". Step 2: the open Log View's GR trace shows the spike at that depth without manual refresh (dataVersion). Step 4: each Ctrl+Z reverts one edit in reverse order — grid values return, Log View spike disappears, zone param restores. Step 5: the (re-done) field_name edit survives restart — it is a real DB write. Esc during editing cancels without a write.
   **Result — T-REP-15:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-16 — DB Inspector negatives: bad input, stale-row edit, read-only Aux Data

**Tool/panel:** Database Inspector pane (dbInspectorPanel.ts → db.rs sample editors)
**Preconditions:** As T-REP-15; a well with an imported Aux dataset (petrography/XRD/perfs) for step 4 — skip step 4 if none.
**Steps:**

1. Standard Curves → double-click a `gr` cell → type `abc` → **Enter**.
2. Clear a `gr` cell completely → **Enter** (empty = set MISSING).
3. Stale-row: keep the Standard Curves grid on screen, then depth-shift or rewrite the well's curves elsewhere (e.g. Curve Editor depth shift), come back WITHOUT paging, and edit a now-moved row.
4. Table = **Aux Data** → double-click any cell.
   **Expected:** (1) "Edit failed: 'abc' is not a number" in the status bar; the cell reverts; Ctrl+Z does NOT replay a phantom edit. (2) The cell empties and persists as NULL/MISSING (verify with SQL: the sample reads NULL). (3) The 0-row update must error ("no … sample matched depth …"), cell reverts, no bogus undo entry — covers REVIEW.md §Low-tier correctness & data-integrity sweep (2026-07-21), unchecked item "DB-inspector edit no longer reports success on a 0-row update". (4) Nothing happens — Aux Data has no editable columns (hint: re-import the file to change values) — covers REVIEW.md §P2-a — Tops-style imports, unchecked item "View it: Data → DB Inspector → table 'Aux Data'".

   **Automated coverage - pinned (pile B, 2026-07-31):** `an_inspector_edit_on_a_row_that_moved_fails_instead_of_reporting_success` (db.rs) covers step 3 for all three sample editors — standard, computed and core — each with a clean edit beside it as the control. The stale depth is half a sample off, which is what a re-run on a shifted grid actually leaves behind, and the refusal has to name the depth and say what to do. `aux_data_can_be_browsed_but_no_editor_will_write_to_it` covers step 4 by exhaustion: every column aux_data exposes is rejected by every editor that exists, including `value_num`, which is the one a reader would assume is editable. Steps 1 and 2 are frontend behaviour (the `abc` message, the empty-to-NULL commit) and are still yours.

   **Worth knowing before you click:** aux data being read-only is a data-integrity rule, not a missing feature — a point sample is what a laboratory reported, so correcting it means re-importing the delivery, which keeps the set model and the provenance intact. Expect the hint rather than an editable cell.

   **KNOWN ISSUE (2026-07-31, finding 20) — OPEN, your call.** The FOURTH editor, behind the Wells grid, has no 0-row check: `update_well_field` validates the column and then updates without checking anything matched, so editing a well that has since been deleted elsewhere reports success and writes nothing. Rarer than step 3's case, because a well_id does not drift the way a depth does, but the same silent outcome — cell updated, status bar happy, undo entry pushed for a change that never happened.
   **Result — T-REP-16:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-17 — SQL Query: starter query + provenance join computed_curves ↔ log_sets

**Tool/panel:** SQL Query pane (src/ui/sqlQueryPanel.ts → src-tauri/src/db.rs run_readonly_query)
**Preconditions:** ≥2 wells with computed curves from module runs (so log_sets has rows).
**Steps:**

1. Data tab → **SQL Query**. Run the pre-filled starter query with **Run (Ctrl+Enter)** (also test the Ctrl+Enter shortcut itself).
2. Replace with:
   `SELECT w.well_name, cc.curve_name, ls.set_name, ls.version, ls.module, COUNT(*) AS samples FROM computed_curves cc JOIN wells w USING (well_id) JOIN log_sets ls ON ls.well_id = cc.well_id GROUP BY ALL ORDER BY w.well_name, cc.curve_name, ls.version`
   → Run.
3. Run `SELECT * FROM standard_curves` on a big well.
   **Expected:** (1) One row per well: sample counts > 0, avg*gr plausible (~20–150 gAPI for Mahakam sand/shale), top < bottom matching the wells' logged intervals. (2) The join executes — every computed curve (VSH, PHIE, SWE, FLAG*\*, …) appears against the well's log-set provenance rows (set_name/version/module tell you what run produced curves for that well); counts consistent with the Curve Catalog. (3) Result silently caps at the display limit ("1000 row(s)") rather than freezing the UI.
   **Automated coverage - end-to-end (pile C, 2026-08-01):** `panels.e2e.mjs` opens the pane,
   requires it to arrive with a runnable starter (an empty box would mean guessing at schema names)
   and RUNS it, asserting rows come back. **That is how finding 23 was caught: the starter shipped
   opening with two `--` comment lines, and `db::run_readonly_query` tests the first keyword of the
   trimmed text - so the very first thing a new user clicked in this panel was refused with "only
   SELECT queries are allowed here", about a query that is a SELECT.** The starter is fixed here.
   The guard still cannot see past a comment; that behaviour is pinned as-is and is your call. **Not
   covered:** the provenance join in step 2, and the "avg_gr plausible" human read.

   **Result — T-REP-17:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-18 — SQL Query rejects writes (negative)

**Tool/panel:** SQL Query pane (src-tauri/src/db.rs run_readonly_query)
**Preconditions:** SQL pane open; note a tops row count first (`SELECT COUNT(*) FROM tops`).
**Steps:**

1. Run `DELETE FROM tops`.
2. Run `UPDATE wells SET kb = 0`.
3. Run `INSERT INTO tops SELECT * FROM tops` and `DROP TABLE wells`.
4. Run the multi-statement smuggle `SELECT 1; DELETE FROM tops`.
5. Run the CTE smuggle `WITH x AS (SELECT 1) DELETE FROM tops`.
6. Re-run `SELECT COUNT(*) FROM tops`.
   **Expected:** (1)–(3) rejected before execution with "only SELECT queries are allowed here" in the results area; (4) rejected with "one statement at a time"; (5) starts with WITH so it passes the prefix check but must still fail as a SQL error from the read-only subquery wrapper — under no circumstance may it delete. (6) The tops count is unchanged from the start — zero rows were harmed. App stays responsive throughout; the status bar logs "Query failed: …" for each rejection.
   **Automated coverage - pinned (pile B, 2026-07-31):** `readonly_query_refuses_every_write_shape_including_a_cte_prefix` (db.rs).

   **Result — T-REP-18:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-REP-19 — Cross-check: theme switch repaints reporting & DB panes

**Tool/panel:** All four panes × theme system (Project tab ▸ Theme select)
**Preconditions:** Composite preview rendered, Report preview rendered, DB Inspector grid and SQL results on screen.
**Steps:**

1. Project tab → **Theme** → **Dark**. Inspect all four panes without re-rendering anything.
2. Switch to **Pertamina**, then back to **Default**.
3. In Dark, re-export a composite PDF and reopen it.
   **Expected:** (1) Form labels, grids, status text, and pane chrome repaint live in every pane; the composite/report preview surface must NOT stay light grey in dark themes (page paper stays white by design — it is a print preview — but the surface around it follows the theme); DB grid rows/NULL styling remain readable. (2) Accent colors follow the client palette; no pane needs reopening. (3) The exported PDF is theme-independent: identical black-on-white print output with the same "Made in SandiBumi" branding regardless of UI theme. Covers REVIEW.md §Wave A-1: tool panes + theme compliance, unchecked theme-check item "…the composite preview surface is no longer light grey in dark themes".
   **Result — T-REP-19:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

# Section AUX — Cross-cutting & auxiliary features

All source files verified. Writing the test plan now.

# UAT Cluster AUX — Cross-Cutting Gaps (second-pass coverage)

This cluster sweeps the tools and behaviours the first drafting pass missed: the new Wave-B features (well-diagram track, fluid contacts, highlights), the monitoring/help chrome (Performance pane, Help tool, Processing live detail), deep state mechanics (undo/redo walk, multi-instance views, pinned-well scope, Workflow Grid), and the negative paths for equations, importers, and i18n. Every step uses the labels actually in the source (index.html, ribbon.ts, the ui/\*.ts panels); REVIEW.md unchecked "[ ]" click-through items are folded in and cited per test. Note one naming trap up front: REVIEW.md calls the monitor "Health", but the shipped ribbon label is **Performance**.

---

### T-AUX-01 — Performance monitor pane opens and updates live

**Tool/panel:** Performance monitor (`src/ui/healthPanel.ts`, `src-tauri/src/health.rs`, ribbon button `#health-btn` in index.html)
**Preconditions:** Any project open; app running via `npm run tauri dev` on Windows.
**Steps:**

1. On the **Petrophysics** tab, in the **Batch** group, click **Performance** (tooltip: "Performance monitor — CPU, memory, and USER/GDI handle gauges").
2. Confirm the pane docks in the sidebar column and shows four gauge rows labelled exactly **CPU**, **MEM System**, **USER Objects**, **GDI Objects**, plus the note "Green < 60% · Yellow 60–80% · Red > 80%".
3. Watch for ~10 s: values should tick (poll interval 1.5 s) — CPU % should visibly move when you drag-pan a busy log view.
4. Hover each label — tooltips describe the metric (e.g. "This process's USER handles vs the 10,000 per-process ceiling"). USER/GDI show the raw handle count in parentheses after the %.
5. Open several heavy panes (2 log views, Crossplot, Histogram) and watch **GDI Objects / USER Objects** climb; close them and confirm the counts stop climbing (a monotonic climb that never stops = leak — report it).
6. If any gauge reads **n/a**, note which one (metrics are Windows-only, best-effort).
7. Right-click empty space in the pane — the context menu heading must read **Performance**, and the pane must not offer Close (it is an anchor pane).
   **Expected:** Four live colour-coded gauges updating every ~1.5 s without flicker; n/a only for genuinely unavailable metrics; pane survives (cannot be closed) and keeps its width when other panes open/close. _(REVIEW.md ▸ "Hardware Health Monitor" — unchecked "[ ] Test: open Health → MEM/USER/GDI show live %; leave a few heavy panels open and watch GDI/USER climb". Note: REVIEW describes a GPU Memory gauge; the shipped panel replaced it with CPU — record what you actually see.)_
   **Automated coverage - end-to-end (pile C, 2026-08-01):** `panels.e2e.mjs` opens the pane and
   asserts every gauge is both LABELLED and shows a value. A gauge rendering an empty string is
   worse than a missing gauge - it reads as "measured, and it is nothing" rather than "not
   measured". **Not covered:** the 1.5 s live tick, the tooltips, and the leak watch, which is a
   human read over time.

   **Result — T-AUX-01:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-02 — Help tool: ribbon button and right-click context help

**Tool/panel:** Help tool (`src/ui/workspace.ts` `openHelpForPanel`, `#help-btn` in index.html)
**Preconditions:** Any project open.
**Steps:**

1. With no panel focused (fresh workspace, click the status bar), click **Project ▸ Help ▸ Help** (tooltip "Help — a guide for the active panel"). Status bar should read "Click a panel first, then press Help (?) for its guide." if nothing is active.
2. Open **Petrophysics ▸ VSH ▾ ▸ VSH from Gamma Ray**, click inside the pane, then click **Project ▸ Help ▸ Help**. A modal titled **Help — VSH from Gamma Ray** opens showing the module's method description (the VSH_GR = (GR − GR_MA)/(GR_SH − GR_MA) text) — as a petrophysicist, confirm the description states the actual equation and the Stieber/Larionov/Clavier options.
3. Close it. Right-click empty space in the **Processing** panel → pick **Help for this panel…** from the context menu. A "Help — Processing" guide opens.
4. Repeat the right-click help on a Log View and on the DB Inspector (its blurb must say "spreadsheet-style", not any vendor name).
5. Confirm each help modal ends with the note "Illustrated help for each panel will open here in a later release."
   **Expected:** The Help button and the right-click **Help for this panel…** entry open the same contextual guide; module panes show the method doc, other panels a short blurb; no vendor trademarks appear. _(REVIEW.md ▸ "Help (?) tool" — unchecked "[ ] click Help (Project ▸ Help) (or right-click any panel → Help for this panel…)"; and ▸ trademark scrub "hover the DB Inspector ribbon button + open Help → reads 'spreadsheet-style'".)_
   **Automated coverage - end-to-end (pile C, 2026-08-01):** `panels.e2e.mjs` opens contextual
   help for the active panel and asserts it carries real text, then checks the provenance rule where
   a user actually reads it: the help must name no vendor (Schlumberger, Halliburton, Techlog,
   Geolog, Interactive Petrophysics). Attribution belongs in comments and in
   `docs/IP_PROVENANCE.md` beside the asset it describes, never in shipped help text. **Not
   covered:** the right-click context-help route, and the per-module help body.

   **Result — T-AUX-02:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-03 — Well Header dialog: TD/KB prefill, surface X/Y survival, edit one field

**Tool/panel:** Well Header dialog (`src/ui/ribbon.ts` `handleWellHeader`, Data ▸ Tools ▾)
**Preconditions:** A well with KB set (e.g. from a prior header save) AND surface coordinates imported via **Import Well Locations…** (run T-AUX-19 first if none).
**Steps:**

1. Select the located well in the Wells pane. Open **Data ▸ Tools ▾ ▸ Well Header…**.
2. Verify the modal **Well Header — {well}** prefills: **Field**, **TD (m)**, **KB (m)** (hint "datum for TVDSS"), **Surface X** (hint "UTM easting"), **Surface Y**, **UTM zone** (e.g. 50S). None of TD/KB may open blank on a well that has them.
3. Change ONLY **TD (m)** (e.g. +1 m), click **Save Header**.
4. Status must read "Updated header for {well}." — reopen the dialog: TD shows the new value AND Surface X/Y/zone are unchanged (the stale-snapshot coordinate wipe was the confirmed bug).
5. Open the **Processing History** pane (**Project ▸ Monitor ▸ History**): a new **Edit** entry "Updated well header" attributed to this well.
6. Restore the original TD.
   **Expected:** TD/KB always prefilled; a partial edit never wipes coordinates; History records the edit; Field Map marker does not move. _(REVIEW.md ▸ "[ ] Well Header shows current TD / KB … the field shows it, not an empty box" and ▸ Field Map "[ ] Tools ▸ Well Header on a located well → Surface X/Y/zone show the imported values (not blank); change only TD and Save → the coordinates survive".)_
   **Result — T-AUX-03:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-04 — Highlight tool: create bands, pan/zoom tracking, persistence

**Tool/panel:** Highlights overlay (`src/ui/highlightsOverlay.ts`, 🖍 button in `src/ui/logViewPanel.ts` toolbar)
**Preconditions:** Project with ≥2 wells carrying GR/RHOB curves; a log view layout saved.
**Steps:**

1. Open a **Log View** (Plot ▸ New Log View) on well A. In the view's own mini toolbar click **🖍** (tooltip "Highlight intervals: drag to paint a colored band…"). Status: "Highlight editing ON — drag paints a band, double-click edits/converts".
2. Confirm turning 🖍 on turned **🏷** (tops editing) off — click 🏷: 🖍 must deactivate, and vice versa. Leave 🖍 on.
3. Drag vertically over a known pay interval → a translucent colored band appears across ALL tracks and the **Edit highlight** dialog opens. Enter label "Pay", pick a color, click **Save**.
4. Add two more bands with different colors (e.g. "Coal", "Washout").
5. Pan and zoom the view — bands must track depth exactly (band edges stay glued to their depths), and curves must stay readable through the translucency. Tops lines must draw ON TOP of bands.
6. Switch to well B in the Wells pane, then back to well A — all three bands reload at their depths.
   **Expected:** Bands render across all tracks, track pan/zoom, persist per well across well switches (they live in the `highlights` DB table). As a domain check: band depths should match the intervals you dragged to ±1 screen pixel of the depth axis. _(REVIEW.md ▸ "Highlight tool — colored depth bands in the Log View", sub-items (a)–(c), (g), unchecked.)_
   **Result — T-AUX-04:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-05 — Highlight edit / delete / Convert to zone / undo chain

**Tool/panel:** Highlights overlay + Zones (`src/ui/highlightsOverlay.ts`, `src/ui/zonesDialog.ts`)
**Preconditions:** T-AUX-04 passed (3 bands exist on well A).
**Steps:**

1. With 🖍 active, **double-click** the "Pay" band → the **Edit highlight** dialog opens with **Save**, **Convert to zone**, **Delete** buttons.
2. Change the color and the top depth by a few metres → **Save** → band updates live.
3. Press **Ctrl+Z** → edit reverts (status shows the undo label). Press **Ctrl+Y** → re-applies.
4. Double-click "Pay" again → **Convert to zone**. Open **Petrophysics ▸ Zones…** — a zone matching the band's name and top/bottom depths now exists. Domain check: run **Cutoffs & Summary…** for the well and confirm the new zone appears as a summation interval.
5. **Ctrl+Z** → the zone conversion is undone (zone disappears from Zones; band remains).
6. Double-click "Washout" → **Delete**. Band vanishes; the **Processing History** pane shows a **Highlights** entry ("Deleted highlight {top}–{bottom}"). **Ctrl+Z** restores it.
7. Enter equal top and bottom depths in the dialog and Save → status "Highlight needs two different numeric depths", nothing written (negative check).
   **Expected:** Every add/edit/delete/convert is undoable and visible immediately; Convert to zone feeds the real zones table (pay summary sees it); degenerate depths are rejected. _(REVIEW.md ▸ Highlight tool sub-items (d)–(f), unchecked.)_
   **Result — T-AUX-05:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-06 — Well-diagram track: COMPLETION/PERFORATION import + log-view rendering

**Tool/panel:** Import Aux (`src/ui/ribbon.ts` `handleImportAux`) + Layout Properties (`src/ui/layoutPropsDialog.ts`) + log view diagram (`src/ui/logViewPanel.ts`)
**Preconditions:** A well with curves and a known completion (casing shoes + perf intervals from the completion report). Prepare two CSVs:
`completion.csv` → `TOP,BASE,OD` / `0,1500,13.375` / `0,2600,9.625` (use your well's real strings)
`perforation.csv` → `TOP,BASE,STATUS` / `2410,2418,OPEN` / `2440,2452,OPEN`
**Steps:**

1. Select the well. **Data ▸ Import Data ▾ ▸ Import Aux…** — in the **Dataset** dropdown note the choices are PETROGRAPHY / XRD / PERFORATION / **Custom…**. Pick **Custom…**, type `COMPLETION` in **Custom name**, click **Choose file & import…**, pick `completion.csv`.
2. Repeat with Dataset = **PERFORATION** and `perforation.csv`.
3. Open a Log View on the well, click **⚙** (Layout properties…). In **Layout Properties**, click the **＋** icon ("Insert track after the selected one") to add a track, set its **Track type** dropdown from **Curves** to **Well diagram**. The dialog must show the note "Draws casing / shoe / tubing / perforations from the well's COMPLETION and PERFORATION datasets (Data ▸ Import aux data). No curves needed." and the Curves table must disappear for this track. Apply/close.
4. In the log view: the new track draws two nested string pairs (13⅜" wider than 9⅝" — width scales with OD), small filled **shoe squares** at each casing base depth, the OD label at the string top, and perforation ticks across 2410–2418 / 2440–2452. No curves may draw underneath the diagram.
5. Domain acceptance: shoe depths and perf intervals must match the completion report depths on the depth axis; the deeper string must be the narrower one.
6. Pan/zoom — the diagram tracks depth like any curve track.
   **Expected:** Both aux imports succeed (History shows two Import entries; re-import replaces, not duplicates); the well-diagram track renders casing/shoe/perfs at the correct depths in the live view. _(REVIEW.md ▸ "Round 3 … (16) Well-diagram track", unchecked: "Try: import a COMPLETION CSV, add a track, set it to Well diagram.")_
   **Result — T-AUX-06:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-07 — Well-diagram track in Composite/Report output + old-layout compatibility

**Tool/panel:** Composite (`src/ui/compositeDialog.ts`, `src-tauri/src/composite.rs`) + Report (`src/ui/reportDialog.ts`)
**Preconditions:** T-AUX-06 passed; the layout with the Well diagram track saved (**Plot ▸ Save Layout…**).
**Steps:**

1. **Plot ▸ Deliverables ▸ Composite…** — pick the diagram layout and a **Print scale** (1:500), click **Render**. Page through with **◀ / ▶**: the diagram track appears on every page with casing lines, shoes and perf ticks at true depths.
2. **Save PDF…** and open the file: vector casing/shoe/perf artwork present, OD label (e.g. `9.625"`) at the string top on the page where the string starts.
3. **Plot ▸ Deliverables ▸ Report…** — set a **Study title**, pick the same layout, **Render**, then **Save PDF…**. The composite pages inside the report carry the diagram track too.
4. Compatibility: load an OLD saved layout (one created before this feature) via the **Plot ▸ Layout** selector — it must open normally, every track behaving as a Curves track (kind defaults to "curves"), no errors.
   **Expected:** Diagram renders identically in live view, composite SVG/PDF and report PDF; legacy layouts unaffected. _(REVIEW.md ▸ "(16) Well-diagram track … Renders in the log view and the composite/report SVG. Old saved layouts still load (kind defaults to 'curves')", unchecked.)_

   **Automated coverage - pinned (pile B, 2026-07-31):** three tests in composite.rs. `a_well_diagram_draws_its_strings_shoes_and_perforations_at_the_declared_depths` checks the artwork against the completion report's own depths — each string draws a symmetric pair of walls spanning exactly its top to its shoe, the wider OD draws wider, shoe markers land at each base, the OD label sits at the string top, and every perf tick falls inside the perforated interval. `a_well_diagram_track_is_redrawn_on_every_composite_page` is the joint — the diagram is not a header block, so a string running the length of the well must be redrawn per page. `a_layout_saved_before_well_diagram_tracks_opens_as_curves` covers step 4 by deserializing a layout written the way a pre-feature one was stored (no `kind`, no `points`/`arrays`/`images` keys at all).

   **Worth knowing before you click:** the diagram track draws from the **COMPLETION** and **PERFORATION** point datasets, and like every other point reader it follows the ACTIVE delivery set — so if a well shows an empty diagram, check Data Sets before suspecting the track. The OD label prefers the row's text value and falls back to the number with an inch mark, so a string stored as 9.625 prints `9.625"`. Step 4 matters more than it looks: an old layout that fails to load takes the user's track widths, scales, colours and curve choices with it, so check the tracks look right, not just that it opened.
   **Result — T-AUX-07:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-08 — Fluid-contacts editor: add/edit/delete at well, field and global scope

**Tool/panel:** Correlation pane ▸ Contacts… (`src/ui/correlationPanel.ts`)
**Preconditions:** ≥3 wells with GR loaded, at least two sharing a **Field** name in Well Header (set via T-AUX-03 if needed); Correlation pane opens with those wells.
**Steps:**

1. **Plot ▸ Multi-Well ▸ Correlation**. In the pane's properties row click **Contacts…** (tooltip "Add / edit fluid contacts (OWC, GWC, …)").
2. The editor shows "No fluid contacts yet — add one below." Click **＋ Add contact**.
3. On the new row: set type via the dropdown (choices exactly **OWC, GWC, GOC, GDT, ODT, FWL**) → pick **OWC**; type a depth (your field's known OWC); leave **TVDSS** unchecked; scope dropdown → verify it offers **All wells**, **Field: {name}** for each field, and **Well: {name}** for each well. Pick **All wells**. Pick a color.
4. Close the editor — a horizontal contact line at that depth crosses EVERY strip, with dashed cross-well connectors and a small triangle at the left edge.
5. Reopen **Contacts…**, change scope to **Well: {well A}** — the line now draws only on well A's strip. Change to **Field: {field}** — only that field's wells.
6. Add a second contact (GWC, different color). Untick **Show contacts in the view** — all contact lines vanish; retick — they return.
7. Delete the GWC row with **✕** — line disappears immediately.
8. Restart the app (`npm run tauri dev` again) and reopen Correlation — the OWC contact persists (DB-backed).
   **Expected:** Contacts CRUD is immediate and persistent; scope controls which strips draw the line; colors honored. Domain check: the OWC must plot at the same measured depth on a vertical well's strip as the log feature (resistivity drop) you know it corresponds to. _(REVIEW.md ▸ "Round 3 … (9) Fluid contacts in Correlation", unchecked.)_
   **Result — T-AUX-08:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-09 — Contacts MD ↔ TVDSS flattening across a deviated well

**Tool/panel:** Correlation depth-mode selector (`src/ui/correlationPanel.ts` `depthModeSel`)
**Preconditions:** T-AUX-08 passed. Correlation showing one near-vertical well AND one deviated well. The deviated well needs TVDSS: note whether its TVDSS exists as a **curve** (check Curve Catalog for mnemonic TVDSS) or only via **Data ▸ Import Data ▾ ▸ Import Deviation…**.
**Steps:**

1. In the Correlation toolbar find the depth-axis dropdown (options exactly **MD** / **TVDSS**; tooltip "Depth axis — measured depth, or TVDSS (fluid contacts are flat in TVDSS)").
2. In **Contacts…** add an OWC, tick its **TVDSS** checkbox, depth = a TVDSS value inside both wells' log intervals, scope **All wells**.
3. In **MD** mode: on the deviated well the contact must draw DEEPER in MD than its TVDSS value (converted per well); on the vertical well approximately at the same number.
4. Switch the dropdown to **TVDSS**: the contact line must now be perfectly flat across both strips at its stored depth, and the deviated well's log strip must visibly re-map (features shift up relative to MD mode).
5. Add a second contact with **TVDSS** unchecked (an MD contact). It must be flat in **MD** mode and break flat (differ per well) in **TVDSS** mode.
6. Domain acceptance: on the deviated well, MD-vs-TVDSS displacement of the contact should be consistent with the well's deviation (MD deeper than TVD by roughly the cumulative 1/cos(inc) effect; sanity-check against the deviation listing).
   **Expected:** TVDSS-stored contacts flatten in TVDSS mode via each well's TVDSS curve; MD contacts flatten only in MD mode; the switch needs no refetch (instant). _(REVIEW.md ▸ "(9) … Try: open Correlation, add an OWC as TVDSS, switch MD↔TVDSS, watch it flatten", unchecked.)_
   **Known issue:** AUDIT-2026-07-21-full-qc.md (Importers B §1, CONFIRMED): "Deviation-survey TVD/TVDSS is computed and stored, but no code path ever exposes it as a fetchable curve". The Correlation pane builds its MD→TVDSS lookup from a curve literally named TVDSS (`names = [opts.curve, "TVDSS"]`); a deviated well whose TVDSS exists ONLY from Import Deviation "falls back to MD == TVDSS" (treated as vertical) — step 4's strip re-map will silently NOT happen unless the well's LAS/DLIS delivery itself carried a TVDSS curve. If step 4 shows no shift on the deviated well, record Fail and note whether Curve Catalog lists a TVDSS curve.
   **Result — T-AUX-09:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-10 — Processing panel live detail: per-well progress, step-boundary text, Cancel

**Tool/panel:** Processing panel (`src/ui/processingPanel.ts`, `src-tauri/src/jobs.rs`) + Workflow Builder (`src/ui/workflowDialog.ts`)
**Preconditions:** ≥20 wells with GR/RHOB/NPHI/RT; a 3-step chain buildable (vsh → phi → sw).
**Steps:**

1. **Petrophysics ▸ Workflow…** → add steps via **Add module** + **+ Add step**: _VSH from Gamma Ray_, a porosity module, a saturation module. Set scope to **All**.
2. Click **Run chain**. The builder's own status line reads "Starting… (progress in the Processing panel)" and the **Processing** panel auto-opens (the builder must NOT show its own progress bar).
3. In the Processing panel watch the running job card: a progress bar with an integrated **Cancel** button on the same row, a current line of the form "Step 1/3: vsh_gr · {done}/{total}", and a counts row of chips (▶ running · ✓ done · ⚠ · ✗ · ⏳ pending).
4. Click **▸ details** — it flips to **▾ details** and lists the currently-running wells individually (notable wells only, not all N).
5. At each step boundary watch the current line: during the batched DB write it must read **"Writing N well(s)…"** instead of freezing at 100%.
6. Let step 2 begin, then click **Cancel** → button text becomes "Cancelling…" and the run stops within a well or two; the job card phase shows **Cancelled**.
7. Cross-check: any log view displaying VSH refreshes to show the wells that DID complete before cancel (dataVersion bumps on cancel), and the History pane records the chain run.
   **Expected:** Live per-well progress, honest step-boundary status, working shared Cancel, no second progress bar in the builder, stale-curve-free plots after cancel. _(REVIEW.md ▸ "[ ] Universal Processing panel — live per-well progress + Cancel", "[ ] Processing panel: the step-boundary 'pause' now says what it's doing", "[ ] Workflow Builder no longer shows its own redundant progress bar", and Round 3 "[ ] dataVersion refresh … on workflow-chain cancel/fail" — all unchecked.)_
   **Result — T-AUX-10:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-11 — Processing panel bulk same-failure summary + module-form one-liner

**Tool/panel:** Processing panel details (`src/ui/processingPanel.ts` family cards) + module pane (`src/ui/moduleDialog.ts`)
**Preconditions:** ≥15 wells where most LACK a given curve (e.g. only 2 wells have NPHI).
**Steps:**

1. Open **Petrophysics ▸ Porosity ▾** and pick a density-neutron module that requires NPHI. Scope **All**. Click **Run**.
2. The module form itself must show only ONE summary line ("All N well(s) computed…" or "…N need attention — see the Processing panel."), never a per-well ✓ list, and the Processing panel comes forward.
3. In Processing → **▾ details**: because many wells failed the SAME way, verify ONE card per failure reason in the form "**{N} well(s) failed — {message}**", listing the first 12 well names then "…(+K more)", plus an advice line starting "**→**".
4. The 2 wells with NPHI show ✓ (counts row: ✓ 2 · ✗ N−2).
5. Click a failed well's row/card — the message text is readable and names the missing input (domain check: the message should let you diagnose "no NPHI" without opening the well).
6. Negative-honesty check: pick one well with NPHI present but wholly NaN over the interval (or temporarily blank it via curve edit) and re-run scoped to that well — the run must report an error/Warned, NOT a green success. Undo any curve edit afterwards (Ctrl+Z).
   **Expected:** Compact per-reason failure cards at scale; per-well results only in Processing; all-MISSING output reported as failure. _(REVIEW.md ▸ "[ ] Bulk failure report", "[ ] Module form no longer lists per-well results", "[ ] Per-well detail lives in Processing → details", and Round 3 "[ ] All-NaN module runs report honestly" — all unchecked.)_
   **Result — T-AUX-11:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-12 — Workflow Grid inspector: Set-all RW / Mask, amber overrides, saved-JSON stability

**Tool/panel:** Workflow Builder List|Grid (`src/ui/workflowDialog.ts`)
**Preconditions:** The 3-step chain from T-AUX-10 (or rebuild it: vsh*gr → phi module → two sw*\* steps to make RW shared).
**Steps:**

1. In **Workflow…**, above the step list click the **Grid** toggle (tooltip "All steps' parameters in one editable grid — the Set-all row edits a shared parameter across every step").
2. Verify the grid: rows = your steps (corner header **Step**), columns ordered input curves → numeric params → options → Mask; a step that doesn't take a column shows "**—**"; hover a column header → its tooltip is the parameter description.
3. In the italic **Set all** row (placeholder "(set all)"), type an RW appropriate to your field (e.g. fresh Mahakam formation water, RW ≈ 0.2–0.4 ohm·m at FT) in the **RW** column. Every sw\_\* step's RW cell updates at once; the status bar reports how many steps took it. Deliberately enter an out-of-range value (e.g. −1): modules whose allowed range excludes it are skipped and reported, others untouched.
4. Confirm edited cells tint amber and each edited step's badge counts up ("· N overrides"); retype a cell back to the module's manifest default → the tint clears and the badge decrements ("· defaults" when zero).
5. **Set all → Mask** column: pick BADHOLE (or your flag curve) — every step's Mask sets in one edit.
6. Toggle **List** ↔ **Grid** repeatedly: values, badges and invalid-input flagging identical in both; reopen the Workflow pane — the last view choice is remembered.
7. Type a name in the **workflow name** box, click **Save**. Click **Load** on it: the grid reloads with IDENTICAL values, tints and badges (saved-JSON shape unchanged). **Run chain** on one pinned well to prove the loaded chain executes.
   **Expected:** One-edit fan-out with range-checked skips, amber only-store-differences override accounting, view parity, and byte-stable save/reload. _(REVIEW.md ▸ "Wave A-4: workflow grid inspector" — all five sub-items unchecked.)_
   **Result — T-AUX-12:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-13 — Undo/redo depth: full walk down and up, redo invalidation

**Tool/panel:** Global undo stack (`src/undo.ts`, `#undo-btn`/`#redo-btn`) across tops editor, highlights, DB Inspector, curve edit
**Preconditions:** One well with tops, curves, and a log view open; DB Inspector openable.
**Steps:**

1. Perform 6 mixed undoable edits, noting each expected state: (a) 🏷 tops mode: click to ADD a top; (b) drag an existing top to MOVE it ~5 m; (c) 🖍: add a highlight band; (d) **Data ▸ DB Inspector**: double-click a curve-sample value cell, change it, commit; (e) right-click a curve in a log view track → **Edit {curve}…** → op **Set constant** over a 2 m interval; (f) rename a track title in **Layout Properties**.
2. Hover **Project ▸ Edit ▸ Undo** — it is enabled. Press **Ctrl+Z** six times, ONE at a time; after each press verify the status-bar label names the correct action (reverse order f→a) and the on-screen state matches (track title back, curve values restored, cell restored, band gone, top back, added top gone).
3. When the stack is empty the **Undo** button disables.
4. Press **Ctrl+Y** six times — each redo replays in order a→f; verify final state equals post-step-1 state exactly.
5. Undo twice (undoes f, e). Now make a NEW edit (add another highlight). Press **Ctrl+Y**: NOTHING may redo (redo stack cleared by the new edit; **Redo** disabled).
6. Cleanup: undo the remaining edits.
   **Expected:** A 6-deep mixed stack walks down and up losslessly with correct labels; a new edit after partial undo invalidates redo (`pushUndo` clears the redo branch); stack cap is 100 so nothing rolls off here.
   **Known issue:** AUDIT-2026-07-21-full-qc.md (Curve edit / undo §2, CONFIRMED): "restore_curve_values (the undo path) has no staleness/version check — an old edit's Ctrl+Z can silently overwrite a curve that's been legitimately recomputed since, and the frontend never checks how many samples actually got restored." If you interleave a module recompute of the edited curve between step 1(e) and the undo walk, the step-2 undo of (e) may splice stale pre-run values into the fresh curve and still report success — do NOT interleave recomputes in this test's main path; optionally probe it and note the result.
   **Result — T-AUX-13:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-14 — Two log views of the SAME well: independent viewports, shared refresh

**Tool/panel:** Log view multi-instance (`src/ui/logViewPanel.ts`) + Plot ribbon
**Preconditions:** One well with GR + a computed VSH; two saved layouts (or the default + one variant).
**Steps:**

1. **Plot ▸ New Log View** twice — two log-view tabs on the same selected well. Drag one tab beside the other so both are visible.
2. In view 1 set the depth scale dropdown to **1:200** and zoom into a reservoir; in view 2 set **1:2000** full-well. Verify the viewports are fully independent (panning one never moves the other).
3. Focus view 2, pick a different layout in **Plot ▸ Layout** — only view 2's tracks change.
4. In view 1 toggle **▤** track headers to compact — view 2's headers unchanged.
5. Run **Petrophysics ▸ VSH ▾ ▸ VSH from Gamma Ray** on this well with a changed GR_SH (e.g. 110 gapi). When the run completes BOTH views must repaint the VSH track with the new values (no stale curve in either).
6. Theme cross-check: **Project ▸ Theme ▸ Dark** (then back) — both canvases repaint to the new palette immediately, bands/tops/core overlays included.
7. Domain acceptance: after step 5 the VSH curve in both views must be identical sample-for-sample (cursor readout at the same depth shows the same value in both).
   **Expected:** N independent viewports/layouts over one well; a single recompute refreshes every instance via dataVersion; theme repaint hits all canvases.
   **Result — T-AUX-14:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-15 — Pinned wells (★) as a batch-run scope

**Tool/panel:** Wells pane star (`src/ui/objectTree.ts`) + scope selector (`src/ui/wellScope.ts`) + module pane
**Preconditions:** ≥5 wells; no wells pinned yet.
**Steps:**

1. In the **Wells** pane click the **☆** star left of two well names → each turns **★** (tooltip on a pinned star: "Pinned — click to unpin. Pinned wells are reusable as a one-click run scope.").
2. Restart the app and confirm the two stars persist (per-project `well_pins`).
3. Open **Petrophysics ▸ VSH ▾ ▸ VSH from Gamma Ray**. The pane shows the compact scope selector with segments exactly **Group · ★ Pinned · Selection · All · Custom…** and a live "N wells" count. Select **★ Pinned** → count reads "2 wells".
4. Click **Run**. When done, open **Curve Catalog** (Data tab) for each of the two pinned wells → fresh VSH/VSH_GR rows with new timestamps/provenance; spot-check a THIRD (unpinned) well → its VSH is untouched (old timestamp or absent).
5. **Processing History**: the run's entry must reflect the wells actually run (a 2-well batch — not attributed to whichever well was globally selected).
6. Probe: with the module pane still open, pin a third well, then reselect **★ Pinned** / press Run — the run must include 3 wells (the scope "resolves against live state at run time").
7. Unpin all but your usual set.
   **Expected:** Stars persist; ★ Pinned scope resolves exactly the pinned set at run time; provenance and History confirm only in-scope wells were written. _(REVIEW.md ▸ "[ ] ★ pin a well in the Wells pane" and ▸ "Well scope — no more well-by-well checklists" with "Group · ★ Pinned · Selection · All · Custom…", and Round 3 "[ ] History attribution" — unchecked.)_
   **Result — T-AUX-15:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-16 — Equation negative: Python in-place output guard

**Tool/panel:** Equation Editor (Inspector pane, `src/ui/inspectorPanel.ts`; guard in `src-tauri/src/python_engine.rs`)
**Preconditions:** Python 3.10+ with numpy findable (the language note in the editor shows "(engine: …)" not the ⚠ warning); one test well selected.
**Steps:**

1. **Data ▸ Curve Catalog** → the Inspector opens; select the **Equation Editor** tab.
2. Create a new equation: **Name** `GR_INPLACE_TEST`, **Input curves** `GR`, **Output curve** `GR` (deliberate collision), **Language** Python (numpy). Script (the classic forgot-to-assign cleanup): `tmp = np.clip(gr, 0, 100)` — note it never assigns `gr`. Leave **Apply to all wells** unchecked. **Save**, then **Run**.
3. The run must FAIL with the guard message ("script never assigned the output curve 'gr' (it still equals the input 'gr')") — it must NOT report success, and Curve Catalog must show NO new GR row in set EQUATION.
4. Fix the script to `gr = np.clip(gr, 0, 100)` and **Run** again → succeeds (values genuinely change in shale where GR > 100). Curve Catalog: a computed GR appears in set **EQUATION** with provenance; the open log view's GR track refreshes (dataVersion bump after equation runs).
5. Regression probe: rename **Output curve** to `np`, Run → must error cleanly (reserved-namespace collision), not crash the worker.
6. Cleanup: delete the EQUATION-set curve (Curve Catalog / log-set delete) and Ctrl+Z anything else.
   **Expected:** A no-op in-place script is rejected loudly; a real in-place edit passes; `np`/`numpy` output names cannot crash the engine; success bumps dataVersion. _(REVIEW.md ▸ Round 3 "[ ] Python in-place equation guard … (Also fixed a worker crash when the output was named np/numpy.)" and "[ ] dataVersion refresh after equation … runs" — unchecked.)_
   **Result — T-AUX-16:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-17 — Equation negative: runtime error mid-batch with per-well isolation

**Tool/panel:** Equation Editor + Processing panel (`src-tauri/src/equations.rs` / `lib.rs` `run_equation` job)
**Preconditions:** ≥3 wells where at least one LACKS NPHI and the others have it; Python engine live.
**Steps:**

1. In the **Equation Editor** create: **Name** `PHIN_TEST`, **Input curves** `NPHI`, **Output curve** `PHIN_TEST`, Language Python. Script:
   `if np.all(np.isnan(nphi)): raise ValueError("NPHI missing in this well")`
   `phin_test = np.clip(nphi, 0, 0.6)`
2. Tick **Apply to all wells**. **Run**.
3. The **Processing** panel shows an **Equation** job ("equation: PHIN_TEST") with per-well items: wells with NPHI finish ✓; the NPHI-less well shows **✗** with the message "script error: NPHI missing in this well" (useful, names the cause).
4. Verify isolation: the failing well wrote NOTHING (no PHIN_TEST in its Curve Catalog), while every good well's PHIN_TEST exists — one well's crash must not abort or poison the batch.
5. Rhai-path variant: switch Language to Rhai (legacy), script `nphi * 1.0`, input `NPHI` → on the NPHI-less well the run must fail per-well with "equation produced no finite output — check the input/output curve name(s) resolve to data", not a clean success.
6. Cleanup: delete the PHIN_TEST curves.
   **Expected:** Runtime errors surface per well in the Processing job with actionable messages; healthy wells complete and write; all-NaN outputs are never written as success. _(REVIEW.md ▸ Round 3 "[ ] All-NaN module runs report honestly … Same guard on Rhai + Python equations (an unresolvable input/output curve → error, not a clean success)" — unchecked.)_

   **Automated coverage - pinned, with a residual (2026-07-31):** both language paths, because they are different functions - `lib.rs` dispatches on the equation's language, so a test on one says nothing about the other. `a_python_raise_in_one_well_leaves_the_rest_of_the_batch_intact` (`python_engine.rs`) runs your exact step-1 script and asserts steps 2 and 4: the raising well writes NOTHING (checked in the database, not just in the return value) and every healthy well completes, with your own "NPHI missing in this well" message reaching the run summary. It runs for real on this machine because numpy is here. `one_failing_well_does_not_poison_a_multi_well_equation_run` (`equations.rs`) does the same for step 5's Rhai path. NOT asserted: step 3's Processing panel actually rendering those per-well ✗ marks. That is still yours.

   **KNOWN ISSUE found while writing that test (2026-07-31) - it makes step 5 narrower than it looks:** the Rhai guard only fires when EVERY sample fails. A Rhai error is caught per sample and written as MISSING, so a script that raises on only some depths produces a curve with holes and reports a **clean success with the full row count** - and that is indistinguishable from a curve whose inputs were simply absent there. Pinned as-is by `a_script_that_raises_on_only_some_samples_still_reports_a_clean_success`, with a control that raises everywhere and IS caught. Logged in `ROADMAP.md` §B1 as **your call**, because counting the raises changes the run summary. The Python path does not have this - it runs the whole well's array at once, so a raise fails the well outright.
   **Result — T-AUX-17:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-18 — Importer negative: corrupt/unparsable DLIS

**Tool/panel:** Import DLIS (`src/ui/ribbon.ts` `handleImportDlis`, `src-tauri/src/dlis.rs`)
**Preconditions:** A selected well; a fake file made by renaming any small text file to `garbage.dlis`.
**Steps:**

1. Note the selected well's current curve count in **Curve Catalog** and the well count in the Wells pane.
2. **Data ▸ Import Logs ▾ ▸ Import DLIS…** (tooltip "Import every curve from a DLIS file into the selected well (via dlisio)"). Pick `garbage.dlis`.
3. Status shows "Importing DLIS into {well}… (dlisio may take a moment)" then must land on **"DLIS import failed: …"** with a real parser reason — the app must not hang or crash.
4. Verify NO side-effects: well count unchanged (DLIS imports into the selected well, so no orphan well may appear), the selected well's Curve Catalog row count unchanged, and **Processing History** has NO "Imported DLIS" entry (History records successes only for this path).
5. Sanity re-check the happy path is intact: import a known-good DLIS → "Imported N curve(s), M samples into {well}." plus a History **Import** entry, and if it replaced same-named DLIS curves the status appends "(replaced K existing curve(s))".
6. With no well selected, open Import DLIS → status "Select a well first (Wells & Tops panel)" and no file picker.
   **Expected:** A clean, specific failure message; zero partial writes, zero orphan wells, zero phantom History entries; guard when no well is selected. _(REVIEW.md ▸ "[ ] DLIS null sentinels + no silent overwrite" and import-robustness batch 2 "(3) DLIS import sanitizes each frame's depth … so one bad sample can't abort the file" — unchecked.)_
   **Result — T-AUX-18:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-19 — Importer negatives: locations CSV blank-WELL rows; SCAL fluid-system guard

**Tool/panel:** Import Well Locations + Import SCAL (`src/ui/ribbon.ts`, `src-tauri/src/ingest.rs`)
**Preconditions:** ≥3 wells (W-A, W-B by their real names). Prepare `loc_blank.csv`:
`WELL,EASTING,NORTHING` / `{W-A},512300,9945210` / `,513100,9946000` / `{W-B},514050,9947120`
**Steps:**

1. Note W-A/W-B and every OTHER well's Surface X/Y (Well Header) beforehand.
2. **Data ▸ Import Data ▾ ▸ Import Well Locations…** — leave **Default UTM zone** at **UTM 50S**, click **Choose file & import…**, pick `loc_blank.csv`.
3. Result line must read "Located 2 well(s) — unmatched: **1 blank-WELL row(s)**. Open Field Map to view." — the blank-cell row is SKIPPED and reported, and crucially NO other well's location changed (re-check the currently selected well's header: the blank row must not have been routed to it).
4. Open **Field Map…** — W-A and W-B post at the right relative geometry (domain check: ~1.2 km apart on the scale bar for the sample coordinates).
5. SCAL guard: **Data ▸ Import Data ▾ ▸ Import SCAL…**, multi-select two files. In the dialog confirm: the doc text warns "One import = ONE lab fluid system: don't mix air-brine and mercury deliveries in a single multi-select…"; the **Fluid system** dropdown offers Air-brine (72) / Air-mercury (367) / Oil-brine (26) / Other / custom, and switching presets rewrites **Lab sigma·cosθ (dyn/cm)** to 72/367/26.
6. Pick **Other / custom** → the σcosθ field CLEARS and focuses (no stale preset silently stored). Click **Import & Fit** with it empty → refused with "Lab sigma·cosθ must be a positive number." and nothing imported.
7. Cancel out (✕) without importing — the well's existing `scal_pc` data must be untouched (an import that parses zero points refuses the replace-write).
   **Expected:** Blank-WELL rows skip-and-report instead of misrouting; the mixed-fluid-system warning path (single system select + doc warning + cleared σcosθ on Other) blocks a biased pooled J-fit; no partial/empty replace-writes. _(REVIEW.md ▸ Field Map "[ ] Import a file that has a WELL column but a blank cell in one row → that row is skipped and surfaced as '1 blank-WELL row(s)'", and ▸ SCAL importers post-review hardening items (1) zero-point refuse, (5) Other clears σcosθ, plus "The dialog also now warns: ONE lab fluid system per import" — unchecked.)_
   **Result — T-AUX-19:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-AUX-20 — i18n depth: Bahasa Indonesia inside dialogs + PDF header policy

**Tool/panel:** Language system (`src/i18n.ts`, Project tab selector) + module dialog + Report dialog (`src-tauri/src/report.rs`)
**Preconditions:** Any project; a well renderable in the Report dialog.
**Steps:**

1. **Project ▸ Language ▸ Bahasa Indonesia** (option labels stay native-named by design). Ribbon spot-checks: Data-tab dropdowns become **Impor Log / Impor Data / Alat**; Plot ▸ **Report…** becomes **Laporan…**; status bar "Ready" → "Siap".
2. Open **Petrophysics ▸ VSH ▾ ▸ VSH from Gamma Ray**: the **Run** button reads **"Jalankan"**; generic words in the pane (Save→Simpan, Cancel→Batal, Close→Tutup where present) translate; parameter mnemonics and units (GR_MA, gapi, VSH) stay ENGLISH — jargon is deliberately untranslated.
3. Open **Plot ▸ Laporan… (Report)**: dictionary-covered generic words in the pane translate; specialist labels not yet in the dictionary (e.g. "Study title", "Render") may remain English — record exactly which stay English for the translation backlog, but their remaining English is not a Fail.
4. Still in Bahasa mode: set a Study title, **Render**, then **Save PDF…**. Open the PDF: the section headers must be the English deliverable set — cover page, **Methodology**, **Zone Parameters**, **Pay Summary (VSH ≤ …, PHIE ≥ …, SWE ≤ …)** — the generated report is deliberately locale-independent (headers are fixed in the Rust generator; only the live UI translates).
5. Switch **Project ▸ Language ▸ Basa Jawa**: Save→Simpen, Reload→"Muat manèh" (spot-check any dialog), then back to **English** → every label reverts exactly to source (originals are remembered, not re-translated).
6. Repaint check: while in a non-English locale open and close two more dialogs — no mixed-language flicker or stuck translations (the MutationObserver translates late-added DOM too).
   **Expected:** Dialog-level translation works beyond the ribbon for all dictionary-covered generic vocabulary; petrophysics jargon and curve mnemonics stay English by design; the PDF deliverable keeps English section headers; round-trip back to English is lossless. _(REVIEW.md ▸ "[ ] Bahasa Jawa (jv) added + fuller Bahasa Indonesia / Basa Sunda … petrophysics jargon still stays English by design … switch back to English → everything reverts from source" and ▸ "[ ] Bahasa Indonesia / Basa Sunda: the new labels translate (Impor Log / Impor Data / Alat)" — unchecked.)_
   **Result — T-AUX-20:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

# Section INT — End-to-end integration & performance

All source reading done — I verified every label against `index.html`, `ribbon.ts`, and the panel sources, re-checked both mandated audit findings against current code (the group-rescope gap is still present in `wellScope.ts`/all dialogs; the module-run UI-block was converted post-audit to `jobs::run_job` in `lib.rs:734`, noted honestly in the Known-issue line), and folded in the unchecked REVIEW.md items this cluster covers.

## Cluster INT/PERF — End-to-end integration + performance/stress

**Shared preconditions:** app launched with `npm run tauri dev` from `D:\XX. SandiBumi`. Two projects are used: a **fresh empty project** created in T-INT-01 (INT tests, 4+ real Mahakam LAS with GR, RHOB, NPHI, RES_DEEP, CALI, DRHO + a matching tops CSV), and the **big field project (100+/540 wells)** for all PERF tests. Keep the **Processing** pane and the **Processing History** panel (**Project ▸ Monitor ▸ History**) visible throughout — most cross-checks read them. INT tests build on each other in order; PERF tests are independent of INT. **For every PERF test, write the rough timing (seconds) in Notes — ROADMAP.md Performance tier #128–132 explicitly awaits these live measurements.**

### T-INT-01 — Fresh project + multi-LAS import (canonical workflow, step 1)

**Tool/panel:** Ribbon Project/Data tabs + Wells pane (`src/ui/ribbon.ts`, `src/ui/objectTree.ts`)
**Preconditions:** app running; 4+ real LAS 2.0 files on disk.
**Steps:**

1. **Project → New Project…**, save as `uat-int.duckdb`. Window title becomes "SandiBumi — uat-int"; Project group caption shows the name.
2. **Data → Import Logs ▾ → Import LAS…**, multi-select all 4+ LAS files, open.
3. Watch the status bar, then the Wells pane and History panel.
   **Expected:** status line "Importing N LAS file(s)..." then "Imported N/N well(s)."; all wells appear in the Wells pane; History shows an "Import" entry ("Imported N/N LAS well(s)") plus one entry per depth-warning well if any; **Data → Curve Catalog** for a selected well lists GR/RHOB/NPHI/RES_DEEP etc. with units and sample counts matching the LAS.
   **Result — T-INT-01:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-INT-02 — Duplicate LAS re-import warns (negative)

**Tool/panel:** LAS importer (`src/ui/ribbon.ts` handleImport; backend `import.rs`)
**Preconditions:** T-INT-01 done.
**Steps:**

1. **Data → Import Logs ▾ → Import LAS…** and re-import one of the SAME files from T-INT-01.
   **Expected:** the import completes but the status/History carries an "already exists" duplicate-name warning; a separate well record is created (merge is deliberate, not automatic) — you should see the warning, not a silent second copy. Covers REVIEW.md §Round 4 — "LAS duplicate-name warning". Delete the duplicate well afterwards to keep the project clean.
   **Automated coverage - pinned, with a residual (pile A):** that the duplicate warns and stays a separate record IS asserted. NOT asserted: the display surface - the status line and the History row. That part is still yours.

   **Result — T-INT-02:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-INT-03 — Tops import → zones from tops (+ empty-well negative)

**Tool/panel:** Import Tops + Zones pane (`src/ui/ribbon.ts`, `src/ui/zonesDialog.ts`)
**Preconditions:** T-INT-01; tops CSV with a WELL column covering the imported wells.
**Steps:**

1. **Data → Import Data ▾ → Import Tops…**, pick the CSV.
2. Select well 1 in the Wells pane; **Petrophysics → Zones…**; click **From Tops**.
3. Check the Zones table against the known formation tops.
4. Negative: select a well the CSV did not cover, open **Zones…**, click **From Tops**.
   **Expected:** (2) status "Built N zone(s) from tops for <well>"; zone Top/Bottom depths match the tops, zones are contiguous top-down; History gets a "Zone" entry per action. (4) "Built 0 zone(s)…" and the table shows `No zones — use "From Tops" or add one below.` — no crash, no phantom zones.
   **Automated coverage - pinned (pile B, 2026-07-31):** `zones_from_tops_are_contiguous_and_absent_tops_make_no_zones` plus `a_top_below_the_logged_interval_never_makes_an_inverted_zone` (db.rs).

   **Result — T-INT-03:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-INT-04 — Conditioning prep: gr_normalize → badhole → condflag → precalc

**Tool/panel:** Module panes from Petrophysics → Data Prep ▾ (`src/ui/moduleDialog.ts`, `src-tauri/src/modules.rs`)
**Preconditions:** T-INT-01/03 done.
**Steps:**

1. **Petrophysics → Data Prep ▾ → GR Normalization (Two-Point Percentile)**. In the pane: Wells scope = **All**; set GR_LOW_REF/GR_HIGH_REF to your field's P3/P97 reference (defaults are the generic 20/120 — replace with your own field values); Output cons = `INTERP`; **Run**.
2. **Data Prep ▾ → Bad-Hole QC Flag**: defaults (DRHO_MAX 0.05, DCAL_MAX 1.0); **Run**.
3. **Data Prep ▾ → Data Conditioning Flags**: defaults; **Run**.
4. **Data Prep ▾ → Pre-Calculation (P / T / Rmf / Ct / Cxo)**: defaults; **Run**.
   **Expected:** each Run: the Processing panel comes forward with a job (kind "Module", progress bar, Cancel), the pane shows one line "All N well(s) computed. Per-well details are in the Processing panel." (no per-well list in the form — covers REVIEW.md §Module-panel cleanup); Curve Catalog gains GRN, BADHOLE, COND_FLAG, precalc outputs with Module/Source and Set `INTERP v1`; History gets a "Module" entry per run. Petrophysics check: GRN preserves GR character but well-to-well histograms now overlay at P3/P97; BADHOLE = 1 exactly where DRHO > 0.05 g/cc or |CALI−BS| > 1 in.
   **Result — T-INT-04:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-INT-05 — Interpretation chain by hand: vsh → phi → sw → perm, each feeding the next

**Tool/panel:** Module panes (`src/ui/moduleDialog.ts`) + Log View (`src/ui/logViewPanel.ts`)
**Preconditions:** T-INT-04 done. Open **Plot → New Log View** on well 1 first, and via **Plot → Properties…** put VSH/PHIE/SWE on tracks so refreshes are visible.
**Steps:**

1. **Petrophysics → VSH ▾ → VSH from Gamma Ray**: GR input = **GRN** (not raw GR), GR_MA/GR_SH from your histogram picks, Mask (optional) = **BADHOLE**, Output cons `INTERP`; **Run**.
2. **Porosity ▾ → Porosity from Density**: RHOB = RHOB, VSH = **VSH** (from step 1); Mask = BADHOLE; **Run**.
3. **Saturation ▾ → SW — Indonesia (Poupon-Leveaux)**: RT = RES_DEEP, PHIE = **PHIE**, VSH = **VSH**; set RW/M/N for the zone; **Run**.
4. **Permeability ▾ → Permeability — Wyllie-Rose**: PHIE = **PHIE**; **Run**.
5. After each run, look at the open Log View.
   **Expected:** every step's input dropdown already offers the previous step's outputs (persistent-pane refresh); after each Run the open Log View repaints with the new curve (dataVersion) without reopening. Petrophysics: VSH ∈ [0,1], high in shales, low in clean sand; PHIE ≤ PHIT ≤ PHIE_MAX (0.30 default) and mirrors VSH inversely; SWE ∈ [0,1], ≈1 in wet/shale intervals, low in pay; PERM > 0 mD, orders-of-magnitude range tracking PHIE. Masked (BADHOLE=1) intervals are blank in ALL outputs. Curve Catalog: each output row shows module, `INTERP` version, timestamp; Constellations list grows one version per run.
   **Result — T-INT-05:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-INT-06 — Negative trio: missing curve, out-of-range param, empty scope

**Tool/panel:** Module panes (`src/ui/moduleDialog.ts`)
**Preconditions:** a well with NO computed PHIE exists (use the T-INT-02 duplicate before deleting it, or a freshly imported extra well).
**Steps:**

1. On that well only (scope = **Selection** with just it, or **Custom…**), open **SW — Indonesia (Poupon-Leveaux)** and **Run** with PHIE pointing at the absent curve.
2. In **VSH from Gamma Ray**, type GR_SH = `2000` (max 1000) and **Run**.
3. Set scope = **Selection** with nothing selected in the Wells pane and **Run**.
   **Expected:** (1) the run is reported as an error/⚠ Warned in the Processing panel — NOT a green all-wells-computed success (an all-NaN output must not pass as success; covers REVIEW.md §Round 4 — "All-NaN module runs report honestly"). (2) inline validation "GR_SH: value must be between 0 and 1000." and no run starts. (3) "No wells in scope — pick a group, pin/select wells, or choose All." and no run starts. No crash in any case.
   **Automated coverage - end-to-end (pile C, 2026-08-01):** legs 2 and 3 are
   `moduledialog.e2e.mjs`; leg 1 is already pinned in Rust by
   `all_nan_module_output_reports_error_not_success` (workflow.rs). Both frontend legs are
   frontend-ONLY: the backend computes happily with an out-of-range parameter and reports success on
   an empty well list, so a click handler is the only thing stopping either, and no Rust test can
   reach them. **The assertion that matters is that NO RUN STARTED** - a project-wide
   `computed_curves` fingerprint is compared before and after, because a dialog that prints a
   complaint and then runs anyway looks identical from the message alone.

   **Result — T-INT-06:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-INT-07 — Cutoffs & Pay Summary + History/flags cross-check

**Tool/panel:** Cutoffs & Summary pane (`src/ui/summaryDialog.ts`)
**Preconditions:** T-INT-05 done on all 4 wells.
**Steps:**

1. **Petrophysics → Cutoffs & Summary…**; scope = **All**; VSH ≤ 0.5, PHIE ≥ 0.1, SWE ≤ 0.6 (or your study cutoffs); click **Compute Summary**.
2. Inspect the table; then check History and the Curve Catalog.
   **Expected:** per-well per-zone rows with SAND/RESERVOIR/PAY flags; Net ≤ Gross, 0 ≤ N/G ≤ 1, Avg VSH/PHIE/SWE inside their cutoff-consistent ranges, HPV ≈ Net·PHIE·(1−SWE); empty aggregates show "—" not a crash. Status "Pay summary: N rows; FLAG curves written". History gains a "Pay Summary" entry with the cutoffs (covers REVIEW.md §Round 4 — "Pay Summary → Processing History"); FLAG_SAND/FLAG_RESERVOIR/FLAG_PAY appear in the Curve Catalog (PAYFLAG set) and any open Log View showing them refreshes.
   **Result — T-INT-07:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-INT-08 — Composite + Report PDF; report numbers must equal the on-screen pay summary

**Tool/panel:** Composite + Report dialogs (`src/ui/compositeDialog.ts`, `src/ui/reportDialog.ts`)
**Preconditions:** T-INT-07 just run; keep its table visible (or re-Compute with identical cutoffs).
**Steps:**

1. **Plot → Composite…** on well 1: Layout + Print scale 1:200; **Render**; page through with ◀/▶; **Save PDF…**.
2. **Plot → Report…**: fill Study title/Prepared by; set **Cutoffs VSH/PHIE/SWE/PERM** to EXACTLY the T-INT-07 values; **Render**; then **Save PDF…**.
3. Compare the report's pay-summary page numbers (Net, N/G, Avg PHIE/SWE, HPV per zone) against the Cutoffs & Summary table.
4. After the report run, glance at an open plot/log view.
   **Expected:** composite pages cover the full logged interval at true print scale with correct depth annotation; the report PDF has cover + methodology + zone parameters + pay summary + composite pages; **every pay-summary number in the PDF matches the on-screen table digit-for-digit** (same cutoffs, same wells). Open panels refresh after the report run and a History entry appears (covers REVIEW.md §Round 4 — "dataVersion refresh after … report runs").
   **Result — T-INT-08:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-INT-09 — Well-group scoping end-to-end (3-well group)

**Tool/panel:** Well Groups manager + Wells pane group bar + module pane scope (`src/ui/wellGroups.ts`, `src/ui/objectTree.ts`, `src/ui/wellScope.ts`)
**Preconditions:** ≥4 wells with GR imported.
**Steps:**

1. Wells pane → **⚙** ("Manage well groups…"); type `UAT-North` in "New group name…" → **Create**; tick exactly 3 wells → **Save membership**; select its radio ("Make this the active group"); close.
2. Confirm status "Active well group: UAT-North (3 wells)" and the Wells pane now lists only those 3.
3. NOW open a batch pane you had NOT opened since activation (e.g. **Petrophysics → VSH ▾ → VSH from Gamma Ray**): the Wells scope row must default to **Group** mode showing "3 wells" (hover the count for names).
4. **Run**; then per well check the Curve Catalog: the 3 members have new VSH; the 4th well must NOT.
   **Expected:** scope defaults to the active group with exactly 3 wells; run writes curves ONLY to the 3 members; the module form reports "All 3 well(s) computed…"; History entry reads "Ran VSH from Gamma Ray on 3 wells" (batch attributed correctly, not to the globally-selected well — covers REVIEW.md §Round 4 — "History attribution").
   **Result — T-INT-09:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-INT-10 — Switch active group WITH a batch pane already open (negative)

**Tool/panel:** Workflow Builder + Wells pane group dropdown (`src/ui/workflowDialog.ts`, `src/ui/wellScope.ts`)
**Preconditions:** T-INT-09 done; create a second group `UAT-South` with the OTHER wells.
**Steps:**

1. With `UAT-North` active, open **Petrophysics → Workflow…** — its Wells scope shows Group `UAT-North (3)`.
2. Leaving the Workflow pane open, switch the Wells-pane group dropdown to `UAT-South`.
3. Look at the still-open Workflow pane's scope row; add one step (e.g. VSH from Gamma Ray) and **Run chain**.
4. Check which wells actually received curves.
   **Expected (desired):** the open pane re-scopes to `UAT-South` and the run covers only its wells.
   **Known issue:** AUDIT-2026-07-21-full-qc.md, §Substrate — well-group scoping sweep #1: "No batch-run dialog re-scopes to a new active well group while it's already open — only the Wells sidebar tree and Map pane react live to a group switch." Expect the pane to keep showing `UAT-North` and the run to silently compute the OLD group's wells with no error — log as known, record which panes you see it in (module panes, Workflow, Monte Carlo, Cutoffs, Report all share the same scope control).
   **Result — T-INT-10:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-INT-11 — Constellation versioning round-trip: two vsh_gr runs, restore v1, downstream consumes it

**Tool/panel:** Module pane + Curve Catalog Constellations (`src/ui/moduleDialog.ts`, `src/ui/inspectorPanel.ts`)
**Preconditions:** T-INT-05 done on well 1; note a shaly interval's VSH/PHIE values.
**Steps:**

1. **VSH from Gamma Ray**, well 1 only, Output cons `TEST`, GR_SH = 120 → **Run** (v1).
2. Same pane, GR_SH = 90 → **Run** (v2).
3. **Data → Curve Catalog** → Constellations section: verify `TEST v1` and `TEST v2` rows, v2 badged "current"; hover a row for its params/inputs (GR_SH must differ).
4. Click **Restore** on `TEST v1`.
5. Run **Porosity from Density** (VSH input = VSH) on well 1 and read PHIE in the shaly interval.
   **Expected:** two versions coexist — nothing overwritten ("Constellations — every run is kept as a version"). After step 2 VSH is HIGHER in shaly beds (lower GR_SH inflates VSH). Restore shows "Version restored (N samples back in the current curves)", bumps every open panel, and VSH drops back to the v1 (GR_SH=120) values. Step 5's PHIE matches a v1-VSH computation (higher PHIE in shaly beds than it would be under v2) — proving downstream modules consume the restored values, and the Curve Catalog shows the new PHIE version's provenance.
   **Automated coverage - pinned (pile B, 2026-07-31):** `a_restored_log_set_version_feeds_the_next_module_run` (workflow.rs).

   **Result — T-INT-11:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-INT-12 — Session round-trip: busy workspace → save → close app → reopen → restore

**Tool/panel:** Quick-access Save/Open Session + workspace (`src/ui/ribbon.ts`, `src/ui/workspace.ts` snapshotSession/applySession)
**Preconditions:** any INT state.
**Steps:**

1. Build a busy workspace on well 2: a Log View with a customized layout (Plot → Properties…), **Histogram**, **Crossplot** (X=NPHI Y=RHOB Color=GR), **Pickett**, and the **Zones…** pane; drag a couple of panes into a second window (**View → New Window**).
2. Quick-access **Save Session As…** → name `UAT Busy` → **Save**.
3. **View → Reset Workspace** (scramble), then **Project ▸ Session ▸ Open Session…** → `UAT Busy`.
4. Close the app entirely (window ✕), relaunch `npm run tauri dev`, then **Open Session…** → `UAT Busy` again.
5. **Project → Theme** → switch to Dark (then back).
   **Expected:** (3) and (4): every pane returns in its arrangement, the session's well (well 2) is active everywhere, the Log View's customized layout is back (not the default), and the plots reopen with their curve choices/zone and persisted plot properties; status "Opened session \"UAT Busy\""; History records the session save/open. (5) every canvas panel — log view, plots, correlation — repaints immediately in the new theme with no stale colors (themeVersion).
   **Result — T-INT-12:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PERF-01 — App launch time on the big field project

**Tool/panel:** app shell + Wells pane (`src/main.ts`, `src/ui/objectTree.ts`)
**Preconditions:** the 100+/540-well project was the last-open project; dev server already compiled once (measure the SECOND launch so Vite compile time doesn't pollute the number).
**Steps:**

1. Launch; stopwatch from window appearing to (a) status bar "Ready" and (b) the Wells pane fully populated.
2. Click through the 6 ribbon tabs and open the Curve Catalog on one well.
   **Expected:** launch completes without white-screen hangs or "not responding"; well tree scrolls smoothly at 540 wells; ribbon stays responsive during load. Record (a)/(b) seconds in Notes — ROADMAP #128/#129 need this baseline.
   **Result — T-PERF-01:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:** s to Ready, **\_\_** s to well tree

### T-PERF-02 — Single-module run across ALL wells: UI responsiveness + Cancel

**Tool/panel:** VSH from Gamma Ray module pane + Processing panel (`src/ui/moduleDialog.ts`, `src/ui/processingPanel.ts`, `src-tauri/src/lib.rs` run_workflow_module)
**Preconditions:** big project; GR present on most wells.
**Steps:**

1. Open **VSH from Gamma Ray**; scope = **All** (540 wells); **Run**.
2. DURING the run: drag the window, switch ribbon tabs, scroll the Wells pane.
3. Watch the Processing panel: progress bar, counts row (▶/✓/⚠/✗/⏳), ▸ details.
4. Re-run and press **Cancel** in the Processing panel mid-run.
   **Expected:** the window stays draggable/responsive for the whole run; the Processing panel shows a live "Module" job with per-well progress and an integrated Cancel; Cancel stops within a well or two. Record total run seconds in Notes.
   **Known issue:** AUDIT-2026-07-21-full-qc.md, §VSH #1: "vsh_gr / vsh_dn standalone module runs never leave the Tauri main thread … no Processing-panel progress and no Cancel" (also §Prep statistical #3, "run_workflow_module … is still a synchronous main-thread-blocking command"). Note: current code contains a post-audit conversion (`run_workflow_module` now routed through the background job registry), so this may in fact pass — if the app DOES freeze with no Cancel, log it as this known finding, not a new bug; if it stays responsive, mark the finding field-verified fixed.
   **Result — T-PERF-02:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:** s, responsive: Y/N

### T-PERF-03 — Full workflow chain across all wells: speed + live progress

**Tool/panel:** Workflow Builder + Processing panel (`src/ui/workflowDialog.ts`, `src/ui/processingPanel.ts`)
**Preconditions:** big project.
**Steps:**

1. **Petrophysics → Workflow…**; add steps **VSH from Gamma Ray → Porosity from Density-Neutron → SW — Indonesia (Poupon-Leveaux)**; scope = **All**; Output cons `PERFTEST`; **Run chain**.
2. During the run: drag the window; watch the Processing panel's "Step k/3: <module>" line, the counts row, and the boundary "Writing N well(s)…" message.
   **Expected:** the chain on ~540 wells finishes in **seconds to a low number of minutes, not 30 min**; the window never goes "not responding"; the progress bar advances per well; step boundaries read "Writing N well(s)…" instead of sitting frozen; on completion the builder shows "Done: 3 steps, N curves across N wells" and open panels refresh. Covers REVIEW.md §540-well test — "Workflow chain runs without freezing…", "A chain of many wells now finishes in seconds…", "Processing panel: the step-boundary pause…". Record seconds/step in Notes.
   **Result — T-PERF-03:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:** s total

### T-PERF-04 — Cancel a 540-well chain mid-run

**Tool/panel:** Workflow Builder / Processing panel Cancel (`src/ui/workflowDialog.ts`, `src/ui/processingPanel.ts`)
**Preconditions:** T-PERF-03 repeatable; a Log View open on a well early in the run order, showing a chain output curve.
**Steps:**

1. **Run chain** (same 3 steps, all wells); at roughly mid-step-2 press **Cancel** (try the Processing-panel Cancel once, the builder's Cancel on a second attempt).
2. Time Cancel-press → "Cancelled at step N".
3. Look at the open Log View and Curve Catalog afterwards.
   **Expected:** cancel drains "in a well or two" (≈ seconds, not minutes); the bar clears; both Cancel buttons drive the same flag. Wells that completed BEFORE the cancel keep their committed step-1/2 curves (by design), and the open Log View/plots REFRESH to show them — no stale pre-run display (covers REVIEW.md §Round 4 — "dataVersion refresh … on workflow-chain cancel/fail" and §540-well test cancel items). Record cancel latency in Notes.
   **Result — T-PERF-04:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:** s to cancel

### T-PERF-05 — Field Dashboard compute on all wells

**Tool/panel:** Field Dashboard (`src/ui/dashboardPanel.ts`)
**Preconditions:** big project with VSH/PHIE/SWE computed (run T-PERF-03 first).
**Steps:**

1. **Petrophysics → Field Dashboard…**; set cutoffs; **Compute**.
2. Tweak one cutoff, **Compute** again. Sort by a column; **Export CSV**.
   **Expected:** compute across ~540 wells takes **seconds, not minutes** (stats-only — it must NOT write FLAG curves; verify no new FLAG versions pile up in a well's Curve Catalog); grid renders with "—" for empty/NaN aggregates and no `toFixed` crash; re-compute is equally fast; N/G ∈ [0,1] everywhere. Covers REVIEW.md §540-well test — "Field Dashboard no longer crashes…" and "Field Dashboard is fast now". Record seconds in Notes.
   **Result — T-PERF-05:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:** s

### T-PERF-06 — Correlation with 20+ strips

**Tool/panel:** Correlation panel (`src/ui/correlationPanel.ts`)
**Preconditions:** big project; tops loaded on the chosen wells.
**Steps:**

1. **Plot → Correlation**; open the **Wells** button menu and tick 20–25 wells along one trend.
2. Pick the shared curve (GR/GRN); flatten on a common datum top.
3. Pan/zoom the depth axis; hover; switch **Project → Theme** once.
   **Expected:** 20+ strips render with tops connectors and the datum flattened to a level line ("(no datum)" labels only on wells genuinely lacking it); pan/zoom stays interactive (no multi-second repaints); the theme switch repaints the canvas immediately. Geology check: correlatable markers align horizontally after flattening. Record redraw feel/seconds in Notes.
   **Result — T-PERF-06:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PERF-07 — Crossplot dense-cloud pan/zoom after the memoized-color fix

**Tool/panel:** Crossplot panel (`src/ui/crossplotPanel.ts` computeCrossplotColors memoization)
**Preconditions:** big project; pick the well with the longest/densest logged interval (or a spliced full-TD well).
**Steps:**

1. **Plot → Crossplot**: X=NPHI, Y=RHOB, Color=GR, Zone = full well; also try a PERM Z with log-Z on (⚙ Properties).
2. Drag-pan, Ctrl+wheel-zoom, drag the parameter handle, and hover from a Log View — continuously for ~10 s each.
3. Switch the Color curve and colormap.
   **Expected:** motion stays smooth on the dense cloud (colors are memoized — pan/zoom/hover must NOT re-sort percentiles per frame); colors, color-bar range and legend unchanged during motion; switching Z/colormap recolors immediately; a module re-run recolors against new data. Covers REVIEW.md §Performance (field-scale speed) — "Crossplot: Z coloring memoized across pan/zoom/hover". Note any stutter in Notes.
   **Result — T-PERF-07:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-PERF-08 — DB size / WAL behavior + force-kill while IDLE (crash recovery)

**Tool/panel:** DuckDB project file + crash-recovery flow (`src/autosave.ts`, `src-tauri/src/db.rs` WAL recovery)
**Preconditions:** big project AFTER the heavy runs above. **Do NOT deliberately force-kill during a write — only when the Processing panel shows no running job.**
**Steps:**

1. In Explorer, note the `.duckdb` file size and whether a `.duckdb.wal` sits beside it after the heavy runs (record both in Notes).
2. With the app IDLE (no Processing job, status quiet), kill it from Task Manager (End task).
3. Relaunch. At the abnormal-exit prompt choose **restore the autosaved workspace** (first run) — later repeat once choosing **Safe Mode**.
4. Open the Curve Catalog and the DB Inspector; spot-check counts against pre-kill.
   **Expected:** relaunch detects the abnormal exit and offers restore/Safe Mode (Safe Mode stashes the autosave as a "Recovered …" session — nothing silently lost); the project opens with NO corruption — every well, curve version and history entry from before the kill is intact; any WAL replays silently (a corrupted-WAL fallback would recover from the last checkpoint and say so in the console — report that if seen). DB size should be broadly proportional to data (hundreds of MB is plausible at 540 wells; note it for the versioning-growth baseline).
   **Result — T-PERF-08:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:** DB **\_\_** MB, WAL present: Y/N, recovery: **\_\_**

---

# Section SHIP — Session 2026-07-28/29 shipping checks

Added 2026-07-29. Covers what that session shipped: the hardened Content-Security-Policy
(`tauri.conf.json`), R30 (loud failure on missing perm curve), R-A (project format stamp),
R-B (pre-migration backup) and the green gate (`tools/check.ps1`). REVIEW.md Rounds 89–93
carry the full narratives. **Realtime status notes below each test say what Claude already
machine-verified** — those tests you can run lighter; the rest need your hands.
Non-app items from the same session (PRD/V1_SCOPE read-through, IP_PROVENANCE lawyer
questions, Python-prerequisite decision, PR #2 merge) live in `docs/manual_check_plan.md`.

### T-SHIP-01 — Packaged app launches under the hardened CSP

**Tool/panel:** packaged build (`src-tauri/tauri.conf.json` `security.csp`)
**Preconditions:** none. The CSP is enforced ONLY in packaged builds (`npm run tauri build`,
with or without `--debug`); `npm run tauri dev` uses the vite dev server and ignores it, so
every dev-mode session to date has never exercised it.
**Steps:**

1. Build through the vcvars pin: `npm run tauri build -- --debug --no-bundle` (or the full
   release build) and launch `src-tauri\target\debug\sandibumi.exe` (or release).
2. Wait for the window.
   **Expected:** a normal SandiBumi window — ribbon, panes, wells list. A blank white window
   means the CSP blocked the app bundle itself (script-src): report which, don't work around it.
   **Realtime status (2026-07-29):** ✅ machine-verified by Claude — packaged debug exe driven
   over the WebView2 debug port with the PR's CSP applied: full UI rendered (12 ribbon tabs,
   dockview, ribbon), policy proven LIVE (deliberate probe violations quote it: remote fetch blocked
   by connect-src, injected inline script refused by script-src), zero unexpected violations.
   Run it once yourself for the release build, but expect green.
   **Result — T-SHIP-01:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHIP-02 — CSP-sensitive features in the packaged app

**Tool/panel:** Vega panel (needs `'unsafe-eval'`), Equation Editor, Composite PDF
**Preconditions:** T-SHIP-01 passed; a project with at least one well of real curves.
**Steps:**

1. Plot tab → Vega Chart — pick X/Y curves, confirm a chart draws (Vega compiles specs with
   the `Function` constructor; this is the directive most likely to be wrong).
2. Equation Editor: run one Rhai and one Python equation.
3. Plot → Composite… → export a PDF; open it.
4. Leave the app via the window ✕ (never task-kill — WAL).
   **Expected:** all three work exactly as in dev mode. Any CSP violation shows as a feature
   silently doing nothing — if one does, note which feature and check the webview console.
   **Realtime status (2026-07-29): ✅ ALL LEGS machine-verified** in a packaged debug build
   carrying the PR's real CSP, on an isolated scratch project. The policy was first proven
   _enforced_ in that exact build — deliberate probes were blocked with messages quoting our own
   directives (remote fetch → `connect-src`, injected inline `<script>` → `script-src`) while
   `eval`/`Function` stayed allowed — so the clean results below mean something:

- **Vega**: real scatter of 20 imported points; `marks` canvas has painted pixels.
- **Equation Editor**: CodeMirror mounts (its lazy chunk loads under `script-src 'self'`);
  a **Rhai** run (`gr * 2.0`) and a **Python/numpy** run (`np.clip(gr/150,0,1)`) each wrote
  20 rows with numerically exact results (77–204.8 and 0.2567–0.6827 for a GR range of
  38.5–102.4). Python engine auto-discovered at `…\Python312\python.exe`.
- **Composite PDF**: `render_composite` → 1 page; `export_composite_pdf` wrote a real
  5,634-byte file starting with `%PDF-`.
- **`printCanvas`** (the genuinely CSP-risky path, checked on purpose): its srcless iframe +
  inline `<style>` + `data:` image all worked — image loaded, style applied, **0 violations**.
  Zero unexpected CSP violations and zero console errors across the whole run.
  **Note on convention** (cost me a false alarm): equation scripts receive inputs **lowercase**
  and assign to the **output curve name** — `vsh = np.clip((gr-20)/120, 0, 1)`, not `GR`/`out`.
  **Result — T-SHIP-02:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHIP-03 — R30: missing perm curve fails loudly, never computes on GR

**Tool/panel:** Lorenz, SHF fit, Facies tie-in dialogs (`plotCommon.preferredCurveSelect`)
**Preconditions:** one well WITH a PERM/KLOGH/K curve, one well WITHOUT any.
**Steps:**

1. On the with-perm well: open each of the three dialogs — check the perm slot preselects
   the real curve (styled dropdown, no duplicate entry).
2. On the without-perm well: the perm dropdown shows the preferred name (e.g. `PERM`)
   anyway; Run each dialog.
   **Expected:** step 2 fails with the backend's own message naming the curve ("permeability
   curve 'PERM' has no data in this well") — NOT a plausible-looking result silently computed
   on GR. REVIEW.md Round 90 has the full story.
   **Automated coverage - pinned (pile B, 2026-07-31):** `a_missing_curve_fails_by_name_rather_than_computing_on_another` (lorenz.rs).

   **Result — T-SHIP-03:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHIP-04 — R-A: the project carries a format stamp

**Tool/panel:** SQL Query panel (`db.rs` `check_and_stamp_format`)
**Preconditions:** any project opened at least once by a build at or after commit `1842bc8`.
**Steps:**

1. Open the project normally — everything must behave exactly as before (stamp is invisible).
2. SQL Query panel: `SELECT * FROM project_meta`.
   **Expected:** two rows — `format_version` = 1, `written_by` = SandiBumi 0.1.0. The refusal
   path (newer file on older build) is cargo-tested; no manual setup can produce it today.
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-SHIP-04:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHIP-05 — R-B: destructive migration backs up first; normal opens write nothing

**Tool/panel:** project open path (`db.rs` `backup_before_destructive_migration`)
**Preconditions:** your current (already-migrated) project; optionally a pre-2026-07-19
project copy that still has the old computed_curves PRIMARY KEY.
**Steps:**

1. Open your current project; check its folder in Explorer.
2. (Optional) Open the old copy; watch its folder and the console.
   **Expected:** step 1: NO new `*-backup.duckdb` appears, launch is not slower — absence is
   the pass. Step 2: a `<name>.pre-1-backup.duckdb` appears BEFORE the rebuild and the launch
   log announces it; the backup opens as a valid project if pointed at directly.
   **Automated coverage - pinned (pile A):** a test on the green gate already checks the arithmetic behind this one. What your tick still adds is that the running app surfaces it correctly.

   **Result — T-SHIP-05:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

### T-SHIP-06 — The green gate from your own shell

**Tool/panel:** `tools/check.ps1`
**Preconditions:** repo checkout; no other cargo build running.
**Steps:**

1. `powershell -ExecutionPolicy Bypass -File tools\check.ps1`
   **Expected:** `GATE GREEN` with the full test count, non-zero exit on any failure.
   **Realtime status (2026-07-29):** the previously-flagged SSC blocker is **resolved**. The
   `ssc.rs` Loglan-alignment work was committed in `d1f0c1e`, which left
   `ssc_swirr_floor_pads_capillary_water` asserting the old contract; the test was corrected to
   the documented one (see REVIEW.md Round 95) and the gate is green with nothing stashed.
   **Result — T-SHIP-06:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---

### T-SHIP-07 — R-C: a normal close must not lose recent writes (added 2026-07-29)

**Tool/panel:** app exit path (`lib.rs` `RunEvent::Exit` checkpoint)
**Preconditions:** a COPY of a project (or a fresh scratch project) — this test writes.
**Steps:**

1. Import one small LAS (or edit one sample value).
2. Close the app with the window ✕ immediately (no waiting).
3. In Explorer, look beside the `.duckdb` file.
4. Reopen the app on the same project.
   **Expected:** after step 3 there is NO `.duckdb.wal` file left (the exit checkpoint flushed
   it) and no new `.corrupt-backup-*` appears; after step 4 the import/edit is still there.
   **Why it exists:** before this fix, every close — window ✕ included, not just force-kills —
   abandoned a live WAL; on the next open the WAL could fail replay and the recovery silently
   dropped everything written since the last auto-checkpoint (reproduced twice: a fresh import
   vanished). Full story: REVIEW.md Round 94.
   **Realtime status (2026-07-29):** ✅ machine-verified on the packaged exe by the exact
   failing scenario (import → ✕ → relaunch: well persists, no WAL, no corrupt-backup). Run it
   once on real data for confidence.
   **Result — T-SHIP-07:**

- [ ] Pass
- [ ] Fail
- [ ] Blocked

**Notes:**

---
