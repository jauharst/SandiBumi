# Review checklist — for Jauhar's click-through in `npm run tauri dev`

## 2026-08-15 — G2 SB-PLT-030: Accessibility for Histogram, Crossplot, Pickett, Correlation and Vega interaction

- [ ] **Automated correctness:** the exact T39 test executes one shared accessibility contract,
      changes the real X view with ArrowRight, refreshes a changed accessible label, opens the
      Properties route with `P`, focuses export with `E`, and proves disposal removes the handler.
      Its source inventory requires the same current-label, keyboard-view, non-pointer Properties,
      export-focus and disposal boundaries in Histogram, Crossplot, Pickett, Correlation and Vega.
      Correlation must mutate its depth viewport and use its existing zoom routine; Vega must mutate
      runtime domains and repaint. Reversing ArrowRight and removing Vega's export route each made
      the test RED. The generated Vega canvas also shares the existing visible focus-ring rule.
      TypeScript and the complete 24-test frontend acceptance file are green; the fresh full gate is
      1036 passed / 0 failed / 37 ignored with 31 separately owned Rust warnings.
- [ ] **Visual:** open Histogram, Crossplot, Pickett, Correlation and Vega, then use only Tab to
      reach every canvas. Confirm each focused canvas has a visible ring and a current description
      naming its selected curves and context. Exercise arrow pan, `+`/`-` zoom and `Home` fit; verify
      the plotted viewport really changes. Press `P` and `E` and confirm Properties and export
      controls receive visible focus without a pointer.
- [ ] **Manual:** repeat the five-panel sequence with keyboard only and a Windows screen reader.
      Confirm changed curve, well, interval, depth basis and Vega channel selections update the
      announced label; no shortcut scrolls the page instead; focus never disappears into the
      generated Vega subtree; and closing a panel removes its keyboard handler.
- [ ] **Field and harsh critique:** repeat on the representative pilot workstation with real plot
      sizes and selections. A `tabindex`, helper call and green synthetic event are not usable
      accessibility if focus is invisible, a view does not move, the accessible name becomes stale,
      or Properties/export still require a mouse. Automated source inventory cannot prove screen-
      reader announcements, browser/renderer focus behavior, user comprehension or field usability;
      those evidence classes remain open until Jauhar records them.

## 2026-08-15 — G2 SB-PLT-029: generation-safe Histogram, Crossplot, Pickett, Correlation, Vega and log-view loads

- [ ] **Automated correctness:** the exact T28/T33 test inventories all 15 asynchronous
      build/refetch boundaries owned by the chapter's five plot surfaces plus the specified
      log-view viewport refetch. It resolves two panel builds in reverse order, starts a new data
      revision while an older refetch is in flight, and requires only the newest generation to
      apply while both stale disposable results are torn down before active-panel mutation.
      Workspace replacement, correlation's applied-only draw, detached Vega embed and live-host
      commit order are pinned separately. Removing the generation comparison and embedding Vega
      directly into the live host each made the same test RED. TypeScript and cargo check are
      green; the fresh full gate is 1036 passed / 0 failed / 37 ignored with 31 separately owned
      Rust warnings.
- [ ] **Visual:** with Crossplot, Histogram, Pickett, Correlation and Vega open, switch curves,
      zones, scoped wells and the selected well rapidly enough to overlap loads. Trigger a data
      revision while each plot is still loading. Confirm the newest selection remains visible,
      no older chart flashes back into the pane, and Correlation does not redraw an earlier well
      inventory after the current one.
- [ ] **Manual:** repeat reverse-order completion with developer throttling or a controllable test
      backend. Close a plot while its build, context fetch, Vega editor import or Vega resize is
      pending; confirm stale content is disposed, no detached Vega result re-enters the live host,
      no stale failure message replaces a current chart, and the log viewport keeps the newest
      generation-tagged half-open refetch.
- [ ] **Field and harsh critique:** repeat with representative multi-well context scopes and data
      volumes. A green deterministic race test proves ordering and disposal at registered
      boundaries; it does not prove network/storage latency realism, visual stability on every
      machine, or that a future unregistered asynchronous branch cannot be introduced. The
      executable inventory is the tripwire for that future change, not permanent omniscience.

## 2026-08-15 — G2 SB-PLT-026: measured paper export and honest raster print

- [ ] **Automated correctness:** the exact T37/T38 test draws a long axis label beyond the source
      canvas's left edge and an outside legend beyond its right edge, then proves SVG and PDF rerun
      the supplied scientific draw, retain the axis/legend/annotation/footer as vector text and
      expand their point-sized page around real TextMetrics glyph bounds. SVG measurement, PDF
      preflight and the final PDF draw must consume the same text width; a declared content box
      smaller than its source canvas is rejected. It also proves the shared print
      control says `Print raster…`, embeds the same provenance/exclusion footer and records backing
      pixels as pixels rather than falsely calling them physical points. A mutation that moved the
      page edge inside its content and the former 0.6-em PDF width each made the test RED before
      repair. A separate Rust test pins the independent write refusal, source-canvas inclusion,
      vector metadata embedding and dishonest-raster-unit control. TypeScript and cargo check are
      green; the fresh full gate is 1035 passed / 0 failed / 37 ignored. Cargo check reports the
      separately owned 31 Rust warnings.
- [ ] **Visual:** export Crossplot, Histogram, Pickett and Vega plots with the longest realistic axis
      labels, context legends, annotations and statistics blocks. Open the SVG and PDF side by side;
      confirm all marks remain readable, no label/footer is clipped, point sizing is useful on paper,
      and the added physical margin does not make the plot implausibly small. Confirm Correlation and
      every Canvas/Vega toolbar call the print route `Raster`, never an unqualified `Print`.
- [ ] **Manual:** inspect SVG text elements and the embedded `sandibumi-paper-export` records, then
      inspect PDF text selection and its embedded record. Verify recorded content bounds sit inside
      page bounds and the visible footer counts match the plot's wells, bindings, axes, statistics,
      exclusions and display-hidden samples. Print one raster route to paper/PDF and verify the title
      explicitly says raster, the footer remains visible and full ancestry/binding records follow.
- [ ] **Field and harsh critique:** repeat with representative long labels, many-well legends,
      decimated plots, non-finite/validity exclusions and vendor-overlay refusals. Measured recorder
      bounds do not prove typography is attractive or every printer driver preserves layout. Raster
      pixels are deliberately not called paper points. Automated green proves custody and no-crop
      geometry for recorded vector marks, not real-printer fidelity, visual legibility or field use.

## 2026-08-15 — G2 SB-PLT-024: vendor chart payload remains a legal blocker

- [ ] **Automated factual inventory:** the generated `src/ui/chartOverlays.ts` remains imported by
      the crossplot surface and contains the 19 vendor-derived numeric definitions recorded by
      `docs/IP_PROVENANCE.md` section 2.1 and CLAIM-013. SB-PLT-023 now blocks their screen and
      deliverable rendering without an approved record, but a blocked renderer does not remove the
      payload bytes from the repository or application bundle. No correctness test is claimed
      because executable code cannot establish licence or redistribution rights. The fresh full
      gate is 1034 passed / 0 failed / 37 ignored; cargo check reports the separately owned 31 Rust
      warnings.
- [ ] **Visual:** confirm the chart selector still labels every current chartbook overlay blocked and
      that choosing one shows the provenance refusal rather than curves. This confirms fail-closed
      behavior only; it is not legal clearance.
- [ ] **Manual/legal:** counsel must select one O-5/CLAIM-013 route before first sale: document a
      sufficient licence, replace the payload with an independently digitized published primary
      source carrying full custody, or remove the payload from the paid build/repository while
      retaining only lawful metadata and tooling.
- [ ] **Field and harsh critique:** an unreachable or visually hidden payload can still be copied in
      distributed source or binaries. Green builds, factual inventory, metadata, and blocked UI do
      not prove ownership, permission, scientific correctness, or lawful redistribution.

## 2026-08-15 — G2 SB-PLT-023: provenance-complete chart custody and refusal

- [ ] **Automated correctness:** the exact SB-PLT-023/T35 test carries one complete metadata-only
      fixture, sourced to Pittman (1992) as already classified in chapter 15, unchanged through
      screen, saved state, template, SVG and PDF surfaces; removes the revision and proves every
      surface refuses; and changes the chart identity to prove a different record cannot authorize
      the selected payload. Removing the revision check made the test RED before restoration. The
      Rust boundary independently validates the record and embeds it in SVG/PDF metadata. Existing
      vendor-derived chartbook overlays remain blocked because no rights-approved source record was
      added. The fresh full gate is 1034 passed / 0 failed / 37 ignored; cargo check reports the
      separately owned 31 Rust warnings.
- [ ] **Visual:** open a Crossplot whose axes match a chartbook overlay. Confirm each existing
      unapproved overlay is labelled blocked in the selector and draws a readable refusal instead of
      chart curves. At the smallest normal dock size, the refusal must not masquerade as a plotted
      result or disappear beneath the axes.
- [ ] **Manual:** with a test build containing one rights-approved record, confirm the same chart ID,
      title/type, typed axes, citation, publisher, revision, digitizer, derivation path, checksum and
      transform survive last-used state, named-template reload, SVG metadata and PDF metadata.
      Remove only the revision and confirm screen, state/template save and both vector exports refuse.
- [ ] **Field and harsh critique:** do not interpret populated metadata as lawful provenance. The
      existing vendor-derived payloads remain unusable until rights and exact source custody are
      independently approved. A green metadata fixture proves fail-closed plumbing, not content
      rights, digitization correctness, scientific validity, visual readability or representative-
      field behavior; those remain open until Jauhar records them.

## 2026-08-15 — G2 SB-PLT-019: one invalidation and disposal contract for every plot

- [ ] **Automated correctness:** the exact SB-PLT-019/T32 test executes one current-value event
      source for each of theme, data revision, interval, selection and size across Crossplot,
      Histogram, Pickett, Vega and Correlation; proves construction is not miscounted as a change;
      proves one theme change reaches each panel exactly once without replacing its data or viewport;
      exercises every other event; and proves idempotent disposal removes all subscriptions and
      cancels pending work exactly once. Omitting the interval subscription made the test RED before
      restoration. Live-source inventory rejects private governed subscription lists. The full gate
      is green at 1033 passed / 0 failed / 37 ignored with 33 owned Rust warnings.
- [ ] **Visual:** open Crossplot, Histogram, Pickett, Vega and Correlation together. Switch light/dark
      theme and confirm all five repaint once without a flash, data reset or viewport jump. Select and
      clear a top interval, brush exact depths, and resize each dock pane. Confirm the four
      zone-windowed plots update, Correlation shows and clears its interval band and exact-depth
      rings, and no label, toolbar or disclosure overlaps at the smallest normal pane size.
- [ ] **Manual:** zoom/pan all five plots to recognisably different viewports, then change theme and
      compare the same data and bounds before/after. Trigger a data revision, interval, selection and
      resize separately; confirm each open panel responds once. Close every panel while a reload,
      brush frame or delayed menu/timer is pending, then change every event again and confirm no
      detached panel redraws, writes status, reopens a menu or replaces active content.
- [ ] **Field and harsh critique:** repeat on representative long, sparse, multi-set and multi-well
      logs. Five calls to one helper are not proof if one handler is visually inert, if Vega resets its
      zoom, or if a closed panel still wins an async race. A green synthetic event test does not prove
      real dock timing, renderer behavior, legibility, memory release or representative-field UX;
      Visual, Manual and Field remain open until Jauhar records them.

## 2026-08-15 — G2 SB-PLT-017: identified log-view viewport refetch

- [ ] **Automated correctness:** the exact SB-PLT-017 T27/T28 test starts from a known loaded
      interval, proves an equally dense contained view does not fetch, proves a crossed high bound
      issues one `[low,high)` request with a generation token, collapses its duplicate, resolves two
      requests in reverse order and renders only the newest, and surfaces both pending and failure
      states in the panel. Inverting the generation guard made the test RED before restoration. The
      Rust query proof excludes the high endpoint. The full gate is green at 1032 passed / 0 failed /
      37 ignored with 33 owned Rust warnings.
- [ ] **Visual:** open a log view on a dense curve, zoom in and pan past the currently loaded depth
      interval. Confirm the track shows a readable provisional-data notice while detail loads, then
      clears it when the denser trace arrives. At the smallest normal dock size, neither the notice
      nor a refresh-failure message may cover the depth scale or curve headers.
- [ ] **Manual:** pan and zoom rapidly enough to overlap two loads, finishing on a visibly different
      interval. Confirm the final interval stays rendered after the older request completes, repeat
      the same settled view and confirm it does not issue another fetch, and verify the query excludes
      a sample exactly at the requested high bound. Confirm no source curve, sampling or Reframe
      output is written.
- [ ] **Field and harsh critique:** repeat on representative long, dense, sparse and multi-set logs.
      If zoom merely enlarges a coarse whole-log trace, if an old response repaints a newer view, or
      if a failed refresh leaves plausible-looking data without disclosure, this requirement fails.
      Automation does not prove visual clarity, user comprehension, response time or field behavior;
      those remain open until Jauhar records them.

## 2026-08-15 — G2 SB-PLT-016: exact depth reconciliation and explicit Reframe handoff

- [ ] **Automated correctness:** the exact SB-PLT-016 T23-T26 test executes equal regular grids,
      exact 0.5/1.0 multiples, both irregular-identical and non-integer 0.5/0.8 refusals, and the
      `[100,101)` half-open interval. It clicks the visible handoff, proves the shell opens Reframe
      once, inventories Crossplot, Histogram, Pickett and shared context loading, and proves no plot
      calls Reframe automatically. An event-name mutation made the test RED before restoration; the
      Rust oracle also passes. The full gate is green at 1031 passed / 0 failed / 37 ignored with 34
      owned Rust warnings.
- [ ] **Visual:** load equal, exact-multiple and incompatible native-grid curves in Crossplot,
      Histogram context and Pickett. Confirm compatible plots render normally; an incompatible plot
      shows one readable warning card and `Open Reframe` button at the smallest normal dock size;
      clicking it opens Reframe and the status explicitly says no plot data were resampled.
- [ ] **Manual:** compare the equal and exact-multiple results against their native samples, confirm
      the factor disclosure is correct, and verify `[lo,hi)` excludes the high endpoint. For both an
      irregular grid and non-integer step ratio, snapshot the source curve bytes, exercise the
      refusal and Reframe handoff without running Reframe, and confirm the source remains unchanged.
- [ ] **Field and harsh critique:** repeat with representative multi-set wells containing equal,
      unequal, sparse and irregular native grids. If incompatible logs merely look aligned, or if a
      refusal gives no obvious next action, the UI is safer-looking rather than safe. This automated
      proof does not establish visual placement, user comprehension, representative-field behavior
      or SB-PLT-017 viewport refetching; those remain separate evidence.

## 2026-08-15 — G2 SB-PLT-015: shared-index decimation and portable tail provenance

- [ ] **Automated correctness:** the exact SB-PLT-015 T21/T22 test derives source indices
      `0,4,8,10` from eligible `0..10` at stride 4, proves depth/X/Y/Z marks all use those same
      indices, preserves the first and forced final sample, labels the view reduced rather than
      complete, and carries counts, algorithm, stride and forced-endpoint state through the live
      context-panel manifest and whitelisted Rust serializer. A deliberate true-to-false endpoint
      mutation made the test RED before restoration. The exact full gate is green at 1030 passed /
      0 failed / 37 ignored with 36 owned Rust warnings.
- [ ] **Visual:** open Crossplot, Histogram and Pickett with enough context points to trigger
      reduction. Confirm each scope row says reduced, shows original→displayed counts, stride and
      whether a tail was forced, remains readable at the smallest normal dock size, and never says
      the displayed view is complete. Export the JSON manifest and check the same fields are visible
      to a human without reading source code.
- [ ] **Manual:** compare the first and final eligible depth plus several interior X/Y/Z marks against
      the unreduced source on all three panels. Export, close and reopen the panels, repeat with one
      well not needing reduction and one requiring a forced tail, and confirm non-stride well/legend
      reductions record null stride/endpoint fields rather than borrowed numbers.
- [ ] **Field and harsh critique:** repeat with representative multi-set wells on native, unequal and
      sparse grids. Decimation that keeps the right count but pairs one channel with another source
      index fabricates rock; a manifest hidden behind a button or unreadable in normal work is weak
      protection. Green arithmetic and serialization tests do not prove that rendered glyphs or a
      delivered PDF look correct, and this increment does not claim the deferred SB-PLT-014
      multi-well allocation or SB-PLT-016 depth-reconciliation contracts.

## 2026-08-15 — G2 SB-PLT-013: channel-specific missing and overflow policy

- [ ] **Automated correctness:** the exact SB-PLT-013 contract test exercises shared X/Y policy,
      Crossplot/Pickett/Vega Z adapters, screen and composite spaghetti-waveform adapters, both
      linear and logarithmic channels, low/high endpoint marks, separate exclusion/clamp counts and
      bit-exact source preservation. The two Rust supporting tests execute the production composite
      consumer and policy. A deliberate high-edge-to-low-edge mutation made the acceptance test RED
      before restoration. The exact full gate is green at 1029 passed / 0 failed / 37 ignored with
      38 owned Rust warnings.
- [ ] **Visual:** open Crossplot, Pickett, generated Vega and a log/composite spaghetti array with
      deliberately low, high, non-finite and non-positive-log samples. Confirm low/high Z overflow
      uses distinguishable endpoint diamonds, every count is readable at the smallest normal dock
      size, waveform disclosure does not cover traces, and composite disclosure remains legible on
      paper/PDF. Automation proves content and call paths, not placement or legibility.
- [ ] **Manual:** compare the same samples across linear and log axes; narrow only display limits;
      confirm X/Y overflow hides without changing the analysis population, Z and waveform overflow
      clamps only the display copy, log-invalid values leave plot and plot statistics, and all offered
      SVG/PDF/PNG or composite routes report the same dispositions. Re-read the source curve before
      and after every interaction and confirm no display operation created an edit or history entry.
- [ ] **Field and harsh critique:** repeat with representative native-grid curves and real array logs,
      including long outlier tails and sparse gaps. A polished plot with silent clipping is a
      plausible lie; a tiny or overlapping disclosure is almost as bad because the operator will not
      use information they cannot read. Green helper and source-inventory tests do not prove that
      users notice or understand the marks, and this increment does not claim deferred multi-well
      allocation, shared reduction or performance contracts.

## 2026-08-15 — G2 SB-PLT-009: statistics carry population, estimator and exclusions

- [ ] **Automated correctness:** the exact SB-PLT-009 T12/T13 test executes Histogram, Crossplot,
      Pickett, Correlation and generated Vega adapters, including active versus pooled wells,
      two-sided and one-sided intervals, sample versus population standard deviation, display-only
      clipping and one record per raincloud group. The Rust boundary test preserves a reconciled
      export record and refuses mismatched totals, foreign wells and channels absent from the plot
      bindings. A deliberate P5-to-minimum whisker mutation made the T13 assertion RED before
      restoration. The exact full gate is green at 1028 passed / 0 failed / 37 ignored with 42
      owned Rust warnings.
- [ ] **Visual:** open Histogram, Crossplot, Pickett, Correlation and Vega, including a grouped
      raincloud. Confirm the disclosure remains readable without covering the plot, grouped details
      collapse rather than forming a wall of text, and a top-to-TD interval is shown as
      `[top,+inf)` rather than `all`. Automation proves content and custody, not visual legibility.
- [ ] **Manual:** switch between active-well and pooled data; set, clear and reverse interval bounds;
      apply and remove a selection; introduce non-finite, log-invalid and validity-excluded samples;
      clip the display without changing analytical `n`; and switch sample/population standard
      deviation. Inspect screen plus every SVG/PDF/PNG, clipboard and print route for identical
      population, interval, estimator, selection, finite-pair and exclusion records.
- [ ] **Field and harsh critique:** repeat on representative multi-set wells, mismatched native grids,
      all-NaN deliveries and categorical groups. A correct numeric statistic is still a plausible lie
      if its population, interval, exclusions or estimator drift; a green adapter/export test does not
      prove the disclosure is readable or understood in real use. This increment does not claim the
      deferred SB-PLT-010 regression record or SB-PLT-011 Pickett identifiability contracts.

## 2026-08-15 — G2 SB-PLT-006: one canonical histogram-bin contract

- [ ] **Automated correctness:** exact SB-PLT-006 is green at 1 passed / 0 failed / 0 ignored;
      the full gate is green at 1027 passed / 0 failed / 37 ignored with 42 owned Rust warnings.
      T06/T07 execute through canonical, primary Histogram, crossplot-marginal and pre-binned Vega
      adapters; the inventory pins log-view glyphs plus Canvas and Vega export custody. A deliberate
      final-upper-endpoint mutation returned `[1,1,1]` instead of `[1,1,2]` before restoration, and
      the Rust supporting test executes the real distribution contract rather than a dead wrapper.
- [ ] **Visual:** open the primary Histogram, enable crossplot marginal histograms and open a Vega
      histogram for the same finite population and governed range. Confirm the primary axis says
      `displayed n=X of analysis n=Y`, the crossplot footer shows both marginal displayed totals,
      Vega says `histogram bins=50 · displayed total=X`, and the labels remain readable at the
      smallest normal dock size. Automation proves strings and data rows, not legible placement.
- [ ] **Manual:** exercise bin requests at 1, 50 and 200 plus out-of-range 0 and 201, then compare
      the final-upper-endpoint fixture and a delivery containing NaN/infinity across the primary,
      marginal and Vega surfaces. Zoom without re-binning, export every offered SVG/PDF/PNG route,
      and confirm bar edges, counts, displayed totals and non-finite exclusions agree with screen.
- [ ] **Field and harsh critique:** repeat on representative pilot deliveries with reversed axes,
      logarithmic crossplot marginals, context wells and finite tails outside the displayed range.
      A clean bar chart is dangerous if one surface shifts an endpoint or silently drops non-finite
      samples; a green adapter/inventory test is not real-app visual proof. This increment does not
      claim HFU, Monte-Carlo or other deferred scientific histograms, and it does not close Manual
      or Field evidence.

## 2026-08-15 — G2 SB-PLT-005: unit-limit content is audited before activation

- [ ] **Automated correctness:** exact SB-PLT-005 is green at 1 passed / 0 failed / 0 ignored;
      the full gate is green at 1026 passed / 0 failed / 37 ignored with 44 owned Rust warnings.
      One exhaustive test inventories the nine source-owned family rows plus the audit-only
      attenuation refusal, proves exact RHOB and screened rounded DT conversions, refuses the
      documented 6.56× attenuation pair and an unknown density unit with reasons, and inventories
      all five live panel consumers. A deliberate RHOB `2950 -> 3000` mutation returned the expected
      RED before restoration; the Rust fixture independently derives the 6.56× result.
- [ ] **Visual:** open Crossplot, Histogram, Pickett, Correlation and Vega with RHOB `kg/m3`, DT
      `us/m` and an intentionally unknown RHOB unit. Confirm the registered rows use the audited
      family display range, the unknown unit uses finite data, and its visible range label names the
      disabled family limit instead of silently borrowing `g/cc`. Check that the reason remains
      readable at each real dock size; automation proves text content, not layout.
- [ ] **Manual:** set user and curve-header ranges above the same curves, then clear them in order.
      Confirm precedence remains user → header → audited family → finite data, and inspect every
      offered save/export record for the same tier, row ID, unit, source and enable/disable reason.
      Verify `g/c3`, `gm/cc` and `us/f` no longer activate a familiar-looking family limit merely
      because older code treated them as aliases.
- [ ] **Field and harsh critique:** repeat on representative pilot deliveries containing observed
      unit spellings. The shipped registry is the cited nine-row seed set, not authority for all 83
      incumbent rows and not a physical-family range table. A schema-valid wider table remains
      disabled until every row has its own source and dimensional audit; attractive axis defaults
      are especially dangerous because a wrong range can hide valid rock while the chart still
      looks polished.

## 2026-08-15 — G2 SB-PLT-004: display clipping and analyst validity are separate populations

- [ ] **Automated correctness:** exact SB-PLT-004 is green at 1 passed / 0 failed / 0 ignored;
      the full gate is green at 1025 passed / 0 failed / 37 ignored with 47 owned Rust warnings.
      Five unequal samples prove display clipping counts two hidden without changing `n=5`, while
      explicit validity excludes two, changes `n` to 3, changes the independently derived mean
      from 2 to 3 and reports the fit-input count. All five pilot panel adapters execute the same
      policy; a deliberate Histogram mutation returned the expected `5 !== 3` RED before restore.
- [ ] **Visual:** open Crossplot, Histogram, Pickett, Correlation and Vega. First zoom or narrow only
      the display and confirm `display hidden` changes while analytical `n`, statistics and active
      fit inputs do not. Then enter complete validity limits, explicitly enable them, and confirm
      `validity excluded`, `n`, statistics and fit inputs change together. A fresh Tauri release was
      built, but the sandbox capture failed before producing a frame, so no screenshot is claimed.
- [ ] **Manual:** save and reopen each panel with validity disabled, enabled and later cleared. Confirm
      incomplete/equal/non-finite pairs refuse or stay disabled, Pickett refuses an excluded anchor
      and clears an invalidated two-anchor fit, Correlation breaks clipped traces instead of clamping
      them to an edge, and a generated Vega regression uses the disclosed filtered population.
- [ ] **Field and harsh critique:** repeat with representative multi-set wells, log axes, context wells
      and reversed display axes. A green helper plus adapter inventory does not prove text remains
      readable at real dock sizes; the current E2E harness is stale on scoped `list_wells`, workflow
      custody and capture-only teardown, and must not be mistaken for product visual acceptance.

## 2026-08-15 — G2 SB-PLT-002: Crossplot, Histogram, Pickett, Correlation and Vega governed axis custody

- [ ] **Automated correctness:** the one owned T01/T02 contract test and full gate are green at
      1024 passed / 0 failed / 37 ignored, with 49 owned Rust warnings. Unequal discriminator ranges prove user over header,
      header after user removal, matching rendered/exported tier records and no validity promotion;
      the same test inventories all five live quantitative panel adapters so an unused helper alone
      cannot pass.
- [ ] **Visual:** for each panel, first leave the range blank and confirm the visible label names
      header display, audited family display or finite data as appropriate; then pan/zoom or enter a
      complete user pair and confirm the label changes to `user`. Confirm a missing governed range
      produces the explicit refusal instead of an invented frame, and a custom Vega spec says why
      custody-dependent save/export is unavailable.
- [ ] **Manual:** set and undo a curve-header display range in Curve metadata. Reopen the plot,
      save project properties or a named session, and export every offered SVG/PNG/PDF path. Compare
      the displayed limits and tier to the embedded binding record; also apply a very different
      validity range and confirm it filters only when requested and never reframes the axes.
- [ ] **Field and harsh critique:** repeat with representative multi-set wells, converted units and
      reversed porosity axes. Automated precedence does not prove labels fit at real dock sizes,
      Vega runtime scale names remain stable after interactive transforms, or the screened family
      seed set covers every pilot delivery; record those observations rather than calling them done.

## 2026-08-14 — G2 SB-PLT-001: Crossplot, histogram, Pickett, correlation, Vega and session binding custody

- [ ] **Automated correctness:** the one owned contract test is green and the full gate is
      1023 passed / 0 failed / 37 ignored. It saves and reloads one project plot state and one
      named template with two wells resolving the same semantic request to different concrete
      curve IDs, units, conversions, sample counts, reasons and source revisions. A missing
      required well/channel refuses both save and export and writes no invalid plot document.
- [ ] **Visual:** create multiwell crossplot, histogram, Pickett and correlation panels where one
      semantic channel resolves to different concrete curves across two wells, plus a Vega panel.
      Save project properties, a named template and a named session; reopen each; export SVG, PNG
      and PDF where offered. Confirm the visible curves remain the intended ones and every action
      either succeeds with exact binding custody or names the unresolved required channel.
- [ ] **Manual:** inspect the reopened/exported artifact against the source curve inventory. A
      similar-looking mnemonic is not enough: confirm well, curve ID, source/display unit,
      conversion, sample count, resolution reason and revision. Record the outcome here; automated
      JSON round trips do not close this checkbox.
- [ ] **Field and harsh critique:** repeat the save/reopen/export path on the representative Gate 4
      corpus. The implementation now refuses missing bindings, but a green serialization test does
      not prove that every real imported alias, multi-set well or long-running context fetch resolves
      to the geoscientist's intended curve.

## 2026-08-14 — G2 SB-DIO-063: every Python sidecar now owns the same Unicode byte boundary

- [ ] **Automated implementation:** exact SB-DIO-T96 was deliberately run and passed 1 / 0 / 0.
      The production DLIS runner now receives its path as UTF-8 JSON over piped byte stdin, and
      the one named test proves the path plus source-well payload through DLIS, Word and Pillow.
- [ ] **Ignored-test custody:** the test remains `#[ignore]` because its subject needs numpy,
      python-docx and Pillow. It is inventoried as `OPTIONAL-PACKAGE`, raises the default ignored
      count to 37, and must never be described as a default-gate pass.
- [ ] **Visual / Manual:** import or export one representative Unicode path through each enabled
      DLIS, Word and image action. Confirm the displayed path, created artifact and source label
      remain exact; a successful row or byte count alone does not expose mojibake.
- [ ] **Gate 3 / Field and harsh critique:** qualify the complete offline Python pack on a clean
      Windows machine and repeat the three actions. Passing on this development machine proves the
      byte contract, not that every packaged interpreter and dependency combination is deployable.

## 2026-08-14 — G2 SB-DIO-062: the named encodings pass but Windows code-page selection is undefined

- [ ] **Automated correctness:** exact SB-DIO-T95 is green at 1 passed / 0 failed / 0 ignored.
      It imports and reports UTF-8, UTF-16LE/BE with and without BOM, and one Windows-1252
      adverse-byte control through the real LAS import result.
- [ ] **Source decision:** publish the supported Windows single-byte code-page inventory and the
      deterministic evidence or user-decision rule used to select among them. The current decoder
      labels every non-UTF-8, non-UTF-16 byte stream `Windows-1252`; the chapter's plural
      “code pages” contract does not authorize that assumption or name the other pages.
- [ ] **Visual / Manual:** import the same representative text delivery in each approved encoding.
      Confirm the visible result names the selected encoding and preserves a distinguishing
      non-ASCII character; a successful row count alone cannot expose mojibake in descriptive text.
- [ ] **Field and harsh critique:** Jauhar must confirm the supported pilot-origin tool/export
      inventory before this closes. Calling all legacy Windows bytes CP1252 is convenient and often
      plausible, which is exactly why it can ship silent character corruption without a red gate.

## 2026-08-14 — G2 SB-DIO-061: malformed-corpus proof remains diagnostic, inventory and memory-bound blocked

- [ ] **Automated implementation:** BLOCKED-SOURCE. A focused RED probe made the existing
      cross-reader matrix inspect the errors it currently discards; 23 reader/fixture failures
      omitted the fixture filename. T92 then checks only selected LAS and delimited paths, so the
      combined green test does not prove the universal diagnostic contract in T91-T94.
- [ ] **Inventory decision:** publish one authoritative full reader inventory covering the chapter's
      LAS, delimited, DLIS, image and workbook boundaries. The current source-derived guard scans
      only `parsers.rs` and `intake.rs`, so adding a reader in another owned module need not fail the
      build until it is registered.
- [ ] **Source decision:** supply a cited maximum import size, or approve a bounded streaming design
      that preserves the mandatory universal encoding boundary. `read_text_file_with_encoding`
      currently allocates the complete delivery, so T91's no-unbounded-allocation clause cannot be
      proved truthfully by a timeout alone and no plausible byte cap may be invented.
- [ ] **Visual / Manual / Field and harsh critique:** after both contracts exist, exercise one
      malformed delivery from each pilot reader family and confirm the refusal names the artifact,
      line or record, failed rule and affected count. A test that merely survives malformed bytes
      while throwing away its errors is a false-green safety claim, not malformed-input custody.

## 2026-08-14 — G2 SB-DIO-060: BIFF5 routing remains blocked by the deferred table reader

- [ ] **Automated implementation:** BLOCKED-DEPENDENCY. T90 genuinely opens a delimited table
      named `.las` through Intake and reports the disagreement. T89 only calls the signature
      detector: it never reads the BIFF5 stream as chapter §6 requires, while the ordinary Intake
      probe continues into the text-table reader after identifying binary BIFF5 content.
- [ ] **Scope decision:** decide whether to promote deferred `SB-DIO-059` into the pilot. Its
      published-specification cell-record reader is the missing route needed to make T89 truthful;
      this increment does not silently pull that one row out of the 689-row post-pilot backlog.
- [ ] **Visual / Manual:** after the reader is promoted and implemented, open one licence-safe
      headerless BIFF5 table named `.xls` and one delimited table named `.las`. Confirm both are
      read by content and each disagreement/structural choice is visible before commit.
- [ ] **Field and harsh critique:** a detector that prints `BIFF5` while handing the bytes to a
      text reader is more dangerous than an explicit unsupported-format refusal: its green unit
      test can make a silent empty or corrupt import look like working format routing.

## 2026-08-14 — G2 SB-DIO-057: log-scale zero handling remains family-registry blocked

- [ ] **Automated implementation:** BLOCKED-SOURCE, so no SB-DIO-T84/T85 is written by treating
      familiar gas, resistivity or permeability mnemonics as an authoritative family registry.
      Current imports can commit exact zeros without a pre-commit logarithmic-family decision.
- [ ] **Source decision:** publish the ENV-reviewed, versioned classification of which exact curve
      families are logarithmic, including its source and alias-resolution boundary. Chapter §5.6
      and §7.1 O-5 deliberately leave that classification ABSENT; UI log-axis choices and current
      display defaults are not import semantics.
- [ ] **Visual / Manual:** after the registry exists, import T84's gas control with 200 exact zeros
      among 4,000 samples. Confirm the pre-commit surface reports 200 without rewriting any value,
      and confirm declining conversion commits all zeros as values with a durable decision record.
- [ ] **Field and harsh critique:** Jauhar must exercise one representative logarithmic pilot curve
      and one genuinely linear zero-bearing control. Until family membership is sourced, either
      test can be made green by a mnemonic guess that silently corrupts the other class.

## 2026-08-14 — G2 SB-DIO-056: whole-index LAS STEP remains source-blocked

- [ ] **Automated implementation:** BLOCKED-SOURCE, so no SB-DIO-T82/T83 is written against an
      invented comparison tolerance. Current `export.rs` still declares the first adjacent
      interval as `STEP` without checking the remainder; uniform export round trips do not expose
      that silent misdeclaration.
- [ ] **Source decision:** supply a cited whole-index STEP-agreement tolerance, or explicitly adopt
      exact equality as the contract. T82's `0.1524 m` input and T83's `0.1 m` then `0.15 m` input
      specify controls, not the boundary between them. Git history and a familiar floating-point
      epsilon are not sources.
- [ ] **Visual / Manual:** after the source contract exists and T82/T83 pass, export one uniform
      and one deliberately irregular native-depth frame. Confirm the former declares its verified
      step and the latter declares `STEP = 0` while preserving every explicit depth row.
- [ ] **Field and harsh critique:** Jauhar must reopen both artifacts in a second LAS reader and
      confirm it does not reconstruct an irregular index as uniform. Until the tolerance is
      sourced, a green test would defend a number we guessed rather than prove the requirement.

## 2026-08-14 — G2 SB-DIO-055: LAS export accounts for every held curve

- [ ] **Automated correctness — not manual evidence:** exact SB-DIO-T80/T81 is green at
      1 passed / 0 failed / 0 ignored. The test reopens the recipient-facing LAS, verifies all
      forty imported curve columns and their own supplied samples, independently derives 46
      written of 48 held, matches both omission records across the result and `~O`, and guards
      the ribbon rendering of counts, identities and reasons.
- [ ] **Many-curve completeness:** export a representative pilot well carrying substantially more
      than the six standard curves. Compare Curve Catalog with the reopened LAS and confirm every
      held identity is either a real column with the expected samples or appears in the omission
      inventory. A mnemonic appearing only in provenance text is not a written curve.
- [ ] **Same omission on both surfaces:** deliberately include one exact-mnemonic collision and
      one curve on a different native depth frame. Confirm the status message and the LAS `~O`
      records name the same set/run-qualified identities and the same reasons, and that the summary
      reports the exact `written of held` counts.
- [ ] **Visual / Manual / Field and harsh critique:** Jauhar still needs to inspect a real export
      in SandiBumi and a second LAS reader. Forty synthetic curves prove enumeration and sample
      custody; they do not prove that a pilot delivery's curve naming, omission explanation or
      recipient workflow is intelligible and accepted.

## 2026-08-14 — G2 SB-DIO-054: skipped import items are named, counted and refused when all are lost

- [ ] **Automated correctness — not manual evidence:** exact SB-DIO-T77/T78/T79 is green at
      1 passed / 0 failed / 0 ignored; the full DLIS module is green at 8 passed / 0 failed /
      1 optional-package test ignored. The strengthened contract executes the production
      row/curve accounting path, stores the readable control curve, inspects the failed result,
      and drives both LAS readers. Source-text markers remain only for the optional Python runner.
- [ ] **Partial DLIS outcome:** import a representative DLIS containing at least one readable
      scalar channel and one unreadable frame or channel. Confirm the result says `partial`, the
      readable curve and samples are present, and every omission displays its kind, source name,
      count and rule. A success toast without the omission inventory is not acceptance.
- [ ] **All-skipped refusal and first malformed LAS row:** exercise a DLIS whose candidate frames
      all fail and confirm the import fails with the complete skip inventory and writes no curve.
      Separately import an unwrapped LAS with several rows shorter than `~C`; confirm the first
      offending source line is named. Re-run with a valid wrapped LAS so strictness does not erase
      declared `WRAP.YES` support.
- [ ] **Visual / Manual / Field and harsh critique:** Jauhar still needs to exercise representative
      malformed pilot deliveries in the desktop application and compare every displayed omission
      with the source artifact. Synthetic frames prove the automated boundary; they do not prove
      that a vendor-specific DLIS failure is named intelligibly or accepted by an operator.

## 2026-08-14 — G2 SB-DIO-053: LAS source-header mapping prevents silent identity invention

- [ ] **Automated correctness — not manual evidence:** exact SB-DIO-T75 and SB-DIO-T76 are
      independently green at 1 passed / 0 failed / 0 ignored each. T75 exercises both LAS reader
      variants and the successful import result; T76 exercises the committed import path rather
      than asserting only an internal parser value.
- [ ] **Verbatim preservation and explicit mapping:** import a LAS carrying a documented `COUNT`
      record plus an unfamiliar `~W` mnemonic with distinctive spacing, value and description.
      Confirm Process History names `COUNT → country` and reproduces the unfamiliar source line
      exactly. An unknown mnemonic must remain unmapped; a raw dump without the cited mapping is
      also incomplete.
- [ ] **No identity invention / harsh critique:** import a file whose name looks like it could be
      a UWI, field, operator or country while its `~W` block states only `WELL` and `NULL`. Confirm
      the returned header inventory contains exactly those source mnemonics and no filename-derived
      record. Field and operator mnemonic mappings remain absent because the chapter cites none;
      adding familiar aliases would be invented metadata, not helpful automation.
- [ ] **Visual / Manual / Field:** Jauhar still needs to import a representative pilot LAS with at
      least one uncommon well-header record, inspect the visible Process History mapping and raw
      line, and compare both with the source file. Synthetic headers prove the automated boundary;
      they do not prove the pilot delivery's vendor vocabulary or operator acceptance.

## 2026-08-14 — G2 SB-DIO-052: LAS export marks working and final curve identities

- [ ] **Automated correctness — not manual evidence:** exact SB-DIO-T74 is green at
      1 passed / 0 failed / 0 ignored. This is a RETAIN increment: the production path is
      unchanged, while the existing proof now verifies the two parsed sample arrays as well as
      the two in-file state records.
- [ ] **Both identities and both datasets:** export a well holding working `PHIE` values and
      different final `PHIE` values. Confirm both columns remain in the LAS, neither is listed as
      omitted, and each parsed column contains its own source samples rather than a duplicated or
      renamed copy of the other.
- [ ] **In-file status / harsh critique:** inspect `SANDIBUMI_CURVE_STATE_V1` records in `~O` and
      confirm the exported mnemonic, source mnemonic, set name and `working`/`final` state agree.
      A `_FINAL` suffix alone is not proof if the data came from the working curve, and an internal
      result object is not evidence for the recipient-facing file.
- [ ] **Visual / Manual / Field:** Jauhar still needs to export a representative pilot well with
      deliberately different working and final curves, inspect both tracks after reopening the
      LAS, and confirm a recipient can identify the intended final curve without SandiBumi open.
      Synthetic paired values are not operator or field acceptance.

## 2026-08-14 — G2 SB-DIO-051: LAS deliverables carry complete curve provenance

- [ ] **Automated correctness — not manual evidence:** the one named SB-DIO-T71/T72/T73
      contract is green at 1 passed / 0 failed / 0 ignored. It inspects the completed LAS text,
      not an internal return value, and pins measured-only, computed, model-derived and refusal
      paths from both sides.
- [ ] **Measured and computed records:** export a measured-only well and confirm every written
      curve is named `measured` inside `~O`. Export a computed curve and compare its method plus
      the complete parameter/value object in `~O` with the stored run record; a selected subset
      is not sufficient.
- [ ] **Saved-model record and refusal:** export a model-derived curve and compare the complete
      saved-model record plus artifact SHA-256 in `~O` with the stored model. Remove or otherwise
      make that cited model unavailable in a controlled diagnostic; export must refuse, naming
      both the curve and model, rather than emitting incomplete provenance.
- [ ] **Identity-conflict refusal / harsh critique:** a stored computed curve that shadows a
      measured standard mnemonic such as `GR` must refuse LAS export. Two same-name columns with
      two origins are not provenance merely because both are parseable JSON; accepting them would
      turn a contradictory audit trail into a successfully self-checked deliverable.
- [ ] **Visual / Manual / Field:** Jauhar still needs to export a representative measured,
      deterministic-computed and saved-model-derived pilot deliverable, inspect `~O` in the real
      file, and confirm a recipient can trace every delivered curve. Synthetic fixtures and a
      green parser round trip are not operator or field acceptance.

## 2026-08-14 — G2 SB-DIO-050: LAS import flags a declared STEP mismatch

- [ ] **Automated correctness — not manual evidence:** exact SB-DIO-T70 is green at
      1 passed / 0 failed / 0 ignored. This is a RETAIN increment: current production behavior
      was reverified rather than rewritten.
- [ ] **Mismatch and matching sides:** import a LAS declaring `STEP.M 0.5` with 1 m source-depth
      intervals and confirm the warning says `possibly re-gridded`, names both intervals, and
      locates the first row pair. Repeat with `STEP.M 1.0`; no re-grid warning may appear.
- [ ] **False-positive guards:** a deep LAS whose original decimal tokens agree at 0.15240 m must
      not be flagged merely because f32 storage rounds them differently, and a missing index row
      must break adjacency rather than create a comparison between non-neighbours.
- [ ] **Scope boundary / harsh critique:** no source supplies a universal suspicious-round-interval
      threshold or the acquisition's expected step. Keep that detector absent; do not turn a neat
      number into evidence of resampling, and do not upgrade `possibly re-gridded` into a factual
      provenance claim.
- [ ] **Visual / Manual / Field:** Jauhar still needs to import a representative known re-gridded
      delivery and its matching control, read the warning in the application, and independently
      compare the file's `STEP` with adjacent source depths. Synthetic LAS fixtures are not field
      acceptance.

## 2026-08-14 — G2 SB-DIO-049: pilot LAS export must pass its own reader

- [ ] **Automated correctness — not manual evidence:** exact SB-DIO-T68 and SB-DIO-T69 are
      independently green at 1 passed / 0 failed / 0 ignored each. This is a RETAIN increment:
      current production behavior was reverified rather than rewritten.
- [ ] **Success and refusal:** export one representative LAS and confirm success says the registered
      SandiBumi reader self-check passed. In a focused diagnostic, corrupt an ASCII row and then
      misdeclare a feet index as metres; both must return an actionable `LAS self-check failed`
      error before success, never a warning.
- [ ] **Scope boundary / harsh critique:** this closes the only registered DIO data writer in the
      approved LAS/delimited pilot surface. It does not prove Office, report, plot, browser-CSV,
      model, backup, or other artifact writers; the chapter's product-wide E-3 remains open. Do not
      market the registry proof as universal file-output qualification.
- [ ] **Visual / Manual / Field:** Jauhar still needs to export a representative feet-based pilot
      LAS, inspect the visible self-check result, re-open the artifact independently, and compare
      declared unit, row count, curve count, and values. Synthetic refusal controls are not operator
      or field acceptance.

## 2026-08-14 — G2 SB-DIO-048: LAS container identity outranks the filename

- [ ] **Automated correctness — not manual evidence:** exact SB-DIO-T67 is green at
      1 passed / 0 failed / 0 ignored. The full ingest module is green at 54 passed /
      0 failed / 1 ignored, and the registered malformed-reader corpus is green at
      1 passed / 0 failed / 0 ignored.
- [ ] **Two-sided identity boundary:** a colonless container `WELL` value wins even when an
      exact-path confirmation disagrees, and the filename proposal is suppressed. When the
      container has no `WELL` value, preflight exposes the stem only as a proposal, import
      refuses and writes no well until an explicit non-empty confirmation is supplied.
- [ ] **Visual / Manual / Field:** Jauhar still needs to open the LAS import dialog with one
      container-identified file and one identity-absent file, confirm the former says the
      filename is unused, edit and approve the latter's proposed identity, then inspect the
      created records after reload. Compiled TypeScript and synthetic fixtures are not operator
      or field acceptance.

## 2026-08-14 — G2 SB-DIO-047: Precision reduction is stated at import and LAS export

- [ ] **Automated correctness — not manual evidence:** exact SB-DIO-T66 is green at
      1 passed / 0 failed / 0 ignored. This is a RETAIN increment: current production behavior
      was reverified rather than rewritten.
- [ ] **Both precision boundaries:** core-point import names `f64 numeric parse → f32 storage`,
      counts the one value that genuinely changes, and does not falsely count exact values. LAS
      export separately names `f32 storage → fixed-decimal-4 LAS text`, counts only the rounded
      sample, returns the report, and embeds the declaration in the deliverable.
- [ ] **Visual / Manual / Field:** Jauhar still needs to run a representative high-precision core
      import and LAS export, read both UI messages, compare source/stored/exported values, and
      confirm the recipient-facing file declaration is understandable. Synthetic proof is not
      operator or field acceptance.

## 2026-08-14 — G2 SB-DIO-044: LAS section strictness is one reported policy

- [ ] **Automated correctness — not manual evidence:** exact SB-DIO-T62 is green at
      1 passed / 0 failed / 0 ignored. The same state machine handles both parser entry
      points and the LAS 2.0 / 3.0 controls; a version-specific warning path cannot satisfy it.
- [ ] **Accepted and refused sides:** unknown `~X`, malformed `~`, and an out-of-order
      recognized `~WELL` are accepted only with ordered structured handling records and an
      import warning. Missing or nonnumeric `~V`, and missing `~W` before `~A`, refuse in
      both parser paths. Existing exporter self-check fixtures now declare `~W` while retaining
      their original row-width and unit-lie assertions. No supported-version range or adjacent
      LAS capability was invented.
- [ ] **Visual / Manual / Field:** Jauhar still needs to import representative tolerated and
      refused deliveries, inspect the policy identity plus handling in the application, and
      confirm an operator can act on the warning/error. Synthetic fixtures are not field proof.

## 2026-08-14 — G2 SB-DIO-040: Wrapped LAS Import stays aligned and LAS Export stays unwrapped

- [ ] **Automated characterization — not manual evidence:** exact SB-DIO-T58 is green at
      1 passed / 0 failed / 0 ignored. The 30-curve `WRAP.YES` delivery enters the real importer;
      all 60 uniquely identifiable samples are queried from their stored curve identities.
- [ ] **Writer control:** export the imported object through the registered default writer, confirm
      `WRAP.NO`, exactly two physical data lines, and depth plus every emitted curve on each line.
      A three-column parser helper or source-text search alone does not close this contract.
- [ ] **Visual / Manual / Field:** Jauhar still needs to import a representative wide wrapped LAS,
      inspect several early/middle/late curves, export it, and open the result independently. The
      synthetic 30-curve fixture is CHARACTERIZATION, not field evidence.

## 2026-08-14 — G2 SB-DIO-003: LAS Import distinguishes NoNull from unset data conventions

- [ ] **Automated correctness — not manual evidence:** exact SB-DIO-T04 and SB-DIO-T05 are
      independently green at 1 passed / 0 failed / 0 ignored each. Both enter the real LAS import,
      inspect the typed per-channel result and query the stored sample; parser helpers alone do not
      close this contract.
- [ ] **Two-sided control:** declare `PWF1` as `NoNull` and confirm the cited genuine `-999.25`
      amplitude remains finite with result mode `no_null`; repeat without a channel declaration and
      confirm ordinary screening writes internal `f32::NAN` with result mode `unset`.
- [ ] **Visual / Manual / Field:** Jauhar still needs to import both representative deliveries,
      inspect the resolved channel-null records returned by the application, and independently
      verify the preserved versus missing sample after reload. Synthetic fixtures are not field evidence.

## 2026-08-14 — G2 SB-DIO-034: Curve Catalog Workflow and log view silently choose family members

- [ ] **Automated contract:** BLOCKED. Exact SB-DIO-T50 does not exist, and the current universal
      read contract fails: equation and Workflow readers accept a family match and return only its
      values under the request key. A well with two same-family curves therefore has a concrete
      choice that is neither stated nor asked.
- **Read-only implementation evidence — not acceptance:** plotting's semantic resolver records the
      concrete mnemonic and resolution reason, but `fetch_generic_curve_aligned` and related frame
      readers do not. Existing family-fallback tests prove useful behavior such as `HDRA -> DRHO`;
      they do not prove the required choice disclosure across every read path.
- [ ] **Decision / architecture:** DEC-030 already owns the shared dependency. Engineering
      recommends typed `EXACT_MNEMONIC` and `SEMANTIC_FAMILY` requests; only the latter may choose a
      member, and it must return concrete mnemonic, set/curve identity and resolution rule. Every
      caller must be classified rather than guessed.
- [ ] **Visual / Manual / Field:** unavailable until the typed contract and all-resolver T50 exist.
      Later review must present two same-family curves and show that each consuming surface asks for
      or states the concrete choice. A green family-ranking unit test is not operator evidence.

## 2026-08-14 — G2 SB-DIO-033: Reframe curve selection is named saved and inspectable

- [ ] **Automated selection correctness — not manual evidence:** exact SB-DIO-T49 is green at
      1 passed / 0 failed / 0 ignored. Saving and reloading normalizes the object name, preserves
      the ordered exact `RHOB`, `GR` membership, removes only the repeated member, and returns the
      explicit `Selected` mode.
- [ ] **Hidden-default refusal:** a document that lists members without declaring its mode cannot
      deserialize as a curve selection. A type-implied, blank-means-all or storage-order selection
      therefore cannot satisfy the same contract merely by returning a list of names.
- [ ] **Visual / Manual / Field:** Jauhar still needs to save a representative Reframe selection,
      close and reopen the project, inspect the name, mode and ordered members, then confirm the run
      consumes that saved object. The in-memory round trip is not operator or project-reload evidence.

## 2026-08-14 — G2 SB-DIO-032: Reframe substitution stays explicit and provenance-bearing

- [ ] **Automated substitution correctness — not manual evidence:** exact SB-DIO-T48 is green
      at 1 passed / 0 failed / 0 ignored. A named substitute is accepted only for an unavailable
      explicitly requested curve; the opposite case refuses before any log set or curve is written.
- [ ] **Identity and ancestry proof:** the accepted run writes the substitute under its own
      mnemonic and persists the exact requested-to-substitute decision in the resulting log-set
      ancestry. Merely relabelling bytes or logging a decision without applying it cannot pass both
      sides of the same test.
- [ ] **Visual / Manual / Field:** Jauhar still needs to exercise a representative Reframe run,
      inspect the named offer before accepting it, and confirm the output identity and processing
      ancestry remain understandable after reload. Synthetic in-memory evidence is not operator or
      pilot-corpus evidence.

## 2026-08-14 — G2 SB-DIO-031: Curve Catalog Workflow and log view need exact-name refusal

- [ ] **Automated contract:** BLOCKED. Exact SB-DIO-T47 does not exist, and no current
      universal proof can pass while the generic resolver accepts `mnemonic = request OR
      family = request`. A requested exact-looking key can therefore receive another curve's
      bytes. The unchanged full gate remains 1015 passed / 0 failed / 36 ignored with 55 owned
      Rust warnings; that green result does not prove this missing MUST NOT.
- **Read-only implementation evidence — not acceptance:** equations readers and sampling
      diagnostics share the family fallback; workflows intentionally depend on cases such as
      `HDRA -> DRHO` and `HCAL -> CALI`. Plotting at least returns the concrete mnemonic and
      resolution reason, while Reframe's explicit accepted-substitute path keeps the substitute's
      own name. The application therefore has two legitimate intents hidden behind one string.
- [ ] **Decision / architecture:** DEC-030 needs Jauhar to approve an explicit request-type split.
      Engineering recommends `EXACT_MNEMONIC`, which never falls back, and `SEMANTIC_FAMILY`,
      which may resolve a member only while returning its concrete identity and resolution rule.
      Existing callers must be classified by intent; guessing would either preserve silent
      substitution or break deliberate family workflows.
- [ ] **Visual / Manual / Field:** unavailable until the split and exact T47 inventory exist.
      Later review must show an exact miss as unavailable and a semantic match under its real
      mnemonic in every consuming surface. A family-fallback unit test is not operator evidence.

## 2026-08-14 — G2 SB-DIO-030: LAS alias rename keeps source identity in the Curve Catalog

- [ ] **Automated rename correctness — not manual evidence:** exact SB-DIO-T46 is green at
      1 passed / 0 failed / 0 ignored. Importing `SGR` produces a public decision and visible
      note containing original `SGR`, applied target `GR`, and exact firing row
      `GR_ALIASES: SGR -> GR`; a silent or source-less rename cannot pass.
- [ ] **Two-store identity proof:** standard GR receives the delivered `71.0` sample while the
      generic catalog still names the curve `SGR` beside applied family `GR`. Merely logging a
      rename without applying it, or applying it by destroying the source mnemonic, fails on
      opposite sides of the same test.
- [ ] **Visual / Manual / Field:** Jauhar still needs to import a representative aliased LAS,
      inspect the import note and Curve Catalog identity, and verify the standard track uses the
      intended target. Synthetic source/target custody is not operator or interoperability evidence.

## 2026-08-14 — G2 SB-DIO-029: LAS MS/FT unit decisions remain file-scoped in the Curve Catalog

- [ ] **Automated no-default correctness — not manual evidence:** exact SB-DIO-T45 is green
      at 1 passed / 0 failed / 0 ignored. An unanswered `MS/FT` delivery is refused before
      any well commits, and the result names both legitimate quantities plus the required
      per-file decision; a silent sonic default cannot pass.
- [ ] **Two-file scope proof:** one batch assigns `microseconds_per_foot` to one source path
      and `millisiemens_per_foot` to another. Their separate public designation records and
      stored curves must respectively become `DT/us/ft` and familyless `MS/FT`; a cached or
      batch-wide answer would corrupt one side and fail the test.
- [ ] **Visual / Manual / Field:** Jauhar still needs to import representative ambiguous-unit
      deliveries, confirm that the dialog asks separately for each file, and inspect both the
      visible decision record and resulting Curve Catalog identity. Synthetic two-file evidence
      is not operator or interoperability evidence.

## 2026-08-14 — G2 SB-DIO-028: every shipped unit factor is independently auditable

- [ ] **Automated arithmetic correctness — not manual evidence:** exact SB-DIO-T44 is green
      at 1 passed / 0 failed / 0 ignored. Its independent table enumerates all ten generated
      transforms and checks family binding, factor, affine offset and automatic-versus-confirmed
      status against the cited exact unit identities; a mutually wrong factor and explanation fail.
- [ ] **Automated derivation custody:** every runtime row must expose the independently required
      arithmetic terms, including `25.4 mm/in`, `0.3048 m/ft`, the Fahrenheit offset and scale,
      and `10^3 mL/L`; a blank, vendor-only or numerically disconnected derivation fails.
- [ ] **Visual / Manual / Field:** Jauhar still needs to inspect representative imported
      conversions and their visible audit records, including the confirmation-only QV case.
      Generated-table correctness is not proof that operators can understand the report or that
      representative deliveries use the declared units honestly.

## 2026-08-14 — G2 SB-DIO-027: LAS unit alias rejection keeps Curve Catalog and standard density honest

- [ ] **Automated rejection correctness — not manual evidence:** exact SB-DIO-T43 is green
      at 1 passed / 0 failed / 0 ignored. A delivered `RHOZ.PPG` channel remains familyless,
      keeps unit `PPG` and source value `9.5` in the generic store, and does not populate the
      standard RHOB channel, which remains `NaN`.
- [ ] **Automated designation evidence:** the same public import result names the rejected
      `density.units: PPG -> density` entry, marks quantity designation required, and exposes a
      warning containing `PPG`, the recorded pressure-gradient conflict and designation. A silent
      family binding or a destructive refusal that loses the source data cannot pass.
- [ ] **Visual / Manual / Field:** Jauhar still needs to import a representative ambiguous-unit
      delivery, inspect the warning and retained Curve Catalog row, and confirm that no standard
      RHOB trace appears until an explicit quantity decision is made. Synthetic LAS evidence is
      not operator, interoperability or representative-delivery evidence.

## 2026-08-14 — G2 SB-DIO-026: LAS affine unit conversion prevents a silent Curve Catalog shortcut

- [ ] **Automated affine correctness — not manual evidence:** exact SB-DIO-T42 is green at
      1 passed / 0 failed / 0 ignored. A `FTEMP.DEGF` import records factor `1/1.8` and
      source-space offset `-32`, stores family TEMP with canonical unit `DEGC`, and maps the
      chapter's `200 °F` input to `93.333… °C`.
- [ ] **Two-sided offset proof:** the same test maps the independently checkable fixed point
      `32 °F` to `0 °C` and rejects the factor-only `111.111… °C` answer for `200 °F`.
      A multiplicative field disguised as an affine transform cannot satisfy both controls.
- [ ] **Visual / Manual / Field:** Jauhar still needs to import a representative Fahrenheit
      temperature curve, inspect the visible factor/offset audit and Curve Catalog unit, and
      compare the stored curve in the log view. The synthetic LAS proof is not operator,
      interoperability or representative-delivery evidence.

## 2026-08-14 — G2 SB-DIO-025: LAS unit coverage and Curve Catalog pass-throughs are not silent

- [ ] **Automated query correctness — not manual evidence:** exact SB-DIO-T41 is green at
      1 passed / 0 failed / 0 ignored. The shipping Tauri command returns exactly CALI, BS,
      RHOB, DRHO, NPHI, DT, DTS and TEMP; the same test pins command registration and the
      typed frontend invoke route, so an internal list with no product query cannot pass.
- [ ] **Automated unsupported-unit custody — not manual evidence:** exact SB-DIO-T40 is
      green at 1 passed / 0 failed / 0 ignored. `RHOZ.FURLONGS` creates no conversion record,
      retains its declared unit and `2400` source sample in the generic store, and produces a
      visible unconverted warning naming both the curve and unit. Silent canonical relabelling
      and a warning detached from changed data fail independently.
- [ ] **Visual / Manual / Field:** Jauhar still needs to import a representative LAS carrying
      an unsupported declared unit, inspect the visible result and Curve Catalog custody, and
      judge the warning wording. The synthetic import and registered IPC route are not operator,
      interoperability or representative-delivery evidence.

## 2026-08-14 — G2 SB-DIO-024: every automatic conversion is visible

- [ ] **Automated correctness — not manual evidence:** exact SB-DIO-T39 is green at
      1 passed / 0 failed / 0 ignored. One import converts independent DTCO and DTSM
      curves from `US/M` to `us/ft`; both public conversion records and the visible note
      name curve, source unit, destination unit and the cited `0.3048` factor.
- [ ] **Two-sided storage proof:** the stored first samples are independently derived as
      `100 × 0.3048 = 30.48` and `150 × 0.3048 = 45.72`. A plausible report attached
      to unchanged values, or reporting only the first converted curve, cannot pass.
- [ ] **Visual / Manual / Field:** Jauhar still needs to import representative convertible
      LAS and DLIS deliveries, judge the result wording and inspect the stored curves.
      Automated synthetic LAS evidence is not representative-format or operator evidence.

## 2026-08-14 — G2 SB-DIO-022: every writer keeps stored samples at defaults

- [ ] **Automated correctness — not manual evidence:** exact SB-DIO-T35 is green at
      1 passed / 0 failed / 0 ignored. Every registered writer receives an irregular,
      non-linear database fixture and must emit the exact stored depths and paired GR
      values; a regularized index or interpolated values fail independently.
- [ ] **Registry boundary:** a new writer cannot inherit this result merely because LAS
      passes. T35 iterates the writer registry and refuses an unadapted output format by
      name, so each format must expose its written samples to the same proof.
- [ ] **Visual / Manual / Field:** Jauhar still needs to export a representative irregular
      well through the UI and compare the delivered file in an independent reader. No
      writer-side resample control ships; if one is proposed later, its naming, default-off
      state and provenance require a separate increment before implementation.

## 2026-08-14 — G2 SB-DIO-021: reads preserve native sampling by default

- [ ] **Automated characterization — not correctness or manual evidence:** exact
      SB-DIO-T34 is green at 1 passed / 0 failed / 0 ignored. Every source-registered
      file reader is classified; every sampled reader preserves the delivered
      `1000.0, 1000.1, 1000.3 m` index, so a hidden regularizer that invents the
      missing `1000.2 m` station cannot pass.
- [ ] **Automated storage boundary:** the shipping LAS, delimited core and WIDE-array
      import paths store those same three depths and create no `OWN` Reframe set at
      defaults. This characterizes the current product; it is not representative-file
      or operator evidence.
- [ ] **Visual / Manual / Field:** Jauhar still needs to import representative native-grid
      LAS, core and array deliveries, inspect their sample counts/depths, then run an
      explicitly chosen Reframe and judge its operation naming. Only Jauhar records that
      operator and field evidence.

## 2026-08-14 — G2 SB-DIO-020: duplicate depths require a declared policy

- [ ] **Automated no-decision proof — not manual evidence:** exact SB-DIO-T33 is
      green at 1 passed / 0 failed / 0 ignored. A LAS with three repeated depth rows
      names the count and missing policy, and commits zero wells.
- [ ] **Automated policy proof — not manual evidence:** exact SB-DIO-T32 is green at
      1 passed / 0 failed / 0 ignored. `keep-first` reports three affected rows and
      retains the first GR and generic PEF samples on the same two stored depths;
      explicit `refuse` also commits nothing. Supporting keep-last and mean checks keep
      independent standard and generic companion columns aligned.
- [ ] **Visual / Manual / Field:** Jauhar still needs to exercise the duplicate-depth
      decision in the LAS import UI, judge the count and policy wording, and inspect a
      representative run-splice result. Automated synthetic imports are not operator or
      field evidence.

## 2026-08-14 — G2 SB-DIO-019: committed depths cannot be re-declared

- [ ] **Automated custody proof — not manual evidence:** exact SB-DIO-T31 is green at
      1 passed / 0 failed / 0 ignored. Reasserting the stored metre unit remains a safe
      no-op; changing to feet is refused, names the one affected well and explains that
      re-declaration would reinterpret rather than convert the stored depths.
- [ ] **Two-sided persistence boundary:** after refusal, the project declaration is still
      metres and every packed depth byte matches the pre-attempt database snapshot. The
      check does not infer a conversion or treat an in-memory fixture as post-write custody.
- [ ] **Visual / Manual / Field:** Jauhar still needs to attempt the same change through
      Data Conventions in a representative populated project, confirm the message is clear,
      and reopen/replot the project. The automated database proof is not operator or field
      evidence.

## 2026-08-14 — G2 SB-DIO-018: LAS family units have one canonical owner

- [ ] **Automated ownership proof — not manual evidence:** exact SB-DIO-T29 is green at
      1 passed / 0 failed / 0 ignored. Its static check is restricted to production source,
      proves `export.rs` has no writer-owned unit table, and proves the LAS writer calls the
      canonical family registry; the assertion cannot satisfy itself from test-source text.
- [ ] **Automated file-boundary proof — not manual evidence:** exact SB-DIO-T30 is green at
      1 passed / 0 failed / 0 ignored. One exported curve from every registered family declares
      that family's reviewed canonical unit with exact spelling and case.
- [ ] **Visual / Manual / Field:** Jauhar still needs to export representative held curves,
      inspect their LAS unit tokens in another reader, and confirm the UI identifies any omitted
      or unsupported curve clearly. Automated source and synthetic-file checks are not
      interoperability or operator evidence.

## 2026-08-14 — G2 SB-DIO-017: LAS export depth units survive round trips

- [ ] **Correctness — not manual evidence:** exact SB-DIO-T27 is green at 1 passed /
      0 failed / 0 ignored. A feet project writes `FT` on `STRT`, `STOP`, `STEP` and
      `DEPT`, writes no opposite `M` declaration, re-imports as a feet project, and
      returns the original `2000.0` depth unchanged.
- [ ] **Characterization — not correctness:** exact SB-DIO-T28 is separately green at
      1 passed / 0 failed / 0 ignored. The corresponding metre project currently writes
      `M` on all four declarations and round-trips `2000.0`; the chapter deliberately
      labels this metre scenario as characterization. The negative self-check is also
      green and rejects feet-valued output falsely declared as metres.
- [ ] **Visual / Manual / Field:** Jauhar still needs to export representative feet and
      metre projects, inspect all four LAS header declarations in the delivered files,
      re-import them through the UI, and confirm the result messaging is clear. Automated
      round trips are not representative-client or interoperability evidence.

## 2026-08-14 — G2 SB-DIO-015: undeclared LAS depth units refuse

- [ ] **Automated — not manual evidence:** exact SB-DIO-T22 and T23 are separately green
      at 1 passed / 0 failed / 0 ignored each. With neither unit source declared, the
      refusal names both the file index and project and commits zero wells. A metre project
      still refuses an undeclared file and commits zero wells until that import explicitly
      confirms `FT`; the stored `1000 ft` sample is then `304.8 m` by the cited NIST factor.
- [ ] **Characterization — not correctness:** exact SB-DIO-T24 is separately green at
      1 passed / 0 failed / 0 ignored. A file-declared `FT` index is converted to metres and
      the current result text reports `converted from ft`; the numeric conversion is sourced,
      but the chapter deliberately classifies the report wording scenario as characterization.
- [ ] **Visual / Manual / Field:** Jauhar still needs to exercise all three imports in the
      LAS dialog, confirm the two refusals keep the project tree unchanged, and judge whether
      the confirmation and conversion messages are readable and actionable. Automated tests
      do not establish operator comprehension or representative-delivery evidence.

## 2026-08-14 — G2 SB-DIO-014: TVD tops require survey-backed MD correlation

- [ ] **Automated — not manual evidence:** exact SB-DIO-T20 is green at 1 passed /
      0 failed / 0 ignored. A production import of a TVD-only tops table writes the
      delivered `900.0` unchanged with `depth_datum = TVD`; the alias remains accepted,
      and an MD-only editor cannot silently overwrite or delete/recreate that custody.
- [ ] **Reference-frame refusal and conversion:** exact SB-DIO-T21 is green at 1 passed /
      0 failed / 0 ignored. Building MD zones from that top without an active deviation
      survey names TVD, MD and the missing survey and writes zero zones. With the literal
      fixture mapping `900 TVD → 1000 MD`, the resulting MD zone starts at `1000`, not the
      raw `900`.
- [ ] **Visual / Manual / Field:** Jauhar still needs to import a representative TVD-only
      tops delivery, confirm Tops/Log/Correlation show `MD ← TVD` only after a survey is
      active, and confirm the no-survey message remains readable at narrow and wide dock
      widths. No automated assertion is promoted to operator or field evidence.

## 2026-08-14 — G2 SB-DIO-013: core import requires explicit index designation

- [ ] **Automated — not manual evidence:** exact SB-DIO-T19 is green at 1 passed /
      0 failed / 0 ignored. An unresolved `SAMPLE,CPOR` table commits zero
      `core_data` rows without a designation; selecting column zero imports both rows
      and reports `UserDesignation`, the selected index and `SAMPLE` mnemonic.
- [ ] **Positive and negative boundary:** the supporting parser regression is green at
      1 passed / 0 failed / 0 ignored. `Depth (m)`, `DEPTH (FT)` and bare `DEPTH`
      resolve by their qualified or bare aliases, while an unrelated `MEASURE` column
      is not guessed. Explicit designation is therefore required only when structure
      and documented names genuinely fail.
- [ ] **Visual / Manual / Field:** Jauhar still needs to exercise the desktop core and
      delimited-intake designation controls with representative deliveries. These
      automated no-write and resolution assertions are not operator or field evidence.

## 2026-08-14 — G2 SB-DIO-012: a decreasing index is refused before commit

- [ ] **Automated — not manual evidence:** exact SB-DIO-T18 drives the production LAS
      importer with 400 finite, non-duplicated rows whose final depth decreases. It is
      green at 1 passed / 0 failed / 0 ignored: the result names data row 400 and the
      required user decision, and the database still contains zero wells.
- [ ] **Decision control:** the same file imports only after the fixture supplies
      `AcceptAsDelivered`; the public result retains data row 400 in its warning. No
      implicit sorting, duplicate policy, or guessed correction is introduced.
- [ ] **Visual / Manual / Field:** Jauhar still needs to confirm the desktop refusal,
      decision control and accepted-warning readability against a representative
      delivery. The automated database assertion is not that evidence.

## 2026-08-14 — G2 SB-DIO-011: the deviation index aliases have no documented source

- [ ] **Automated contract:** BLOCKED. The exact source-tree audit found four
      index-bearing alias lists, while the passing test named “every” inspected only the
      three lists enumerated in chapter §5.3. It has been renamed as supporting evidence:
      those three declarations cite §5.3, and `TVD` remains outside every MD list. The
      uncited `DEV_MD_ALIASES = [MD, DEPTH, DEPT, MEASURED_DEPTH]` path means no
      qualifying SB-DIO-T17 proof exists. Focused supporting test: 1 passed / 0 failed /
      0 ignored. Full gate: 1007 passed / 0 failed / 36 ignored, including backend 950
      passed / 0 failed / 36 ignored and 55 owned Rust warnings.
- **Source boundary — not acceptance:** the immutable chapter owns deviation-survey
      ingestion but §5.3 names values and sources only for LAS, core-table and tops
      index aliases. Git history proves the deviation list arrived with the baseline; it
      does not supply an authority for those four accepted headers.
- [ ] **Source required:** supply a named, auditable source for the deviation-survey MD
      alias list. Then cite it beside the declaration and make T17 discover every
      index-alias declaration mechanically, so adding a fifth undocumented path fails.
      No historical implementation and no plausible industry spelling becomes a source.
- [ ] **Visual / Manual / Field:** no operator or representative-delivery evidence is
      claimed. A green partial static test is not proof that all import paths have sourced
      namespaces.

## 2026-08-14 — G2 SB-DIO-010: LAS index proof does not prove the structural-reader arm

- [ ] **Automated contract:** BLOCKED. The existing exact test is green at 1 passed / 0
      failed / 0 ignored and its LAS arm drives the production importer, proving that a
      second-column `MD` cannot steal LAS's positionally guaranteed first index and that
      the public result records `positional_guarantee`. Its structural arm supplies
      headers and `REFERENCE | LOG` classes directly to `resolve_index_column`; no
      Geolog flat-ASCII file is read, so it is supporting resolver evidence rather than
      the chapter's file-import T15. Full gate: 1007 passed / 0 failed / 36 ignored,
      including backend 950 passed / 0 failed / 36 ignored and 55 owned Rust warnings.
- **Read-only source evidence — not acceptance:** the cited local Geolog
      `xyz.flat_ascii_format` declares `CLASSES = REFERENCE LOG LOG`, and a second cited
      spec uses the row-oriented form. The shipping tree has no Geolog flat-ASCII reader
      or Tauri import command that consumes either declaration. A test-only spec parser
      would be another helper, not proof of import behavior.
- [ ] **Decision / scope:** DEC-029 must reconcile exact T15's Geolog import with
      DEC-003 and G2-T04's LAS-2/delimited pilot surface. Either authorize the sourced
      Geolog reader as necessary SB-DIO-010 infrastructure or explicitly correct the
      acceptance boundary; engineering will do neither silently.
- [ ] **Visual / Manual / Field:** unavailable for the missing structural-reader arm.
      The current serialized LAS result is automated evidence, not Jauhar's inspection
      of an operator-visible decision and not representative-delivery proof.

## 2026-08-14 — G2 SB-DIO-009: the import result reports the chosen and passed-over aliases

- **Automated — not manual evidence:** exact SB-DIO-T14 drives the production LAS import
      function with the chapter's all-null `NPHIED` and populated `NPHI_LS` fixture. The
      returned public result has one competing-alias decision, names `NPHI_LS` as chosen
      and `NPHIED` as passed over, carries finite counts 2 and 0, and marks both chosen
      states. The Tauri command and TypeScript IPC contract carry that same typed result.
      Full gate: 1007 passed / 0 failed / 36 ignored, including backend 950 passed / 0
      failed / 36 ignored and 55 owned Rust warnings.
- [ ] **Visual:** import the disposable fixture and inspect the desktop result surface; it
      must display the chosen and passed-over mnemonics and both coverage counts, not merely
      receive them over IPC.
- [ ] **Manual:** Jauhar confirms the displayed decision is understandable enough to
      diagnose a wrong heuristic choice. The green backend result is not this usability
      judgment.
- [ ] **Field:** pending Gate 4; no representative delivery with competing aliases has
      verified the operator-facing report.

## 2026-08-14 — G2 SB-DIO-008: alias coverage wins and exact ties follow the declared priority

- **Automated characterization — not correctness or manual evidence:** exact
      SB-DIO-T12/T13 imports an all-null earlier `NPHIED` beside populated `NPHI_LS`,
      then imports an equal-coverage `RESD`/`RES_DEEP` pair whose source-column order is
      deliberately opposite the chapter's declared alias priority. The populated alias
      wins, both repeated tie imports choose `RES_DEEP`, and the test says
      `characterizes_...` because chapter §6 classifies these outputs as characterizations.
      Full gate: 1007 passed / 0 failed / 36 ignored, including backend 950 passed / 0
      failed / 36 ignored and 55 owned Rust warnings.
- [ ] **Visual:** import both disposable fixtures and inspect the alias-decision surface so
      the winner, passed-over columns, finite counts and declared tie priority are readable.
- [ ] **Manual:** Jauhar independently repeats the equal-coverage import and confirms the
      same binding, while recognizing that automated repeatability is not cross-machine
      operator evidence.
- [ ] **Field:** pending Gate 4; no representative delivery with competing aliases has
      established that the declared priority is appropriate for the pilot data.

## 2026-08-14 — G2 SB-DIO-007: empty versus explicitly nulled is blocked on its deliverable representation

- [ ] **Automated contract:** BLOCKED. Exact SB-DIO-T11 requires consecutive empty and
      explicit-sentinel cells to remain absent for arithmetic while an import/export
      round trip distinguishes their source states. No qualifying test was written against
      a representation the chapter does not select.
- **Read-only evidence — not acceptance:** Intake retains raw preview strings and a
      column-level kind, but its import rows reduce both cases to `f32::NAN`; curve storage
      has no per-sample source-state channel, and the LAS writer emits the same project
      sentinel for every NaN.
- [ ] **Decision / architecture:** select and version one compact source-cell-state
      representation, define how each supported deliverable carries or accompanies it, and
      preserve `f32::NAN` as the arithmetic value. A bitset, table column, sidecar, or
      manifest is not chosen by this increment.
- [ ] **Visual / Manual / Field:** unavailable until the representation is adjudicated and
      exact T11 exists. The current Intake preview is not proof that the distinction survives
      storage or export.

## 2026-08-14 — G2 SB-DIO-006: one null-exception entry retains all six name patterns

- **Automated — not manual evidence:** exact SB-DIO-T10 loads the chapter's cited
      one-entry, six-pattern rule shape from its serialized representation, requires the
      loader to keep one rule and all six patterns, resolves every matching source channel,
      excludes an unmatched channel, and pins explicit `NoNull` against the ordinary unset
      screen. Full gate: 1006 passed / 0 failed / 36 ignored, including backend 949 passed /
      0 failed / 36 ignored and 55 owned Rust warnings.
- [ ] **Visual:** load a disposable rule document with six patterns in one entry and inspect
      the import preview/result so all six matched channels are reported without flattening
      or truncation.
- [ ] **Manual:** Jauhar independently confirms a genuine sentinel-shaped amplitude is
      preserved for a matched `NoNull` channel while an unmatched channel follows the LAS
      convention. The serialized automated fixture is not this operator check.
- [ ] **Field:** pending Gate 4; no representative vendor rule document and delivery have
      been exercised together through the installed application.

## 2026-08-14 — G2 SB-DIO-005: plural nulls stay attached to their source channel in every LAS reader

- **Automated — not manual evidence:** exact SB-DIO-T09 imports the chapter's cited
      `-999`, `-999.25`, and `-32767` controls through both shipping LAS readers. It
      requires both nulls declared for one channel to screen there, the other channel's
      distinct null to screen there, and all three values to survive when they belong to
      the other channel. Full gate: 1006 passed / 0 failed / 36 ignored, including backend
      949 passed / 0 failed / 36 ignored and 55 owned Rust warnings.
- [ ] **Visual:** import a disposable LAS with two channels carrying the three cited
      controls; inspect both curve surfaces and confirm gaps appear only under each
      channel's declared convention.
- [ ] **Manual:** Jauhar independently queries both imported source channels and confirms
      their own sentinels became `f32::NAN` while the cross-channel controls remained
      finite and exact. The two-reader automated fixture is not this operator check.
- [ ] **Field:** pending Gate 4; no representative delivery has confirmed that its
      channel-identity and plural-null metadata reach the same reader paths unchanged.

## 2026-08-14 — G2 SB-DIO-004: null recognition has one relative transform and never rewrites

- **Automated — not manual evidence:** exact SB-DIO-T06/T07/T08 recognises the cited
      near-sentinel and f32/f64 representation controls, preserves a finite nonmatch,
      converts a declared near-sentinel only to `f32::NAN`, and scans every Rust source
      file through the mandatory decoder to require one parser-owned relative-comparison
      helper while rejecting the retired epsilon form. Full gate: 1006 passed / 0 failed /
      36 ignored, including backend 949 passed / 0 failed / 36 ignored and 55 owned Rust
      warnings.
- [ ] **Visual:** import a disposable LAS containing one declared near-sentinel and one
      nearby genuine reading; inspect the curve and QC surfaces to confirm only the former
      appears missing.
- [ ] **Manual:** Jauhar independently queries the imported samples and confirms absence is
      `f32::NAN` internally while the genuine finite value is byte-preserved. The static
      inventory is not this operator/data check.
- [ ] **Field:** pending Gate 4; no representative delivery was used to assess whether its
      observed formatter noise is covered without false-positive screening.

## 2026-08-14 — G2 SB-DIO-002: the only default export path honours the project sentinel

- **Automated — not manual evidence:** exact SB-DIO-T03 proves the registry has exactly
      one default, requires that writer to honour the sentinel, exports the cited
      non-default Baker value through the real default path, and proves an incapable
      synthetic format exposes its limitation instead of looking equivalent. Full gate:
      1006 passed / 0 failed / 36 ignored, including backend 949 passed / 0 failed / 36
      ignored and 55 owned Rust warnings.
- [ ] **Visual:** no alternate data format ships, so there is currently no meaningful
      format choice to inspect. When a second format is registered, verify the picker
      shows exactly one default and renders any sentinel limitation before export.
- [ ] **Manual:** Jauhar exports a disposable well without changing the format and opens
      the result independently to confirm the declared project sentinel. This operator
      workflow is not claimed by exact T03.
- [ ] **Field:** pending Gate 4; no representative field delivery or third-party reader
      was exercised by this increment.

## 2026-08-14 — G2 SB-DIO-001: one project sentinel reaches the complete writer registry

- **Automated — not manual evidence:** exact SB-DIO-T01 sets the cited non-default Baker waveform
      sentinel, enumerates every registered writer, requires each output to declare and
      use it, proves no writer emits the project default instead, and requires the
      registered self-reader to accept the result. Exact SB-DIO-T02 enumerates the same
      registry and pins the required non-optional `WriterSettings` function signature.
      Full gate: 1006 passed / 0 failed / 36 ignored, including backend 949 passed / 0
      failed / 36 ignored and 55 owned Rust warnings.
- [ ] **Visual:** not claimed. Export a disposable well after selecting a non-default
      project sentinel and inspect the completed export/status surface for the chosen
      format and self-check result.
- [ ] **Manual:** Jauhar opens the exported file independently and confirms its null
      declaration and missing samples use the selected project sentinel. The synthetic
      registry test proves custody, not the operator workflow or a third-party reader.
- [ ] **Field:** pending Gate 4; no representative field delivery was exported by this
      increment.

## 2026-08-14 — G2 SB-DBM-042: recovery copies name what they restore

- [x] **Automated:** exact SB-DBM-T01 hashes a newer-format project before and after
      refusal and checks both versions plus writer identity. Exact SB-DBM-T02 proves the
      real destructive-open backup precedes the rewrite, is reported, never overwrites,
      aborts deterministically on copy failure and is absent on an additive open. Exact
      SB-DBM-T43 proves consecutive source formats produce distinct `pre-0` and `pre-1`
      recovery copies containing the promised states. Full gate: 1006 passed / 0 failed /
      36 ignored, including backend 949 passed / 0 failed / 36 ignored and 55 owned Rust
      warnings.
- [ ] **Visual:** not claimed. On a disposable legacy project, inspect the Processing
      history/status notice and confirm it displays the exact source-labelled backup path.
- [ ] **Manual:** Jauhar opens the recovery copy and confirms the pre-migration project is
      usable. Automated synthetic DuckDB files prove structure and bytes, not the operator's
      recovery workflow on representative data.
- [ ] **Field:** pending Gate 4; no customer or field project was used by this increment.

## 2026-08-14 — G2 SB-DBM-041: full inspector trace blocked on the audit store

- [x] **Automated count half:** exact SB-DBM-T41 remains green: inspector
      `total_rows` is the true count and capped SQL results use `returned_rows` with an
      explicit non-total flag. Full gate remains 1005 passed / 0 failed / 36 ignored,
      including backend 948 passed / 0 failed / 36 ignored and 55 owned Rust warnings.
- [ ] **Automated full trace:** BLOCKED. SB-DBM-T42 requires a computed curve to be
      traced through the inspector to its run, parameters, inputs, model row and the
      `audit_entry`/`audit_detail` tables owned by SB-DBM-011. Those audit tables do not
      exist, and SB-DBM-011 is blocked on DEC-022 and DEC-023. No reduced T42 was written.
- [ ] **Decision / architecture:** settle DEC-022's legacy timestamp classification and
      DEC-023's zone-set scope, then implement SB-DBM-011's controlled audit relations.
      Only after that may the inspector inventory be derived and exact T42 added.
- [ ] **Visual / Manual / Field:** not claimed. Exposing only the currently available
      provenance tables would improve the screen but still falsely imply a complete audit
      path. Jauhar's click-through and Gate 4 remain pending after the blocker is removed.

## 2026-08-14 — G2 SB-DBM-040: cancellation reports what the worker observed

- [x] **Automated:** exact SB-DBM-T40 reuses the already-integrated three-job regression
      that drives an observing worker, a non-observing worker and non-cancellable work.
      It proves only the observed request finalizes `Cancelled`, an unobserved request
      preserves actual `Completed`, the non-cancellable view is false, every advertised
      cancellable registration routes an observing handle, and the Processing source
      gates its control on active plus cancellable. This is the chapter's declared
      `CHARACTERIZATION`, not a new behavior or a new test count. Full gate remains 1005
      passed / 0 failed / 36 ignored, including backend 948 passed / 0 failed / 36 ignored,
      with the unchanged 55 owned Rust warnings.
- [ ] **Visual:** at narrow and wide Processing-panel widths, inspect one active
      cancellable job and one active monolithic job. Confirm only the former offers
      Cancel, the latter visibly says it cannot be interrupted, long job labels do not
      hide either state, and an observed cancellation ends with an unambiguous status.
- [ ] **Manual:** on a disposable project, cancel one polling workflow, let one requested
      cancellation go unobserved until the work actually completes, and run one
      non-cancellable operation. Verify the final phases match the worker outcomes and no
      control promises interruption where none is implemented. Jauhar owns this evidence.
- [ ] **Field:** pending Gate 4. Repeat cancellation on sanitized representative pilot
      workloads whose duration is long enough to observe the control and final state.
      Source inventory and synthetic async jobs are not field-performance evidence.

## 2026-08-14 — G2 SB-DBM-039: degraded work cannot look clean

- [x] **Automated:** exact SB-DBM-T39 runs one clamped well, one substituted-input well
      and one clean well, proves their typed degraded/degraded/clean results and
      Warned/Warned/Ok job items, proves the aggregate outcome is degraded, forces the
      25-job prune, then recovers the structured reasons from the durable run record.
      Exact SB-DBM-T41 separately proves an inspector page over 10,000 rows reports the
      true total while a 100-row SQL page exposes `returned_rows` and explicitly says it
      is not a total. The curve rows, outcome and ordered degradation events share one
      transaction. Full gate is 1005 passed / 0 failed / 36 ignored, including backend
      948 passed / 0 failed / 36 ignored, with the unchanged 55 owned Rust warnings.
- [ ] **Visual:** at narrow and wide dock widths, run a three-well module batch containing
      a source-qualified clamp, an explicit documented input substitution and a clean
      result. Confirm the module dialog distinguishes clean/degraded/failed counts, the
      Processing card says `Done with warnings`, each affected well names its structured
      reason without clipping, Curve Catalog exposes the durable outcome/reasons, and the
      SQL console labels a capped result as rows returned rather than a total.
- [ ] **Manual:** on a disposable project, produce the same three result classes, record
      the output set identities, complete at least 25 later jobs, close/reopen the project,
      and verify the two degraded run records still name their reasons while the clean run
      remains clean. Jauhar owns this click-through and records any UX defect separately;
      this automated increment does not claim the click path has been exercised.
- [ ] **Field:** pending Gate 4. Repeat the durable-result and capped-count checks on
      sanitized representative pilot output. The synthetic three-well and 10,000-row
      fixtures prove the reporting boundary, not field suitability or operator acceptance.

## 2026-08-14 — G2 SB-DBM-037: backend scope is the authority

- [x] **Automated:** exact SB-DBM-T37 creates the cited 540-well project with an active
      group of 12, directly exercises all 44 registered backend authorization boundaries, and
      proves the 43 scoped operations resolve exactly the current 12 while the deliberately
      exhaustive integrity operation reports `PROJECT_WIDE` and 540 wells touched. The test also
      inventories every corresponding Tauri wrapper and pins the downstream well, contact, top and
      statistics loaders to SQL-scoped ids, so resolving 12 and then loading/filtering 540 cannot
      satisfy it. SB-CORE-035 still proves Active Group, named Group, All and Explicit remain
      distinct and stale membership is refused. Full gate is 1003 passed / 0 failed / 36 ignored,
      including backend 946 passed / 0 failed / 36 ignored, with the unchanged 55 owned Rust
      warnings.
- [ ] **Visual:** with an active group, inspect the object tree, map, contact-consistency pane,
      FWL check, top-order warning and TVD materialization at narrow and wide dock widths. Confirm
      ordinary surfaces show only the active wells, explicit project-administration surfaces still
      offer All, and Database Inspector visibly says `PROJECT WIDE — N wells examined` for the
      exhaustive integrity check without clipping the finding table.
- [ ] **Manual:** on a disposable project, create a named group, invoke Active Group and named
      Group directly, then change membership while a dialog remains open and verify the next run
      uses the new membership. Exercise All and an Explicit selection as separate intentional
      modes, and confirm an out-of-scope top check is refused. Jauhar owns this click-through.
- [ ] **Field:** pending Gate 4. Repeat the scope and project-wide-disclosure checks on a sanitized
      representative pilot project and compare the touched well identities to the operator's group.
      The synthetic 540/12 fixture proves the engineering boundary, not field qualification or
      acceptable performance at representative project scale.

## 2026-08-14 — G2 SB-DBM-035: restore appends history instead of rewinding it

- [x] **Automated:** exact SB-DBM-T35 begins with archived versions 1 and 2 plus current
      version 3. It proves SQL-console UPDATE/DELETE and the stale ordinary-delete command refuse,
      restoring version 1 creates a fresh set identity at version 4, the structured run record
      names source version 1, current rows cite version 4, version 4 contains the source values,
      and every row in versions 1–3 is unchanged. The existing catalog and downstream-workflow
      integration tests also pass. Full gate is 1002 passed / 0 failed / 36 ignored, including
      backend 945 passed / 0 failed / 36 ignored, with the unchanged 55 owned Rust warnings.
- [ ] **Visual:** in Curve Catalog at narrow and wide dock widths, inspect a set with at least
      three versions. Confirm there is no ordinary Delete action, restore version 1, and verify the
      new version 4 row visibly says `restore`, `restored from v1`, and `current` without clipping
      the curve list, date, or Ancestry/Restore controls.
- [ ] **Manual:** on a disposable project, record the values and identities of versions 1–3,
      restore version 1 while version 3 is current, close/reopen the project, and verify the
      current values now cite version 4 while all four versions remain selectable. Restore version
      3 again to reverse the operation as another append-only run. Jauhar owns this click-through.
- [ ] **Field:** pending Gate 4. Repeat on sanitized representative pilot output and compare the
      restored curve bytes and complete run record against the selected source version. The green
      synthetic fixture is not evidence that a real delivery, large archive, or operator workflow
      has been qualified. Backed-up format migration and typed reversible integrity quarantine are
      separately bounded maintenance paths; neither is an ordinary history-delete permission.

## 2026-08-14 — G2 SB-DBM-033: categorical curves never become arithmetic quantities

- [x] **Automated:** exact SB-DBM-T33 uses the chapter's cited 0.1524 m-to-0.1 m
      resample and producer-declared fixture codes 1/4. It proves the committed output contains
      only those existing codes, the Reframe payload and UI report both target samples that cross
      the source transition, the declaration remains active, Rhai and Python arithmetic each
      refuse before writing, and unreadable type metadata stops Reframe instead of silently
      treating the curve as continuous. Full gate is 1001 passed / 0 failed / 36 ignored, including
      backend 944 passed / 0 failed / 36 ignored, with the unchanged 55 owned Rust warnings.
- [ ] **Visual:** in Reframe at narrow and wide dock widths, request Interpolate for one
      producer-declared class curve. Confirm the result says `nearest`, shows the number of
      category-boundary samples, and lists each target depth plus its two source codes/depths
      without clipping or hiding the ordinary run notes.
- [ ] **Manual:** on a disposable project, run a producer that declares a categorical curve,
      Reframe it to a finer explicitly stated sampling, and inspect every reported transition.
      Then select the same curve in both equation editors and confirm each refuses by mnemonic and
      writes no output. Jauhar owns this click-through.
- [ ] **Field:** pending Gate 4. Exercise one sanitized producer-declared class curve and compare
      its source/output code sets and transition depths. The synthetic labels and green test do
      not prove a real facies delivery, and this increment does not claim arbitrary imported-curve
      retyping is available.

## 2026-08-13 — G2 SB-DBM-032: single-handle parameter policy conflict

- [ ] **Automated implementation:** blocked by DEC-028. No SB-DBM-T32 test was added because its
      required one-handle warning contradicts the already-passing P0 SB-INS-T18 missing-ordinal
      refusal on the same `parameter_pack.rs` load surface. The mismatch refusal remains intact;
      no assertion or production guard was weakened.
- [ ] **Architecture / source boundary:** choose whether a semantic-only row and an ordinal-only
      row each refuse or load with a warning, then correct the conflicting requirement/test before
      unit, source, tilt and append-only ordinal custody are implemented. Engineering recommends
      refusing both for the first pilot; warning-and-continue recreates the documented class of
      plausible wrong-parameter activation.
- [ ] **Visual / Manual:** after adjudication and implementation, load the crossed-handle fixture,
      both one-handle fixtures and a two-zone logarithmic tilt. Confirm the refusal or warning names
      the actual handles and the boundary steps to the next zone rather than interpolating across
      it. Jauhar owns this click-through.
- [ ] **Field:** blocked. A sanitized pilot parameter pack must prove its semantic IDs, permanent
      ordinals, units, sources and zone tilts against its producer; a synthetic JSON file or green
      loader test cannot establish that custody.

## 2026-08-13 — G2 SB-DBM-031: depth unit/datum and correlation-contact refusal

- **Automated — not manual evidence:** exact SB-DBM-T31 stores an MD zone and a TVDSS contact,
      refuses their comparison without a well frame while naming both datums, then permits the same
      pair through a declared frame and asserts positive-down TVDSS / positive-up elevation. The
      format-2 migration backs up real projects, converts only explicitly TVDSS system stores once,
      and leaves an untyped legacy zone NULL and unreadable rather than relabelling it MD. The
      correlation view no longer substitutes MD when its TVDSS frame is absent. Full gate is
      1000 passed / 0 failed / 36 ignored with the unchanged 55 owned Rust warnings.
- [ ] **Visual:** in Correlation at narrow and wide dock widths, select TVDSS with one well that has
      a declared TVDSS curve and one that does not. Confirm the latter is labelled
      `no TVDSS frame`, draws no curve/top/contact comparison, and the status says MD was not
      substituted. Confirm the positive-down wording remains visible in the contact editor.
- [ ] **Manual:** on a disposable pre-format-2 project copy, record the original survey-derived
      TVDSS/contact values, open it, confirm the adjacent backup exists, and verify the converted
      values are positive down exactly once. Insert or retain an untyped legacy zone and confirm it
      refuses use until its datum is explicitly assigned through Database Inspector.
- [ ] **Field:** blocked pending source/operator declarations for every legacy and imported depth
      frame in the sanitized representative delivery. Compare the declared MD/TVDSS reference and
      elevation against the delivery documentation; a mnemonic, unit, sign, or green synthetic test
      is not evidence of datum.

## 2026-08-13 — G2 SB-DBM-030: null-state contract needs adjudication

- [ ] **Automated implementation:** blocked by DEC-027. No strict Geolog-family store screen or
      partial T29/T30 proof is presented as the whole contract. The unchanged full gate remains
      999 passed / 0 failed / 36 ignored with the existing 55 owned Rust warnings.
- [ ] **Architecture / source boundary:** decide two conflicts before code changes. SB-DBM-003
      requires an unsupplied required parameter to remain a queryable `REQUIRED_UNSET` row with
      SQL-NULL value/source, while SB-DBM-030 requires absence-of-row. T29 also requires its bound
      to derive from the value the export path emits, while §5 refuses both conflicting Geolog
      magnitudes and assigns that future export choice to SB-DIO.
- [ ] **Visual / Manual:** after adjudication and implementation, confirm a missing curve sample is
      visibly a gap—not a plotted sentinel—and an unsupplied required parameter is visibly named
      `REQUIRED_UNSET`, never shown as zero, a magnitude, an empty string or a clean value. Jauhar
      owns this click-through.
- [ ] **Field:** blocked. The sanitized pilot delivery must later prove that source-specific vendor
      nulls are flagged before storage and that exact-threshold data survives. Synthetic values and
      a green gate cannot establish which Geolog magnitude a real export must declare.

## 2026-08-13 — G2 SB-DBM-029: protect every existing reference frame

- [x] **Automated:** exact SB-DBM-T28 drives the real module API with its output renamed to
      `DEPTH`, requires the refusal to name the existing `STANDARD` frame, and snapshots every
      standard column plus a computed peer so any movement fails byte-for-byte. Its positive
      control runs Reframe with an explicit synthetic fixture step, proves the distinct basis is
      archived under `frame = 'OWN'`, and proves the original frame is still byte-identical. Full
      gate is 999 passed / 0 failed / 36 ignored with the existing 55 owned Rust warnings.
- [ ] **Visual:** in a module's Output curves card, enter `DEPTH` at narrow and wide dock widths.
      Confirm the refusal visibly names both `DEPTH` and the `STANDARD frame`, explains that
      Reframe creates an `OWN` frame, and does not clip the recovery instruction.
- [ ] **Manual:** on a disposable project, attempt the same output rename and confirm no raw or
      computed curve changes. Then use Data > Sampling with an explicitly chosen step and confirm
      the new log set is selectable on its own depths while the original set is unchanged. Jauhar
      owns this click-through.
- [ ] **Field:** pending Gate 4. On the sanitized representative pilot delivery, compare the source
      and reframed depth inventories and inspect neighbouring curves before and after. The
      synthetic byte-snapshot proves the write contract, not the field sampling choice.

## 2026-08-13 — G2 SB-DBM-028: verify a set's declared sampling style

- [x] **Automated:** exact SB-DBM-T27 supplies its tolerance as a unit-typed fixture input and
      first proves that neither the sampling style nor the regular-verification tolerance has a
      production default. The cited 0.1524 m fixture then omits 40 mid-interval rows, requires the
      stored declaration to become effective `CONTINUOUS_IRREGULAR`, names depth and missing-row
      count, and keeps the known post-gap sample at its native depth rather than 6.1 m shallow. A
      genuinely regular control remains regular, while an unverified legacy set is refused by the
      explicit frame reader. Full gate is 998 passed / 0 failed / 36 ignored with 55 owned Rust
      warnings.
- [ ] **Visual:** open Import LAS at narrow and wide modal widths. Confirm Sampling style starts
      unselected, Regular-step tolerance starts empty and disabled, selecting regular enables both
      value and unit without filling either, and selecting irregular clears/disables them. Confirm
      validation and the named post-import warning remain legible without clipping.
- [ ] **Manual:** on a disposable project, import one regular LAS with an explicit tolerance, one
      declared-regular LAS containing a known gap, and one declared-irregular LAS. Inspect the
      stored `import_sets` rows and confirm style, effective verdict, original tolerance/unit,
      warning, gap depth and row count agree with the delivery. Jauhar owns this click-through.
- [ ] **Field:** pending Gate 4. Use the sanitized representative pilot delivery to confirm the
      tolerance is project/source justified, then inspect one sample after every contradicted gap
      against the source LAS. The synthetic 40-row fixture does not qualify a field tolerance.

## 2026-08-13 — G2 SB-DBM-027: complete integrity report and recoverable quarantine

- [x] **Automated:** exact SB-DBM-T26 seeds one archive row whose `set_id` cannot resolve and one
      well-group membership whose well is gone, while leaving orphan curve samples at the cited
      zero. It requires all seven live classes by name and count, proves the read-only check changes
      no row, rejects a bare `clean`, and drives selected typed quarantine through restore and exact
      reapply. The frontend sends class IDs only; no SQL or sample array crosses IPC. Full gate is
      997 passed / 0 failed / 36 ignored with the existing 55 owned Rust warnings.
- [ ] **Visual:** open Database Inspector and check at narrow and wide dock widths. Confirm all
      seven rows, counts, repair eligibility and required actions remain readable; zero findings
      must say `Checked 7 integrity classes; 0 findings.` rather than showing a generic green
      badge. Confirm report-only ML/duplicate rows never receive a destructive checkbox.
- [ ] **Manual:** on a disposable project copy, create or recover a known orphan, run the checker,
      quarantine only the selected class, close/reopen the app, restore the persisted batch, then
      exercise Ctrl+Z and Ctrl+Y. Confirm legacy current rows with `set_id IS NULL` remain reported
      and are not selected for deletion. Jauhar owns this click-through evidence.
- [ ] **Field:** pending Gate 4. Run the read-only checker on the sanitized representative pilot
      delivery before and after its workflow. Every nonzero count must be investigated by class;
      the synthetic 1/1/0 fixture and a green gate do not prove a real project is healthy.

## 2026-08-13 — G2 SB-DBM-026: depth uniqueness follows declared set type

- [x] **Automated:** exact SB-DBM-T25 drives both `CONTINUOUS_REGULAR` and
      `CONTINUOUS_IRREGULAR` writes through the real computed-curve boundary and requires the
      named depth plus both source rows before any current/archive mutation. It also corrupts an
      archive fixture deliberately and proves Restore cannot bypass that refusal. The shipped
      auxiliary-data writer accepts same-depth POINT observations under an explicit `PRESERVE`
      declaration, while `PERTURB` refuses without a positive unit-typed offset and logs both rows
      when the cited 0.01 ft fixture value is supplied. `computed_curves` remains PK-less and no
      upsert or uniqueness index was added. Full-gate result is recorded in the status ledger.
- [ ] **Visual:** no new control was introduced. In Gate 4, inspect one preserved duplicate POINT
      delivery in its existing point-data surface and one continuous duplicate refusal. A database
      row or green test is not proof that the operator can see which policy was applied.
- [ ] **Manual:** on a disposable project, import a representative pressure/core-point delivery
      containing legitimate same-depth observations and confirm both remain addressable; then
      attempt a continuous duplicate write and Restore and confirm the error names depth and both
      rows without changing the previous curve. Jauhar owns this click-through evidence.
- [ ] **Field:** pending Gate 4. A real pilot delivery must confirm that preserved same-depth points
      are scientifically intentional and that no delivery-specific workflow expected a silent
      survivor. The synthetic 0.01 ft fixture is cited correctness evidence, not a project default
      and not field acceptance.

## 2026-08-13 — G2 SB-DBM-025: cross-module constant registry is source-blocked

- [ ] **Automated implementation:** blocked, so the existing `PHIE_FLOOR = 0.001` is not promoted
      into a source-bearing registry and no partial T23/T24 test is presented as the whole
      contract. The unchanged full gate remains 995 / 0 / 36 with 55 owned Rust warnings.
- [ ] **Architecture / source boundary:** settle DEC-026. The floor crosses the selected density,
      analytic D-N and pay paths. `CLAUDE.md` mandates 0.001, while SB-POR-045 and its immutable
      parameter table require ABSENT after one held source attests both 0.001 and 0.0001. A central
      registry would amplify whichever side we guessed; it would not adjudicate the contradiction.
- [ ] **Visual / Manual:** after the precedence decision and implementation, confirm the chosen or
      required-empty floor state appears once, carries both competing citations, and survives the
      run/pay record. Jauhar owns this review; a source-tree scan is not UI evidence.
- [ ] **Field:** blocked. A representative tight/zero-porosity interval must later prove that the
      explicit floor choice, unlimited companion and pay classification remain distinguishable.
      No synthetic fixture may be labelled field verification.

## 2026-08-13 — G2 SB-DBM-023: schema vocabularies have one source

- [x] **Automated:** one typed registry now owns standard columns, sampling style,
      duplicate-depth resolution, set frame, depth datum, audit location/mode and named provenance
      absence. Exact SB-DBM-T23 adds a
      synthetic eighth schema member and requires every derived projection to see it, then scans
      the Rust source tree for a second declaration or copied full standard-column registry. The
      full gate passes 995 / 0 / 36 with 55 owned Rust warnings.
- [ ] **Visual:** no new control is intended. In Gate 4, confirm a representative standard curve,
      an edited standard curve and an OWN-frame set still appear under their existing names and
      units; an unchanged screen is useful regression evidence, not proof that every code consumer
      derives from the registry.
- [ ] **Manual:** on a disposable project, import a representative delivery, edit and undo one
      standard curve, browse `standard_curves`, and create/read one Reframe set. Jauhar owns this
      click-through; the automated source mutation test remains the registry-completeness proof.
- [ ] **Field:** pending Gate 4. A real pilot delivery must preserve native curve identities and
      frame behavior. Synthetic PEF in T23 is only a schema-name mutation and carries no invented
      petrophysical value, endpoint or default.

## 2026-08-13 — G2 SB-DBM-017: physics-driving metadata is scope-blocked

- [ ] **Automated implementation:** blocked, so explicit `nphimat` parameters are not presented as
      the stored metadata contract and no synthetic T17 was added. Exact T17 still needs the
      run-time attribute value, stale-output invalidation after a change and named refusal when the
      attribute is absent. The unchanged full gate remains 994 / 0 / 36 with 55 owned Rust warnings.
- [ ] **Architecture / scope:** settle DEC-025. SB-POR-024 requires neutron matrix-basis metadata,
      but SB-ENV-012 owns its typed enum, persistence and consumer validation and is outside
      DEC-018's immutable pilot manifest. Either authorize that seam as infrastructure or revise and
      re-approve the manifest; do not invent a Logging Contractor, tool, salinity or matrix default.
- [ ] **Visual / Manual:** after implementation, show the declared curve basis in the run review,
      change it in a disposable project and confirm prior outputs become visibly stale. Remove it
      and confirm the UI names the missing attribute instead of substituting a default. Jauhar owns
      this click-through evidence; source inspection is not visual or manual proof.
- [ ] **Field:** blocked. A representative pilot delivery must preserve the imported basis and make
      a changed or absent value fail visibly. Vendor precedent and a synthetic project do not prove
      the real delivery's metadata is trustworthy.

## 2026-08-13 — G2 SB-DBM-016: fresh-process order independence

- [x] **Automated:** exact SB-DBM-T16 runs two byte-identical copies of one imported two-well
      project in fresh Rust test processes. Their live 64-key HashMap iteration witnesses must
      differ, while every computed curve's packed bytemuck depth/value bytes and every pay-summary
      field must agree exactly. A third process changes recorded Rw and must move both artifacts,
      preventing an empty or constant comparison from passing. The full gate passes 994 / 0 / 36
      with 55 owned Rust warnings; no production behavior or scientific value changed.
- [ ] **Visual:** no new UI exists. During Gate 4, compare the curve catalog and summary presentation
      after two representative runs; a green binary comparison does not prove understandable
      presentation or that the operator selected the intended saved inputs.
- [ ] **Manual:** rerun the approved deterministic chain on a disposable sanitized project, reopen
      it, and compare the complete curve inventory and summary—not only SWE or one headline net-pay
      value. Record any mismatch rather than rounding it away.
- [ ] **Field:** pending Gate 4. The repository fixture proves fresh-process order independence for
      the approved chain, not a real delivery, a different machine, the absent T15 manifest resolver
      or any of the 689 deferred requirements. Jauhar records representative field acceptance.

## 2026-08-13 — G2 SB-DBM-015: complete re-run manifest is dependency-blocked

- [ ] **Automated implementation:** blocked, so deterministic fragments are not presented as the
      complete T15 proof. No stored manifest resolver or "re-run this set" command exists, and no
      test proves the unmutated byte-identical replay plus all four element-naming refusals. The
      unchanged full gate remains 993 / 0 / 36 with 55 owned Rust warnings.
- [ ] **Architecture / scope:** settle DEC-021's build-derived module identity, DEC-023's versioned
      zone-set seam and DEC-024's conditional stochastic/model identity seam. Exact T15 reaches
      pilot-excluded SB-DBM-014 and SB-DBM-019/020 plus SB-DBM-017's physics-driving attributes.
      Do not omit those manifest arms or import deferred capabilities without explicitly revising
      and re-approving DEC-018.
- [ ] **Visual / Manual:** after implementation, expose the stored manifest and make each unresolved
      element refusal name the exact module, input version, zone set or model. Jauhar should verify
      the wording against a disposable project; source-level resolution is not UI evidence.
- [ ] **Field:** blocked. A representative run must later be re-run from its stored record alone and
      compared byte-for-byte. A same-process deterministic unit test is not cross-process or field
      replay evidence.

## 2026-08-13 — G2 SB-DBM-013: provenance cannot be disabled

- [x] **Automated:** SB-DBM-T13 executes the real module runner on three sides. An ordinary run
      writes two VSH samples with one complete run record. A test-only database constraint makes
      the second `log_sets` insert fail after the first has run, and requires the transaction to
      leave zero FAULT run records, zero computed rows and both serialized processing items
      `Failed`. The legacy-looking `skip_version` request is also executed and must refuse with
      zero PAYFLAG records/curves. Whole-corpus inventories enumerate environment, DuckDB,
      project-document, installed-user, persisted and session preference reads and reject any raw
      computed writer. The full gate passes 993 / 0 / 36 with 55 owned Rust warnings; no production
      behavior, scientific value, schema key or upsert changed.
- [ ] **Visual:** in the Processing panel, exercise a safely induced run-write failure and confirm
      every affected well is visibly Failed with a useful message; confirm no UI or deployment
      setting offers a provenance-off mode. Source scanning proves reachability, not readability.
- [ ] **Manual:** in a disposable project, run one ordinary module and confirm its output resolves
      to exactly one live ancestry record. Induce a safe record-write refusal, reopen the project
      and confirm neither a partial run record nor output curve survived. Do not alter a real
      project merely to manufacture this evidence.
- [ ] **Field:** on a representative pilot run, confirm a saved output and its ancestry remain
      inseparable through Save Project As and an ordinary deliverable path. The synthetic unique
      constraint is automated failure evidence, not field acceptance; Jauhar records this check.

## 2026-08-13 — G2 SB-DBM-011: structured audit is dependency-blocked

- [ ] **Automated implementation:** blocked, so the current free-text process log is not presented
      as a relational audit. It survives Save Project As, but one capped JSON blob and more than 70
      free-text UI emitters provide no controlled location/mode vocabulary, typed value/unit rows or
      explicit uninterrupted-gesture coalescing. The unchanged full gate remains 992 / 0 / 36 with
      55 owned Rust warnings.
- [ ] **Product/data decisions:** settle DEC-022's legacy UTC classification and DEC-023's scope
      conflict. Exact T11 requires zone-set identity, but SB-DBM-008 is outside DEC-018's immutable
      pilot manifest. Either authorize that seam as audit infrastructure or revise and re-approve
      the manifest; do not quietly weaken T11. DEC-020 already supplies explicit HUMAN/AUTOMATED
      operator identity.
- [ ] **Visual / Manual:** after implementation, change three zone parameters, rename one curve and
      drag one crossplot point repeatedly without releasing the gesture. Confirm controlled rows,
      one entry for the gesture, local timestamp display, exact operator/zone-set custody and legacy
      text history retained as visibly unstructured rather than silently discarded.
- [ ] **Field:** blocked on both decisions. Jauhar must inspect the audit inside a Save Project As
      copy and confirm the controlled rows remain queryable; synthetic row insertion alone is not
      evidence that the real UI actions are all captured.

## 2026-08-13 — G2 SB-DBM-010: complete deliverable provenance is source-blocked

- [ ] **Automated implementation:** blocked, so no synthetic T10 is presented as complete
      provenance proof. LAS already embeds machine-readable ancestry, current parameter sources
      travel inside it, and legacy computed curves remain labelled and counted. PDF and Office
      deliverables already carry shared human-readable ancestry rows. None of those paths can
      supply SB-DBM-005's missing method-derivation citations. The unchanged full gate remains
      992 / 0 / 36 with 55 owned Rust warnings.
- [ ] **Source decision:** approve a complete source-controlled registered-module citation map for
      SB-DBM-005. A free-form output derivation such as a module or fixture description is not a
      literature/specification citation and must not be relabelled as one merely to fill T10.
- [ ] **Visual / Manual:** after the citation inventory exists, export representative computed and
      legacy curves through every pilot format. Confirm the UI names any format that drops a
      machine-readable sidecar before export; inspect the LAS record, PDF/Office ancestry and
      sidecar resolution back to exact run records, parameter sources and citations.
- [ ] **Field:** blocked on the same source inventory. Jauhar must inspect representative client
      deliverables and confirm each numeric computed curve resolves without opening SandiBumi;
      repository JSON presence alone is not field acceptance.

## 2026-08-13 — G2 SB-DBM-009: legacy timestamp meaning is decision-blocked

- [ ] **Automated implementation:** blocked, so no test is presented as proof that the whole UTC
      storage/local-display contract ships. Current curve ancestry records a Unix-epoch instant,
      but Inspector renders it in UTC; process history records an epoch instant and renders it
      locally on screen, while its text export drops an explicit zone; `log_sets.created_at`
      remains local/unspecified. The unchanged repository gate remains 992 / 0 / 36 with 55 owned
      Rust warnings.
- [ ] **Product/data decision:** settle DEC-022. The engineering recommendation is to mark every
      pre-migration timestamp `ZONE_UNKNOWN`, preserve its literal legacy text, store every new
      timestamp as an unambiguous UTC instant and convert only at the UI edge. Do not infer the old
      authoring zone from the machine that later opens the project.
- [ ] **Visual / Manual:** blocked until DEC-022 and the shared timestamp representation are
      implemented. Then inspect one new record in two machine zones and one legacy record: the new
      instant must remain identical while its display changes, and the legacy value must remain
      visibly zone-unknown rather than being silently shifted.
- [ ] **Field:** blocked on the same decision and on SB-DBM-011's structured audit store. Jauhar
      must record representative cross-zone and legacy-project evidence; a synthetic epoch test
      alone cannot close project-history custody.

## 2026-08-13 — G2 SB-DBM-007: provenance absence has a name

- [x] **Automated:** SB-DBM-T09 executes a real equation whose run has no configurable
      parameters and requires schema-v3 ancestry to round-trip `NOT_APPLICABLE`, never an empty
      string or an empty parameter object masquerading as meaning. Its other half injects a module
      parameter-serialization error through the real module runner and requires a reported failure,
      zero module `log_sets` rows and zero computed VSH rows. Older schema-v1/v2 empty parameter
      collections are read as `LEGACY_UNRECORDED`; required-but-unsupplied named parameters remain
      `REQUIRED_UNSET`. The full repository gate passes 992 / 0 / 36 with 55 owned Rust warnings.
      Sample missingness remains `f32::NAN`; no petrophysical value, default, endpoint, limit or SQL
      schema was introduced.
- [ ] **Visual:** inspect an equation run and a pre-v3 run in Ancestry. Confirm the current run says
      parameters are not applicable, the legacy run says they were not recorded, and neither is
      rendered as a blank field, `{}`, `null`, zero or a raw parsing error. A serialized enum in the
      backend is automated evidence, not proof that this distinction is understandable on screen.
- [ ] **Manual:** in a disposable project, run a parameterless equation and query its `params_json`.
      Confirm the embedded ancestry has an empty parameter list plus `NOT_APPLICABLE`, while the
      equation definition metadata remains present. Open a pre-v3 fixture with an empty parameter
      list and confirm the reader reports `LEGACY_UNRECORDED` rather than rewriting history. Keep
      the injected serialization-failure test as the controlled failure proof; do not manufacture
      malformed production data merely to click this path.
- [ ] **Field:** inspect representative pilot equation and module run records after the workflow is
      frozen. Confirm a reviewer can distinguish no parameters, an unsupplied required parameter,
      and legacy unrecorded provenance. Automated synthetic evidence does not close this check;
      Jauhar records field acceptance.

## 2026-08-13 — G2 SB-DBM-006: each run names the curve decision it actually used

- [x] **Automated:** SB-DBM-T08 creates three GR curves across two imported sets, marks one
      curve Final, runs a module, reflags the other set and runs again. It requires the exact
      chosen UUID/set/version, both rejected UUID/set/version identities, `FINAL_FLAG`, the
      changed choice, independently derived output bytes and refusal of an incomplete schema-v2
      record. A second two-sided regression requires a RAW family match to beat an exact mnemonic
      outside RAW, then requires that attached exact curve after the RAW curve is removed. Existing
      native-track and deterministic-replay regressions remain green. The full repository gate
      passes 991 / 0 / 36 with 55 owned Rust warnings; no petrophysical value,
      endpoint, cutoff, range or default was introduced, and `computed_curves` remains PK-less
      with no upsert path.
- [ ] **Visual:** in Curve Catalog, confirm exactly one curve in a duplicated family carries the
      Final badge, Mark/Clear Final is understandable, and Ancestry presents the chosen identity,
      set/version, decision rule and rejected candidates without requiring raw JSON. Confirm an
      ordinary blank-set log track still shows its established current standard projection.
- [ ] **Manual:** in a disposable project, load three same-family curves across two sets, mark one
      Final, run a module, reflag a different curve and rerun. Query both run records and confirm
      each winner, both rejected candidates and set versions match the actual numeric inputs.
      Also request a family that RAW carries under an alias while an attached set carries the exact
      mnemonic: confirm RAW wins, then delete the RAW curve and confirm the attached curve becomes
      eligible. Undo the reflag and confirm the displaced Final designation is restored.
- [ ] **Field:** repeat the decision trace on a representative pilot well with a genuinely
      duplicated delivered curve family. Confirm a reviewer can explain which physical curve fed
      the result and why. Automated synthetic evidence does not close this check; Jauhar records
      field acceptance.

## 2026-08-13 — G2 SB-DBM-005: method derivation is source-blocked

- [ ] **Automated implementation:** blocked, so no synthetic mechanism test is presented as proof
      that shipping metadata exists. Live source still has no derivation field in `ModuleSpec` or
      `CurveAncestry`, no fail-closed registration, and no citation to propagate through LAS,
      report or Office deliverables. The unchanged full gate remains 989 / 0 / 36 with 55 owned
      Rust warnings.
- [ ] **Source decision:** approve a complete, source-controlled map assigning every registered
      shipping module either its primary literature/specification/patent citation or an explicit
      `FIRST-PRINCIPLES` marker naming the module's own derivation document. Comments, neighboring
      chapters and engineering memory are not sufficient custody for an audit claim.
- [ ] **Visual / Manual:** blocked until the registered metadata exists. After implementation,
      confirm a normal run shows the method citation beside its effective parameters and a module
      without one is refused before it can write a run record.
- [ ] **Field:** blocked on the same inventory. Once approved and implemented, confirm the exact
      citation travels into representative pilot run records and every number-carrying deliverable;
      do not close this from a repository-only citation.

## 2026-08-13 — G2 SB-DBM-004: effective parameters retain their manifest origin

- [x] **Automated:** SB-DBM-T06 now saves every declared effective ModuleSpec parameter, not just
      request overrides. The owned correctness test independently derives the configurable-manifest
      hash, requires five synthetic parameters to persist as exactly two `EXPLICIT` and three
      `DEFAULTED`, changes one later manifest default, and proves the original value and manifest
      identity remain unchanged. Ordinary module runs and workflow chains share the same recorder;
      REQUIRED_UNSET remains value-less and no petrophysical value, default, endpoint or range was
      added. The full repository gate passes 989 / 0 / 36, with the existing 55 owned Rust warnings.
- [ ] **Visual:** run a disposable module once with two visible overrides and other controls left at
      their manifest defaults. In Sets / Inspector, confirm the saved ancestry shows all effective
      values, the two `EXPLICIT` flags, the `DEFAULTED` flags and a manifest version for every
      default. Raw JSON presence alone is not proof that the presentation is readable.
- [ ] **Manual:** inspect `run_parameters` for an ordinary run and a workflow-chain run. Confirm
      explicit rows have no default-manifest version, defaulted rows share the manifest identity
      used by that module, and historical schema-v1 rows remain unclassified rather than being
      guessed into either state.
- [ ] **Field:** repeat one representative pilot interpretation after the parameter manifests are
      frozen. Confirm the run record matches every value actually consumed. This increment does not
      close SB-DBM-002's build-derived module identity or SB-DBM-015's full rerun manifest; those
      remain separate contracts.

## 2026-08-13 — G2 SB-DBM-003: parameter absence is named and queryable

- [x] **Automated:** SB-DBM-T05/T09/T30's source-state contract now writes every complete run's
      parameters into an indexed relation in the same transaction as its run record. The owned
      correctness test requires a sourced synthetic value, an unsupplied required input stored as
      NULL value/source plus `REQUIRED_UNSET`, a blank-source refusal, and conservative backfill of
      pre-index ancestry. All 14 equations controls and the full 988 / 0 / 36 gate pass. No
      parameter value, endpoint, conversion or default was invented,
      and `computed_curves` remains deliberately PK-less with no upsert path.
- [ ] **Visual:** open Ancestry for a disposable run containing one deliberately unsupplied required
      input. Confirm the record reads as a named absent state with null value/source and that an
      ordinary sourced value still shows its value and citation. Typed JSON alone is not proof that
      the human presentation is understandable.
- [ ] **Manual:** in a disposable project, query `run_parameters` by `state = 'REQUIRED_UNSET'` and
      confirm the returned names match the run's intentionally absent inputs. Reopen a project
      written before this index and confirm source-bearing ancestry becomes queryable without any
      malformed or source-less record being silently repaired.
- [ ] **Field:** exercise representative pilot runs after the pilot parameter inventory is approved.
      Confirm every numeric parameter carries its actual source and every unavailable required input
      remains absent. This automated increment does not approve that inventory or replace Jauhar's
      field evidence.

## 2026-08-13 — G2 SB-DBM-002: build-derived module identity is decision-blocked

- [ ] **Automated implementation:** blocked, not declared green from the populated ancestry field.
      Every current complete-run builder records `module_version = CARGO_PKG_VERSION`, but that
      value is hand-maintained and can remain unchanged when one module's compiled artefact changes.
      SB-DBM-T04 and the module-version arm of T15 therefore remain missing; no snapshot test was
      added to defend the known divergence. The unchanged repository gate remains 987 / 0 / 36 and
      the owned Rust warning inventory remains 55.
- [ ] **Product/architecture decision:** settle DEC-021's exact artefact boundary, derivation,
      stored representation and stability rule. A whole-binary digest, per-module source digest,
      build id or hash policy would all be plausible but materially different contracts; none was
      selected by engineering while `MODULE_VERSION_SOURCE` remains deliberately absent.
- [ ] **Manual:** manual inspection can confirm existing records contain the package version, but it
      cannot prove that value changes with the module artefact. Do not mark this requirement done
      from a visible non-empty version string.
- [ ] **Field:** field use cannot repair an ambiguous build identity. After DEC-021 is implemented,
      Gate 4 may exercise records created by two controlled builds; it must not choose the identity
      scheme retroactively.

## 2026-08-13 — G2 SB-DBM-001: legacy computed values stay visible and honest

- [x] **Automated:** SB-DBM-T03 sends one ancestry-complete computed curve and one seeded legacy
      curve through the shared resolver, Curve Catalog payload, number-carrying disclosure, LAS
      `~O` record and export summary. It requires the first to be `RECORDED`, the second to be
      `LEGACY_UNRECORDED` with its exact row count, and proves no method or parameters are invented.
      The production-writer inventory and existing LAS provenance contract remain green; the full
      gate passed 987 / 0 / 36 with the owned warning inventory unchanged at 55.
- [ ] **Visual:** in a disposable project copy, delete the run-history version behind a current
      computed curve, then reopen Curve Catalog. Confirm the row remains visible, its Set reads
      `LEGACY_UNRECORDED`, the badge includes the row count, and Ancestry is disabled rather than
      opening an empty or invented record.
- [ ] **Manual:** export that disposable well to LAS. Confirm the result message names one
      `LEGACY_UNRECORDED` curve and the file's `~Other Information` contains both the curve-level
      class/count and the export summary. Confirm an ordinary recorded curve still names its real
      log set, version, method and stored ancestry.
- [ ] **Field:** open a legally controlled pre-versioning pilot project if one exists and inventory
      every legacy computed curve before delivery. This test proves classification and transport;
      it does not prove that an old project's history can be reconstructed, and the software must
      never claim that it can.

## 2026-08-13 — G2 SB-INS-019: one generated curve and unit vocabulary

- [x] **Automated:** SB-INS-T24 compares every one of the accepted 15 families and 42 unit tokens
      across generated Rust runtime, LAS import UI, Markdown and test-manifest consumers carrying
      one version and source digest. It deliberately changes one generated output and one family
      dimension and proves both make release validation fail. Existing unit conversion tests stay
      green, the warning inventory remains 55, and the full gate passed 986 / 0 / 36.
- [ ] **Visual:** open Import LAS and expand the recognized-vocabulary disclosure. Confirm the
      version/count summary is readable, the complete family/unit list remains usable in the
      560-pixel dialog, and the collapsed state does not distract from set/depth decisions.
- [ ] **Manual:** compare representative delivery mnemonics and unit spellings against the generated
      documentation. Record any missing alias as a new source-reviewed registry change; do not add
      a guessed synonym from memory.
- [ ] **Field:** exercise the accepted vocabulary across pilot deliveries and confirm unknown tokens
      stay observable and unconverted. This migration preserved the exact live population and added
      no alias, family, conversion factor, endpoint or default.

## 2026-08-13 — G2 SB-INS-018: missing units cannot become mappings

- [x] **Automated:** SB-INS-T23 sends an absent unit, an empty unit, placeholder symbols and an
      empty-to-empty mapping row through one serialized `missing_unit` state. It proves generic
      import preparation stores no placeholder, all four mapping rows register zero conversions,
      and a valid `mm`-to-`in` length row still registers so a catch-all refusal cannot pass lazily.
      The focused missing-unit, observed-token and unit-registry suites are green; the full gate
      passed 985 / 0 / 36.
- [ ] **Visual:** import fixtures carrying empty and placeholder unit forms and inspect the completed
      import presentation. Confirm absence is readable and is never presented as a successful
      conversion. Typed IPC is not proof that the interface communicates the state well.
- [ ] **Manual:** inspect stored curve metadata and the import audit together. Confirm placeholders
      do not leak into the stored unit field while the original observed spelling remains available
      for diagnosis.
- [ ] **Field:** exercise representative pilot deliveries containing genuinely absent or placeholder
      units and confirm no false mapping is produced. This increment adds no alias, conversion
      factor, physical endpoint or petrophysical default.

## 2026-08-13 — G2 SB-INS-017: observed unit and encoding evidence survives interpretation

- [x] **Automated:** SB-INS-T21 imports exact `mV` and `mv` spellings through the product LAS
      path, proves both spellings survive in stored curve metadata, permits only registered `mV`
      to acquire the canonical electric-potential interpretation, and requires the unaliased pair
      to emit a drift warning. SB-INS-T22 loads a declared CP1252 parameter pack carrying byte
      `0x92`, reconstructs every original byte from exported typed provenance, and proves a false
      UTF-8 declaration refuses. Focused unit, parser and pack suites are green; the full gate
      passed 985 / 0 / 36.
- [ ] **Visual:** import a LAS carrying the case-variant pair and inspect the completed import
      report. Confirm both raw spellings and the explicit drift warning remain readable; a unit
      test or stored row is not evidence that the UI presents them well.
- [ ] **Manual:** load a CP1252 pack through the governed product surface and inspect declared
      versus decoded encoding plus source-byte provenance. Confirm a contradictory declaration
      blocks before rows become available.
- [ ] **Field:** exercise representative pilot deliveries and confirm registry aliases cover only
      reviewed source spellings. This increment adds no universal case rule, encoding default,
      conversion factor, physical endpoint or petrophysical default.

## 2026-08-13 — G2 SB-INS-016: typed unit registry is enforced before launch

- [x] **Automated:** startup now validates every shipping canonical token and conversion bridge
      before constructing Tauri. SB-INS-T19 still proves the recognised `md` permeability to `m`
      length bridge refuses before arithmetic; renamed SB-INS-T20 proves startup enforcement,
      exact cited `mm` to `in` and `us/m` to `us/ft` conversions, and NaN preservation. The dead
      validator warning was closed, not silenced; live warning inventory is now 55. The full gate
      passed 983 / 0 / 36.
- [ ] **Visual:** deliberately corrupting the compiled registry is not an ordinary UI scenario.
      If a startup problem surface later replaces the fail-fast launch refusal, visually confirm
      it names the invalid tokens and quantity kinds; do not infer that from the unit tests.
- [ ] **Manual:** import real same-kind convertible units and confirm the conversion provenance is
      visible, then present an unknown token and confirm it remains unconverted rather than guessed.
- [ ] **Field:** confirm representative pilot imports use only reviewed same-kind bridges. This
      increment adds no unit alias, factor, physical endpoint or default.

## 2026-08-13 — G2 SB-INS-015: ambiguous parameter packs refuse at the product boundary

- [x] **Automated:** SB-INS-T17 now crosses the semantic ID of one real shipping-schema row with
      another row's ordinal through the registered load command and requires the refusal to name
      the file, pack row and both schema rows. SB-INS-T18 sends missing-ordinal, duplicate-key,
      unsupported-version and empty-key files through the same command; all four refuse and none
      returns a partial pack. No existing guard or error detail was loosened. The full gate passed
      983 / 0 / 36.
- [ ] **Visual:** no parameter-pack picker or refusal panel exists yet. If later work adds one,
      confirm the exact backend error is visible without replacing IDs/ordinals with ambiguous
      labels; automated command tests are not visual evidence.
- [ ] **Manual:** exercise all five refusal shapes from a future governed pack-selection surface
      and confirm no value can be applied after any failed load.
- [ ] **Field:** pack values remain unapplied to computation. Field acceptance waits for the later
      typed-unit, observed-token, generated-registry and attestation/provenance contracts.

## 2026-08-13 — G2 SB-INS-014: parameter-pack identity is product-reachable

- [x] **Automated:** the backend now derives every configurable module row from the shipping
      manifest as `module.argument` plus a one-based configurable-row ordinal, and versions that
      exact manifest with a deterministic SHA-256. The Tauri command accepts a module and file,
      never a frontend-supplied schema. SB-INS-T16 loads two identical labels through that command's
      production function and proves both exact keys resolve while a crossed pair does not. The
      full gate passed 983 / 0 / 36.
- [ ] **Visual:** no pack-selection UI is introduced by this increment. If a later increment adds
      one, confirm labels are presented as labels while semantic ID, ordinal, schema version and
      source file remain inspectable; do not record this checkbox from the IPC test.
- [ ] **Manual:** call the schema and load commands with a duplicate-label fixture and confirm both
      returned rows remain visibly distinct. This is optional review evidence, not a substitute for
      the automated identity contract.
- [ ] **Field:** no parameter-pack value is applied to a computation yet. Do not field-approve pack
      application until SB-INS-015 through SB-INS-020 close mismatch, typed-unit, observed-token,
      generated-registry and attestation/provenance boundaries.

## 2026-08-13 — G2 SB-CORE-044: Tier-C release policy remains source/legal-blocked

- [x] **Automated inventory:** the distributed-dependency notice now reads Cargo's normal edges
      and npm's installed production graph instead of sweeping hoisted development tools. The
      generated result records 292 Rust crates, 111 npm packages, six MPL-family attention items
      and zero undeclared licences. A deliberate stale-file probe was rejected, regeneration
      restored the file, and the full gate now enforces `--check`. This closes only the dependency
      inventory slice; it is not legal clearance and is not a whole SB-CORE-044 correctness test.
- [ ] **Primary-source/legal blocker:** digitized chart payloads, the vendor-merged endpoint
      library and four branded theme identities still ship under unresolved routes. Obtain a
      counsel-approved existing route or remove/independently re-source the chart payloads; supply
      exact per-value primary custody for the endpoint library; and obtain permission or approve a
      neutral/user-owned theme route. Counsel must also review the generated dependency attention
      items. Until then SB-CORE-044 is BLOCKED, not “mostly done.”
- [ ] **Manual:** review `docs/IP_PROVENANCE.md` beside the actual paid-pilot file manifest and
      record counsel/source evidence per row. Jauhar's product approval cannot substitute for a
      missing primary source or legal disposition.
- [ ] **Field:** no field workflow can clear an IP/provenance defect. After the source/legal routes
      close, Gate 4 may confirm that removed assets are absent or approved replacements are the
      ones actually installed; it must not be used as retroactive clearance.

## 2026-08-13 — G2 SB-CORE-036: cancellation controls tell the truth

- [x] **Automated:** the owned correctness test executes an unobserved late click as Completed,
      an observed request as Cancelled, and a monolithic job as non-cancellable. It inventories
      all seven live `run_job` families, the one manual workflow-chain registration, every worker
      observer, and both visible Cancel surfaces. Reverification caught and repaired the chain's
      final-step race: a click after the last worker had completed and committed can no longer
      relabel that run as Cancelled. Focused test: 1 passed / 0 failed / 0 ignored. Full gate:
      983 passed / 0 failed / 36 ignored; Rust retains the owned 56-warning inventory.
- [ ] **Visual:** while one cancellable job and one monolithic job are active, confirm Processing
      shows Cancel only on the first and “can't be interrupted” on the second. For a workflow
      chain, confirm the dialog and Processing panel remain synchronized and terminal text is
      readable. No screenshot is claimed as behavioral proof.
- [ ] **Manual:** cancel LAS import, an equation/workflow run, Monte Carlo, ML and SandiMin during
      actual work; confirm each stops after its documented polling boundary and clearly retains
      any completed partial results. Also click Cancel too late on a nearly finished workflow and
      confirm it reports Completed rather than denying already committed work.
- [ ] **Field:** repeat the mid-run and late-click cases on a sanitized legally controlled pilot
      project, retain the job/result receipt, and confirm reopened persisted results match the
      terminal message. Automated inventory coverage does not count as field acceptance.

Everything below is implemented, unit/integration-tested, and browser-smoke-tested,
but has **not** been clicked through in the real desktop app with real field data.
Work through this list when you have time, marking items as you go.
Marks: **`[x]` = confirmed done** (works as described); `[ ]` = not yet checked. If something is
**wrong**, tell me directly (like your 540-well notes) and I'll fix it and log it in
**ROADMAP.md §4 (Field-review backlog)**.

## 2026-08-13 — G2 SB-CORE-035: backend-owned well scope

- [x] **Automated:** ActiveGroup, named Group and All are resolved from current DuckDB membership
      inside each operation; Explicit active/pinned/selection/custom ids are existence-checked.
      The owned correctness test changes membership after selector construction, pins both allowed
      alternatives and refusal cases, and inventories all 36 live scoped Rust command boundaries.
      `tsc --noEmit` pins every typed TypeScript caller. Per-well parameter and accepted
      autocorrelation writes authorize the current scope; their undo/redo targets the exact
      historical wells. No solver math, parameter default or computed-curve write discipline moved.
      Full gate: 982 passed / 0 failed / 36 ignored in 159s; Rust retains the owned 56-warning
      inventory.
- [ ] **Visual:** open Workflow, Reframe, Statistics, Crossplot, Histogram, Pickett, Dashboard,
      Reports, ML, rock-typing and marker-autocorrelation surfaces; confirm their existing Run on
      controls/counts still paint correctly, and that a backend refusal remains readable in its
      status area. This is a UI wiring increment, so no screenshot is claimed as behavioral proof.
- [ ] **Manual:** with two saved groups, leave a scope-bearing pane open, change membership, then
      run it. Confirm Group/ActiveGroup uses the new membership, All remains all wells, and an
      Explicit pinned/selection/custom scope stays explicit. Repeat one parameter edit and one
      accepted autocorrelation; verify undo/redo returns the exact historical wells.
- [ ] **Field:** repeat the representative import → QC → VSH → POR → SAT → pay → review → export
      chain on a sanitized legally controlled pilot project and retain the scope/operation receipt.
      Automated scope isolation does not count as field acceptance.

## 2026-08-13 — G2 SB-CORE-015: DLIS round-trip closure is source-blocked

- [ ] **Automated implementation:** blocked, not declared green from the working LAS subset. The
      complete `export::tests` module passes 13 / 0 / 0 and re-proves non-default LAS values,
      feet/metres declarations and mandatory self-reader refusal. No DLIS writer exists, so the
      named T15 and the universal T16 cannot execute. No file-existence or internal-`Result` test was
      substituted for a semantic round trip. Full gate: 981 passed / 0 failed / 36 ignored in 150s;
      Rust retains the owned 56-warning inventory.
- [ ] **Source acquisition:** obtain the normative API RP66 V1 multi-dimensional writer sections
      named by `21_data-io.md` §7.2 A-1 and approve DLIS export scope. Do not derive the writer from
      `dlisio` behavior: the chapter explicitly warns that doing so can produce a file readable only
      by the implementation it copied.
- [ ] **Visual/manual and field:** after a sourced writer exists, export one non-default DLIS fixture,
      re-import it in a fresh SandiBumi project, and compare values, units, nulls, index conventions
      and user-visible success/refusal. This remains Gate 4 evidence and is not inferred from LAS.

## 2026-08-13 — G2 SB-CORE-013: cited disagreement stays beside the choice and with the run

- [x] **Automated:** the exact DEC-003 pilot registry covers 15 contested topics without turning
      any disclosed vendor position into a SandiBumi default. The owned correctness test pins every
      value/absence, source, tier and editor binding; exact and ranged matches against an unmatched
      interpreter choice; real persisted VSH ancestry from both sides; custody retention; and all
      three pay-cutoff decisions. Old ancestry remains readable. Full gate: 981 passed / 0 failed /
      36 ignored; Rust retains the owned 56-warning inventory.
- [x] **Visual:** a real debug Tauri window ran with an isolated config, WebView profile and
      temporary project. At 802×632, the maximized VSH editor kept GR_MA and GR_SH source panels
      collapsed by default; expanding GR_MA alone showed Techlog, Geolog and IP values with their
      tier/source lines. The first inspection exposed `.param-sources-body { display:flex }`
      overriding `hidden`; the explicit hidden selector was added and the two states were rechecked.
      This is visual evidence, not field evidence. The app closed cleanly with no corrupt backup.
      Environment policy refused deletion of the inert sandbox before executing the command, so
      `C:\Users\ARUNIKA\AppData\Local\Temp\sandibumi-visual-core013-00c660f7a5bb44208b1d4735eb6aff13`
      remains for manual cleanup; no stronger delete was attempted.
- [ ] **Manual:** Jauhar opens the VSH, porosity, saturation, cutoff, report, dashboard, Results QC,
      Monte Carlo and workflow editors; confirms each prompt is adjacent to the value it qualifies;
      expands selected panels; runs one cited value and one own value; and inspects the persisted
      decision in Curve Catalog and one delivered ancestry disclosure.
- [ ] **Field:** Gate 4 retains one sanitized legally controlled pilot project showing the same
      selected decision after reopen and in the delivered artifact. Post-pilot corpus expansion is
      deferred; this increment does not claim all product domains or all recorded disagreements.

## 2026-08-13 — G2 SB-CORE-011: the recorded project re-run is byte-identical

- [x] **Automated:** the active T16 imports a repository-controlled two-well LAS2 delivery, copies
      that raw project into isolated databases, and runs the same DEC-003 representative VSH →
      porosity → saturation → pay-summary chain. It requires exact bytemuck-packed depth/value
      bytes, exact serialized pay-summary bytes and identical scientifically material ancestry;
      a third run with changed recorded `Rw` must change both outputs. The initial RED run found
      that reopening a modern import invoked a legacy backfill and generated fresh duplicate RAW
      identities. Modern import now records completion atomically with its native generic writes,
      so identical project copies cite identical inputs. Full gate: 980 passed / 0 failed /
      36 ignored.
- [ ] **Visual/manual:** use Save As on one representative project, replay the recorded chain in
      both copies, inspect final-curve ancestry and compare the pay table/export. This is Jauhar's
      review and is not inferred from the automated byte comparison.
- [ ] **Field:** during Gate 4, retain a sanitized legally controlled two-well delivery, the exact
      input/parameter record and both output receipts. No client, field, block, basin, operator,
      well or project identity is stored in this automated fixture.

## 2026-08-13 — G2 SB-CORE-010: every computed curve carries complete ancestry

- [x] **Automated:** every production computed-curve writer now requires a validated, per-well,
      versioned ancestry record: module/version; effective input curve, well and log-set identities;
      parameter values and named sources; zone scope; explicit HUMAN/AUTOMATED session actor;
      timestamp; and output derivation. A missing actor/source or an ancestry-free writer refuses
      before allocating a version or replacing current rows. Computed edits and undos create new
      versions; raw edits remain reversible in their own store. T14 inventories production Rust
      writers and proves both complete-record success and missing-custody refusal; T15 proves the
      exact record survives Save As/reopen. Full gate: 979 passed / 0 failed / 36 ignored.
- [x] **Automated deliverables:** Curve Catalog/Inspector exposes the record on demand. LAS,
      standalone and ordinary PNG/SVG/PDF plots, report PDFs and Office exports embed or print the
      backend-resolved record; no frontend-supplied ancestry can replace project truth. The same
      complete record remains attached to current and archived curve versions.
- [ ] **Visual/manual and field:** in the desktop app, enter one HUMAN and one AUTOMATED session
      operator, run the approved raw-to-pay chain, inspect ancestry in Curve Catalog/Inspector, then
      open the delivered LAS, plot, report and Office files and a reopened Save As project. Confirm
      the values are readable and identical. This remains Jauhar's review; no manual or field pass is
      inferred from the automated gate.

## 2026-08-13 — G2 SB-CORE-007: universal parity is blocked on two contract boundaries

- [ ] **Automated implementation:** blocked, not partially declared green. RED discovery against the
      live registry found the real `RHO_MA` 2.645/2.65 conflict and eleven repeated declared output
      keys, including physically different VSH, porosity, saturation and permeability methods. The
      discovery tests were removed after collecting the inventory: T19/T20/T23 remain missing rather
      than being weakened into snapshots or committed red. Full gate: 977 passed / 0 failed / 36
      ignored.
- [ ] **Contract decision:** define how T23 treats a producer whose required parameters deliberately
      ship `ABSENT`, because the required no-parameter fixture cannot run it. Separately define the
      distinction between a canonical method output and an explicitly user-renamed working curve;
      different methods and the categorical `SW_METHOD` flag must not be forced numerically equal.
- [ ] **Visual/manual and field:** not eligible until the universal registry contract is executable.
      When it is, inspect unique default output names, explicit intentional replacement, saved-chain
      migration and formation-temperature ownership before accepting any field result.

## 2026-08-13 — G2 SB-CORE-006: one saturation name now selects one equation

- [x] **Automated:** the standalone modules and SandiMin share the same Archie, parameterized
      Indonesia, Bardon-Pied and modified-SLB implementations. Canonical equation IDs replace bare
      vendor adjectives in new selectors and provenance; legacy values are input-only aliases that
      retain their old equation. Every saturation run emits a categorical `SW_METHOD` curve whose
      exact numeric code resolves through the backend-owned equation catalog; missing results carry
      `f32::NAN`. The two named correctness tests independently check the cited equations, engine
      parity, labels, documentation, persisted method identity, finite flags and missing flags. Full
      gate: 977 passed / 0 failed / 36 ignored.
- [ ] **Visual/manual:** in the desktop app, inspect the standalone typed-Simandoux selector and the
      SandiMin Sw-equation selector; confirm the canonical ID leads each label, method-specific inputs
      appear only on their equation, and `SW_METHOD` is inventoried as a class curve rather than a
      continuous quantity. This remains Jauhar's review, not an automated pass.
- [ ] **Field:** during Gate 4, run the same cited fixture through both engines and retain the UI,
      current curves, log-set provenance and exported method flag. The complete foreign-import alias
      table and whole saturation-family output inventory remain their own SB-SAT requirements.

## 2026-08-13 — G2 SB-CORE-005: endpoint provenance remains blocked, not cosmetically labelled

- [ ] **Automated implementation:** blocked. History begins with the 27-row endpoint matrix already
      described as merged; the preserved IP and reference tables corroborate many numbers but do not
      identify which source supplied every shipped value. No production field or green snapshot test
      was added, because either would convert missing custody into a defended invariant.
- [ ] **Source/legal resolution:** rebuild coherent libraries from exact vendor assets or primary
      references, ship every unresolved value ABSENT, retain per-value source through the UI and
      deliverable, add SB-MIN-T09, and close CLAIM-012 with counsel before first sale.
- [ ] **Visual/manual and field:** not eligible until the automated custody contract exists. A badge
      reading only “vendor-derived” would not make the current within-row merge auditable.

## 2026-08-13 — G2 SB-CORE-004: every numeric default is sourced or absent

- [x] **Automated:** every registered numeric parameter carries a named source beside its finite
      default or exact `ABSENT` beside an empty value. The complete registry fails its build gate on
      an omission; required absent values refuse before computation; branch-only values are demanded
      only by the method that consumes them; the UI renders the same custody state. Full gate: 975
      passed / 0 failed / 36 ignored.
- [ ] **Visual/manual:** open representative dialogs for one cited default, one required ABSENT value
      and one branch-specific ABSENT value. Are the source and refusal guidance readable before Run,
      and does changing methods hide requirements that branch does not consume?
- [ ] **Field:** during Gate 4, run one cited-default case and one interpreter-supplied ABSENT case
      using representative pilot data; retain the dialog, refusal/success and run-history evidence.

## 2026-08-13 — G2 SB-CORE-003: sourced preconditions refuse before computation

- [x] **Automated:** module manifests can carry source-bearing enumeration, per-sample range,
      branch-conditional, required-companion and relational conditions; the public runner evaluates
      them before dispatch and returns the condition id, offending value, expected rule, statement
      and source. The UI renders the same statement/branch/source beside the field. The live linear-
      GR method now refuses unknown ids, cited range violations and inverted endpoints while its
      valid control still returns 0.5. Full gate: 973 passed / 0 failed / 36 ignored.
- [ ] **Visual/manual:** open the VSH-from-GR dialog and confirm each condition and source is readable
      beside its field; attempt an inverted endpoint pair and confirm the refusal is actionable
      without obscuring the values the interpreter entered.
- [ ] **Field:** during Gate 4, run one valid and one invalid-precondition case for every selected
      pilot method and retain the UI/run-history evidence. SB-CORE-003 remains BLOCKED until that
      method inventory is complete; this increment proves the mechanism and first live method only.

## 2026-08-12 — G2 SB-CORE-002: degraded results stay visibly degraded

- [x] **Automated:** all seven owned correctness proofs remain green after SB-CORE-001. They inspect
      the failed job and returned Monte Carlo error, atomic import rollback and named per-file error,
      degraded PDF and batch record, rendered absent-versus-zero pay rows, partial/all-failed ML
      status plus History, the stats-only Dashboard refusal, and the zero-contributor ML warning.
      Full gate: 972 passed / 0 failed / 36 ignored.
- [ ] **Visual/manual:** exercise each applicable degraded path in the desktop app and confirm the
      warning or refusal appears where the user reads the result, while its clean control remains
      visually distinct.
- [ ] **Field:** force one representative pilot-workflow failure and one partial result with real
      delivery data, retain the resulting UI/export/history evidence, and confirm no clean result
      claim survives.

## 2026-08-12 — G2 SB-CORE-001: no depth-bearing path invents metres

- [x] **Automated:** the deterministic-module registry classifies every live module, an undeclared
      unit refuses every registered depth-dependent module while an independent module still runs,
      the shared Monte Carlo planner uses the same guard, and the reusable metres fallback has been
      removed from imports, saturation-height fitting, LAS/Office exports and image-depth handling.
      Metre-qualified temperature, shift and splice parameters now produce the same physical result
      when the project stores its depth index in feet. Full gate: 972 passed / 0 failed / 36 ignored.
- [ ] **Visual/manual:** in an undeclared legacy project, exercise a depth-dependent module, core and
      curve-table import, array/image import, saturation-height fit, LAS export, workbook, report and
      deck. Does every path name the missing declaration and point to Data Conventions, while a
      depth-independent calculation remains available?
- [ ] **Field:** repeat a representative metre project and foot project from import through compute
      and deliverable export, confirming that the native declared unit is carried without an
      invisible metres reinterpretation.

## 2026-08-09 — SB-CORE-002: degraded results remain visibly degraded

- [ ] Exercise one failed Monte Carlo chain, one partial full-curve import, uninterpreted and true
      zero pay rows, partial and all-failed ML runs, a Pay Summary section failure during batch PDF
      export, the stats-only Dashboard, and a zero-contributor training selection. Does each visible
      surface name its degradation without a clean success or false zero, and does the batch PDF
      remain written while its Pay Summary failure also appears in the run record?

## 2026-08-09 — SB-CORE-001: depth-dependent saturation-height maths carries the project unit

- [ ] Run the same physical Skelt–Harrison case in metre- and foot-declared projects. Does SWH
      remain identical and does HAFWL remain recorded in metres? In a project whose depth unit is
      undeclared, does Saturation-Height refuse before attempting to resolve its input curves?

## 2026-08-09 — SB-PLT-035: clay-overlay parity is characterized, not overstated

- [ ] Compare the interactive Thomas–Stieber overlay endpoints with the batch module equations.
      Do the laminated and dispersed endpoint constructions still agree algebraically? Confirm the
      UI's current duplicate formula is not presented as a governed-equation call.

## 2026-08-09 — SB-CORE-044: Tier-C policy is auditable but partial

- [ ] Review `docs/IP_PROVENANCE.md`. Does it retain the same-increment maintenance rule, blocked
      Tier-C treatment, publication re-derivation doctrine and primary-source fallback routes?
      Confirm the current known-asset register is not presented as exhaustive capability coverage.

## 2026-08-09 — SB-CORE-042: the machine gate remains manually invoked

- [ ] Run `tools\check.ps1` and confirm it executes the verification matrix, frontend acceptance
      tests/build and Rust tests. Confirm the repository still has no automatic per-change workflow
      and this PARTIAL gate is not presented as CI.

## 2026-08-09 — SB-INS-018: missing unit spellings create no mapping

- [ ] Load absent, empty, `-`, `?` and empty-to-empty unit fixtures. Do all remain unmapped rather
      than creating a successful bridge? Confirm the current no-mapping state is not described as
      the still-absent richer typed missing-unit record.

## 2026-08-09 — SB-INS-017: raw unit and encoding tokens remain observable

- [ ] Import a Windows-1252 LAS carrying distinct `mV` and `mv` raw unit spellings. Are the chosen
      encoding and both raw spellings retained before canonical interpretation? Confirm the current
      case fold is not presented as an explicit alias or drift-warning implementation.

## 2026-08-09 — SB-INS-021: support-report fragments are honestly classified

- [ ] Inspect the current installation-support payload. Does it expose interpreter selection,
      candidate reasons, package versions and the capability matrix without project data or secret
      fields? Confirm it is not presented as the still-absent full release/configuration report.

## 2026-08-09 — SB-INS-007: remediation targets the selected interpreter

- [ ] For each unavailable optional package, copy the remediation command. Does it name the exact
      selected executable and distribution, and offer a re-probe after installation?

## 2026-08-09 — SB-INS-006: missing packages fail at preflight

- [ ] Open DLIS import with the selected interpreter missing `dlisio`. Is the exact package and
      interpreter reported before any file parser subprocess starts?

## 2026-08-09 — SB-INS-002: native work remains available without Python

- [ ] On a machine with no discoverable Python, launch and open a project, run a native module,
      render a histogram and inspect native export formats. Do only Python-backed capabilities
      report unavailable?

## 2026-08-09 — SB-PLT-030: plot canvases remain keyboard reachable

- [ ] Focus an interactive canvas and use the arrow and zoom keys. Does the viewport change while
      the current accessible label remains present, and does closing the plot remove the handler?

## 2026-08-09 — SB-PLT-029: stale plot builds cannot replace active content

- [ ] Trigger two plot builds and let the older request finish last. Is the older content disposed
      before panel mutation, leaving only the newest generation visible?

## 2026-08-09 — SB-PLT-028: partial layer separation is explicit

- [ ] Pan, hover and change Z-colour options on a crossplot. Does the invariant draw remain
      separate from transient redraw work, and does Z-colour recomputation follow data/options?
      Confirm this does not imply that every required plot layer and transformed array is memoized.

## 2026-08-09 — SB-PLT-026: export-route labels are honestly classified

- [ ] Open a canvas plot's export menu. Are SVG and PDF explicitly marked vector, and does Print
      still use a PNG image? Confirm the current Print label does not yet claim the required
      explicit raster disclosure.

## 2026-08-09 — SB-PLT-025: future template fields survive normalization

- [ ] Apply a crossplot template carrying an unknown future field. Is the field preserved rather
      than silently discarded while the known options are normalized?

## 2026-08-09 — SB-PLT-018: linked-selection limits are explicit

- [ ] Brush exact depths in one plot, then brush a different scope. Does the second brush replace
      the first and clearing remove it? Confirm the current PARTIAL state is not described as the
      required coexistence of named, coloured, revision-bound, persistable selections.

## 2026-08-09 — SB-PLT-010: regression coverage is honestly classified

- [ ] Fit the arithmetic line `y=2+3x` for `x=1…5`. Does the panel still return intercept 2,
      slope 3, R² 1 and five valid pairs? Confirm this four-value result is not presented as the
      still-absent versioned record of model, method, transformed space, exclusions, interval,
      wells and source revisions.

## 2026-08-09 — SB-PLT-009: statistics coverage is honestly classified

- [ ] Inspect a statistics result for `[1,2,3,NaN,+∞]`. Does it still report count 3,
      mean 2 and median 2? Confirm the current panel does not imply that this arithmetic-only
      summary already carries the population, interval, selection, exclusion, interpolation and
      standard-deviation metadata that remain absent from the PARTIAL implementation.

## 2026-08-09 — SB-CORE-040: verification is indexed by capability

- [ ] Open `docs/VERIFICATION_MATRIX.md` and look up LAS import, Monte Carlo and machine
      learning. Does each row state whether real-well exercise is absent, partial or complete,
      show checked versus listed scenarios and give only the dated `REVIEW.md` ledger date?
      Temporarily change one mapped checklist mark in a copy of `REVIEW.md`: does `--check`
      reject the stale matrix until the committed generator is run again?

## 2026-08-09 — SB-PLT-031: every plot reduction is disclosed and exportable

- [ ] Exceed the context-point budget and the context/well-preview/fit-scatter legend limits.
      Do the live surfaces state displayed and original counts? Does **Manifest** export a
      validated JSON record with those counts, every represented well's point algorithm,
      absent-well reasons and any refusal? Enter more than eight histogram percentiles: are all
      valid unique values retained instead of a silent prefix? Exceed Vega's categorical-group
      ceiling: does it refuse with the observed count rather than returning the first groups?

## 2026-08-09 — SB-PLT-023: chart rendering requires complete provenance

- [ ] Select a chart overlay whose record lacks its source revision/date. Does the chart
      payload stay absent from both the live plot and its vector/raster draw paths, with a
      visible provenance refusal? Do the selector and status identify the block? Inspect a
      complete fixture: does persisted plot state retain chart ID/title/type, X/Y quantity and
      unit, citation, publisher, revision/date, applicable digitizer, approved derivation path,
      payload SHA-256 and the actual orientation/unit transform?

## 2026-08-09 — SB-PLT-020: plot-derived parameter writes are traceable and undoable

- [ ] Promote a crossplot handle/marker, histogram marker and two-point Pickett fit to zone
      parameters. Does every write store the stable plot ID/type, concrete axis quantity/unit/
      conversion/revision bindings, viewport, selection revision, `[lo,hi)` interval, method,
      applicable fit record, OS user and UTC timestamp? Does Ctrl+Z restore the exact prior row
      (or clear a previously absent row), and does a missing source record refuse the write?

## 2026-08-09 — SB-PLT-016: depth-step reconciliation is conservative

- [ ] Plot inputs at equal steps, then at 0.5/1.0, then at 0.5/0.8. Does the first
      proceed with factor 1, the exact multiple decimate to the coarsest step and report
      factor 2, and the non-integer pair refuse with a route to the DIO resampling workflow?
      For `[100,101)`, are depths 100 and 100.5 retained while 101 is excluded?

## 2026-08-09 — SB-PLT-015: decimation preserves identity and provenance

- [ ] Reduce eligible source indices 0…10 at stride 4. Does the view use exactly
      `[0,4,8,10]`, report 11 original / 4 displayed, name the stride-from-first algorithm,
      and say the final endpoint was forced? Inspect depth, X, Y and Z: were all four
      sampled with that same index vector, and is the reduced view never labelled complete?

## 2026-08-09 — SB-PLT-014: multi-well budget follows finite-pair screening

- [ ] Request two context wells where one required Y curve is a full-length NaN vector.
      Does that well appear in the outcome as absent with zero finite aligned pairs and zero
      quota? Does the valid well receive the available budget only after screening, with its
      first and final eligible source samples retained? If the budget cannot retain every
      represented well's endpoints, is the context plot refused explicitly?

## 2026-08-09 — SB-PLT-013: range policy is channel-specific

- [ ] Plot non-finite values and non-positive values on logarithmic X/Y axes. Are they
      excluded and counted separately? Move finite X/Y values beyond the viewport: are they
      clipped and counted without changing statistics or source arrays? For Z colour and
      array waveforms, are overflow values clamped only in derived display values, counted,
      and—on Z—marked at the low/high colour edge?

## 2026-08-09 — SB-PLT-011: Pickett discloses only identifiable parameters

- [ ] Fit a two-point Pickett water trend without separately sourced `a` or `Rw`. Does the
      plot label `m` and `a·Rw`, state that `a` and `Rw` are not separately identified, omit
      saturation guides, and avoid emitting either factor? Supply one factor with provenance:
      is the other then derived without introducing a default?

## 2026-08-09 — SB-PLT-008: percentile probability is not range position

- [ ] Load or export typed percentage values. Is `PercentileP=130` refused while
      `RangePositionPct=130` and `RangePositionPct=-5` retain their exact values and explicit type
      tags, with no shared clamping path?

## 2026-08-09 — SB-PLT-006: histogram binning is canonical

- [ ] Plot values exactly on every edge, including the final upper endpoint. Are bins half-open
      except for that final inclusion, with the displayed total equal to their sum? Add NaN and
      infinity: are both excluded and reported separately? Do saved options accept 1–200 bins and
      default to the cited 50?

## 2026-08-09 — SB-PLT-005: unit-limit rows require a dimensional audit

- [ ] Pass the documented attenuation converted-unit exemplar through the audit. Does it report
      the cited 6.56× divergence and remain disabled with the missing registered-conversion reason?
      Does a row activate only when its converted value exactly follows a reviewed typed transform?

## 2026-08-09 — SB-PLT-004: validity filtering and display clipping stay distinct

- [ ] Set narrow display axes without enabling validity filtering. Does the plot count hidden
      marks while leaving `n`, statistics and the fit population unchanged? Then enable explicit
      validity bounds: do `n`, statistics and fits change, with the excluded count shown separately?

## 2026-08-09 — SB-PLT-003: chart overlays require typed quantity and units

- [ ] Select a chart whose aliases match the chosen mnemonics but whose resolved quantity or unit
      is incompatible. Is drawing refused? With a registered compatible conversion, is the chart
      placed in the source-axis coordinates and are source/display units plus the affine transform
      retained in the binding rather than treating a mnemonic match as authorization?

## 2026-08-09 — SB-PLT-002: axis ranges expose one precedence tier

- [ ] Open a crossplot whose user, header, audited-family and finite-data ranges differ. Does the
      user range win and does the plot/export label that tier? Clear the user range: does the header
      display range win next, while a validity range never becomes a display range?

## 2026-08-09 — SB-PLT-001: plot requests retain concrete per-well curve resolution

- [ ] Open a histogram or crossplot over more than one well, then inspect the plot binding record.
      Does it retain the semantic request separately from every well's resolved curve ID, mnemonic,
      typed quantity, units, conversion, finite sample count, resolution reason and SHA-256 source
      revision? Does a required curve with no typed resolution refuse the plot instead of silently
      substituting a same-named source?

## 2026-08-09 — SB-INS-023: release qualification covers every serviced Windows target

- [ ] Supply the release-time Microsoft-serviced Windows 11 x64 feature-release inventory and run
      Pro and Enterprise through all nine clean-machine scenarios. Does one failed or omitted
      scenario name its exact release, edition and scenario and keep the installer unpublished?

## 2026-08-09 — SB-INS-016: unit conversions are quantity-typed

- [ ] Validate the unit registry with the demonstrated `md` → `m` bridge, then with `mm` → `in`
      and `us/m` → `us/ft`. Is permeability-to-length refused before arithmetic, while the two
      same-kind samples reproduce only their existing cited factors and preserve missing values?

## 2026-08-09 — SB-INS-015: ambiguous parameter packs stop before activation

- [ ] Against a module-supplied schema, try a crossed ID/ordinal row, missing ordinal, duplicate
      key, unsupported schema version and empty semantic key. Does every load refuse the whole
      file and name its file/row conflict, without guessing or returning a partial pack?

## 2026-08-09 — SB-INS-014: parameter rows are keyed, not name-joined

- [ ] Load a parameter-pack fixture with two identical display labels but distinct semantic IDs
      and ordinals. Can each row be retrieved by either key, while a crossed ID/ordinal pair finds
      nothing and the display label never participates in selection?

## 2026-08-09 — SB-INS-010: installed settings stay immutable

- [ ] On a clean user profile, launch once and inspect the per-user `settings.json`, edit its
      settings map, then relaunch. Does the edit survive, does origin retain the installed
      template's application version and SHA-256, and is the bundled template byte-identical?

## 2026-08-09 — SB-INS-008: offline deployment is release-gated end to end

- [ ] With public network access blocked, have IT silently deploy the application MSI and the
      separately signed qualified Python pack per machine. Does the resolver select the pack's
      application-local interpreter, do all six claimed capabilities pass, and does the release
      gate retain a zero-request network trace plus the pack and release-lock digests?

## 2026-08-09 — SB-INS-005: one session interpreter has an explainable resolution

- [ ] Configure a higher-priority Python without NumPy and a lower-priority Python with NumPy,
      then open Project → Help → Prerequisites. Is the lower candidate's exact `sys.executable`
      selected once for every capability, with the precedence rule and higher rejection shown?

## 2026-08-09 — SB-INS-004: dependency detection and messages share one manifest

- [ ] Open Project → Help → Prerequisites with the qualified Python pack, note the selected
      executable and observed package versions, then remove one required package and re-probe.
      Does only its dependent capability become unavailable, with remediation derived from the
      same equation/DLIS/plate/workbook/document/deck package matrix?

## 2026-08-09 — SB-INS-003: prerequisite claims are capability-level and manifest-derived

- [ ] On a machine with no Python, open Project → Help → Prerequisites. Are Python equations,
      DLIS import, spreadsheet plate extraction, workbook export, document export and deck
      export each named unavailable, while native project open, plotting and exports remain
      available? Do the MSI notice and release-note fragment name the same package rows?

## 2026-08-09 — SB-INS-001: the Windows release is a qualified per-machine MSI

- [ ] On a clean supported Windows 11 x64 image with neither Rust nor Node.js, have IT deploy
      the signed MSI silently under the system context, then launch SandiBumi as a standard
      user. Does the release evidence record the final MSI SHA-256 and build commit, and do the
      installed name, version and identifier exactly match `tauri.conf.json`?

## 2026-08-09 — SB-DIO-020: the malformed duplicate exemplar requires a policy

- [ ] Import `bad_dup_depth.las` without choosing a duplicate-depth policy. Does it name all
      five repeated rows, require a policy, and commit no well, while `bad_null_depth.las`
      retains its existing all-null-depth refusal?

## 2026-08-09 — SB-DIO-012: non-increasing index confirmation is tested independently

- [ ] Import a 400-row LAS whose final depth decreases by half a depth unit without repeating
      an earlier depth. Is row 400 refused until delivery order is explicitly accepted, then
      imported with the accepted conflict retained in the audit result?

## 2026-08-09 — SB-DIO-013: unit-qualified depth headers remain explicit aliases

- [ ] Import core tables whose second column is `Depth (m)`, `DEPTH (FT)` and bare `DEPTH`.
      Does each resolve that named column as the index, while a second column with an unrelated
      name is still refused instead of being guessed by position?

## 2026-08-09 — SB-DIO-061: every public reader runs the malformed corpus

- [ ] Add a new public parser entry point and run the gate. Does the source-derived inventory
      refuse it until an explicit malformed-corpus adapter is registered, while every registered
      reader still runs against every corpus fixture?

## 2026-08-09 — SB-DIO-062: text encoding is detected and reported

- [ ] Import the same ASCII LAS encoded as UTF-16LE once with a BOM and once without. Do both
      import with the same rows, and does each result explicitly name `UTF-16LE with BOM` or
      `UTF-16LE without BOM`? Do UTF-16BE, UTF-8 and a Windows-1252 description likewise name the
      decoder actually chosen?

## 2026-08-09 — SB-DIO-060: format recognition follows content

- [ ] Open a comma-delimited table deliberately named `.las` through Intake. Is it read as a
      delimited table, with the `.las` disagreement reported? Inspect a headerless BIFF5 stream:
      is `09 08 06 00` named as the reason BIFF5 was chosen? For a PK container, does the report
      name the workbook entries that distinguish XLSX from generic ZIP? BIFF5 table loading remains
      unavailable until the out-of-scope P2 BIFF reader ships.

## 2026-08-09 — SB-DIO-052: final and working curves stay distinguishable

- [ ] Export a well whose `RAW` and `FINAL` generic sets both hold `PHIE`. Are both curves in the
      LAS (`PHIE` plus the collision-safe state suffix), and does `~Other` name each export
      mnemonic, its original `PHIE`, its source set, and its `working` or `final` state? Does the
      visible result count both states?

## 2026-08-10 — imported LAS native grid, catalog statistics, and interaction cost

- [ ] Import two LAS deliveries for one well whose sets have visibly different source spacing.
      Does Curve Catalog retain each file's exact source row count under its own set, without a
      merged/common depth count?
- [ ] In Layout Properties, select the set explicitly for each displayed curve. Does the legend
      name that set, does the trace follow that delivery rather than the well's standard grid, and
      does progressively zooming in reveal more source detail instead of magnifying a fixed coarse
      whole-well reduction? Pan back out: does the whole-well depth extent remain unchanged?
- [ ] Change the mnemonic in Layout Properties. Does its Set list contain only deliveries carrying
      that mnemonic, and does an incompatible prior set clear instead of silently drawing nothing?
- [ ] Export the set-qualified layout to SVG and PDF. Does each exported trace use the same source
      set and native depths as the on-screen curve, including a set whose interval does not overlap
      the well's standard frame?
- [ ] For a curve containing finite and missing samples, does Curve Catalog show total rows plus
      finite Valid/Missing counts, and do min/max/mean agree with an independent finite-only check?
      For an all-missing curve, are the counts present while min/max/mean remain blank?
- [ ] Expand and collapse the same well, set and data-set nodes repeatedly. Are pure toggles
      immediate and free of a sample-table scan, while an import/edit/delete invalidates the cache
      and shows the changed inventory on the next expansion?
- [ ] On the same release build and the same local LAS copies, time import before and after this
      repair. Record file count, depth rows, curve count, storage medium and both wall times; do not
      substitute the 89.6 → 61.2 s debug observation for this machine-specific field result.
- [ ] Import a deliberately invalid delivery that fails during the all-channel constraint write.
      Is the result a failure, with no well, standard rows, generic metadata, native samples or
      adopted project depth unit left behind?
- [ ] Import a deep LAS whose source depth tokens advance exactly by its decimal STEP. Is there no
      false **possibly re-gridded** warning from f32 rounding? Change one adjacent source token by a
      real amount: does the warning name the declared spacing, actual spacing and first row pair?

## 2026-08-09 — SB-DIO-050: declared STEP is checked against the samples

- [ ] Import a LAS declaring `STEP.M 0.5` whose actual index spacing is `1.0`. Does the import
      succeed with **possibly re-gridded** naming both values and the first mismatching row pair?
      Does an otherwise identical file declaring `STEP.M 1.0` avoid that warning? The separate
      round-interval detector remains absent because the chapter cites no threshold for it.

## 2026-08-09 — SB-DIO-049: every writer must pass its own reader

- [ ] Export LAS from a feet project. Does success explicitly state that the SandiBumi reader
      self-check passed? In a debugger or focused test, corrupt one ASCII row or falsely label the
      feet index as metres: does export return **LAS self-check failed** instead of success or a
      warning?

## 2026-08-09 — SB-DIO-047: precision reductions are declared

- [ ] Import a core-analysis table containing more numeric precision than `f32` can retain. Does
      the result and History state `f64 numeric parse → f32 storage` and count only the values that
      changed? Export a LAS containing a value beyond four decimal places: does the result state
      `f32 storage → fixed-decimal-4 LAS text`, count the reduction, and carry the same declaration
      in `~Other`?

## 2026-08-09 — SB-DIO-045: multi-well DLIS containers stay separated

- [ ] Import a DLIS whose logical files name three different source wells. Before any write, does
      the confirmation show each source well and its logical-file ordinals mapped to a separate new
      project well? After confirming, are there three wells with no curve merged across them? If two
      logical files name the same source well, do they remain two runs on one mapped well?

## 2026-08-09 — SB-DIO-041: LAS 3.0 and unread sections are explicit

- [ ] Import a LAS declaring `VERS. 3.0` and carrying `~Core_Data` and `~Tops`. Does the result
      explicitly say **LAS 3.0 recognized** and name both sections as unread, while the ordinary
      `~Curve`/`~ASCII` log array still imports correctly?

## 2026-08-09 — SB-DIO-039: DLIS sentinel screening is per-channel and counted

- [ ] Import a DLIS channel containing a legitimate `-999.25`. With its exact mnemonic entered
      under **Keep LAS sentinel values in**, does the sample survive and is the exception recorded?
      Without the exception, is that sample missing and does the result name the channel, count the
      one deletion, and state the LAS-sentinel fallback rule?

## 2026-08-09 — SB-DIO-037: partial DLIS loads are explicit and named

- [ ] Import a DLIS with one readable scalar channel and one encrypted, unsupported, or otherwise
      unreadable channel. Does the result say **Partially imported**, show the loaded-versus-declared
      count, and name the omitted channel with its reason instead of reporting ordinary success?

## 2026-08-09 — SB-DIO-036: duplicate DLIS mnemonics never default to merge

- [ ] Import a DLIS whose mnemonic/frame already exists anywhere on the selected well. Does the
      preflight name every existing set/run and write nothing until you answer? Does **Keep
      separate** place an exact RAW collision in a fresh set and record that choice per curve?
      Is there no merge-into-existing default or action?

## 2026-08-09 — SB-DIO-035: DLIS interval extension needs confirmation

- [ ] Import a DLIS whose converted index extends above or below the selected well's existing
      finite interval. Does the first pass write nothing and name both the declared and incoming
      extents? If you decline, is the existing interval unchanged? If you explicitly accept, does
      the completed import retain the same conflict in its audit notes?

## 2026-08-09 — SB-DIO-033: curve selections are named saved objects

- [ ] In Reframe, create **PRIMARY INPUTS** with ordered members `RHOB, GR`, save it, close/reopen
      the pane, and inspect it. Is the same name, explicit selected mode and member order present?
      Is Reframe blocked when no saved selection is chosen, with no blank-means-all fallback?

## 2026-08-09 — SB-DIO-032: substitutions are named, accepted and recorded

- [ ] In Reframe, request a curve the selected source does not hold, choose an exact source
      mnemonic in **Use instead**, and leave **Accept substitution** unchecked. Is the run refused
      before writing? After checking it, does the output retain the substitute's own mnemonic and
      show the requested-to-substitute decision in the set provenance and run note?

## 2026-08-09 — SB-DIO-030: alias renames preserve both identities

- [ ] Import a LAS carrying only `SGR`. Does its standard curve appear as `GR`, while the generic
      catalog still says `SGR` with family `GR`? Does the result and visible note name the exact
      firing row `GR_ALIASES: SGR -> GR`, even though no alias competed with it?

## 2026-08-09 — SB-DIO-029: MS/FT has no default meaning

- [ ] Import a file declaring `MS/FT` with no answer. Is nothing committed until that exact file
      is designated as microseconds/ft or millisiemens/ft? Does the former retain DT and canonical
      `us/ft`, while the latter stays familyless, with either answer recorded on the file result?

## 2026-08-09 — SB-DIO-028: every unit factor carries its arithmetic

- [ ] Inspect the unit-rule query/code table. Does every factor carry a reproducible derivation,
      including corrected `MEQ/L -> meq/mL ×10^-3` from `1 L = 10^3 mL`? Does that corrected row
      remain confirmation-only because §7.1 O-2 says affected files may already contain meq/mL?

## 2026-08-09 — SB-DIO-027: the vendor PPG-to-density alias is rejected

- [ ] Import a `RHOZ.PPG` column. Is it retained verbatim but excluded from both standard RHOB
      and the generic RHOB family, with `density.units: PPG -> density` named as the rejected entry
      and an explicit quantity-designation requirement?

## 2026-08-09 — SB-DIO-026: unit transforms carry and apply offsets

- [ ] Import `FTEMP.DEGF` containing 200 and 32 °F. Does the audit show factor `1/1.8` and
      source-space offset `-32`, with stored canonical `DEGC` values 93.33 and 0 rather than the
      multiplicative-only 111.11 and 17.78?

## 2026-08-09 — SB-DIO-025: conversion coverage and pass-throughs are explicit

- [ ] Query the unit-conversion capability list. Does it return exactly CALI, BS, RHOB, DRHO,
      NPHI, DT, DTS and TEMP? Import a density with an unsupported declared unit; is its value and unit
      retained verbatim while the result flags it as unconverted?

## 2026-08-09 — SB-DIO-024: automatic unit conversions are visible

- [ ] Import a LAS sonic declared in `US/M`. Does the result name the curve, source unit,
      canonical `us/ft` unit and factor `0.3048`, while the stored generic-curve samples are
      actually converted by that same factor?

## 2026-08-09 — SB-DIO-022: export re-sampling defaults off

- [ ] Export a well whose stored depths are irregular. Does the LAS `~ASCII` block retain every
      stored depth and its paired value exactly, with no regular grid or interpolated samples
      introduced at the default settings?

## 2026-08-09 — SB-DIO-020: duplicate depths have a declared policy

- [ ] Import a LAS with repeated depths. Does it commit nothing until you choose keep-first,
      keep-last, mean or refuse? For a resolving choice, does the result name the policy and exact
      repeated-row count, with standard and generic curves using the same samples?

## 2026-08-09 — SB-DIO-019: stored depths cannot be re-declared

- [ ] In a metre project that already holds curves, try changing the project depth unit to feet.
      Is it refused with the affected well count, while the declaration and every stored depth
      remain unchanged?

## 2026-08-09 — SB-DIO-018: canonical units have one owner

- [ ] Export a well carrying one curve from every family. Does each LAS curve unit exactly match
      `curves::FAMILIES`, including spelling and case, with no writer-owned standard-unit table?

## 2026-08-09 — SB-DIO-013: unknown table indexes are designated

- [ ] Open a delimited/core table whose depth column has an unfamiliar header. Does it refuse to
      pick column 0, commit nothing, and require you to mark the index? After designation, does the
      result record the selected column and `user_designation` mechanism?

## 2026-08-09 — SB-DIO-012: descending indexes require a decision

- [ ] Import a LAS whose index first decreases late in the file. Is the import blocked before any
      well commits, with the exact data row named? After explicitly accepting the delivered order,
      does it commit while retaining that row in the result audit?

## 2026-08-09 — SB-DIO-011: index aliases keep their namespaces

- [ ] Inspect the LAS, core-table and tops index aliases. Does each path cite its source? Is `TVD`
      still accepted for tops, but held in a separate TVD namespace rather than any MD/reference
      alias list?

## 2026-08-09 — SB-DIO-010: index resolution names its mechanism

- [ ] Import one LAS whose second column is named `MD`, and inspect the per-file result. Does the
      first column remain the index and say `positional_guarantee`? On a structurally declared
      table, does the `REFERENCE` column win even when it is not first?

## 2026-08-09 — SB-DIO-009: competing aliases are auditable

- [ ] Import a LAS in which two aliases target the same standard curve and one has greater finite
      coverage. Does the per-file result name the chosen mnemonic, every passed-over mnemonic and
      the finite-sample count for each, while targets with only one match stay out of the report?

## 2026-08-09 — SB-DIO-006: null exceptions are many-to-many

- [ ] Load one null-exception entry carrying several channel-name patterns. Are all patterns
      active? For an entry declared `NoNull`, does a genuine `-999.25` amplitude survive while
      the same value on an unset channel is screened normally?

## 2026-08-09 — SB-DIO-005: null conventions are plural and per channel

- [ ] Import a file with two channels whose declared null lists differ. Does each channel lose
      only its own declared values, including more than one value on a channel, while the other
      channel's sentinels survive as measurements?

## 2026-08-09 — SB-DIO-002: the default export format honours the sentinel

- [ ] Open the export format list. Is LAS 2.0 the single default and marked as honouring the
      project sentinel? If a fixed-null format is added later, does the picker name that limitation
      instead of presenting it as equivalent?

## 2026-08-09 — SB-DIO-001: one declared sentinel reaches every writer

- [ ] Set the project's export sentinel to a non-default finite value, export a LAS with missing
      samples, and inspect both `~W NULL` and `~A`. Do they carry only that declared value, with
      no writer-owned `-999.25` leaking into the file?

## 2026-08-09 — SB-DIO-061: malformed inputs have a shared regression corpus

- [ ] Add a synthetic malformed fixture under `src-tauri/tests/fixtures/dio-malformed`, then run
      the gate. Does every registered parser/Intake reader execute it, and do failures name the
      file, line or row/column, affected count and failed rule without a panic or hang?

## 2026-08-09 — SB-DIO-054: every discarded DLIS item is reported

- [ ] Import a DLIS with one unreadable frame and one readable frame. Is the good frame imported
      while every skipped frame/channel/row is named with a count and rule? Does an all-skipped
      file fail instead of reporting an empty success?

## 2026-08-09 — SB-DIO-055: LAS export omissions are explicit

- [ ] Export a well with many imported curves, including one duplicate mnemonic and one curve on
      another frame. Are aligned curves written, and are both omissions named with the same reason
      in the completion message and the LAS `~O` section?

## 2026-08-09 — SB-DIO-051: provenance travels inside LAS deliverables

- [ ] Open an exported LAS at `~O`. Is every curve labelled measured or computed, does each
      computed curve name its method and parameter values, and does each fitted curve carry its
      ordered inputs, training/runtime record and model-artifact SHA-256?

## 2026-08-09 — SB-DIO-017: LAS exports declare their actual depth unit

- [ ] Export and re-import one feet project and one metre project. Do `STRT`, `STOP`, `STEP`
      and `DEPT` all carry the project unit, with the depth numbers surviving unchanged?

## 2026-08-09 — SB-DIO-016: DLIS index units are reconciled

- [ ] Import a feet-indexed DLIS into a metre project. Are depths converted, with the index
      channel's own `UNITS` attribute named in the result? Does an undeclared index refuse until
      its file unit is explicitly confirmed?

## 2026-08-09 — SB-DIO-015: an undeclared depth unit now refuses

- [ ] Import a LAS with no unit on its depth curve. Does it refuse until **File depth unit when
      undeclared** is explicitly set, even when the project already has a depth unit?

## 2026-08-09 — SB-DIO-004: one null-recognition rule

- [ ] Import a LAS whose declared null differs slightly after decimal formatting. Is it missing,
      while a nearby real reading outside the relative tolerance remains unchanged?

## 2026-08-05 — Fluorescence off the UV frame, and PDF import is off

You said not to build PDF import — you will export the plates yourself. That is recorded in
`docs/plan_core_photo.md` §4a with the design kept in a fold, and it unblocks everything else,
so this increment is the UV measure you asked for: *"extract inferred payzone from UV"*.

**Advance ▸ Core Imaging ▸ Photo Log… now has a Light choice.** Pick **🔦 Ultraviolet** and the
three daylight measures are replaced by fluorescence.

- [ ] Open **Photo Log…** on a well with a UV delivery. Is there a **Light** row with ☀ Daylight and
      🔦 Ultraviolet, defaulting to Daylight?
- [ ] Switch to Ultraviolet. Does a colour-band card appear — the same hue wheel the Pore Area tool
      uses, plus a **Pale limit** slider?
- [ ] Switch back to Daylight. Does the card disappear and the trace behave exactly as it did before?
      **Nothing about the daylight path should have changed.**
- [ ] Read the trace on the UV delivery. You should get **CPHOTO_FLUOR** (how much of each slab
      fluoresces, 0–1) and **CPHOTO_FLUOR_I** (how bright it is).
- [ ] Does the note say it is an **INFERRED SHOW, not a pay flag**? That sentence is the point of the
      whole thing — minerals, drilling-fluid additives and dead oil all fluoresce.

**Tune the band against ONE photograph, and judge it against your own show descriptions —
never by whether the average looks about right.** That is not a platitude: on your petrography
delivery a colour band could be tuned until its median landed within 5% of the petrographer's own
count while the per-plate agreement stayed at −0.10.

- [ ] Drag the hue ends and the brightness floor until the mask matches what you can see glowing.
      Does the trace change shape sensibly, rather than just moving up and down?
- [ ] Compare a bright interval against a show you already know about. Does it land in the right
      place? **This is the only check that means anything.**

**Two guards, and I want you to try to trip the second one.**

- [ ] Point the Ultraviolet measures at your **DAYLIGHT** delivery on purpose. It should measure, show
      you the numbers, and then **refuse to write**, saying the band claimed at least 95% of every
      slab and naming the wrong-light cause. A daylight frame read as UV would otherwise store a core
      that fluoresces end to end — which reads as a spectacular show.
- [ ] Now the case that must NOT be refused: a **genuinely strong show** over most of a box. That has
      to go through and be written. (My first version of this guard refused exactly that, and the
      test caught it — a half-stained core is the answer this measure exists to give.)
- [ ] A box with **no fluorescence at all** should also go through normally. No show is a real
      reading, and it is what gives the box above it meaning.

**If your show descriptions separate bright yellow-green from dull blue-white:**

- [ ] Click **+ Another kind of fluorescence**. Name the two, and give the dull one a low **Pale
      limit** — white is the absence of colour, so it cannot be set with the brightness floor.
      Each kind gets its own curve (`CPHOTO_FLUOR_BRIGHT`, `CPHOTO_FLUOR_DULL`).
- [ ] **Tell me whether that split is actually how you describe shows.** I deliberately ship ONE
      generic band, because saying the hue split means live-versus-dead oil would be putting an
      interpretation in the software that nothing in the repo can source. If it is your practice,
      say so and I will make it the default.

**One thing to watch:** if you have adjusted exposure, contrast or white balance on the UV
photographs, the run says so by name. `CPHOTO_FLUOR` counts pixels above a fixed brightness, so half
a stop moves the answer — and a white balance picked on a UV frame means nothing anyway, because
there is nothing neutral under a UV lamp.

- [ ] Condition a UV photograph's exposure, then read the trace. Does the note name that photograph?

## 2026-08-01 — A fluid contact now knows its sand, its stack and its fault block

You asked where the calculation parameters live and said they should be at marker level. **They
already are** — RHO_SH, NPHI_SH, the RtC coefficients, M, N, RW, FWL, the cutoffs, all of them, in
`zone_params` keyed by well and marker, with `*` for the whole well. Precedence is: the module's
default → what you type in the dialog → the `*` value → the marker's own value.

**Fluid contacts were the exception, and it was costing numbers.**

- [ ] **A contact now carries the MARKERS it governs.** Two stacked sands with two different
      oil-water contacts used to be pooled into one surface fit — which landed between them and
      then flagged every well as disagreeing with a contact that was never there.
- [ ] **Several sands can share ONE contact.** That was your second question, and it is why the
      markers are a list rather than a single field: a hydraulic unit of three stacked sands is one
      contact governing three markers, and the QC treats it as one surface. The order you type them
      in does not matter.
- [ ] **Compartments, named.** Two fault blocks are not in pressure communication and have no
      reason to sit on the same contact. Give each its name and each is checked on its own.
- [ ] Existing contacts in your projects keep working and get no marker and no compartment —
      nothing stored says which sand or block they were picked in, and I would rather admit that
      than invent it. Fill them in and the QC sharpens immediately.
- [ ] **Plot ▸ Multi-Well ▸ Fluid Contacts…** — a working pane, so it sits beside the correlation
      panel or a log view. Every stored contact in one table, editable in place: type, compartment,
      markers, depth, TVDSS or measured, label. Add and delete, with undo on the delete.
- [ ] **QC section** lists every contact as its own group — type, markers, compartment — and says
      how many wells are on it, its mean level, the spread, and names any well off the surface with
      how far. A group with only one pick says so rather than showing a blank.

### The one that changes saturations

- [ ] **A free-water level lived in two places and nothing checked them against each other**: the
      contact you pick and draw, and the FWL parameter a saturation-height run actually computes
      from. The panel could show one level while every Sw in the report came from another — both
      perfectly plausible numbers.
- [ ] The pane now measures the gap per well and marker, says which one the arithmetic is using,
      and offers one button to copy your picked level across. **It is a copy, not a live link** —
      so a run you did last month still says which number it used — and it is undoable.
- [ ] **A contact picked on measured depth is refused rather than converted**, by name. The stored
      parameter carries no reference of its own, so converting to force a comparison would be
      asserting something the project never said. Re-pick in TVDSS, or set the parameter by hand.

## 2026-08-01 — A four-column core plate, its own Photo Log tool, and a launch screen

From the whole-core delivery you pointed me at. Two findings before anything else:

- **It arrives as PDF**, one file per core, pages alternating white light (plate `1a`) and UV
  (plate `1b`). Nothing in the app can import a PDF yet — that is the first thing standing
  between this suite and your rock, and it is the next increment I would do. Plan in
  `docs/plan_core_photo.md`.
- **Each page is four columns of core, each its own barrel**, with `PRESERVED` gaps printed
  between them and a short last column where recovery ran out. Splitting that into four equal
  parts of one span — all the old "rows of core" setting could do — puts every sample below the
  first gap at the wrong depth.

- [ ] **Advance ▸ Core Imaging ▸ Photo Log…** is a NEW tool: reading the trace and building depth
      strips moved out of Condition Core Photos, as you asked. Conditioning is done once per
      delivery; a trace is read, checked against GR and read again.
- [ ] **Detect columns** measures where the runs of core are and proposes them — it does not
      apply anything, and it never guesses a depth, because nothing in the picture says what depth
      a column of rock came from. The proposal lands in a table you edit, drawn over the picture.
- [ ] **Each column takes its own depth top and base.** A preserved interval stays a GAP in the
      curve instead of depth smeared across it, and a part-filled last column is a short barrel
      rather than a quarter of the plate.
- [ ] **Half a plate labelled is refused, as you type**, not after a run: placing the blank
      columns would mean assuming the core runs on without a break, which is exactly what the
      preserved interval on the same page disproves. Fill them all in, or clear them all.
- [ ] The column table is saved with the project, so working through a delivery plate by plate
      does not mean retyping it. The filmstrip tile says which plates are done.
- [ ] Nothing changed for an ordinary core-box photograph: with no column table it is still
      equal lanes over the picture's own interval, read in order.
- [ ] **Condition Core Photos gains "Recommend conditioning"** — it measures the picture and
      proposes a white balance, exposure and contrast, with the measurement behind each one
      written out. Nothing is applied; Apply is still Apply, and your crop, rotation and corners
      are never touched. "Recommend for the delivery" also tells you whether the run was shot
      under one light or not, which is the question behind "apply this look to the whole run".
- [ ] **It will not recommend Clarity, Sharpen or Denoise** even where the picture would clearly
      benefit — it says so instead, and says what it would cost: local contrast roughly halves the
      darkness contrast the trace is reading. Use them for the eye, not before a trace.
- [ ] **A UV plate is recognised and left alone.** It is meant to be dark and there is nothing
      neutral in it to balance against, so exposure and white balance are declined by name — a
      lift would drown the fluorescence the plate exists to show. A dim white-light frame with a
      tray in it still gets lifted, so it is not just giving up on dark pictures.
- [ ] **A launch screen**: portrait card, artwork in our own colours, the mark, "SandiBumi 2026 ·
      v0.1.0" and the copyright. It only fills the wait that already exists — it appears after
      0.4 s so a fast open never flashes it, and it disappears the instant the project is open.
      It cannot make a start slower.
- [ ] Your UV question is answered as a plan rather than as code: `docs/plan_core_photo.md` §4
      covers a fluorescence measure off the UV plate, a discrete sand/shale curve, and the
      "unfold" shear for dipping beds — with three questions for you at the end.

## 2026-08-01 — Every remaining tool that popped up is now a pane (your sweep)

You asked me to check whether those tools still pop up and make them panes. Six dialogs,
seven ribbon buttons. Verified in the browser: clicking all seven opened dock tabs and
`#modal-root` stayed empty — nothing popped up.

- [ ] **Advance ▸ Petrography ▸ Pore Area…** is a pane. Tune the band, look at the mask,
      check the agreement figure, try another reference plate — all with the plate tracks
      and the Wells pane still visible. The tried-settings table survives while you work.
- [ ] **Advance ▸ Petrography ▸ Plate Details…** is a pane. The scale-bar measurement (⇹ on
      a row) still opens as its own popup and the table stays put behind it.
- [ ] **Advance ▸ Petrography ▸ Condition Plates…** and **Advance ▸ Core Imaging ▸ Core
      Photos…** are **two separate panes**, not one. They are two deliveries with two
      recipes, so opening one no longer loses your place in the other — you can have both
      docked side by side. Verified: a layout save/restore rebuilds each on its own subject.
- [ ] **Data ▸ Core ▸ Register Depth…** is a pane, so the correlogram sits beside the log
      view the decision is actually made from. **One behaviour change here:** Apply used to
      close the dialog. Now it clears the proposal and refreshes the barrel table and the
      history instead — the core has moved, and pressing Apply again on a shift computed
      against the old depths would have doubled it.
- [ ] **Advance ▸ Calibration ▸ Calibrate RtC…** and **Calibrate S…** are panes. The Close
      button is gone (the dock closes a pane); the Run button is unchanged.
- [ ] All seven are in the ＋ menu too, and re-clicking a ribbon button focuses the open pane
      rather than opening a second copy.
- [ ] **A leak fixed on the way past.** The plate filmstrip holds an image-loading observer
      and one object URL per thumbnail — a delivery is hundreds of plates at about a megabyte
      each — and neither Pore Area nor the Mineral Classifier ever released them, because a
      popup has no "closed" hook to release them from. A pane does. If the app used to feel
      heavier the longer you worked through a petrography delivery, this is why.
- [ ] **Picture panes now use the pane's real width.** Form panes are capped at a readable
      column; a filmstrip, a plate preview, a correlogram and an eight-column plate table are
      not forms, so those five opt out. This also widens the **Mineral Classifier**, which had
      the same squeeze. Measured: 1156px of a 1180px pane, no horizontal scrolling.
- [ ] **Still popups on purpose** — tell me if you want any of these moved too: the naming
      prompts (Save Layout/Session As, Open Session), the import wizards (LAS/DLIS set, SCAL,
      Aux, Deviation, Images, Well Locations), the exports (Workbook…, Deck…) and the short
      forms (Shift Core…, Well Header…, Data Sets…). Each of those is filled in once and
      dismissed rather than worked beside a log.

## 2026-08-01 — Mineral Classifier is a pane, and Plug QC is proportional (your catches)

- [ ] **Petrophysics ▸ Petrography ▸ Mineral Classifier…** now opens as a **working pane**,
      not a popup — dock it, split it, leave it open beside the Wells pane while you click
      through a delivery plate by plate. It is also in the ＋ menu as "Mineral Classifier
      (point counts)". Everything it does is unchanged: labels, training, apply, save.
- [ ] **Plug QC** was a pane wearing dialog-era layout — a fixed 180px label column against
      a full-width control, in a pane with no gutter at all, so labels sat flush on the card
      edge. It uses the same two-column form as the module panes now (labels above, controls
      even), with the pane gutter every other tool pane has. Measured: both columns 303px,
      everything inside the content box. The Mercury-saturation row still hides itself
      unless an axis is the throat radius.
- [ ] **Standing rule recorded**: tools open as working panes from now on. `openModal` stays
      only for real interruptions — confirmations, refusals, Help — or when you ask for a
      popup. **Still popups, and I did NOT convert them without your word: Pore Area… and
      Condition Plates… / Condition Core Photos….** Say the word and they follow.
## 2026-08-01 — Five tools come out of the Tools ▾ dropdown (your call)

You asked where the core conditioning menu was — it was buried in Data ▸ Tools ▾, which is
the answer to the question. Nothing is hidden in a dropdown any more if it is part of a
workflow.

- [ ] **Data ▸ Core** (new group): **Register Depth… · Shift Core… · Data Sets…** — the core
      depth job in the order you do it, all three as labelled buttons.
- [ ] **Advance ▸ Core Imaging** (new group): **Core Photos…** — conditioning, the CPHOTO
      darkness/redness/texture traces and the depth strips. It is in Advance because reading
      a log off a photograph is an interpretation method, not data management, and it sits
      next to Petrography so the imaging work is together.
- [ ] **Advance ▸ Petrography** gains **Plate Details…**, beside Condition Plates…, Pore
      Area…, Mineral Classifier… and Plug QC….
- [ ] All five must be GONE from Data ▸ Tools ▾ — that menu now holds Autocorrelate Tops…,
      Well Header… and Compact Project… only. Check nothing you use daily is still in there.
- [ ] Every moved button opens exactly what it opened before — same dialogs, same panes.
- [ ] Note for the record: the Petrography group is in the **Advance** tab. My notes had said
      Petrophysics, which was simply wrong; they are corrected.

## 2026-08-01 — One edge, on the pane you are working in (your call)

The hairline came off every card, and the active pane states itself instead.

- [ ] No panel draws its own outline any more — the doubled line in the gap, and the
      two arcs colliding at the rounded corners, are both gone.
- [ ] **The pane you are working in carries a 2px edge, in the NEUTRAL strong-border
      colour — not the accent.** Click between panels: the edge follows, and nothing
      inside any panel shifts by a pixel while it does (it is an outline, not a
      border, precisely so log tracks and canvases never reflow on a click).
- [ ] It follows the 12px corner, and stays visible across the tab strip at the top.
- [ ] **Check a client skin** — this is the one you flagged. The edge must read as the
      card's own border drawn heavier, never a coloured frame: Schlumberger #a4b3cf
      soft blue-grey rather than the deep #0033a0, Halliburton grey rather than red.
      The accent stays for controls you act on; a pane is a surface, not a control.
      It is also the only line on screen now, since no other card draws one — so it
      does not need colour to be found. If it reads too faint on the cream default
      (measured 1.5:1 against the white card, against 2.1:1 on the near-white skins),
      one step darker is a one-line change.
- [ ] **Worth your eye:** with no hairline, a card is now told apart from the ground by
      its fill alone, and that is a small difference on the near-white skins —
      measured 1.19:1 on the default cream, 1.11:1 on Halliburton, 1.08:1 on dark. The
      7px gap still reads as a groove, but if panel boundaries feel too soft to you on
      any skin, say so: a soft shadow or a hairline on the outer edge only is a
      one-line follow-up, and I would rather you judge it on screen than have me guess.

## 2026-08-01 — Panel top corners were square, not rounded (your catch)

You were right that the corners looked unfinished — the top ones genuinely were square.
The cards carry a 12px radius, but dockview puts a `transform` on the tab strip so it
can scroll, and a composited element like that is NOT clipped by its parent's rounded
corners. The strip painted its own square corners straight over the card.

- [ ] Every panel — Wells, Tops, Processing, Log View, Inspector, and any you open —
      should now show a clean 12px round at **all four** corners, with the cream ground
      visible through them. The top two are the ones that changed.
- [ ] Check it after **splitting** a window and after **dragging a panel** into another
      group: new groups get the same treatment.
- [ ] The ＋ add-panel menu, right-click menus and dialogs must still open normally and
      must NOT be cut off by a panel edge (the fix is deliberately scoped to the tab
      strip so nothing that needs to escape a panel can be trapped by it).
- [ ] Bottom corners needed no change — measured as already correct, including under the
      log view's WebGPU canvas. If any bottom corner still looks square to you, say so:
      that would be a different cause and I'd want to see which panel.

## 2026-08-01 — The brand stops changing colour with the skin (your catch)

The wordmark was painted with `--accent`, so every client skin re-rolled it: SandiBumi
read Halliburton red on that theme, SLB blue on the next. It is a `--brand` token now,
theme-independent by construction and never repeated in any `[data-theme]` block.

- [ ] Switch through **every** theme (Halliburton, Schlumberger, Pertamina, LAPI-ITB,
      white, dark, default): the ribbon wordmark stays the SandiBumi terracotta — the
      look in your image 2 — while the rest of the UI recolours as before.
- [ ] Same on the **boot overlay** (relaunch) and the **start sheet** (close every pane).
- [ ] The logo tile is rounded a little more (5px on the 18px ribbon mark), so its baked
      cream square reads as an intentional logo tile on Halliburton's grey instead of a
      bare block. The mark's own colours are untouched on every theme.
- [ ] Switch **language to Bahasa Indonesia / Basa Sunda**: "SandiBumi" must stay
      "SandiBumi" everywhere (the three surfaces are `data-no-i18n` now).

## 2026-08-01 — Organic increment 5: LAS import, report pane, plot surfaces, and the sweep (1e · 1f · 1c)

The last handoff screens plus the harmonization pass over components the handoff never
named. Nothing functional changed anywhere in this batch.

**Import LAS (1e).**
- [ ] Import a delivery: the set dialog now lists the picked files as a rail (up to six,
      then "+N more"), and carries the footer line "Every import is versioned with
      provenance — re-importing never overwrites RAW." Set-name suggestion unchanged
      (blso*_fprooh → FPROOH).
- [ ] Deliberate deviation: the mockup's 3-step wizard with a mnemonic-mapping table is a
      FEATURE (the app maps mnemonics in Rust, automatically) — not built. Want a manual
      mapping step? That's its own increment, and worth discussing first.

**Report pane (1f).**
- [ ] **Render** is the one primary pill; Save PDF / Word / PNG / Template / Batch are
      secondary pills. The rendered page now floats on the neutral rail with a soft
      shadow — the page itself stays white, since what you preview is the paper.

**Plot surfaces (1c).**
- [ ] Histogram, crossplot, Pickett, the calibration QC scatters and the correlation
      strips now draw their data area on the warm neutral with **white gridlines** —
      card-on-card, like the mockup. Points, axes, overlays: identical.
- [ ] **Dark theme**: plot areas keep dark surfaces and dark gridlines (no white glare).
      Client skins take their own alt tint automatically. Check one plot in dark and one
      in a client skin.
- [ ] The log view's tracks are NOT touched — this is the plot suite only.

**Harmonization sweep (components the handoff never named).**
- [ ] Every remaining primary action button in the app (Composite, Cutoffs, Map apply,
      Monte Carlo, Pickett picks, Zones add…) is a pill now — one shared rule, no
      per-dialog change. Layout Properties buttons match. The crash/startup dialog is
      16px like every other dialog, its buttons pills.

## 2026-08-01 — Organic increment 4: start surfaces (design 1g)

- [ ] Launch the app on a slow open: the boot overlay is the identity column now — 72px
      rounded logo, "SandiBumi" in the display face, one-line description, then the same
      progress bar, elapsed clock and one-time notes as before. A fast open still shows
      nothing (the 400 ms rule is untouched).
- [ ] Close every content pane: the blank canvas is a **start sheet** — wordmark + New
      Project / Open Project pills on the left, RECENT PROJECTS on the right with the
      current one tagged "open now" (disabled), a missing file marked and disabled, and
      the sessions tip card at the bottom.
- [ ] Click a recent project row: it must go through the SAME guard as Project ▸ Recent ▾
      — a running chain still blocks the switch with the same message.
- [ ] New Project / Open Project on the sheet behave exactly like the ribbon tools (they
      are the ribbon tools).

## 2026-08-01 — Organic increment 3: the module pane (design 1d)

One pattern, every manifest-driven module — nothing was written per module (rule 9 holds:
a new module still needs zero frontend work). **Same runner, same validation, same
defaults**; only the form changed shape.

- [ ] Open any module (e.g. **Petrophysics ▸ VSH**): header shows a 34px initial chip +
      the title in the display face + a **? Help** button on the right that opens the
      module's method note. The dockview tab title is unchanged.
- [ ] The well scope now reads **RUN ON** with the modes as a segmented pill and the live
      well count as a tag — this control is shared, so EVERY batch dialog (cutoffs,
      exports, fits, Monte Carlo…) picked up the same look. Spot-check two others.
- [ ] Parameters sit in a **two-column grid**, labels in small caps above each field, and
      a numeric parameter's **unit sits to the right of its input** (it used to be inside
      the label). Narrow the pane: the grid collapses to one column.
- [ ] The sage callout states the precedence rule: values here are whole-well defaults,
      Zones-pane parameters win inside their zones.
- [ ] Footer: **Run VSH** (the module's short name) as a solid pill, last-run status
      right-aligned beside it. Out-of-range values still refuse by name before any run.
- [ ] Deliberate deviation from the mockup: **no "Preview one well" ghost button** — that
      is a new feature (a run that writes nothing), not a restyle. Say the word if you
      want it and it becomes its own increment.

## 2026-08-01 — Organic increment 2: Field Dashboard (design 1b)

The dashboard now matches your mockup. **Every number is the same arithmetic as before** —
the KPI cards read the existing aggregation, they never recompute it. One behaviour
refinement from the mockup: uninterpreted zones now appear GREYED at the bottom of the
grid instead of vanishing with a count.

- [ ] **Petrophysics ▸ Batch ▸ Field Dashboard…** — header row: "Field Dashboard" in the
      display face, and after a Compute a sage tag saying which group and how many wells
      the numbers describe. Export CSV and Compute are pills on the right.
- [ ] Cutoff strip is one rounded band; **Flag and Metric are segmented pills** now
      (active = solid terracotta). Same choices as the old dropdowns — click through
      PAY/RESERVOIR/SAND and the metrics; everything re-renders from the held rows.
- [ ] **Five KPI cards**: Total net (terracotta tint), Total HPV (sage tint), net-weighted
      mean PHIE and SWE, and ZONES EXCLUDED. Check TOTAL NET and TOTAL HPV against the
      By-zone table's Σ columns — they must agree exactly (same helpers).
- [ ] **Uninterpreted zones are greyed at the grid's bottom** — gross keeps its number
      (geometry), net/N-G/averages/HPV show "—" (a zero there would read as computed).
      The footnote under the box plots says they are excluded, never averaged as zero.
      The KPI card count, the greyed rows and the footnote must all agree.
- [ ] CSV export still contains ONLY the interpreted rows — no dashes, no phantom zeros
      in a spreadsheet that has no grey styling to explain them.
- [ ] Top row of the grid (current sort) is highlighted; sorting still works from the
      headers; the PERM no-data warning still appears when it applies.
- [ ] Box plots: terracotta boxes, darker median (thicker than before) — still the same
      quartiles.

## 2026-08-01 — The Organic reskin, increment 1 (your redesign handoff, foundation)

Your `SandiBumi UI Redesign.zip` is now the standing design system; the handoff is banked
in-repo at `docs/design_organic/`. This increment is the foundation — tokens, fonts and
chrome; the per-screen passes (dashboard KPI cards, LAS wizard step pills, report preview
rail, start screen) come next. **No number changed anywhere** — this is look only.

- [ ] The app opens on a **cream ground** with **white rounded panel cards** and visible gaps
      between them. The active ribbon tab is a solid **terracotta pill with white text**; the
      tool area below the tabs is a white rounded card. The brand and every dialog title are
      in the display face (Caprasimo); everything else is Figtree.
- [ ] Make something dirty (edit a value) with a tab other than Project active, then activate
      **Project**: the unsaved dot must be **white on the terracotta pill** — visible, not
      drowned. On the inactive tab it stays red.
- [ ] Buttons are pills everywhere; hover is a **pale terracotta tint**, never grey; a dialog
      has 16px corners and drags by its header as before.
- [ ] Narrow the window: the ribbon overflow chevrons ‹ › still appear INSIDE the rounded
      card and scroll the tools; nothing pokes out of the corners.
- [ ] **Dark theme** keeps its own colours with the new shapes (pills, cards). The five client
      skins (Pertamina/Halliburton/Schlumberger/LAPI-ITB/white) recolor the pills but keep
      the shape — switch through them once.
- [ ] Log tracks, grids, trees and tables kept their density — no new air in data surfaces.
      Canvas text (track headers, axis labels) now renders in Figtree; if anything reads
      worse at 10–11px than Segoe did, say so — the canvas face is one token.
- [ ] Fonts are bundled (`public/fonts/`) — pull the network cable and restart: the faces
      must not fall back.

## 2026-08-01 — Your four answers, applied (numbers moved — check these first)

All four change what a run computes, so these matter more than anything else on the list. The
triage findings are 6, 7, 9 and 16 in `docs/review_triage.md`.

**Temperature is a well property now (finding 6).**

- [ ] **Pre-Calculation ▸ Zones…** — set TEMP_GRAD on a NAMED zone and Run. It must be **refused by
      name**, naming the parameter, the zone, and telling you the `*` scope still works. Is the
      message something you could act on without asking me?
- [ ] Clear that, set TEMP_GRAD on scope **`*`** instead, Run. It must SUCCEED and shift the whole
      trend. This is the route the per-well parameter grid uses, so it has to keep working.
- [ ] Plot FTEMP vs depth: one straight line. No 10 °C step at a formation top any more.
- [ ] **PGRAD on a named zone still works, deliberately** — FPRESS should step at the boundary,
      because a pressure compartment is real. Set it and confirm. If you would rather pressure
      behaved like temperature, say so; it is one flag.
- [ ] Same rule on **Formation Temperature** (`ftemp_grad`): TSURF, TGRAD, BHT and TD_BHT are all
      well-level now.

**A permeability cutoff applies to every well it is asked for (finding 7).**

- [ ] **Cutoffs & Pay Summary**, PERM ≥ something, on a well with **no PERM curve**. Net pay should
      now be **0**, where it used to be full. This is the reserves change — confirm it is what you
      meant.
- [ ] That zero must not look like a wet well. The **Field Dashboard** should name the well above
      the roll-up, and a **report** for it should print a note under the pay table saying the zero
      records an absence of evidence. Both present?
- [ ] With **no** PERM cutoff set, neither note appears and nothing changes.
- [ ] A well that HAS permeability is unaffected, cutoff or not.

**PHIE is floored at 0.001 (finding 16).**

- [ ] Run **Porosity from Density** (or Density-Neutron) over an interval with a tight streak. The
      `PHIE` curve must never go below 0.001 — but `PHIE_DEN` / `PHIE_DN` must still show the
      negative excursion, because that is how you see RHO_MA is wrong. Both true?
- [ ] Pay Summary on the same well: the SAND row's HPV should now be higher than before, and no HPV
      anywhere may be negative.
- [ ] A floored streak must still **fail** the porosity cutoff — it must not have crept into
      RESERVOIR. Compare the RESERVOIR and PAY rows before and after.
- [ ] A well with no PHIE at all still reports "not interpreted", not a column of 0.001.

**Pittman's radii, corrected against the paper (finding 9).**

- [ ] **Rock Typing ▸ Pittman Pore-Throat Radii**, on good sand. PR10 > PR15 > … > PR50 > PR75, all
      the way down now. At 25 % porosity / 100 mD expect PR50 ≈ 2.22 µm and PR75 ≈ 0.27 µm (it used
      to return 2.95 µm for PR75, which was larger than PR50 — impossible in rock).
- [ ] **PR50 changed by about 25 %** on every well you have ever run this on: it was carrying
      Pittman's r45 coefficients. If a study quoted PR50, it needs re-running.
- [ ] **On TIGHT rock the family still crosses over below about 11 % porosity, and that is the
      paper's own arithmetic** — the rows are independent regressions. Try it on a tight interval
      and you should see PR50/PR75 turn back upward. Nothing is clamped, because clamping would
      report radii Pittman never published. Does the module doc's advice (use r25–r35 as APEX in
      tight rock) read clearly enough to act on?
- [ ] **Worth checking your own studies:** anything that picked `r75` as APEX in a tight interval
      built RAPEX and RT_PITT on a row that had turned back up.

## 2026-07-31 — A machine can now drive the real app end to end (optional)

`npm run test:e2e` starts the **built** `sandibumi.exe` and drives it through Tauri's WebDriver
route: a real LAS import, a real module run, a real export, against a real DuckDB file. Five
tests, all passing. It is **optional and never part of the green gate** — `tools\check.ps1` stays
green on a machine with none of it installed. Setup and the reasoning are in
`docs/e2e_harness.md`.

- [ ] **Run it.** `npm run test:e2e`. Expect five green ticks in about 30 seconds. First run on a
      new machine needs `cargo install tauri-driver --locked` through the vcvars pin; msedgedriver
      downloads itself.
- [ ] **It did not touch your project.** This is the one that matters. While it runs, the app opens
      a throwaway project in your temp folder — check the first line it prints (`e2e sandbox: …`).
      Your `src-tauri/project.duckdb` must be untouched: same size, same timestamp. The harness
      asks the running app which project it opened and aborts before any test if the answer is not
      inside that sandbox.
- [ ] **It refuses to run while SandiBumi is open.** Start the app, then run it — it should stop
      immediately and say so, rather than risk confusing your session with a leftover process.
- [ ] **Nothing was force-killed.** After a run there should be no new `.corrupt-backup-*` file in
      `src-tauri/`. The harness checks this itself and fails the run if one appears, keeping the
      sandbox as evidence.
- [ ] **What it cannot do.** It cannot tell you a plot looks right — the log views are a WebGPU
      canvas, and WebDriver sees a rectangle. There are deliberately no pixel assertions. If you
      ever see one added, that is a bug.
- [ ] **After changing the frontend, rebuild before believing it.** The built binary embeds the
      frontend from build time, so a UI test can pass or fail against markup older than your repo.

## 2026-07-31 — Trained models are kept, named and re-runnable

Until now a model died with the run: you could not train on your cored wells and apply **that
same model** to the rest of the field later, and a delivered curve could not say which model
made it. Now it can.

- [ ] **Train and keep.** Petrophysics ▸ ML Models…, pick a supervised task (say regression,
      PHIT or PERM as the target), select your cored wells as training wells, and type a name in
      **Save model as** (e.g. `PERM_FROM_CORE`). Run. The status line should end with
      *"model saved as 'PERM_FROM_CORE'"* and it should appear in the **Saved models** list with
      its algorithm, its input curves, how many samples and from how many wells, and its size.
- [ ] **Apply it to wells it has never seen.** Change the well scope to the uncored wells, then
      press **Apply to scope** on the saved model. Nothing is refitted — check the Processing
      monitor says "apply saved model", not "training".
- [ ] **The result is traceable.** The new curves' log set records `ml:apply:<model name>` with
      the model id, so months later you can answer "which model produced this?".
- [ ] **A missing input is named.** Apply a model to a well that lacks one of its input curves —
      it should tell you **which curve by name**, not just "missing input curve data".
- [ ] **Retraining does not overwrite.** Run again with the same name: it should save as
      `..._1` and say so. A model an existing delivered curve was made with must never be
      silently replaced.
- [ ] **The scaler went with it.** If you trained with "Standardize" on, the applied curve should
      look right on wells whose GR/RHOB ranges differ from the training wells. (This is the
      subtle one — re-standardizing on the new wells would give a different, wrong answer.)
- [ ] **Rename and Delete** work, and Delete asks first. Deleting a model does not remove curves
      it already produced — but they can no longer be reproduced from it.
- [ ] **Only supervised models are offered.** The "Save model as" field disappears for
      clustering and reduction, because those are fitted on the very wells they are applied to.
- [ ] **Project size.** A random forest can be a few MB. Check the size column; if your project
      grows more than you like, Data ▸ Tools ▸ Compact Project still works.

## 2026-07-31 — The field as an asset-team deck

Last of the office deliverables. **Plot ▸ Deliverables ▸ Deck…** builds a PowerPoint from the
data — you chose matplotlib figures over pasted composite pages, so that is what it does.

- [ ] **Export a deck.** Pick a scope, a title, who is presenting, and the cutoff level
      (**PAY** by default). Open it in PowerPoint. Seven-ish slides: title, scope and cutoffs,
      field roll-up by zone, net + HPV per zone, N/G–PHIE–SWE distributions, well ranking, and
      any well that produced nothing.
- [ ] **The box plots should match the Field Dashboard.** They are the same statistics — the
      app computes them and matplotlib only draws them, precisely so the two can't disagree.
      Compare a zone's PHIE box against the dashboard. **If they differ, tell me.**
- [ ] **Each box says how many wells are behind it** (`n=` under the label). A box from three
      wells is not the same statement as one from ninety.
- [ ] **A zone nobody interpreted gets no bar — not a zero bar.** It still gets its axis label
      so you can see it exists. Check this on a zone you know is uninterpreted.
- [ ] **The cutoff level is stated on the title slide.** A deck speaks about one level; SAND and
      RESERVOIR stay in the workbook. Try switching to RESERVOIR and confirm the whole deck
      follows.
- [ ] **Long tables continue on more slides** ("1 of 3") rather than shrinking. If your field
      has many zones, check the table is still readable from the back of a room.
- [ ] **The well ranking says what it cut** ("Top 20 of 44 interpreted wells"). A silent top-N
      would read as the whole field.
- [ ] **The last slide names the wells that produced nothing.** That is the counterpart to
      every average on the slides before it.
- [ ] **Everything is editable** — real PowerPoint tables and text, and the charts are pictures
      you can resize or replace.
- [ ] **Without the packages.** If python-pptx or matplotlib is missing, the dialog names which
      one before the save dialog. You have both.

## 2026-07-31 — The report as an editable Word document (+ an encoding bug fixed)

Second of the office deliverables. The report pane now has **Save Word…** next to Save PDF…,
and the **Batch** button has a format select beside it (`as PDF` / `as Word`) so a whole field
can go out either way.

- [ ] **Save Word on one well.** Open the Report pane, set your title/author/methodology as
      usual, press **Save Word…**. Note you do NOT have to press Render first — the document
      carries no log plots, so there is nothing to preview.
- [ ] **It is genuinely editable.** Open the `.docx` and change the methodology wording, drop
      in your client's letterhead, restyle the tables. That's the whole point of this format —
      the PDF stays the deliverable that must not be altered.
- [ ] **The tables match the PDF.** Cover, methodology, zone parameters (zone name and depths
      printed once per zone, not repeated down every parameter row), pay summary with the
      cutoffs in the heading. Export both for the same well and compare — they read from the
      same numbers, so any disagreement is a bug.
- [ ] **A zone with no parameters is still listed.** Dropping it would tell a client the zone
      was not evaluated when it simply took the defaults.
- [ ] **A dash, not a blank, in the document.** Where the workbook leaves an uninterpreted cell
      empty, the Word document prints "-" like the PDF does. That difference is deliberate:
      Excel's arithmetic skips an empty cell, a document has no arithmetic and your eye needs
      the mark.
- [ ] **No composite log pages in the Word file** — on purpose, and the document says so at the
      end. A composite at 1:200 stops being at 1:200 the moment somebody drags its corner in
      Word. If you want them in there anyway, tell me and I'll add a rasterized appendix.
- [ ] **Nothing is written back.** Unlike the PDF path (which writes FLAG curves as it renders),
      the Word export touches nothing in the project.
- [ ] **Batch as Word.** Set the select to `as Word`, pick a folder, and check you get one
      `<WELL>_report.docx` per well in scope.
- [ ] **Names with special characters.** This one is a **bug fix worth testing**: import a
      picture whose file path or folder contains a non-ASCII character (an en dash, `é`, or an
      Indonesian folder name), and check it now imports. Before this, the import failed with
      "No such file or directory" naming a filename you never had — text was being read from
      the wrong character set on the way into Python. The same bug would have mangled a well
      name in the Word report.

## 2026-07-31 — The study as an Excel workbook

First of the office deliverables. Until now `export.rs` wrote LAS and everything else left as
a PDF, an SVG or a flat CSV — so the table an asset team actually works in was re-typed by
hand. **Plot ▸ Deliverables ▸ Workbook…** writes it directly.

- [ ] **Export a workbook.** Pick a scope (group / ★ pinned / selection / all), check the
      cutoffs it opened with — they should be **the same numbers the pay summary and the report
      use**, because all three read one saved default. Press Export, choose a filename, open it
      in Excel.
- [ ] **The numbers are numbers.** Click a Net or PHIE cell: the formula bar should show
      `12.5` / `0.185`, not text. Sort, filter and pivot the Pay Summary sheet — if any of that
      refuses to work, a column came through as text and I want to know.
- [ ] **A blank is not a zero.** Find a well you have NOT interpreted yet (no VSH/PHIE/SWE).
      Its net, N/G, PHIE, SWE and HPV cells must be **empty**, while Gross still shows a number
      (geometry is known either way) and Samples shows 0. Select the Net column: Excel's status
      bar average must ignore those rows. **This is the one thing I most want checked** — a 0.00
      there would quietly drag down a field average.
- [ ] **The Summary sheet is the audit trail.** It should name the cutoffs actually used, the
      depth unit, the export time, and — if any well produced nothing — list those wells by name
      under "Well without results". A well that contributed nothing must never just be missing.
- [ ] **Two N/G columns on the Field Summary sheet.** `N/G (field)` is Σnet/Σgross (the
      volumetric ratio for a resource number); `Mean N/G` is the average of the per-well values,
      which is what the **Field Dashboard** shows. Compare a zone against the dashboard: Mean
      N/G, PHIE and SWE should match it. If they do not, tell me — they read the same rows.
- [ ] **Zones read shallow to deep**, not alphabetically, on the Field Summary sheet.
- [ ] **PAY rows are tinted** on both table sheets, so the pay level stands out from SAND and
      RESERVOIR — all three levels are exported, not just PAY.
- [ ] **Nothing is written back.** Export a workbook, then check the Processing history and the
      Wells pane: no new FLAG curves, no new log-set version. Saving a spreadsheet must not
      count as an interpretation run.
- [ ] **Zone Parameters sheet.** The interval parameters your interpretation used, one row each.
      Zone `*` is the whole-well default. Check a well where you set a per-zone `RW` or `M`.
- [ ] **Without xlsxwriter.** If Python or the package is missing, the dialog says so **before**
      the save dialog and names the interpreter to `pip install` into. It should never fail
      after you have already chosen a filename.
- [ ] **Field scale.** Try it on a few hundred wells. It runs as a job, so the **Processing**
      monitor should show it while it works.

## 2026-07-31 — Pictures in their own track (thin sections, core photos)

Your ask: *"images in separate tracks, such petrography thin section, core photo, or any
picture format that can be adjustable (later we should have capablites to digitize it as
well)"*. Done for the DISPLAY half; digitizing is deliberately a later phase.

- [ ] **Import a folder of thin sections.** **Data ▸ Import Data ▸ Import Images…** with a
      well selected, pick several files. The wizard lists every file with its true pixel size
      and **the depth it read from the file name** — nothing is stored until you press Import.
      Check the guesses: `BLSO-01_1523.50.jpg` should read 1523.50, and a plain `BLSO-01.jpg`
      should read NOTHING (an amber "required" box), because a two-digit well number must
      never be mistaken for a depth. Fix any depth in the table before importing.
- [ ] **A photographed interval.** A file named `..._1523.5-1524.0.jpg` should come in with
      BOTH a depth and a base. You can also type a base by hand. Leave the base empty for a
      thin section — a plug has no thickness, and the empty cell is what says so.
- [ ] **Show them.** Right-click a log view ▸ **Layout Properties…**, add a track, set
      **Track type = Images**, then **＋ Add image series** and pick your dataset. The plates
      appear at their depths with a leader line to the track edge.
- [ ] **Adjustable, as you asked.** In that same editor try: **Width of track** (how big the
      plate is), **Align** left/centre/right, **Placement** — *Anchored at depth* (fixed size,
      centred on the sample) vs *Scaled to interval* (the picture spans its own top-to-base,
      only meaningful when it has a base depth), and for a scaled one **Fit** *Whole picture*
      vs *Fill and crop*. Nothing ever squashes the picture out of shape — tell me if you
      ever see a stretched plate.
- [ ] **Overlapping plates.** Zoom out until two thin sections would collide. The deeper one
      **disappears and leaves a short tick** at its true depth rather than sliding down to fit.
      Zoom back in and it returns. That is deliberate — say if you would rather they stacked.
- [ ] **Print it.** **Plot ▸ Composite…** with that layout — the plates must appear in the PDF
      and in the SVG at the same place and size as on screen. Open the SVG somewhere else
      (a browser) to confirm the pictures travel INSIDE the file, not as broken links.
- [ ] **A second delivery does not double the plates.** Import the same folder again with the
      same delivery name. It should land as `NAME_1`, become the live one, and the track must
      show **one** set of plates, not two. **Data ▸ Tools ▸ Data Sets…** has a new **Images**
      section — switch back to the first delivery and the track follows. The Wells pane ▸
      twisty also lists **Images** per well; double-click switches the live one, and expanding
      a delivery lists each plate with its depth and size.
- [ ] **Project size is visible.** The Data Sets dialog and the tree both show MB per delivery
      — the only store where the cost is worth showing. Stored pictures are capped at 2400 px
      on the long edge by default; the wizard lets you raise it (or set 0 for full resolution)
      if you need to zoom further, at the cost of a much larger project file. Tell me if 2400
      is too soft for your thin sections.
- [ ] **TIFF.** If your petrographer delivers TIFF, it needs Pillow (`pip install pillow`).
      With Pillow present TIFF imports and displays normally. Without it, the wizard says so by
      name rather than failing quietly, and a non-JPEG prints as a **labelled frame** in the
      PDF so a deliverable can be checked against the delivery list.
- [ ] **All three languages** translate the new labels (Import Images…, Images, Placement,
      Align, Fit, Frame, Caption…); technical terms stay English as always.

## 2026-07-30 — Quick-access buttons become labelled Project-tab tools

Your ask: *"those QAT buttons should become labelled tools, together with performance and
processing button moved from petrophysics tabs"*. Done — and the icon strip left of the ribbon
tabs is **gone**, not duplicated.

- [ ] **The icon strip is gone and nothing was lost.** Launch the app: there is no row of small
      icons left of the **Project / Data / Petrophysics / …** tabs. Open **Project** — all seven
      of those buttons are there with words under them:
      **Project** — Open Project… / New Project… / **Save Project As…** / Recent ▾
      **Session** — Save Session… / Open Session…
      **Edit** — Undo / Redo
      **Monitor** — History / **Processing** / **Performance**
      **Appearance** — Theme · **Language** · **Help** — Help
      (The tabstrip is 24px tall and the ribbon body is 80px — that height difference is the
      whole reason these could not carry captions where they used to live.)
- [ ] **Processing and Performance are no longer in Petrophysics.** Open **Petrophysics ▸ Batch**:
      it now holds Workflow… / Monte Carlo… / Field Dashboard… only. Both moved buttons open the
      same panels as before, from **Project ▸ Monitor**. They watch the whole application rather
      than a petrophysics run, which is why they sit with History.
- [ ] **Undo still reads what it will undo.** Make an undoable edit (add a top, edit a curve
      value, shift core). On **Project ▸ Edit**, **Undo** enables and its tooltip names the action
      — e.g. "Undo add top UAT_TOP (Ctrl+Z)"; after clicking, **Redo** enables with the matching
      label. **Ctrl+Z / Ctrl+Y are unchanged and are still the fast path** — the buttons exist to
      make the action readable, not to replace the shortcut.
- [ ] **The unsaved warning is still visible from wherever you are.** This one needs a deliberate
      look: Save Session… now lives *inside* the Project tab, so its red dot alone would only be
      visible to someone who already went looking for it. Sit on the **Petrophysics** tab, edit a
      log view (drag a track wider) — an **amber dot appears on the Project TAB itself** without
      you switching to it (hover: "Unsaved changes — Project ▸ Session ▸ Save Session…"). The tab
      must NOT change width when the dot appears. Save a session; both dots clear.
- [ ] **The ribbon overflow arrows work now.** These have been broken since they were written and
      nothing was ever wide enough to reveal it — Project is the first tab that is. If your window
      is narrower than ~1470px you will see a **›** box at the right edge of the Project tab:
      click it and the ribbon really scrolls (Help comes into view, a **‹** appears, the **›**
      hides); click **‹** to come back. It jumps rather than glides — that is deliberate: smooth
      scrolling is silently a no-op on this element, so an unanimated scroll that works beats a
      pretty one that does not. **Tell me if the overflow bothers you** — on a 1366 laptop the
      Project tab will always need one arrow-click to reach Help, and I can win back about 100px
      by merging Language into Appearance and folding Help into Monitor.
- [ ] **Bahasa / Basa Sunda / Basa Jawa cover the new labels.** Switch language on the Project
      tab: Undo/Redo/History/Processing/Performance and the Session, Edit and Monitor captions all
      translate, and switching back to English restores the exact original wording.
- [ ] **`docs/manual_test_plan.md` was updated with this** — every step that said "QAT" or
      "quick-access bar" now names the real ribbon path (T-SHELL-01/-05/-07/-10/-12/-13/-14 and
      ~20 more). Since you are working through that plan, it should no longer send you looking
      for buttons that moved.

## Round 99 — Depth units, increment 2: the Pc fix and the m/ft view toggle (2026-07-29)

**1. The saturation-height error is fixed.** `pc = 0.433 psi/ft/SG · Δρ · h` is per FOOT of
column, but `satheight.rs` and `shf_fit.rs` multiplied the height by 3.28084 unconditionally,
assuming it arrived in metres. On your foot-declared Rokan projects that scaled an
already-foot height and returned a Pc **3.28× too high**.

The test that pins it takes one physical well described twice — 100 m above the FWL in a
metre project, the identical 328.084 ft in a foot project — and requires the same Sw. Against
the old formula it fails with **Sw 0.2685 vs 0.1670**: a 38% error in water saturation that
computed, plotted and would have shipped. It now passes.

`ModuleContext` carries a typed `depth_unit` rather than a magic options key, deliberately: a
missing string key would silently mean metres, which is the failure mode itself. `FT_PER_M` is
deleted rather than left unused — it *was* the assumption.

**2. The m/ft view toggle you asked for.** A small **m / ft** button in each Log View's own
toolbar, beside the zoom controls. It changes what you READ and never touches stored data —
that separation is the whole point, and the button turns accent-coloured whenever the numbers
on screen are converted rather than stored, so a converted view can't be mistaken for the
real ones. The choice persists per machine and defaults to the project's own unit, so doing
nothing shows depths exactly as your files delivered them.

**3. Print scales no longer lie on foot projects.** `PX_PER_UNIT_1_1` derived px-per-depth-unit
from 96 px/in ÷ 0.0254 m/in — metres, always — so every named 1:N scale on a foot project was
off by 3.28×. It now reads the project unit: 3779.53 px/m or exactly 1152 px/ft (96 px/in ×
12). Verified that **1:200 in a 400 px pane shows 21.17 m in a metric project and 69.44 ft in
a foot one — the same physical section.** Note the scale follows the STORED unit, not the
display toggle: "1:200" is a ratio of rock to paper, so it can't depend on which unit you
happen to be reading.

**4. Re-declaring a project's unit is refused once it holds wells** — their depths are already
stored in the old unit, so a re-declaration would silently reinterpret every one of them
(a 2,438 m well would start reading as 2,438 ft). The error says so and points at the display
toggle instead. Converting stored data would be a real migration, not a settings change.

**Verified:** `cargo test` 384 passed / 0 failed; `npx tsc --noEmit` and `cargo check` clean;
conversion, print-scale and toggle behaviour driven live in the browser (8000/8050/8100 ft →
2438/2454/2469 m on the depth axis, stored unit unchanged, button state and tooltip correct
in both directions).

- [ ] **Try:** open a foot project and check the depth axis reads feet, then click **ft → m** in the Log View toolbar. Depths convert, the button turns accent-coloured, and the status line says the data is unchanged.
- [ ] **Try:** with the display in metres, check the **1:N** dropdown still frames the same physical section it did in feet — the scale must not move when you change what you're reading.
- [ ] **Try:** run **sw_height** (Leverett) on a foot project against a well you know. This is the number that was 3.28× wrong; if the Sw still looks off, tell me before trusting it.
- [ ] **Try:** the depth readout under the cursor now carries its unit ("Depth: 8000.0 ft").
- [ ] **Still metres-only, increment 3:** tops/zones panels, composite scale bar, report pages, dashboard depth columns and depth-coloured plot axes still print raw stored depths without conversion. They are correct on a project whose display unit equals its stored unit — which is the default — but they do not yet follow the toggle.

## Round 98 — Depth units, increment 1: the project declares one, imports convert to it (2026-07-29)

Your instinct was right, and it lands on an **already-verified audit finding** (engineering
review **F2e**, "fix-now", high confidence) that nobody had actioned. The LAS index unit was
being parsed at `parsers.rs` and **thrown away** under `#[allow(dead_code)]`, and `curves.rs`
FAMILIES has no DEPTH entry — so `convert_to_canonical` never touched the index. A foot-indexed
Rokan/Caltex LAS put its raw foot numbers in the same column as a metric Mahakam well, and the
import was reported as clean. A top at 8,000 (ft) and one at 2,438 (m) in the *same formation*
sat 5,500 units apart, and correlation, contact planes and the tops slide window compared them
as if that were real.

Per your two decisions: **the project declares its depth unit and imports must match**, and
**depth first, curve units later**.

**What ships now (the storage layer):**

- `src-tauri/src/units.rs` — one place that knows metres from feet. Exact international foot
  (0.3048; the US survey foot differs by 2 ppm ≈ 5 mm over a 2,500 m well, so it is not
  modelled). Unrecognized unit strings return `None` rather than a guess, because guessing is
  the exact failure this exists to stop.
- **Project setting**: stored as a `documents` row, so no schema migration. A **fresh project
  adopts the unit of its first import** — the common case needs no decision from you at all.
- **Import reconciliation**: a file matching the project stores as-is and says nothing; a file
  in the *other* unit is converted and the import is flagged; a file declaring no unit is
  assumed and flagged. Every case except a clean match produces a note in the import warning
  you already see.
- Both stores convert **identically** — the generic-store loader re-reads the same file, so it
  had to apply the same conversion or the two would hold the same curves 3.28× apart.
- `wells.depth_unit` records what the stored numbers mean, next to the data itself.
- Both `#[allow(dead_code)]` attributes are **gone**, so the compiler can never again hide the
  fact that nothing reads the index unit.

**Verified:** `cargo test` 383 passed / 0 failed, including 5 new unit tests (unit-string
spellings that occur in real field LAS, 8000 ft = 2438.4 m exactly and back, NaN preserved
through conversion, and every project×file unit combination).

**One thing this does NOT yet fix, stated plainly.** A project declared in **feet** still has
two places that assume metres:
`satheight.rs:181` and `shf_fit.rs:897/1069/1284` compute `pc = 0.433·Δρ·(h · 3.28084)`, i.e.
they assume the height above free water arrived in metres — so **Pc is 3.28× off on a
foot-declared project**; and `LogCanvasRenderer.PX_PER_UNIT_1_1` derives the true 1:N print
scale from 96 px/in ÷ 0.0254 m/in, so every named scale is mislabelled by the same factor.
A **metric** project is correct today, and mixed-unit imports are now correct because they
convert. Both sites are increment 2, together with the view toggle.

- [ ] **Try:** import a metric LAS into a fresh project, then a foot-indexed one. The second should import with a note that the depth index was converted, and its tops should line up with the first well's in correlation rather than sitting thousands of units away.
- [ ] **Try:** import a LAS whose `~C` block declares no index unit — expect the note "this file declares no index unit — depths assumed to be m".
**Answered (2026-07-29): feet.** Rokan/Central-Sumatra projects will be declared in FEET, keeping
the depths you know. That makes the increment-2 Pc fix **live rather than theoretical** — a
foot-declared project returns a saturation-height Pc 3.28× too high until it lands. You have
deferred it to your manual-test pass of the saturation-height section, which is a reasonable
call because it surfaces in testing rather than in a deliverable. The one rule that follows:
**do not trust or ship an SHF/`sw_height` result from a foot-declared project until increment 2
is in.** Metric projects are unaffected.

## Round 97 — SHELL field-test fixes: Pin OFF, plot right-click, repeat reload key (2026-07-29)

From your run through **Section SHELL** of `docs/manual_test_plan.md` — 16 of 18 passed;
T-SHELL-16 and T-SHELL-17 failed. Three separate causes, all fixed.

**1. "Pin off, never follow well even for active panel"** — the real bug of the three, and a
good catch. Pin OFF is meant to mean *only the active panel follows*, and it asked
dockview "is this pane active?" **at the moment the selection changed**. But a well is
selected by **clicking it in the Wells tree**, and that click makes the *tree* the active
pane — so at that instant no viewer was active and **nothing followed at all**. The pin
effectively became "freeze everything".

The gate now reads a **working pane** (`src/ui/activeViewer.ts`): the last *viewer* you
clicked into. Browsing panes (Wells, Tops, Inspector) never claim the role, so clicking a
well can't steal it. If no viewer has ever been activated the first one to ask claims it,
so "pin off" can never again degrade to "nobody follows". Applies to log views, plots and
the well-bound tool panes alike.

**2. "right click in xplot showed properties instead of option like in log view"** — the
plot canvases swallowed right-click to open Properties directly, which cost them the pane
menu every other panel has (Split right/down, Float, Maximize, image export, Close).
Right-click on a plot now opens the **normal pane menu with `Properties…` as its first
entry**, so both are one click away. Double-click still opens Properties on histogram and
crossplot; Pickett keeps its ⚙ toolbar button (its double-click is reserved for picks).

**3. "ctrl+R does nothing"** — this one was half a documentation defect. The step said
"Press F5, then Ctrl+R", so by the time Ctrl+R was pressed the F5 dialog was **already
open** — and the guard returned silently rather than opening a second one. Correct
behaviour, invisible feedback. A repeat reload key now **pulses the open dialog** instead.
Two related hardenings while in there: the key is matched on physical `code` as well as
`key` (a non-US layout would have missed it), and **Escape** now closes the confirm even
after focus has left it (it was bound to the dialog, so one stray Tab left Escape dead).
The step-4 wording in the test plan was ambiguous and has been rewritten.

**Verified:** `npx tsc --noEmit` clean, `npm run build` clean, and the reload guard driven
through five live scenarios in the browser (Ctrl+R alone → dialog; foreign-layout `KeyR` →
dialog; F5-then-Ctrl+R → one dialog, pulsed; Escape with focus outside → closes; Cancel →
closes). The working-pane tracker's semantics were unit-exercised live. What I could **not**
drive from a browser is a real dockview activation with real wells — that is exactly what
the re-test below covers.

- [ ] **Try:** two Log Views, pin OFF. Click into Log View 1, select well C in the tree → **only Log View 1** moves. Click into Log View 2, select well A → **only Log View 2** moves. This is the failure you reported; it should now be impossible for nothing to move.
- [ ] **Try:** with pin OFF, open a Crossplot and a Log View side by side. Click the crossplot, pick a well — the crossplot follows and the log view holds. Plots obey the same working-pane rule now.
- [ ] **Try:** right-click a Crossplot, a Histogram and a Pickett canvas → pane menu with **Properties…** on top, then export items, then Split right / Split down / Float / Close. Compare against a right-click in the Log View.
- [ ] **Try:** F5 → Escape. Ctrl+R → Cancel. F5 then Ctrl+R while the dialog is up → one dialog, pulsing. **This is the one to check first — if Ctrl+R on its own still does nothing, the cause is not what I diagnosed and I need to know.**
- [ ] **Try:** the two re-tests above are T-SHELL-16 and T-SHELL-17 — your original Fail marks are left in place as the record; re-run those two rows in the xlsx.

## Round 96 — Non-colour design tokens, and a client-skin colour bug found on the way (2026-07-29)

**The important part of this round is not the polish — it is a pre-existing bug the polish
exposed.** On any machine whose **OS is set to dark** (yours is), the five *light* client
skins — Pertamina, Halliburton, Schlumberger, LAPI-ITB, white — kept their white panels but
silently picked up the **dark** `--qc-*` status colours. Measured contrast of the Results-QC
scorecard against the white panel:

| Token | Was | Now | WCAG AA (4.5:1) |
|---|---|---|---|
| `--qc-ok` | 2.24:1 | **5.13:1** | fail → **pass** |
| `--qc-alert` | 3.49:1 | **5.62:1** | fail → **pass** |
| `--qc-warn` | 2.19:1 | 3.78:1 | fail → still fail |

Cause: `@media (prefers-color-scheme: dark)` was scoped to `:root:not([data-theme="light"])`,
but `theme.ts` **deletes** the attribute for "system" and **sets** it for every other choice —
so the block also caught the explicitly-chosen light brand skins. Now `:root:not([data-theme])`
— "no theme chosen at all" — so an explicit choice ignores the OS preference entirely. The
comment in `:root` claiming the skins "inherit these unchanged" is finally true.

Note `--qc-warn` at 3.78:1 still misses AA. That is the light theme's own designed amber
(`#c07000`), not a regression, and darkening a QC semantic colour is your call — flag it if
you want it changed.

The polish itself: colour was the only axis this stylesheet ever tokenised, so radius, type
size, motion and elevation had been decided per rule by hand — **12 distinct corner radii and
11 font sizes**, four of them half-pixel (11.5/10.5/12.5/9.5px, 45 declarations) which land off
the pixel grid and render soft. Added `--r-*`, `--s-*`, `--fs-*`, `--dur-*`/`--ease`, `--el-*`
and `--focus-ring`, then swept **104 radius** and **201 font-size** literals onto them.
Chips and badges became true pills; dockview's own `--dv-border-radius` / tab font-size /
floating shadow now read from the same scale.

Motion and focus are **one block** near the top of the file, not a line added to forty rules —
reviewable and revertable in one place. Two properties keep it safe: `:where()` has zero
specificity so every existing rule still wins (`.btn` verifiably kept its own 0.12s), and only
**paint** properties are transitioned — never transform/width/position — so dockview drags,
sash resizing and canvas panning stay instant. Buttons got a focus ring (they had none; form
fields already did, and were left alone). Dialogs fade in on opacity only — no transform,
because the modal is drag-positioned and a transform animation would fight an immediate grab.

Verified: ribbon geometry **byte-identical** before and after (112px ribbon / 80px panel /
24px QAT, A/B'd against the stashed original at a fixed viewport — an earlier 122px reading was
a viewport artifact, not a reflow); **663 elements swept for unresolved `var()`, zero found**
(an undefined token would silently collapse radius to 0); all 7 themes checked; gate green
378/0/7.

- [ ] **Try:** switch to a **client skin** (Project ▸ Appearance ▸ Pertamina) and open a
  Results-QC scorecard. The pass/warn/fail colours should now be legible dark green/amber/red
  on white, not the pale dark-theme versions. This is the one item with a real deliverable
  consequence.
- [ ] **Try:** hover the ribbon tabs and buttons — they should ease rather than snap. If
  anything feels laggy against real field data, the whole motion layer is one block in
  `styles.css` and can be cut without touching anything else.
- [ ] **Try:** drag a dockview panel between windows and pan a log view. Both must still feel
  instant — geometry was deliberately excluded from the transitions.
- [ ] **Try:** confirm the tighter type reads as *cleaner* and not *cramped* at your normal
  window size, on a dense panel (Monte Carlo params, the multimin endpoint matrix).

## Round 95 — SSC gas conditioning changed the numbers; the stale test that hid it is fixed (2026-07-29)

**This one needs your eyes on real data — SSC output values moved.** Your `d1f0c1e` commit
re-aligned `ssc.rs` to the Loglan reference, and one change is numerical, not cosmetic: the
gas/HC conditioning now pulls a point onto the sand base line at the **RMS midpoint**
(`sqrt((φD²+φN²)/2)`, matching `sspw.lls`'s gas branch) instead of the old 1.6-weighted form,
which overshot the midpoint and inverted the density-neutron crossover. Any gas-affected
sample will therefore report a different PHIT/PHIE than before. Per `RELEASE.md`, that is a
"numbers that changed" event.

That commit also left the gate red: `ssc_swirr_floor_pads_capillary_water` asserted
`SWIRR_T >= SWIRR_MIN`, which contradicted **its own name** and both references. The floor
(`ssc.rs` `if ... bw / phit < swirr_min`) pads **CWSH** — capillary water — raising BW;
`SWIRR_T` is deliberately the *pre-conditioning* ratio (`.lls` 213-216, and
`docs/method_ssc_sspw.md` §8 computes SWIRR first, then lists the conditioning). So the code
matched the spec and the test was the stale artifact. I did not touch any physics.

The test now pins **both** halves of that contract — the floor must raise CWSH and lift
BW/PHIT to SWIRR_MIN, *and* SWIRR_T must stay the pre-conditioning value — plus a guard that
the fixture actually starts below the floor, so it can't pass vacuously. Gate: green with
nothing stashed.

- [ ] **Try:** re-run SSC on a well with a known gas effect and compare PHIT/PHIE against
  your previous run (or the reference-suite LAS export). The non-gas samples should be
  unchanged; gas-affected ones will differ, and the new values are the ones that match the
  Loglan. If they don't match the reference export, tell me — that is a real finding.

## Round 94 — R-C: closing the app no longer risks losing the writes since the last checkpoint (2026-07-29)

Found by the packaged-build verification, not by code review — and it is the biggest catch of
the session. Tauri exits through `std::process::exit`, which skips Rust destructors, so the
DuckDB connection **never closed cleanly on any exit**: every close — including a plain window
✕ — abandoned a live WAL. Reproduced twice against the packaged app: import a 20-row LAS,
close with ✕, relaunch → the WAL fails replay, `init_db_resilient` moves it aside as
`.corrupt-backup-<ts>`, and the import is **silently gone** (`Wells (0)`). Writes below
DuckDB's auto-checkpoint threshold live only in the WAL, so the writes at risk are exactly the
small, recent ones: an import, a parameter edit, a tops pick made just before closing. This
also explains the WAL-corruption plague CLAUDE.md attributes to `tauri dev` force-kills — every
close abandoned a WAL; the force-kills were just the ones caught badly enough to notice.

Fix: `lib.rs` now runs the app with a `RunEvent::Exit` handler that locks the connection and
executes `CHECKPOINT` — every graceful exit flushes the WAL into the project file while the
process still can. Force-kills stay covered by `init_db_resilient` exactly as before.

Verified end-to-end on the packaged exe (isolated scratch project, `SANDIBUMI_CONFIG_DIR`):
same import-close-relaunch sequence → after close there is **no `.wal` at all** beside the
project, relaunch lists the imported well, and no new corrupt-backup appears. Full green gate:
`GATE GREEN in 68s` (378/0/7, SSC WIP stash-roundtripped).

- [ ] **Try (= T-SHIP-07 in `docs/manual_test_plan.md`):** in a COPY of a project, import one
  LAS, close the app with ✕ immediately, look beside the `.duckdb`: no `.wal` should remain.
  Reopen — the imported well must still be there and no new `.corrupt-backup-*` file appears.

## Round 93 — R-B: a destructive migration now backs up the project file first (2026-07-29)

Requirement R-B from `docs/RELEASE.md` §3.2, the sibling of Round 92's R-A and the other
1.0-gate item. The finding: the PK-drop migration (the one that made 100-well chains 2.4×
faster) **rebuilds the whole `computed_curves` table in place** — `DROP TABLE` mid-sequence —
with no recoverable copy. On a field-scale file, a crash mid-rebuild loses computed results
with nothing to fall back to.

At this increment, when that migration was actually going to run (and only then — additive
migrations like the R-A stamp and the generic-store backfill are exempt, so backups stay
meaningful), the project was first copied beside itself as `<name>.pre-1-backup.duckdb` and
the launch log said so. Gate 2 SB-DBM-042 later corrected that target-labelled name to
`<name>.pre-<source-format>-backup.duckdb`. Two honesty properties remain: a **failed backup aborts the migration** (the un-migrated file
still opens fine — the PK only slows writes — so refusing costs nothing, while proceeding
would break the exact promise), and an **existing backup is never overwritten** (collision →
timestamped name, the WAL-recovery convention). One Windows reality the test caught: DuckDB
holds its file with exclusive sharing, so a filesystem copy of an open project is impossible —
the copy is made by the engine itself (`ATTACH` + `COPY FROM DATABASE`), which also preserves
the schema *with* the PK, so the backup is provably the pre-migration file.

Verified: 2 new `db.rs` tests against real temp files — the destructive path writes the backup
first (openable, PK intact, both rows present), a no-op open writes nothing, a fresh project
never accumulates backups, and a name collision takes a new name. Full green gate:
`GATE GREEN in 39s`, **378 passed / 0 failed / 7 ignored** (SSC WIP stash-roundtripped as
before).

- [ ] **Try:** open your real project — since increment 5 already migrated it, the pass
  condition is **absence**: no new `*-backup.duckdb` file beside it, launch not slower.
  To see it fire, open any pre-2026-07-19 project copy that still has the old PK: a
  `<name>.pre-0-backup.duckdb` appears beside an unstamped legacy source (otherwise
  `pre-<its-source-format>`) and the console log announces it before
  the rebuild. (Full list of session-wide manual checks: `docs/manual_check_plan.md`.)

## Round 92 — R-A: the project file now carries a format stamp, and an older build refuses a newer file by name (2026-07-29)

Requirement R-A from `docs/RELEASE.md` §3.1 (on the 1.0 gate; the doc arrives with PR #2). The
finding behind it: the project `.duckdb` carried **no format version anywhere** — every table is
`CREATE TABLE IF NOT EXISTS`, read by name — so an older SandiBumi opening a file written by a
newer one would open it, find the tables it knows, silently ignore the rest, and present a partial
project as the whole thing. Months of interpretation, shown with pieces missing, no warning. That
is the cardinal rule (a degraded result presented as clean) with a whole project as the blast
radius, and it was the *default* behaviour.

Now: a `project_meta` table (`format_version`, `written_by`) is stamped into every project on
open. `db::FORMAT_VERSION` starts at 1; the check runs **before** `create_schema` on purpose,
because `CREATE TABLE IF NOT EXISTS` is itself a mutation and a newer file must be refused
*untouched*. Three cases: no stamp (fresh file or legacy project) → stamp it, additive; stamp ≤
current → open normally, re-stamp if older; stamp > current → **refuse**, naming the file's
format, the app that wrote it, and what to do ("this project was written by SandiBumi X (file
format N); this build reads format 1 and lower - upgrade SandiBumi to open it (the file was left
unmodified)"). A missing or unparsable version row counts as legacy, never as newer — refusal
requires positive evidence. The refusal message contains no "WAL", so `init_db_resilient` can
never mistake it for corruption and move a healthy newer file's WAL aside.

Verified: 3 new tests in `db.rs` — fresh project stamped with format 1 + `written_by SandiBumi
0.1.0`; a legacy pre-stamp project (full schema, no meta) is stamped on open; a future-format
file (stamp 999, deliberately without the current schema) is refused with all three message parts
AND left byte-honest — `wells` still absent after (proving `create_schema` never ran), stamp
still 999. Full green gate: `GATE GREEN in 47s`, **376 passed / 0 failed / 7 ignored** (SSC WIP
stashed for the run and restored after, as in Round 91).

- [ ] **Try:** open any existing project normally — everything must work exactly as before (the
  stamp is invisible). Then in the **SQL Query** panel run `SELECT * FROM project_meta` — expect
  two rows: `format_version` = 1 and `written_by` = SandiBumi 0.1.0. The refusal path needs a
  future build to demonstrate for real, which is the point — it exists so that *next year's*
  files are safe in *this year's* app; the test suite stands in for the future build today.

## Round 91 — the green gate: one command that proves the tree is healthy (2026-07-29)

Q3 of the 1.0 quality bar (`docs/V1_SCOPE.md` §5, defined in `docs/RELEASE.md` §5 step 0) — until
now the three verification gates were run by hand, separately, from memory. **`tools\check.ps1`**
runs them in order and exits non-zero at the FIRST failure: (1) `npm run build` (tsc runs inside
it, so no duplicate type-check pass), then (2) full `cargo test` in src-tauri **through vcvars
pinned to 14.29** when that toolset exists (this machine's 14.50 is broken), plain `cargo test`
otherwise — so the same script works on a healthy machine. `-SkipRust`/`-SkipFrontend` exist for
the inner loop, but "green" means the full gate. It also prepends the known node/cargo homes to
PATH, so it works from a fresh shell that missed the installer PATH updates.

Verified with real runs, not by reading the script: **(a) green** — full gate on the committed
tree: frontend 7 s, backend 37 s (373 passed / 0 failed / 7 ignored), `GATE GREEN in 44s`, exit 0;
**(b) red** — its very first full run caught a REAL failure and propagated it (`GATE FAILED at
backend (cargo test) (exit 101)`, script exit 1); **(c) toolchain failure** — a bogus
`-VcVarsVer 99.99` fails fast at vcvars before cargo ever runs, exit 1.

**Worth knowing about (b), because it's a live finding in your working tree:** the failure it
caught is the in-progress `ssc.rs` edit (another session's work, dated 2026-07-29, uncommitted) —
it moves `SWIRR_T` to the pre-conditioning value per the Loglan, and the old test
`ssc_swirr_floor_pads_capillary_water`, which pins the post-conditioning floor semantics, now
fails against it. Proven by stash round-trip: HEAD's `ssc.rs` passes all 6 ssc tests; the WIP
version fails that one. Nothing was changed — the SSC work is mid-edit and its test reconciliation
is that session's to finish — but until it is, **a full-tree gate run will be red**, and that red
is true.

- [ ] **Try:** from PowerShell in the repo root run
  `powershell -ExecutionPolicy Bypass -File tools\check.ps1` — expect the two stage banners and
  `GATE GREEN in ~45s` (first run after a Rust change recompiles, so longer). Then break something
  trivial on purpose (e.g. add `let x: number = "no";` to any .ts file), run it again — it must
  stop at stage 1 with `GATE FAILED` and a red message, and `$LASTEXITCODE` must be 1. Revert the
  break. (If you run it before the SSC session finishes its test reconciliation, expect the honest
  red described above.)

## Round 90 — R30: three dialogs silently computed on GR when the curve they wanted was missing (2026-07-29)

From the F1 sweep (finding #4), verified still open against live code before touching anything.
Three dialogs — **SMLP/Lorenz**, **SHF fitting**, and **Facies tie-in** — had byte-identical private
copies of a curve-dropdown builder that walked the catalog and pre-selected the first "preferred"
name it found (`PERM`, `PHIE`, `TVDSS`, …). The trap was the miss path: **when none of the
preferred names existed in the well, it selected nothing — and an unset `<select>` falls back to
option 0 of the catalog, which is deterministically GR** (the catalog seeds `GR, RES_DEEP, NPHI,
RHOB, DT, SP` ahead of everything else). GR in gAPI (20–150) is numerically indistinguishable from
permeability in mD, so the Lorenz backend — which *does* guard honestly ("permeability curve 'PERM'
has no data in this well") — never got the chance to refuse: the dialog handed it a curve that
**did** have data, and it computed a fully plausible Lorenz coefficient and flow-unit table from
gamma ray. A clean cardinal-rule violation: a wrong result indistinguishable from a right one.

Fixed by deleting all three private copies and routing the **9 call sites** through one shared
helper (`plotCommon.ts preferredCurveSelect`): when no preferred curve exists, the first preferred
name (e.g. `PERM`) stays **selected and visible** in the dropdown — `curveSelect` prepends it as a
real option — so the run reaches the backend's own guard and fails loudly with the named curve,
instead of silently substituting GR. Bonus from the shared path: the private copies never set
`.form-control`, so all 9 dropdowns were also unstyled (the R13 defect class); they now match the
rest of the app. Two legs of the original report were corrected during verification and are noted
for honesty: the faciesTie leg was already *functionally* dead (the backend errors when predicted
== reference, which is what the double-GR fallback produced), and the headline TVDSS example was
weak (shf_fit drops non-positive heights) — the real damage was Lorenz-PERM and SHF-PHIE.

Verified: `tsc` + `vite build` clean; browser functional test against the real modules
(vite-only, server stopped afterward): catalog-without-PERM now yields a dropdown showing `PERM`
(7 options, styled), not GR; a catalog containing the preferred curve selects it with no duplicate
option; the full Lorenz dialog builds with φ=`PHIE`, k=`PERM` on an empty catalog.

- [ ] **Try:** open **Petrophysics → Rock Typing → SMLP / Lorenz…** on a well that has **no**
  permeability curve computed or imported. The Permeability (k) dropdown must show **PERM** (not
  GR). Click **Run** — you must get *"permeability curve 'PERM' has no data in this well"*, not a
  plot. Then compute/import a PERM and reopen — it should be found and selected as before. Same
  shape in **SHF fitting** (φ shows PHIE on a bare well) and **Facies tie-in**. All curve dropdowns
  in these three dialogs now also render with the app's styled look instead of the native browser
  select.
## Round 89 — PRD pass: webview CSP turned on, unused OS capability removed (2026-07-29)

Not an R-chain bug fix — this came out of writing `docs/PRD.md`, where §7.5 asks the question a
client's IT department asks: *what leaves the machine, and what can this app do that it doesn't
need to?* Two answers were worse than they should have been.

**1. The webview had no Content Security Policy at all** (`"csp": null`). That matters here
specifically because of R9 in this file — a hostile well name inside an imported LAS reaching the
DOM. That hole is closed by escaping, but a null CSP meant there was no second line of defence
behind it, and untrusted text arrives with *every* imported file. There is now a real policy in
`tauri.conf.json`. Two relaxations are deliberate: `script-src` keeps `'unsafe-eval'` because Vega
compiles chart expressions through the `Function` constructor and would silently stop rendering
without it, and `style-src` keeps `'unsafe-inline'` because CodeMirror injects a `<style>` element
and the print path writes one into its hidden iframe. Neither re-opens R9 — inline handlers and
inline `<script>` need `'unsafe-inline'` in **script-src**, which is absent.

**2. `tauri-plugin-opener` was registered and permitted but never used.** It grants the app the
ability to hand a URL or path to the OS. There were **zero call sites** anywhere in `src/`, so
nothing was ever passed to it — but a granted capability the product doesn't use is exactly what an
enterprise security review flags. Removed at all four layers: the Rust plugin registration, the
crate dependency, the `opener:default` capability entry, and the npm package.

Also in this pass: `README.md` no longer describes the product as "the reference suite-class"
(competitor-referential copy in the customer-facing document), `CLAUDE.md`'s collaboration protocol
now states this file's **actual** mark convention (`[x]` = accepted — it had preserved the
superseded `[o]` legend, under which your 72 accepted items read as 72 broken ones), and
`docs/IP_PROVENANCE.md` records where every piece of reference data in the repo came from.

Verified: `tsc` + `vite build` clean, `cargo check` clean after the plugin removal.

- [ ] **Try:** the CSP **cannot be tested with `npm run tauri dev`** — with a dev URL the webview
  loads Vite directly and Tauri never delivers the policy. It only applies to a packaged build. So:
  run `npm run tauri build`, install/launch the built app, and exercise the three paths the
  relaxations exist for — (a) open the **Vega** panel and render a chart, (b) open the **Inspector**
  (Equation Editor) and confirm the editor appears and highlights, (c) open any crossplot/histogram
  and use **Print** from its toolbar. All three must work exactly as before. If any of them is
  blank or dead, open DevTools ▸ Console and look for a `Content Security Policy` violation — the
  message names the directive that needs widening. Everything else in the app should be unaffected.
  Separately, confirm nothing anywhere tried to open an external link (nothing should — there were
  no call sites).

## Round 88 — R29: the Equation Editor leaked a whole CodeMirror editor every time you closed it (2026-07-25)

Sixth F5 fix, and a pure hygiene one — nothing renders wrong, no result goes stale, no data is at
risk. `InspectorPanel` had **no `dispose()` at all**. It correctly recycles its CodeMirror `EditorView`
on internal re-renders (pick another equation, switch language), but the **last** view of each panel
lifetime was simply abandoned. That is not just a detached DOM node: an `EditorView` registers four
listeners rooted at `window`/`document` — `resize`, `scroll`, `beforeprint` and `selectionchange`
(verified in `@codemirror/view/dist/index.js:7480-7492`) — and the **only** code path that removes them
is `EditorView.destroy()` (7513→7521). `window` and `document` are GC roots, so each abandoned view
kept itself, its history/autocomplete state, the python parse tree and the detached editor DOM
reachable **for the life of the process** — and every caret move anywhere in the app still dispatched
into every one of them.

It compounds faster than "how often do I close that panel?" suggests: the Inspector is closable, and
`dock.clear()` runs on **every session switch and every workspace reset**, so each of those strands
another editor too. `vegaPanel.ts` already destroyed its own `EditorView`, and the DB Inspector and
History panels are already wired to `dispose()` at `workspace.ts:419/428` — so this was an omission,
not a decision.

Fixed by giving `InspectorPanel` a `dispose()` (destroy + null + a `disposed` flag) and calling it from
the workspace cleanup closure alongside the two existing unsubscribes. The `disposed` flag matters on
its own: the editor now mounts **asynchronously** behind a dynamic `import("codemirror")`, and the
existing `host.isConnected` guard conflates "inactive tab" with "closed panel" (dockview detaches
inactive tabs), so it is not a dispose signal — without the flag, a panel closed during that import
window mounts a brand-new editor into a dead panel, with no remaining reference to destroy it.

Verified: `tsc` + `vite build` clean. A leak is invisible to `tsc` — not destroying an object is
perfectly well-typed — so proof is two-part. (1) A codebase invariant: `src` has exactly **two**
`EditorView` construction sites, `vegaPanel.ts:1053` and `inspectorPanel.ts:250`, and both are now
destroyed on dispose. (2) `inspector_leak_harness.mjs` models the lifecycle against a listener registry
using the real listener set: 15 open/close cycles strand **60** listeners on the old code and **0** on
the new, stranded views are confirmed still-undestroyed and still holding their payload, a panel closed
mid-import mounts an unreachable editor on the old code and refuses to mount on the new, and internal
re-renders still recycle to exactly one live view. **8/8 pass.** Frontend-only.

- [ ] **Try:** hard to see directly — it is memory, not behaviour, so mainly confirm **nothing broke**.
  Open the **Inspector** (Equation Editor), pick an existing equation, switch its language Rhai↔Python,
  edit the script and **Save** — all must behave exactly as before. Then close and reopen the Inspector
  ~10 times and confirm the editor still appears and still loads the selected equation's text each
  time. If you want to see the fix working, open DevTools ▸ Memory, take a heap snapshot before and
  after 10 close/reopen cycles: the `EditorView` count should stay flat instead of climbing by one per
  cycle.

## Round 87 — R28: the Tops pane could window every plot to another well's depths (2026-07-25)

Fifth F5 fix, and the second wrong-well one. `TopsPanel.refresh()` assigned `this.wellId = wellId`
**synchronously** but `this.tops` only **after** `await listTops(wellId)` — and nothing cleared the list
in between. So for the entire width of the DuckDB query the pane showed **well A's rows, still
clickable, under an id that already said well B**. Click one and `toggle()` paired the two live fields
and published `{wellId: B, topName: <A's top>, depthMin: <A's depth>}`. Both consumers accept an
interval on the **id match alone** — `logViewPanel.ts:341` scrolled well B's log view to well A's depth,
and `plotCommon.ts:322` re-windowed every crossplot / histogram / Pickett of well B to a foreign depth
range. That is a **parameter pick (Rw, m/n, cutoffs) read off the wrong zone**, and the wrong numbers
travel into a deliverable long after the session ends. It also defeated the invariant the workspace
explicitly documents at `workspace.ts:917-921` — "followers never see a foreign interval".

Worth stating plainly: this is **not** a lost race. `list_tops` is a synchronous `#[tauri::command]`
(`lib.rs:694`), and Tauri runs non-async commands inline in the IPC handler, so responses already
resolve FIFO — the generation token the original report proposed would have fixed nothing. The defect
was deterministic and fired on the *load window of every well switch*.

Fixed by making the id and the rows **one unit**: a `TopsView { wellId, tops }` snapshot, assigned only
together, **captured into each row's click closure** so a row can only ever emit the interval for the
well it was painted for. On a well change the list is cleared to "Loading tops…" before the await, so
the stale row is not there to be clicked at all; a *same-well* refresh (dataVersion after a run) keeps
its rows, so a recompute does not flicker the pane. A `refreshGen` token is still worth its three lines,
but for the honest reason — it drops a **superseded repaint**, not a stale write. Same snapshot shape as
R26's `GridView`. Also primed the `dataVersion` double-subscribe at `workspace.ts:968`, which was firing
a second identical `list_tops` and a second full DOM rebuild on every pane open.

Verified: `tsc` + `vite build` clean. A wrong-well emit is invisible to `tsc` — every type is correct,
the mismatch is *which* well the id belongs to — so `tops_wrongwell_harness.mjs` models both versions
against a hand-driven `listTops`: the old code emits `{wellId: B, topName: A-Sand 1}` and the harness
confirms a log view on well B **accepts** it, while the new code has nothing clickable during the
window, emits B's own top once B lands, and — even when a row is deliberately held past its refresh —
still emits a self-consistent pair. **9/9 pass.** Frontend-only.

- [ ] **Try:** open the **Tops** pane, a **Log View** and a **Crossplot** on a large project. Click well
  **A**, wait for its tops, then click well **B** and *immediately* click a top row while the pane is
  still mid-load. The pane must show **"Loading tops…"** with nothing clickable — never well A's names
  under well B. Then let B finish, click one of **B's** tops: the log view and plots must window to that
  depth. Also confirm a **recompute** (run any module) refreshes the pane **without** flashing "Loading".

## Round 86 — R27: a Python equation run showed 0% and no failures (2026-07-24)

Fourth F5 fix, and the first backend one of the tier. `run_python_equation` reported per-well progress
on its cancelled / fetch-error / no-data / all-MISSING branches — but **not** on the three that end a
normal run: the successful write, the write failure, and the script error. `finish_item` is the only
thing that increments a job's `done`, and `start_item` has already flipped each well to amber
"Running", so a healthy 20-well Python run rendered **"0%" and "0/20"** with all 20 wells apparently
mid-flight, then flipped to a **"Completed"** card still reading 0/20. Worse for honesty: a plain
Python **syntax or runtime error** — the commonest authoring mistake — left its well amber "Running"
instead of red "Failed", so the Processing panel showed **no failure signal at all** for a script that
never ran. The tell that this was a slip rather than a design choice: the *cancelled* branch did report,
so an **aborted** run displayed more progress than a **successful** one.

Fixed by mirroring the Rhai sibling (`equations.rs`) on all three branches — `finish_item(Ok)` after a
successful write, `finish_item(Failed, e)` on a write error and on a script error. Display/observability
only: `write_equation_output` already ran, so no curve data was ever wrong or lost — but the live
progress and the per-well states were.

Verified: `cargo test` — **373 passed / 0 failed / 7 ignored**, whole crate. New
`python_equation_reports_progress_on_every_terminal_branch` asserts on the `JobView` the panel actually
renders (done-count + item state), not on the return value, and covers the success and script-error
branches end-to-end (python is present here, so they really ran) plus the no-python early return as a
guard on machines without it. I also confirmed it is a **real** guard by reverting just the success
branch and watching it fail with "a successful write must count one unit of progress (was stuck at 0)".
Backend-only.

- [ ] **Try:** save an equation with **language = python** (e.g. `vshp = gr / 100.0`) and Run it over
  several wells. The **Processing** panel must count up to **N/N / 100%** with each well turning green —
  not sit at 0% with amber rows. Then deliberately break the script (e.g. `vshp = undefined_name + 1`)
  and Run again: the wells must go **red/Failed** with the Python error as the message, not stay amber.

## Round 85 — R26: the DB Inspector could write a cell edit to the wrong well (reload race) (2026-07-24)

Third F5 lifecycle-tier fix, and the one with teeth — a **silent wrong-row write into your own
well-log DuckDB**. `dbInspectorPanel.reload()` had no token, and `renderGrid()`/`commitEdit()`
**re-read live state** (`this.tableDef()`, `appState.selectedWell.get()`) at paint/commit time instead
of the scope the shown page was fetched under. Two failure shapes: (a) a lost race — pick Standard
Curves (slow 200-row query), then switch table/well before it lands; the slow page renders under the
now-live def; (b) the sharper one the verifier flagged — switch from well A to B while A's grid is still
on screen (the header flips to "B" synchronously, the grid lags), double-click a GR cell and Enter, and
`commitEdit` re-read `selectedWell` = B → `updateStandardSample(B.well_id, <A's depth>, "gr", v)`.
`db.rs` UPDATEs `WHERE well_id AND depth`, so it's rejected *unless* B has a sample at that depth — and
Mahakam wells share the 0.1524 m grid, so it usually **does**: a real value silently overwritten in the
wrong well, with an undo entry recording the wrong inverse so Ctrl+Z compounds it.

Fixed with the pattern the sibling plots already use (`crossplotPanel`/`pickettPanel` `reloadGen`), plus
the piece a token alone can't cover: bundle the fetched `(def, well, offset, page)` into a `GridView` and
thread it through `renderGrid → beginEdit → commitEdit`, so an edit is **always** bound to the rows on
screen — never a live re-read that a mid-flight reload moved on. A `reloadGen`/`disposed` token drops a
superseded page after its await (and prevents a write to a torn-down panel). One file, no API change, no
backend change, happy path unchanged.

Verified: `tsc && vite build` clean. A race is invisible to `tsc`, so a headless
`dbinspector_race_harness.mjs` models both decision points: with a stale grid on screen the OLD live-
re-read corrupts well B at A's depth while the NEW view-snapshot writes to well A (the row shown), and the
`reloadGen` token drops a slow reload that resolves after a newer one. 5/5. Verified-by-construction
against the two proven token siblings the fix mirrors. Frontend-only.

- [ ] **Try:** open **Database Inspector**, pick **Standard Curves** on a well with a long log. In the
  **Wells & Tops** pane switch to a *different* well and, immediately (before the grid repaints), double-
  click a GR cell and press Enter. The edit must land on the well whose rows you can see — never the newly
  selected one — and the status line's well name must match the grid. Then page/table-switch rapidly a few
  times: no stale rows should ever appear under a mismatched header.

## Round 84 — R25: the Correlation panel leaked a window `pointerup` listener every open/close (2026-07-24)

Second F5 lifecycle-tier fix, and a corroborated one — dimensions F5a and F5b flagged it independently.
`correlationPanel.ts` registered `window.addEventListener("pointerup", () => (dragging = false))` with an
**anonymous** handler and a `dispose()` that released only the ResizeObserver and two subscriptions. A
`window` listener outlives the panel (unlike the canvas-scoped ones, which die with the detached `el`
subtree), so every close stranded one dead handler — and because it closes over `dragging`, which shares a
scope with `strips`, each stranded listener pinned that build's **entire `WellStrip[]`**: per well a
1400-sample decimated curve pair plus a two-`Float64Array` TVDSS map, for every well in the active group.
Correlation panels are `freshId(kind)` (never singletons), so the retained set grew per open/close cycle —
~1.5–7 MB pinned per cycle on Jauhar's 40–200-well groups, monotonic for the process life, surviving Reset
Workspace / Open Session (same dispose path). `LogCanvasRenderer.ts:540-561` even carries a comment warning
about this exact trap; correlation was the lone panel builder that fell into it.

Fixed by the documented house pattern: hoist to a named `const onWindowPointerUp`, register it, and add
`window.removeEventListener("pointerup", onWindowPointerUp)` to `dispose()`. Same edit captures the
`setTimeout(fit, 50)` as `fitTimer` and `clearTimeout`s it in dispose, so a panel closed inside 50 ms can't
run `fit()`→`draw()` against an already-detached canvas. No behaviour change — pure teardown hygiene.

Verified: `tsc && vite build` clean. Proof for a leak is dispose symmetry: a repo-wide grep of every
`window.addEventListener` now shows every **per-panel** listener has a matching `removeEventListener`
(crossplot 2047↔2101, correlation 1049↔1114, map, plotCanvas, vega 1129↔1179, viewerChrome,
LogCanvasRenderer) — the only add-only ones left are the app-shell singletons built once at boot (ribbon,
workspace, autosave, main, interactionGuard), which the F5 review classifies as one-off, not defects. So
correlation was the last panel builder missing its removal, and it no longer is. Verified-by-construction
against the three proven siblings the fix copies. Frontend-only.

- [ ] **Try:** hard to see directly (it's a leak), but sanity-check nothing regressed: open a **Correlation**
  panel on a multi-well group, **drag** a strip up/down to pan (release the mouse *outside* the canvas — panning
  must still stop cleanly), hover to confirm the linked depth still syncs, then close and reopen the panel a few
  times. Everything should behave exactly as before; the fix only frees memory on close.

## Round 83 — R24: the Report pane never opened in the multi-select state (TDZ crash) (2026-07-24)

A flat user-facing bug, not an honesty one: with **no active well group** but a **multi-selection or ★-pins**
present, opening **Report** failed outright — the pane showed "Failed to open the report generator:
ReferenceError: Cannot access 'batchBtn' before initialization". That is exactly the state you are in when
you reach for **batch** report export, which is why it survived: the usual active-group state dodges it.

Root cause is an async-constructor / synchronous-observer collision. `buildWellScope` is `async` and, after
awaiting `listWells`/`listWellGroups`, subscribes to `pinnedWellIds`/`multiSelectedWellIds`. `Observable.subscribe`
fires its listener **synchronously** on subscribe (`state.ts:29`), and when `smartDefault()` lands on "pinned"/
"selection" that first fire runs `emit()` → the caller's `onChange`. But the caller (`reportDialog`) is still
parked on `await buildWellScope(...)`, so the `const batchBtn` its `onChange` reads is still in its temporal dead
zone → `ReferenceError`, which rejects the builder's promise and the whole pane. Same failure mode as the earlier
V3 Vega TDZ, in a different place.

Fixed with the house **primed-flag** pattern (as in `plotCommon.ts:349` / `mapPanel.ts:434`): a `let ready = false`
gates both subscribe callbacks, set `true` only after the scope's own first paint. The synthetic construction-time
fire is suppressed; genuine post-construction pin/select changes still emit. Nothing is lost — every caller does its
own first paint (reportDialog sets the batch label from `getWellIds()`, cutoffDialog awaits `refreshZoneDst()`), and
of the 13 `buildWellScope` callers only those two pass an `onChange` at all. Frontend-only.

Verified: `tsc && vite build` clean. TDZ is a runtime error `tsc` cannot see, so a **headless Node harness**
(`wellscope_tdz_harness.mjs`) models the exact mechanism — a synchronous-fire Observable, an async builder that
subscribes after two awaits, a caller whose `onChange` reads a const declared after its await — and proves it:
the unguarded pattern throws the TDZ ReferenceError, the guarded (`ready`) pattern opens cleanly with the right
label, the construction-time fire is suppressed, and a real post-construction change still emits. 5/5 pass.

- [ ] **Try:** with a project open, leave the group selector on **All wells** (no active group). In the **Wells**
  pane, **Ctrl-click two wells** (or ★-pin one and clear the selection). Ribbon → **Report**. The pane must open
  normally showing a **Batch (N wells)…** button — not "Failed to open". Then pin/select another well while it is
  open: the **Batch (…)** count must update live. Repeat with a group active to confirm nothing regressed there.

## Round 82 — R23: the Field Dashboard Compute posted a redundant "Pay summary" job card (2026-07-24)

The tail of R19. `run_pay_summary`'s silent-run guard was `if req.stats_only && req.skip_version`, but
the Field Dashboard sets `stats_only` **alone** (`skip_version` defaults false) — so that branch matched
**no** caller, and every dashboard **Compute** fell through to `run_simple_job`, posting a
"Pay summary — cutoffs & pay" card in the Processing panel. That card is redundant (the dashboard already
reports "Computing N well(s)…" then the result in its own status line) and mildly misleading — labelled
"cutoffs & pay" for a run that, being `stats_only`, writes nothing (a faint echo of the R19 lie).

Fixed by keying the silence on the real invariant: `if req.stats_only`. A stats-only pay summary persists
nothing (`workflow.rs` gates every FLAG_* write behind `!stats_only`), so it is a pure read and never
needs a job card. The dashboard is the only stats-only caller, so this touches only it; a **persisting**
pay summary — an explicit Cutoffs & Summary run, or a report render (`skip_version`, `stats_only` false) —
still shows a job. The old guard encoded "dashboard" by an incidental two-flag coincidence that the
stats_only refactor had silently broken; the new one ties silence to "persists nothing".

Verified: `cargo test pay_summary` — 4/4 green via the pinned 14.29 toolchain, incl.
`pay_summary_stats_only_persists_nothing` (the invariant this fix relies on); whole-crate compile clean,
no warnings. This is a Tauri command (not directly unit-testable), and the change is grep-proven to affect
only the dashboard (`stats_only: true` has one command-level caller). Backend-only.

- [ ] **Try:** open the **Field Dashboard**, press **Compute** a few times. The **Processing** panel must
  stay quiet — no "Pay summary" card appears — while the dashboard's own status line shows progress and the
  result. Then run **Cutoffs & Summary** (or export a **report**): those must still show a job card as before.

## Round 81 — R22: the legacy Multimin module is retired (your decision) (2026-07-24)

This one is a **decision**, not an F-sweep finding — the follow-on R17 surfaced. The legacy fixed
4-component `multimin` inversion (superseded by SandiMin, hidden from every UI picker since long ago)
was still a **live compute path**: `list_modules` returned it, so any saved workflow chain with a
`multimin` step — or a restored `module:multimin` dockview panel — still ran the old solver, silently,
with endpoint defaults that could drift from SandiMin's library. You chose **graceful retirement** over
a hard delete or a keep-and-consolidate.

Implemented so a retired module fails **loudly and actionably** rather than vanishing or running stale
physics: a new backend registry `modules::retired_module(name)` is the single source of truth;
`run_module` checks it first and returns *"The Multimin module is retired… Re-run this step with
SandiMin (Advance ▸ Mineral Solver)."* before any dispatch. The `multimin` **spec is kept** in the
catalog on purpose — a saved chain step still resolves by name and renders its stored parameters, so
you can see what it was before re-doing it in SandiMin — but the solver body and its R17 physics tests
are removed (unreachable now; R17's reusable `rho_e` Pe↔U relation stays in `multimin2`, where SandiMin
uses it). New-chain wiring already excluded it; the two frontend comments that still claimed *"it runs
in saved chains"* are corrected.

Why graceful, not hard-delete: a hard removal would drop the id from the catalog, so a saved chain would
die with a cryptic *"unknown module 'multimin'"* instead of a message that tells you what to do. Why not
keep-and-consolidate: you asked for retirement — the trade-off is that a delivered chain containing a
`multimin` step can no longer reproduce its old output; it must be re-run in SandiMin.

Verified: full `cargo test` — **372 passed / 0 failed / 7 ignored**, whole-crate compile clean with **no
warnings** (the solver removal left no dead code / unused imports). New `multimin_is_retired_but_still_cataloged`
(registry + still-cataloged) and the converted end-to-end guard `phase7_generic_store_feeds_modules_and_mask`
(running `multimin` now returns a SandiMin error and writes no curves) both pass; every SandiMin/`multimin2`
test still passes. `tsc --noEmit` + `vite build` clean.

- [ ] **Try:** if you have any saved **workflow chain** with a Multimin step, run it — the step must
  fail with "…retired… Re-run this step with SandiMin (Advance ▸ Mineral Solver)", *not* run silently
  and *not* say "unknown module". Confirm SandiMin (Advance ▸ Mineral Solver) still runs normally. New
  chains: the step picker must not offer Multimin.

## Round 80 — R21: ML training wells that contributed zero samples were silently dropped (2026-07-24)

Supervised ML pools labelled rows across the selected training wells. `fetch_curve_frame` returns an
**all-NaN** column for any curve a well lacks, so a training well with **no target curve under the
chosen mnemonic** (or no input, or fully masked) contributed **zero** rows through the `is_finite()`
filter — invisibly. Nothing recorded which training wells were actually used; `MlResult.wells` only
ever carries the *apply* wells. The `n_train < 10` guard never fires because the few real wells supply
tens of thousands of samples at 0.1524 m. So the run returned success with R²/RMSE, and the user
believed a 20-well model was fitted. The scenario is the *normal* one here: core-calibrated PERM/facies
models where CPERM or core-facies exist in a small minority of the field — select 20, have the target
tied to the log grid in 3, and you ship a "20-well field model" that is a **3-well model**, with a
wrong-mnemonic typo (CPERM vs KCORE) producing output identical to a correct run. The **Compare** button
in the *same file* (`run_ml_eval`) already warns about exactly this ("N of M training well(s)
contributed no samples") — only the **Run** button was silent.

Fixed by tracking, per training well, whether it moved the labelled pool at all, and collecting the
ones that didn't — whatever the cause (unreadable, missing target/feature, or fully masked). A new
`notes: Vec<String>` on `MlResult` carries a count summary ("{k} of {n} training well(s) contributed
no usable samples … the model was fit on the remaining {n−k}"), mirroring the `run_ml_eval` sibling;
`mlDialog` renders it as a `⚠` warning at the top of the results (glyph + `--warn`, honouring R16's
redundant-coding rule). The two **dead** `else { continue }` guards the finding flagged (the all-NaN
fallback made them unreachable) are gone, and the previously-silent `fetch_curve_frame` **error** branch
now also lands in the empty-well list instead of vanishing.

The honesty-critical logic — *which wells contribute nothing* — was extracted into a pure
`assemble_training` helper so it is unit-testable **without python** (the existing `run_ml` tests skip
when sklearn is absent). Backend + a small additive frontend note.

Verified: `cargo test ml::` — 11/11 green via the pinned 14.29 toolchain, incl. the new
`assemble_training_flags_wells_with_no_target` (a well with the target contributes all its rows; a
target-less well is flagged empty, not dropped) **and** the python-backed end-to-end tests, which ran
and passed — so the extraction didn't regress the real `run_ml` path. `tsc --noEmit` + `vite build`
clean.

- [ ] **Try:** run a supervised model (e.g. regression PERM, or k-NN facies) over a group where the
  **target** curve exists on only some wells — select 10+ training wells, of which only a few actually
  carry the target under the chosen mnemonic. The results panel must show a **⚠** line like "7 of 10
  training well(s) contributed no usable samples … fit on the remaining 3", not a clean metrics-only
  card. Then run one where every training well has the target: **no** warning line.

## Round 79 — R20: the SQL console reported the LIMIT-capped row count as the true total (2026-07-24)

The SQL console runs every query through `runQuery(sql, 1000)`; the backend wrapped it in
`LIMIT 1000` and returned `total_rows = rows_out.len()`, so a query that matched 400,000 rows came
back as exactly **1000** and the panel printed **"1000 row(s)"** — no truncation marker anywhere,
indistinguishable from a genuine 1000-row result. And this is the *common* case, not an edge: any
row-level query against `standard_curves` blows past 1000 on a single well (a 2000 m interval at
0.1524 m ≈ 13,000 samples), so essentially every non-aggregate query a petrophysicist types — counting
shaly samples above a GR cut, sizing how many rows a cleanup would touch — silently truncated and then
reported the cap as the answer. The **DB Inspector** one dock over renders `${from}–${to} of
${total_rows}` from a real `COUNT(*)`, which actively trains the user to read `total_rows` here as a
true total.

Fixed with a **definitive** signal, not a guess. The sweep's verifier proposed a frontend-only
heuristic (`rows.length === limit`), but that mislabels a result that is *exactly* 1000 rows as
"maybe truncated" — a false positive. Instead the backend now fetches **`LIMIT + 1`**: if more rows
come back than the cap, it sets a new `truncated: bool` on `TablePage` and returns exactly `limit`
rows. A result that fills the cap exactly fetches `limit + 1` = one-too-few and reads as **complete**.
`truncated` is a shared-struct field; the paginated inspector path (real `COUNT(*)`) always sets it
**false**, so the flag cleanly means "`total_rows` may undercount the true result." The panel now
renders "1000 row(s) shown — display cap reached; more rows exist (not the total)" when set.

I chose the backend flag over the verifier's frontend heuristic deliberately: it's **exact** (no
exactly-at-cap false positive) *and* it's the only version that is **cargo-testable** — with the
in-app browser down this session, a frontend-only change would have no verification surface.

Verified: `cargo test inspector_tests` — 11/11 green via the pinned 14.29 toolchain, incl. the new
`readonly_query_flags_truncation_at_the_cap`, which locks all three boundaries (below cap → truncated;
above cap → complete; **exactly at cap → complete**, the heuristic's false positive) and confirms the
inspector path still reports its real `COUNT(*)`. `tsc --noEmit` + `vite build` clean.

- [ ] **Try:** in the **SQL console**, run a row-level query that exceeds 1000 rows — e.g.
  `SELECT depth, gr FROM standard_curves` on any well with a long interval. The footer must read
  "1000 row(s) shown — display cap reached; more rows exist (not the total)", **not** a bare
  "1000 row(s)". Then run a small query (e.g. `SELECT well_name FROM wells` on a <1000-well project):
  the footer must read a plain "N row(s)" with no cap marker.

## Round 78 — R19: the Field Dashboard claimed "FLAG curves written." on the path that writes nothing (2026-07-24)

Pressing **Compute** on the Field Dashboard runs `run_pay_summary` with `stats_only: true` — the
comment three lines above the write even says *"compute the stats, persist nothing."* Yet the panel's
status line asserted **"FLAG curves written."** `workflow.rs` gates the *entire* FLAG-write block
(both the in-place and the versioned branches) behind `if !req.stats_only`, so with `stats_only: true`
**nothing is written** — this is pinned by the unit test `pay_summary_stats_only_persists_nothing`
("stats_only must not write any FLAG_* curve", "…must not create a PAYFLAG log set"). A petrophysicist
who read that line and then opened a Log View or picked `FLAG_PAY` as a crossplot Z-curve found
nothing, with no error to explain it — a classic hunt-for-the-bug-that-is-a-lying-status-message. The
sharper case: if an earlier **Cutoffs & Summary** run already wrote `FLAG_PAY`, the dashboard claimed
"FLAG curves written" after a **cutoff tweak** while Log View still showed **stale** flags computed at
the *old* cutoffs — silently wrong, not merely absent.

Fixed the status line to tell the truth — *"Stats only — no FLAG curves written; run Cutoffs & Summary
to persist flags."* — which covers both the absent-flags and the stale-flags cases (it says **this**
Compute persisted nothing, so any `FLAG_*` in Log View is from a prior run, possibly at other cutoffs).

The lie was not confined to that one string: the same stale attribution — *"the Field Dashboard writes
`FLAG_*` in place / sets `skip_version`"* — was mirrored across **five** comments, the TS doc the sweep
named being merely a mirror of its Rust struct-doc source. Post-`stats_only` refactor the dashboard
sets `stats_only` **alone**; `skip_version`'s only real writer today is the **report/composite render
pass** (`report.rs:398`). Corrected all five (`ipc.ts`, `workflow.rs` struct-doc + write-branch +
test, `lib.rs`) so a future maintainer deciding whether `skip_version`/`stats_only` can be collapsed
reads the truth. All backend edits are **comment-only** — zero logic change.

Surfaced but deliberately **not** changed (behavior decision, needs your call): `lib.rs`'s silent
off-thread guard is `if req.stats_only && req.skip_version`, but the dashboard sets only `stats_only`
(`skip_version` defaults false), so it now takes the **job-card** path — every dashboard Compute posts
a "Pay summary" card, the opposite of the silence that guard was meant to give it. I documented the
gap in the comment rather than silently flipping the guard to `if req.stats_only`.

Verified: `tsc --noEmit` clean + `vite build` clean (frontend string/JSDoc); `cargo test
pay_summary_stats_only_persists_nothing` green via the pinned 14.29 toolchain (whole-crate recompile
clean, so the five comment edits didn't break anything, and the test is itself the proof the old
string lied). Browser-independent.

- [ ] **Try:** open the **Field Dashboard**, press **Compute**. The status line must read
  "…Stats only — no FLAG curves written; run Cutoffs & Summary to persist flags." — never "FLAG curves
  written." Then open a **Log View** on any well: there must be no *newly* written `FLAG_PAY`/`FLAG_SAND`
  from that Compute. To actually persist flags, run **Cutoffs & Summary** and re-open the Log View.

## Round 77 — R18: the report PDF silently dropped the Pay Summary section on error (2026-07-24)

Section 4 of the report did `run_pay_summary(...).unwrap_or_default()`, which collapses **both** an
`Err` (the `FLAG_*` write at `workflow.rs` failing — read-only DB, disk full, appender error) **and**
a legitimately empty result into the same empty `Vec`. The `if !pay_rows.is_empty()` guard then
dropped the **entire** section — header included — from the deliverable PDF, and `report_pages`
returned `Ok`. The PDF was indistinguishable from a well that genuinely has no pay, and
`export_report_batch` recorded the well in `written`, not `errors` — so a 540-well Mahakam batch could
ship 540 "successful" client PDFs, every one missing its pay table, with an **empty error list**. The
sharpest part: the pay numbers are computed in memory *before* the write side-effect, so a storage
error suppressed a table whose values were already fully renderable.

Fixed by emitting the section header **unconditionally** and branching on the `Result`: the table on
rows, an explicit **`Pay Summary unavailable — {e}`** note page on `Err`, and a "no curve data to
classify" note on the legitimately-empty case. It deliberately does **not** propagate the `Err`
(that would abort the whole PDF and lose the composite log pages the user did want over one bad pay
run) — the well is still counted as `written`, but the document now always carries a visible trace of
what happened. New `note_page` helper (section header + wrapped note) for the two non-table branches.

Verified: `cargo test` green via the pinned 14.29 toolchain; new `note_page_shows_section_header_and_message`
asserts the header, the well name, and the failure note all render (the old code rendered none of
them). Whole-crate compile clean. Backend-only, browser-independent.

- [ ] **Try:** export a report (or a **batch** export) for a well whose pay run can't complete — e.g.
  a well with no computed curves, or with the project DB file set read-only. The PDF must still show
  a **Pay Summary** header page with a note ("unavailable — …" or "no curve data …"), never a report
  that simply skips from Zone Parameters straight to the composite log pages with no pay section.

## Round 76 — R17: the legacy Multimin solver mixed PEF by the wrong physics (2026-07-24)

The legacy `multimin` module (superseded by SandiMin/`multimin2`, hidden from every UI picker but
**still registered** at `modules.rs:201`/`:240`, so `list_modules` returns it and any pre-existing
saved chain or dockview layout holding panel id `module:multimin` still runs it) pushed the **raw
per-electron PEF** straight into its NNLS linear system. Photoelectric factor does **not** mix
linearly by volume — the **volumetric** photoelectric factor `U = Pe·ρe` does. `multimin2` already
converts to U before mixing; the legacy solver never did.

The consequence isn't just biased numbers — it's the QC curve lying about **who is at fault**. With
the module's own defaults a 50/50 quartz-water sample carries a 0.30 b/e PEF residual (physical
PEF ≈ 1.38, the linear-Pe law gives 1.085) — **exactly 1.0× the default `SIG_PEF`** — so `RECON_ERR`
reads a full sigma of *model* error and reports it as *log* misfit, telling the user to re-condition
perfectly good PEF data. And the bias is directional: linear mixing under-predicts Pe for a
light-fluid mix, so NNLS over-assigns the high-Pe clay endpoint (3.10), inflating `VSH_MM` and
deflating `PHIT_MM`/pay — the wrong direction for Mahakam-delta shaly sand.

Fixed by converting every PEF endpoint **and** the measured reading to `U = Pe·ρe` before they enter
the system, and carrying the uncertainty in U space (`σ_PEF·ρe`). The `ρe(ρb)` relation is now a
single `pub(crate)` function in `multimin2` that **both** solvers call, so their Pe physics can't
drift apart (the standing hazard the finding flags). A live RHOB is required to get ρe; with RHOB
absent the PEF row is **dropped** rather than mixed wrongly, and the existing `n_tools < 3` gate then
skips the sample honestly. The module's own recovery test was **complicit** — it forward-modelled the
synthetic PEF with the *same* wrong law (`vs*1.81 + vw*0.36`), so it passed by construction and could
never catch this; it now forward-models with the U law, making it a genuine regression guard.

Verified: `cargo test` green (46 passed) via the pinned 14.29 toolchain. Two new tests lock the fix —
`multimin_pef_uses_volumetric_u_mixing` (the finding's 50/50 worked example: asserts the physical
PEF ≈ 1.382, that it differs from the raw-Pe law by > 0.25 b/e, and that the solver recovers 50/50
with `RECON_ERR` < 0.2) and `multimin_drops_pef_when_rhob_absent`. Entirely a backend physics change,
so it's cargo-proven and browser-independent. Backlog (unchanged, separate item): the two solvers
still keep divergent endpoint tables (legacy's hardcoded `PEF_CLAY 3.10` / `RHOB_CLAY 2.55` vs
`multimin2::multimin_library`); unifying or retiring the legacy module is its own decision.

- [ ] **Try:** if you hold a saved workflow chain or a saved dockview layout that references the
  hidden **Multimin — Mineral Inversion** module, re-run it on a well that has a PEF curve. `RECON_ERR`
  should no longer sit near a flat ~1σ floor on clean intervals, and `VSH_MM` should come down
  (PEF-misfit was inflating it). Wells without PEF are unaffected.

## Round 75 — R16: the Results-QC scorecard status was carried by brand colour alone (2026-07-24)

The one panel whose entire job is to tell you a result is degraded encoded each check's verdict
(`ok` / `warn` / `alert` / `na`) as a **9px colour dot only**, and the dot's colour reused the
**brand** `--accent` / `--accent2` / `--warn` tokens — chosen for branding, never for pass/fail
meaning. Two consequences, both live on **default** screens, not just demo skins:

- **Default theme:** `warn` mapped to `--accent2` = `#5f7350` (olive **green**) and `ok` to
  `--accent` = `#b5651d` (ochre). So a Buckles BVW check that trips its warn threshold paints green
  next to a passing check painted orange — **the degraded result reads as the clean one**. That is
  exactly the cardinal data-honesty rule inverted.
- **Halliburton skin:** `ok` = `#e31b23` (bright red) vs `alert` = `#b3141b` (dark red) — at 9px
  these are one colour, so every clean zone reads as an alarm across a 60-dot scorecard. And `warn`
  (graphite `--accent2`) collided with `na` (dimmed `--text-dim`).

Fixed with **redundant coding** — shape *and* hue, so neither channel alone has to carry the
verdict. (1) Each row now shows a **glyph** (`✓` / `⚠` / `✗` / `–`, the set `processingPanel`
already renders as monochrome text in this runtime) via `dot.textContent`, plus `role="img"` +
`aria-label` (`pass` / `warning` / `fail` / `not run`) so a screen reader announces the status word.
(2) New **semantic** `--qc-ok` / `--qc-warn` / `--qc-alert` tokens (green / amber / red) drive the
colour, decoupled from the brand palette — declared once in `:root` (all five brand skins are
light-background, so they inherit an identical, legible triple) with a brighter override in the two
dark contexts. `.rqc-dot` became a glyph carrier instead of a filled circle. This also removes a
standing hazard: every future client skin previously re-rolled the meaning of the QC colours for
free; now it can't.

Purely additive — one DOM line + three CSS vars per theme + the `.rqc-dot` restyle; no computation,
threshold, or shared component touched, and the `na` text rows ("run SandiMin recon QC first", etc.)
and the CSV export were already fully readable and are unchanged. Verified: `tsc && vite build`
clean; grep confirms `--qc-*` defined in `:root` + both dark blocks and consumed only by the three
`.rqc-dot-*` rules, and that no `.rqc-dot` rule still references a brand token. Browser-observable
(needs the full Tauri app to populate a scorecard + a theme switch), and the in-app browser is still
down this session — so this carries a click-through Try line, and the exact before/after colour
mapping is written out above (the `--accent` / `--accent2` / `--warn` hexes vs the new `--qc-*`
triple) rather than shown in a live screenshot.

- [ ] **Try:** run a full interpretation + SandiMin recon + Monte Carlo so the **Results-QC** panel
  shows a scorecard with a mix of pass / warn / fail rows. Each row must show a `✓` / `⚠` / `✗` glyph
  (not a bare dot). Switch the theme to **Halliburton** and to the **default** earth-tone: a passing
  check must never look like a failing one, and a warn must never look like a pass, in **any**
  palette. Confirm the glyphs stay monochrome (not colour-emoji).

## Round 74 — R15: the Vega panel keeps plotting pre-run values after a module run (2026-07-24)

The interactive Vega panel (the V1–V6 work) was the **only plot panel with no `dataVersion`
subscription**. Its siblings — `crossplotPanel`, `histogramPanel`, `pickettPanel`,
`correlationPanel` — each carry the same primed `appState.dataVersion.subscribe(… reload)` block, so
after a SandiMin / equation run they re-fetch and redraw with the new curves. The Vega panel didn't:
it subscribed only to `brushedDepths` and `themeVersion`, and `workspace.createPlot` only rebuilds on
`selectedWell`. So you could run SandiMin to recompute SW, watch the crossplot beside it redraw with
the new cloud, while the Vega scatter of the **same two curves** silently kept showing the pre-run
values — **two contradictory clouds on screen, the stale one presented as a clean result**. That is
exactly this app's cardinal data-honesty violation, and the Vega panel is the one with the SVG/PNG
export path, so a stale cloud can walk straight into a client deliverable. A second symptom: the newly
written curves (`MM_PHIE`, `MM_SW`) never appeared in the X/Y/Colour/Group dropdowns until the panel
was closed and reopened — which reads as "the run didn't write the curves."

Fixed by mirroring the sibling pattern: a **primed** `dataVersion` subscription (first synchronous
fire swallowed, so panel build doesn't double-load) that refills the four curve selects from a fresh
`loadCurveNames()` and calls the existing `render()` (which re-fetches through `getCurveData` and is
already race-guarded by its `gen` counter + `disposed` check). Released in `dispose` alongside
`unsubBrush`/`unsubTheme`. The refill is done by a small `refillCurveSelect` helper that **preserves
the current selection** — a curve that has vanished from the catalog is kept as a leading option so
the axis never silently jumps to a different curve. A `dataVersion` bump resets the vega zoom/pan (a
full `render()`), which the file already accepts explicitly for theme repaints. `loadCurveNames`
failure is caught and still triggers a re-render, so a fetch error surfaces through `render`'s own
"Failed to load curves" path rather than freezing on stale data.

Verified: `tsc && vite build` clean. A 20-check headless harness pins the `refillCurveSelect`
invariant — selection preserved across curves added / removed / renamed, the `— None —` and
`By zone` lead options never duplicated, every outcome resolves to an existing option, idempotent.
The fix is a line-for-line copy of a subscription proven in four sibling panels, so the wiring is
verified by construction; the live redraw itself is browser-observable but needs the full Tauri app
to bump `dataVersion` (a module run), and the in-app browser is still unresponsive this session — so
this one carries a click-through Try line rather than a captured screenshot.

- [ ] **Try:** open a **Vega** scatter of PHIE vs SW for a well, then run **SandiMin** (or any module
  that recomputes SW). The Vega cloud must redraw to match the crossplot beside it — not keep the old
  cloud — and the newly written `MM_*` curves must now be pickable in the X / Y / Colour / Group
  dropdowns **without** closing and reopening the panel. Confirm your current axis selections are
  preserved across the redraw.

## Round 73 — R14: finish the innerHTML sweep R9 deferred — a real well-name XSS was still open (2026-07-24)

R9 closed the five interpolated-`innerHTML` sites it scoped but explicitly deferred "the full 17-site
sweep." Finishing it turned up a **genuine miss of the same RCE class**: `autoCorrDialog` builds an
error row as `tr.innerHTML = \`<td>${wellName}</td><td colspan=4>${wp.error}</td>\``, and `wellName`
is the **LAS-supplied `~W WELL` value, stored verbatim** — the exact R9 vector. With `csp: null`, a
hostile header injects markup into the autocorrelate results table: the same XSS→(via `save_png`)→RCE
reach R9 was about, at a site R9 didn't touch.

Swept **all 14** interpolated-`innerHTML` sites across `autoCorrDialog` / `zonesDialog` / `workspace`
/ `dashboardPanel` / `topsPanel`. Every interpolated **string** value is now wrapped in `escapeHtml()`
(the safeDom primitive); numeric interpolations (`.toFixed()`, `.length`) are left alone (they can't
carry markup). The genuinely untrusted ones: the well name (autoCorr), zone / param names and values
(zones — import-supplied), and backend error strings (`workspace` `${err}`). The rest — dashboard
flag/metric labels, the tops empty-state text, panel/kind labels — are app-controlled today, but
escaping them too keeps the invariant total, so a future dynamic value can't silently reopen a hole.
Table-row sites keep `innerHTML` with `escapeHtml()` on the data (concise, and structure-preserving);
R9's message-only sites had used DOM construction — both are safe, chosen per context.

Verified: `tsc && vite build` clean. A grep of the whole `src` confirms **every** `innerHTML`
interpolation is now `escapeHtml(...)` or a number — **zero** unescaped string interpolations remain
— and there are **no** other HTML-injection sinks (`insertAdjacentHTML`, `outerHTML =`, or
`+`-concatenated `innerHTML`) anywhere. `escapeHtml`'s inertness is the R9-established `textContent`
round-trip (markup → text). This is entirely browser-independent, which is just as well — the in-app
browser is still unresponsive this session.

Still deferred (unchanged from R9): a real `csp` (risks vega-embed / CodeMirror inline styling —
wants live testing the browser can't give right now) and scoping `save_png`.

- [ ] **Try:** import a LAS whose `~W WELL` value is `<img src=x onerror=alert(1)>`, then run
  **Autocorrelate** across wells so one returns an error row. The Well cell must show the literal
  text `<img …>` (not a broken image, and nothing executing). Same for a zone renamed with markup in
  the **Zones** pane, and a computed-curve error surfaced in a panel.

## Round 72 — R13: six module-dialog Run buttons stop rendering as native grey buttons (2026-07-24)

Cosmetic-consistency, but wrong on every theme. The **Facies Tie / HFU / Lorenz / ML / SHF /
Thomeer** dialogs (and the **Workflow** runner) build their Run button as a raw `<button>` whose only
class is `primary` — and `.primary` had **no standalone CSS rule**. It exists only in compound
selectors (`.lp-btn.primary` accent-override, `.guard-confirm button.primary`, and
`.workflow-run-row .primary` which set only `font-weight: 600`), and the app's base `button` rule
sets nothing but font inheritance. So these buttons fell through to the browser's **native grey UA
button** — the only Run buttons in the app not accent-filled. The class was added expecting a
primary-button style that was never written as a base.

Fixed with one scoped rule — `.mc-run-row .primary:not(.mm-run-btn), .workflow-run-row .primary` —
giving the app's accent primary look (accent fill, white text, 4px radius, 6px 24px padding, bold,
`--accent-dim` hover, dimmed `:disabled`), mirroring the multimin `.mm-run-btn`. Scoped to the two
run-rows so nothing else moves: **multimin keeps its own treatment** (excluded via `:not(.mm-run-btn)`
— it legitimately carries both classes and lives in `.mc-run-row`), the **Monte Carlo** run button
isn't a `.primary`, and `.lp-btn.primary` / `.guard-confirm` / the autosave restore button sit in
other containers. The old `.workflow-run-row .primary { font-weight: 600 }` folds into the new rule,
so the Workflow Run button also becomes accent-filled and matches its siblings. Per the R11 lesson I
verified `--accent` and `--accent-dim` are defined in **all 8** palettes first — no repeat of the
undefined-variable trap.

Verified: `tsc && vite build` clean (bundled CSS 182.21 kB, +0.44 kB). Deterministic: I read all
seven call sites and confirmed each appends its `.primary` button into `.mc-run-row` (six) or
`.workflow-run-row` (one), so the selector matches; at specificity 0,3,0 nothing competes for the six
targets, so the accent styling wins where before there was no rule at all. What I could **not** do:
capture a live screenshot — the in-app browser was unresponsive again this session, and the real
dialogs need the Tauri backend plus a user action to open. So this rests on the selector-match +
specificity argument, not a pixel view.

- [ ] **Try:** open **Facies Tie**, **HFU**, **Lorenz**, **ML**, **SHF**, **Thomeer**, and the
  **Workflow** runner. Each Run button should now be an accent-filled button (like Multimin's and the
  plot Run buttons), darkening on hover — not a native grey one. Multimin's Run button should look
  exactly as before.

## Round 71 — R12: one cutoff source, so Monte Carlo net-pay reconciles with the pay summary (2026-07-24)

A data-consistency finding — the quiet-but-expensive kind. The pay-cutoff quartet (VSH ≤ / PHIE ≥ /
SWE ≤ / PERM ≥) was independently hard-coded in **five** panes, and two had drifted: **Monte Carlo**
*and* the **Results-QC cutoff-sensitivity probe** defaulted to **PHIE ≥ 0.08 / SWE ≤ 0.5**, while the
canonical **Cutoffs & Pay Summary** uses **PHIE ≥ 0.1 / SWE ≤ 0.6**. So an MC net-pay run "with
defaults" used *different* cutoffs than the deterministic pay summary "with defaults" — the P50 net
would not reconcile with the deterministic net, and nothing on screen said why (it reads like an
uncertainty result, not a cutoff mismatch). The MC settings tooltip even said **"Cutoffs match the
pay summary"** — an invariant the code documented but did not enforce. Separately, only the pay
summary loaded the project's **saved** default cutoffs; the other four ignored them and showed frozen
literals, so a saved cutoff set never propagated.

Fixed at the root: one shared `src/ui/cutoffs.ts` — a canonical `DEFAULT_CUTOFFS` (VSH 0.5 / PHIE 0.1
/ SWE 0.6 / PERM off) and one `loadCutoffDefaults()` (the saved `cutoffs/__default__` document merged
over the constant → always a complete, finite set). **All five** panes (cutoff editor, pay summary,
Monte Carlo, report, Results-QC) now seed from it, and the cutoff editor's save-fallbacks route
through the same constant. The defaults are now **un-copyable**: "matches the pay summary" is
structurally true, and every pane honours the user's saved cutoffs.

Not a physics change — cutoffs a user explicitly enters are untouched; only the **defaults** a pane
opens with, and only for MC and Results-QC (0.08 / 0.5 → 0.1 / 0.6). Anyone who wants 0.08 sets it
once via **Save default cutoffs** and it now flows everywhere, instead of living in two panes by
accident.

Verified: `tsc && vite build` clean. Headless (`scratchpad/cutoffs_check.mjs`, the shared merge
logic ported verbatim), **10 checks**: canonical fallback on missing / partial / garbage / NaN /
Infinity saved data; finite saved values pass through; `perm_min = 0` kept as a real value (not
"off"); and the two regressions — a fresh project now yields **PHIE 0.1 not 0.08** and **SWE 0.6 not
0.5**. Not verified live (needs the Tauri backend + a project with saved docs); the Try line covers
the click-through.

- [ ] **Try:** in **Cutoffs & Pay Summary** set custom cutoffs and click **Save default cutoffs**.
  Open **Monte Carlo**, the **Report**, and **Results-QC** — each should now open pre-filled with
  those saved values (before, MC and Results-QC showed 0.08 / 0.5 regardless). On a fresh project all
  five should read VSH 0.5 / PHIE 0.1 / SWE 0.6. Run MC and the pay summary with defaults — the
  net-pay now rests on the same cutoffs.

## Round 70 — V6: Raincloud plots in the Vega panel (your PtitPrince ask) (2026-07-24)

A requested feature, not a review item. New **Raincloud** chart type in the Vega panel: per group a
half-violin KDE **cloud** (top), a **box** (IQR + median + Tukey whiskers, middle), and a jittered
strip of raw **rain** points (bottom). A **Group** dropdown drives it — *By zone* (each sample
assigned to the zone whose interval contains its depth) or *any curve* (rounded to categorical
classes: rock-type / facies / RT). It shares the value (X) axis; Y / Colour / Trend don't apply and
dim out. Themed, exportable (PNG/SVG/PDF) and last-used-persisted like the other Vega types.

Design worth recording: Vega-Lite has no native violin, and its density / boxplot / facet paths
fight the panel's `width:"container"` autosize (every other chart type is single-view). So the
geometry — Gaussian KDE (Silverman bandwidth, robust via min σ, IQR/1.349), per-group quartiles +
1.5·IQR fences, and the jitter — is computed in **JS** and drawn with trivial single-view marks
(`area` / `bar` / `rule` / `point`) on a synthetic group-lane y-axis. That drops into the existing
sizing / export / repaint / theme machinery unchanged, and — the real payoff — makes the whole
thing **numerically verifiable** instead of needing a screenshot.

Data honesty, per the cardinal rule: samples outside every zone form an explicit **"(outside
zones)"** lane instead of being dropped; a group curve with **>24** distinct values is **refused**
with a message pointing at categorical curves, not silently binned into noise; samples missing a
group value are counted and surfaced in the status line ("· N with no <curve>").

Verification: `tsc && vite build` clean (vegaPanel lazy chunk 864.35 → 870.82 kB, main bundle
unchanged). Headless (`scratchpad/rc_geom.cjs`, real vega-lite compile + vega render), **13 checks,
all green**: geometry invariants — cloud never inverts, stays inside its lane and actually bulges;
quartiles monotonic; whiskers bracket the box; every rain point sits in its lane; recovered medians
match the injected distribution order — **and** the exact production spec shape both *compiles* with
`container` sizing and *renders* with the real empty-top-data + per-layer-data structure (the two
ways it differs from a toy spec). What I could **not** do: see it in the running app — the panel
needs the Tauri backend plus a well with curves, and the in-app browser was unreliable this session,
so there is no live screenshot. Correctness rests on the numeric geometry proof + the compile/render
check, not a pixel view.

- [ ] **Try:** open a **Vega Chart** panel, Type → **Raincloud**, X → **PHIE** (or Sw), Group → **By
  zone**. Expect one cloud+box+rain stack per zone, each labelled, sharing the value axis, medians
  landing where each zone's distribution centres. Switch Group to a **rock-type / facies** curve →
  one lane per class. Pick a **continuous** curve as Group → it should refuse with "pick a
  categorical curve". Hover a point/box for the tooltip; export SVG and PNG.

## Round 69 — R11: the depth-scale dropdown gets its themed background back in every palette (2026-07-24)

Small but real, and wrong in **all eight** theme blocks. `.lv-scale` — the log-view depth-scale
`<select>` (1:20 … 1:5000, top-right of the track toolbar, `logViewPanel.ts:167`) — set
`background: var(--bg)`. No palette defines `--bg` (the contract is `--bg-app` / `--bg-panel` /
`--bg-panel-alt` / `--bg-hover`), so the declaration was **invalid at computed-value time** and
`background` fell to its initial `transparent`. Being a native `<select>`, it never vanished — it
just quietly stopped matching the filled, themed look of every other control, worst on the brand
palettes (Pertamina/Halliburton/SLB/LAPI-ITB) where a transparent control on tinted chrome reads as
unstyled. This is exactly the failure mode a linter misses: no parse error, no total disappearance.

Fixed to `var(--bg-app)` — the canonical themed form-control surface, the same variable
`.form-control` uses and the one `.mm-dialog select` was explicitly switched to (there's a comment
at `styles.css:4399` enforcing "the same brand surface … so the whole app reads one theme"). The
depth-scale select now matches every other themed input, in all eight palettes.

Verification: grep-proved `--bg` is defined in **zero** palettes and `--bg-app` in **all eight**, so
the change is a deterministic computed-value swap (IACVT-transparent → the palette surface), not an
empirical guess; `tsc && vite build` clean. The one thing I could **not** do live: read the real
select's computed background in-app — the control only exists once a log view is open, which needs
the running Tauri backend, and a static-snapshot of a standalone repro can't run JS to read the
computed style. So this rests on the deterministic CSS proof + the grep evidence, not a screenshot.

- [ ] **Try:** open a log view, look at the depth-scale dropdown at the top-right of the track
  toolbar. It should have the same filled input surface as the other controls, not a see-through
  background — check one brand palette (e.g. Pertamina) where it was most visible.

## Round 68 — R10: a failed undo no longer vanishes silently while claiming success (2026-07-24)

A data-integrity finding, and squarely the cardinal rule of this whole review: a failed
operation must never look like a clean one. `undo()` did `undoStack.pop()` **before**
`await action.undo()` — it committed the stack change before the risky effect. Most reversals are
database writes (`upsertTop`, `updateStandardSample`, …), so when one rejects — a DB locked mid
autocorrelate-sweep, a since-deleted well, a value the Rust side refuses — the popped action was
**gone from both stacks**: un-undoable (popped) and un-redoable (the `redoStack.push` never ran).
Worse, both callers used `void undo().then((label) => …)` with **no** rejection handler, so the
`.then` fulfilment never fired — the status bar kept the *previous* success message — and the
rejection became a console-only unhandled promise. From the user's seat: press Ctrl+Z, the DB
reversal silently failed, the status bar still reads like it worked, and the action has disappeared
so they can't even retry it. The edit is still in the database, contradicting what the UI implies.

Fixed by only mutating durable state after the effect resolves. `undo`/`redo` now capture the
action, run the reversal inside a `try`, and on rejection **push it back where it was** (staying
reversible) and **re-throw** so the caller can report it. Both callers (`undo.ts` hotkeys +
`ribbon.ts` quick-access toolbar) grew a rejection arm: *"Undo failed — the change was not undone:
<err>"*. Both `undo`/`redo` are also now **serialized** through a small promise chain: a held Ctrl+Z
auto-repeats keydown, and without this the unawaited calls overlapped — running two reversals
against the single-writer DuckDB at once. The chain reverses one action at a time, in order, and
absorbs each outcome so one failed reversal doesn't stall the queue behind it. LIFO is preserved: a
top action whose reversal keeps failing blocks undoing *older* ones rather than silently skipping it
and reversing out of order.

Verification: `tsc && vite build` clean. Beyond that I ported the shipped `serialize`/`undo`/`redo`
bodies **character-for-character** into a headless Node harness (`scratchpad/undo_check.mjs`, real
promise scheduling, stub stacks) — 16 checks, all green: a rejected undo keeps its action and
rejects the promise; nothing leaks to the redo stack; a held Ctrl+Z reverses newest-first with max
one DB write in flight and never double-reverses the same action; a transient failure is retried by
the next request once the cause clears; LIFO holds throughout. The harness even caught a wrong
assumption of mine (I expected a persistently-failing top action to fall through to an older one —
it correctly does not). This is a stronger verification than R9 got; what it does **not** cover is
the live desktop path (a real rejected Tauri write), which is the click-through below.

- [ ] **Try:** in the DB inspector, double-click a `standard_curves` cell and commit an edit, then
  make the underlying write fail on undo — easiest repro: start a long Autocorrelate sweep (holds
  the DB), then immediately press Ctrl+Z. The status bar must say **"Undo failed — the change was
  not undone: …"**, the Undo button must stay **enabled** (action still on the stack), and a second
  Ctrl+Z after the sweep finishes must then undo it cleanly. Nothing should vanish silently.

## Round 67 — R9: a hostile LAS well name can no longer run code (2026-07-24)

F4c, a genuine remote-code-execution chain, verified end to end before fixing:
`parsers.rs::extract_well_name` stores the `~W WELL` value **verbatim** (trims whitespace, filters
no characters — confirmed at `parsers.rs:552`), and `vegaPanel.ts:504` wrote `well.well_name`
straight into `innerHTML`. With `tauri.conf.json` carrying `"csp": null` (confirmed), an
`<img src=x onerror=…>` embedded in a vendor's LAS header parses into the live document and runs.
The finding traces it on to the unscoped `save_png` write → a `.bat` in the Startup folder. LAS
files come from service companies, partners and clients, and this app ships client-brand palettes,
so it is meant to leave the developer's machine — "our tool executed a payload from a vendor's LAS"
is a reputational event, not a lint.

Fixed at the vector — escaping, not the sink — so it closes the path for **every** invokable
command at once, not just `save_png`. Scope per the finding's own recommendation: the three
`vegaPanel` message lines (well name + curve mnemonics, both LAS-supplied) plus the two DB-panel
error paths, building each with `textContent` via a new shared `messageNode` helper instead of
`innerHTML`. While there, the three byte-identical private `escapeHtml` copies
(dashboard/inspector/tops, plus inspector's `escapeAttr`) collapse into one `src/ui/safeDom.ts`, so
the next interpolated-innerHTML site has an obvious safe primitive to reach for. Left as backlog,
per the finding: the full 17-site sweep, a real `csp` (risks breaking vega-embed's inline styling),
and scoping `save_png`.

There is now **zero** interpolated-`innerHTML` in the three touched panels (grep-verified). The
DB-cell *value* renderers were already safe (`td.textContent`), so the exposure was only the
message/error lines.

Verification: `tsc && vite build` clean — which type-checks the `replaceChildren`/`messageNode`
usage — and the inertness holds by construction (`textContent` never invokes the HTML parser). I
wrote a standalone repro that runs the exact old vs new paths against a live `<img onerror>`
payload, but **could not execute it — the in-app browser was unresponsive this session**, and the
true end-to-end path also needs the Tauri backend plus a crafted LAS import, which I can't stage
here. So this rests on the construction argument and the source-level proof, not a live repro.

- [ ] **Try (optional):** import a LAS whose `~W` block has `WELL. <b>x</b> : WELL`, open a Vega
  chart on it with a zone that yields no samples. The empty-state line must show the literal text
  `<b>x</b>` (not a bold `x`, and certainly nothing executing). Same for a SQL error in the SQL
  console.

## Round 66 — R8: the test suite compiles from a fresh clone again (2026-07-24)

Not a runtime item — a build-integrity one from the F-sweep. `db.rs`'s WAL-recovery test
`include_bytes!`s two fixtures at **compile time**: `corrupt_torn.duckdb` and `corrupt_torn.wal`.
The `.wal` was committed, but the `.duckdb` was silently caught by the repo-wide `*.duckdb` ignore
rule (there to keep well databases out of the repo). A missing `include_bytes!` file is a hard
compile error, so **the whole `src-tauri` test suite could not build from a fresh clone or in CI** —
it only ever built for us because the file sat untracked in the working tree.

Before versioning it I checked it carries no well data: the `.duckdb` holds only the DuckDB header
and version string, the `.wal` only `create_schema`'s DDL (table/column names). It is a
freshly-created, schema-only project torn mid-write — exactly what the recovery test needs, and
nothing a client would recognise. A scoped `.gitignore` exception now tracks both, with a comment
recording that check and why a synthetic pair can't substitute (the test comment already notes a
garbage WAL doesn't reproduce the same DuckDB internal-error path).

Verified with `git archive HEAD`: a fresh checkout now materialises both fixtures (12288 + 3707 B),
byte-identical to what the test runs against. No code or test logic changed.

- [ ] **Try (optional, for CI/handover):** clone the repo somewhere clean and run
  `cargo test --lib` in `src-tauri`. It must compile — before this it failed at
  `include_bytes!("../tests/fixtures/corrupt_torn.duckdb")` with "No such file or directory".

## Round 65 — R7: the Cancel button is gone from jobs it could never stop (2026-07-24)

This is the **other half of R3's own acceptance criterion**, which I only did half of at the time.
R3 said: for each job kind, *either observe the cancel flag, or do not render the button.* R3
made "Cancelled" honest after the fact — a run that never observed the flag reports Completed, not
a false Cancelled. But the button was still offered on every active job, including the ~20-odd
monolithic ops that cannot observe it. Clicking it did nothing, silently. A control that does
nothing is the same lie R3 set out to remove, just on the other side of the click.

The split is structural, not a hand-maintained list. A `run_simple_job` worker is a bare
`FnOnce() -> Result` — it is handed **no** `JobHandle`, so it *cannot* poll the flag; every render,
export and single subprocess goes through it. A `run_job` worker gets a handle and every current
one polls (Import LAS, Equation, Module, Monte Carlo, ML, SandiMin, and the workflow chain). So
`run_simple_job` hardcodes `cancellable = false` and `run_job` takes it as an **explicit
parameter** — a future non-polling `run_job` caller is forced to pass `false` and cannot silently
inherit a button that would do nothing.

Active jobs that aren't cancellable now show a muted "can't be interrupted" tag where the button
was, so it reads as a deliberate status rather than a missing control.

cargo **373/0/7** (one new test: `cancellable` reaches the `JobView` both ways), release build and
`tsc && vite build` clean.

Not browser-verified — the panel only shows a button when there is a live job, and jobs exist
only under the Tauri backend, which `npm run dev` alone does not start.

- [ ] **Try:** run a per-well operation with many wells (a **Module** run, or **Monte Carlo**) and
  confirm the Processing panel still shows a working **Cancel** — click it and the run must stop
  early, reported Cancelled.
- [ ] **Try:** run a monolithic op — **Report → export PDF**, or **Composite → export SVG**, or a
  **core/tops/SCAL import**. The card must show **"can't be interrupted"** instead of a Cancel
  button. (Before this, it showed a Cancel that did nothing.)

## Round 64 — R6: the app can no longer fail to start without telling you (2026-07-24)

This was on the deferred list from the F-sweep, and it is the worst user-facing item on it.

Three `.expect()` calls ran **before the window was created** — `init_db_resilient` plus the two
launch migrations. The release profile sets `panic = "abort"` and `windows_subsystem = "windows"`,
so any one of them failing killed the process with **no window, no dialog and no console**. You
double-click SandiBumi and *nothing happens*. Nothing to read, nothing to send me.

`init_db_resilient` self-heals a corrupted WAL and nothing else, and the likeliest trigger is
completely mundane: **DuckDB takes an exclusive lock, so launching a second SandiBumi while the
first still has the project open used to silently kill the second one.** A read-only volume, a
network drive that dropped, or a file written by a newer DuckDB did the same.

The runtime path was already graceful — `open_project` returns a `Result` and reports failures
properly. Only startup panicked. So the open-and-migrate sequence is now shared between the two
(`project::open_and_migrate`), and startup treats failure as a value:

1. Open the real project. Normal case, unchanged.
2. If it fails → open a throwaway `sandibumi-recovery-<stamp>.duckdb` in the temp folder, so the
   app starts and can explain itself.
3. If *that* fails → memory only.

All three land you in a running app showing a dialog that names the file, quotes the DuckDB error
verbatim, says plainly that **your project file has not been changed**, and points at the likely
cause. The failed project is deliberately **not** added to the recents — a file that would not
open should not be the first thing tried at the next launch. "Save As" follows the recovery, so a
recovered session cannot checkpoint the temp database and then copy the project that never opened.

Two tests pin the contract startup depends on: an unopenable path returns `Err` rather than
panicking, and a fresh recovery file really is created with a working schema. cargo **372/0/7**,
release build and `tsc && vite build` clean.

I could not exercise this end-to-end myself — it needs two real app instances against a real
project, and the first instance would open your working file read-write.

- [ ] **Try:** launch SandiBumi normally and confirm nothing has changed. Then, **with it still
  open**, launch it a second time. The second window must appear (it used to not appear at all)
  with a dialog naming your project and the lock error. Click Continue — you should be in an
  empty temporary project. Confirm your real project is untouched: close both, reopen once, and
  check your wells are all still there.
- [ ] **Try:** check the recents dropdown after that — the failed project must **not** have been
  pushed to the top of the list by the second instance.

## Round 63 — refining R1–R5: one regression R5 introduced, one defect it met (2026-07-24)

I re-read the five landed diffs adversarially instead of trusting my own summary of them. Two of
the six findings are real defects; the rest are hardening. **Round 62's claim that saving before
the editor mounted was "already safe" was wrong** — see the strikethrough below.

**1. R5 introduced a data-loss window (the important one).** `renderEquationEditor` calls
`this.editor?.destroy()` but never nulls the field. `destroy()` tears down the DOM yet leaves
`view.state` readable — **a destroyed view is not a null view** — so `readFormIntoCurrent` kept
answering with the *previously open* equation's text. That was harmless while the mount was
synchronous, because the field was reassigned on the very next line. R5 put an `await` in that
gap. Result: open equation A, pick equation B, hit **Save** before the CodeMirror chunk finishes
loading, and **A's script is written into B**. The guard I described in Round 62 as already
present did not exist; it does now (`this.editor = null`).

**2. Cancelling a LAS import still reported every file as imported.** R3 added a cancel path that
returns an entry with neither a well nor an error, and `ribbon.ts` counted success as `!r.error` —
so cancelled files counted as imported. Cancel an import of 120 files at file 75 and the status
line read **"Imported 120/120 well(s)"**, with that same sentence written into the permanent
History, followed by 45 per-well notes each saying "cancelled". Exactly the class of defect R4
existed to close, created by R3 landing next to it. Counting is now partitioned on `well_id` —
the only field that proves a well row was actually committed — and cancelled files are reported
as their own count.

**3. The R1 wire guard had a hole on the Rust side.** `SPEC_FIELDS` was hand-maintained and only
TypeScript was compared against it, so a field added to the Rust struct carrying
`#[serde(default)]` — the one shape that deserializes happily forever — could sit there
permanently unknown to `ipc.ts`. The contract is now also checked against **serde's own** field
list, recovered from the `deny_unknown_fields` error text. Proven by dropping a name from the
contract and watching it fail. Worth noting: adding a field to the struct *also* breaks the build
outright, because the tests construct it with struct literals — an incidental second layer I
hadn't credited.

**4–6. Hardening.** The dashboard's row filter set its "n excluded" counter as a side effect while
the CSV handler called it outside the render path, so the note could describe a different
selection than the table — now returns the count with the rows. The out-of-range parameter check
rejected non-finite zone values but let non-finite request values through (unreachable today, as
JSON carries neither NaN nor Infinity — but two rules where there should be one). Plus the
"—" explanation sentence, which parsed as gibberish because the em dash it was describing sat
mid-sentence unquoted.

cargo **370/0/7**, release build and `tsc && vite build` clean. Eager chunk 664.53 kB (+0.18 kB).

- [ ] **Try (the R5 fix, most important):** Inspector → Equation. Open an equation with a
  distinctive script, then pick a **different** equation from the dropdown and hit **Save**
  immediately — before the editor finishes appearing. The saved script must be the one you
  selected, not the one you were just looking at. Re-open both to confirm neither was overwritten.
- [ ] **Try (the import fix):** start a LAS import of a large folder, hit **Cancel** partway.
  The status line must read "Imported *n*/*N* well(s). *m* cancelled before import." with
  *n* matching the wells that actually appeared in the tree — not *N*/*N*. Check the History
  panel says the same thing.
- [ ] **Try (the dashboard fix):** Field Dashboard with at least one uninterpreted well in scope —
  the "*n* interval(s) excluded" note must match what the table shows, and **Export CSV** must
  contain only the interpreted rows.

## Round 62 — R5: 461 kB of CodeMirror off every launch (2026-07-24)

I suspected this during scouting — CodeMirror is a dependency and the Vega spec editor is
documented as dynamic-importing it, yet **no CodeMirror chunk appeared in the build output at
all**. F4a found where it went: `inspectorPanel.ts` imported it **statically**, so the whole CM6
stack sat in the eager startup bundle — **461.3 kB, 41.0% of it** — loaded on every launch for a
panel most sessions never open. That also silently defeated `vegaPanel`'s own dynamic import:
once a module is in the eager chunk, deferring it elsewhere buys nothing.

The Inspector now dynamic-imports it the same way vegaPanel does, and fetches the Python language
mode **only** when the equation is Python — so a Rhai-only session never pays for the lezer parser
either.

The mount became async, which needed two guards: a generation counter, so a re-render (equation
picked, language switched) that lands while the import is in flight owns the host and the stale
mount drops itself; and a check that the host is still connected. ~~Saving in the window before the
editor mounts was already safe — `readFormIntoCurrent` falls back to the stored script rather than
reading a null editor.~~ **← wrong, corrected in Round 63.** It needed a third guard: the editor
field was destroyed but never nulled, and a destroyed CodeMirror view still answers with the *old*
equation's text.

**Measured, not estimated:**

| | before | after |
|---|---|---|
| eager `index` chunk | 1,125.01 kB | **664.35 kB** |
| CodeMirror | in the eager chunk | 3 lazy chunks totalling **461,537 B** |

That 461,537 B matches F4a's predicted 461.3 kB to the byte. The old baseline was quoted in three
places across the tracker and the review prompt; all now record the new one.

- [ ] **Try:** launch the app and confirm it feels no different — then open **Inspector →
  Equation**. The editor should appear after a brief first-load (the chunk fetching), then behave
  exactly as before: syntax highlighting on a Python equation, none on Rhai. Switch the language
  dropdown a few times quickly — the editor must track the last selection, not a stale one.

## Round 61 — R4: four places that reported success they hadn't earned (2026-07-24)

Your cardinal rule is that a degraded or failed result must never be presented as a clean one.
The review found four live violations; this closes all four.

**1 · Monte Carlo swallowed module errors.** A failed chain step was dropped with `if let Ok`,
leaving the pool unchanged — so every downstream step read NaN and the study came back as a
**P10 = P50 = P90 table of zeros** with nothing to explain it. The trigger needs no unusual setup:
`gascorr` with `OPT_GATE = FLAGGED` (the manifest **default**) on a well where `condflag` was
never run. Its own guard exists to stop exactly this, and the message it raises is the actionable
one — and it was being thrown away one call site from where it was written. The first failure is
now captured and the well is reported **Failed** carrying the module's own text.

**2 · A failed full-curve load reported a clean import.** When the generic-store load fails, the
six standard curves are in but PEF, CALI, DTS and any second run are not. That went to `eprintln!`
only — invisible in a release build — while `ImportResult` said success. Every later module that
resolves those mnemonics silently gets all-NaN, with no trace of why. It now rides in the existing
per-well warning. (The import status line also said "N well(s) had depth issues" for *any*
warning; it now says "imported with warnings" and lets the per-well notes speak.)

**3 · Pay summary printed a fabricated zero.** A well whose VSH/PHIE/SWE were never computed
classifies to NaN everywhere, leaving Net 0.0 / N/G 0.00 / HPV 0.00 — **byte-identical to a
genuine wet zone**, and `report.rs` puts it in a client PDF. Rows now carry `n_classified`; when
it is 0 the dialog shows "—" with a note, the PDF prints "-", and the **Field Dashboard excludes
the row entirely** — there, zeros would have dragged every median and box plot down with data
that does not exist, which is worse than a mis-rendered cell.

**4 · The ML dialog claimed wells it never wrote.** It reported the *scope* count, so a k-means
run on a 12-well group where only 2 wells have NPHI+RHOB said "wrote FACIES_ML to 12 well(s)" —
and wrote that into the **permanent History**. The backend was honest the whole time; only this
dialog lied. Now `ok/total`, with "N well(s) need attention", and no History entry at all when
nothing was written.

cargo **370/0/7**, tsc + build clean.

- [ ] **Try (1):** build a chain with `gascorr` (leave `OPT_GATE` at FLAGGED) on a well where you
  have not run `condflag`, and run it through Monte Carlo. Before: a tidy table of zeros. Now:
  the well is marked Failed with gascorr's own explanation.
- [ ] **Try (3):** import a LAS and press **Compute Summary** without running any interpretation.
  Before: Net 0.0 / N/G 0.00 / HPV 0.00, indistinguishable from a wet well. Now: "—" plus a note
  telling you to run VSH/PHIE/SWE. Check the Field Dashboard too — those rows are excluded and
  counted.
- [ ] **Try (4):** run ML on a group where only some wells carry the feature curves, then open
  **History**. It should record `ok/total`, not the full group.

## Round 60 — R3: Cancel stops telling you it worked (2026-07-24)

Found independently by **two** review passes that weren't told about each other (F2d and F5e) —
which is why I trusted it before reproducing it.

**The lie.** A Cancel button is rendered for every active job, but only about **5 of ~27 job
kinds** ever read the flag. The rest ran to completion, **committed their writes**, and were then
reported as **"Cancelled"** — with every item ticked green. Pick the wrong folder, start a 120-file
LAS import, hit Cancel at file 3: all 120 wells were still created, the status bar said
"Imported 120/120 well(s)", and the Processing card said Cancelled. Two contradictory reports of
the same run, and 120 wells you thought you'd stopped.

**The systemic fix is one idea:** *Cancelled* must mean the work actually **stopped**, not that
you clicked. `JobHandle::is_cancelled()` now records the fact that a worker **observed** the flag,
and `run_job` finalizes on that observation instead of on the flag. A worker that never polls
cannot have drained early, so its honest report is **Completed** — the cancel simply arrived too
late. This corrects every job kind at once, with no per-call-site churn.

Two paths read the raw flag instead of going through `is_cancelled()` — chain steps and module
runs. Chains already set their own terminal state so they were fine, but **module runs were not**:
they would have started reporting a genuinely drained run as Completed. That is the same lie in
the opposite direction, so those mark the observation explicitly. I caught this by tracing the
raw-flag readers rather than by a test failing.

**Cancel is now real in three more places**, each a per-item loop that simply never polled:
LAS import (checks *before* each DB write, so it stops wells being created), Rhai equations
(previously the Cancel button's behaviour depended on the equation's **language** — the Python
branch drained, the Rhai branch didn't, same job kind, same button), and the ML write-back loop.

cargo **369/0/7**, two new tests pinning the distinction the whole fix rests on: a set flag alone
is not evidence, and observation is shared across the handle clones rayon workers hold.

- [ ] **Try:** select a folder of ~20 LAS files, Import, and hit **Cancel** after a couple land.
  Before: all 20 imported and the card said Cancelled. Now: the remaining files are skipped and
  marked "cancelled", and the card says Cancelled *because it genuinely stopped*.
- [ ] **Try:** run any **export or render** (composite PDF, report), hit Cancel. It cannot be
  interrupted, so it finishes — and now correctly reports **Completed**, not a false "Cancelled".

**Still open** (deliberately not in R3): the Processing panel offers a Cancel button for *every*
job with no capability check, so on monolithic ops it is now honest but still inert — `JobView`
needs a `cancellable` flag and the button should be hidden when false. Also unfixed: Monte Carlo
polls only between wells (a single-well 100k-realization run is uncancellable), Report batch uses
the single-unit helper so it has no per-well progress, and both Autocorrelate commands hold the
DB lock across the whole sweep.

## Round 59 — R2: three panics reachable from your own data (2026-07-24)

All three came out of the F2a review pass (`docs/review_sweep/F2.md`). Release builds set
`panic = "abort"`, so none of these was a caught error — they killed the run, and one of them
poisoned the DB mutex for the rest of the session.

**1 · A `NaN` top depth panicked Auto-correlate.** `pandas.to_csv(na_rep='NaN')` and `np.savetxt`
write a literal `NaN` for a missing marker, and `f32::from_str` parses that happily — nothing
between the tops importer and the database tested finiteness. The NaN then reached
`markers.sort_by(partial_cmp().unwrap())`. Worse than a dead run: the panic unwound **while the DB
lock was held**, poisoning the mutex, so every later query panicked too and the app was unusable
until restart. Fixed at both ends — the importer drops a non-finite depth row, and the sort no
longer unwraps. An unorderable depth is not a top.

**2 · A percent-entered zone parameter panicked the module run.** `f64::clamp` asserts
`lo <= hi`, and the bounds are themselves parameters: entering irreducible water saturation as
`25` instead of `0.25` produced `limit(swt, 25.0, 1.0)`. The zone override is *designed* to beat
the module dialog, so it also skips the dialog's range check — and the DB Inspector edits
`zone_params.value_num` raw. Now enforced at the one choke point where every path converges
(`workflow::resolve_param_arrays`), using the `min`/`max` the manifest **already declared**.
**Rejected, not clamped** — silently pulling 25 down to the spec maximum would have answered with
a plausible-but-wrong saturation. The error names the parameter, the value, the zone and the valid
range. `modules::limit` was hardened too, as a backstop for future modules.

**3 · An infinity panicked the synthetic-log KNN.** `f32::from_str` returns `inf` for a cell like
`1.0E+40` or the literal token `inf`, and everything downstream screens for missing with
`is_nan()` only — so it survived into the compute cores, where `inf − inf` made the z-score NaN
and `partial_cmp` on two NaNs panicked the neighbour sort. The DLIS importer already stripped
exactly this; the LAS path did not. Fixed at three points, because the verifier found the LAS cell
is *not* the likeliest source: **the Rhai equation engine is** — `1.0/0.0` and `exp(100)` both
reach `inf`, and the existing guard only rejected an *entirely* non-finite column, so a single
infinite sample was written to a computed curve that could then be picked as a predictor. So:
the LAS value path maps non-finite to missing, equation output does the same, and the KNN skips a
non-finite distance instead of sorting on it. The z-score floor also changed from `if *s < 1e-9`
to `if !(*s >= 1e-9)`, because a NaN std slipped straight past the old form.

cargo **367/0/7** — five new tests, each fed the *exact* malformed input from the finding rather
than a synthetic near-miss.

- [ ] **Try (1):** make a tops CSV with a `NaN` in the depth column (or export one from pandas with
  a missing marker), import it, then run **Auto-correlate** from that well. Before: the run died
  with `worker thread failed` and every later action failed until restart. Now: the bad row never
  imports, and correlation runs on the real tops.
- [ ] **Try (2):** Zones → set `SWT_IRR` to `25` for zone `*`, then run **SW-Archie**. Before: opaque
  `worker thread failed`. Now: a message naming `SWT_IRR = 25`, the zone, and the valid range.
  Set it to `0.25` and the run proceeds normally.
- [ ] **Try (3):** run an **equation** like `1/0` or `exp(1000)` into a new curve, then use that curve
  as a predictor in **Synthetic Log (KNN)**. Before: the app aborted. Now: those samples read as
  missing and the prediction runs.

**Not fixed here** (out of R2's scope, still open in F2a): the startup `.expect()` on DB init —
a locked or newer project file kills the process before the window exists, with no dialog, and
`startup_path()` re-selects the same unopenable file every launch.

## Round 58 — R1: the net-flag polygon actually works now (2026-07-24)

**Correction to Round 47.** The flag-polygon feature has never worked — not once, since it shipped
in `a4e05e9`. `NetFlagSpec` was declared in **camelCase** in `ipc.ts` while `netflag.rs` expects
**snake_case**, so `run_net_flag` could not deserialize a single request; `NetFlagResult` had the
same slip in the other direction, so the status line read `undefined`. I marked Round 47 verified
on the strength of a frontend twin-count check that never crossed the wire. That was the gap:
the lasso's live in-polygon count is computed in the browser, so it agreed with itself perfectly
while the backend was rejecting every call.

Found by the F1c pass of the engineering review (`docs/review_sweep/F1.md`), then confirmed by
hand against both files before touching anything.

**The fix went into TypeScript, not Rust.** Struct DTOs cross this wire in snake_case — Tauri
camel-cases only the top-level command *argument* key (`{ spec }`), never the fields inside it,
and `rename_all` is used in this codebase only on enums for their string tag values. Every other
DTO already follows that (LorenzResult, ZoneParamEntry, HighlightEntry). NetFlag's TS was the
outlier, so adding `rename_all` to the Rust would have made it the one struct with a different
wire shape.

Three tests now hold the contract, because a Rust-only serde test cannot see `ipc.ts` — and the
two sides disagreeing *while each was internally consistent* is exactly what happened:

- the spec deserializes from the **literal JSON `crossplotPanel.ts` sends**, and the old camelCase
  shape is asserted to be **rejected** rather than half-parsed into defaults;
- the result serializes under the names the status line reads;
- a **cross-language** test reads the real `src/ipc.ts`, extracts both interfaces, and fails on
  drift. I proved it fires by regressing `ipc.ts` back to the shipped-broken shape and watching it
  fail, then reverting.

`NetFlagSpec` also gained `deny_unknown_fields`, so a TS field Rust doesn't know now fails loudly
instead of being silently dropped — the silent direction of the same class.

cargo **362/0/7**, tsc + build clean.

- [ ] **Try:** open a Crossplot on a well with PHIE/RHOB, draw a lasso polygon around the clean-sand
  cloud, **Write Net Flag** with a name like `NET_TEST`. Before this fix the button did nothing and
  the status line said `Net flag failed: …`. It should now report
  `Net flag NET_TEST: <n> / <m> samples net (<k> written)` with a real curve name, and `NET_TEST`
  should appear in the Curve Catalog and plot as a 0/1 track in the log view. Check the count
  against the lasso's own live in-polygon readout — those two numbers agreeing is the thing that
  was never actually tested.

## Round 57 — SandiMin: RMS vs core (2026-07-24)

Closes the second first-half residual (playbook #2). RECON/incoherence only says the model
reproduces **its own input logs** — it cannot catch endpoints that are wrong in a way those logs
can't see. Core plugs are an *independent* measurement, so a run on a cored well now also reports
how far the solution sits from them.

Three numbers per cored well, each an **RMS of (model − core)** with a signed **bias** (so the sign
says which way the model reads) and the plug count:

- **Core φ vs PHIE** *and* **vs PHIT** — both, because which one a plug should match depends on the
  drying protocol (oven-dried drives off clay-bound water → PHIT; humidity-dried retains some →
  nearer PHIE). Showing the bracket is more honest than picking one for you.
- **Core ρg** — the grain density implied by the solved **solid** volumes (Σv·ρ / Σv over the
  non-fluid components). This is the one that tests the **mineral model** specifically: bound water
  is a fluid here, so it correctly sits outside the sum, matching a cleaned-and-dried plug. Where
  RHOB was not itself an input tool this is a fully independent check.

Plugs tie to the nearest **solved** sample within 1 m (the same tie-in tolerance already used for
core elsewhere); an unsolved sample is skipped rather than matched. A well with no core — or an
all-null column — shows nothing at all, never a 0.000 that would read as a perfect match. Plugs
outside a physically valid range are dropped rather than fitted, so a φ column imported in **percent**
reports "no fit" instead of a confident-looking RMS of ~14.85.

**Try:** open **SandiMin** on a well that has **core** loaded, set up your usual mineral model, and
**Run**. A new **Core calibration** block appears under the results table. (1) Check the **plug
count** is roughly what you'd expect over the solved interval. (2) Look at **Core ρg** — on a sound
quartz/clay model it should sit within a few hundredths of a g/cc; a large bias here points at a
matrix-density endpoint rather than at the logs. (3) Compare **vs PHIE** against **vs PHIT** — your
plugs should sit nearer whichever matches how they were dried; a big gap on *both* is worth a look
at the clay-bound-water setup. (4) Run a well with **no core** and confirm the block is absent
entirely. (Verified: cargo **359/0/7** including two new tests — hand-computed RMS/bias literals
covering the depth tolerance and the NaN-skip, and a full run asserting the fits appear only for the
cored well, with a percent-φ and a 999.25-ρg plug planted *inside* depth tolerance to prove the value
gate rejects them rather than the depth gate; tsc + production build clean.)

## Round 56 — Monte Carlo: per-parameter uncertainty widths seeded from IP (2026-07-24)

Closes a first-half residual (playbook #1). Until now, adding an uncertain parameter gave it a
**generic** width — 10% of its own value as the normal σ, 20% as the uniform/triangular half-range.
That is wrong whenever the parameter isn't naturally relative: **RHO_MA = 2.645** was getting
σ = **0.26 g/cc** (a ±10% matrix density — about **9× too wide** against the ±0.03 convention), and
**GR_SH = 120** was getting σ = 12 API where the field convention is ±10.

Defaults now come from a table imported from IP's `MonteCarloDefaults.par` (Tier-A), so each
parameter gets a width **in its own units**: `M`/`N` ±0.2, `A` ±0.1, `GR_MA`/`GR_SH` ±10 API,
`RHO_MA` ±0.03, `RHO_FL` ±0.02, `RHO_SH` ±0.05, `RHO_DSH` ±0.1, `NPHI_SH` ±0.05, and the two
resistivities `RW`/`RT_SH` as **±20% of their value** (they *are* naturally relative). A muted **IP**
badge on the row marks a seeded width — hover it for the source. Anything unseeded (`C`, `SWE_IRR`,
`PHIE_MAX`, …) keeps the old generic width exactly as before. Widths stay fully editable — this only
changes what a **freshly added** row starts at.

Provenance, the mapping table, and the σ reading adopted (the tabulated shift is taken as **one
standard deviation**; IP's file doesn't state its percentile convention, so this is SandiBumi's
documented choice, not a claim of matching IP run-for-run) are banked in
`docs/ref_monte_carlo_seeds.md`.

**Try:** open **Monte Carlo** on the default chain (VSH → Porosity → SW-Indo). (1) **+ Add uncertain
parameter** and pick **RHO_MA** → confirm the **std dev** reads **0.03** (not 0.26) and an **IP**
badge sits on the row; hover the badge for the source. (2) Add **M** → σ **0.2**; add **GR_SH** → σ
**10**; add **RW** → σ **0.02** (= 20% of 0.1). (3) Switch one of them to **triangular** → confirm
min/mode/max straddle the value by that same width (M → 1.8 / 2.0 / 2.2) and the sparkline redraws.
(4) Pick a parameter with **no** badge (e.g. **SWE_IRR**) → confirm it still gets the old generic
width, and that its fields stay column-aligned with the badged rows. (5) Run it and confirm the
tornado still reads sensibly — the point of the change is that the P10/P90 spread is now built on
priors with the right units. (Verified: tsc + production build clean, MC dialog still a lazy chunk,
main bundle unchanged; a headless check evaluates the real source's seed table and width maths — 36
assertions, all pass — including that every unseeded parameter's fallback is byte-identical to the
old behaviour and that a % seed on a zero value degrades to the floor instead of collapsing the row
to a point mass. No Rust changed.)

## Round 55 — Vega-Lite interactive charts, V5: density + trend overlay (2026-07-24)

Builds on V4 (Round 54). The capstone adds two analytical modes:

- **Density** — a new **chart type**: a 2D binned heatmap (viridis by bin count). This is the view
  for clouds too dense to read as a scatter — a Mahakam NPHI–RHOB cloud overplots into a blob, but
  the binned counts show where the mass actually is. Hover a cell for its bin range and count. (Like
  the histogram it's an aggregate, so it doesn't take part in brushing.)
- **Trend** — a regression overlay on the **Scatter**: tick **Trend** to draw a fit line plus its
  **R²**, with a method dropdown (**linear / log / exp / pow / quad**). It layers over the point
  cloud, so hover / brush / zoom on the points still work; log/exp/pow assume positive data. Works
  alongside a Colour curve and a Zone.

**Try:** open a **Vega Chart**. (1) Set **Type = Density** on a dense NPHI–RHOB pair → confirm a
viridis heatmap of counts, and hover a cell for its bin + count. (2) Back on **Scatter**, tick
**Trend** → confirm a fit line + an "R² = …" label appear; change the **method** (e.g. **log**, the
por–perm shape) and confirm the line + R² update. (3) With Trend on, set a **Colour** curve and a
**Zone** and confirm all three coexist. (Verified: tsc + offline build keep vega + CodeMirror lazy,
main bundle unchanged; a headless check renders the density spec and every trend method — each shows
an R² label, keeps the brush signals, and still dims on a shared brush through the layering. The
headless pass caught two real layered-spec bugs — a duplicate `grid_x` signal and an unresolved
`brushedActive` — now fixed by splitting the params across the layer. The live density hover and the
trend line/R² on field data are what this Try line confirms.)

## Round 54 — Vega-Lite interactive charts, V4: export + spec editor (2026-07-24)

Builds on V3 (Round 53). The Vega Chart panel becomes a report/export surface and gains an escape
hatch for power users:

- **Export.** New toolbar buttons: **⧉ Copy** (PNG to clipboard), **⭳ Image** (save PNG), **⭳ SVG**
  (a true-vector SVG from vega's own renderer), **⎙ Print**. Same affordance as the crossplot /
  histogram export.
- **Spec editor.** **⧉ Spec** reveals a JSON editor showing the *effective* Vega-Lite spec (with the
  data rows elided). Edit the grammar — point size, a title, an extra layer, a scale — and **Apply**;
  the chart re-renders with your override and the current rows are re-injected, so the control bar
  still drives which curves / zone fill it. **Reset** returns to the generated spec. Changing the
  **chart type** clears an override (the grammar is type-specific). Invalid JSON is reported inline;
  an invalid spec shows "render failed" rather than a broken chart. (Linked brushing keeps working
  through an override.)
- **Opens where you left off.** The control selections (type / curves / zone) are remembered, so a
  new Vega chart opens with your last settings.

**Try:** open a **Vega Chart**. (1) **⭳ Image** and **⭳ SVG** — save each and open the files; **⎙
Print**. (2) Click **⧉ Spec**, change something in the JSON (e.g. `"size": 20` → `120`, or add
`"title": "My chart"`), **Apply** → confirm the chart changes; **Reset** → confirm it reverts. Type
some invalid JSON and confirm the inline error. (3) Set Type = Line + a Zone, close the panel, open a
new Vega chart → confirm it opens as Line on that zone. (Verified: tsc + offline build keep vega **and
CodeMirror** as separate lazy chunks — the editor only loads when you open Spec; a headless check
confirms the spec round-trips through the editor, still renders, and keeps its brush signals, and that
an invalid override throws. The live save dialogs, printing and editor typing are what this Try line
confirms.)

## Round 53 — Vega-Lite interactive charts, V3: theme repaint + linked brushing (2026-07-24)

Builds on V2 (Round 52). The Vega Chart panel now joins the rest of the workspace — it repaints with
the theme and takes part in the shared brush:

- **Live theme repaint.** Switch the theme with a Vega chart open and it repaints in the new palette
  immediately (it re-embeds from the cached data, so no re-fetch). One deliberate trade: a theme
  switch resets the chart's zoom/pan back to full extent.
- **Brush → other panels.** On a **Scatter** or **Line**, **drag a box** over the points; the samples
  inside are published as the shared selection, so the crossplot, histogram and log view of the same
  well highlight the *same* depths (live, as you drag). A click on empty space (or a zero-size box)
  clears it.
- **Other panels → Vega.** When you brush in a crossplot (or any panel that publishes a selection),
  the Vega **scatter dims the un-selected points** so the shared samples stand out. (A line is one
  path, so it only emits; a histogram takes part in neither — its bars are aggregates.)
- **Gestures.** Because plain-drag now *brushes*, **pan moved to Shift-drag** and **zoom stays on the
  wheel**. Hover tooltips are unchanged.

**Try:** open a **Vega Chart** (Scatter, e.g. NPHI–RHOB). (1) Switch the theme (ribbon) and confirm
the chart repaints in the new colours. (2) **Drag a box** over a cluster and confirm a crossplot /
histogram / log view of the same well lights up the same samples. (3) Brush in a **crossplot** and
confirm the Vega scatter dims everything except those samples. (4) **Shift-drag** to pan and **scroll**
to zoom. (Verified: tsc + offline build keep vega a separate lazy chunk; a headless vega-lite→vega
compile+render check confirms the brush/pan event selectors and the array-form opacity condition are
valid and that driving the consume signals dims the right points — 2 bright / 238 dimmed. The live
drag/pan gestures are what this Try line confirms — the harness can't drive vega's pointer input.)

## Round 52 — Vega-Lite interactive charts, V2: control bar (type / colour / zone) (2026-07-24)

Builds on V1 (Round 51). The Vega Chart panel gains a real control bar so you can shape the plot
without leaving it:

- **Chart type** — **Scatter / Line / Histogram**. Scatter is the X–Y cloud; Line connects the
  samples in depth order (a trajectory through crossplot space); Histogram is the X curve's
  distribution (binned count).
- **Colour curve** (scatter only) — colour the points by a third curve on a **viridis** scale with a
  legend (e.g. NPHI–RHOB coloured by GR). "— None —" falls back to the theme accent.
- **Zone filter** — restrict the plot to a named zone's depth range (follows the top-interval like
  the other plots); "all" plots the whole well.
- Controls that don't apply to the active type dim out (Y on a histogram, Colour off scatter), so the
  bar reads honestly. Selections carry across a well switch.

**Try:** Plot ribbon → **Vega Chart**. Switch **Type** to Histogram (the X curve's distribution) and
to Line; on Scatter, set **Colour** to a curve (e.g. GR) and confirm the points take a viridis ramp
with a legend; set a **Zone** and confirm the plot restricts to it (status line shows the zone). Pan /
zoom / hover still work on scatter and line. (Verified: tsc + offline build; all three chart types
render non-blank canvases with a clean console on the dev server — the small-canvas note is just the
uncomposited preview pane, not the app.)

## Round 51 — Vega-Lite interactive charts, V1: engine vendored + one live chart (2026-07-24)

New feature (your "Altair on SandiBumi" ask, built as *interactive Vega-Lite in-app*): a chart
rendered by the real **vega** engine, vendored **offline** into the app. V1 lands the engine + one
live chart; richer controls, theme-repaint, brush-linking and a spec editor are V2–V4.

- **New "Vega Chart" button** on the Plot ribbon (next to Crossplot). It opens a well-bound panel:
  pick an X and a Y curve (defaults NPHI / RHOB) and it plots the selected well, following the Wells
  pane like the other plots. Hover for a tooltip; drag to pan; scroll to zoom — vega's built-in
  grammar-of-graphics interactivity, the thing the Canvas-2D plots don't give for free.
- **Offline + lazy.** `vega` / `vega-lite` / `vega-embed` are bundled into the app (no CDN, works with
  no network). The engine is ~850 KB, so it is a **lazy** chunk — it loads only the first time you
  open a Vega chart and stays out of the main startup bundle.
- **Themed** from the active theme's CSS vars (axes, grid, points), so it matches the brand themes.
  Colours are read when the chart builds; live repaint on a mid-session theme switch is V3.

**Try:** Plot ribbon → **Vega Chart** (select a well first). Confirm the scatter draws in your theme's
colours, then **hover a point** (tooltip = X / Y / Depth), **drag** to pan, and **scroll** to zoom.
Switch the X / Y curves and confirm it redraws. (I verified the render, theming and offline bundle
with a screenshot against synthetic data; the live pan / zoom / tooltip is what this Try line
confirms — the automated harness couldn't drive vega's canvas input.)

Note: `npm audit` flags 7 high-severity advisories in vega's dependency tree. I did **not** auto-fix
(it wants breaking changes). For an offline desktop app rendering local numeric data the exposure is
minimal, but say the word if you want me to look at pinning/patching them.

## Round 50 — Monte Carlo: per-row PDF preview sparkline (2026-07-24)

Playbook **#1 (Monte Carlo)** residual: *"per-row live distribution (PDF) preview."* Each uncertain-
parameter row in the Monte Carlo dialog now carries a small inline **sparkline of the distribution
shape** you configured — a bell for Normal (mean/sd), a flat-topped box for Uniform (min/max), a
triangle for Triangular (min/mode/max). It redraws **live as you type**, so you can see the shape
before running anything.

- Purely a preview — it reads the row's own `(kind, a, b, c)` and **never feeds the sampler**, so the
  P10/P50/P90 are untouched. Colours come from the theme (`--accent`/`--border`), so it repaints with
  the brand themes like the rest of the UI.
- Collapsed spreads don't go blank: `sd≤0`, `min==max`, or a NaN field renders a narrow **point-mass
  spike** (a delta). Swapped bounds (min>max) auto-normalize; a Triangular mode outside [min,max]
  clamps to the nearest edge — the preview always shows a sensible shape.
- Verified: `tsc` clean + **15/15 geometry assertions** on the exact path function (bell apex centred
  at the peak, box edges at the right fractions, triangle apex at the right x, every degenerate case →
  spike). The in-app pixel look is what this Try line is for.

**Try:** open **Monte Carlo**, add an uncertain parameter, and watch the little chart beside the
number fields. Switch the kind (normal → uniform → triangular) and edit mean/sd (or min/mode/max): the
sparkline should update as you type — a bell that narrows as you shrink sd, a box that widens with the
range, a triangle whose peak slides with the mode. Set sd to 0 (or min=max) and it should collapse to a
thin spike.

## Round 49 — Monte Carlo: physical-plausibility guard (impossible Sw>1 / PHIE<0 fraction) (2026-07-24)

Playbook **#1 (Monte Carlo)** residual: *"reject/flag impossible combos (Sw>1, PHIE<0) and report the
rejected fraction."* The MC engine now reports, per well, **how often a sampled parameter combination
drove the petrophysics out of physical bounds** — a QC signal that your input distributions may be too
wide.

- The trick: the chain's saturation/porosity modules **clamp** the final `PHIE`≥0 and `SWE`≤1, so the
  impossible values never reach the limited curves. But every one of them also emits an **unlimited
  companion** (`PHIE_DN`, `SWT_ARCH`, `SWE_INDO`, …) where the raw `Sw>1` / `PHIE<0` survives. The
  guard scans those (spec-driven: any produced `v/v` curve named `PHI*`/`SW*`), per realization, over
  the in-zone samples, and counts the ones outside `[0,1]`.
- **Reported, never excluded.** The module clamp already gives an impossible draw the physically-correct
  volumetric answer (an over-dense matrix → zero effective porosity; a supersaturated combo → fully
  wet), so those realizations are **valid low/high tails** — dropping them would bias P10/P90. So the
  headline percentiles are **unchanged**; you just get a new advisory line. A large fraction means
  "narrow your inputs," not "the result is wrong."
- The MC dialog's notes area gains one line per well: **⚠** with the fraction + a `Sw>1` / `PHIE<0`
  breakdown when impossible draws occurred, **✓** when every realization stayed in bounds, and a neutral
  **•** "not checked" when a well had no porosity/saturation to judge (never a fabricated clean pass).

**Verification:** three new `montecarlo.rs` unit tests — matrix density pinned below RHOB → `PHIE_DN<0`
flagged on 100% of realizations; cementation exponent pinned high → Indonesia `Sw>1` flagged (porosity
stays clean); a normal clean-sand study → 0% impossible. The headline HPV still computes in every case,
and the pre-existing reproducibility tests still pass **byte-identical** (the guard is purely
observational — it never touches the RNG or the reported percentiles). Full lib suite **357/0/7**, `tsc`
clean.

**Try:** open **Monte Carlo**, set up any run (e.g. vary `RW` or `M` with a wide spread on a real well),
and run it. Look at the notes area under the results: you should see a **✓ … within physical bounds** on
a well-behaved study. Now widen a distribution aggressively (e.g. `RW` normal with a big σ, or `M` up to
4) and re-run — the line should flip to **⚠ … % of realizations hit impossible petrophysics (Sw>1 …)**,
while the P10/P50/P90 HPV stay sensible. Tell me if the fraction looks off for what you dialed in.

## Round 48 — UI polish #9C follow-on: free-form net-flag polygon on the crossplot (2026-07-23)

The crossplot's scalar cutoff-box (Round 45) is now joined by a **free-form net-reservoir polygon**:
draw an arbitrary shape around a cloud of points and write its interior straight to a **discrete 0/1
net-flag curve** — the general case the rectangular cutoff can't express (e.g. a curved φ-k fairway, an
L-shaped sand window).

- A new **⬡ Net polygon** toolbar toggle enters draw mode: click to drop vertices, and a small bar
  shows **Undo point / Clear / Write net flag…** with a live `N / total points inside` readout. The
  polygon fills faintly, its edges + a dashed closing edge + a rubber-band to the cursor draw as you
  go, and — because vertices are captured in **data space** — it stays registered under zoom/pan.
- **Write net flag…** names the curve (default `NET_FLAG`) and calls the backend, which computes the
  flag over the crossplot's current depth window and writes it as a computed curve like any module
  output: **1** inside / **0** outside / **NaN** where a sample can't be evaluated (either input NaN,
  or ≤ 0 on a log axis — the same samples the crossplot excludes). Other views refresh so the new
  curve shows up.
- **`netflag.rs`** does the work: even-odd point-in-polygon run in the axes' **drawing plane** (log10
  on a log axis), so "inside the drawn polygon" is exact for log scales (straight screen edges are
  straight edges there) and matches the on-screen count. The frontend's live count uses an **exported
  twin** (`netPolygonContains`) of that same test, so the preview equals what gets written.

**Verification:** `netflag.rs` has 5 unit tests — concave (notched-square) point-in-polygon, a written
0/1/NaN curve over a synthetic cloud, the depth-window restriction, the ≥3-points / distinct-axes
guards, and a **log-axis** case (a decade box on a log X axis captures exactly the right samples and
rejects a ≤ 0 vertex). In-browser, the frontend `netPolygonContains` was checked against the *same*
cases and agrees with the backend on every one — linear, concave, and log. Adversarial review caught
one interaction bug (a double-click while drawing dropped two vertices **and** opened the Properties
dialog), now guarded. `tsc` + full lib suite green.

**Try:** open a **Crossplot** (e.g. a φ-k or NPHI-RHOB cloud), click **⬡ Net polygon**, and click
around the group of points you consider net; watch the inside-count update. Click **Write net flag…**,
name it (say `NET_POLY`), and Write — then add that curve to a **log view** track and confirm it reads
1 exactly where your polygon was, 0 elsewhere. Re-draw a different shape to overwrite.

## Round 47 — UI polish: true-vector PDF export for the Canvas-2D plots (2026-07-23)

The vector story is now complete: the **crossplot, histogram, and Pickett** plots also export a
**true-vector single-page PDF** — a portable, self-contained figure to drop straight into a Word/LaTeX
report — via a new **⭳ PDF** button in each plot's toolbar (and an "Export PDF (vector)…" right-click
entry), sitting alongside the ⭳ SVG button from Round 46.

- **`pdfExport.ts` — `PdfRecorder`**: the sibling of `SvgRecorder`. It drives the **same**
  `drawCrossplot` / `drawHistogram` / `drawPickett` code through a recording 2D context, but serialises
  every call into a **PDF content stream** (operators in points, bottom-left origin) instead of SVG — so
  again **no chart is re-implemented** and the PDF can't drift from the screen. Handles the full surface
  the plots use: affine transforms (rotated axis labels via the PDF text matrix), rectangular clips
  (`q … re W n … Q`), circles (as béziers), dashes, text alignment/baseline, and all the colour forms
  the plots emit (`#hex`, `rgb()`, `hsl()`).
- **Split of concerns**: the frontend owns only the *drawing operators*; the backend
  (`save_plot_pdf` → `composite::assemble_single_page_pdf`) wraps them in the PDF *document*
  (catalog, xref, Helvetica fonts) — reusing the exact, already-tested assembler that powers the
  composite-log PDF, so the fiddly document scaffolding lives in one place.
- Text renders in base-14 Helvetica (no font embedding, same as the composite PDF) and transparency is
  flattened against the plot background — *exact* for these plots, which only use alpha for gridlines /
  marginals drawn straight over that background. (The SVG export remains the fully device-independent
  option; the PDF is the portable single-file one.)

**Verification:** the new `assemble_single_page_pdf` has a Rust unit test (valid `%PDF`, one Page,
MediaBox at the requested point size, stream embedded); the full lib suite stays green (356 pass).
In-browser, against the real `PlotCanvas` draw methods (log X + inverted Y, every colour form): the
content stream has balanced `q`/`Q` and `BT`/`ET`, béziers/clips/dashes/text present, **no
NaN/Infinity**, and every colour operand in [0,1]; the text matrix was checked exactly for the
identity and the rotated-y-label cases. Adversarial review caught one fidelity slip (a forced round
cap/join where canvas/SVG use butt/miter), now fixed. `tsc` clean.

**Try:** open a **Crossplot / Histogram / Pickett**, arrange it how you like, then click **⭳ PDF** and
save. Open the `.pdf` in a viewer and zoom right in — text and curves stay razor-sharp — then drop it
into a report to confirm it embeds cleanly. Compare against **⭳ SVG** for the same chart: same figure,
two portable vector formats.

## Round 46 — UI polish #9B: true-vector SVG export for the Canvas-2D plots (2026-07-23)

The **crossplot, histogram, and Pickett** plots previously exported raster PNG only (the log
composite already had a vector path). They now export a **true-vector SVG** — infinitely scalable,
editable in Illustrator/Inkscape/PowerPoint — via a new **⭳ SVG** button in each plot's toolbar
(and an "Export SVG (vector)…" right-click entry).

- **`svgExport.ts` — `SvgRecorder`**: a recording 2D context that duck-types
  `CanvasRenderingContext2D` and serialises every draw call to SVG. A detached canvas carries the
  recorder via a private property that `PlotCanvas` reads, so the **same** `drawCrossplot` /
  `drawHistogram` / `drawPickett` code paints into the recorder — **no chart is re-implemented**,
  so the SVG can't drift from what's on screen. Handles the full surface the plots use: affine
  transforms (rotated axis labels), rectangular clips (incl. nesting), circles, dashed lines,
  alpha, text alignment + baseline, and the colorbar/marginal/regression overlays.
- The export re-runs each panel's **static** draw only (a shared `drawStatic` in the crossplot),
  so transient decorations — hover ring, brush highlight, cutoff shading, parameter handle — are
  omitted: you get the clean, publishable chart. Written to disk as UTF-8 through the existing
  save path (no backend change).

**Verification (in-browser, against the real draw code):** SVGs from all three panels parse as
valid XML (DOMParser), with the correct element counts (e.g. 249 points for a 250-pt cloud with one
NaN, 59 bars for a histogram), balanced/nested clip groups, correct affine composition
(translate∘rotate → exact matrix + mapped points), and no NaN/undefined/Infinity tokens — exercised
with marginals + regression + a viridis colorbar and with log axes. Adversarial review confirmed the
wired panels correct on all fronts and caught one forward-looking gap (a dropped `textBaseline`),
now fixed and re-verified non-regressive. `tsc` clean; no Rust changes.

**Try:** open a **Crossplot / Histogram / Pickett**, arrange it how you like (zoom, colorby, picks),
then click **⭳ SVG** in the toolbar and save. Open the `.svg` in a browser or vector editor and zoom
in — the text and curves stay razor-sharp (unlike the PNG). PDF-for-charts is the natural next step.

## Round 45 — UI polish #9C follow-ons: Pickett brush-rings + crossplot cutoff region (2026-07-23)

Two interaction upgrades that build on Round 43's linked brushing and the crossplot's draggable
parameter handle.

- **Pickett brush-rings** — the **Pickett** plot is now a brushing *consumer*: samples you Shift+drag on
  a **crossplot** of the same well are ringed (accent-2) on the Pickett log-log, so a selection made in
  one plot is visible in the other. Depths match bit-exactly off the shared backend grid; rings are
  clipped to the plot and skip log-invalid points.
- **Crossplot cutoff region** — a new **"Net cutoff"** dropdown next to the pick rows turns the draggable
  parameter handle into a pair of cutoffs. Pick a *net side* (X ≥/≤ pick, Y ≥/≤ pick) and the crossplot
  draws the two cutoff threshold lines through the handle, **shades the net quadrant**, and reads out how
  many plotted points fall inside it (`net cutoff: N / tot pts (P%)`). The sense is chosen explicitly —
  no cutoff direction is inferred from the axes — and the quadrant maps data→pixels through the axis
  extents, so it stays correct under log / inverted axes. Default **off** (unchanged appearance).
  Dragging the handle still writes the two zone parameters as before; the net side persists in plotprops.

**Verification:** the cutoff quadrant→pixel mapping and the 4-sense point-count were unit-tested against
the real `PlotCanvas.toPx` (counts + NaN exclusion exact for all four senses; correct side under linear
and inverted-Y axes). Adversarial review caught and fixed two bugs: a template-apply path that left the
Net-cutoff dropdown out of sync with `opts.netSense`, and an uncancelled hover `requestAnimationFrame` on
Pickett dispose. `tsc` clean.

**Try:** open a **Crossplot** and a **Pickett** of the same well side by side; **Shift+drag** a box on the
crossplot — the same samples ring on the Pickett. Then on the crossplot pick a **Net cutoff** side from the
new dropdown, drag the ringed handle around the cloud, and watch the shaded net box + the live
`net cutoff: N / tot pts (P%)` readout follow.

## Round 44 — UI polish #9D: accessibility & motion (2026-07-23)

The plot canvases were unlabelled and unfocusable, and transitions ignored the OS "reduce motion"
setting. Both fixed, via two shared helpers plus one CSS media query.

- **`makeCanvasAccessible(canvas, label)`** (plotCanvas.ts) — sets `role="img"`, an `aria-label`, and
  `tabindex=0`. The **crossplot / histogram / Pickett** canvases now announce themselves to screen
  readers with a live description (e.g. "Crossplot: RHOB versus NPHI, coloured by GR", "Histogram of
  PHIE", "Pickett plot: RES_DEEP versus PHIE") that updates as the plotted curves change.
- **`attachKeyboardPanZoom({canvas, getPlot, view, redraw, axes})`** (plotCanvas.ts) — a focused plot
  canvas now takes **arrow keys** to pan (Shift = bigger step), **+/−** to zoom around centre, and
  **0/Home** to reset, driving the same `ViewportRef` as the mouse (log-safe, `axes:"x"` on histograms).
  Wired into all three panels; only handled keys are consumed so Tab/Enter still work.
- **`.plot-canvas:focus-visible`** — an accent focus ring so keyboard focus is visible.
- **`@media (prefers-reduced-motion: reduce)`** — neutralises every transition/animation (the 5 CSS
  transitions the survey found: form inputs, `.btn`, mm-chevron, proc-bar, health-bar) for users who
  opt out of motion.

**Verification (in-browser):** `makeCanvasAccessible` → `role=img`, `aria-label="Test chart"`,
`tabindex=0`; `attachKeyboardPanZoom` → ArrowRight panned the viewport (xMin 0→0.8), `+` zoomed in
(width 10→8.3), `0` reset to auto, and the disposer stopped handling; both the reduced-motion media
rule and the `.plot-canvas:focus-visible` rule are live in the stylesheet. `tsc` clean.

**Try:** click a **Crossplot / Histogram / Pickett** plot to focus it (an accent ring appears), then
use **arrow keys** to pan, **+/−** to zoom, **0** to reset — no mouse needed. A screen reader now reads
the chart's axes. Turn on the OS "reduce motion" setting and UI transitions stop animating.

## Round 43 — UI polish #9C: linked brushing (crossplot → log view + histogram) (2026-07-23)

Rectangular **Shift+drag** on a **crossplot** selects a cloud of samples; every plot and log view of
the same well highlights those same samples. A new `appState.brushedDepths` observable
(`{wellId, depths:Set<number>}`) carries the selection; membership is an exact `Set.has` on the shared
well depth grid (all a well's curves come off the same backend f32 grid — verified in the adversarial
review against the Rust `fetch_curve_data`).

- **Crossplot (source + consumer):** Shift+drag draws a selection rectangle (accent2, dashed); on
  release the samples inside are published, and the brushed points are drawn emphasised. A tiny
  rectangle clears the selection. The gesture takes precedence over pan/param-handle/pick — it
  `stopImmediatePropagation()`s so `attachZoomPan` never pans, and marks `movedSinceDown` so the
  trailing click doesn't drop a parameter pick.
- **Histogram (consumer):** the brushed samples' values are over-painted as an accent2
  **sub-distribution** in the same bins — you see where the brushed cloud falls in any property.
- **Log view (consumer):** `HighlightsOverlay.setBrush` paints the brushed depths as thin accent
  **ticks** across every track, redrawn each frame; a well switch re-applies (gen-guarded) so the
  previous well's ticks never linger.

**Adversarial review (subagent):** cleared event-coexistence, the exact-float grid match (checked
against the backend), lifecycle/teardown, published-set correctness, and NaN/null safety. Two real
issues found and **fixed**: (1) the log-view brush re-apply in `loadWell` wasn't gen-guarded — a fast
well-switch could wipe the winning load's ticks; (2) `rafId` wasn't cancelled on dispose in the
crossplot/histogram. Both patched.

**Verification (in-browser):** state plumbing (`setBrushedDepths` → `W1:3` → `clearBrush` → null →
empty-set → null); `drawHistogram(brushValues)` over-painted the sub-distribution (12.1 k changed
pixels); `HighlightsOverlay.setBrush([4 depths])` painted 1400 tick pixels, `setBrush([])` → 0. `tsc`
clean.

**Deferred (9C follow-on):** Pickett rings on brushed samples (same pattern, cheap) and the draggable
cutoff *polygon* → zone params (the crossplot already has a draggable param **handle** that writes
cutoffs; a full lasso/polygon is a separate feature).

**Try:** open a **Crossplot** and a **Log view** of the same well side by side. **Shift+drag** a box
around a cluster on the crossplot — the log view lights up **ticks** at those depths, and if you have a
**Histogram** of PHIE/SWE open, the selected samples show as a highlighted **sub-distribution**. Drag a
tiny box (or Shift-click) to clear.

## Round 42 — UI polish #9B inc 1: shared colour-bar + scatter hover tooltip (2026-07-23)

Visualization richness, starting with two shared primitives in `plotCanvas.ts` so every chart gets the
same treatment instead of a bespoke copy:

- **`drawColorbar(plot, {map, lo, hi, label, log})`** — the continuous Z colour-bar, extracted from
  its one bespoke copy inside `drawCrossplot`. The crossplot now calls it; Pickett/HFU can adopt it in
  one line. Same look, one place to theme.
- **`attachScatterTooltip(canvas, hit)`** — a hover **tooltip bubble** showing the sample under the
  cursor. `hit(px, py)` returns the lines to show (or null to hide); the bubble is a
  `pointer-events:none` node positioned by the cursor and clamped to the viewport, so it never steals
  the canvas's own mouse events. `fmtValue(v)` gives compact 4-sig-fig labels.
- Wired into the **crossplot** (depth + X/Y/Z values, suppressed while dragging a handle) and **Pickett**
  (depth + Rt + porosity, suppressed while panning/picking). New `.plot-tooltip` CSS, all theme vars.

**Still open in 9B:** true **vector SVG/PDF export at print scale** for the Canvas-2D charts. Today only
the log *composite* has a vector path (`export_composite_svg/pdf` via `composite.rs`); the crossplot /
histogram / Pickett charts export raster PNG only. A real vector route needs an SVG-emitting renderer or
a new Rust command — a sizeable increment on its own, flagged for a scoping call rather than rushed.

**Verification (in-browser):** `fmtValue` → `["0.1823","2.5","1.235e+4","1.23e-4","—","0"]`; `drawCrossplot`
with a continuous Z rendered the scatter + colour-bar (87.8 k coloured pixels, non-null plot);
`attachScatterTooltip` showed the bubble (`display:block`, correct text, `pointer-events:none`), hid on
`mouseleave`, and removed its node on dispose. `tsc` clean.

**Try:** open a **Crossplot** (NPHI–RHOB coloured by GR) and hover the cloud — a bubble now shows that
sample's **depth, NPHI, RHOB and GR**. The Z **colour-bar** top-right is unchanged (now shared code).
Open a **Pickett** plot and hover — depth + Rt + porosity. Dragging the parameter handle (crossplot) or
panning suppresses the bubble so it doesn't fight the gesture.

## Round 41 — Results-QC #8 inc 4: recon / MC / cutoff rollup rows (2026-07-23)

The scorecard now reads as **one verdict per zone** — the two on-open checks (Sw-method spread, Buckles)
plus three rollup rows that **aggregate the sibling analyses** so you don't have to open three panels:

- **Recon incoherence** — mean/max of the SandiMin `*_RECON` curve (Quanti.Elan incoherence, σ units)
  over the zone, with the fraction of samples >2σ. Green ≤1σ, amber ≤2σ, red beyond — *do the solved
  volumes rebuild the logs?* Picks the most-recently-written `*_RECON` on the well; read-only.
- **MC uncertainty** — mean P50 and the mean **LOW–HIGH band** of the persisted `MC_<curve>_LOW/_P50/_HIGH`
  curves (PHIE, else SWE/VSH), as a fraction of |P50|. Green ≤15 %, amber ≤35 %, red beyond — *how wide
  is the input-uncertainty envelope?* Read-only.
- **Cutoff sensitivity** — a **live** `run_cutoff_sweep` nudging the PHIE≥ cutoff ±0.02 v/v around its
  operating value (VSH≤ / SWE≤ held), reporting the fractional net-pay move. Green ≤15 %, amber ≤40 %,
  red beyond — *is net pay robust to the cutoff, or does a small change move the number?*

Each row degrades to a **grey "na — run X first"** when its source curves are absent (SandiMin recon-QC
or Monte-Carlo persist not yet run) — never a silent pass. New operating-cutoff inputs (**VSH≤ / PHIE≥ /
SWE≤**, defaults 0.50 / 0.08 / 0.50) sit beside the Sw params; the user confirms them, nothing is
fabricated. CSV gains 12 columns (recon mean/max σ, %>2σ; MC P50/band/rel; cutoff net/sens/peak).

**Verification (in-browser, mocked IPC):** two zones — a shaly SAND-A and a clean SAND-B. Full scenario:
SAND-A flags Recon (2.20σ, 73 % >2σ), MC (band 56 %), Cutoff (±87 % net) all red; SAND-B all green
(0.50σ, 13 %, ±2 %); status line counts 5 flags. Bare scenario (no recon/MC curves): both rows show
"run … first" (na) while the live cutoff row still fires. CSV header + rows confirmed with the new
columns. Guard added: net pay ≤0 at the operating cutoff → na (no "±Infinity %"). `tsc` clean.

**Try:** open **Results QC** on a well where you've run **SandiMin (Reconstruction QC on)** and **Monte
Carlo (Persist curves on)**. Each zone card now shows five rows — the new **Recon incoherence**, **MC
uncertainty**, and **Cutoff sensitivity** lights. Hover any row for the full explanation. On a well where
you *haven't* run those, the recon/MC rows read "run … first" — run them, hit **Recompute**, and watch
the lights populate. Tweak **PHIE≥** and Recompute to see the cutoff-sensitivity light move. **⭳ CSV**
now carries the recon/MC/cutoff columns.

## Round 40 — Results-QC #8 inc 3: Sw-envelope track + Buckles crossplot + CSV (2026-07-23)

The visual payoff for the scorecard — a **detail view** under the cards, plus CSV export. All frontend,
reusing the per-zone data the scorecard already computed (cached, no refetch).

- **Sw-method envelope track** (`PlotCanvas`) — depth (Y, inverted) vs Sw (X): a shaded min/max **band**
  with one line per model (stable colour per model, Archie first), and a dashed **depth marker** that
  tracks `appState.hoverDepth`. This is where a wide fresh-water-sand spread is read at a glance.
- **Buckles crossplot** (`PlotCanvas`) — Sw (X) vs PHIE (Y): the zone's SWE·PHIE samples over dashed
  **constant-BVW hyperbolae** (0.02–0.10), so an irreducible leg lines up on one hyperbola and a
  transition/inconsistency fans across them.
- A **Detail zone** dropdown (and **clicking any scorecard card**) focuses both plots on that zone; a
  legend names the model colours and the band/hyperbola conventions.
- **⭳ CSV** exports the whole per-zone scorecard (zone, top/base, models, mean/max spread, worst-spread
  depth, fraction divergent, BVW mean/CV/n).

Canvas colours all come from `readTheme` (`--accent` band, per-model `faciesColor`, `--warn` marker,
`--grid` hyperbolae); the plots redraw on theme change and resize.

**Verification (in-browser, mocked IPC):** mounted against canned `list_zones` / `sw_method_spread` /
byte-packed `get_curve_data`. Both canvases rendered real content (non-uniform pixel counts ~738/737, not
blank frames); the legend listed Archie/Simandoux/Indonesia/Juhász; the Detail-zone dropdown held both
zones and switching to SAND-B redrew the Buckles plot; setting hoverDepth redrew the envelope's depth
marker; and **⭳ CSV produced the correct header + one row per zone** (mean_spread 0.17/0.01,
frac_divergent 0.7/0, bvw_n 25). tsc exit 0; cargo unchanged at 348. (Screenshot skipped — the preview
pane wasn't compositing; verified via pixel-content + DOM + captured CSV text instead. Console clean of
panel-origin errors.)

> **Try:** open **Results QC**, pick a zone in **Detail zone** (or click its card). The **Sw-method
> envelope** shows the model band — watch Archie ride above the shaly-sand lines in fresh-water sand;
> drag the log crosshair and the dashed depth marker follows. The **Buckles** plot shows your SWE·PHIE
> against constant-BVW curves — a clean pay leg hugs one curve. Hit **⭳ CSV** for the scorecard table.

## Round 39 — Results-QC #8 inc 2: panel + per-zone QC scorecard (2026-07-23)

New well-bound panel `src/ui/resultsQcPanel.ts` (`buildResultsQcContent`), registered like the other
singletons — `buildRenderer` case, `openResultsQc`, the ＋-menu entry, a **Results QC…** ribbon button
(next to Field Dashboard), and `#results-qc-btn` wiring. Follows the selected well (`wellPane`,
`followData`), so it rebuilds when the interpretation changes.

For every zone of the well (or "All depth" when none) it shows a **per-zone card** with a traffic-light
per check:

- **Sw-method spread** — calls the inc-1 `sw_method_spread` per zone and lights **ok / caution / alert**
  on the fraction of divergent depths (≤10 % / ≤40 % / more), with `mean · max @ depth · % divergent`
  and the model list + notes on hover.
- **Buckles (BVW)** — BVW = SWE·PHIE over the zone; lights on the coefficient of variation (≤15 % /
  ≤30 % / more) with `BVW mean · CV% · n`. Framed as a prompt (transition zone vs. inconsistent Sw), not
  a verdict — the crossplot that resolves which comes in inc 3.

A compact Sw-params row (Rw, Rw °F, Form °F, m, n, Rsh, a, divergence threshold — editable defaults the
user confirms, nothing fabricated) drives a **Recompute**. Traffic-light dots are theme-var coloured
(`--accent` ok / `--accent2` caution / `--warn` alert — never hard-coded red/green). The card under the
crosshair highlights via `appState.hoverDepth`.

**Verification (in-browser, mocked IPC):** mounted the panel against canned `list_zones` /
`sw_method_spread` / a byte-packed `get_curve_data`. Two zones rendered correctly — SAND-A: Sw-spread
**alert** (mean 0.180, max 0.190 @ 2010 m, 66 % divergent) + Buckles **ok** (BVW 0.060, CV 1 %); SAND-B:
Sw-spread **ok** (0 % divergent) + Buckles **alert** (CV 31 %); status "2 zone(s) · 2 flagged".
hoverDepth 2020→SAND-A, 2070→SAND-B, null→neither (highlight follows the crosshair). Screenshot confirms
the cards; console shows only the pre-existing backend-absent boot errors — none from the panel. tsc exit
0; cargo unchanged at 348.

> **Try:** ribbon **Batch → Results QC…** (or ＋ → Results QC). With a well selected, each zone gets a
> card: the **Sw-method spread** light goes amber/red where Archie and the shaly-sand models disagree
> (fresh-water sand), and **Buckles (BVW)** flags zones whose bulk-volume-water wanders. Tune Rw/m/n/Rsh
> and hit **Recompute**; move the log crosshair and the matching zone card highlights.

## Round 38 — Results-QC #8 inc 1: Sw-method spread backend (2026-07-23)

First increment of the Results-QC / Sw-comparison dashboard. New Rust module `src-tauri/src/resultsqc.rs`
+ command `sw_method_spread` (ipc `swMethodSpread`) — the one metric the dashboard genuinely needs from
the backend, because the five Sw models are pure `fn`s in `multimin2` that the frontend can't call.

Per depth it evaluates every Sw model whose input curves are present and returns the **envelope**
(sw_min / sw_max / spread), a per-series value set, and a **divergence summary** (mean/max spread, the
depth of worst disagreement, the fraction of comparable depths above a 0.10-Sw threshold, and a notes
trail). **Archie / Simandoux / Indonesia / Juhász** run from the always-available logs; **Waxman-Smits**
joins only with a Qv curve and **Dual-Water** only with a bound-water-saturation curve — no CEC/Qv/Swb is
ever fabricated to force a model in. Fluid conductivities reuse the app's own `fluid_calc`/`waxman_b`
path (no divergence, no invented constants); the classic fresh-water-sand story falls straight out —
Archie over-reads Sw while the clay-aware models cluster below it.

**Adversarial review (1 skeptic, math-heavy) — 3 medium + 3 low, all fixed:** (M) a null Qv silently
collapsed Waxman-Smits to Archie via `(B·Qv).max(0)` → now returns NaN at any non-finite/negative Qv;
(M) `BQV` (= B·Qv) was auto-aliased into the Qv slot and re-multiplied B → dropped from auto-candidates,
needs an explicit override; (M) model activation counted *columns* not *finite data*, so an all-null
column inflated the "active" count and muted the warning → a model is kept only with ≥1 finite Sw, the
insufficient-data note keys on comparable-depth count, dropped columns are reported by name; (L) a note
now fires when PHIE is absent; (L) ambiguous `PHI` moved off the PHIE candidate list onto PHIT; (L)
added numeric Juhász, WS/DW-reduce-to-Archie-at-zero, null-Qv→NaN, and all-null-column tests. The review
also cleared the units (Rw=1/Cw at formation T, Cwb=virgin, B(T,Rw)), envelope, and index-alignment.

**Verification:** cargo 348 passed / 0 failed / 7 ignored (+8 new resultsqc tests); tsc exit 0. Read-only
— computes nothing to disk.

> **Try:** no UI yet — the per-zone scorecard + Sw-envelope track that consume this land in the next
> increment (#8 inc 2/3). The command itself is exercised by those; nothing to click through here.

## Round 37 — Contacts #6.2 inc B: assisted contact picking — the panel UI (2026-07-23)

Second increment — wires inc A into the correlation panel's **Contacts…** editor with two new sections:

- **Suggest from logs** — pick a well and a depth zone (defaults to the visible window), hit **Suggest**,
  and get the ranked candidates (Sw crossover / resistivity drop / density-neutron gas base), each showing
  `type @ depth — method (confidence%)`; low-confidence (<40%) rows are dimmed. **Accept** on a candidate
  creates a well-scoped MD contact at that depth (and appears in the editor's table) — **never
  auto-committed**, one click per pick.
- **Cross-well consistency** — pick a contact type, hit **Check**, and get a readout: `N wells · dip
  plane|flat mean · mean TVDSS · rms`, then a per-well table (TVDSS, predicted, residual) with **⚠-flagged
  wells** that disagree with the flat-TVDSS surface.

**Verification (in-browser, mocked IPC):** mounted the correlation panel, opened the Contacts editor, and
drove both sections from the DOM: Suggest rendered 3 ranked candidates with the 35% one flagged weak;
**Accept called upsert_fluid_contact with `{well W1, OWC, 2148.5, MD}`** and added the row + set the
status (no auto-commit); Check rendered the summary ("3 wells · dip plane · mean 2076.1 · rms 1.4 m") and
flagged the 12 m-off Well-3 while clearing the inliers. tsc green; cargo unchanged at 340. (Console shows
only pre-existing backend-absent boot errors — none from the panel.)

Deferred (noted, not silently dropped): **snap-to-log-feature while dragging a contact line** — contacts
aren't draggable in the panel yet (drag is pan), so a hit-test + drag handler is a larger change left for
a follow-up; the Suggest/Accept flow covers the assisted-picking need in the meantime.

> **Try:** open a **Correlation** panel → **Contacts…**. Under **Suggest from logs**, choose a well with
> Sw/resistivity/density-neutron over a hydrocarbon-water zone and hit **Suggest**; **Accept** the pick you
> trust — it drops in as an OWC in that well. Then set contacts of that type in a few wells, and under
> **Cross-well consistency** hit **Check** — any well off the flat-TVDSS surface shows a ⚠.

## Round 36 — Contacts #6.2 inc A: assisted contact picking — backend (2026-07-23)

First increment of assisted fluid-contact picking (the existing contacts editor + TVDSS-flat
rendering was already built and committed in the Wave-B chain — that is #6 inc 1). New `contacts.rs`
with two read-only commands:

- **`suggest_contacts`** — from one well's logs within a depth zone, proposes contact depths from
  three independent indicators, each with a confidence, ranked: the **Sw = cutoff crossover** (default
  0.5; confidence = the below-minus-above contrast), the **deep-resistivity drop** (steepest downward
  step in log10 Rt; confidence ∝ decades fallen), and the **density-neutron gas base** (where φN−φD
  closes back through −0.03 — gas-down-to). Uses whichever curves are present. Nothing is written — the
  user accepts/edits (inc B).
- **`check_contact_consistency`** — a contact is flat in TVDSS, so it fits a **least-squares dip plane**
  (z = a + b·x + c·y, on centred UTM coords) through every well's pick of a type, converts MD picks to
  TVDSS via each deviation survey, and **flags wells whose residual exceeds a threshold** (default 3 m).
  Falls back to a flat mean when < 3 wells have coordinates.

**Adversarial review** (math-heavy) confirmed the crossing interpolation, the resistivity-drop loop, the
plane solve, and the MD→TVDSS interpolation correct with no panics/divide-by-zeros, and surfaced four
issues I fixed: (1, **medium**) the consistency check was **mixing baselines** — coord wells scored vs
the plane, coordless wells vs the flat mean → false flags and a blended RMS; now it uses **one baseline**
(coordless wells are left *unscored* under a plane, RMS over scored points only); (2) resistivity depth
was ~win/2 shallow → refined to the sharpest single-sample step; (3) noisy Sw flooded candidates → cluster
dedup; (4) the neutron PU/fraction unit is now decided once per curve, not per sample.

**Verification:** cargo **340 passed / 0 failed** — Sw crossover recovers a known 2050 m contact; the
~1.4-decade resistivity drop scores high and lands on the step; the D-N gas base hits 2040 m; `fit_plane`
recovers a known dipping plane; the consistency check flags a 12 m outlier while clearing inliers; and a
coordless well is left unscored (not false-flagged) under a plane. tsc green. Backend-only — the panel
wiring is inc B.

> **Try:** backend-only this round — no new button yet. The **Suggest from logs** action and the
> cross-well consistency readout land in the next increment inside the **Contacts…** editor.

## Round 35 — Autocorrelate #5 inc 3: the dialog — warp toggle, multi-select, per-marker review (2026-07-23)

Third increment — the UI that makes inc 1/2 usable. The **Autocorrelate** pane is rewritten:

- **Tops are now a checkbox list** (with an **All** toggle): tick **one** top to correlate a single marker,
  or **several** to propagate a consistent set together. The run button tracks it — "Correlate 2 wells"
  vs "Correlate 3 tops → 2 wells".
- **Method** dropdown — **Rigid shift (fast)** or **Elastic warp** — wired to inc 1/2's `method`. The
  **Max stretch ×** control appears only for warp; the **Window ±** control appears only for a single top
  (multi derives its window from marker spacing).
- **Per-marker review** — single mode shows a well×proposal table; multi mode shows a (well, marker) table
  grouped by well, each row with its **own r**. Strong matches (r ≥ 0.7) pre-ticked; **low-confidence rows
  flagged** (dimmed) and left unticked; a well with no data shows an error row.
- **Accept/reject per row**, then **Apply** writes only the ticked picks as **one undoable batch** (undo
  restores/deletes each pick).

**Verification (in-browser, mocked IPC):** the pane mounted and every interaction was driven and read back
from the DOM — control show/hide is exactly right (max-stretch only on warp, window only single, label
tracks selection); the multi table renders 3 markers under a well with the low-r marker flagged/unticked
and the errored well shown; **Apply invoked upsert_top for exactly the two ticked markers** (not the weak
one, not the errored well) and set the batch status; the single path passes `method:"shift"` and flags its
r 0.61 row. tsc green; cargo unchanged at 334. (Console shows only the pre-existing backend-absent boot
errors — none from the dialog.)

> **Try:** open the **Autocorrelate** pane (＋ menu or the ribbon). Tick **one** top, set **Method =
> Elastic warp**, give a **Max stretch** (say 1.5), and **Correlate** — review the r per well, untick weak
> matches, **Apply**, then **Ctrl-Z** to confirm the batch undoes. Then tick **several** tops and
> **Correlate** again: you get a per-marker table, and the applied set stays in stratigraphic order (no
> crossings) in the correlation view.

## Round 34 — Autocorrelate #5 inc 2: multi-marker simultaneous propagation — backend (2026-07-23)

Second increment. Adds `autocorrelate_multi` (new `autocorrelate_multi` command): propagate **several
markers together** into each target well as one **consistent** set, each with its **own** confidence.

- **Consistency by construction** — markers are propagated top-down, each warped in its own local window
  (the inc 1 warp), and a **hard monotone guard** forbids a later marker from crossing above an earlier
  one. The guess for each marker is *guided* from the previous proposal (carry the source spacing
  forward), so the search per marker is small and can't lock onto a neighbour's feature.
- **Per-interval confidence** — each propagated marker carries its own Pearson r (the per-marker score
  the spec asks for), not one r for the whole well.
- Empty selection ⇒ all source tops; markers can be named explicitly. Read-only — the dialog (inc 3)
  reviews and applies.

Refactor: the single-marker `autocorrelate_top` and the new multi path now share `build_template` +
`propagate` (rigid best-lag → optional warp-refine with the better-of guard). No behavior change to the
single path — all its tests still pass. inc 2 adds no new math; it reuses inc 1's adversarially-reviewed
primitives, so the review this round was a focused self-check of the guided-guess / monotone-guard
orchestration (no k=0 index underflow; a skipped marker never corrupts the set).

**Verification:** cargo **334 passed / 0 failed** — new test propagates 3 markers through a ×1.25 stretch
(each moves a different amount), recovering all three to <3 m, in strict order, each scored. tsc green.
Still backend-only — nothing browser-observable yet.

> **Try:** backend-only again — no new button this round. The multi-marker propagation becomes usable in
> the next increment's dialog (select several tops, one **Correlate**, review a per-marker table, apply as
> one undoable batch).

## Round 33 — Autocorrelate #5 inc 1: elastic depth warp (subsequence DTW) — backend (2026-07-23)

First increment of the marker-autocorrelation enrichment. `tops.rs` today propagates a top from the
source well to others by a **rigid best-lag** GR match (slide the pick window, keep the max-Pearson
depth). That is unchanged and stays the fast default. This increment adds an **elastic depth-warp**
mode alongside it:

- **`subseq_dtw`** — open-begin/open-end subsequence dynamic-time-warping. The `(1,1)/(1,0)/(0,1)` step
  set makes the alignment **monotone and non-inverting by construction** (no depth crossovers), and a
  per-step stretch penalty keeps it near slope 1 unless the log clearly warps.
- **`warp_refine`** — refines the rigid pick: builds a target window sized for the requested stretch,
  **P3/P97-normalizes** both logs (the `gr_normalize` two-point idea, applied window-locally) so the warp
  compares *shape*, not tool calibration/datum, then reads off the depth the marker (window centre) warps
  to. Reported r is the template-vs-warped-target Pearson — the **same metric as the rigid r**.
- Request gains `method` (`shift`|`warp`) and `max_stretch`, both serde-defaulted, so the existing
  dialog call is byte-identical (`shift`). **No UI yet** — the warp/shift toggle, max-stretch control and
  per-interval tie-lines come in inc 2/3.

**Adversarial review** (math-heavy) confirmed the DTW recurrence, back-pointers, monotonicity, and
marker mapping correct with no OOB/underflow/panic paths, and surfaced three behavioral gaps I then
fixed: (1) warp could **silently regress** a better rigid pick → added a better-of guard (keep warp only
if its r ≥ rigid r − ε); (2) a marker could be placed **in a data gap** with a plausible r → reject a
warp whose marker lands on a NaN sample (fall back to rigid) and raised the NaN step-cost so DTW avoids
nulls; (3) the `max_stretch` doc **overstated** a hard local cap → reworded to the honest soft/window
control it is. cargo: **333 passed / 0 failed** (rigid recovers a known 7.5 m lag; warp recovers a known
×1.5 piecewise-stretched section to ~1 m where rigid is ~10 m biased; warp does not regress a pure shift;
DTW path proven monotone/complete on noisy input). tsc green.

> **Try:** backend-only this round — open the **Autocorrelate** pane and correlate a top as before; it
> should behave **exactly** as it did (rigid shift, unchanged). The warp mode has no button yet; it lands
> in the next increment where you'll get a **Rigid / Elastic-warp** toggle and a max-stretch control.

## Round 32 — Unconventional #7 inc 5: ΔlogR overlay + Langmuir isotherm panel (2026-07-23)

Fifth and final increment — the visual companion to the four compute modules. A new workspace pane,
**Unconventional (ΔlogR + Langmuir)**, opens from **Petrophysics → Unconventional → ΔlogR + Langmuir
Visuals…** (also in every window's ＋ menu). It follows the active well like the other tool panes and
carries two pictures side by side:

- **Passey ΔlogR overlay** (depth track) — deep resistivity on a log/decade axis and a baselined
  porosity curve (sonic **DT** or density **RHOB**) drawn so the two **overlie in non-source rock and
  fan the opposite way over organic-rich intervals**; the shaded lens between them **is ΔlogR**, the
  input to `toc_passey`. Uses that module's exact scaling — resistivity at log10(R/R_base), porosity at
  −0.02·(DT−DT_base) [sonic] / +2.5·(RHOB−RHOB_base) [density] — so the picture and the number agree.
  R_base and the mode's baseline are editable; picks are read on a clay-rich, non-source shale.
- **Langmuir isotherm** — Gs = VL·P/(PL+P) (scf/ton) with the **VL** ceiling, the **PL** half-saturation
  point (Gs=VL/2 at P=PL), the **reservoir-pressure** operating point, and — given an in-situ gas
  content **GC** < VL — the **critical desorption pressure Pcd = PL·GC/(VL−GC)** for undersaturated
  coal/shale. This is the adsorbed term of the `gip` module, drawn.

Display-only (no new physics, no backend). Verified in-browser against synthetic source-rock curves
(a resistivity + Δt/ρb kick): the sonic and density overlays both render the correct opposing fan, and
the isotherm's PL/Pres/Pcd markers land where the formulae put them. **Two defects were caught in that
pass and fixed:** (1) the porosity curve was drawn at `xR − poroTerm` instead of the absolute `−poroTerm`,
which understated ΔlogR and leaned both curves the same way; (2) the baseline field toggled both DT_base
and RHOB_base together, hiding RHOB_base in density mode. tsc green; the four compute modules are
unchanged (330 cargo tests still pass).

> **Try:** open **Petrophysics → Unconventional → ΔlogR + Langmuir Visuals…**, select a well with a deep
> resistivity + sonic (or density) curve. On the left, set **R_base / DT_base** on a clay-rich,
> non-source shale and confirm the two curves overlie there and split (shaded) over your organic zones —
> that lens should track where `toc_passey` gives high TOC. Switch **Overlay = Density** to pair RHOB
> instead. On the right, type your **VL / PL / reservoir pressure**; for undersaturated coal add a **GC**
> and read **Pcd** off the isotherm.

## Round 31 — Unconventional #7 inc 4: brittleness index (elastic + mineralogical) (2026-07-23)

Fourth increment. A new module, **Brittleness index (elastic / mineralogical)** (Petrophysics →
Unconventional), scores rock brittleness (0 ductile .. 1 brittle) two ways:

- **METHOD = elastic** — dynamic Young's modulus and Poisson's ratio from **DT, DTS, RHOB** (moduli in
  GPa via ρ·V², Vp/Vs = 304.8/slowness, E→Mpsi), then Rickman et al. 2008 BI = (E_norm + ν_norm)/2. The
  normalization endpoints (E 1..8 Mpsi, ν 0.4..0.15 — Barnett defaults) are editable **params** so you
  can recalibrate to Mahakam. Also outputs the dynamic **YME** / **PR**.
- **METHOD = mineral_jarvie** — Jarvie 2007 BI = Qz/(Qz+carbonate+clay). **mineral_wanggale** — Wang &
  Gale 2009 BI = (Qz+Dol)/(Qz+Dol+calcite+clay+organic), moving dolomite to the brittle side. Feed the
  SandiMin **VOL_*** volumes (a missing mineral counts as absent); the organic term is the inc-2 **VKER**.

Tier-B, cited (Rickman et al. 2008; Jarvie et al. 2007; Wang & Gale 2009); the elastic moduli
reimplement the Techlog RockPhyEquations forms. Math in `docs/ref_unconventional.md` §4. Elastic E,ν are
dynamic (apply a static correlation before geomechanics, not before the Rickman index).

Verified: **330 cargo tests** (7 new — elastic recovers a known E/ν/BI from slowness, Jarvie/Wang-Gale
groupings, BI monotone in quartz, invalid-shear + negative-Poisson rejection, all-absent→NaN) + tsc,
adversarial review = FIX-FIRST → fixed: the elastic branch now rejects ν<0 (Vp/Vs<√2, a bad shear log)
instead of emitting a negative PR and a falsely max-brittle BI.

> **Try:** for the elastic index, open **Petrophysics → Unconventional → Brittleness index**, keep
> **METHOD = elastic**, set **DT / DTS / RHOB** (needs a shear sonic), and Run — check **BI** rises in
> the stiff, quartz-rich (high-E, low-ν) beds. For the mineral index, run **SandiMin** first, switch
> **METHOD = mineral_jarvie**, and map **VQTZ / VCARB / VDOL / VCLAY** to your VOL_* curves; compare
> against the elastic BI where you have both.

## Round 30 — Unconventional #7 inc 3: gas-in-place (free + Langmuir adsorbed) (2026-07-23)

Third increment. A new module, **Gas-in-place (free + Langmuir adsorbed)** (Petrophysics →
Unconventional), gives per-depth gas CONTENT (scf per ton of rock) so it composites like any curve:

- **GIP_ADS** = VL·P/(PL+P) — Langmuir adsorbed gas. **GIP_FREE** = 32.0368·φ·(1−Sw)/(RHOB·Bg) —
  compressed free gas, with **BG** = 0.02827·z·T/P (T in Rankine). **GIP_TOTAL** = free + adsorbed.
- **MODE = cbm** applies the dry-ash-free correction GIP_ADS·(1−F_ASH−F_MOIST) and, given a measured
  in-situ gas content **GC**, emits the **critical desorption pressure PCD** = PL·GC/(VL−GC) — the
  pressure the coal must be dewatered below before gas desorbs.

The Langmuir VL/PL default to shale placeholders (100 scf/ton, 1000 psia ≈ IP's 7000 kPaa) — override
with core desorption/isotherm data. Feed the **PHI** slot your effective porosity or the inc-2
**PHIT_OMC**. Tier-B, cited (Langmuir 1918; GRI / Mavor-Nelson 1996). The Ambrose pore-volume
correction (which trims free gas by the adsorbed-phase volume, ~10% in high-TOC/high-P shale) is
deferred with its derivation banked — so **GIP_TOTAL is an upper bound** until it lands. Math in
`docs/ref_unconventional.md` §3.

Verified: **323 cargo tests** (7 new — Langmuir at P=PL/0/∞, free gas pinned to an independent hand
literal 167 scf/ton, Bg pinned to 0.0055947, total = free+adsorbed, CBM ash/moisture, Pcd, Sw=1→0,
out-of-range rejection) + tsc, adversarial review = SHIP (constants 32.0368 / 0.02827 recomputed by
hand; all divisions Inf-guarded).

> **Try:** open **Petrophysics → Unconventional → Gas-in-place (free + Langmuir adsorbed)**, set
> **PHI** (PHIE or the inc-2 **PHIT_OMC**), **SW**, **RHOB**, and reservoir **RES_P / TEMP_F / Z_FAC**;
> enter your core **VL / PL**. Run and confirm **GIP_ADS** dominates in the organic-rich (low-φ)
> section while **GIP_FREE** dominates where porosity is higher, with **GIP_TOTAL** their sum. For coal
> switch **MODE = cbm**, set **F_ASH / F_MOIST**, and enter a canister **GC** to see **PCD**.

## Round 29 — Unconventional #7 inc 2: kerogen volume + OM-corrected porosity (2026-07-23)

Second increment. A new module, **Kerogen volume + OM-corrected porosity** (Petrophysics →
Unconventional), turns the TOC curve into a kerogen VOLUME and corrects total porosity for the organic
matter that low-density kerogen inflates on the density log:

- **TOM** = k_toc2om · TOC/100 — organic-matter weight fraction (k_toc2om ≈ 1.2 accounts for the
  H/O/N/S beyond carbon).
- **VKER** = TOM · RHOB / ρ_kero — kerogen volume fraction of the *bulk* rock (Passey/Vernik
  bulk-density conversion). ρ_kero defaults to **1.10 g/cc** to match the SandiMin **Kerogen** mineral,
  so VKER reconciles with a SandiMin **VOL_KEROGEN**.
- **PHIT_OMC** = PHIT − VKER — strips kerogen's apparent-porosity contribution (feed a density-derived
  PHIT).

Chains off inc 1 (reads the **TOC** curve by default) and feeds inc 3 (GIP needs kerogen volume).
Tier-B, cited (Passey et al. 2010; Vernik & Nur 1992). Method math in `docs/ref_unconventional.md` §2.

Verified: **316 cargo tests** (5 new — bulk mass balance recovers a known VKER, OM-correction subtracts
and floors, zero-TOC is inert, VKER rises with TOC, missing RHOB falls back to TOM only) + tsc,
adversarial review = SHIP (fixed one wrong default pre-commit: ρ_kero was 1.20 but SandiMin's Kerogen
is 1.10 — now reconciled).

> **Try:** run **TOC — Passey ΔlogR** first (Round 28) so a **TOC** curve exists, then open
> **Unconventional → Kerogen volume + OM-corrected porosity**, set **RHOB** and (optionally) a
> density-derived **PHIT**, and Run. Check **VKER** is a few percent where TOC is a few wt% (light
> kerogen occupies ~2× its weight fraction), and that **PHIT_OMC** reads a touch below your input PHIT
> in the organic-rich section. Compare **VKER** against a SandiMin **VOL_KEROGEN** run (organic preset)
> — they should track at the default ρ_kero 1.10.

## Round 28 — Unconventional #7 inc 1: TOC from Passey ΔlogR + Schmoker (2026-07-23)

First increment of the unconventional / shale suite (playbook Part II #7). A new **Unconventional**
group on the Petrophysics ribbon, with its first module: **TOC — Passey ΔlogR + Schmoker**. It
estimates total organic carbon two independent ways:

- **Passey (1990) ΔlogR** — the separation between deep resistivity and a *baselined* porosity curve.
  Choose the **overlay**: *sonic* (`ΔlogR = log10(R/R_base) + 0.02·(DT−DT_base)`) or *density*
  (`−2.5·(RHOB−RHOB_base)`). Set the baselines (`R_BASE`, `DT_BASE`/`RHOB_BASE`) on a clean, clay-rich,
  **non-source** interval where the two curves overlie (ΔlogR≈0), then
  `TOC = ΔlogR·10^(2.297−0.1688·LOM) + background`. LOM (maturity, 6..12) defaults to 10.6.
- **Schmoker-Hester (1983)** density-TOC `154.497/RHOB − 57.261` as an independent cross-check
  (writes `TOC_SCHMOKER` whenever a density curve is present, regardless of overlay).

Outputs: `DLOGR` (the raw separation, for the overlay panel coming in inc 5), `TOC` (Passey, wt%),
`TOC_SCHMOKER` (density cross-check). In non-source rock (ΔlogR<0) TOC floors to the *background*
value, not below it. Tier-B, cited in code (Passey et al. 1990; Schmoker & Hester 1983); the LOM and
baseline defaults are Tier-A IP seeds, per-well overridable. The **neutron** overlay is deferred — its
sign convention is inconsistent across the literature and needs core verification. Method math banked
in `docs/ref_unconventional.md` §1.

Verified: **311 cargo tests** (7 new — sonic/density overlays recover a known TOC, TOC decreases with
LOM, non-source floors to background, missing overlay curve falls back to Schmoker) + tsc green +
adversarial review (found & fixed one clamp-order defect pre-commit: a nonzero background must be the
floor, not zero). Additive — nothing existing moves.

> **Try:** open **Petrophysics → Unconventional → TOC — Passey ΔlogR + Schmoker**. Set **overlay =
> sonic**, **RES** = deep resistivity, **DT** = sonic. On a clean *non-source* bed read R and Δt and
> enter them as **R_BASE** / **DT_BASE** (so ΔlogR≈0 there); set **LOM** from your Ro/Tmax (or leave
> 10.6) and Run. Confirm **TOC** rises through the organic-rich section, and compare against
> **TOC_SCHMOKER** where RHOB exists. If you have core TOC, nudge LOM until the Passey curve matches.

## Round 27 — SandiMin per-depth formation temperature (FTEMP curve) (2026-07-23)

Formation temperature can now come from a **per-depth curve** instead of one fixed number. On the
**Fluids** tab there's a new **FTEMP curve (opt)** box next to *Formation temp (°F)*. Leave it blank to
use the fixed value (unchanged). Type a curve name (e.g. **FTEMP_F**, the curve Prep builds from a
gradient/BHT) and, for every depth where that curve is finite, SandiMin recomputes the temperature-
dependent quantities at that sample's temperature:

- **Cw / Cmf / Cbw** (formation-water, filtrate and clay-bound-water conductivities),
- the **auto CT/CXO uncertainties**,
- the **clay bound-water tie** (BNDWAT multiplier k, via t_c),
- the **Waxman-Smits B(T,Rw)**.

The α (diffuse-layer) expansion and salinities come from the *Rw/Rmf* sample temperatures, so they don't
move with formation temperature — only the conductivities do. A sample where the curve is missing or
out of range (a null like ±999.25, or anything outside 32–600 °F) quietly falls back to the fixed °F, so
selecting the curve is safe even on wells that lack it. With the box blank the solve is **byte-for-byte
identical** to before (a test pins that a constant FTEMP curve equal to the fixed value reproduces the
fixed-temperature run exactly), and the per-tool reconstruction-QC curves stay consistent under a curve.

> **Try:** run **Prep** so a **FTEMP_F** curve exists (or import one), then open **SandiMin → Fluids**,
> put **FTEMP_F** in **FTEMP curve (opt)**, and Run. Compare **SWE** with and without the curve over a
> long interval with a real geothermal gradient — the hotter, deeper section reads a bit lower Sw (hotter
> water is more conductive). Blank the box to confirm you get the fixed-temperature numbers back.

## Round 26 — SandiMin Waxman-Smits saturation model (2026-07-23)

The last of the Sw models. **Waxman-Smits (B·Qv)** joins the **Sw model** dropdown (Fluids tab). Like the
other post-solve forms it runs the mineral inversion untouched, then replaces the water/HC split from the
deep resistivity — here via `Ct = φt^m·(Cw·Swt^n + B·Qv·Swt^(n−1))`:

- **Qv** is built from the **solved clay volumes**: `Qv = Σ v_clay·CEC·ρ_clay / φt` (meq/mL). So each clay's
  **CEC** (Clay tab) drives the excess conductivity — a clean sand (no clay ⇒ Qv=0) collapses to Archie.
- **B** is the counterion conductance from the **Juhász (1981) B(T,Rw) fit** — the same closed form Techlog
  and IP use — computed from formation temperature and Rw automatically. Because that fit is known to
  overshoot above ~120 °C, a **B override (0 = auto)** box (shown only for this model) lets you pin a
  core-measured B.
- Uses your **m/n as m\*/n\***. PHIE/PHIT stay exactly as the mineral solve made them; only SWE/SWT/SXOT move.

Verified: the conductivity root and the B(T,Rw) fit are hand-anchored in unit tests (n=2 closed form, n=3
bisection, Qv=0/B=0 → Archie, B(25 °C,0.1)=3.895, B(100 °C,0.05)=15.51, monotonic in T and Rw), plus a
full-run integration test that recovers a known Sw. Nothing else moves — the default model is still linear
dual-water.

> **Try:** open **Petrophysics → SandiMin**, **Fluids** tab, set **Sw model → Waxman-Smits (B·Qv)**. Make sure
> a **CT** (deep-resistivity) tool and a **U-zone hydrocarbon** component are set, and that your clays carry a
> **CEC** (Clay tab). Run and compare **SWE** vs **Archie** (Waxman-Smits reads lower on shaly intervals) and
> vs **Juhász**. Leave **B override** at 0 for the auto B(T,Rw); enter a core B to pin it and re-run.

## Round 25 — SandiMin Constraints tab: porosity source + program-constraint toggles (2026-07-23)

The UI for item B (your image 2). A new **Constraints** tab (after Clay) holds two things:

- **Porosity source** radio — **Cation Exchange Capacity** (default) vs **Wet Clay Porosity**. This picks
  what drives the clay bound-water tie: CEC uses `α·96·CEC·ρ/(T+298)`; WCP uses the geometric `k = φ/(1−φ)`
  from a **per-clay φ editor** now on the **Clay** tab (pre-filled with Techlog WCLP defaults — Illite 0.104,
  Kaolinite 0.058, etc.). Running the dry-clay converter also fills a clay's φ, so the two stay consistent.
- **Program constraints** — enable toggles for **UNITY**, **POROSITY**, **X&U BNDWAT**, **WATER MUD**, plus a
  **Constraint tolerance σ** (default 0.01). All four already ran in the solver; this exposes them. UNITY moved
  here from the run footer (there's no longer a "Hard unity" box down by Run).

Defaults are unchanged behavior: CEC, all four on, σ=0.01 — so an untouched Constraints tab solves exactly as
before (a backend test pins that "absent request fields = on"). WATER MUD defaults on for water-based mud (it
keeps flushed-zone water ≥ virgin water; ignored for OBM) — tell me if you'd rather it default off.

> **Try:** open **Petrophysics → SandiMin**, click the **Constraints** tab. Flip **Porosity source** to
> **Wet Clay Porosity**, check the **Clay** tab's per-clay φ list, then Run and compare **PHIE/SWE** vs CEC
> (WCP moves PHIE for clays). Toggle a constraint off (e.g. **WATER MUD**) or change **σ** and re-run to see
> the effect. Confirm the run footer has **no** "Hard unity" box (it's now the UNITY toggle on this tab).

## Round 24 — SandiMin Wet-Clay-Porosity bound-water source (backend) (2026-07-23)

Starting item B (constraints editor + porosity source). This first slice is the **backend route** for
the **Porosity Source** choice from your image 2: the clay bound-water constraint can now be driven by
either **CEC** (default — `v_bw = α·96·CEC·ρ/(T+298)·v_dryclay`, nothing moves) or **Wet Clay Porosity**
(`v_bw = φ_clay/(1−φ_clay)·v_dryclay`, geometric). It's the same physics the Clay-tab wet→dry converter
already used (`dry_clay_calc`); this exposes it as a selectable source. Clays now carry Techlog's WCLP
defaults (Illite 0.104, Kaolinite 0.058, Chlorite 0.101, Glauconite 0.156, Montmorillonite 1.0, Clay 0.12).

Default stays **CEC**, so every reviewed number is untouched (verified: the CEC path is byte-identical to
before). The **UI radio + per-clay φ editor + the constraints panel (UNITY/POROSITY/X&U BNDWAT/WATER MUD)
land in the next slice** — nothing to click yet. Tests: the WCP multiplier equals the CEC route's
`cec_equiv` (the dry_clay_calc bridge) and drives the same bounded solve; Techlog WCLP defaults asserted;
adversarially reviewed. Note: the WCP source **moves PHIE** for clays (bound water is now geometric, not
CEC-derived) — that's the design you approved.

**Smectite fix (adversarial review caught this before commit).** Techlog carries `WCLP_Smectite = 1.0`,
but it only ever consumes that value *post-solve* for wet-clay-volume reporting (flooring `1−φ` at `1e-4`),
never as an inversion constraint. My first cut fed it straight into the BNDWAT *solver* row as `φ/(1−φ)`
with a `0.95` cap → `k ≈ 19`, ~100× every real clay and ~30× smectite's own CEC route — it would have
swamped the bound-water constraint and forced absurd bound water wherever montmorillonite appears. Fixed:
a degenerate `φ ≥ 0.5` (Techlog's real clays are all ≤ 0.156, so this cleanly isolates the `1.0`
placeholder) now **falls back to the CEC-calibrated multiplier** for that clay, so the two porosity
sources *agree* for smectite (`k ≈ 0.6`) instead of diverging 30×. New test
`wcp_degenerate_smectite_falls_back_to_cec` pins it; `library_has_expected_shape` asserts every
non-smectite clay's WCLP stays a physical geometric porosity. Real clays (Illite φ=0.104, etc.) are
unaffected — they still use the geometric `φ/(1−φ)` route.

## Round 23 — SandiMin Juhász / normalized-Qv Sw (the wet-shale model) (2026-07-23)

The **Juhász (normalized Waxman-Smits)** model — the wet-parameter one you grouped with Indonesia/
Simandoux — is now in the Sw dropdown as **"Juhász / normalized Qv."** Instead of dual water's
temperature-form clay conductivity, it reads the excess conductivity straight from the **shale point**:

    Cwsh = 1/(Rsh·φ_sh^m),   QVN = Vsh·φ_sh/φt,   Cw·Swt^n + QVN·(Cwsh−Cw)·Swt^(n−1) = Ct/φt^m   (a=1)

so it uses your wet-shale parameters directly (Rsh from a shale pick + **φ_sh = wet-clay porosity**, a new
input that appears only for this model). Runs **post-solve** like the others — the mineral solve is
untouched, **PHIE/PHIT/unity preserved**, only SWE/SWT/SXOT move. With Vsh=0 it collapses to clean-sand
Archie (tested). Equation matches the Geolog `sw_juha` / cookbook normalized-Qv form.

Internally, dual-water and Juhász now share one root solver (`sw_cond_root`) — the only difference is the
excess-conductivity coefficient (dual water `Swb·(Cwb−Cw)`; Juhász `QVN·(Cwsh−Cw)`). The dual-water
numbers are unchanged (same 30 tests green). Hand-computed literals at n=2 (closed form) and n=3
(bisection), Vsh=0→Archie, and NaN guards all pass; adversarially reviewed.

**Note on the porosity source:** Juhász here uses φ_sh only *inside the conductivity equation* — the
water/HC split still uses the CEC-solved bound water (so PHIE stays put). The *full* "Wet Clay Porosity"
porosity-source that redefines bound water (image-2 constraints panel) arrives with that editor; the two
are the same underlying mechanism and I'll wire them together there.

- [ ] **Juhász vs Simandoux/Indonesia.** On a shaly interval with a good shale pick (Rsh, φ_sh), confirm
      Juhász SWE sits in a sensible band with the other shaly-sand models; on a clean sand it should track
      Archie. Try: Fluid tab → Sw equation → *Juhász / normalized Qv*, set Rsh + φ_sh, Run.

## Round 22 — SandiMin log-input grid + tidy Run button (2026-07-23 field review)

Two visual fixes from your screenshots:

- **Log inputs (image 3 style).** The cramped single column with wrapping labels
  ("Formation Density" breaking across lines) is now a **multi-column grid** — one column
  when the pane is narrow, more as it widens, scrolling both ways, matching the mineral list.
  Labels ellipsis instead of wrapping so the checkboxes stay aligned; hover shows the full
  name + mnemonic.
- **Run button (image 1 style).** No longer a full-width slab — it's now a **tidy, left-aligned
  button** with standard module proportions like Porosity-from-Density, and (per your "then for run
  button" go-ahead) in the **theme accent** so it matches every other module's Run across the
  client-brand skins. This supersedes the earlier "distinct green" — say the word if you actually
  wanted it kept a different colour and I'll bring the green back.

Verified in the browser against the live CSS: log grid resolves to 2 columns at a 560 px pane
(1 when narrower), labels truncate with ellipsis (no wrap), Run renders 76 px wide (not full
width) in the accent colour (rgb(217,140,63) in the dark skin). tsc clean.

- [ ] **Log inputs read cleanly** at your usual pane width — columns wrap sensibly, no label
      overflow, checkboxes line up.
- [ ] **Run button** looks right in the accent colour where it sits at the top of the pane.

## Round 21 — SandiMin Archie (clean-sand) Sw + deduplicated menu decision (2026-07-23)

You chose the **deduplicated** Sw menu (one entry per distinct model). First of the remaining ones:
**Archie (clean sand)** — `Sw = (a·Rw/(φt^m·Rt))^(1/n)`, no shale term. It's the exactly-invertible
baseline (so there's no separate "Archie linear/nonlinear" — they'd be identical). Runs post-solve like
the others: PHIE/PHIT/unity preserved, only the water/HC split moves; on shaly sand it reads
optimistically high (by design — it's the baseline the shaly-sand forms correct). Tests: hand-computed
literals at n=2 and n=3, clamp/NaN guards, and a check that Archie ≡ Indonesia with Vsh=0. cargo + tsc clean.

Menu now: Linear dual-water (default) · Dual-water non-linear · **Archie** · Indonesia · Simandoux.
Still to come: **Waxman-Smits** (dry BQv, Waxman-Thomas B default) and **Juhasz / Normalized-Qv**
(wet-param — brings in the wet-clay-porosity input that also feeds the image-2 porosity-source toggle).

- [ ] **Archie baseline.** On a clean water/HC sand, confirm Archie SWE matches your quick-look Archie;
      on a shaly interval, confirm it reads higher than Simandoux/Indonesia (the expected over-estimate).

## Round 20 — SandiMin non-linear dual-water Sw (the 4th model you picked) (2026-07-23)

The **non-linear dual-water** you asked me to continue is now in the Fluid-tab "Sw equation" dropdown as
**"Dual-water non-linear (m, n separate)."** Unlike the default *linear* dual-water — which folds the
exponents into a single `w = 0.75m+0.25n` and solves the conductivity as a linear row inside the
inversion — this solves the **exact** Clavier-Coates-Dumanoir form honouring **m and n separately**:

    Ct = (φt^m · Swt^n / a) · [ Cw + (Cwb − Cw)·Swb/Swt ]

It runs **post-solve** (same as Indonesia/Simandoux): the mineral inversion runs untouched (the CT tool
stays in, so the split stays well-posed), then Swt is solved from that equation and the water/HC split
redistributed — **PHIT, PHIE and hard unity are preserved**, only SWE/SWT/SXOT move. The **bound-water
saturation comes straight from the solved bound-water volume** (Swb = v_bw/φt), so no lab Qv is needed,
and the clay-bound-water conductivity Cwb is the temperature form already in the fluid calc. Equation
verified against the Geolog `sw_dual` stdlib form.

Tests: hand-computed numeric-literal point (φt=0.3, Swb=0.2, Cw=2, Cwb=5, m=n=2 ⇒ Rt=10.288 ⇒ SWT=0.6),
the effective-Sw conversion (SWE=0.5), a general-n bisection round-trip, NaN guards, and an end-to-end
run recovering a known deep Sw with PHIE untouched. `linear_dw` stays the default — reviewed numbers
unmoved.

Still to come from your image-1 menu: Archie linear/nonlinear, Waxman-Smits, Juhasz + Normalized
Dual-Water (the wet-param normalized-Qv forms), and the wet/dry-clay-parameter wiring.

- [ ] **Dual-water non-linear.** On a well with CT + an HC component (ideally with a clay + BoundWater so
      Swb>0), run once on Linear dual-water then again on **Dual-water non-linear** with your m and n —
      confirm SWE/SWT move to the exact-equation answer while PHIE/PHIT come out identical to Linear.

## Round 19 — SandiMin dialog layout (your field review: run-on-top, tab order, multi-column) (2026-07-23)

Four layout fixes from your image markups, all in `src/ui/multiminDialog.ts` + `src/styles.css`:

- **Run / apply-to-wells on top.** The Apply-to-wells scope, output options, and the **Run** button now
  sit in a boxed section **above** the parameter tabs, so you launch a run without scrolling past every tab.
- **Run button is a distinct green** (`#2e7d4f`), set apart from other modules' accent-coloured runs.
- **Log inputs tab is first** (Log inputs → Minerals → Fluid → Clay) and the pane **opens on Log inputs**.
- **Minerals / Clays / Fluids lists are multi-column** — they wrap to as many columns as the pane width
  allows and scroll both ways, instead of one endless single column.

Browser-verified in the live DOM: tab order + default tab, run-section-before-tabs, the green run colour
(rgb 46,125,79 on white), and the minerals list laying out in 3 columns at a 920-px pane. tsc 0. Nothing
about the solve changed — this is layout only.

- [ ] **Layout sanity.** Open SandiMin: confirm Run + Apply-to-wells are on top, the Run button is green,
      Log inputs is the first/active tab, and the Minerals/Clays/Fluids lists show multiple columns
      (narrow the pane and confirm they reflow / scroll).

## Round 18 — SandiMin Sw-equation selector on the Fluid tab (your request, increment 3b) (2026-07-23)

The backend from Round 17 is now selectable. The **Fluid tab** has a new **"Sw equation"** dropdown —
**Linear dual-water (default)** / **Indonesia (Poupon-Leveaux)** / **Simandoux (modified)**. Pick a
shaly-sand form and two extra fields appear (**Rsh** shale resistivity, default 4.0 ohmm; **Archie a**,
default 1.0) plus a one-line note explaining it runs post-solve and needs a CT tool + a U-zone HC
component. Leave it on Linear and everything behaves exactly as before. Browser-verified: the three
options render, the Rsh/a fields + note show only for Indonesia/Simandoux and hide again on switch back,
and the selector lives inside the conductivity-gated fluid box (so it's present exactly when Rt is). tsc 0.

Still to come (the 4th option you picked — "all of them"): the **in-inversion non-linear dual-water**
(Gauss-Newton, honours m and n separately). It'll drop into this same dropdown when it's ready.

- [ ] **Pick your Sw equation.** Open SandiMin ▸ Fluid: confirm the "Sw equation" dropdown, that
      choosing Indonesia/Simandoux reveals Rsh + Archie a, and that Rsh prefills 4.0 (**set it from a
      shale pick — a too-high Rsh inflates Sw**, wrong-way for fresh-water LRLC pay).
- [ ] **It changes Sw, not porosity.** On a well with CT + an HC component, run once on Linear, then
      again on Indonesia (or Simandoux) with your Rw/Rsh — confirm SWE/SWT move to the shaly-sand answer
      while PHIE/PHIT come out identical to the Linear run.

## Round 17 — SandiMin saturation models: linear dual-water + Indonesia + Simandoux (your request, increment 3a) (2026-07-23)

You asked for a selectable conductivity/Sw equation, "linear and non linear," because it's significant
to the wet/dry clay framework. This increment lands the **backend + math**; the Fluid-tab selector that
exposes it is the next increment (3b), so there's nothing to click yet — this entry is for the record.

What's in the solver now (`src-tauri/src/multimin2.rs`), all behind a new `sw_model` request field that
**defaults to `linear_dw`, so every run you've already reviewed is byte-for-byte unchanged**:

- **Linear dual-water** (default) — the existing in-inversion `Ct^(1/w) = Σ v·C^(1/w)`, `w = 0.75m+0.25n`.
- **Indonesia (Poupon-Leveaux 1971)** — effective-porosity form `1/√Rt = [Vsh^(1−Vsh/2)/√Rsh + √(φe^m/(a·Rw))]·Sw^(n/2)`.
- **Modified Simandoux (Bardon-Pied)** — `1/Rt = φe^m·Sw^n/(a·Rw·(1−Vsh)) + Vsh·Sw/Rsh` (closed-form quadratic at n=2, bisection otherwise).

Both shaly-sand forms are **post-solve**: the mineral inversion runs as usual (the deep-conductivity tool
stays in, so the solve stays well-posed), then Sw is replaced by the closed form using the solved effective
porosity and shale volume, and the U-zone water/HC split is redistributed to honour it — **φe and hard unity
are preserved**, so only SWE/SWT/SXOT change, never PHIE. New fluid inputs `Rsh` (shale resistivity, default
4.0 ohmm) and Archie `a` (default 1.0) feed the shaly-sand forms; the dual-water model ignores them.

Adversarially reviewed (3 lenses — equation transcription, solver integration, contracts). Confirmed the
equations against the standard references and the linear default as unchanged; fixed a real defect (a
shared-zone fluid would be double-scaled by the U- then X-zone override → silent PHIE/unity corruption; now
the flushed override runs only on a zone-disjoint split) and hardened the tests (added an **independent**
hand-computed Archie/shale check so a transcription error fails rather than being self-confirmed by the
round-trips). cargo 288/0.

- [ ] *(No click-through yet — UI is 3b.)* When the selector ships, the check will be: on a fresh well pick
      **Indonesia** or **Simandoux**, set Rw/Rsh, Run, and confirm SWE moves to the shaly-sand answer while
      PHIE/PHIT stay exactly as the linear run produced them.

## Round 16 — SandiMin dialog polish: theme parity + shrinkable/scrollable lists (your review) (2026-07-23)

Two of the three things you flagged on the tabbed SandiMin pane (the third — the conductivity/Sw
equation selector — is a separate, larger change I'm holding for your model choice):

- **Theme parity.** The pane's inputs, selects, and checkboxes were rendering as raw browser
  controls (white box, OS-blue tick) instead of the themed look every module pane uses (your
  image 2, the Porosity Ceiling pane). They now use the brand surface — `--bg-app` fields with
  `--border`, and checkboxes/radios take the theme accent instead of OS blue — so the whole pane
  reads one theme. Scoped to SandiMin for now; a one-line global rule would fix every other pane's
  checkboxes the same way if you want that (say the word).
- **Shrinkable + scrollable lists.** The mineral list is now three collapsible groups —
  **Minerals** (open), **Clays** and **Fluids** (collapsed by default) — each capped-height and
  scrollable, with a live `selected/total` badge on the head. The **Log inputs** list is likewise
  one collapsible, scrollable group with a `on/total` badge. Click any head to shrink/expand.

Browser-verified: the four groups render with correct open/collapsed defaults and counts
(Minerals 1/4 open, Clays 1/2 collapsed, Fluids 2/3 collapsed, Log inputs 5/16-on open), clicking
a head toggles both the collapsed state and the body, and the themed fields/accent resolve to the
active theme's variables. tsc 0.

- [ ] **The pane matches the app theme.** Open SandiMin: the mineral checkboxes, the endpoint
      inputs, and the fluid/clay fields should look like the Porosity Ceiling pane — brand accent
      ticks, themed field backgrounds — not white boxes with blue ticks.
- [ ] **The lists shrink and scroll.** On **Minerals**, confirm Clays/Fluids start collapsed and
      click their heads to expand; on **Log inputs**, confirm the 16-row list scrolls within its
      box. The head badges should track what you've selected/turned on.

## Round 15 — SandiMin dialog: tabbed setup (your request) (2026-07-23)

The Mineral Solver pane was one long scroll — minerals, log inputs, fluid properties, and the
clay converter all stacked. It's now **tabbed**: **Minerals** (component selection + presets +
the endpoint matrix), **Log inputs** (the tool list + user-defined inputs), **Fluid** (Rw/temps/
m/n/mud + precalc autofill), and **Clay** (the wet→dry converter). The run controls — well scope,
output prefix, unity/reconstruction toggles, the Run button and the results/QC — stay in a
**persistent footer below the tabs**, so you set things up across tabs and run from anywhere
without losing your place. The Fluid tab shows a short hint (instead of going blank) when no
conductivity tool is active, since the fluid numbers only matter to CT/CXO. Nothing about the
solve, endpoints, or wiring changed — this is purely how the pane is organized. Browser-verified:
tab switch shows exactly one panel, the CT toggle flips the fluid hint/grid, the footer stays put.

- [ ] **The pane is easier to navigate.** Open SandiMin (Modules ▸ SandiMin): confirm the four
      tabs across the top, that clicking each shows only that section, and that the Apply-to-wells
      scope + Run button + results stay visible no matter which tab you're on.
- [ ] **Nothing regressed.** Set up a clastic run as before — pick minerals on **Minerals**,
      confirm your tools on **Log inputs**, set Rw/temp on **Fluid**, Run from the footer — and
      check you get the same curves and DOF/incoherence readout as before the reorganization.

## Round 14 — Saturation-height solvers: Thomeer, log-driven Leverett-J, per-rock-type laws (playbook #4, increment 4a) (2026-07-23)

The SHF fitting engine now covers all five families and can split by rock type. **Thomeer** joins
the height-domain forms: the carbonate-standard hyperbola `Sw(H) = 1 − (1−Swirr)·exp(−G/log10(H/Hd))`
(Thomeer 1960), fitted with the same bounded simplex as Skelt — Hd is the entry height (the
displacement pressure expressed in metres above the FWL) and G the pore-geometrical factor
(≈0.1 well-sorted → >2 poorly sorted). **Leverett-J now fits from logs**, not only at SCAL
import: each sample's height becomes reservoir Pc (0.433·Δρ·h_ft), J = 0.21645·Pc/σcosθ·√(k/φ)
from the PERM/PHIE curves, and Sw = A·J^B is regressed in ln-ln space (Leverett 1941). Fluid
defaults are Tier-A seeds — σ·cosθ 26 dyn/cm (IP cap-pressure table, Water-Oil 30 dyn/cm·cos 30°),
HC density 0.7 g/cc (Techlog) — all per-run overridable. **Per-rock-type fits**: hand any family
an RT/facies curve and it fits one law per rock-type class alongside the pooled law (the single
biggest SHF accuracy win on stacked Mahakam sands); classes that can't fit are reported with the
reason, never dropped. **Nothing is dropped silently anymore**: every excluded sample is counted
by reason (Sw > 1, at/below the FWL, below the φ cutoff, no permeability), scoped wells that
contributed zero samples are named in a note, and a Buckles check (Buckles 1965) flags when the
above-transition BVW isn't one constant — the classic sign you need per-rock-type laws. The
breakdown survives even when the fit itself fails — that's when you need it most.

Adversarially reviewed (37 agents, 4 lenses → 3-skeptic verification): 8 confirmed findings → 4
distinct defects, all fixed pre-commit — a Thomeer bounds panic on sub-millimetre height ranges
(HIGH), silent zero-contribution wells, discarded exclusion counters on two FOIL error paths, and
the failed-group NaN→null IPC contract. cargo 283/0, tsc 0. (Dialog UI for all of this = 4b, next.)

## Round 15 — Saturation-height dialog: 5 families, per-rock-type tabs, draggable FWL (playbook #4, increment 4b) (2026-07-23)

The Saturation-Height dialog now drives everything the 4a solvers added. The **SHF-form dropdown
has five entries** (FOIL / Brooks-Corey / Skelt / Thomeer / Leverett-J); picking **Leverett-J**
reveals a permeability-curve picker and a fluid-property block (system dropdown that flips σ·cosθ
between the Water-Oil 26 and Water-Gas 50 dyn/cm Tier-A seeds, plus ρw/ρhc — all editable). A
**"Fit per rock type" checkbox + RT-curve picker** turns any family into per-class fits: the
results panel grows a **tab strip** (All / RT 1 / RT 2 …), each tab showing that class's
parameters, R², and its own Sw-vs-height curve; classes that couldn't fit show a ⚠ tab with the
reason instead of vanishing. Every result now carries a **diagnostics line** — the excluded-sample
breakdown (Sw > 1, at/below the FWL, φ-cutoff, no-perm counts) and the honesty notes (zero-
contribution wells, the Buckles warning) — shown on both success and failure. The **FWL is
draggable**: drag horizontally on any result plot to nudge it (0.2 m/px) and it re-fits on release,
or click straight on the FWL-scan curve to pick a candidate. An **RMS** row joins R² in every
parameter table. tsc 0.

- [ ] **All five families fit.** Analysis ▸ Saturation-Height on BLSO: run each of FOIL,
      Brooks-Corey, Skelt, Thomeer, Leverett-J. Thomeer and Leverett-J should return sensible
      params (Thomeer G ~0.1–2, Leverett B negative) with a curve through the Sw-vs-H cloud.
- [ ] **Leverett-J uses PERM.** Pick Leverett-J → the PERM picker + fluid block appear; switch
      the system Water-Oil↔Water-Gas and watch σ·cosθ flip 26↔50. Fit with your PERM curve.
- [ ] **Per-rock-type split.** Tick "Fit per rock type", pick your RT curve, fit: a tab per RT
      class appears, each with its own law + curve; a thin class shows a ⚠ tab with the reason.
- [ ] **FWL by drag / click.** Drag left-right on the crossplot — the status shows the trial FWL
      and it re-fits on release; on FOIL with the scan on, click the scan curve to jump the FWL.
- [ ] **Nothing hides.** Set the FWL above the whole cloud and fit: the failure now shows an
      "Excluded: at/below the FWL: N" breakdown instead of a bare error.

## Round 14 — Saturation-height solvers: Thomeer, log-driven Leverett-J, per-rock-type laws (playbook #4, increment 4a) (2026-07-23)

## Round 13 — Theme sweep: canvas typography + color tokens (playbook #9A, increment A) (2026-07-22)

Every canvas font and the last hard-coded colors that bypassed the theme are now driven by the
theme system, so plots, dialogs, and overlays stay legible and on-brand across all eight skins
(light / dark / Pertamina / Halliburton / Schlumberger / LAPI-ITB / white / system). An inventory
workflow (4 parallel sweeps) found **111 bypasses across 20 files**; all fixed. New tokens:
`--font-canvas` (the Segoe-variable stack) and `--font-mono` in styles.css, a `canvasFont(theme,
size, weight)` helper on the shared plot scaffolding (`PlotTheme` gained `fontFamily`), so all
~55 `ctx.font` literals now resolve through one token. Color fixes: the well-diagram casing
strings/shoes (was mid-gray `#5a5a5a`/`#333` — invisible on dark) now use `--text`; perforation
ticks use `--warn`; the crossplot/Pickett "no-data" gray marker now derives from `--text-dim`;
the highlights default palette and the "Add curve" default color are built from the live theme
accents instead of fixed light-theme values. Browser-verified across all six branded palettes:
6 distinct accents, 6 distinct no-data markers, the font token resolves and stays stable, and the
derived palettes are all valid hex (safe for the color pickers). tsc 0, production build clean.

- [ ] **Themes stay legible everywhere.** Cycle the theme (ribbon ▸ theme) through dark and a
      client brand (Pertamina/SLB) with a log view, a crossplot, and the well-diagram track open:
      axis/label text, casing strings + perforations, and crossplot no-data points should all
      stay readable — nothing washes out or disappears the way the old mid-gray casing did on dark.
- [ ] **New curves + highlights adopt the brand.** On a branded theme, add a curve in Layout
      Properties and drag a highlight band: both should come up in the theme's accent, not the
      light-theme terracotta.

## Round 12 — Monte Carlo sampling engine: LHS, rank correlation, convergence (playbook #1, increment 1.1) (2026-07-22)

The Monte Carlo engine's draw generation is rebuilt to commercial grade. **Latin Hypercube
Sampling is now the default**: each parameter's probability range is split into N equal strata
with one jittered draw per stratum (order shuffled per parameter), so the sampled CDF matches the
distribution far tighter than independent draws at the same N — P10/P90 bands stabilize with
fewer iterations (McKay–Beckman–Conover 1979). The old scheme survives as `sampling: "random"`
and reproduces pre-upgrade results byte-for-byte at the same seed. Two new opt-ins: **parameter
rank correlations** (Iman–Conover 1982 — e.g. tie RHO_MA to GR_MA at ρ 0.7; marginals are only
reordered, never altered, and inconsistent/unknown pairs come back as notes, not errors) and a
**convergence check** (running P10/P50/P90 of total HPV per batch; in random mode the run stops
early once the trace goes stationary — LHS always runs its full design, since truncating one
would leave strata unsampled). `montecarlo.rs` + `ipc.ts`; 5 new tests (legacy request shapes parse with LHS defaults;
exactly-one-draw-per-stratum + analytic mean; achieved Spearman hits ±targets and marginals are
pure reorderings; flat series early-stops with a consistent truncated result; LHS never
truncates). cargo 274/0, tsc 0. The LHS/random toggle, correlation editor, and convergence
sparkline arrive in the dialog with increment 1.3 — until then the pane simply runs LHS.

Adversarially reviewed (18-agent workflow, 4 lenses × 2 skeptics); all 4 confirmed findings fixed
in the same round: (1) correlation targets are now pre-adjusted by the Spearman→Pearson map
2·sin(πρ/6), so the achieved rank correlation centers on your ρ instead of landing ~0.014 low;
(2) a duplicated/conflicting correlation pair now reports "last entry wins" in `notes` instead of
resolving silently; (3) the convergence trace folds the remainder into its final batch, so the
end-of-run "converged" verdict can't be inflated by a runt 4-realization checkpoint; (4) a
**pre-existing tornado bug**: with a zone that has no pay at the parameter medians, switching the
sensitivity metric to Avg PHIE/Avg SWE crashed the pane (`null.toFixed`), and a single dry sweep
endpoint drew a bar anchored at a fabricated 0 — the renderer now says the base case has no
anchor and drops non-finite endpoints.

- [ ] **LHS is quietly better, not different.** Monte Carlo pane ▸ your usual GR_MA/RHO_MA setup on
      a real well, 1 000 iterations, seed 42 ▸ Run twice — identical results (reproducibility
      holds). Then drop to 300 iterations and re-run a few seeds: the P10/P90 HPV band should sit
      noticeably steadier across seeds than you remember from the old sampler at 300.
- [ ] **Dry-zone tornado no longer crashes.** Monte Carlo ▸ tornado on ▸ pick a marginal zone that
      has no pay at your median cutoffs ▸ after the run, switch the sensitivity Metric to
      Avg PHIE: you should get the "base case yields no Avg PHIE" message (previously this threw
      `TypeError … toFixed` and left a half-drawn panel).

**Increments 1.2 + 1.3 (same round):** distributions can now be **zone-scoped** — each uncertainty
row has a zone box (suggestions from the scoped well's zonation); a scoped draw applies only inside
that zone, everything outside follows the deterministic zone parameters, and the tornado/Spearman
rows are labeled `PARAM @ ZONE`. **Save LOW/BASE/HIGH curves** writes per-sample uncertainty curves
to a fresh **version** of the MONTECARLO log set per well (never overwrites — the Sets manager can
restore any run): `MC_<KEY>_LOW/_P50/_HIGH` are per-sample percentiles across realizations and
`MC_<KEY>_BASE` is one deterministic run at every parameter's median, for each of VSH/PHIE/SWE/PERM
the chain produces. The dialog grew the **Sampling** select (Latin Hypercube default / Random
legacy), the **Correlations** mini-editor (param ↔ param, ρ), **Convergence check** and **Save
curves** checkboxes, a per-well **convergence sparkline** (running P-low/P50/P-high with a
converged/not-converged badge), and a notes panel that surfaces backend advisories (skipped
correlation pairs, persist confirmations). Status line reports sampling, early-stop count, and
saved-curve count. 5 more cargo tests (zone-scoped spread stays in its zone + unknown-zone note;
persisted curves ordered LOW ≤ P50 ≤ HIGH and versioned v1→v2; inverted zone; input-skip;
stale-family reclaim + degenerate base). Browser-smoke-tested end-to-end.

The 1.2 backend was adversarially reviewed too (27-agent workflow); all 7 distinct confirmed
findings fixed before commit: an inverted zone (top ≥ bottom, storable via the DB inspector) now
yields a note instead of **panicking the whole run**; correlations naming a parameter that appears
in several zone-scoped entries note that ρ binds only the first; persisted curves are gated on
what the chain **produces** (inputs it merely consumes no longer come back as zero-width fake
uncertainty bands); the kept-snapshot pool survives convergence early stops (first-N prefix
instead of a precomputed stride); a re-run that writes fewer curve families reclaims the previous
version's stale MC_* rows from the current store (archive keeps every version restorable); a
degenerate all-median base run skips only MC_*_BASE with a note instead of discarding the valid
percentile curves; and a well whose persist write fails now finishes its job item **Warned**, not
Ok.

- [ ] **Zone-scoped uncertainty stays in its zone.** Monte Carlo ▸ add GR_MA, type a real zone name
      in its zone box (the box suggests your zones) ▸ Run: the named zone's P10–P90 band spreads,
      every other zone's collapses to a single value, and the tornado row reads "GR_MA @ <zone>".
- [ ] **Saved uncertainty curves land as a versioned set.** Tick "Save LOW/BASE/HIGH curves" ▸ Run
      ▸ open a layout and add MC_PHIE_LOW/P50/HIGH on a track: a proper uncertainty envelope
      around the P50, with MC_PHIE_BASE hugging your deterministic PHIE. Re-run — the Sets manager
      shows MONTECARLO v2 alongside v1.
- [ ] **Correlated draws + convergence read sensibly.** Add GR_MA and RHO_MA, correlate them at
      ρ 0.7, tick Convergence check, sampling Random, 5 000 iterations ▸ Run: the sparkline
      flattens and the run stops early with "stationary after N" (with LHS it always runs full
      size and says so).



Backend for the SandiMin reconstruction check. The existing **RECON** curve is now documented as the
**incoherence** — the σ-weighted RMS of (reconstructed − measured) over the live tool rows (Quanti.Elan
Eq 79). With the new **`recon_qc`** request flag the reconstruction is **decomposed per tool**:
`<prefix>_<KEY>_REC` = the log rebuilt from the solved volumes (in the tool's display units, so it
overlays the measured curve) and `<prefix>_<KEY>_DIF` = that tool's σ-unit residual (whose RMS over
tools is RECON). The result also reports model **degrees of freedom** `dof = (tools + soft + unity) −
components`, with a note when `dof == 0` (exactly determined → RECON is forced to ~0 and can't validate
the model). `multimin2.rs` + `ipc.ts`; 2 new tests (a forward-modeled 3-mineral well reconstructs to
incoherence ~0 and a wrong illite density inflates it + localizes to the density residual; the
exactly-determined case flags its note). cargo 269/0, tsc 0. **The recon-QC view shipped in the same
round (increment 2d):** a **Reconstruction QC** checkbox in the SandiMin dialog turns the per-tool
curves on; after the run the result shows the **model DOF** (with the exactly-determined warning) and a
**measured-vs-reconstructed crossplot** (each tool min-max normalized, points on the dashed 1:1 line =
a perfect fit, scatter off it = that tool's incoherence). Browser-smoke-tested: checkbox → run → DOF
line + crossplot render.

**Increment 2c** completed **#2** per your call to keep smectite as-is: a **Preset** selector atop the
component picker with four named GROUPINGS of existing library components — **Clastic**
(quartz–illite/kaolinite–water+bound), **SSC-style** (quartz–feldspar–clay, to compare VOL_* against
the SSC module's VSAND/VSILT/VCLAY), **Carbonate** (calcite–dolomite–anhydrite), **Organic/coal**
(quartz–illite–coal–kerogen, whose VOL_KEROGEN feeds the upcoming unconventional workflow). Presets
carry **no endpoint values** — Montmorillonite keeps RHOB 2.63 etc., so no reviewed number changed;
manually ticking a component drops back to "— custom —". Browser-smoke-tested all four.

- [ ] **Presets assemble the right model.** SandiMin ▸ Preset ▸ each of the four: the component
      checklist follows the grouping (note under the selector explains each), endpoints stay exactly
      what the library/your edits hold, and a manual tick resets the selector to custom. Run the
      Clastic preset on a Mahakam well and sanity-check VOL_QUARTZ/VOL_ILLITE against your SSC results.

- [ ] **Reconstruction flags a bad model.** In **SandiMin ▸ tick "Reconstruction QC" ▸ Run**. On a
      good model the crossplot points hug the 1:1 line and the incoherence stays low; force a wrong
      endpoint (or drop a needed mineral) and confirm the points for the broken tool scatter off the
      diagonal and the incoherence rises. The written `<prefix>_<KEY>_REC` curves can also be laid over
      the measured logs in a log view for a depth-by-depth check.
- [ ] **DOF honesty.** Build a model with exactly as many inputs as components (e.g. 3 minerals, 2 logs
      + unity). Confirm the dialog shows **DOF 0** in orange and warns that RECON can't validate the
      model; add one more input log and DOF rises to 1 (RECON becomes meaningful).

## Round 10 — Stratigraphic Modified Lorenz Plot: flow-unit solver (playbook #3, increment 3a) (2026-07-22)

New backend `lorenz.rs` — the **Stratigraphic Modified Lorenz Plot** (Gunter et al. 1997, SPE 38679).
It walks a well's φ + k logs in **depth order**, accumulates flow capacity Σ(k·h) against storage
capacity Σ(φ·h) (each normalized 0..1), segments the depth-ordered log10(k/φ) profile into **flow
units** with an exact contiguous dynamic program (auto-K by marginal gain, or a caller-set K), and
reports the **Lorenz heterogeneity coefficient** (Schmalz & Rahme 1950). Command `run_lorenz` +
`runLorenz` in `ipc.ts`. cargo **265/0** (9 new `lorenz` tests, incl. a synthetic 3-flow-unit column →
3 units), tsc **0**. Adversarially reviewed (4 lenses → **1 confirmed** IPC-nullability fix applied;
math + segmentation lenses clean). Method banked in `docs/ref_rock_typing.md`.

The **visual** (increment 3c-1) shipped in the same round: new pane **Lorenz Plot (flow units)** in
the ＋ add-panel menu — well + φ/k curve pickers (group-filtered, defaults to the selected well;
PERM list prefers PERM/KLOGH/PERM_RT), auto or forced K, optional MD window, then the SMLP curve
coloured by flow unit against the dashed 45° homogeneous diagonal, the per-unit table (top/base,
storage %, flow %, slope, **speed/baffle** character), and the Lorenz-coefficient headline.
Browser-smoke-tested on a stubbed 3-regime column: 3 units recovered, unit 1 = speed with 90 % of
flow from 33 % of storage, row-click highlight dims the other units.

**Increment 3c-2** completed **#3**: (a) a **Winland/Pittman pore-throat grid** on the crossplot —
Crossplot Properties ▸ *Rock-type grid* draws iso-radius lines at the port-class bounds
(0.1/0.5/2.5/10 µm) when one axis is porosity and the other permeability (Kolodzie 1980 R35 or
Pittman 1992 r25/r35/r50); (b) the **facies tie-in now also reports k-variance-reduction** — how
much of the core log10(k) spread the predicted rock-type class explains (ANOVA 1 − SSw/SSt), so the
tie-in is validated against permeability, not just class purity; (c) **RT as a FACIES block track**
needs no new code — set any integer RT curve's fill to **Facies blocks** in the log-view layout
props. cargo 267/0, tsc 0. (3b, the Pittman full-apex r10–r75 table, was already the `pittman_rx`
module.)

- [ ] **SMLP + flow units on a real well.** On a well with PHIE + a permeability curve (imported
      KLOGH, computed PERM, or the rock-typing PERM_RT), open **＋ add-panel ▸ Lorenz Plot (flow
      units)** ▸ Build Lorenz Plot. Confirm the curve ends at (1,1), and steep **speed** segments
      coincide with your best reservoir sands (high k/φ) while flat **baffle** segments fall on
      shale / tight streaks — the flow-unit boundaries should track your net-sand tops.
- [ ] **Lorenz coefficient sanity.** A clean, well-sorted sand gives a **low** coefficient (near 0);
      a layered sand-shale interval a **high** one (→1). Use the MD window (a zone's top/base) to
      Lorenz two zones you know differ in heterogeneity and confirm the number moves the right way.
- [ ] **Winland/Pittman grid on a φ-k crossplot.** New Crossplot ▸ X = PHIE, Y = a permeability
      curve (log Y on) ▸ Properties ▸ **Rock-type grid = Winland R35** (or a Pittman rX). Confirm the
      dashed iso-radius lines (0.1/0.5/2.5/10 µm) fan across the cloud and your best plugs sit in the
      macro/mega band. Flip the axes — the grid should still draw (orientation auto-detected).
- [ ] **Facies tie-in explains permeability.** On a well with a core-derived RT + a log RT and core
      k, run **Facies Tie-in**. Besides purity, confirm the **k variance reduction %** appears and is
      high when the classes separate core k, low when they don't (needs core plugs within 1 m of the
      log samples).

## Round 9 — Cross-feature fix: survey TVD/TVDSS must not shadow an imported one (2026-07-22)

A cross-feature adversarial review of the four shipped feature_work commits (constants/TVD/ML-MASK/
DLIS) found one real HIGH seam bug between TVD materialization (Round 6) and the standard→computed→
generic resolution order (Round 8): importing a deviation survey wrote a **computed** TVD/TVDSS, which
outranks the generic store, so it silently shadowed an authoritative TVDSS a user had imported from a
vendor LAS/DLIS — with a possibly wrong datum (no-KB wells fall back to a sea-level datum) or NaN
outside the survey's MD range, and no recourse via Promote (disabled on a "served by computed" row).
Fixed in `materialize_tvd_curves` (ingest.rs): it now only materializes a name the well does not
already resolve from an import, and clears any stale survey-derived computed curve so the import keeps
winning. cargo 256/0, tsc unchanged. Test `materialize_tvd_keeps_imported_tvdss_authoritative`.

- [ ] **Vendor TVDSS survives a survey import.** On a well that has a TVDSS curve from its LAS, import
      a deviation survey. Confirm the plots/modules still read the **imported** TVDSS (unchanged values,
      full depth coverage) — not a survey-derived one. TVD (if not imported) still appears from the survey.
- [ ] **Recompute is still safe.** Edit KB and run Data ▸ Recompute TVD/TVDSS. A well WITHOUT an imported
      TVDSS refreshes its survey-derived TVDSS; a well WITH an imported TVDSS keeps the imported one.

## Round 8 — DLIS/LAS mnemonic-shadow resolution in the Curve Catalog (2026-07-22)

When a DLIS and an LAS (or two DLIS runs) carry the **same mnemonic**, the Curve Catalog now
detects the collision, badges the resolver's current winner, and lets you **Promote** the one you
want or **Delete** a duplicate — without editing files. Backend `db.rs` (new `pinned` column +
promote/delete), resolver tiebreak in `equations.rs` + `curve_edit.rs`, frontend
`inspectorPanel.ts`/`ipc.ts`/`styles.css`. cargo 255/0, tsc 0. Adversarially reviewed (4 lenses →
**5 confirmed findings, all fixed**): the resolver no longer lets a pin leak across a family, and the
Catalog no longer claims a Promote "wins" when a higher-priority store actually resolves the curve.

- [ ] **Promote resolves a real same-mnemonic shadow.** On a well where a DLIS and an LAS both carry a
      **non-standard** mnemonic (e.g. `PEF`, `CALI`, `DTS`, or a core `PERM` with no computed PERM),
      open the **inspector ▸ Curve Catalog**: the two rows show **`resolves`** / **`shadowed`** badges.
      Click **Promote** on the shadowed one → it flips to `resolves` + `pinned`, and any plot/module
      reading that curve now picks up the promoted values. **Delete** the loser → the sibling resolves.
- [ ] **No false "it now wins" for standard logs.** For `GR / RES_DEEP / NPHI / RHOB / DT / SP`, the
      real curve is served from the standard log column, not the RAW catalog copy. Those rows now show a
      neutral **`served by log`** badge and **Promote is disabled** (tooltip: "resolution comes from the
      standard log column — promoting has no effect"). Previously Promote here claimed victory but changed
      nothing on any plot — that lie is gone.
- [ ] **No false win when a computed curve owns the name.** If you've computed a curve (say `PERM` from
      Coates) and also imported a raw `PERM`, the raw row shows **`served by computed`** and Promote is
      disabled — the computed curve resolves first, so promoting the raw one would have been a silent
      no-op.
- [ ] **A pin doesn't hijack the family (deep-R sanity).** Promoting one same-mnemonic shadow must NOT
      change which curve a **family** request resolves. On a well whose deep-resistivity feeds Sw, promote
      an unrelated same-mnemonic shadow and confirm Sw is unchanged (the pin now applies only to its own
      mnemonic, and family requests rank by base run — deterministic across re-import/reopen).

## Round 7 — MASK support in the ML pipeline (2026-07-22)

Optional flag curve in the ML dialog: samples where the mask = 1 are excluded from training AND left
blank (NaN) in the prediction — the same 0/1 convention as the module MASK. Backend `ml.rs` + frontend
`mlDialog.ts`/`ipc.ts`. cargo 253/0, tsc 0. Adversarially reviewed (3 lenses → 2 confirmed honesty
fixes applied).

- [ ] **Masked training + apply.** On a well carrying a BADHOLE / FLAG_PAY / COAL 0-1 flag curve, open
      ML Models, pick a **Mask (exclude)** curve, run a regression/classification → confirm the output
      curve is BLANK (NaN) at flagged depths and the per-well "Predicted samples" count drops.
- [ ] **Mask governs clustering/PCA too.** For an unsupervised task the mask keeps flagged samples out
      of the fit AND leaves them blank — facies with vs without a mask differ (bad-hole shouldn't shape
      facies).
- [ ] **Leaderboard honesty.** In **Compare algorithms** with a mask that empties a whole training
      well, the header shows the TRUE contributing-well count and a note that blind-well CV fell back
      to random KFold (previously it hid the collapse behind the requested well count).

## Round 6 — TVD/TVDSS as fetchable curves (2026-07-22)

Materialize the deviation survey onto the log depth grid as `TVD` and `TVDSS` computed curves,
so height-based tools can consume them by name. Backend `deviation.rs`/`ingest.rs`/`lib.rs` +
frontend `ipc.ts`/`ribbon.ts`. cargo 250/0, tsc 0.

- [ ] **Deviation import now writes TVD/TVDSS curves.** On a **deviated** well with logs loaded,
      Data ▸ Import Deviation… a survey → confirm `TVD` and `TVDSS` appear as computed curves
      (Curve Catalog / any module's log-input dropdown). TVD should be shallower than MD in the
      built section; TVDSS = KB − TVD.
- [ ] **`sw_height` TVD input now works.** Run the Saturation-Height module selecting the new `TVD`
      curve for the TVD input — on a deviated well the height (HAFWL) and SWH now use true vertical
      depth instead of MD (previously the TVD input was a silent no-op → MD fallback → optimistic pay).
- [ ] **SHF fits can use the materialized TVDSS.** In the Cuddy FOIL / Brooks-Corey / Skelt / Thomeer
      panes, pick the new `TVDSS` curve as the vertical-depth input and confirm the fit runs.
- [ ] **Correlation TVDSS depth-mode** now works from the survey (not only from an imported TVDSS log).
- [ ] **Data ▸ Recompute TVD/TVDSS Curves** — run after importing logs *after* the survey, or after a
      KB edit. Status reports "computed for X of Y surveyed well(s), N samples"; surveyed wells with no
      logs yet are counted as pending. *(Note: the survey-derived TVDSS is written to the computed store,
      which takes precedence over an imported TVDSS log of the same name when fetched.)*

## Round 5 — Rock-typing constants verification vs papers (2026-07-22)

Read-only cross-check of every hardcoded literature constant in `rocktyping.rs` / `shf_fit.rs` /
`thomeer.rs` / `hfu.rs` (+ `satheight.rs`) against `docs/research_2026-07/ref_rocktyping_shf.md` and
the published sources. Full write-up: `docs/constants_verification_2026-07-22.md`. **2 corrections
applied (both number-changing, Jauhar approved); 1 held pending a primary-source glance.** cargo
247/0, tsc N/A (no TS).

- [ ] **GHE FZI bins corrected** (`rocktyping.rs`). Was `…1.5, 2.5, 4, 6, 8`; now the Corbett-Potter
      2004 ×2 series `…1.5, 3, 6, 12, 24`. Run the **Rock Typing (FZI/R35/PGS)** module with
      `METHOD=ghe` on a cored well and confirm the `RT` (GHE class) curve looks right for the
      best-quality rock — high-FZI samples now land in the correct GHE6–GHE10 bands (previously
      compressed). `PERM_RT` follows the class, so it shifts too.
- [ ] **PGS definitions corrected** (`rocktyping.rs`). `PGEOM` is now `√(k/φ)` (was `k/φ`) and the
      `PS_EXP` default is `3.0` (was `3.5`) — the ACS Omega 2024 / Kozeny-Carman form. Diagnostic
      curves only (RT class is unaffected). Confirm `PGEOM`/`PSTRUC` plot sensibly; `PS_EXP` is still
      an editable param if you want a different exponent.
- [ ] **Pittman r75 — HELD (not changed).** The code's r75 row `(1.243, 0.674, −1.517)` diverges from
      the widely-cited `≈(0.778, 0.626, −1.205)` while r10–r50 all match. Couldn't confirm online
      (Pittman's Table 1 is an image; primary is paywalled). If you can check **AAPG Bull. v76 (1992)
      p191-198, Table 1**, tell me the r75 coefficients and I'll fix the one row. Only affects `PR75`
      and `RT_PITT` when APEX=r75 (default r35 is fine).

## Round 4 — AUDIT-2026-07-21 safe-bucket follow-through (2026-07-22): correctness / honesty / robustness

Continuation of task #159 (the 65-finding full-QC audit). After batches 1–3 (`1d6b521`/`5e44620`/`1dcfeba`)
and the RT≤0 fix (`f33e126`), this round works the remaining **safe** bucket — fixes that harden behaviour
or improve reporting honesty WITHOUT changing interpretation numbers for valid data. Audit references were
re-verified against CURRENT code first (several were already fixed by the round-2/3 refactors — e.g.
correlation already subscribes to dataVersion; recordProcess already wired in ML/multimin/inspector).
**cargo 247 pass / 0 fail / 7 ignored; tsc EXIT 0. Nothing committed.**

Backend (Rust, unit-tested):
- [ ] **Cutoff-sweep geometric clamp.** `run_cutoff_sweep` now integrates each sample's clamped overlap
      with the zone ∩ DST interval (mirrors `run_pay_summary`), so NTG can no longer exceed 1 when a
      zone/DST boundary lands mid-sample. Sample-aligned results are byte-identical. **Try:** run Cutoff
      Sensitivity with a DST interval whose edges don't fall on log samples — NTG should stay ≤ 1 and agree
      with the Pay Summary for the same well/zone/cutoff.
- [ ] **Per-well isolation** in `run_pay_summary` + `run_cutoff_sweep`: one well's fetch/zone read error
      now skips just that well instead of zeroing the whole Field Dashboard / sweep response.
- [ ] **All-NaN module runs report honestly.** A module run whose every output sample is MISSING (e.g.
      gascorr with no precalc, or a module fed an all-NaN input, or SW-RtC on a well with no PHIT) is now
      reported as an error / Warned in the Processing panel — not a green "N samples → …" success. Same
      guard on Rhai + Python equations (an unresolvable input/output curve → error, not a clean success).
- [ ] **Python in-place equation guard.** An equation whose output curve name collides with an input
      (a "clean this curve in place" script) no longer silently writes the untouched input back when the
      script forgot to (re)assign it. (Also fixed a worker crash when the output was named `np`/`numpy`.)
- [ ] **LRLC SSPW fallback.** SW-RtC / SW-IMTS now fall back to the SSPW-named curves (PHIT_SSPW /
      CAPBW_SSPW / CBW_SSPW) when the SSC ones are absent — so they run on an SSPW-processed well instead
      of silently producing all-NaN. SSC-only wells are unchanged. **Try:** run SW-RtC on a well processed
      through SSPW porosity (no SSC curves).
- [ ] **LAS duplicate-name warning.** Importing a LAS whose (normalized) well name already exists now
      warns (still creates a separate record — merge is a deliberate action, not automatic). **Try:**
      import the same LAS twice; the second shows a "already exists" warning.
- [ ] **New test coverage** (no behaviour change): phi_den / phi_dn edge cases (VSH≥0.95 shale branch,
      SHALE_REDUCED-vs-MAXIMUM cap, density shale-reduction clamp, AVERAGE-vs-GAS_RMS), SSC `*_GR` family
      closure + degenerate-VWSH guard, and `run_ml`'s DB-integration guards.

Frontend (TS, tsc-clean):
- [ ] **History attribution.** A scoped module run records the wells actually run (single by name, batch
      as null) instead of the globally-selected well (which a scoped run may not have touched).
- [ ] **Blank "(none)" for optional inputs.** Optional log-input dropdowns now offer "(none)" so you can
      deliberately drop a curve slot even when a curve of that name exists in the project.
- [ ] **dataVersion refresh** after equation / ML / report runs and on workflow-chain **cancel/fail**
      (a cancelled chain routinely committed the earlier wells) — open plots/log views no longer show stale
      curves.
- [ ] **Race guards** on the module pane's data refresh (a slow refresh can't overwrite a fresher one) and
      SandiMin's **Autofill-from-precalc** (a well switch mid-fetch no longer stamps stale FTEMP/RMF).
- [ ] **Pay Summary → Processing History** (the FLAG-writing Compute now leaves a trace); **curve-edit
      Set-constant** rejects non-finite (Infinity) input; the deprecated legacy **`multimin`** module is
      filtered out of the Workflow step picker (use SandiMin).

Deferred / needs your call (see the summary I sent):
- Report "Tables only" still computes the composite geometry (efficiency, not correctness) — a truly safe
  fix must reproduce the cover interval exactly, which needs the same expensive fetch. Held.
- Low-value polish left: MC histogram theme-repaint; ml/wellScope dataVersion subscribe.
- **4 findings that WOULD change interpretation numbers** await your sign-off (perm_coates default 100→70;
  phi_son OPT_CP DT_SH>100 gate; log_predict masked-fill survival; MC MASK/computed_only parity).
  - ~~legacy-multimin RECON_ERR at 3 tools~~ — **CLOSED 2026-08-01, it never needed your sign-off**
    (`docs/review_triage.md` finding 11). Legacy `multimin` is retired and refuses to start; the
    concern is inherited linear algebra rather than a bug — with as many equations as components the
    solve reproduces the measurements whatever the endpoints are — and SandiMin already DETECTS the
    condition (`dof == 0`) and returns `dof_note` saying RECON is forced to ~0 and to add an input
    log. Pinned by `an_exactly_determined_model_hides_a_wrong_endpoint_and_only_the_dof_note_says_so`.
    The one thing still worth your eye is a UI question: does the SandiMin pane make that note hard
    to miss? A warning nobody reads is the same as no warning.
  - ~~MC PERM cutoff when chain-produced~~ — **FIXED 2026-08-01** (finding 8). Not a judgement call
    after all: the cutoff was reading an external-input pool that a chain-produced PERM never enters,
    so adding a permeability model switched it off. The realization pool has the values.

## Round 3 — Feature Wave B chain (2026-07-22): fluid contacts, ML leaderboard, well-diagram, rock typing + SHF

Four Wave B features built back-to-back after the round-2 commit (`d64bdc7`). Each is tsc-clean and
either cargo-tested or cargo-check-clean; the novel math in each is unit-tested. **Not yet clicked
through in the real app with field data. Nothing committed.**

- [ ] **(9) Fluid contacts in Correlation.** New `fluid_contacts` store (well/field/global scope,
      OWC/GWC/GOC/GDT/ODT/FWL, depth, TVDSS flag, colour) + editor (Correlation ▸ **Contacts…**).
      Contacts draw as horizontal lines + cross-well connectors. New **MD / TVDSS depth mode** on the
      Correlation toolbar: in TVDSS a TVDSS-stored contact is **flat across every well** (converted per
      well via the TVDSS curve; falls back to MD == TVDSS for vertical wells). *(Verified: the TVDSS↔MD
      round-trip math — a TVDSS contact renders flat across two wells with different deviation, an MD
      contact flat only in MD mode; cargo check + tsc clean.)* **Try:** open Correlation, add an OWC as
      TVDSS, switch MD↔TVDSS, watch it flatten.
- [ ] **(3) ML comparison leaderboard.** In the ML pane (supervised tasks), a **Compare algorithms**
      button ranks every algorithm × a curve-subset strategy (full / leave-one-out / singles) by
      **blind-well GroupKFold CV** — whole wells are held out, fixing the depth-leak in the old random
      5-fold. Shows a sortable leaderboard (R²/accuracy + RMSE/macro-F1), **permutation importance** bars,
      and a **confusion matrix** for the selected row. *(Verified: 2 new Rust tests exercise the real
      sklearn GroupKFold path — blind-well R²≈1 for a linear law across 3 wells, 2×2 confusion for a
      classifier. Needs ≥2 train wells.)* **Try:** ML ▸ regression, pick ≥2 train wells + curves ▸ Compare.
- [ ] **(16) Well-diagram track.** Any layout track can be set to **kind = Well diagram** (Layout editor ▸
      Track type). It draws casing/tubing/liner strings (with shoe symbols) + perforation ticks from the
      well's **COMPLETION** and **PERFORATION** aux datasets (Data ▸ Import aux data; value_num = OD in
      inches, depth_top..depth_base = the run). Renders in the log view **and** the composite/report SVG.
      Old saved layouts still load (kind defaults to "curves"). *(Verified: cargo check + tsc clean;
      renderer skips curves for diagram tracks so nothing draws underneath.)* **Try:** import a COMPLETION
      CSV, add a track, set it to Well diagram.
- [ ] **(8) Rock typing + SHF — increment 1.** Two pieces:
      **(a) Rock Typing module** (Petrophysics ribbon ▸ new *Rock Typing* group) — from φ + k writes
      RQI, PHIZ, FZI (Amaefule), Winland **R35**, PGS **PGEOM/PSTRUC**, an **RT class** (GHE fixed FZI
      bins or Winland port classes) and **PERM_RT** (class-grouped geometric-mean-FZI perm estimate).
      *(4 unit tests: FZI→GHE7 for φ0.2/k100, Winland R35→macro, perm predictor, MISSING handling.)*
      **(b) Cuddy FOIL SHF fit** (workspace ▸ **SHF Fit (Cuddy FOIL)**) — pools computed PHIE/SW/TVDSS
      across wells, fits **BVW = a·H^b** above the FWL with a BVW-vs-H log-log crossplot, and an optional
      **FWL scan** (Cuddy 1993 Eq 19) that finds the common contact. *(3 unit tests: recovers a known
      power law, rejects degenerate input, scan lands on the true 2000 m contact.)*
      **NOTE (per the reference doc):** the PGS exponent (3.5) and GHE bins are literature/recall values —
      flagged in the module doc for verification before field release.
- [ ] **(8) increment 2 — first chunk (2026-07-22):** **Lucia Rock-Fabric Number** module
      (Petrophysics ▸ Rock Typing, carbonate) — inverts the Jennings-Lucia transform analytically for
      RFN + a 1–3 class; completes the FZI / Winland / PGS / Lucia rock-typing quartet. *(1 new test:
      Lucia round-trips RFN 1.0/3.0.)* **Try:** run it on a well with carbonate stringers. *(A Mahakam
      phi-k perm preset was built and tested but PULLED from the repo — those are proprietary Pertamina
      Hulu Mahakam production constants; kept out per the client-data rule.)*
- [ ] **(8) increment 2 — SHF forms (2026-07-22):** the **SHF Fit** pane got a form selector — besides
      Cuddy FOIL it now fits **Brooks-Corey** (Sw = Swirr + (1−Swirr)·(He/H)^λ, via a Swirr-grid + log-log
      linear fit) and **Skelt-Harrison** (Sw = 1 − A·exp(−(B/(H+D))^C), via a compact Nelder-Mead) to the
      log-derived Sw-vs-height cloud, with a Sw-vs-H scatter + fitted-curve overlay and a params/R² table.
      *(3 new tests: Brooks-Corey recovers a synthetic curve, Skelt reaches R²>0.98 + monotone Sw, both
      reject too-few points.)* **Try:** SHF Fit ▸ pick Brooks-Corey / Skelt-Harrison. *(Increment 2
      remainder — Thomeer Pc fit, SCAL importers, Pittman full rX table, and Ward/histogram HFU
      clustering — is now all shipped; see the entries below. Task #158 is complete.)*
- [ ] **(8) increment 2 — electrofacies tie-in (2026-07-22):** two parts. **Rock Type from Cutoffs**
      module (Petrophysics ▸ Rock Typing) — a Vsh + PHIE cutoff ladder → **RT_LOG** (1 best / 2 moderate
      / 3 non-net), to propagate rock types to uncored intervals. **Facies Tie-in** pane (workspace ▸
      *Facies Tie-in (RT confusion)*) — cross-tabulates the predicted log RT against a reference/core RT
      curve across wells and reports the **confusion matrix + dominant-class purity** (the check that
      the log classification faithfully reproduces core rock types). *(3 new tests: the cutoff ladder
      classifies clean/moderate/shaly correctly, the confusion tally scores purity, empty input is
      rejected.)* **Try:** run `rt_cutoff` to make RT_LOG, then Facies Tie-in ▸ RT_LOG vs your core RT.
- [ ] **(8) increment 2 — SCAL importers (2026-07-22):** **Import SCAL…** (Data ▸ Import Data) now
      takes **multiple files** and **three formats** (or **Auto-detect** per file): the existing flat
      PC/SW CSV, the **porous-plate wide table** (Corelab-style: preamble junk tolerated, pressure
      columns 1…150 psi as headers, one row per plug with Sample/Depth/Perm/Poro, cells = brine Sw
      %PV — unpivoted to long Pc points), and **centrifuge per-plug blocks** (SAMPLE/DEPTH/PERM/PORO
      key-value lines then a Pc/Sw table; several blocks per file, or multi-select one file per plug —
      the digitized-workbook shape). All selected files land in ONE combined replace-write of the
      well's `scal_pc` rows, then the Leverett-J fit runs over the pooled points as before. Lettered
      plug ids ("12A", "S-16A") keep their numeric part; %PV and %-porosity auto-convert; a bad file
      fails the whole import (nothing partial) and names the file. Also fixed on the way: a `PORO`
      header now resolves as porosity in every core/SCAL CSV import (it previously matched no alias).
      *(6 new tests: wide-table unpivot incl. a missing cell, headerless-file rejection, two-block
      centrifuge parse with no metadata leak between plugs, table-less block rejection, the format
      sniffer on all three shapes, multi-file import + replace-not-append + bad-file atomicity.)*
      **Try:** Import SCAL… ▸ multi-select your W-MND-1 porous-plate/centrifuge CSV exports ▸
      Auto-detect ▸ Import & Fit; then re-import to confirm points replace, and check SHF Fit sees
      the pooled cloud.
      **Post-review hardening (same day, ultracode 3-lens adversarial review — 10 confirmed
      findings, all fixed):** (1) an import that parses ZERO points now refuses the replace-write
      instead of silently wiping the well's existing SCAL data; (2) auto-detect no longer misroutes
      files whose cover sheets contain "No. of Samples,6"/"Sample Type,plug" lines — the centrifuge
      verdict now needs corroboration (a numeric DEPTH/PERM/PORO key-value line or a bare PC/SW
      header); (3) merged centrifuge files where the table header appears only above the first plug
      no longer silently drop plugs 2..N (header carries over); (4) repeated per-page header rows
      and numeric "Average" footers in wide tables no longer import as phantom Sw points (a data
      row must carry a sample id or depth); (5) regional Excel formats parse: ';' list separator
      (sniffed from line 1) and ',' decimals/thousands ("2,695.3", "98,5", "1,000"); (6) the flat
      parser keeps lettered plug ids ("12A"→12) like the other two. The dialog also now warns: ONE
      lab fluid system per import (mixed air-brine + mercury multi-selects would bias the pooled
      J-fit). *(+7 tests, suite 211 passed / 0 failed, tsc EXIT 0.)* **Deferred to the Thomeer /
      J-from-SCAL chunk:** a per-row fluid-system/IFT column in `scal_pc` (schema migration) so
      mixed-system imports can be stored and standardized properly, per the reference doc's long-
      table spec. *(→ delivered same day, see the Thomeer entry below.)*
- [ ] **(8) increment 2 — Thomeer Pc fit (2026-07-22):** new **Pc Fit (Thomeer)** pane (workspace ▸
      add pane). Fits the Thomeer (1960) hyperbola **Bv = Bv∞·exp(−G/log₁₀(Pc/Pd))** per plug over
      the scoped wells' imported SCAL Pc points (Bv = φ·(1−Sw); poro-less plugs are skipped and
      counted, not silently dropped). Per-plug table (row click selects) + the **Bv-vs-Pc QC plot**
      with the fitted hyperbola and Pd marker + the **Pd–G plane** — the Thomeer-class rock-typing
      crossplot. Also reports the Swanson apex (Bv/Pc)max and **Swanson k = 399·(Bv%/Pc)^1.691**
      (constants flagged: verify vs Swanson 1981 before field release, same policy as PGS). ONE
      pore system per plug this increment; multi-modal stacking (2–3 systems, dBv/dlogPc detection)
      is a later increment. **Schema:** `scal_pc` gained per-row **`system` + `ift`** columns
      (ALTER-migrated on old projects; the deferred review item) — the Import SCAL dialog now has a
      **Fluid system** select (air-brine 72 / air-mercury 367 / oil-brine 26 / custom) that
      auto-fills the sigma·cosθ and stamps every stored point. *(3 new tests: synthetic-hyperbola
      recovery pd/G/Bv∞ + R²>0.98, too-few/uninvaded rejection, DB-level grouping + poro-less skip
      + system/ift round-trip.)* **Try:** import MICP as
      Air-mercury ▸ Pc Fit (Thomeer) ▸ Fit — check the Pd–G clusters against your rock types.
      **Post-review hardening (same day, ultracode 2-lens review — 7 confirmed findings, all
      fixed):** (1) **Pc now standardizes to Hg-air equivalent (×367/σcosθ) BEFORE fitting** — the
      review caught Swanson k being applied to raw air-brine/oil-brine Pc (16–88× inflation) and
      the Pd–G plane mixing lab systems; G is scale-invariant so only Pd/apex move, and plugs from
      any system now share one comparable plane. Rows without a recorded σcosθ fit raw, show
      "(raw)" in the new System column, and get NO Swanson k. (2) Plugs group per **well_id** (two
      same-named wells no longer pool) and numbered plugs key on the sample number alone (blank
      depth cells no longer split a plug). (3) The long parser **forward-fills merged-cell plug
      context** (sample/depth/perm/poro on first row only — the common Excel export shape). (4)
      Entry-truncated curves flag **Pd ⚠ (pinned at a search bound)** instead of posing as resolved
      entries; plateau-only data no longer reports R²=0 for a perfect constant fit. (5) "Other"
      fluid system clears the σcosθ field (no stale preset silently stored). (6) perm/swanson_k
      typed `number | null` (NaN→null over IPC). *(+2 tests: air-brine plug recovers the same
      Hg-equivalent Pd as its mercury twin & legacy no-ift rows suppress Swanson; merged-cell
      forward-fill. Suite 216 passed / 0 failed; tsc EXIT 0.)*
- [ ] **(8) increment 2 — Pittman rX + HFU clustering (2026-07-22, closes task #158):** two pieces.
      **Pittman pore-throat radii** — new `pittman_rx` module (Petrophysics ▸ Rock Typing) writes the
      full **Pittman (1992) r10…r75** family (PR10…PR75 µm, each log₁₀ rX = C0 + C1·log₁₀ k + C2·log₁₀ φ%),
      an **APEX** selector (r10…r75, default r35) → **RAPEX** + its Hartmann-Beaumont **RT_PITT** port
      class. The r35 row (0.255/0.565/−0.523) matches the reference doc; the full table is transcribed
      from Pittman 1992 and flagged verify-before-release. **HFU Clustering** — new **HFU Clustering
      (FZI)** pane (workspace ▸ add pane). Reads the scoped wells' **core φ-k** (routine core analysis,
      not log curves), computes FZI, and partitions log₁₀(FZI) into K units by **Ward** (exact
      minimum-variance K-partition via DP — the global optimum, no greedy drift) or **histogram**
      (boundaries at the log-FZI histogram antimodes). Per-HFU table (FZI min/max, geometric-mean FZI,
      φ mean, and the Amaefule perm-transform R²) + the **RQI–φz** unit-slope crossplot coloured by HFU
      + the **log₁₀ FZI histogram** with the cut lines; row click highlights a unit. Read-only (writes
      no curves). *(10 new tests: Pittman r35 vs the published regression, apex-selector switching, Ward
      DP splits two separated bands + recovers each k, histogram finds the bimodal valley, invalid-plug
      skip + distinct-level cap note, empty-input error.)* **Try:** run `pittman_rx` (pick APEX) for the
      radius family; then HFU Clustering (FZI) ▸ pick Ward or Histogram + K ▸ Cluster — check the RQI–φz
      unit-slope lines and the FZI histogram breaks against your rock types.
      **Post-review hardening (same day, ultracode 4-lens adversarial review — 6 confirmed findings, all
      fixed; 2 refuted correctly):** (1) the **histogram path could emit an empty interior HFU**
      (two valleys flanking an empty bin gap) → non-contiguous ids like {1,3} and a boundaries/clusters
      count mismatch; ids are now remapped to contiguous 1..K and boundaries are recomputed from the
      final assignment (one cut per populated pair) for BOTH methods. (2) the selected-row highlight
      (`ml-diag`) was a no-op outside `.ml-confusion` tables → CSS broadened to cover plain mc-table
      selection rows (also repairs the Thomeer pane's identical latent no-op). (3) FZI_gm unit-slope
      lines now **clip to the plot rectangle** (a line whose slope-1 extension overshot could paint over
      the axis label/frame). (4) the pane now **redraws its canvases on resize** (was stale/blurry until
      a row click). (5) frontend histogram bins aligned to the backend clamp (8–40) so bars and cut
      lines share resolution. *(+1 regression test locking HFU-id contiguity across an empty gap. Suite
      227 passed / 0 failed; tsc EXIT 0.)*
- [ ] **Correctness — RT ≤ 0 → +Infinity in the Sw modules (2026-07-22, closes AUDIT-2026-07-21):**
      the three deterministic saturation modules (`sw_arch`, `sw_indo`, `sw_sim`) only screened
      **missing** RT (NaN). A genuine RT value **≤ 0** — almost always a null coded as `0`, or a bad
      processing artifact — flowed through: `sw_arch`'s `(a·Rw/(φ^m·RT))^(1/n)` and `sw_indo`'s
      `1/(RT·…)` both **diverged to +Infinity**, and since the "missing" test is NaN-only, +Inf leaked
      into the *unlimited* raw curves (`SWT_ARCH` / `SWE_INDO`) and **poisoned catalog min/max and plot
      autoscale** (the *limited* SWT/SWE looked fine because `limit()` clamps +Inf → 1.0, which masked
      it). `sw_sim` instead let the Newton-Raphson solver diverge and silently drop the sample. **Fix:**
      added `r <= 0.0` to each module's input guard, so an RT ≤ 0 sample is dropped to **missing (NaN)** —
      exactly matching the existing convention already used by `sw_rtc` / `sw_imts` (LRLC modules) which
      guard `rt_i <= 0.0`. *(Proven complete: an f32-sourced RT can't overflow f64 even at the smallest
      positive value, so no tiny-positive-RT can sneak a +Inf through; the LAS null −999.25 is negative
      → caught. Downstream contract verified safe — `classify_sample` already treats a missing SWE as
      "exclude from PAY", so a garbage RT that used to read as a fabricated `Sw=1.0` water sample now
      simply drops out; net pay is unchanged and average-SWE-over-reservoir is if anything cleaner.)*
      **Verification:** +3 regression tests (RT = 0 *and* −5 → NaN, never ±Inf, in all three modules);
      **suite 230 passed / 0 failed / 7 ignored**. Ran a 3-lens adversarial review (physics / downstream
      contract / edge-cases, 2 skeptics per finding, static-read only) → **0 confirmed, 7 refuted**.
      Two accurate-but-inconsequential observations were recorded, not fixed: *(i)* for the
      doubly-degenerate `(PHIE<0.005 AND RT≤0)` sample the porosity-state branch order makes `sw_arch`→NaN
      but `sw_indo`/`sw_sim`→SWE=1.0 (a non-reservoir sample excluded from pay either way; unifying it
      would mean restructuring `sw_arch`'s tested branch for zero benefit); *(ii)* `resolve_rw` could
      emit +Inf only at FTEMP = *exactly* −21.5 °C in the non-default MEASURED/SALINITY mode
      (physically impossible, pre-existing, orthogonal to this fix). **Try:** load a well whose deep
      resistivity has a zero/null streak and run `sw_arch` — the streak now reads as a gap in `SWT_ARCH`
      instead of pinning the curve autoscale to a huge number.
- [ ] **AUDIT-2026-07-21 full-QC triage — backend robustness batch 1 (2026-07-22):** a 65-finding
      parallel QC audit was triaged against current code (3 already fixed incl. the RT≤0 one above; 51
      safe-to-fix; 6 need your sign-off; 1 needs a live 100-well run; 4 feature-work). **This batch = 12
      safe backend fixes, none of which change any valid interpretation value** (suite 236/0/7):
      **(1)** `vsh_dn` now skips a **degenerate matrix/shale/fluid triangle** (`|c−d|<1e-6`) instead of
      writing ±Infinity into the unlimited VSH_DN (was poisoning catalog min/max + autoscale, same class
      as the RT≤0 bug). **(2)** `ftemp_grad` BHT mode skips a **TD_BHT ≤ 0** zone override (was a
      finite-looking ±Inf FTEMP). **(3)** `perm_wyllie_rose` now skips **negative PHIE** uniformly — the
      integer MORRIS_BIGGS/TIXIER exponent used to fabricate a plausible PERM from it while TIMUR NaN'd it.
      **(4)** `perm_transform` emits **MISSING instead of +Infinity** when `10^(PT_A·φ+PT_B)` overflows the
      f32 cast (reachable at in-range PT_A=100/PT_B=5). **(5)** `nphi_env_corr`'s FTEMP is now a
      **computed-only** input (a raw degF FTEMP can no longer be silently applied as degC), matching
      gascorr. **(6)** SandiMin **output prefix is upper-cased** so a re-cased prefix can't leave a stale
      curve. **(7)** the four computed-curve **delete-then-append writers now DELETE case-insensitively**
      (`upper(curve_name)`), closing the root-cause shadow-row bug where a re-cased equation output left a
      duplicate row that could silently win; the log-set restore subquery too. **(8)** curve-edit
      `locate_curve` got a deterministic `ORDER BY`. **(9)** **LAS export** looks up columns by upper-cased
      name, so a mixed-case computed curve ("Vsh_final") exports its real values instead of an all-NULL
      column. **(10)** Monte Carlo `summarize()` returns **NaN (→ "—")** for a dry/no-data metric instead
      of a fabricated 0.00. **(11)** the IMTS method doc's clay-term formula fixed to divide by Sw (matches
      code). *(+6 new tests locking the guards. No TS changed, so tsc unaffected.)*
- [ ] **AUDIT-2026-07-21 — import-robustness batch 2 (2026-07-22):** five importer fixes so one bad row no
      longer aborts a whole import, all mirroring existing verified patterns (LAS `depth_keep_indices`
      sanitize + the locations importer). **(1)** Core-CSV import **dedups duplicate plug depths** (first
      kept) instead of aborting the well's core import on the `core_data (well_id, depth)` PK. **(2)**
      Deviation-survey import **dedups duplicate station MDs**. **(3)** DLIS import **sanitizes each frame's
      depth** (drops non-finite + dedups) so one bad sample can't abort the file. **(4)** Tops import is now
      **transaction-wrapped** like the sibling Locations importer — a mid-file error no longer strands half
      the tops. **(5)** Tops import now **skips a blank WELL cell in a multi-well file** (was misrouting it
      to the selected well, silently attaching a top to an unrelated well) and reports the dropped count.
      *(+2 tests updated for the new `has_well_column` flag; suite 236/0.)*
- [ ] **AUDIT-2026-07-21 — dead-code removal batch 3 (2026-07-22):** deleted two dead source files and
      their IPC surface. **(1)** `petrophysics.rs` was fully dead (never declared as a `mod`, zero
      references; its math — linear Vsh, density porosity, plain Archie — is long since live in
      `modules.rs`). **(2)** `inversion.rs` was a hardcoded-stub solver (`run_stochastic_inversion`
      returned a fixed `[0.25,0.15,0.20,0.40]` regardless of input) still exposed over IPC as
      `start_inversion`/`get_inversion_status` with **zero frontend callers** and a latent
      `tokio::spawn`-from-sync-command panic; removed both commands from the handler, the
      `.manage(inversion::new_registry())`, the `mod`, and the file. *(No behavior change — nothing
      called either. Suite 236/0.)*

## Round 2 — panes, shift-select, MC plot props + table + polish (2026-07-21, Jauhar feedback batch #2)

Follow-up batch after the first round: (1) Shift-select was painting a native blue text
highlight; (2) the "4 main panes" clarification — they should always **STAY** (never vanish when
other panes pop/close) but stay manually resizable; (3) MC + other UI polish toward the **Cutoff
Sensitivity** panel look (image 3); (4) MC — add **plot property panels** (resize, colour, axes)
for the histogram + tornado, and make the histogram look like a **real histogram**; (5) MC — move
the **results table to the very bottom**. **tsc EXIT 0; browser-verified on an isolated vite (port
1428, never touched your 1420). Nothing committed.**

- [ ] **Shift-select no longer turns blue.** Range-select (Shift-click) was triggering the browser's
      native text selection across the well labels. Added `user-select: none` to the tree nodes and
      both tree bodies (Wells + Tops). *(Verified: `.tree-node` computes `user-select: none`.)*
- [ ] **The 4 anchor panes now STAY.** Wells / Tops / Processing / Performance can no longer be closed
      — the ✕ is hidden on their window header, Close panel/Close window are dropped from their
      right-click menu, and they can't be floated out of the sidebar. So opening/closing other windows
      can never make them disappear. They remain **freely resizable** (drag the splitter; the
      minimum-width floor only stops full collapse). A restored old layout that had lost the Wells pane
      re-adds it. *(If you'd rather they could still be closed, say so.)*
- [ ] **The anchor panes keep their WIDTH when other panes/windows pop up or close.** dockview lays out
      proportionally (that option is hardcoded on and not exposed), so opening/closing a pane was
      reflowing the sidebar. The fix pins each anchor group to a **fixed width (min == max)**, which
      dockview excludes from redistribution entirely — so no add, close, or window resize can move it.
      You can still resize it: grabbing the splitter (`.dv-sash`, caught in the capture phase so the
      drag goes live) unlocks the anchors for the drag, and they're re-pinned at the new width on
      release. *(Two earlier heuristic attempts — restore-on-layout-change — held on close but not on
      add, because an add fires extra reflow passes. This fixed-width approach needs no heuristic.
      Verified end-to-end against the real dockview build in isolation: add 4 panes → held 260; close 2
      → held 260; real DOM sash-drag → 340; add 3 more → held 340.)*
- [ ] **MC results table is at the very bottom.** Order is now **histogram → tornado → table**.
      Click a table row to plot that well-zone's HPV distribution in the histogram above.
      *(Browser-verified: the three result blocks render in that DOM order, table last.)*
- [ ] **Histogram is a real histogram now.** Added a frequency **y-axis** (nice-stepped count ticks
      0/20/40/… with a "count" title), horizontal **gridlines**, x-axis HPV min/mid/max labels, and the
      P10/P50/P90 markers. *(Browser-verified by capturing the canvas draw calls: count ticks + "count"
      + "HPV" + P10/P50/P90 all drawn; canvas re-rasterises crisply on resize.)*
- [ ] **⚙ Plot properties on both plots.** A gear on the histogram and the tornado opens an inline
      panel: **Height (resize)**, **colour** (bar colour / low-side + high-side bar colours), and
      toggles — histogram: P-markers, gridlines, y-axis; tornado: row stripes, ρ labels. Height 0 on the
      tornado = auto-size to the parameter count. *(Browser-verified: height 220→320 px live; bar colour
      set to #1f77d0 and the sampled bar pixel read back rgb(31,119,208).)*
- [ ] **MC UI polished toward the Cutoff panel.** Full-width brown **Run** button (matches Compute),
      `form-control`-styled selects/inputs, and tidier uncertainty-parameter rows (flexible param name,
      compact distribution pill). *(Browser-verified: Run button is full-width with the accent
      background.)*
- [ ] **Rw-for-PHIE gating still holds** after the tornado rewrite. *(Re-verified by capturing drawn
      labels: RW is drawn for HPV — it drives HPV via Sw — and dropped for Avg PHIE.)*

## Pane layout + MC/workflow polish + well-scope selector (2026-07-21, Jauhar feedback batch)

Jauhar's batch: (1) panes — two "Wells", tops-in-wells, non-resizable anchors; (2) MC — polish,
percentiles, table, ugly/stretching plots, and Rw showing sensitivity for PHIE it doesn't affect;
(3) workflow polish; (4) cross-cutting: stop checklisting wells one-by-one — use groups + pins.
**tsc EXIT 0; Rust montecarlo suite 7/7 (1 new: configurable percentiles); cargo check EXIT 0;
browser-verified on an isolated vite (port 1428, never touched your 1420) — see the proofs noted
per item. Nothing committed.**

### Panes
- [ ] **No more "two Wells".** The wells pane had a static "WELLS" title *and* the ObjectTree's own
      "Wells (N)" header — plus a **concurrent-refresh race** that appended the header (and every well)
      **twice**. Fixed both: dropped the static title; added a generation guard to `ObjectTree.refresh`.
      *(Browser-verified: 1 header, 9 well nodes — not 18 — for a 9-well group.)*
- [ ] **Tops is its own pane now.** Split out of the combined "Wells & Tops": a standalone **Tops** dock
      panel that follows the selected well through app state, docked directly below the **Wells** pane.
      It's a real dockview panel — drag it anywhere, tab it, resize it. *(Verified: panel list shows
      separate "Wells" and "Tops".)* Old saved layouts get the Tops pane auto-added on open.
- [ ] **Sidebar panes are resizable.** The Wells / Tops / Processing / Performance anchors were locked
      at a fixed width (min == max). Now they have a **minimum-width floor only** — drag the splitter to
      any width; they still won't collapse or auto-stretch when a neighbour closes. *(This reverses the
      earlier fixed-width lock, per your request — tell me if you preferred fixed.)*
- [ ] **★ pin a well** in the Wells pane (the star to the left of each name; persisted per project).

### Well scope — no more well-by-well checklists (imagine 2000 wells)
- [ ] Every run dialog (**Monte Carlo, Workflow, every module pane, Multimin, ML-apply, Cutoff,
      Summary, Report-batch**) now shows one compact **scope selector** instead of a checkbox per well:
      **Group** (defaults to the active group) · **★ Pinned** · **Selection** (your Ctrl-click set) ·
      **All** · **Custom…** (a searchable checklist for the rare precise pick), with a live "N wells"
      count. *(Verified: defaults to the active group and resolves 9 wells.)*
- [ ] Groups already existed and already scoped dialogs — the gap was purely the UI. **Pinned wells are
      new** (a `well_pins` table + ★ toggle) since a reusable pin-subset didn't exist before (the old 📌
      is only the workspace-follow toggle, unchanged). ML's *Train wells* and Auto-correlation's *targets*
      are deliberately **not** scope-swapped (they're a different concept, not "run on N wells").

### Monte Carlo
- [ ] **Rw no longer shows sensitivity for PHIE.** This was **not** a calculation bug — Rw is correctly
      routed only to the saturation step, so the PHIE *curve* is independent of it. The tornado was
      rendering statistically-insignificant **noise** (finite-N Spearman ≈0.05) and zero-width OAT rows.
      Fixed at the display layer, principled: a parameter appears for a metric **only if its one-at-a-time
      sweep actually moves that metric** (the sweep is deterministic → a non-contributor moves it by
      exactly 0), and ρ labels show **only above the significance floor** (1.96/√N). *(Browser-verified by
      capturing the canvas text: the tornado draws Rw for **HPV** — it does drive HPV via Sw — but **drops
      Rw for Avg PHIE**, while GR_SH/RHO_MA/NPHI_SH/GR_MA remain.)*
- [ ] **Percentile option.** A **Percentiles** dropdown in Settings (P10/P90 default, P25/P75, P5/P95,
      P1/P99) drives both the reported spread **and** the tornado's input sweep. *(Verified: switching to
      P5/P95 re-labels the table columns and the histogram markers.)*
- [ ] **Tidier table.** P50 as the headline number with the (P10–P90) band on a quiet sub-line, a new
      **Gross** column, tabular figures, zebra rows, and dynamic Pxx headers.
- [ ] **Plots don't stretch on pane resize any more.** Both the histogram and tornado canvases now
      re-rasterize to the pane's width via a ResizeObserver (before, the browser scaled a stale bitmap →
      the blur/stretch you saw). *(Verified: shrinking the pane redrew the bitmaps 618→484 px.)* Tornado
      also got rounded bars, alternating row shading, and a height that tracks the parameter count.

### Workflow
- [ ] Same scope selector replaces the well checklist; the rest of the builder (steps, grid, cons in/out)
      is unchanged.

## Monte Carlo parameter sensitivity + tornado (Wave B #13, 2026-07-21)

The uncertainty engine already ran N realizations but **threw away the parameter draws** — it only
kept the resulting P10/P50/P90. It now retains them and reports **which parameters actually drive
the result**. **tsc EXIT 0; Rust montecarlo suite 6/6 pass (3 new); off-by-default so existing runs
are byte-identical.** Nothing committed yet.

- [ ] **Open Monte Carlo** (Advance ribbon → Monte Carlo). There are two new checkboxes under a
      **Sensitivity** row — *Rank sensitivity (Spearman)* and *Tornado sweep (P10 / P90)*, both on by
      default. Add one or two uncertain parameters (e.g. GR_MA, GR_SH, RW), pick a well, **Run**.
- [ ] **Tornado chart** appears below the HPV histogram with a **Zone** and **Metric** selector
      (HPV / Net pay / NTG / Avg PHIE / Avg SWE). With the tornado box ticked it shows **range bars**:
      each parameter swept to its P10↔P90 with the others held at their medians, sorted most-influential
      on top, around a common **base** line, annotated with the Spearman ρ. Untick *Tornado* (leave
      *Rank sensitivity* on) → it falls back to **signed correlation bars** on a −1…+1 axis.
- [ ] **Sanity checks**: (a) the parameter you'd expect to matter most (usually GR_SH or Rw) sits at
      the top; (b) switching **Metric** re-sorts and re-scales; (c) switching **Zone** redraws for that
      well-zone; (d) a parameter you give **zero spread** (sd = 0) shows ρ = NaN / no bar (it can't be
      ranked); (e) unticking **both** boxes → no tornado section, and the headline P10/P50/P90 table is
      unchanged. Verified: Spearman sign+magnitude, tornado low≤base≤high ordering, and opt-out
      reproducibility are covered by unit tests; the live chart render awaits your click-through.

## Highlight tool + ribbon overflow + trademark scrub + typography (2026-07-21)

B2 UI/workflow polish + two follow-ups. **tsc EXIT 0; `cargo check --tests` EXIT 0; Rust 177 pass / 0 fail.** Nothing committed yet.

- [ ] **Ribbon overflow chevrons (Office-style)** (ribbon.ts, styles.css). When the window is too narrow
      to show all the tools on a tab, the raw scrollbar is gone — a boxed **‹ / ›** appears at the
      overflowing edge and scrolls the group row a page at a time (like PowerPoint's ribbon). Test: narrow
      the window until a tab's groups don't all fit → a **›** box appears at the right edge; click it →
      the row scrolls and a **‹** appears at the left; at the end only **‹** shows. Switch tabs / resize →
      the chevrons re-evaluate. (Verified live: at 720px the Petrophysics row overflows 238px, right
      chevron shows at scroll-start, left appears after scrolling, correct box at the right edge.)

- [ ] **Highlight tool — colored depth bands in the Log View** (new `highlightsOverlay.ts`; `highlights`
      table + `list/upsert/delete_highlight` in db.rs/lib.rs; IPC in ipc.ts). Open a **Log View**, then
      in that view's toolbar click **🖍** (next to the 🏷 tops button). Drag vertically over a depth
      interval → a **translucent colored band** appears across the tracks and an **Edit highlight**
      dialog opens. Give it a label (e.g. "Pay") + color → **Save**. Add a couple more with different
      colors. Test: (a) bands render across all tracks, translucent so curves read through; (b) they
      **track pan/zoom**; (c) switch to another well and back → bands **persist** and reload; (d)
      **double-click** a band → dialog to recolor / relabel / edit top+bottom / **Delete** / **Convert
      to zone**; (e) **Convert to zone** creates a zone (check it appears in **Zones** / pay summary);
      (f) **Ctrl+Z** undoes add / edit / delete / convert; (g) **🖍 and 🏷 are mutually exclusive** —
      turning one on turns the other off. Bands sit **below** the tops lines so tops stay legible.
- [ ] **Text sharpness — font hinting** (tauri.conf.json `additionalBrowserArgs`). You flagged text as
      slightly fuzzy/washed-out. I confirmed the CSS is clean and contrast is already high (~12.9:1), so
      it's not a color issue — the softness is Chromium's GPU grayscale AA (WebGPU forces GPU on) plus
      Windows display scaling. I added `--font-render-hinting=medium`. **This only takes effect on a full
      relaunch** (`npm run tauri dev` restart). Test: relaunch, eyeball the panel text vs before. If it's
      still soft, check **Windows Settings ▸ Display ▸ Scale** — at 125%/150% the webview raster-scales;
      tell me and we can add a text-size control or bump the base font. (Not verifiable from my side —
      the browser tools can't reproduce WebView2 rendering.)
- [ ] **AspenTech trademark scrubbed repo-wide (keeping Loglan)** (per your request — "except loglan").
      The prior-tool name is now gone from the whole tree — shipped app, code comments, and dev docs —
      except: **Loglan / `.lls`** (kept deliberately: SandiBumi runs Loglan, so those stay), your real
      data-folder paths in test fixtures (can't rename your disk), the English word "geology", and your
      own verbatim words in `Review.txt`. The comment/doc pass replaced the vendor name with neutral
      wording ("the reference suite", "commercial suite", etc.). Nothing to click-test — grep the repo
      for the old name and you'll only find the exceptions above. Test the one user-visible change: hover
      the **DB Inspector** ribbon button + open **Help** → reads "spreadsheet-style".

## 540-well test — perf & crash fixes (2026-07-21)

From your ~540-well stress test. A read-only 5-agent diagnosis traced every "not responding"
freeze to one root cause (heavy commands run **synchronously on the UI thread**) plus a specific
speed bug per subsystem. **Rust 176 pass / 0 fail; tsc EXIT 0. Nothing committed.** The async
piece can't be verified without running the app, so these especially want your click-through.

- [ ] **Field Dashboard no longer crashes on ~540 wells** (dashboardPanel.ts, summaryDialog.ts).
      "Compute failed: TypeError: Cannot read properties of null (reading 'toFixed')" was a zero-net
      zone row whose avg VSH/PHIE/SWE come back as NaN → serde encodes non-finite floats as JSON
      **null**, and the old `Number.isNaN` guard doesn't catch null. The formatter now shows "—" for
      null/NaN. Test: **Field Dashboard ▸ Compute** across all wells → the grid renders (empty
      aggregates show "—"), no crash. Same latent fix in the single-well **Cutoffs & Summary** table.
- [ ] **Field Dashboard is fast now** (workflow.rs `stats_only`). Compute took >5 min because it
      secretly wrote 3 FLAG_* curves per well (~1,600 DB transactions) on every press, though the
      panel only reads the returned numbers. It now computes the stats without persisting anything.
      Test: Compute on all wells → **seconds, not minutes**; tweak a cutoff and re-Compute → still
      fast. Behavior change to note: the dashboard no longer leaves FLAG_* curves in the wells —
      persist those from **Cutoffs & Pay Summary** (unchanged) when you actually want them written.
      Test `pay_summary_stats_only_persists_nothing`.
- [ ] **Workflow chain runs without freezing the app + live progress works** (lib.rs — `DbState` is
      now `Arc<Mutex<Connection>>`; `run_workflow_chain` now runs on a background thread). Build a
      chain (e.g. vsh_gr → phi_dn → sw_indo) and run it on a batch of wells: (a) the window stays
      **draggable/responsive** during the run (was frozen "not responding"), (b) the **progress bar
      advances** step-by-step, and (c) **Cancel** actually stops it. This is the first of the async
      conversions — import, dashboard, multimin, Monte Carlo and equations will follow the *same*
      pattern, so confirming this one works validates the whole approach.
- [ ] **A chain of many wells now finishes in seconds, and Cancel is near-instant**
      (workflow.rs two-phase batched write; equations.rs `create_log_sets_batch` +
      `write_computed_curves_versioned_batch`; chain.rs). The 30-min chain / 30-min-to-cancel
      was ~2 fsync-bound DB transactions **per well** per step (≈1,000 commits on 500 wells). Each
      step now computes every well in parallel (reads only), then does **one** batched versioned
      write — ~2 commits per step. Cancel is checked per well, so it drains in a well or two.
      Test: run a chain (vsh_gr → phi_dn → sw_indo) on a big well set → **seconds**, and **Cancel
      stops almost immediately**. Test `batched_module_run_writes_every_well_correctly` proves two
      wells write distinct, un-crossed, correctly-versioned results.
- [ ] **Cancel empties the progress bar** (workflowDialog.ts). Pressing **Cancel** now clears the
      bar to empty (and hides it) the instant you click, then the status confirms "Cancelled at
      step N". Test: start a chain, hit Cancel → the bar goes empty right away.
- [ ] **Input/Output are now "cons" (constellation) pickers, not free text** (workflowDialog.ts,
      moduleDialog.ts, new `list_log_set_names` command). Terminology changed from "set" to **cons**
      throughout the UI (Workflow, module dialogs, Curve Catalog "Constellations"). **Input cons** is
      a strict dropdown of existing constellations (blank = latest values — you can only read from
      one that exists). **Output cons** is an editable combobox: pick an existing constellation *or*
      type a brand-new name. Both are filled from the project's real constellation names. Test: open
      Workflow / any module → Input cons lists your existing constellations; Output cons suggests
      them but also accepts a new name like `FINAL2`.
- [ ] **Universal Processing panel — live per-well progress + Cancel for the whole run**
      (new `jobs.rs` registry + `list_jobs`/`cancel_job`; `processingPanel.ts`; ribbon **Processing**
      button). New dock panel that shows, for a running workflow chain: a **progress bar with an
      integrated Cancel**, the current **"Step 2/3: sw_indo"** line, a live **counts row**
      (▶ running · ✓ done · ⚠ warned · ✗ failed · ⏳ pending), and a **details** toggle listing the
      *notable* wells (running/warned/failed) with messages — so you can see **which well failed and
      why** at 500-well scale without a 500-row dump. It **auto-opens** when you press Run in the
      Workflow Builder, or open it anytime from the **Processing** ribbon button. Cancel here shares
      the *same* flag as the run, so it stops the chain whether launched from the panel or the
      builder. This is the reusable spine: import, module runs, multimin, Monte Carlo and reports
      will each report into it as they move off the IPC thread. Test: Run a chain → the Processing
      panel opens and fills live; click a well's ⚠/✗ in details to read the message; hit Cancel →
      the bar stops within a well or two.
- [ ] **Processing panel: the step-boundary "pause" now says what it's doing** (workflow.rs).
      Each chain step computes every well (bar fills), then does ONE big batched DB write with no
      per-well signal — so the bar used to sit at the boundary / 100% looking frozen. It now shows
      **"Writing N well(s)…"** during that write, so the wait reads as working, not stuck. Test: run
      a chain and watch between steps and at the end → the current line reads "Writing … well(s)"
      during the pause, then advances/completes.
- [ ] **Workflow Builder no longer shows its own redundant progress bar** (workflowDialog.ts). The
      inline `<progress>` bar is gone now that the Processing panel owns the live bar + Cancel; the
      builder keeps a one-line status ("Step 2/2: … — see Processing panel", "Done: …"). Test: run a
      chain → progress shows only in the Processing panel; the builder just shows a status line.
- [ ] **Hardware Health Monitor** (new `health.rs` + `health_snapshot` command; `healthPanel.ts`;
      ribbon **Health** button). A Petrel-PHM-style panel of four colour-coded gauges — **MEM System**
      (system memory %), **GPU Memory** (GPU video-memory current/budget %), **USER Objects** and
      **GDI Objects** (this process's handle counts vs the 10,000-per-process ceiling — the classic
      desktop-leak signal, raw count shown in the value). **Green < 60% · Yellow 60–80% · Red > 80%**,
      polled every 1.5 s. Open from the **Health** ribbon button (next to Processing). Metrics are
      Windows-only; any unavailable value shows **n/a** (so if GPU Memory reads "n/a" on your machine,
      tell me — the DXGI path is best-effort and I'll adjust). Note: this is GPU *memory* load, not
      engine-utilisation % (that needs PDH GPU counters — a possible refinement). Test: open Health →
      MEM/USER/GDI show live %; leave a few heavy panels open and watch GDI/USER climb.

## P0 senior-audit backlog — correctness & data-integrity fixes (2026-07-20)

The eight P0 findings from `AUDIT-2026-07-20.md` (ROADMAP §4b), plus the LAS-import
robustness residual (#118) that closing them surfaced, are implemented and unit-tested
(full lib suite **160 pass / 0 fail**, tsc clean). These are the ones that made answers
wrong or silently lost data, so they matter most for Mahakam work:

- [x] **MASK now blanks module INPUTS, not just outputs** (workflow.rs). Run **GR
      Normalize** (or **Log Predict**) with a BADHOLE / COND*FLAG curve set as the
      **Mask**. The well P3/P97 (and the KNN training set) are now computed from the
      unmasked samples only — casing/washout/hot-streak GR no longer shifts the two-point
      transform, so good-hole output stops drifting. For log_predict the repaired synthetic
      now survives \_inside* the masked (washout) interval it exists to fill, instead of
      being blanked there. Test: `mask_excludes_flagged_samples_from_gr_normalize_percentiles`.
- [x] **SW-height uses TVD and allows a subsea FWL** (satheight.rs). The sw_height module
      now takes an optional **TVD** input (defaults to measured depth when absent) and the
      **FWL** field accepts negative (subsea TVDSS) values. On a deviated well, height above
      the contact — and therefore SWH — is no longer optimistically overstated by ~1/cos(inc).
      Run on a deviated Mahakam well with the TVD curve mapped and confirm SWH rises vs the
      old MD-based result. Test: the negative-TVDSS deviated case in satheight.rs.
- [x] **Pay summary: thin-zone clamp + honest averages** (workflow.rs). Each sample's
      thickness is clamped to its overlap with the zone, so the last in-zone sample no longer
      bleeds a full step past the zone base and **net can never exceed gross** (sub-step-thick
      zones). SAND-row `avg_phie` is normalised over the thickness where PHIE is actually
      valid, so a sample with good VSH but missing PHIE no longer drags the average toward
      zero. Cross-check a well with thin zones / patchy PHIE against the old numbers. Test:
      `pay_summary_clamps_thin_zone_and_normalizes_avg_phie_over_valid`.
- [x] **Crash-safe curve writes** (db.rs `with_txn`). Every delete-then-append writer
      (computed curves, restore/delete log set, core/aux/SCAL/curve-sample/well-path inserts,
      group members, zones-from-tops) now runs inside a single BEGIN/COMMIT/ROLLBACK, so an
      app kill mid-write can no longer leave the DELETE committed but the re-append lost.
      Nothing to click — just note that a tauri-dev restart mid-run won't silently drop curves.
- [x] **IMTS clay-conductivity direction fixed** (lrlc.rs sw*imts). The excess-conductivity
      term now \_divides* by Sw (Waxman-Smits `Cw + B·Qv_eff/Sw`), so it grows as hydrocarbon
      displaces water instead of vanishing. IMTS SwE now sits at/just below Waxman-Smits in
      pay (the old `·Sw` form gave Sw^(n\*+1) and over-stated Sw exactly in the LRLC pay this
      method exists to find). Re-run sw_imts on an LRLC interval and confirm SwE dropped.
      Test: `imts_credits_clay_conductivity_in_pay_zone`.
- [ ] **DLIS null sentinels + no silent overwrite** (dlis.rs). DLIS absent/sentinel values
      (−999.25/−9999, non-finite, |v|>1e30) are screened to MISSING on import, and each DLIS
      frame gets its own run number so a frame-0 channel no longer silently replaces a
      same-mnemonic LAS curve — the status line reports "replaced N existing curve(s)" when a
      collision does happen. Re-import a DLIS over a well that already has LAS curves and check
      the replaced-count note.
- [ ] **SandiMin refuses under-determined models** (multimin2.rs). Selecting fewer than
      (components − 1) input tools is now rejected up front ("need at least N input logs to
      constrain M components") instead of solving to an arbitrary vertex, and per-sample the
      solver skips depths with too few live curves. Test: `rejects_underdetermined_request`.
- [ ] **LAS import survives duplicate/odd-depth files on BOTH stores** (parsers.rs, ingest.rs).
      Non-finite and duplicate depths are dropped (first occurrence kept) before insert, so a
      **spliced/merged LAS with a repeated depth section** imports instead of aborting on the
      `(well_id, depth)` PK — and the fix now covers the _generic_ store too (PEF/CALI/extra
      runs), which previously PK-failed silently and left the well without those curves. Extras:
      a Schlumberger **TDEP**-indexed (or any non-`DEPT`) file resolves depth via the first
      column; an auxiliary **MD/TDEP track** in a later column can no longer steal the depth
      role from the true first-column index; a file whose depth is entirely null now **errors
      cleanly** instead of creating an empty orphan well; and a non-monotonic depth (column 0
      wasn't really the index) is surfaced as an import warning. The dropped/duplicate/odd-index
      counts appear in the import status line and History. Import a re-spliced LAS and a TDEP
      file and confirm both load with the expected row counts. Tests:
      `duplicate_depth_las_imports_standard_and_generic_curves`,
      `all_null_depth_las_errors_without_creating_well`,
      `parse_las_2_auxiliary_md_curve_does_not_steal_depth`,
      `sanitize_dedups_signed_zero_depths`, `parse_las_2_tdep_index_populates_depth`.

## P1 — reliability (frontend state) (2026-07-20)

The three P1 reliability findings from ROADMAP §4b (stale plots, async races, listener
leaks). Frontend-only (TypeScript, `tsc` clean); these are async-lifecycle behaviors with
no unit tests, so they were hardened by an adversarial review (4 lenses → per-finding
skeptical verify: **6 confirmed / 0 refuted**, all fixed — including one real HIGH bug in
the first-pass init guard) plus a focused **second-pass verify of the fixes** (renderer-dispose

- sticky-reset lenses: **clean, 0 defects**). Click-through in `npm run tauri dev`:

* [ ] **Plots refresh after a module run, keeping their viewport** (histogramPanel,
      crossplotPanel, pickettPanel, correlationPanel, logViewPanel). Open a Histogram of
      PHIE (zoom in), then run a module/equation that recomputes PHIE (or import / undo —
      anything that bumps `dataVersion`). The plot now re-reads the new curve **in place**,
      preserving your current zoom/pan, instead of showing the pre-run curve until you close
      and reopen the panel. Each builder subscribes to `appState.dataVersion` and calls
      `reload(preserveView=true)`; a `dataPrimed` guard swallows the subscribe's immediate
      fire so nothing double-loads on open.
* [ ] **Fast well/curve/zone switching never shows stale data** (loadWell, reload,
      createPlot). Click quickly through 5 wells in the Log view, or spam curve/zone changes
      in the Crossplot/Pickett/Histogram. A slow earlier load can no longer land after and
      overwrite a newer selection — each async load captures a generation token before its
      first `await` and bails if superseded. And a viewport **reset** intent (switching wells
      / changing the curve) still fires exactly once even if a `preserveView` refresh commits
      first, via a sticky `resetPending`/`viewResetPending` flag — so you neither keep a stale
      zoom that should have reset nor lose a reset that a background refresh raced past.
* [ ] **Opening/closing Log panels doesn't leak listeners or GPU loops** (logViewPanel
      dispose, LogCanvasRenderer). Repeatedly open and close Log view panels. The renderer's
      `window` pointerup/pointermove handlers are now removed and its `requestAnimationFrame`
      loop cancelled on `dispose()` (previously leaked one set per open); disposing a panel
      _during_ WebGPU init now disposes the fully-initialized local renderer rather than a
      no-op on an already-nulled field. Nothing visible per-close, but memory/handler count
      stays flat over a long session.
* [ ] **Dialog Escape is scoped to the dialog — closes the P1 modal-Escape sliver** (modal.ts).
      The carried-over P1 sliver ("overlapping dialogs share one Escape handler"). The listener-
      leak half was already handled — `openModal` single-instances via `activeClose` (a new dialog
      closes the prior one and removes its keydown listener; no modal opens a nested modal, so
      there's no stack). The remaining gap: the dialog's Escape was a `document`-level handler that
      closed the dialog but didn't `stopPropagation`, so one Escape also bubbled to `window`/app-level
      Escape handlers. It now stops there — kept on the **bubble** phase so the numeric-edit guard's
      capture-phase `stopPropagation` still shields a dialog from closing while you edit a number
      field. Also tears down any in-flight title-bar drag listeners on close (no leak if a dialog is
      dismissed mid-drag). tsc clean. Test: start drawing a **map polygon**, open any dialog (⚙
      Properties, an import dialog…), press **Escape** → the dialog closes and the half-drawn polygon
      is **still there** (Escape no longer cancels it too); double-click a number field in a dialog,
      press Escape → you exit the field's edit mode and the **dialog stays open**; a second Escape
      closes the dialog.

## Polish — UX (veteran-interpreter friction) (2026-07-20)

Hardening-backlog **Polish** tier (ROADMAP §4b, was "P3"). Small, mostly-frontend fixes;
each tsc-clean. Mapped by a read-only investigation wave before implementing.

- [ ] **Cursor readout: real units + no more mangled values** (plotCommon.ts `formatValue`,
      viewerChrome.ts `renderReadout`, logViewPanel.ts). The log-view cursor readout used a
      blanket `toFixed(2)`, which flattened permeability 0.003 → "0.00" and showed no units.
      It now uses an adaptive significant-figures formatter (perm stays "0.003", RT reads
      "2151", φ "0.18") and appends the unit the curve catalog already carries (RT "ohm.m",
      PHI "v/v", RHOB "g/cc"). Units are cached per well-load and refresh on `dataVersion`.
      Test: hover the log view over a permeability track and a deep-resistivity track — values
      keep resolution and each shows its unit. (Values whose catalog unit is blank show no unit.)
- [ ] **Correlation: fresh well list + Ctrl+wheel zoom** (correlationPanel.ts). The Wells
      menu was built once and never refreshed, so a newly imported well never appeared. It now
      re-fetches the well list on `dataVersion` — new wells appear (and draw as strips), deleted
      wells drop out, active-group filter re-applies. Also added **Ctrl/Cmd+wheel zoom about the
      cursor** (same factors as the other plots); plain wheel still pans through depth. Test:
      open Correlation, import/deviation-load another well → it shows up without reopening the
      panel; Ctrl+wheel over the strips zooms at the cursor depth.
- [ ] **Processing history now covers every operation** (processLog.ts + call sites across
      ribbon / inspectorPanel / mlDialog / monteCarloDialog / workflowDialog / zonesDialog /
      topsEditor / mapPanel / cutoffDialog). The audit trail (History panel / QAT History)
      previously logged only LAS import/export, module runs, core shift, well header, curve edits,
      project/session, exports. It now ALSO records: **DLIS / deviation / SCAL / core / tops
      imports; equation runs; ML runs; Monte Carlo; workflow chains; log-set restore/delete; zone
      add/edit/delete + per-zone parameter overrides; manual tops add/edit/delete; cutoff-default
      saves; map-polygon→group assignment.** Test: perform each and confirm it appears in the
      History panel with the right `[kind]` and detail. (Batch/field-wide actions — equation, ML,
      MC, workflow, log-set, cutoffs — intentionally show no well name.)
- [ ] **Pickett v2 — properties dialog, typed M/Rw, configurable axes, Z-color** (pickettPanel.ts).
      The Pickett plot's RT/PHIE axes were hard-coded (0.1–1000 / 0.01–1) with no properties
      dialog. Now a **⚙ / right-click Properties** dialog sets RT & PHIE axis ranges, point size,
      and **color-by-curve** (rainbow/viridis, optional log-Z), persisted via plotprops. The
      toolbar gained **M and Rw** fields next to N — type them and the Sw=1 / iso-Sw lines follow;
      a two-point pick still fits the line and fills the same M/Rw fields (one shared source).
      Zoom/pan and the line survive a data refresh (P1 preserveView). Test: open Pickett on a well
      with RES_DEEP + computed PHIE; pick two points on the wet trend → M/Rw fill and lines draw;
      type a different Rw → lines follow; right-click → set RT axis 0.2–200 and color by SW/VSH →
      points recolor; reopen the panel → settings persist.
- [ ] **Pay-summary provenance — FLAG\_\* versioned + cutoffs recorded** (workflow.rs, backend).
      run*pay_summary wrote FLAG_SAND/RESERVOIR/PAY with the old in-place `write_computed_curve`
      — no version history, and the VSH/PHIE/SWE cutoffs that produced them were recorded nowhere.
      Now the explicit **Cutoffs & Pay Summary** run versions the three flags into a **PAYFLAG**
      log set whose provenance = module `pay_summary` + the cutoffs (in `log_sets.params_json`) +
      inputs, exactly like every other module output — so a re-run keeps history and any version
      is restorable/prunable from the Curve Catalog. The **Field Dashboard** and **report** passes
      set `skip_version` (field-wide QC side-effect) so they keep overwriting in place — no version
      churn per refresh. Test `pay_summary_versions_flags_with_cutoffs_in_provenance` (161 lib
      tests pass / 0 fail / 7 ignored; tsc clean). Click-through: run Cutoffs & Pay Summary on a
      well → Curve Catalog shows a PAYFLAG version whose provenance lists the cutoffs; re-run →
      version N+1; run the Field Dashboard → FLAG*\* update but no new version piles up.

## Performance (field-scale speed) (2026-07-20)

Hardening-backlog **Performance** tier (ROADMAP §4b, was "P2"). The rest of the tier (#128–132)
changes DB/IPC semantics and needs a live 100-well benchmark to sign off; this first item is the
one pure-frontend, low-risk win.

- [ ] **Crossplot: Z coloring memoized across pan/zoom/hover** (crossplotPanel.ts). Every crossplot
      redraw rebuilt the whole per-point color array from scratch — for a continuous Z that's **two
      `percentile` sorts** (each allocates + sorts a NaN-filtered copy of all samples) plus an
      N-length `colorRampEx` string array; for a discrete Z, an N-length `categoricalColors` map.
      That ran on **every** pan-drag / zoom-wheel / handle-drag `mousemove` and every synchronized-
      hover frame, even though the colors depend only on the Z data + colormap, never the viewport.
      The color computation is now a pure `computeCrossplotColors()` that the panel **memoizes**,
      keyed by (Z curve, colormap, log-Z, fixed color, data generation); pan/zoom/hover reuse the
      cached array and only a data or color-setting change recomputes. Output is pixel-identical —
      this is a speed change only, most visible on dense (100-well / full-field) clouds. tsc clean.
      Test: open a crossplot colored by a curve (e.g. NPHI-RHOB by GR, or a PERM Z with log-Z on),
      drag the parameter handle / Ctrl+wheel-zoom / pan / hover from a log view — motion stays
      smooth on a big cloud; the colors, color-bar range, and facies legend are unchanged; switching
      the Z curve, colormap, or log-Z toggle still recolors immediately; a module re-run (dataVersion)
      recolors against the new data.

## Low-tier correctness & data-integrity sweep (2026-07-21)

The 15 low-severity findings from `AUDIT-2026-07-20.md` (never adversarially verified at audit time)
were each re-checked against the current code by an independent per-finding verifier. One was
**already fixed** (SandiMin all-zero conductivity-row guard, `multimin2.rs`), two are **held for your
sign-off** because they change numeric output (Wyllie compaction correction; histogram re-bin), one is
**held to land with the depth-scale-ratio fix** (scale-dropdown staleness). The rest — the safe
correctness/crash/data-integrity fixes below — are applied. **Rust suite green; tsc clean.** Nothing
committed.

Backend (with new regression tests):

- [ ] **SSC `SWIRR_EFF` no longer 0 at a 100 %-shale point** (`ssc.rs`). At the wet-clay point effective
      porosity is floored to 0, and the `1 − φt·(1−SWIRR_T)/φie` divide gave `−inf→0` ("all water
      movable") or `0/0→NaN` — exactly backwards. Now a zero-effective-porosity sample reports
      `SWIRR_EFF = 1.0` (fully bound). _Only the degenerate φie==0 samples change; every producing
      point is unchanged._ (The deeper SWIRR_T/SWIRR_EFF ordering inconsistency is the separate held
      item.) Test: run SSC on a shale-heavy well; SWIRR_EFF in massive shale reads ~1, not 0.
- [ ] **Archie `SWT_ARCH` no longer writes `+Infinity`** (`modules.rs` `sw_arch`). A coal/tight sample
      with PHIT=0 but PHIE absent used to fall through to `a/0^m = +inf` and store it in the SWT_ARCH
      curve, poisoning catalog min/max and plot autoscale. The zero-porosity "all water" guard now
      keys on PHIT alone. Test (regression `sw_arch_zero_porosity_missing_phie_is_all_water_not_inf`):
      the curve catalog's SWT_ARCH min/max stays finite over coal/tight zones.
- [ ] **Simandoux (SCHLUMBERGER) no longer divides by zero at VSH=1** (`modules.rs` `sw_sim`). Pure
      shale hit a `1/(1−VSH)` singularity and the sample was silently dropped; it now resolves to
      all-water (SWE=1), matching the low-porosity and Indonesia branches. Test:
      `sw_sim_schlumberger_pure_shale_is_all_water`.
- [ ] **LAS import fails loudly on a truncated row** (`parsers.rs`, both parsers). A physically short
      `~A` row used to shift every following value one column left silently (GR into RES, RHOB into
      DT) for the rest of the file. Leftover tokens at EOF now raise a clear import error instead of
      mis-columning. Test: import a LAS whose last data line is cut mid-row → you get an explicit
      "leftover token(s)…truncated or corrupt LAS?" error, not corrupted curves.
- [ ] **DB-inspector edit no longer reports success on a 0-row update** (`db.rs`, all three sample
      editors). If the matched depth had moved/been rewritten, the UPDATE hit 0 rows but the UI said
      "saved" and pushed a bogus undo entry. It now errors, and the inspector already reverts the cell + shows "Edit failed". Test (`…is_err()` assertion added): edit a sample, then edit a
      non-existent depth → "no … sample matched depth …", cell reverts, no phantom undo.
- [ ] **Well Header shows current TD / KB** (`db.rs` list_wells + `ipc.ts` + `ribbon.ts`). The dialog
      used to open with blank TD/KB, so you edited the datum blind — and KB silently drives TVDSS in
      deviation import. TD/KB are now carried on `WellSummary` and prefilled. Test: open Well Header on
      a well with a KB set → the field shows it, not an empty box.

Frontend:

- [ ] **Stats / regression reject `±Infinity`, not just NaN** (`plotCanvas.ts`: basicStats, linearFit,
      percentile, drawScatter/drawDiamonds). One inf sample (e.g. a Python `1/phi` at phi=0) used to
      make the histogram's Mean/Std chips read "Infinity" and silently kill a crossplot regression.
      Now non-finite values are skipped everywhere. Test: compute an equation that divides by a
      zero-porosity sample, then histogram/crossplot it — chips and the fit stay sane.
- [ ] **Zone-param "Set" button surfaces write failures** (`plotCommon.ts` pickRow — histogram Pick
      A/B, Pickett M/Rw). A rejected `setZoneParam` used to be swallowed while the status still said
      "set". It now shows "Failed to set …". Test: (hard to force by hand) — behaviour only differs on
      a backend write error; the success path is unchanged.
- [ ] **Duplicate track titles prevented** (`layoutPropsDialog.ts`). Renaming a track to an existing
      track's title collapsed both in every title-keyed lookup (weights, cursor hit-testing, core
      overlay, drag-drop). A colliding rename is now auto-suffixed ("RES 2"). Test: in Layout
      Properties, rename a track to another track's exact name → it becomes "name 2"; retyping a
      track's own name is a no-op.
- [ ] **Histogram: constant curves render; the `n` never silently disagrees** (`histogramPanel.ts`).
      A constant curve (flag/class curve, single-sample zone) used to show "No valid data"; it now
      draws one central bar. And when the P2–P98 axis window clips tail samples, the axis label reads
      `n = X of Y` so it no longer contradicts the stats chips (which count all samples). _(The full
      full-range re-bin — which would change every bar height — is the held item.)_ Test: histogram a
      constant/flag curve (draws), and a curve with fat tails (label shows "of").
- [ ] **Log-view smoothness** (`LogCanvasRenderer.ts`, speed only). The clear color is no longer read
      via `getComputedStyle` every rendered frame (cached, invalidated on theme change), and the
      cursor readout uses a binary search instead of scanning every sample per mouse-move. Values and
      colors are identical. Test: drag-pan a busy log view — motion is smoother; theme switch still
      repaints; the cursor readout still tracks correctly.

### Held-item resolutions (2026-07-21 — your call: 1 yes / 2 leave / 3 yes / 4 yes + Bahasa Jawa)

Your answers to the four held items above. **Rust suite 164 pass / 0 fail; tsc EXIT 0; the two
browser-observable pieces verified live in the vite preview.**

- [ ] **Wyllie lack-of-compaction (Cp) correction — shipped as opt-in** (`modules.rs` `phi_son`,
      `OPT_CP` **default OFF**). ON divides the WYLLIE porosity by `Cp = DT_SH/100`; RHG is
      self-compacting and is never touched. Nothing changes until you switch it on. Test
      (`phi_son_wyllie_cp_opt_in_only_scales_wyllie`): OFF unchanged; ON ≈ +11 % at DT_SH=90; RHG
      unaffected. In the app: run Porosity → Sonic with OPT_CP=ON on a shallow well → PHIT rises a
      few p.u.; OPT_CP=OFF reproduces the old numbers exactly.
- **Histogram full-range re-bin — left as-is** at your request (bars keep clipping the extreme tails;
  bar heights and the mode/P50 you read off them are unchanged).
- [ ] **Depth-scale dropdown now shows the TRUE scale + the mislabel is fixed**
      (`LogCanvasRenderer.ts`, `logViewPanel.ts`). The default was labelled "1:100" but was really
      ~1:3937, and the `[0.02, 20]` px/unit clamp made **1:20, 1:50 and 1:100 all collapse to the same
      zoom**. Now: a true 1:1 = `96/0.0254` px per depth unit is single-sourced; the view opens at an
      honest **1:2000**; the clamp reaches a true 1:10; and after any Ctrl+wheel/± zoom the selector
      re-reads the live ratio (a transient "1:N ⟳" entry when it's between presets). Test: pick 1:50
      then 1:100 → visibly different scales (identical before); Ctrl+wheel zoom → the box tracks the
      real ratio instead of freezing on the last preset.
- [ ] **Quiet Ctrl+S save + Escape closes ribbon menus** (`ribbon.ts`). Ctrl/Cmd+S re-saves the
      current session in place once it has a name (no dialog), falling back to Save Session As the
      first time; it's ignored while typing in an input/CodeMirror so editors keep their own Save.
      Escape closes any open ribbon dropdown without disturbing modal Escape handling. _(A Ctrl+P
      print-active-plot shortcut was deliberately deferred — resolving "the active canvas" from the
      ribbon is fragile; the per-plot Print button still works.)_ Test: name a session, edit the
      workspace, Ctrl+S → "Session … saved" with no dialog and the unsaved dot clears.
- [ ] **Bahasa Jawa (jv) added + fuller Bahasa Indonesia / Basa Sunda** (`i18n.ts`, `index.html`).
      A full Javanese (ngoko) dictionary joins id/su, and ~55 common UI phrases (New/Open/Edit/Search/
      Print/Value/Zone/Session/… + statuses) were added to all three — petrophysics jargon still stays
      English by design. Test: Project → Language → **Basa Jawa** → menus/buttons switch (Save→Simpen,
      Depth→Jero, Reload→"Muat manèh"); switch back to English → everything reverts from source.

## Reference-library correctness fixes (2026-07-20)

Two physics fixes distilled from the ITB team reference shelf (Ellis Ch12/14, Halliburton FE Ch27):

- [ ] **Multimin — PEF now converts to U before mixing.** In the Multimin (SandiMin) dialog,
      select **Photoelectric (PEF)** as an input tool (instead of, or alongside, U) and run on
      a well with a PEF curve + RHOB. Confirm VOL\_\* volumes are sensible and RECON is low in
      clean zones. Physics: per-electron PEF does NOT mix by volume — the solver now converts
      the PEF curve to U = Pe·ρe per sample (ρe from RHOB) and mixes against the U endpoints.
      Picking U directly is unchanged. Needs a RHOB curve present; where RHOB is missing the
      PEF row is simply skipped that sample.
- [ ] **VSH from Density-Neutron — new VSH_DN_FLAG clay-type guard.** Run **VSH from
      Density-Neutron** with the optional **GR** input mapped and set GR_MA/GR_SH/FLAG_TOL.
      Confirm a new `VSH_DN_FLAG` curve = 1 where the N-D VSH is off-model (gas crossover /
      beyond the shale point) or diverges from the GR VSH by more than FLAG_TOL (0.25 v/v
      default) — the signature of clay-type or gas ambiguity. Leaving GR unmapped still flags
      off-model samples. VSH/VSH_DN themselves are unchanged.

## Field Map — well surface coordinates + polygon → group (2026-07-20 #27)

**Field Map** (View/Batch ribbon map button, or the Petrophysics ▸ Field Map… button) — a
standalone dock pane that posts wells by UTM surface location and lets you rubber-band a
polygon to select wells into a well group. Coordinates arrive two ways: **Data ▸ Import Well
Locations…** (a CSV/TXT with EASTING/NORTHING, optional WELL and ZONE columns, plus a
choosable default UTM zone covering Indonesia — zones 46–54, N and S — applied to rows/files
without a ZONE column), or per-well via **Tools ▸ Well Header** (Surface X / Surface Y / UTM
zone fields). Coordinates persist as DOUBLE in new `wells` columns
(surface_x/surface_y/utm_zone) — southern-hemisphere northings ≈ 9.4e6 exceed f32's ~1 m
precision, so f64 is required. The pane draws markers with pan (drag), cursor-anchored
wheel-zoom, a faint coordinate grid, a scale bar, and labels for ≤80 wells; the active well
group is ringed. Draw mode: click to drop polygon vertices, close near the first vertex /
double-click / Enter; vertices are draggable; enclosed wells highlight live (a TS
point-in-polygon mirrors the Rust ray-cast). **Assign to group…** runs the authoritative
backend `wells_in_polygon` (PNPOLY, half-open crossing rule) and unions the result into a new
or existing group. Raw easting/northing is plotted directly — no reprojection; a multi-zone
project is a documented follow-on. The polygon is a transient selection tool — the persistent
artifact is the well-group membership (persisting polygon shapes as documents is a noted
follow-on vs the roadmap's original wording).

Adversarial review (4 lenses — geometry-math / import-parse / integration /
frontend-robustness, each finding skeptically verified): **3 defects confirmed, all fixed; 0
refuted.** (1) [high] Well Header Save wrote surface_x/y/zone unconditionally from a stale
`selectedWell` snapshot that is never re-broadcast on a data change, so re-saving after an
import (or a prior save) NULLed out the just-set coordinates — fixed by re-reading the well
from the DB when the dialog opens, so the fields always reflect current state. (2) [medium] A
blank WELL cell in a multi-well locations file collapsed into the same "no well column" case
as a headerless file and was routed to the selected well, silently overwriting an unrelated
well's location — fixed by returning a `has_well_column` flag from the parser so the importer
only falls back to the selected well for a genuinely column-less file, and skips (and
reports) blank-cell rows; the import loop is now wrapped in a transaction so a mid-file error
rolls back instead of leaving a partial write reported as a total failure. (3) [medium] When
coordinates first arrived while the pane was already open, the data-driven reload never fit
the view, so markers rendered off-screen until a manual Fit — fixed by fitting on the first
appearance of laid-out wells. cargo test 143/143; tsc clean.

- [ ] **Data ▸ Import Well Locations…** — pick your Indonesia UTM zone (e.g. 50S for
      Mahakam), import a CSV with WELL/EASTING/NORTHING → the status line reports N wells
      located; open **Field Map** and confirm the wells post at the right relative geometry.
- [ ] Import a file that has a WELL column but a blank cell in one row → that row is skipped
      and surfaced as "1 blank-WELL row(s)", and no unrelated well's location changes.
- [ ] **Tools ▸ Well Header** on a located well → Surface X/Y/zone show the imported values
      (not blank); change only TD and Save → the coordinates survive (were being wiped before).
- [ ] With Field Map already open on a project that had no coordinates, run Import Well
      Locations → the map fits to the new wells automatically (no manual Fit needed).
- [ ] Field Map ▸ **Draw polygon**, enclose a few wells, **Assign to group…** → the enclosed
      wells land in the chosen/new well group; the group filter elsewhere reflects it.

## φmax porosity ceiling — phimax module (2026-07-20 #26)

**Petrophysics ▸ Porosity ▸ Porosity Ceiling (φmax)** — caps a computed porosity at
the field's compaction-controlled upper limit (the deck slide-64 "max core porosity"
line). A `MODE` dropdown picks the ceiling model: **constant** (a flat `PHIMAX0`,
per-zone overridable — the literal max-line), **linear** (`φmax = PHIMAX0 −
PHIMAX_GRAD·(TVDSS − TVDSS_REF)/1000`), or **athy** (`φmax = PHIMAX0·exp(−ATHY_K·
(TVDSS − TVDSS_REF)/1000)`, the exponential compaction law). TVDSS is a
positive-downward depth-below-datum curve (same convention as **precalc**), so deeper
= larger TVDSS = lower ceiling; with no TVDSS curve it falls back whole-curve to
measured DEPTH (fine for near-vertical wells). All four parameters are zone-overridable,
so each formation (Post-Main / Main / Massive / Talang Akar) can carry its own ceiling
or its own trend coefficients. Writes `<PHI>_CAP = min(PHI, φmax)` (preserving MISSING)
and the ceiling curve `<PHI>_MAX` for a QC overlay; the input porosity is never
modified. The dialog is auto-generated from the manifest, so it appears in the Porosity
dropdown with no bespoke UI. Standalone by design — it caps _any_ porosity output
(phi_den/phi_dn/phi_son or SandiMin's PHIT); a solver-internal φmax box constraint is a
noted follow-on.

Adversarial review (4 lenses — math / integration / edge / contract — each finding
verified): **0 defects confirmed, 4 refuted** (all were test-coverage/doc-completeness
notes over correct, deliberate behaviour). Two of the flagged-untested paths were
locked in with regression guards anyway: the ceiling clamp to [0,1] (a sub-zero trend
ceiling forces porosity to 0; a super-unit one clamps to 1), and the partial-NaN TVDSS
pass-through (a NaN-depth sample gets a MISSING ceiling and passes porosity through
uncapped). cargo test 136/136; tsc clean.

- [ ] **Petrophysics ▸ Porosity ▸ Porosity Ceiling (φmax)** opens (auto-dialog). Run
      **constant** mode, PHIMAX0 = your field max (e.g. 0.35), input **PHIE** → the
      `PHIE_CAP` curve should equal PHIE below 0.35 and flatten at 0.35 above it;
      `PHIE_MAX` is a flat 0.35 line.
- [ ] Crossplot `PHIE_CAP` vs depth (or overlay `PHIE_MAX` on the porosity track) — the
      capped cloud should sit under the ceiling with no points poking above it.
- [ ] Switch to **linear** (or **athy**) with your TVDSS trend: PHIMAX0 at a shallow
      `TVDSS_REF`, a sensible `PHIMAX_GRAD` (or `ATHY_K`). `PHIE_MAX` should fall with
      depth; confirm a deep zone's ceiling is lower than a shallow zone's.
- [ ] Set **per-zone** `PHIMAX0` (or trend coeffs) in Zones/zone params and re-run —
      each formation's ceiling should honour its own value.
- [ ] Well with **no TVDSS curve**: linear/athy should still run (trend reads against MD
      DEPTH) — sanity-check that the ceiling still declines with depth. Deviated wells
      want a real TVDSS curve (survey→TVDSS bridge is a follow-on).
- [ ] Feed `PHIE_CAP` into **Cutoffs & Pay Summary** as the PHIE input — pay should drop
      where the cap trimmed optimistic porosity.

## Cutoff Sensitivity pane (2026-07-20 #25)

**Reporting ▸ Cutoff Sensitivity** — two ways to defend a VSH/PHIE/SWE pay cutoff
against DST-tested rock (KKT ONWJ deck slides 84–87), in one dock pane with a
Sweep / DST-Crossplot toggle. **Sweep** varies one cutoff across a range while the
other two stay fixed and plots the pay response per well — net thickness, HC
pore-thickness (HPV), or net-to-gross — so the _elbow_ shows where loosening the
cutoff stops adding real pay; the shared pay math is the **same `classify_sample`
the pay summary uses**, so the numbers reconcile. **DST Crossplot** is PHIE vs a
shale/Sw curve with every sample dim and DST-interval samples coloured per well,
plus a draggable red crosshair at the candidate cutoffs. Either mode's pick writes
into the VSH/PHIE/SWE fields and can be **saved as the pay-summary default** so the
cutoff you defended flows straight into the report. Optional zone and DST/perf
filters scope both modes.

Adversarial review raised 13, confirmed 10 (from two independent full passes),
all fixed before shipping: the sweep's PERM-cutoff scope now matches the pay
summary exactly (whole-well, not just the analysed window); overlapping
perforation/DST intervals are unioned so N:G isn't understated; switching the
swept property/metric after a run clears the stale plot so a pick can't be written
into the wrong cutoff; the "(all samples)" DST choice survives editing the well
set; the zone/DST pickers union over _all_ checked wells (was capped at 16); the
crosshair stays inside the plotted range; empty-state text is centred on HiDPI; the
plot repaints on theme change; wells with no pay / missing inputs are flagged
rather than shown as a silent flat line. cargo test 131/131; tsc clean.

- [ ] **Reporting ▸ Cutoff Sensitivity** opens as a dock pane. Tick a few wells,
      keep **Sweep**, property **VSH**, metric **Net**, Compute — one line per well;
      the curve should rise and flatten (an elbow), not be a straight ramp.
- [ ] Click/drag on the plot to place the red cutoff line; the readout shows the
      net/HPV/N:G each well delivers _at that cutoff_. Click **Use pick as VSH
      cutoff** → the VSH field updates.
- [ ] Switch the metric to **N:G**, Compute again; switch property to **PHIE** — the
      plot should **clear** and ask you to Compute (it must not keep showing the VSH
      sweep while the button says "PHIE").
- [ ] Cross-check one well against **Cutoffs & Pay Summary** at the same fixed
      VSH/PHIE/SWE (whole well, no zone/DST): the sweep's Net at those cutoffs should
      match the pay summary's Net for that well. Repeat with a **PERM ≥** cutoff set.
- [ ] **DST Crossplot** mode with a well that has a DST/perf set: dim cloud + coloured
      DST points; drag the crosshair; **Apply crosshair → cutoffs** writes PHIE and
      VSH (or SWE if the X curve is an Sw). Pick a "PHIE vs Sw" preset and confirm the
      Apply maps to SWE.
- [ ] Pick a DST set, then switch the DST dropdown to **(all samples)**, then tick/untick
      a well — the dropdown must **stay** on "(all samples)" (not snap back to the DST set).
- [ ] **Save as pay-summary default** → open **Cutoffs & Pay Summary**: its VSH/PHIE/SWE
      inputs should already carry your saved cutoffs.
- [ ] Switch the app theme with the pane open — the plot repaints immediately in the
      new palette (no stale colours).

## All tools as dockview panes (2026-07-20 #24)

Your ask: "i want all tools shows as pane, for existing and future tools." Every
computation/analysis tool now opens as a **dockable pane** instead of a pop-up. The
big one: the **auto-generated module form** (every Petrophysics ▸ Data Prep / VSH /
Porosity / Saturation module) is now a pane — one per module — so you can keep
several docked side by side and re-run each as you iterate, and **any new module I
add in Rust gets its pane automatically** with no extra UI work. **Zones,
Autocorrelate Tops, Composite Log, and Report** are panes too; they follow the
selected well the way the plots do, and refresh their lists when data changes. Quick
pop-ups stayed pop-ups on purpose (curve editor, layout properties, save/open
session, import prompts). Adversarial review found 9 real issues, all fixed before
this shipped (pin-off panes catching up to a selection, no stale-well writes after a
project switch, the autocorrelate "pick a top first" message re-checking itself once
you pick one, etc.). tsc clean; module-pane behavior browser-verified.

- [ ] Open a module (e.g. Gas Correction) from the Petrophysics tab — it should
      appear as a pane you can dock/split/float, not a pop-up. Run it; the result
      lines stay in the pane (no auto-close). Open a second module — both panes
      coexist (the old pop-ups could not).
- [ ] With a module pane open, compute a curve, then open another module: the new
      curve should already be selectable in its input dropdowns (the pane refreshes
      its lists on data changes without losing what you'd already picked).
- [ ] Multi-select several wells in Wells & Tops, THEN open a module: all selected
      wells should be pre-ticked (not just the active one).
- [ ] Open the **Zones** / **Composite** / **Report** pane with no well selected —
      it shows "Select a well… will follow" instead of a "select a well first"
      toast; pick a well and it fills in and the tab title updates.
- [ ] **Autocorrelate Tops** on a well with no tops: the pane says "pick one in the
      log view first" — go pick a top, and the pane should update itself (no need to
      close/reopen). Apply a correlation: the proposals clear.
- [ ] Switch projects with a Zones/Report pane docked in a background tab: it must
      reset to the "select a well" hint, NOT keep showing a well from the old
      project (this prevents editing the new project with a stale well).
- [ ] Docking sanity: the panes save/restore with the workspace layout, appear in
      the ＋ "add panel" menu, and the log-view right-click "Print / export layout…"
      opens the Composite pane.

## Gas Correction module — iterated density de-gassing (2026-07-20 #23)

**Petrophysics ▸ Data Prep ▸ Gas Correction (density, iterated)** — the KKT deck
slide-65 loop. Density porosity and Archie SWT are solved from the current density,
then RHOB_GC = RHOB + Φt·(1−Sw)·(RHO_FL − GASDEN) replaces gas with liquid, iterated
to |ΔΦt| < 1e-4 (non-converging samples stay MISSING). GASDEN is the real-gas density
of an SG_GAS 0.65 gas at FPRESS/FTEMP (Standing pseudo-criticals + Papay z, pinned
0.1297 g/cc at the KK example's 2743 psi / 93.9 °C) — **run precalc first**; FTEMP and
FPRESS accept only precalc/log-set curves, never a raw import (a the reference suite LAS's degF
FTEMP can't sneak in as degC). Default **OPT_GATE = FLAGGED** corrects only where the
gas flag > 0.5 (chain condflag's XOVER_FLAG, which excludes coal and washout) and
errors loudly if the flag curve has no data; **EVERYWHERE** is there for wells without
condflag, but beware coals/resistive washouts — high RT + low density reads as gas to
the Archie loop. The adversarial review raised 13 confirmed findings → all fixed
(FLAGGED default, flag > 0.5 gate, no-flag-data error, degenerate RHO_MA/RHO_FL and
RHOB<RHO_FL and Rw≤0 guards, non-convergence → MISSING, NaN-proof Archie cap,
computed-only P/T inputs, RHOG→GASDEN rename, doc rewrite). 127 cargo tests green.

- [ ] Run precalc → condflag → Gas Correction (defaults) on a KK-style gas well: the
      detached high-porosity gas cloud on PHIE vs wet-clay (slides 66–67) should
      collapse after correction; RHOB_GC ≈ RHOB in water zones (self-limiting there).
- [ ] Check a coal streak stays untouched under the FLAGGED default (XOVER_FLAG
      excludes coal) — no phantom high-porosity pay in coals.
- [ ] Without condflag run: the FLAGGED default must error "gas flag has no data —
      run condflag first or set OPT_GATE = EVERYWHERE", not silently pass through.
- [ ] Without precalc run: outputs stay MISSING (never uncorrected pass-through),
      even if the well's LAS carries its own raw FTEMP/FPRESS curves.
- [ ] Feed RHOB_GC to **phi_den** (or use PHIT_GC directly). Do NOT feed it to phi_dn
      or a SandiMin solve that includes NPHI — their gas handling assumes an
      uncorrected density-neutron pair (the module doc says this too).

## SandiMin: wet→dry clay converter + fluid autofill from precalc (2026-07-20 #22)

Two additions inside the **SandiMin** pane (Advance tab), from your Multimin
Parameters.xlsx workflow (Wave E item 18). **Wet clay → dry clay** panel: enter the
wet-clay picks from a shale interval (RHOB/NPHI/GR, optional DT) and the assumed
dry-clay density (2.70 marine / 2.78 deltaic per the KKT deck slide 60); it computes
φ_clay = (ρdry−ρwet)/(ρdry−1) and the dry endpoints with the xlsx formulas verbatim
(water 1.00 g/cc, 189 µs/ft), previews them live, and **Apply** writes them to the
chosen clay, ticks it + BoundWater, and sets a **CEC_eq** on the clay that makes the
solver's Dual-Water bound-water constraint enforce exactly v_bw = φ/(1−φ)·v_dryclay —
the deck's slide-59 bookkeeping (SWB = VOL_UBNDWAT/PHIT). Unphysical picks error
instead of applying: NPHI must be a fraction (percent entry rejected — the reference suite habit
guard), GR positive, wet DT above the 189·φ water term. **Autofill from precalc**
(fluid box): pick a zone of the selected well and **Read** — fills Formation temp
from FTEMP_F and the Rmf sample from precalc's RMF (retied to formation temp, an
Arps no-op, only when both curves came back; a raw RMF without FTEMP_F is refused
as not-precalc). The zone dropdown follows your well selection live.

- [ ] KK-1 Post Main check: wet 2.18333/0.48958/110 with dry density 2.70 → the
      preview must read φ_clay 0.3039, NPHI 0.2667, GR 158.0 (the xlsx values).
- [ ] Apply to Illite, then run SandiMin with CT on: solved VOL_UBNDWAT/VOL_DRYCLAY
      should sit at ~0.4366 (= φ/(1−φ)) in clay-rich intervals; SWB = VOL_UBNDWAT/PHIT
      comparable to the deck's slide-59 CWB-panel behaviour.
- [ ] Note the pairing rule: CEC_eq is tied to the clay's **RHOB endpoint** and the
      fluid **T/Rw/α** at Apply time — if you edit any of those afterwards, re-Apply
      (the status line and the CEC column tooltip both say so now).
- [ ] Autofill on a precalc'd well: Read (whole well and one zone) fills FTEMP/Rmf
      and the previews update; on a well without precalc it must refuse with "run
      the precalc module first", not fill garbage.
- [ ] Switch wells with the SandiMin pane open: the autofill zone list must follow
      the selection (it re-reads the new well's zones).

## Neutron Matrix Conversion module — NPHI LS/SS/DOL (2026-07-20 #21)

New Prep module **Neutron Matrix Conversion** (`nphimat`) in the Data Prep dropdown
and workflow builder (your request 2026-07-20). Converts a neutron log recorded in
one matrix convention into all three — **NPHI_LS / NPHI_SS / NPHI_DOL** — using the
chartbook porosity-equivalence curves digitized at vector precision: **Por-5** for
the CNL thermal tools (**NPHI** ratio method; **TNPH** env-corrected, FRESH / 250 kppm
SALT variants) and **Por-4** for the epithermal tools (**APLC/FPLC** = APS, **SNP** =
legacy sidewall). Tell it what the log is (TOOL + MATRIX_IN); the input convention
passes through unchanged and the other two are read through the chart (SS/DOL inputs
invert back to the apparent-limestone axis first). The book's printed worked example
(TNPH 18 pu @ 250 kppm → sandstone 24 pu) reproduces to 0.04 pu. Feed the output
matching your RHO_MA (NPHI_SS with 2.65) — that removes the ~0.04 LS-vs-SS offset the
condflag doc warns about, so XOVER_MIN can stay at 0.04. Also in this increment:
APS/legacy neutron mnemonics (APLC/FPLC/SNP/NPOR/HNPO/NEUT/FSTP) now fill the
standard NPHI column at LAS import, an all-NaN standard column now falls back to the
raw store (family alias) instead of silently feeding NaN to modules, and workflow-
builder input dropdowns now offer every module's outputs so `nphimat → phi_dn
(NPHI = NPHI_SS)` is buildable in a fresh project.

- [ ] Run nphimat on a Mahakam well (TOOL matching the delivery, MATRIX_IN per the
      LAS header — usually LS or SS): NPHI_SS ≈ NPHI_LS + 0.03-0.04 in clean sand,
      NPHI_DOL well below both (thermal dolomite bow).
- [ ] Sanity vs the paper chart: pick one depth, read Por-5 by hand, compare all
      three outputs (expect agreement within ~0.5 pu).
- [ ] Feed NPHI_SS + RHO_MA 2.65 into phi_dn / condflag: crossover in a known gas
      sand appears at XOVER_MIN 0.04 without the limestone-unit offset fudge.
- [ ] Workflow builder in a fresh project: chain nphimat → phi_dn with the NPHI
      input overridden to NPHI_SS (now offered in the dropdown before any run).
- [ ] If you have an APS well (APLC): import fills NPHI now — check the curve
      arrives and nphimat TOOL=APLC gives sensible (small) matrix shifts.

## Data Conditioning Flags module — coal / tight / crossover + shoulder (2026-07-20 #20)

New Prep module **Data Conditioning Flags** (`condflag`) in the Data Prep dropdown
and workflow builder (your request 2026-07-20). One run writes five 0/1 flag
curves: **COAL_FLAG** (RHOB < 1.9 & NPHI > 0.35, plus DT > 100 µs/ft where a sonic
exists; samples with BADHOLE = 1 are never called coal — washouts mimic coal),
**TIGHT_FLAG** (density porosity and NPHI both < 0.05; DPHI uses **RHO_MA/RHO_FL —
the same params and zone overrides as the density-porosity modules**),
**XOVER_FLAG** (gas crossover DPHI − NPHI > 0.04; coal and bad hole excluded —
NPHI must be matrix-consistent with RHO_MA, else raise XOVER_MIN to ~0.08 for
limestone-unit neutron), **SHOULDER_FLAG** (the adjustment you asked for: samples
within SHOULDER of a coal/tight bed edge — or a bad-hole interval ≥ MIN_THICK —
carry boundary-averaged readings and get flagged so no shoulder log survives the
mask), and **COND_FLAG** (combined mask: coal | tight | badhole | shoulder, plus
crossover only when OPT_XCOND = YES). Beds thinner than MIN_THICK are dropped as
spikes; a missing sample inside a bed does not split it. MIN_THICK/SHOULDER are
in the depth curve's unit (defaults suit metres — roughly ×3 for feet). Run
badhole first; feed COND_FLAG as the Mask on later runs, but leave the Mask empty
on the condflag run itself. BADHOLE and COND_FLAG are now always offered in every
Mask dropdown, even in a fresh project where they haven't been computed yet.

- [ ] Run badhole → condflag on a Mahakam well with coals: COAL_FLAG picks the
      coal streaks (check against the density track), and no coal call inside
      washouts.
- [ ] TIGHT_FLAG on a calcite-cemented/tight streak; XOVER_FLAG on a known gas
      sand; crossover NOT flagged over coals.
- [ ] SHOULDER_FLAG brackets each coal/tight bed by ~SHOULDER depth units; a
      lone one-sample BADHOLE blip is masked in COND_FLAG but does NOT dilate.
- [ ] MIN_THICK: single-sample spikes dropped; a real bed with one null sample
      in the middle is kept whole.
- [ ] Feed COND_FLAG as Mask on a porosity run: flagged + shoulder samples go
      missing in the outputs; confirm COND_FLAG appears in the Mask dropdown of
      a fresh workflow before condflag has ever run.
- [ ] Zone overrides: RHO_MA 2.71 in a carbonate zone shifts TIGHT/XOVER there
      (same override the density-porosity modules use).

## Wave E-17: pre-calculation module — P / T / Rmf / Ct / Cxo (2026-07-20 #19)

New Prep module **Pre-Calculation (P / T / Rmf / Ct / Cxo)** in the Data Prep
dropdown and the workflow builder (ROADMAP §4c item 17, from your KKT ONWJ
workflow). One run writes six curves: FTEMP (**always degC** — the unit every
downstream module assumes) plus FTEMP_F (the degF twin, for SandiMin fluid
entry) and FPRESS as linear trends in TVDSS (gradients per depth unit of the
TVDSS curve — per-metre values for metric wells; no TVDSS curve → measured
depth is used), RMF at formation temperature (ARPS from a surface Rmf
measurement, or TREND regression `RMF_A + RMF_B·log10(TVDSS)` for wells
without mud data — the shipped defaults are the ONWJ **feet-based** fit), and
CT = 1000/RT, CXO = 1000/RXO in mmho/m as QC/plotting conductivities (note:
SandiMin's CT/CXO tool rows read the resistivity curves directly — don't feed
these to them). Params are SURF_TEMP/TEMP_GRAD (own names, so zone overrides
never cross-apply with Formation Temperature's degC-only TSURF/TGRAD); entry
unit degF/degC via OPT_TU.

- [ ] Run it on a KKT-style well with your fits (SURF_TEMP 77 / TEMP_GRAD
      0.0260292, PSURF 44.2823 / PGRAD 0.539812, degF): FTEMP_F/FPRESS match
      the deck's trend lines; spot-check one depth by hand; FTEMP = same in degC.
- [ ] Deep resistivity input defaults to the RES*DEEP family (same as the sw*\*
      modules) so CT fills for wells whose deep curve is ILD/LLD/AT90 etc. —
      confirm CT is not blank on a standard import.
- [ ] ARPS mode: RMF at depth ≈ your surface Rmf pulled down by (T₁+6.77)/(T₂+6.77);
      TREND mode with A 0.517068 / B −0.116517 reproduces the field regression.
- [ ] degC mode on a metric well (e.g. SURF_TEMP 25, TEMP_GRAD 0.03 degC/m):
      FTEMP in degC, FTEMP_F in degF, RMF still Arps-correct.
- [ ] CT/CXO: 1000/RT and 1000/RXO, missing where RT/RXO are missing or ≤ 0.
- [ ] Zone overrides: give one zone a different TEMP_GRAD in the Zones dialog —
      the FTEMP trend kinks at the zone boundary (per-zone params resolve per
      sample).

## Wave A-4: workflow grid inspector (2026-07-20 #18)

The Workflow Builder pane has a **List | Grid** toggle above the step list
(ROADMAP §4c item 12). Grid = the multi-line inspector: rows are your chain's
steps, columns are the union of every step's inputs/params/options (+ Mask), so a
parameter shared by several modules lines up in one column. The italic **Set all**
row under the header edits a parameter across every step that takes it in one go.

- [ ] Build your standard chain (vsh → phi → sw\_\* …), switch to **Grid**: input
      curves come first, then numeric params, then options, then Mask; steps that
      don't take a column show "—". Header tooltips = parameter descriptions.
- [ ] **Set all → RW**: type one RW in the Set-all row — every sw\_\* step that takes
      RW updates at once (status bar reports how many). A value outside one
      module's allowed range is skipped for that module only and reported.
- [ ] Edited cells tint amber and the step's override badge counts up — same
      only-store-differences rule as the per-step editors, so a value typed equal
      to a module's default clears that override (cell untints). Zone params still
      override these whole-well values per zone at run time, as before.
- [ ] **Set all → Mask** sets opts.MASK (e.g. BADHOLE) on every step in one edit.
- [ ] Toggle List ↔ Grid: values, badges and invalid-input flagging stay in sync
      (both views edit the same steps). The chosen view is remembered.
- [ ] Save the workflow, reload it, re-run — saved JSON is unchanged in shape, so
      old saved workflows load into the grid fine.

## Wave A-3: project open/switch, IP style (2026-07-20 #17)

You can now keep separate project databases (balam.duckdb, minas.duckdb, …) and
switch between them inside the app (ROADMAP §4c item 2). Project ribbon tab, new
group left of Appearance:

- [ ] **New Project…** creates a fresh, empty .duckdb and switches to it — import a
      couple of Balam South LAS files there, confirm they do NOT appear in your main
      project, then switch back.
- [ ] **Open Project…** switches to an existing file; **Recent ▾** lists the last 12
      projects (current one marked ●, deleted files greyed "(missing)"), stored in
      `%APPDATA%\SandiBumi\projects.json` — outside any project.
- [ ] On switch: window title + group caption show the project name, well list /
      plots / catalogs all reload, well selection and undo history clear (old-project
      undo entries would corrupt the new one — deliberate).
- [ ] **Next launch reopens the last project you had open** (falls back to the old
      `project.duckdb` if the recents list is empty — first launch after this update
      behaves exactly as before).
- [ ] Switching is refused while a workflow chain is running (try it: start a long
      chain, then Open Project — you should get a clear error, not a corrupted run).
- [ ] Note: **Project ▸ Project ▸ Save Project As…** stays a backup copy (app keeps working on the
      current file) — tell me if you'd rather it switch to the copy, IP-style.

## Wave A-2: compact import ribbon (2026-07-20 #16)

The Data tab's eleven flat import buttons are now three Office-style dropdowns
(ROADMAP §4c item 4) — same handlers, just organized:

- [ ] **Import Logs ▾** (LAS, DLIS), **Import Data ▾** (Core, SCAL, Tops, Aux,
      Deviation), **Export LAS** (unchanged flat button), **Tools ▾**
      (Autocorrelate Tops, Shift Core, Well Header). Run one import of each kind —
      behaviour must be identical to the old buttons; tooltips moved onto the
      menu entries.
- [ ] Only one menu opens at a time; picking an item or clicking elsewhere closes it.
- [ ] Bahasa Indonesia / Basa Sunda: the new labels translate (Impor Log / Impor
      Data / Alat) including the previously untranslated Import Tops / Import Aux /
      Autocorrelate entries.

## Wave A-1: tool panes + theme compliance (2026-07-20 #15)

Four tools moved from popup dialogs to dock panes (ROADMAP §4c item 14) — they now
dock/float/tab like the Workflow Builder and can't be dismissed by a stray click:

- [ ] **Cutoffs & Pay Summary**, **ML Models**, **Monte Carlo**, **SandiMin** ribbon
      buttons each open a PANE (singleton: clicking again focuses the existing one).
      Run each on Balam South data — results should be identical to the old popups.
- [ ] The ＋ add-panel menu on any window now lists all four (under Workflow Builder);
      the right-click menu inside each pane shows its own heading.
- [ ] SandiMin's endpoints matrix now uses the full pane width (was capped at 620px).
- [ ] Panes reopen after an app restart (from the autosaved workspace) in their
      docked position — internal selections (cutoff values etc.) reset, same as the
      Workflow Builder.
- [ ] **Theme check** (switch to Dark, then Pertamina): the log-view cursor readout
      pill now inverts with the theme (was unreadable in dark); crossplot/Pickett/
      histogram pick swatches + histogram pick markers follow the theme accents
      (Pertamina = blue/lime, was always brown/green); core-plug diamond outlines
      visible in dark; workflow invalid-input red and error text use the theme warn
      color; the composite preview surface is no longer light grey in dark themes.

## Chartbook overlay library + audit quick fixes (2026-07-20 #14)

The single D-N overlay grew into a **chart overlay library** (Properties → Overlays →
Chart overlay): every crossplot-family chart from your 2013 chartbook, digitized from
the PDF vector artwork with the same validation stack (graduation sequences, 5-multiple
long dashes, worked examples). Charts matching the current axes are listed first; a
chart draws only when the plot axes actually match it (either orientation).

- [x] **CNL Por-11/12** (as before, now via the new select — old saved props migrate).
- [x] **EcoScope Por-18 (BPHI) / Por-19 (TNPH)** on an LWD well — these are the ones
      that matter for your Mahakam development wells; check a known sand against the
      sandstone line for both BPHI and TNPH inputs.
- [x] **adnVISION675 Por-16** if you have ADN wells.
- [x] **APS Por-13/14** (APLC and FPLC variants listed separately).
- [x] **PEF: Lith-3/4** on a PEF-RHOB crossplot — quartz ~1.65-1.8, calcite ~5.08,
      dolomite ~3.1 curves with 10-pu labels.
- [x] **Sonic-neutron Por-20** (both time-average AND field-observation families) on
      a DT-NPHI crossplot — TA curves reproduce Wyllie with tf 190 to R² 0.99999.
- [x] **Density-sonic Por-22** (TA + FO) on a DT-RHOB crossplot, with the 7 mineral
      points (Sylvite, Salt, Trona, Gypsum, Sulfur, Polyhalite, Anhydrite).
- [x] **Th-K clay chart Lith-2** on a POTA-THOR crossplot — the Th/K ratio fan is
      drawn at the _labeled_ ratios (the chartbook's own printed lines sag a few %
      off their labels; ours are exact), plus the dashed clay/feldspar lines and
      mineral-field labels. Judge your Mahakam illite/kaolinite mix against it.
- [x] **Pe-K and Pe-Th/K clay boxes Lith-1** (the Th/K variant needs the X axis in
      log mode — turn on X log in Properties).
- [x] **Umaa-Rhomaa MID Lith-6** — the ternary triangle with 20/40/60/80 subdivisions + K-feldspar/Barite/Anhydrite/Kaolinite/Illite/Salt points. Needs computed
      UMAA/RHOMAA curves (equation engine for now; a dedicated module is a good next
      increment if you want it).

**Audit quick fixes** (from the full senior audit — see AUDIT-2026-07-20.md and
ROADMAP §4b for the 35-finding backlog):

- [x] **Pay summary change**: with a PERM cutoff active, samples with **missing PERM
      now FAIL the cutoff** (they silently passed before). Re-run a pay summary on a
      well with patchy PERM — net pay may legitimately decrease. Tell me if you'd
      rather missing-PERM samples pass (the reference suite's default behavior differs by setup).
- [ ] **LAS import**: the file's own ~W NULL declaration is now honored (deliveries
      using -99999 etc. no longer import sentinels as data), and **multi-word well
      names survive** ("BALAM SOUTH-01" no longer truncates to "SOUTH-01"). Re-import
      one such file and check the Wells pane name.
- [ ] **Depth scale presets are now TRUE ratios** (1:200 = 1 m of well per 5 mm of
      screen at standard DPI). They were ~39x too compressed before, so 1:200 will
      look much more stretched than you're used to — the numbers are honest now.
- [ ] **Tops editor**: adding a top with an existing name is an overwrite; Ctrl+Z now
      restores the previous depth instead of deleting the top.
- [x] Case-insensitive computed-curve lookup (lowercase equation outputs now resolve).

## P2-f+ — D-N chartbook overlay (2026-07-20 #13)

Digitized from the Schlumberger 2013 chartbook you sent (Por-11 fresh / Por-12 salt,
extracted from the PDF's vector artwork — graduation-dash positions, not eyeballed;
calcite identity check rms 0.13 pu, both charts' worked examples reproduce).

- [x] **Crossplot Properties → Overlays → D-N chart**: pick _Fresh mud (Por-11)_ on an
      NPHI-RHOB crossplot → quartz/calcite/dolomite curves appear with porosity
      graduation dots + labels every 5 pu, dashed iso-porosity connectors, and curve
      names written along the lines. Compare against your paper chartbook page 225.
- [ ] **A real Mahakam sand interval** should plot on/left of the quartz sandstone line
      (shale pulls points right/down toward higher NPHI). Crossplot porosity read off
      the graduations should match your PHIE within ~1-2 pu in clean sand.
- [x] **Salt variant** (Por-12) shifts the curves left at high porosity — only relevant
      if you ever work salt-mud wells; check it renders and the graduations differ from
      Fresh.
- [x] **Zoom/pan**: the overlay must stay registered to the data under Ctrl+wheel zoom
      (it's drawn in data space). Also check the flipped orientation (X=RHOB, Y=NPHI).
- [ ] **Gating**: on a GR-RHOB plot or with a log axis the overlay silently stays off
      (chart geometry only means something on linear NPHI-RHOB).
- [x] **Note**: the chartbook draws its dolomite curve for ρma **2.85** (validated
      against the chart's own graduation ticks), while the _Matrix points_ overlay keeps
      the textbook single point at 2.87 — so Dol point and Dol curve start won't
      coincide exactly. Tell me if you'd rather I move the matrix point to 2.85.

## Fix batch from your o/x review (2026-07-19 #2)

Your full review is triaged in **ROADMAP.md §4** — these five landed immediately:

- [x] **Ctrl+wheel = zoom** on Histogram / Crossplot / Pickett. Plain wheel now scrolls the
      page/pane like you asked; hold **Ctrl** to zoom toward the cursor. Drag-pan and
      double-click-reset unchanged.
- [x] **Pertamina theme** rebuilt from your swatch card: blue #006BB8 (accent), green
      #A6C210 (secondary), red #ED1A2F (warnings/alerts), text #161B22 on white. If you'd
      rather have **red** as the main accent (it's the dominant brand color), say so —
      one-line swap.
- [x] **Theme dropdown**: "Light" is now called **Default** (also translated: Bawaan / Baku).
- [x] **Advance tab regrouped**: a single **Advance Methods** group holds SSC, SSPW, RtC,
      IMTS and **Thin Beds** (moved out of Petrophysics — its old dropdown is gone). The
      wrong "Sand-Silt-Clay" caption over SSPW is gone.
- [x] **Multimin → SandiMin**: the generalized solver button/dialog is now **SandiMin —
      Mineral Solver** (original name, no plagiarism concern). The legacy fixed 4-component
      "Multimin — Mineral Inversion" is **removed from the Saturation dropdown** (mineral
      solving is independent of Sw); it still runs inside saved workflow chains. Tell me if
      you want the legacy one back as its own button.
- [x] **Blurry text fix** (your answer: blurry; your display is at 100% scale, so it's not
      Windows scaling): the desktop app now launches WebView2 with `--enable-lcd-text`,
      which forces ClearType subpixel antialiasing on GPU-composited panels (dockview
      layers otherwise fall back to fuzzy grayscale smoothing). **Needs the `npm run tauri
dev` restart** (config change). Look closely at ribbon/dialog text afterward — if it
      still reads soft, next steps are a base-size bump 12→13px and/or semibold.
- [ ] **T-S triangle now appears** (your "not showing (?)"): the triangle is drawn on
      VSH (0–1) vs PHIT axes — before, ticking it on the default NPHI-RHOB crossplot put
      every line off-scale, so nothing visibly happened. Now ticking **T-S triangle**
      auto-switches the X/Y axes to the well's VSH/PHIT curves (status bar tells you), and
      if the well has no VSH/porosity curves yet it says to run those modules first.
      Check: tick it on a fresh crossplot → axes flip, triangle + drag handles visible.

## P1-a — Interaction safety batch (2026-07-19 #3)

- [x] **Right-click lockdown**: right-click anywhere that has no SandiBumi menu (ribbon,
      buttons, tables, empty space) → **nothing** appears (the WebView menu with its
      dangerous Refresh is gone). Panel backgrounds still show our own menus; right-click
      inside a text box still shows the normal cut/copy/paste menu.
- [ ] **Reload guard**: press **F5** or **Ctrl+R** → a blocking confirm appears instead of
      an instant refresh; Cancel keeps everything, Reload restarts the workspace. Alt+←/→
      and the mouse back/forward side-buttons do nothing.
- [x] **Double-click-to-edit numbers** (app-wide): single-click any numeric parameter
      field (module dialogs, plot properties, SandiMin, zones…) → it focuses with a dashed
      outline but typing/arrows/wheel change **nothing**; **double-click** → solid outline,
      value selected, editing works. Tab-into-field still edits directly (deliberate).
      Scrolling a dialog with the wheel can no longer spin a value.
- [x] **Workflow Builder is a pane**: Petrophysics → Workflow… now opens a docked
      **Workflow Builder** pane (tab, movable/floatable like any panel) instead of a popup.
      No more losing a half-built chain to a stray click; it survives layout changes and
      reopens via the ＋ panel menu too. Run/cancel/progress unchanged; closing the pane
      mid-run cancels the chain.

## P1-b — Crash safe-mode, autosave, unsaved markers (2026-07-19 #3)

- [x] **Autosave**: the workspace (panes, arrangement, active well, every log view's
      layout) autosaves every 10 seconds. Nothing to click — just know it's there.
- [x] **Crash recovery**: if the app dies abnormally (crash, force-kill, power loss),
      the next launch shows a choice **before** anything loads: _Restore autosaved
      workspace_ (everything back as it was moments before the exit) or _Start in Safe
      Mode_ (clean default layout; the autosaved workspace is stashed as a "Recovered …"
      session under Open Session, so nothing is lost). To test without crashing for real:
      end the task from Task Manager while the app is open, then relaunch.
- [x] **Normal restart is less lossy now**: on a clean exit + relaunch, the app also
      brings back the **active well** and each log view's **layout/track state** (before,
      only the pane arrangement survived).
- [ ] **Unsaved markers**: edit a log view (track widths, properties, curve visibility)
      → its tab shows **●**, the **Project ribbon tab** gets an amber dot (visible without
      leaving the tab you are on), and **Project ▸ Session ▸ Save Session…** gets a red dot. **Save
      Layout** clears that panel's ●; **Save Session** clears everything. The dot means
      "not in a named save yet" — the crash autosave protects you regardless.

## P1-c — Log sets: versioning, provenance, catalog search (2026-07-19 #3)

- [ ] **Never overwrite**: every module dialog now has an **Output set** field (default
      INTERP; type any name — FINAL, TEST, …). Run a module, then re-run it with different
      parameters: the Curve Catalog's "Log sets" section shows **v1 AND v2** — the old
      run's values are kept, not destroyed. Plots/log views show the latest (v2).
- [ ] **Restore a version**: in Inspector → Curve Catalog, click **Restore** on v1 → all
      open log views and plots flip back to the v1 curves. Restore v2 to return.
- [ ] **Per-curve provenance**: the catalog now lists every computed curve's **set + version,
      module, and timestamp** (hover a set row for the exact parameters and input curves
      it was run with). Answering "where did this VSH come from?" is now one glance.
- [ ] **Catalog search/filter/sort**: one search box matches mnemonic, set, module, unit,
      or date; click any column header (Mnemonic, Set, When, n, Min, Max, Mean…) to sort,
      click again to reverse. Statistics (n/min/max/mean) shown per computed curve.
- [ ] **One version per chain run**: the Workflow Builder also has an Output set field —
      a whole chain run (VSH → porosity → Sw) lands as ONE version, not one per step.
- [ ] **Prune old versions**: Delete on a set version (two clicks — it asks "Confirm
      delete") removes only that version's history; current curves are never touched.
      Equation runs land in set EQUATION, ML in ML, SandiMin in SANDIMIN, automatically.
- [ ] **Input set** (the other half of set in/out): run VSH into Output set **FINAL**,
      then re-run with different parameters into **INTERP** (current values are now
      INTERP's). Open a module that consumes VSH (e.g. sw_indo), set **Input set =
      FINAL** → the run uses FINAL's VSH, not the current one. Blank Input set = normal
      behavior (latest values). Works in the Workflow Builder too; curves the input set
      never wrote (GR, RHOB…) still come from the usual sources.

## P2-a — Tops-style imports (2026-07-19 #4)

- [ ] **Import Tops…** (Data tab): pick a CSV or TXT tops file. With a WELL column
      (WELL/WELLNAME/UWI…) every matching project well gets its tops in one import —
      names match case-insensitively, unmatched names are reported in the status bar.
      Without a WELL column the tops land in the selected well. Columns understood:
      TOP/MARKER/SURFACE/FORMATION/HORIZON + DEPTH/MD/TOP_MD; also bare headerless
      "NAME DEPTH" text lines. Delimiters auto-detected (comma / semicolon / tab /
      spaces). Re-import updates depths but keeps colors you've set.
- [ ] **Import Aux…** (Data tab): petrography, XRD, or perforation data for the
      selected well (or a custom-named dataset). Needs a TOP/DEPTH column; a
      BASE/TO column makes rows intervals (perforations); every other column becomes
      an item — numbers (mineral %, grain size) and text (status, remarks) both kept.
      Re-importing a dataset replaces only that dataset for that well.
- [ ] **View it**: Data → DB Inspector → table "Aux Data" shows the imported rows
      per well (read-only — re-import the file to change values). Tops appear
      immediately in the Wells & Tops pane and all log views/correlation.

## P2-f — Crossplot v2 (2026-07-20 #12)

- [x] **Properties dialog**: double-click or right-click the crossplot (or ⚙ Properties)
      → sectioned dialog (Plot / Axes / Z color / Regression / Overlays). The old
      always-visible properties row is gone; the toolbar is just X/Y/Color/Zone.
- [x] **Marginal histograms + percentiles**: enable marginals on NPHI-RHOB — X histogram
      on top, Y histogram on the right, aligned with the axes (RHOB's inverted axis
      included). Percentiles `25, 75` draw dashed reference lines on both axes.
- [x] **Regression options**: on a PHIE-vs-PERM cloud try Power + RMA — the fit line
      must be straight on log axes and curved on linear ones, equation + R² + method
      tag shown top-left. Compare Y-on-X vs RMA slope on a noisy cloud (RMA steeper).
- [x] **Log-safe Z coloring**: color by PERM with "Log Z scale" + Viridis — low and high
      decades must stay distinguishable (rainbow + linear crams everything in one hue);
      the color bar is labeled "(log)".
- [x] **Plot size**: set Fixed 500×400 — the plot stops stretching with the pane
      (consistent exported figures). "Fill panel" restores the old behavior.
- [x] **Universal defaults**: Qtz/Cal/Dol matrix points no longer appear on NPHI-RHOB
      unless ticked in Properties; Color has a "— None —" option (custom point color
      applies); the pick rows + drag handle can be hidden ("Show parameter pickers" —
      still ON by default so your drag-to-set-shale-point workflow is unchanged).

## P2-e — Histogram v2 (2026-07-20 #11)

- [x] **Properties dialog**: double-click or right-click the histogram plot (or the ⚙
      Properties button) → one dialog holds display mode (bars/line), bins, normalize,
      cumulative overlay, box plot, color, percentiles, statistics placement, and the
      parameter-picker toggle. When zoomed, the first double-click resets the zoom, the
      next one opens properties.
- [x] **Box plot + cumulative overlay together**: enable both on a GR histogram — the
      P25–P75 box with P50 line and P5/P95 whiskers sits under the marker labels, and
      the cumulative % curve (secondary color, % labels on the right edge) tracks the
      bars. Zoom in with Ctrl+wheel: box and whiskers follow the axis.
- [x] **User percentiles**: type `10, 90` in Properties → P10/P90 marker lines on the
      plot and removable chips above it (click a chip to drop that percentile). Values
      must match what you'd read off the cumulative curve.
- [x] **Statistics inside the plot**: set Statistics → "Inside the plot" (chips hide) or
      "Both" — the in-plot block shows the active stats incl. new Min/Max. Check it in a
      dark theme too (block background must follow the theme).
- [x] **Universal by default**: a fresh histogram opens with NO Pick A/B rows and clicking
      the plot does nothing — enable "Show parameter pickers" in Properties to get the
      GR_MA/GR_SH picking workflow back. Your saved bar color / percentiles / etc. must
      survive closing and reopening the panel.

## P2-d — Log-view layout interaction (2026-07-19 #10)

- [x] **Collapsible track headers**: ▤ in the log-view toolbar cycles full → compact
      (curve names as inline chips, no scale lines) → titles only. Headers also cap at
      ~a third of the pane and scroll inside, so a 15-curve track can't eat the screen.
      Try it on your densest layout.
- [x] **Move/copy curves between tracks**: drag a curve name from one track header onto
      another track's header — the curve MOVES there (its color/scale/fill travel with
      it). Hold **Ctrl** while dropping to COPY instead (e.g. overlay NPHI on the GR
      track). Ctrl+Z undoes either.
- [x] **Track borders**: ▦ in the toolbar — solid / dashed / none, width 1–4 px, theme
      color (follows light/dark) or a custom color. Default is a thin solid separator
      at every track boundary; check it looks right in dark themes too.
- [x] **Readout follows ONE track now**: hovering shows only the curves of the track
      under the cursor (not all 15). CLICK a track to lock the readout to it (header
      highlights, click again to release) — then you can run the cursor over the whole
      layout while reading just that track's values.
- [x] **Right-click log editing**: right-click on a track → "Edit CURVE…" for each of
      its curves. Ops: **Wireline shift** (whole-curve depth shift, resampled onto its
      own grid — NaN where it slides past the logged interval), **Set constant**,
      **Blank (erase)**, **Interpolate across** (bridge a bad interval linearly),
      **Scale a·v + b** (recalibration). Works on raw (GR/RHOB…), computed, and
      imported generic-store curves alike; every apply is ONE Ctrl+Z entry that
      restores the previous samples bit-exactly, and lands in the History panel.
      Suggested check: blank a washout interval on RHOB, interpolate across it,
      then Ctrl+Z twice — the original curve must come back exactly.

## P2-c — Well pin rework + multi-select (2026-07-19 #9)

- [x] **Pin is now a mode, not a lock.** 📌 ON (default): clicking a well in Wells &
      Tops moves EVERY log view and plot to it — the old behavior. 📌 OFF: each view
      keeps the well it's showing; only the panel you're working in (the active tab)
      follows your clicks. Open two log views, turn the pin off, activate the second
      view, click different wells — only the second view changes. That's the
      side-by-side compare workflow.
- [x] **The old lock is gone**: no more "Active well is locked" blocking when you
      click other wells, and no more weird interaction with a second wells pane.
- [x] **Multi-select**: Ctrl-click wells to build a selection (highlighted with an
      accent edge, count shown in the Wells label), Shift-click for a range,
      ⇄ inverts within the visible list, plain click clears it. Then open any batch
      dialog (module run, Workflow Builder, Multimin, ML, Monte Carlo, Cutoffs &
      Summary) — the multi-selected wells come pre-ticked instead of just the active
      well.

## P2-b — Petrel-style tops editor + autocorrelation (2026-07-19 #4/#13)

- [ ] **Tops lines in the log view**: every log view now draws the well's tops as
      colored labeled lines across all tracks (like the correlation view). They track
      pan/zoom exactly and repaint on theme change.
- [ ] **🏷 edit mode** (log view toolbar): toggle it on, then — **click** an empty
      depth to add a top (name/depth/color dialog, name auto-uppercased); **drag** a
      line to move it (dashed preview while dragging); **double-click** a line to
      rename, change color, or delete. Mouse-wheel zoom still works while editing.
      Everything is undoable (Ctrl+Z) and instantly visible in Wells & Tops, other
      log views, and correlation.
- [ ] **Crossing warnings**: after any pick/move, SandiBumi compares this well's top
      order with every other well. If a pair is reversed vs the majority (e.g. TOP_B
      above TOP_A here but below it elsewhere), a ⚠ warning appears in the status bar
      naming the pair and the vote (e.g. "below it in 4 of 5 other wells").
- [ ] **Autocorrelate…** (Data tab): pick a top in the selected (source) well, choose
      the log (GR default), pattern window ±m and search range ±m — SandiBumi slides
      the source log shape over each target well (active group) and proposes the
      best-match depth with its correlation coefficient r. Strong matches (r ≥ 0.7)
      come pre-ticked; weak ones are dimmed for your judgment. **Apply** writes the
      ticked picks as ONE undoable batch. Try it on a marker you know — e.g. pick an
      MFS on GR in one Balam well and propagate to the rest, then check r values
      against your hand picks.

Issues you marked `[x]` that need real work (all in ROADMAP §4, P1/P2): well-pin
semantics rework, right-click lockdown (accidental refresh), TVD depth scale UI.
Everything you marked `[o]` has been cleared out of this file.

## Theme switch repaints everything immediately (2026-07-19)

- [x] Open a log view + histogram + crossplot, switch Dark ↔ Default ↔ a client theme —
      every pane recolors instantly, no mouse-over needed
- [x] Switch theme while a second tabbed panel is inactive, then activate it — correct colors

## SandiMin — the reference suite-parity mineral solver (2026-07-19, v2)

Rebuilt to the reference suite Multimin / IP Mineral Solver conventions (spec extracted from your
the reference install helpset + IP2018 install → `docs/multimin_ref_spec.md`, `docs/multimin_ip_spec.md`).

- [ ] **Advance → SandiMin…** now shows the full IP mineral list, grouped: 12 minerals (Calcite,
      Quartz, Dolomite, Orthoclase, Albite, Anhydrite, Halite, Gypsum, Pyrite, Siderite, Muscovite,
      Biotite), 6 clays (Glauconite, Kaolinite, Chlorite, Illite, Montmorillonite, Clay — each with
      an editable **CEC**), and 7 zone-typed fluids (Water Sxo / Water Sw / BoundWater / Oil Sxo /
      Oil Sw / Gas Sxo / Gas Sw; "flushed"/"unflushed" badges). Defaults: Quartz, Illite,
      Water Sxo, Water Sw.
- [ ] **Input logs**: 16 tools — Density, Neutron, Sonic, Total GR on by default; PEF, U, spectral
      Th/K/U, Vp, Vs, EPT, EATT, Sigma optional; **CT (Unflushed Conductivity, from RES_DEEP)** on
      by default and **CXO (from RXO)** optional — CT/CXO take a RESISTIVITY mnemonic; the backend
      converts to conductivity (dual-water linear: Ct^(1/w) row, w = 0.75m + 0.25n). Their σ is
      auto (0.03·C^(1/w)) unless you type one. **+ Add user-defined input** adds a custom log with
      its own endpoint column (default σ 0.015, the reference suite's user-defined default).
- [ ] **Endpoints matrix**: editable per component×tool; unflushed-zone fluid cells show "—" for
      nuclear tools (only CT sees them — the reference suite convention); CT/CXO cells show "auto"; per-row
      **Max** bound (fluids default 0.5, the reference suite's cap).
- [ ] **Fluid properties** panel (visible when CT/CXO on): Rw@temp, Rmf@temp, formation temp, m, n,
      mud type. The preview line shows the computed w, Cw, Cmf, Cbw, α(x/u) and auto CT/CXO σ —
      sanity-check Cw against your Pickett Rw (Cw = 1/Rw@FT, mho/m).
- [ ] **Run** on a Balam well with RHOB+NPHI+DT+GR+RES*DEEP: writes VOL*\* per component +
      MM_PHIE, MM_PHIT, MM_SWE, MM_SWT (+ MM_SXOT, MM_MOVEDHC when both zones present),
      MM_VSH (clays + bound water), MM_RECON. Check: **Σ(minerals + unflushed fluids) ≈ 1**,
      **MM_SWT is sensible vs your sw_indo/RtC runs** (this is the new resistivity coupling —
      "resistivity convert to ct and cxo" as requested), and MM_RECON spikes where the model fails.
- [ ] Add **BoundWater** with Illite selected: VOL_BOUNDWATER should track ≈ 0.18×VOL_ILLITE at
      ~150°F (the the reference suite dual-water bound-water constraint, k = 96·CEC·ρ/(T°C+298)).
- [ ] Add **Oil Sxo + Oil Sw** with CXO available: SXOT ≥ SWT in water-based mud (WATER MUD
      constraint) and MM_MOVEDHC = unflushed HC − flushed HC ≥ 0 across invaded pay.
- [ ] Requested upgrade (ROADMAP §4 P3): optional **nonlinear Sw equation iterated to
      convergence** inside the solve loop.

## ML suite (2026-07-19)

- [x] **Petrophysics → ML Models…** opens the Machine Learning dialog (non-blocking, like all
      dialogs now). Four tasks: regression, classification, clustering, reduction — algorithm
      list, hyperparameters, and default output name switch with the task.
- [x] **Field-wide electrofacies**: task = clustering, K-Means or GMM, check GR first in the
      input curves, check ALL wells under Apply — one model over the pooled samples, so class
      ids are consistent across wells (class 0 = cleanest by GR). Set the output (FACIES_ML)
      to "Facies blocks" in a layout and compare wells side by side (📌 pin one panel).
- [x] **Supervised prediction**: task = regression, target = a curve you trust (e.g. CPERM-
      calibrated PERM or RHOB in a well where it's good), train on wells that have it, apply
      to a well missing it. Check r2_cv5 in the metrics table before trusting the output.
- [x] **Classification with core/interpreted labels**: target = FACIES (or an imported
      lithology curve), train on interpreted wells, apply elsewhere — writes ML_CLASS +
      ML_CLASS_PROB; PROB should dip where the log character is ambiguous.
- [x] **PCA/t-SNE**: reduction task writes PC1..PCn (metrics show explained variance %) or
      TSNE1/TSNE2 — crossplot TSNE1 vs TSNE2 colored by FACIES to sanity-check cluster
      separation. t-SNE refuses >20000 samples by design.
- [x] **DBSCAN noise**: noisy/rare samples get NaN (empty in a blocks track), noise_pct in
      metrics. If everything is noise, raise eps.
- [x] Machine needs Python with numpy + scikit-learn (already present — the test suite used
      it); xgboost optional (falls back to sklearn boosting with a note in metrics).

## GMM soft electrofacies (2026-07-19)

- [ ] **Run "Electrofacies (GMM, soft)"** (Petrophysics → Facies dropdown) on a well where you
      already ran the k-means Electrofacies: FACIES_GMM should broadly agree with FACIES in
      clean intervals. Add FPROB to a track (0–1): it should dip at facies boundaries and in
      mixed/transitional beds — that dip is the point of the module.
- [ ] **Crossplot QC**: color a crossplot by FACIES_GMM (categorical palette + F0/F1/… legend,
      same as FACIES); optionally set FACIES_GMM to "Facies blocks" fill in a layout.

## Click-through fix batch (2026-07-19) — remaining item

- [x] **Monte Carlo / Batch buttons no longer clipped.** Petrophysics tab: Workflow, Monte
      Carlo, and Field Dashboard now sit in one row inside the Batch group.

## FACIES block track (2026-07-19)

- [ ] **Facies layout renders colored blocks.** Run Electrofacies on a well (Petrophysics →
      Electrofacies), then pick the new built-in "Facies" layout in the ribbon layout picker:
      the FACIES track should show solid colored blocks (same colors as the crossplot's
      categorical Z-coloring), with gaps where FACIES is missing. The track header shows a
      striped swatch and "class blocks" instead of a min/max scale.
- [ ] **Blocks survive pan/zoom and well switching**, and the header swatch toggles the whole
      track's visibility like any other curve.
- [ ] **Any discrete curve can be block-rendered.** Layout Properties → a curve's Fill
      dropdown now has "Facies blocks" — try it on FLAG_PAY in a custom layout.
- [ ] **Composite export shows the blocks.** Export a composite (SVG or PDF) with the Facies
      layout: the FACIES track should print as colored rectangles at true scale.

## Electrofacies — k-means (Phase 10 increment 1, 2026-07-18)

- [x] **Petrophysics ribbon → Facies → "Electrofacies (K-means)…"**: pick input curves
      (defaults GR + RHOB + NPHI + DT + SP; leave a slot blank/absent and it's dropped),
      set **K** (number of facies, 2–12) and a **seed**, run on one or several wells. It
      writes a **FACIES** curve (integer 0..K-1). Re-running with the same seed must give
      identical facies (deterministic).
- [x] **Facies numbering is monotone in GR**: FACIES 0 should be your cleanest/sandiest
      class and the highest index your shaliest — confirm on a well where you know the
      sand/shale split. (Clustering is **per well**; the GR ordering is what makes the
      numbers roughly line up between wells.)
- [ ] **Crossplot QC**: open a Crossplot, set **Color = FACIES**. Points should be colored
      by discrete class from a qualitative palette with a **swatch legend (F0, F1, …)**
      top-right — not the blue→red continuous ramp.

## Monte Carlo uncertainty (2026-07-18)

- [x] **Petrophysics ribbon → Batch → "Monte Carlo…"**: pick a chain (the default VSH→φ→Sw, or
      one you saved in the Workflow Builder), click **+ Add uncertain parameter**, choose a
      parameter, pick a distribution (normal / uniform / triangular), set cutoffs + iterations,
      and **Run**. You get a per-well-per-zone table of **P10/P50/P90** net pay, NTG, avg PHIE,
      avg SWE and HPV, plus an **HPV histogram** (click a row to switch zones) with P10/P50/P90
      markers.
- [x] Requested upgrade (ROADMAP §4 P3): **finalize parameters → print LOW / BASE / HIGH
      curves** from the chosen result percentiles.

## Phase 8.5 — your method suite in core (remaining validations)

- [ ] **SSC — Sand-Silt-Clay (Advance tab)**: run on an LQR-style well with
      GRN + RHOB + NPHI (sandstone units). Check VSAND/VSILT/VDCL/VWCL, PHIT/PHIE/PHIFF,
      CBW/CWSH/BW, SWIRR_T/SWIRR_EFF and the `*_GR` GR-equivalent volumes against your
      the reference suite run. Defaults are the LQR `.info` values (wet clay 2.3/0.6, dry clay 2.71,
      wet silt NPHI 0.3, DCLF_SI 0.1). Two deliberate deviations, flag if they matter:
      (1) `RANNORMAL(SWIRR_MIN·PHIT, 0.005)` is deterministic here; (2) the Loglan's
      NPHIMA limit 0.5–5 (a copy-paste of the RHOMA limit) is corrected to 0–1.
- [ ] **SSPW (Advance tab)**: the Loglan exec body wasn't on disk, so the
      arithmetic (PHIT from VSH-mixed dry matrix, CBW = VSH·VOL_CBW_SH,
      CAPBW = VSH·(PHIT_SH − VOL_CBW_SH), PHIE = PHIT − CBW, PHIFF, SWIRR floor) is
      **reconstructed from the spec — please validate against your the reference suite "LAS PHIT
      PHIE" exports** and tell me any systematic difference; I'll adjust the equations.

## Phase 8b — report generator (2026-07-18)

- [x] **Report… dialog**: select a well first, then set study title, author, cutoffs
      (VSH ≤ / PHIE ≥ / SWE ≤ / optional PERM ≥), layout + print scale + page size, and
      **Render** — page through the preview (◀ ▶). Check: cover (title/well/field/
      interval/TD/KB), methodology table, zone parameter table (from your zone_params),
      pay summary (SAND/RESERVOIR/PAY rows with gross/net/NTG/avg PHIE/VSH/SWE/HPV —
      needs VSH+PHIE+SWE computed curves), then the composite pages.
- [x] **Methodology table is editable**: one line per row, `Parameter | Method | Remarks`.
      Blank = a built-in default reflecting your standard workflow. **Save Template**
      persists it (documents table) and it reloads next time.
- [x] **Save PDF…** writes the whole report as one multi-page PDF — open in Acrobat and
      check the tables (word-wrap in Remarks cells, header row repeated on overflow
      pages) and that ≤/≥ symbols render.
- [x] **Save PNG (page)…** rasterizes the CURRENT preview page at ~150 dpi for slide decks.
- [x] **Batch (N wells)…** exports one report PDF per well into a folder you pick,
      named `<WELL>_report.pdf`, using the same settings for every well. Wells that
      fail (no curves) are reported without aborting the rest.
- [x] **Tables only** checkbox skips the composite pages (fast parameter/pay-summary
      handout).

## Field Dashboard (Phase 9 increment 4, 2026-07-18)

- [ ] **By zone** table aggregates across wells: well count, Σ net, Σ HPV, mean N/G,
      net-weighted mean PHIE/SWE per zone.

## Deferred small item (Phase 7)

- [ ] **QC plot for sat-height**: the Pc/J-vs-Sw QC plot with the fitted curve + core
      points overlaid is NOT built yet — the `get_scal_pc` IPC is ready for it. Say "go"
      when you want it.

## Module-panel cleanup, Help tool, bulk Processing report, responsive resize (2026-07-21)

Five asks from your VSH-panel screenshot (SandiMin deferred for later review).

- [ ] **Module form no longer lists per-well results**: run a module (e.g. VSH from
      Gamma Ray) — the form now shows one summary line ("All N well(s) computed. Per-well
      details are in the Processing panel." or "…N need attention…") and the Processing
      panel comes forward. The old `✓ well: samples → curves` list is gone from the form.
- [ ] **Per-well detail lives in Processing → details**: expand a job's **▸ details** —
      running wells show individually; the narration paragraph that used to sit at the top
      of the module form is gone (it moved to Help, below).
- [ ] **Bulk failure report**: when many wells fail the SAME way, Processing → details shows
      **one card per reason** — "N well(s) failed — <message>", the well list (first 12 +
      "…(+K more)"), and a "→ what to do" advice line — instead of one row per well.
- [ ] **Help (?) tool**: click the **?** in the top quick-access bar (or right-click any
      panel → **Help for this panel…**). A guide opens for whatever panel is active — a
      module pane shows that method's description; other panels show a short blurb. (This is
      the placeholder that will later link to the full illustrated help library.)
- [ ] **Ribbon dropdowns still work**: open **Petrophysics → VSH/Porosity/Saturation**, and
      **Data → Import Logs/Import Data/Tools**, and **Project → Recent** — each menu must drop
      fully below the ribbon (this was a regression the review caught and fixed).
- [ ] **Resize the whole window**: the content panes (log views / plots / inspector) reflow to
      fill; **Wells & Tops, Processing, and Performance keep their width** (they're a fixed
      sidebar now). Try both wider and narrower — nothing should get clipped or leave dead space,
      and the ribbon stays reachable (scrolls if very narrow).
- [ ] **Close panes without the sidebar growing**: close a plot/log view — the freed space goes
      to the other content, NOT to the sidebar. Close *everything* down to just the sidebar and a
      blank **Workspace** pane fills the rest (rather than the sidebar stretching); open any log
      view/plot and the blank pane disappears again.

## Multi-well crossplot overlay (T-SHELL-16 increment 1, 2026-07-30)

Crossplot only in this increment (histogram is next; Pickett needs a decision — its
m/n/Rw are per-well parameters). Design: extra wells draw as a FADED CONTEXT LAYER
behind the active well; everything interactive stays on the active well.

- [ ] **Wells: Active button** in the crossplot toolbar (after Zone) — click it to open
      the well-scope row: Active / Group / ★ Pinned / Selection / All / Custom…, the same
      control as the batch dialogs. Default **Active** = today's single-well plot, unchanged.
- [ ] **Pick a wider scope** (e.g. All, or a group): the other wells' points fade in
      BEHIND the active well's cloud, one colour per well, with a **Wells legend** top-right
      (active well first; long names truncate; >10 wells collapse to "+N more"). The legend
      footer says "context is display-only".
- [ ] **Context wells are display-only**: brushing (Shift+drag), the draggable parameter
      handle, zone-parameter writes, core overlay, T-S endpoints, regression, tooltips and
      the net polygon all still act on the ACTIVE well only — check the brush highlights
      only active-well points and log views follow only its depths.
- [ ] **Zone windows resolve per well by NAME**: with a zone (or a selected top) chosen,
      each context well shows ITS OWN depths for that same-named zone/top — wells without
      it are skipped and counted in the scope row ("N skipped"), never guessed from the
      active well's depths.
- [ ] **Point budget**: a huge scope decimates context wells to ~60k points total — the
      scope row reports "~N pts (decimated)". The active well is never decimated.
- [ ] **Axis auto-range covers the field**: with context wells on and auto ranges, a
      neighbour whose cloud sits outside the active well's spread is still visible (not
      clipped); manual ranges and mnemonic defaults (NPHI/RHOB…) behave as before.
- [ ] **Scope survives a well switch**: set scope All, click another well in the Wells
      pane — the rebuilt crossplot keeps scope All (and the new active well takes over the
      interactive role). SVG/PDF/PNG export includes the context layer.

## Multi-well histogram overlay (T-SHELL-16 increment 2, 2026-07-30)

Same scope treatment as the crossplot, adapted to distributions: context wells draw
as **stepped outline curves** behind the active well's bars, one colour per well.
The comparability rule: **each context well is normalized to its own sample count**
and scaled to the active axis — you compare distribution SHAPES, so a neighbour
with 3× the samples never dwarfs the active well (this is the GR-normalization
use case). Pickett is deliberately NOT scoped yet — your call pending (m/n/Rw are
per-well parameters).

- [ ] **Wells: Active button** in the histogram toolbar (after Zone) — same scope row
      as the crossplot. Default **Active** = today's single-well histogram, unchanged.
- [ ] **Wider scope**: context wells appear as stepped outlines behind the bars, with a
      **Wells legend top-left** (active well first with a filled swatch, context wells
      with line swatches matching how they render; footer "context: per-well shape ·
      display-only"). Works in bars and line mode, count and Normalize-% mode.
- [ ] **Shape, not size**: overlay a small zone of a big well — its outline peaks near
      the active well's bars (same shape → same height), NOT 3× above them. In
      Normalize-% mode the outline is that well's true per-well percentage.
- [ ] **Pooled X range**: a context well whose distribution sits outside the active
      well's P2–P98 (e.g. an unnormalized hot GR well) stretches the axis so its curve
      is visible, not clipped. Single-well range behaviour unchanged.
- [ ] **Stats stay active-well**: chips, P5/P50/P95/mean markers, user percentiles, box
      plot, cumulative curve, picks A/B and the brushed sub-distribution all still read
      the ACTIVE well only — context outlines never move a statistic.
- [ ] **Same zone-by-name + skip rule** as the crossplot; scope row reports counts,
      decimation and skips. Scope survives a well switch; SVG/PDF export includes the
      outlines and legend.

## Pickett v2 completion + multi-well overlay (T-SHELL-16 increment 3, 2026-07-30)

The Pickett already had free M/N/Rw fields, Properties (axes, point size, Z-colour)
and viewport-preserving N changes from an earlier pass. This increment adds the rest
of the audit items plus the scope overlay. The multi-well decision, as agreed: the
**overlay shows whether neighbours share the ACTIVE well's water line** — m, n and Rw
are per-well parameters and never come from a context well.

- [ ] **Wells: Active button** in the Pickett toolbar (after Zone) — same scope row as
      the other plots. Default **Active** = today's single-well plot, unchanged.
- [ ] **Wider scope**: context wells' clouds fade in behind the active well's, one
      colour per well, Wells legend top-right, footer "context is display-only". The
      water-line readout adds "line = ACTIVE well's parameters" whenever context is on.
      A neighbour sharing the water line hugs the same Sw=1 edge; one with different Rw
      sits visibly shifted — that's the point of the overlay.
- [ ] **Water-line picks, M/N/Rw, brushing, tooltips, zone writes**: all still act on
      the ACTIVE well only. Clicking two points fits M/Rw from the active cloud even
      with context wells showing.
- [ ] **Template bar** (★ Save template / recall / 🗑) — Pickett display settings
      (axes, point size, Z-colour) now save under a name like Histogram/Crossplot.
      Recalling a template with garbage values is safe (everything sanitized).
- [ ] **New default RT axis 0.2–2000 ohmm** (audit fix — 0.1–1000 clipped
      high-resistivity pay). Your saved axis ranges are untouched; only a fresh
      panel/profile sees the new defaults.
- [ ] **Sw lines span the visible window**: set a custom porosity range (e.g.
      0.02–0.5) or zoom — the Sw = 1 / 0.5 / 0.25 lines run edge to edge instead of
      stopping at the old fixed φ = 0.01–1 span.
- [ ] **Scope survives a well switch**; SVG/PDF export includes the context clouds
      and legend. Same zone-by-name + skip rule, budget and scope-row reporting as
      the other two plots.

## MID plot module — UMAA / RHOMAA (2026-07-30)

Feeds the Lith-6 chart overlay that has been sitting in the chartbook library with
nothing to plot on it. New **Lithology** category in the Petrophysics ribbon.

- [ ] **Petrophysics → Lithology → Apparent Matrix (MID plot)** runs on a well with
      RHOB + NPHI + PEF and writes four curves: **UMAA**, **RHOMAA**, **U** (volumetric
      photoelectric factor) and **PHIA** (the apparent porosity it actually used —
      exposed so the basis is never hidden).
- [ ] **Crossplot X = UMAA, Y = RHOMAA** opens on the chart's own window (UMAA 0–16,
      RHOMAA 2.2–3.1 with density increasing downward). Properties → Chart overlay now
      lists **"Lith-6 Umaa-Rhomaa MID plot"** under *For these axes* — switch it on and
      the quartz / calcite / dolomite triangle, the clay and anhydrite points and the
      percentage lines land around your cloud.
- [ ] **Read a known carbonate or a clean sand** and check the cloud sits where the
      lithology says it should. Please push back if the placement disagrees with your
      chartbook reading — the analytic apparent porosity is the one approximation here.
- [ ] **The porosity basis is a visible choice** (OPT_PHIA in the run dialog), and the
      default **CHART** now reads the density-neutron crossplot the way you would by hand
      on Por-11 — it solves for the porosity at which both tools imply the same matrix,
      interpolating across the chartbook's sandstone / limestone / dolomite curves. Pick
      the curve family with **TOOL** / **SALINITY** (same choices as Neutron Matrix
      Conversion). **NPHI must be in apparent-limestone units** — run Neutron Matrix
      Conversion first if your log is recorded in sandstone or dolomite units.
- [ ] **Compare CHART against XPLOT on a dolomite or mixed-carbonate interval**: XPLOT
      (the analytic average commercial suites take, kept for comparison) leaves dolomite
      about 0.06 g/cc light and 0.34 b/cm³ left of its chart point; CHART puts it on the
      dolomite line. If your chartbook reading disagrees with CHART, that's the one I
      most want to hear about.
- [ ] **Anhydrite / pyrite intervals stay heavy** rather than dropping out (denser than
      every matrix line, they clamp to the end of the search and plot in the chart's
      high-RHOMAA corner). **Gas** pushes points low-left, exactly as on the printed
      chart — the module does not "fix" gas, so the gas signature stays readable.
- [ ] **Density-only porosity is deliberately absent** — it is algebraically degenerate
      (it returns the assumed matrix density for every sample, a constant curve that
      would still plot convincingly). There is a unit test stating the trap.
- [ ] **Barite mud warning** is in the method note: PEF is unreadable there. Run with
      **Mask = BADHOLE** on rugose intervals.
- [ ] **Over-porous samples drop out** as blanks rather than as huge numbers (PHIA_MAX,
      default 0.5, is an editable parameter — not a hidden constant).

## Per-well parameter override table (Phase 9-2, 2026-07-30)

The last open Phase 9 item. A workflow step carries one parameter set for every well,
which breaks when a field needs a different Rw per fault block. The storage already
allowed the fix (a `zone_params` row with zone `*` is a whole-well override, and runs
already apply it) — what was missing was a way to reach it for more than one well at a
time. **Resolution order is unchanged: step value → this whole-well override → named
zone.** Nothing about how your existing runs resolve has moved.

- [ ] **Petrophysics → Batch → Workflow…**, build or open a chain, then **Per-well
      parameters…** next to Run. Rows are wells, columns are the numeric parameters the
      chain's steps actually take.
- [ ] **Grey = inherited, amber = overridden.** A fresh grid is all grey (every well
      inherits the step value). Double-click a cell to give one well its own value — it
      turns amber. The cell tooltip tells you which it is.
- [ ] **Double-click to edit, not single-click** — same rule as every other numeric field
      in the app, so a stray click near a parameter can't change it. Enter commits, Escape
      cancels, blank clears the override.
- [ ] **Typing the inherited value back clears the override** (cell returns to grey)
      rather than storing a duplicate — the same only-store-differences rule the per-step
      editors use.
- [ ] **Columns marked ⚠ behave differently on purpose.** If two steps in the chain take
      the same parameter with *different* step values (e.g. Archie RW 0.05, Indonesia RW
      0.07), the header shows ⚠ and the column displays only the first step's number.
      There, typing the displayed value **stores** it instead of clearing — because
      clearing would leave the two steps disagreeing again, when what you meant was "this
      value for every step in this well". Hover the header for the explanation.
- [ ] **Out-of-range values are refused with a status-bar message, not clamped.** Try
      entering RW = 25 (a v/v value typed as a percentage). This matters: the run itself
      REJECTS an out-of-range override and fails the whole chain, so catching it here turns
      a failed 2000-well run into a red cell.
- [ ] **Set for all shown / Clear for all shown**: pick a column, type a value, and every
      well currently listed takes it in one write. Narrow the list first with the **Wells**
      scope (All / Group / ★ Pinned / Selection / Custom) and the **Filter** box — the
      buttons act on exactly what you can see.
- [ ] **One Ctrl+Z reverses a whole sweep.** Set a column across 50 wells, then undo once —
      all 50 revert together, not one per press. Redo re-applies them.
- [ ] **Copy as CSV** puts the shown grid on the clipboard so you can diff it against your
      own well table in Excel. *(CSV import back is the obvious next step and is NOT built
      yet — tell me if you want it, it's small now that the write path exists.)*
- [ ] **Zone parameters still win.** Set a whole-well RW here and a different RW on one
      zone (Zones panel) — the zone value should govern inside that zone and the grid value
      everywhere else. This is the check that matters most.

---

## 2026-07-30 — Example import datasets (`dataset for test/examples/`) + BLSO core header fix

One folder with a working exemplar of EVERY import format, pooled where you asked:
`dataset for test/examples/`. Three synthetic wells (SANDI-01/02/03) with shared,
physically consistent geology — a gas sand and a water sand whose core, SCAL and log
values all agree by construction. The `README.md` in that folder is the map: each file →
exact ribbon menu → what the status bar should say → what each parser accepts (the full
alias lists), so you can shape a confusing real delivery against the nearest analogue.
These files are ALSO parsed by `cargo test` on every gate run (`example_data_test.rs`) —
if a parser ever changes in a way that would break the published examples, the gate goes
red. Regenerate with `py -3 tools/make_example_data.py` (deterministic).

- [ ] Data → Import Logs ▾ → **Import LAS…** → multi-select the three `SANDI-*.las` →
      3 wells, ~394 rows each; PEF/CALI appear in the Curve Catalog (set RAW); Bad-Hole QC
      flags the deliberate 1-m washout gap mid-SAND-A.
- [ ] Follow the README's numbered import order (tops → locations → deviation → core →
      3 SCAL shapes → petrography/XRD/perforations). Every import should succeed with the
      README's stated result — any deviation is a bug, tell me.
- [ ] N/D crossover shows gas in SAND-A on any well; Archie in SAND-B gives Sw ≈ 1 —
      the README's "known-good expected values" section is the eyeball checklist.
- [ ] **Real-data fix:** your BLSO core-log delivery (`blso*_lapi2023_core.csv`,
      `03. Core Logs`) now imports grain density — the `GDEN_1` header resolves (it
      silently dropped before). CPERM_1/CPOR_2/CSW_1 already resolved; the FEET units row
      is skipped safely. Re-import one BLSO core CSV and check CGD in the DB Inspector.

---

## 2026-07-30 — Import sets: one well, many deliveries (T-IMP-02, -03, -04, -06)

Your Geolog screenshots, built. A delivery folder is now a **set**: `01. Final Log`'s RAW,
FPROOH, MULTIMIN, SSC and SSPW can all land on **one** well record instead of five
same-named ones, and you can see which is which.

- [ ] **Import LAS… now asks first.** A "Import LAS — curve set" dialog opens with the set
      name already filled from what your filenames share: pick the FPROOH folder's files and
      it suggests `FPROOH`; MULTIMIN suggests `MULTIMIN`. Verified against all five of your
      BLSO folders. Blank = RAW.
- [ ] **Attach to existing wells (default ON).** Import blso00025 from **RAW**, then again
      from **FPROOH**, then **MULTIMIN** — you should end with **ONE** well carrying three
      sets, not three wells. The status line says how many were new and how many attached.
- [ ] **A set name is never overwritten.** Import the same FPROOH folder twice: the second
      lands as `FPROOH_1` (Geolog's WIRE → WIRE_1 rule). Nothing from the first import moves.
- [ ] **▸ twisty in the Wells pane** expands a well into its sets, and a set into its curves
      (mnemonic + unit; hover for sample count, family, run number). Both FPROOH's PHIE and
      MULTIMIN's PHIE are visible under their own sets — that was the whole ask.
- [ ] **Existing projects behave EXACTLY as before.** This is the check that matters most:
      **set RAW keeps absolute priority** in curve resolution. A module asking for PHIE still
      gets RAW's PHIE when RAW has one; only a mnemonic RAW does *not* carry (e.g. `PHIFF`,
      `VOL_QUARTZ`) is looked up in the attached sets. Re-run a module you have run before
      and confirm the numbers are identical.
- [ ] **Import DLIS… also asks for a set name.** Give a second tape its own name and both are
      kept instead of the second replacing the first — your "we don't always know what's
      inside" point. Leaving it as RAW keeps the old replace-and-count behaviour.
- [ ] **The malformed exemplars you asked for now exist** (you wrote "where do u provide
      dup_depth.las?"): `dataset for test/examples/bad_dup_depth.las` imports with a
      dropped-duplicates warning and 35 rows; `bad_null_depth.las` fails cleanly and creates
      no well row. Both are asserted by cargo test.

*Not built, and worth saying plainly:* selecting files from **two different sets in one
import** (e.g. an FPROOH and a MULTIMIN file together) finds no common name, falls back to
RAW, and mixes them — one import batch is one set by design. Import per folder.

---

## 2026-07-30 — Core & aux import v2: the "hundred wells with cores" workflow (T-IMP-07/-09/-10/-11)

Import Core is now **probe → confirm → commit**: nothing is written until you have seen and
approved what the file means. Your note is the spec: well names come FROM THE DATA, every
property column is confirmed first (name, type, unit, percent), and 1-or-many CSV **or
TXT/tab-delimited** files work in one action. BLSO is just the exemplar — the reader takes
any delimited text and shows each column's sniffed type (number/text/empty).

- [ ] **Data → Import Core… with NO well selected** → pick
      `dataset for test/examples/core_rcal_multiwell.csv` → the wizard shows: WN as the well
      column, 3 wells with row counts, the units row detected and skipped, depth unit `m`,
      CPOR/CSW flagged as percent, and a 5-row preview. Import → plugs land on all three
      SANDI wells by name.
- [ ] **Real data:** multi-select ALL 321 files in `03. Core Logs\BLSO_LAPI2023_CORE` in one
      Import Core. The mapping is confirmed once (by header name) and applied per file;
      depth unit should read `ft` from the units row and convert to the project unit.
      Unmatched well names are listed by name, never guessed.
- [ ] **The Duri trap:** import the parent folder's `Core.csv` — it has a numeric `WELL`
      column (804) AND a textual `WELL NAME` (DURI00804). The wizard must pre-pick **WELL
      NAME** (a pad number can't route rows); check the routing line before importing.
- [ ] **Wrong mapping is refusable:** change Depth to a text column, or blank the well
      column with no well selected — Import refuses with a reason, writes nothing.
- [ ] **Import Aux… routes by WELL now:** pick `xrd_multiwell.txt` (tab-delimited) with any
      well selected → rows land on all three wells; the result box names unmatched/blank
      rows. A file with no WELL column still binds to the selected well as before.
- [ ] **Shift Core (T-IMP-09) is unblocked** — run it as written in the test plan; it still
      shifts the SELECTED well's plugs only.

---

## 2026-07-30 — Core import: the EXTRA columns come in too ("any column, any data type")

`core_data` holds four measurements (porosity, permeability, grain density, Sw). A real lab
export is wider — lithology descriptions, So, Kv/Kh, sample IDs, tape names. Those columns
now ride along from the SAME wizard: they land as **point data at the plug depths**, typed
per cell (numbers as numbers, anything else as text), so a wide delivery imports whole in
one pass instead of needing a second Import Aux run.

- [ ] **Import Core… → `dataset for test/examples/core_rcal_multiwell.csv`** (the exemplar
      now carries `SO_1`, `LITH` text and mixed `SAMPLE_ID`). Tick **Extra columns** → the
      5 leftover columns appear with their type (`LITH (text)`, `TAPE_NAME (empty)`…).
      Untick `TAPE_NAME`/`TOOL_STRING`, leave the rest, Import → the status line reports the
      plugs AND "Plus N point-data value(s) from SO_1, LITH, SAMPLE_ID".
- [ ] **Check the values landed as themselves:** Database Inspector → `aux_data`, dataset
      `CORE` — `LITH` in value_text ("SANDSTONE"), `SO_1` in value_num, depth_base empty
      (they are point samples), depths matching the plug depths.
- [ ] **A column can't be stored twice:** with Extra columns on, re-point **Water saturation
      (CSW)** at `SO_1` — it leaves the extras list immediately and `CSW_1` takes its place.
      Columns you unticked stay unticked.
- [ ] **Dataset name is yours:** change "Store them under dataset" to e.g. `CORE RCAL`;
      re-importing the same file replaces that dataset for the well (same discipline as the
      plugs themselves), it never doubles up.
- [ ] **Real data (the point of it):** a BLSO core CSV's extra columns, or the Duri
      `Core.csv` wide export — everything the four core slots don't claim is available
      without a second import pass.

**Note by design:** extras are stored **verbatim** — no percent→v/v or feet→metres
conversion is applied to them (the depth they hang on IS converted). The wizard confirms
what a column *is*; it does not reinterpret its values. If you want a specific extra
treated as a real curve/measurement instead, tell me which and it becomes a mapped role.

---

## 2026-07-30 — Core sets & survey versions: nothing overwrites anything (T-IMP-08 / T-IMP-12)

You marked T-IMP-08 **Fail** with "refer T-IMP-02 about how duplicated data managed", and
T-IMP-12 the same. That is now the rule for core and surveys as well: **one delivery = one
named set, and an import never overwrites an earlier one.**

One difference from curve sets, on purpose: curve sets are read TOGETHER (a set supplies
mnemonics RAW lacks). Two core deliveries measure the SAME plugs, so reading both would
double your φ-k cloud. Exactly **one core set and one survey are ACTIVE** per well, and
everything reads that one — log overlay, crossplots, HFU, SandiMin calibration, Shift Core,
DB Inspector edits, TVD/TVDSS.

- [ ] **Import the same core file twice.** Import Core suggests a set name from the filename
      (`blso00025_lapi2023_rcal.csv` → `RCAL`). Second import → status says
      `Core set RCAL_1 — 1 well(s) already had a 'RCAL' set, so theirs was suffixed`. Both
      deliveries are kept, the newest is live.
- [ ] **The plug count does NOT double.** Open a φ-k crossplot or the core overlay after that
      second import — same number of points as one delivery, not two.
- [ ] **Data → Tools ▾ → Data Sets…** on that well: both sets listed with plug
      count, source file and import date, ● on the live one. Click **Use** on the older one →
      the plots repaint to that delivery. **Delete** asks first; deleting the live one hands
      over to the next newest (never leaves plugs no panel can see).
- [ ] **Surveys:** import a preliminary survey (`SURVEY`), then a definitive one
      (`DEFINITIVE`). Both listed; TVD at TD reflects the definitive. Switch back with **Use**
      → status says TVD/TVDSS was rebuilt, and TVD at TD changes back. This is the part worth
      checking hardest on a real deviated well — a stale TVD would quietly feed every
      height calculation.
- [ ] **Your existing projects:** open one that already has core and/or a survey. It migrates
      on launch (a backup copy is written beside the project first, per the release rule), the
      old data appears as set/survey **RAW**, active, and **every number reads exactly as
      before**. Check a φ-k plot and a TVD you know.
- [ ] **Duplicated depth inside ONE file** still drops first-kept with the note — that is a
      broken row in a single delivery, not a second delivery.

---

## 2026-07-30 — …and the same rule for EVERY point dataset, plus the tree

Your note: *"not only core, any kind of point data should behave universally like core — we
have a lot such xrd, cec, oil show, etc."* Right — those all live in one store, and until now
a second delivery of any of them silently replaced the first. They now version exactly like
core: **one delivery = one named set, one live per (well, dataset)**.

- [ ] **Import Aux… now has a Set field** (default `RAW`). Import an XRD file twice → the
      result box says `Set RAW_1`, both deliveries are kept, the newest is live, and the
      panel counts show ONE delivery's values, not the sum.
- [ ] **Datasets are independent.** With XRD switched to the older delivery, CEC / oil show /
      perforation stay exactly as they were — activation is per dataset, not per well.
- [ ] **Wells pane ▸ twisty** now shows, under each well: its curve sets (as before), then
      **Core**, **Surveys** and **Point data** with ● on the live one.
      **Double-click** a dimmed row (○) to make it live — panels repaint. Single click does
      nothing on purpose, so a stray click in a long well list can't repoint your data.
      Deleting stays in the manager dialog.
- [ ] **Core extras follow their core set** — a core file's LITH/So/sample-id columns are
      stored under the SAME set name as the plugs, so switching a well's core switches its
      extras with it instead of leaving a mismatched pair.
- [ ] **Old projects:** point data predating this is adopted as set `RAW`, active — your XRD
      and petrography read exactly as before. (Unlike core, this needs no table rebuild.)

---

## 2026-07-30 — SCAL deliveries version too; the manager is now "Data Sets…"

The last store that still overwrote on re-import. A capillary-pressure report is now a named
delivery like everything else — **the files you select together in one Import SCAL are ONE
set** — and only the live one feeds Pc QC, the Leverett-J fit and Thomeer.

- [ ] **Import SCAL… has a SCAL set field** (default `SCAL`). Import a centrifuge set, then a
      porous-plate report → status says `Set SCAL_1`, both are kept, the newest is live, and
      the Pc QC plot shows ONE report's points.
- [ ] **Switch back** in **Data → Tools ▾ → Data Sets…** (renamed — it now has four sections:
      Core, SCAL, Deviation surveys, Point data) or by double-clicking the row in the Wells
      tree → the Pc plot and any J-fit you re-run follow the other report.
- [ ] **Old projects:** existing Pc points are adopted as set `SCAL`… actually `RAW`, active —
      your saturation-height work reads exactly as before.

That completes the sweep: **curves, core, SCAL, surveys and every point dataset now version
the same way.** Nothing in the app silently overwrites a delivery on re-import any more.

---

## 2026-07-30 — Field-scale open hardening: memory cap, Compact Project, visible upgrades

From your BLSO report (2.5 GB file, ~6 GB RAM, 15-minute open). The 15 minutes was the two
one-time storage upgrades each backing up the whole project first — but the file itself was
~75% dead space (632 MB of live data in a 2,487 MB file), the engine was allowed ~80% of the
machine's RAM, and all of it happened silently. All three fixed:

- [ ] **Second open of BLSO is fast.** The upgrades ran once; reopening the project should
      take well under a minute now. If it is still slow, tell me — that would be a
      different problem than the one fixed here.
- [ ] **RAM stays civil.** With BLSO open, SandiBumi's memory should sit near 4 GB at the
      very worst (the engine is capped at min(≈20% of RAM, 4 GB), spilling to disk beyond
      that instead of taking the machine). Power users: set `SANDIBUMI_DB_MEMORY=8GB` in the
      environment to raise it on a big field machine.
- [ ] **Data → Tools ▾ → Compact Project…** on BLSO: after the confirm, the status line
      should report roughly `2,487 MB → ~630 MB`, everything still opens and plots, and the
      original file is parked beside the project as `.pre-compact-<ts>.duckdb` — delete it
      yourself once satisfied. Every table's row count is verified before the swap; any
      failure puts the original back untouched.
- [ ] **Save Project As now compacts too** — it exports through the engine (live rows only),
      so a Save As of a bloated project lands at its true size.
- [ ] **Nothing silent any more:** opening a project that needs a one-time upgrade shows
      "Opening project… (a first open after an update can run one-time storage upgrades…)"
      while it works, and afterwards the status line + History panel say what ran, how long
      it took, and where the backup went.

---

## 2026-07-30 — Audit backlog #128: long operations no longer freeze the window

Follow-on from the open-hardening work. Anything that can run for minutes was still executing on
the app's main event-loop thread, so while it worked the window itself was frozen — Windows shows
"not responding", nothing repaints, no button responds. Six such operations now run on a worker
thread. (Chain/ML/SandiMin runs were already off-thread; this closes the rest.)

- [ ] **Open Project on BLSO** (or any large project): the window stays alive and repainting the
      whole time, the status line's "this can take minutes" message is readable, and the app is not
      greyed out / "not responding". This is the one worth checking first — it is the operation you
      hit the 15 minutes on.
- [ ] **Compact Project** and **Save Project As** on BLSO: same — the window stays responsive
      while gigabytes are rewritten. Panels that need the database will pause until it finishes
      (correct — they must not read a half-swapped project), but the window itself never freezes.
- [ ] **Recompute TVD/TVDSS Curves** across many wells: window stays alive.
- [ ] **SQL Query panel**: run a deliberately heavy query (e.g. a join over `computed_curves`
      with no WHERE). It should be interruptible-feeling — the window stays responsive instead of
      locking up until the query returns.
- [ ] **Nothing changed in behaviour** — same results, same errors, same undo. This increment is
      purely *where* the work runs.

**Startup itself is fixed in the next section.**

---

## 2026-07-30 — The window now opens before the project does

The last and worst version of the same problem: SandiBumi opened your project *before* creating
its window, so during those 15 minutes there was **nothing on screen at all** — you double-clicked
and the machine appeared to ignore you. Now the window comes up immediately and the project opens
behind it.

- [ ] **Launch on BLSO:** a window appears within a second or two, showing a small
      **"Opening project…"** card with a moving bar and a running clock. The app is visibly alive
      and on screen the whole time. After ~20 seconds it adds a line explaining that a first open
      after an update upgrades the project's storage, backs it up first, and happens only once.
- [ ] **The card tracks what the backend is doing** — when the storage upgrade starts, its message
      changes to name the backup file it just wrote.
- [ ] **A normal (fast) launch shows no card at all** — open a small project; it should go
      straight to the workspace with no splash flash.
- [ ] **Afterwards**, the History panel and the status line record how long the open took and what
      ran, so a slow launch has an explanation you can go back and read.
- [ ] **Nothing appears before its data is ready** — no empty well list, no "0 wells" flash. The
      workspace is not built until the project is genuinely open. **If you ever see an empty
      Wells pane on a project that has wells, tell me — that would mean the gate leaked.**
- [ ] **A broken project still explains itself:** the existing "could not open" dialog still
      appears (now after the card, not instead of a window).

---

## 2026-07-30 — Imports no longer refuse a file over its text encoding

Your Duri core table failed with `Core import failed: io error: stream did not contain valid
UTF-8`. The cause, found in the bytes: **330 KB of pure ASCII except two `0x95` bytes** — the
Windows bullet "•" that opens a lithology description — and the whole delivery was refused over
two characters in a comment field. Any file that has been near Excel or Word can carry those
(smart quotes, en/em dashes, °, µ).

Every text import now decodes tolerantly: a byte-order mark is honoured first (so Excel's
"Unicode text" UTF-16 export works too), then UTF-8, and anything left falls back to Windows
cp1252 — which cannot fail, so **an import is never refused over encoding again**. This covers
core, LAS, tops, aux/point data, SCAL and deviation alike, not just the file that reported it.

- [ ] **Import your Duri `Core.csv`** — it should now read 12 columns, **3,045 plugs across 15
      wells** (DURI00513 … DURI01887), depth detected as **ft**, and CPOR/CPERM/CGD(GDEN)/CSW
      mapped automatically. The DESC / LITH / CORE_NO / KV / CSO columns are offered as extra
      point-data columns in the same wizard.
- [ ] **The bullet survives as a bullet** in the description, not as a `?` or a black diamond —
      check a DESC value in the Database Inspector after import.
- [ ] **Nothing else changed**: re-import an ordinary UTF-8 or plain-ASCII file (BLSO core, a
      LAS) and confirm identical results to before.

---

## 2026-07-30 — Wells pane: right-click on everything, and point data expands like curves

Your two asks: expanded items should have a right-click menu (including a route into the Curve
Catalog for editing), and non-curve data should behave like curves — expandable within a set,
with its own menu.

- [ ] **Right-click a curve** (under an expanded set) → Open in Curve Catalog · Edit name /
      unit / family… · Make this curve win its name · Delete. "Open in Curve Catalog" should
      land on the Inspector's Catalog tab **already filtered to that curve**, not on a list of
      everything.
- [ ] **Double-click a curve** opens the same edit dialog (single click stays inert on purpose —
      these rows sit in the same list as wells, and a stray click must not move the workspace).
- [ ] **Rename a curve and check it took**: `GRN_CS` → `GR` on your Duri well. Values must be
      unchanged (same sample count in the Catalog), and a **GR-based module should now see it** —
      that is the real reason to rename, not cosmetics. **Ctrl+Z undoes it.**
- [ ] **Point data / core / SCAL / surveys now have a ▸ twisty** and expand:
      - Core → the properties its plugs actually carry (`CPOR (61)`, `CPERM (61)`, …)
      - Point data → its named items (`LITH (305)`, `CSO (61)` — your Duri core extras)
      - SCAL → one row per plug with its Pc point count
      - Surveys → station count, MD range, TVD at TD, max inclination
      Only the **live** delivery expands; an inactive one says so rather than showing the
      active one's contents (which would be a lie).
- [ ] **Right-click a delivery** → show contents · make it the live one · Open Database
      Inspector · Data Sets…. Deleting still lives only in Data Sets…, never a stray click.
- [ ] **Right-click a well** → expand · Curve Catalog · Database Inspector · Data Sets… · pin.

---

## 2026-07-30 — Blocky curves and crossover shading

Your two display asks: "option to display curves as continuous or blocky style", and "we also
don't have shading to other logs". Both live in the same place — **Layout Properties → the
curve table**, which gained a **Style** column and two new Fill choices.

- [ ] **Blocky (step) curves.** Layout Properties → pick **Blocky** in the new Style column on
      any curve. The value should now hold flat all the way down to the next sample and then
      jump, instead of sliding diagonally between sample centres. Try it on something genuinely
      piecewise-constant — a zone-constant parameter curve, a block-averaged or upscaled log,
      VSH from a coarse pass. **The shading follows the step**: a blocky curve's edge fill is a
      stack of rectangles, not a stack of wedges.
- [ ] **Continuous is still the default** — every existing layout you have saved should open and
      draw exactly as before. Nothing needs re-saving.
- [ ] **Crossover shading.** Layout Properties → Fill → **Crossover to curve**. It auto-picks the
      other curve in the same track as the reference and seeds the two swatches with the two
      curves' own colours, so you can see the separation immediately. **Shading** now shows two
      swatches: left one = where the styled curve reads LEFT of the reference, right one =
      where it reads RIGHT.
- [ ] **The reference must be in the SAME track.** That is deliberate, not a limitation: the
      reference is positioned with **its own min/max**, and compatible scaling is the whole
      meaning of a neutron-density crossover. Naming a curve from another track shades nothing.
- [ ] **The built-in Standard Layout now ships the NPHI/RHOB crossover** (grey where NPHI reads
      left of RHOB — shale / clay-bound water; yellow where it reads right — gas effect). The
      Facies layout's porosity track matches. **Scales are unchanged** (NPHI 0.45→−0.15,
      RHOB 1.95→2.95). Tell me if you would rather the built-ins stayed plain.
- [ ] **Check it on a real gas sand** in BLSO or Duri: the colour should flip exactly where the
      two curves cross, not a sample early or late.
- [ ] **Print agrees with screen.** Plot ribbon → Composite… on a layout using both features —
      the PDF/SVG must show the same blocky steps and the same two-colour crossover.
- [ ] **Bug fixed in passing**: a curve whose Fill you had set to **None** used to print with a
      left-edge shading in the Composite/report PDF even though the screen showed it clean. It
      now prints unshaded. Worth a glance at any deliverable you generated before today.

---

## 2026-07-30 — Point-data tracks: core plugs, XRD, text, box plots and histograms

Your ask: "we dont have any option to show point data, text data, or even image with its own
style option to show it as histogram or box plot per x range interval with its own adjustment
as well such percentile showing, whisker, etc." Images are still to come; everything else is
here. Layout Properties → **Track type → Point data**.

- [ ] **Add a point track**: Layout Properties → set Track type to **Point data** → **＋ Add
      point series**. Source **Core plugs** lists your well's real plug properties
      (CPOR/CPERM/CGD/CSW); source **Point dataset** lists your real datasets — for Duri that
      is CORE with LITH, CSO, KV, and whatever else the wizard carried in as extras.
- [ ] **Points** (default) draws one diamond per plug at its own depth and value. Unlike the
      old core overlay this is a track of its own, so you can scale it how you like instead of
      borrowing a curve's scale.
- [ ] **Text** draws the sample's text at its depth — your `LITH` descriptions, oil show.
      Labels are thinned so a densely described core stays readable rather than a black smear,
      and truncated at the track edge instead of spilling into the neighbour.
- [ ] **Box plot** summarises the plugs inside each depth bin: box edges, median, whiskers,
      outliers. All adjustable per series — **Bin height** (blank = follow the zoom, a value =
      a fixed depth interval that stays put at every scale), **Box low/high %**, **Whiskers**
      (Tukey k×IQR / Percentiles / Full range), and **Show samples** to draw the individual
      plugs as ticks above the box.
- [ ] **The whisker rule is a real choice, so check both.** Tukey answers "which plugs are
      unusual for this interval" and flags outliers individually; Percentiles answers "where
      do 80% of the plugs lie" and flags nothing. Switch between them on a Duri interval with
      a wild plug and confirm the picture changes the way you expect.
- [ ] **Histogram** draws a value-axis histogram per depth bin, bars scaled to that bin's own
      peak count so a thinly sampled interval is still readable next to a dense one.
- [ ] **Nothing is clamped.** A plug outside the track's Min/Max is skipped, not pinned to the
      edge — check by narrowing Max below your highest CPOR and confirming those plugs vanish
      rather than stacking on the right-hand border.
- [ ] **A blank cell is not a zero.** If your core table has an empty CGD column for some
      plugs, those plugs must contribute nothing to a CGD track — not a cloud at 0 g/cc.
- [ ] **Print agrees with screen**: Plot ribbon → Composite… on a layout with a point track.
      Same boxes, same medians, same outliers, same labels.
- [ ] **Existing layouts are untouched** — a saved layout with no point track opens exactly
      as before.

**Note on where this is heading** (your instruction): the box/percentile/whisker machinery is
deliberately written to know nothing about core plugs. It takes a set of values and a depth
bin. That is so **array logs — your 1000-realization Monte Carlo PHIE — reuse it unchanged**,
because 1000 realizations at one depth is the same statistic as 40 plugs over an interval.
When we do array logs, the display options you set here will already mean the same thing.

---

## 2026-07-30 — Array logs: adjustable band, spaghetti and density heat map

This is the array-log increment the point-data note above was written for. The
box/percentile machinery was reused **unchanged** — no second statistics path was created.

**Producing one** (Petrophysics → Batch → Monte Carlo…):

- [ ] **Options** now has **Store realizations (array log)**, greyed out until *Save
      LOW/BASE/HIGH curves* is ticked (it rides the same pass over the kept runs, so on its
      own it would silently do nothing).
- [ ] Run with both ticked. The status line reports the saved curves, and the notes list
      `stored MC_<KEY>_REAL — N depths x M realizations` per well.
- [ ] Only outputs the chain **produces** get a matrix — an input curve it merely reads must
      not come back as a fake zero-width band. (Same rule as the percentile curves.)
- [ ] With more than 256 realizations kept, a note says the stored set is the first 256, so a
      band drawn from it can differ slightly from the MC_*_LOW/_HIGH curves. **Nothing should
      differ silently.**

**Displaying it** (log view → ⚙ → **Track type → Array log** → **＋ Add array series**):

- [ ] The **Array curve** box suggests what this well actually has (`MC_PHIE_REAL`, …). With
      no array logs at all, the panel says so and points at the Monte Carlo option rather
      than offering an empty picker.
- [ ] **Uncertainty band** — shaded P-low to P-high with the P50 line through it.
- [ ] **This is the adjustable part**: change *Band low %* from 10 to 5 (or to 40/60) and the
      band redraws immediately from the same stored realizations. **No re-run.** That is the
      whole reason the matrix is stored rather than just three curves.
- [ ] *Median line* off leaves the shading alone; *Shading* sets the fill opacity.
- [ ] **Spaghetti** — individual realizations. *Traces* sets how many. They are sampled
      **evenly across the run**, not the first N: the first N of a Latin-hypercube design sit
      in one corner of the sampled space and would understate the spread.
- [ ] **Density heat map** — per-depth value histogram, darker where more realizations landed.
      *Value bins* sets the resolution.

**Data-honesty rules to try to break:**

- [ ] **A gap stays a gap.** At a depth where too few realizations converged, the band
      **splits** rather than shading straight through. Shading across it would claim an
      uncertainty range for a depth the study gave no answer for.
- [ ] **A failed realization breaks its own trace** in spaghetti instead of being bridged to
      the next depth — the bridge would draw a path that realization never took.
- [ ] **Off-scale heat-map values are dropped, not clamped.** Narrow the track min/max until
      part of the distribution falls outside: those samples contribute **no** cell rather than
      a false dark column at the track edge.
- [ ] Band and spaghetti, being continuous readings, **clip at the track edge** like any log
      curve — deliberately different from a core plug, which is skipped.

**Print + back-compat:**

- [ ] **Print agrees with screen**: Plot ribbon → Composite… on a layout with an array track.
      Same band, same gaps, same traces, same heat map.
- [ ] **Existing layouts are untouched** — a saved layout with no array track opens exactly as
      before, and an older project migrates without a backup pause (the old `array_logs` stub
      never held a row, so there is nothing to protect).

**Worth knowing:** a stored matrix is the only Monte Carlo output whose size scales with
iterations (~2 MB per curve per well at the 256 default). If a project starts to drag, the
matrices can be dropped without touching the study that produced them, and Data → Tools ▾ →
Compact Project reclaims the space.

---

## Provenance & exposure sweep — Tier A + B applied (2026-07-31)

`docs/provenance_sweep_prompt.md` run end to end: 24 findings, **11 Tier A + 2 Tier B applied**,
6 Tier C and 5 Tier D routed and untouched. Full register with `file:line` in the gitignored
`docs/commercial/PROVENANCE_SWEEP.local.md`; questions for counsel in `LAWYER_PACKET.local.md`.

**Two behaviour changes — check these first, they are the only things that alter what you see:**

- [ ] **GR Normalization defaults changed.** Petrophysics → Prep → GR Normalization now opens
      with `GR_LOW_REF = 20`, `GR_HIGH_REF = 120` gAPI (was 53.68 / 133.93). The old pair was one
      field's regional calibration from 562 wells — somebody else's field standard, shipping to
      every user, and silently wrong anywhere else. The new pair is the app's own generic
      clean/clay endpoints (`vsh_gr`'s GR_MA / GR_SH). **The doc string now tells you to set your
      own field reference** — read it and confirm it says what you would tell a junior.
      *Your real pair is preserved in `docs/commercial/`. Re-runs of old wells will differ; that
      is expected — enter your own reference to reproduce a previous study.*
- [ ] **Python environment variable renamed** to `SANDIBUMI_PYTHON`. Every message that used to
      say `ARSHILLA_PYTHON` — DLIS import, ML, image import, Workbook, Word, Deck, the equation
      editor — now says the new name. **Your existing `ARSHILLA_PYTHON` still works** and is read
      silently; nothing to change on your machine. Confirm by opening Plot → Deliverables →
      Workbook… and reading the message if Python is missing.

**Client material out of the tree:**

- [ ] The 20 hard-coded delivery paths in the `#[ignore]`d field tests are gone. They now read
      **`SANDIBUMI_FIELD_FIXTURES`** — point it at a folder with `las/` and `core/` subfolders
      and the tests use whatever is in it. Verified both ways: unset, all five skip with a
      printed reason; set at the example wells, the core probe resolved 11 headers / 30 rows /
      3 wells and the full chain ran to a pay summary.
- [ ] `dataset for test/Core.csv` — real core plugs from one client well, referenced by no code —
      is **still in the tree**. Removing it is one command; it is left for you because git history
      keeps it either way and that is your decision, not a fix. Same for the tracked
      `Prompt/*.pdf`, which `CLAUDE.md` wrongly claimed was gitignored (now corrected).
- [ ] `Review.txt` / `Review 2.txt` moved to `docs/commercial/` and untracked (superseded by this
      file; one named two client assets).

**Licences — new file:**

- [ ] `THIRD-PARTY-LICENSES.md` now exists: 289 crates, 154 npm packages, **zero undeclared**,
      six weak-copyleft (MPL-family, all transitive, none modified — they permit shipping a closed
      binary). Generated by `node tools/gen-third-party-licenses.mjs`; re-run after any dependency
      change. There is still **no project `LICENSE` file** — that is your text to write.

**One judgement call worth your eye:**

- [ ] `multimin2.rs` cited `docs/multimin_geolog_spec.md` for the incoherence statistic — **a file
      that does not exist**. I replaced it with the primary source I believe is correct: Mayer &
      Sibbit, SPE 9341, *GLOBAL, a new approach to computer-processed log interpretation* (1980).
      Confirm that citation before it is quoted to anyone; it is the one provenance claim in this
      batch I chose rather than found.

**Left alone on purpose** (Tier C/D — do not read these as missed): the four client-branded
themes, the tooltips naming which vendor tables seeded a default, the study citation in `lrlc.rs`,
the RtC regression coefficients (no neutral default exists — that is a petrophysics decision and
it is yours), the 2.9 MB of vendor research extractions, and git history.

---

## scipy in the equation engine (2026-07-31)

Petrophysics → Database Inspector → **Equation Editor**, language **Python (numpy)**. When scipy
is installed in the interpreter SandiBumi picked, your scripts can now use `signal`,
`interpolate`, `optimize`, `stats` and `ndimage` directly — no import line needed. numpy is still
the only requirement; nothing changes if you never touch scipy.

**The note tells you before you write, not after you run:**

- [ ] Open the Equation Editor with language **Python (numpy)**. The grey note under the tab now
      ends with the interpreter path **and** `· scipy 1.18.0`. If scipy were missing it would say
      `· no scipy — install it for signal/interpolate/optimize/stats` — a note, not a warning,
      because the engine is fully usable without it.

**Four things worth trying on a real well** (inputs `GR`, output as named):

- [ ] **Despike** — output `GR_DS`:
      `gr_ds = signal.medfilt(gr, 5)`
      A 5-sample median. Casing collars and washout spikes go; the bed boundaries stay put,
      which a mean filter would smear.
- [ ] **Smooth** — output `GR_SM`:
      `gr_sm = signal.savgol_filter(gr, 11, 2)`
      Savitzky-Golay preserves peak height and shape far better than a running mean.
      **Despike first.** A polynomial fit over an un-despiked curve fits the spike rather than
      the rock — try it both ways on a washed-out interval and you will see it immediately.
- [ ] **Fit your own φ-k** — inputs `PHIE, PERM`, output `PERM_FIT`:
      ```
      import numpy as np
      ok = np.isfinite(phie) & np.isfinite(perm) & (phie > 0) & (perm > 0)
      def model(x, a, b): return a * np.power(x, b)
      p, _ = optimize.curve_fit(model, phie[ok], perm[ok], p0=[1.0, 3.0], maxfev=20000)
      perm_fit = model(phie, *p)
      ```
      Mask the invalid samples yourself — `curve_fit` has no NaN handling and will simply fail.
- [ ] **Resample / fill** — `interpolate.interp1d(depth[ok], curve[ok], bounds_error=False)`.

**Two rules you may want to test deliberately:**

- [ ] **A curve wins a name collision.** If a well ever has a curve called `STATS`, your script
      gets *your curve*, not `scipy.stats`. Your data never yields to a library name.
- [ ] **A missing scipy names the fix.** On a machine without scipy, a script using `signal`
      fails with the interpreter path and the exact `pip install` command — not
      `NameError: name 'signal' is not defined`. Worth checking on a colleague's machine, since
      that is the whole point of the message.

**Also renamed here:** the interpreter override is `SANDIBUMI_PYTHON` (see the previous entry);
your existing `ARSHILLA_PYTHON` still works.

---

## RtC calibration from your own water zone (2026-07-31)

**Advance ▸ Calibrate RtC…** This closes the last open item from the provenance sweep. `sw_rtc`
always told you to "recalibrate per field from water-zone excess conductivity" and never gave
you a way to do it — so in practice one study's coefficients ran on every field. Now you point
it at a water sand and it gives you *your* A_CAP / B_QV / C0.

**Try it on a well where you know the water leg:**

- [ ] Click a water-bearing top in the Tops pane first — the dialog seeds the interval from it.
- [ ] Set Rw / M to **the same values your `sw_rtc` run will use**. They define the clean
      baseline the excess is measured against; a fit against a different Rw is a fit for
      different rock. The dialog says so.
- [ ] Fit, then read **R² and the "Not fitted" line before the coefficients.** If R² is low the
      excess here is not explained by CAPBW and Qv, and the coefficients are not worth having.
- [ ] **Copy**, then paste into the `sw_rtc` parameters. Deliberately not auto-applied — that
      would skip the step that matters.
- [ ] Compare SWE_RTC before and after on a known interval. This is the real test: does your
      own calibration move Sw the way your core and tests say it should?

**Three things to poke at deliberately:**

- [ ] **It refuses without a water zone.** Clear both depth boxes and the flag curve, then Fit.
      You should get a refusal explaining that fitting over pay hands the hydrocarbon's
      resistivity to the clay term. That refusal is the most important behaviour here — over
      hydrocarbon the fit reads Sw too HIGH, so a careless calibration *erases* pay rather than
      inventing it.
- [ ] **Nothing is dropped silently.** The "Not fitted" line counts every excluded sample by
      reason — outside the interval, not flagged wet, incomplete inputs, or "no excess to
      explain" (Rt reads above what clean water-filled rock can be, usually meaning Rw is wrong
      for the interval or it is not actually wet). A calibration from 12 samples of a sand you
      thought held 500 is a different statement.
- [ ] **RSF is held fixed**, and the result says the coefficients are only valid for that RSF.
      Change RSF afterwards and they are void — RSF multiplies the whole bracket, so it and the
      three coefficients cannot be separated by this regression.

**Worth knowing:** with no QV log and CEC = 0 the clay term cannot be fitted at all. It is
reported as **0 with a note** rather than guessed, and the capillary term absorbs whatever
constant clay conductivity is present.

---

## IMTS S-factor calibration from your own lab CEC (2026-07-31)

**Advance ▸ Calibrate S…** Same story as RtC, one module along. `sw_imts` defines S as a
measurement — your lab CEC divided by the CEC the clay model predicts — and the app shipped
**0.5**, which was never measured in any rock. S multiplies the whole clay-charge term, so a
wrong S scales Qv_eff straight through to SwT with nothing on the log to show for it.

**Try it on a well with a CEC suite:**

- [ ] Point it at the dataset and item holding your lab CEC. Get the item name wrong on purpose
      — it should tell you **which items are actually there**, not just "no data".
- [ ] **Name the clay curves your `sw_imts` run will use** (VDCL / VILL by default), not the XRD
      table the CEC came from. This is the trap: calibrate against one estimate of clay and run
      against another and S is wrong by the difference — invisibly, because both are clay
      volumes.
- [ ] Fit, then **Copy** — it copies S together with CEC_KAOL and CEC_ILL, because S multiplies
      those constants and the three are one setting.
- [ ] Run `sw_imts` with your S and compare SWT_IMTS against the shipped 0.5. On clay-rich rock
      the difference should be substantial; that gap is what the placeholder was costing you.

**Read these before the number:**

- [ ] **Plug ratios P10 → P90.** This is the real check, not R². If the plugs' own ratios span
      more than a factor of two, no single S describes them and it says so. Either S genuinely
      drifts with clay content, or the lean plugs are noisy — a small measured CEC divided by a
      small modelled clay volume is a noisy ratio either way.
- [ ] **The "Not fitted" line.** A plug further than the depth tolerance from any log sample is
      **dropped, not snapped** to the nearest one. If most of your plugs land there, the core is
      not depth-shifted to the log. Worth knowing: a shift that happens to be a whole number of
      log samples is invisible to this check — the log grid cannot see it — so the tolerance is
      not a substitute for shifting against core gamma.
- [ ] **Plugs where the clay model says no clay** are excluded rather than divided by zero. If a
      plug there has real measured CEC, that is evidence against your clay curves, not a data
      point.
- [ ] **S above 1** gets flagged. The method expects lab CEC *below* the XRD-theoretical value,
      so above 1 your clay model is under-calling exchange capacity — most often a mineral it
      does not carry. Smectite is 80-150 meq/100g against illite's 25, so a few percent of it is
      enough. That S then only suits rock with the same smectite fraction as your cored plugs.

**Also fixed here:** both calibration dialogs used to open with a blank, greyed-out Fit button
until you touched the well scope. They now label themselves on open.

---

## Calibration QC scatter, on both fits (2026-07-31)

Both calibration dialogs now draw the fit, not just report it. A calibration comes down to two or
three numbers, and **R² tells you how much scatter there is but not what kind** — curvature, one
well sitting off the trend, a cluster of plugs dragging the line. Those only show in the picture.

- [ ] **Calibrate RtC…** now plots **measured against fitted** excess conductivity with a dashed
      1:1 line. On a good fit the cloud straddles the line evenly. A bow above or below it at one
      end means CAPBW and Qv are not linear over that interval — worth knowing before you accept
      the coefficients.
- [ ] **Calibrate S…** plots **lab CEC against modelled CEC** with the fitted line through the
      origin. Deliberately the regression itself rather than measured-vs-fitted, because only this
      version puts clay content on the x axis: a curved cloud is S drifting with clay, a fan
      opening toward the origin is noise on the lean plugs, and a cluster off the line is one core
      suite. That turns the "plug ratios P10 → P90" number into something you can name.
- [ ] **Points are coloured by well**, with a legend underneath. On a field-wide calibration this
      is the question the table cannot answer: is one well pulling it?
- [ ] **Hover a point** — it names the well, the depth, and the values. On the S plot it shows the
      plug depth *and* the log depth it was paired with, so a bad pairing is visible.
- [ ] **⧉ Copy / ⭳ Image / ⎙ Print** on each plot, the same buttons as every other plot.

**Two things to check deliberately:**

- [ ] **The 1:1 line should sit at 45°** on the RtC plot regardless of how wide the scatter is.
      Both axes are forced to the same range on purpose — scale them independently and the aspect
      ratio alone can make a clean fit look biased.
- [ ] **The S plot should always show zero** on both axes even if your plugs are all clay-rich.
      Through-the-origin is the model's claim, and cropping to the data would hide whether your
      cloud actually heads for zero — which is the one thing that would disprove it.


---

## Accepting a calibration writes it, instead of you retyping it (2026-07-31)

Both fit dialogs now have an **Apply** row under the Copy button. It writes the coefficients as
parameter overrides, so the next `sw_rtc` / `sw_imts` run and every workflow chain picks them up
without anyone remembering to. Copy is still there — Apply is the shortcut, not a replacement.

- [ ] Fit, then **Apply** with the default scope. It writes to **the wells the fit actually
      used**, which is not the same as the wells you scoped. Hover the dropdown — it names them.
- [ ] Check the second option: **"all N well(s) in scope (+k never calibrated)"**. That is the
      fit-here-apply-there move, and it should say plainly how many wells are getting a
      calibration they contributed nothing to. Hover it and it names those too.
- [ ] **Ctrl+Z.** One undo reverses the whole sweep. A parameter that had no override before goes
      back to having **no override** — not to zero. If a well already had its own value, that
      value comes back.
- [ ] Leave the zone box **blank** for a whole-well override. That is usually what a saturation
      calibration wants: you calibrate in one interval and apply everywhere. Type a zone name to
      narrow it — it only bites in wells that carry a zone of that name.
- [ ] Open the **per-well parameter grid** afterwards (Petrophysics) and confirm the values are
      there as overrides on exactly the wells you expected.

**The one thing to notice:** Apply writes **RSF along with A_CAP / B_QV / C0**, and **CEC_KAOL /
CEC_ILL along with S_FACTOR**. That is not tidiness. In both fits the constant and the
coefficients are not separable — the constant multiplies the whole term — so coefficients applied
without their constant are a calibration for different rock, and nothing downstream would catch
it. They go in one batch or not at all.


---

## Calibrate S… now offers your data instead of asking you to type it (2026-07-31)

The CEC dataset and item boxes were free text, so the most likely first mistake — getting the
item name wrong — could only be discovered by running the fit. They are now dropdowns built from
what your project actually holds.

- [ ] Open **Advance ▸ Calibrate S…**. The dataset list should show every point dataset with its
      item count, defaulting to CEC (or CORE if you have no CEC dataset).
- [ ] Switch datasets. The item list follows, and each item shows **how many rows and how many
      wells** carry it — so an item present in 2 of your 12 wells says so before you fit it.
- [ ] **A text-only item is greyed out**, marked "no numeric values". A lithology description
      cannot set a scaling factor, and this is the honest way to say so: it stays visible, so you
      can see it is there and see why it is not a choice.
- [ ] If a whole dataset has nothing numeric in it you get "(nothing numeric in this dataset)"
      rather than an empty box.
- [ ] On a project with no point data at all it falls back to typing, and says plainly that there
      is nothing to pick from yet.

**Worth knowing:** the list is built from the **ACTIVE delivery** of each dataset, like every
other point-data reader. Switch a well's CEC set in Data → Data Sets… and the picker follows —
a superseded delivery is not offered as a choice.



`sw_rtc`'s own description now says plainly that the shipped defaults are one field's, and
points at this dialog.

---

_Made in SandiBumi._ © 2026 SandiBumi. All rights reserved.

---

## Register core depth against a log (2026-07-31)

Data ▸ Tools ▾ ▸ **Register Depth…**. Core arrives on the driller's tally and the log on the
wireline's; until now the only tool for the difference was typing a number into Shift Core, which
meant already knowing the answer.

- [ ] Select a cored well, open **Register Depth…**. The **Core reference** list should show every
      plug column and every point-data measurement with enough samples to correlate — and default
      to a **core gamma** if you have one, since that is the strongest reference there is.
- [ ] The note under the reference should say whether the pairing is **like-for-like** (core gamma
      against GR: the same quantity) or a **proxy** (core porosity against GR: different quantities
      that co-vary inversely). Switch the reference and the log curve and watch it change.
- [ ] **Propose a shift.** Check the proposed shift, the correlation, and — importantly — **where
      it sits now**, which tells you whether the proposal improved anything at all.
- [ ] The left plot draws the log with the core **hollow where it sits now and solid where it would
      sit**. The right plot is the **correlogram**: correlation against every candidate shift.
- [ ] **Read the correlogram before accepting.** One sharp peak = the shift is well determined.
      Several near-equal peaks = the section repeats and the maximum is close to a coin toss. The
      dialog counts rival peaks within 5% and says so, but the picture is the real answer.
- [ ] Type a different shift in **Shift to apply** — both plots follow immediately. The proposal is
      a suggestion, never applied on its own.
- [ ] Before applying, check the list of **point datasets that ride along**. A measurement made on
      a plug must move with that plug or it ends up registered against rock it was never taken
      from. Untick anything that is on the wireline scale rather than the core's.
- [ ] Apply, then **Ctrl+Z**. The status line reports plugs AND point samples moved; the undo puts
      every one of them back.

**Worth knowing:** on a proxy pairing the shift is chosen on the STRENGTH of the relationship, not
its sign — a core porosity should come back with a strongly NEGATIVE correlation, and the dialog
says "inverse" when it does. On a like-for-like pairing a negative correlation is never accepted:
two gamma measurements that run opposite are not aligned, they are wrong, and the proposal you get
in that case deliberately disagrees with the porosity answer so that the disagreement is visible.

**Also changed:** the old **Shift Core…** now moves the core's point data with the plugs too, and
its status line says how many of each.

---

## Plate depths — fixing a picture's depth without re-importing (2026-07-31)

Data ▸ Tools ▾ ▸ **Plate Depths…**. Until now a thin section imported at the wrong depth could only
be corrected by deleting the whole delivery and importing it again.

- [ ] Select a well with pictures and open **Plate Depths…**. The dataset box lists each kind with
      its plate count; the table shows the LIVE delivery, sorted by depth.
- [ ] Each plate shows **point** or **interval**. A thin section is a point — cut from one plug,
      with no thickness — and a core photograph with a base depth is an interval.
- [ ] **Shift every plate by** a constant, with a dataset selected. This is the normal repair: a
      delivery read off one mis-registered tally is wrong by one number. Check the status line for
      how many moved, then **Ctrl+Z**.
- [ ] Correct one plate: edit its top, name or caption and press **Save**. Also undoable.
- [ ] **Leave a base blank and it stays a point sample** — a shift moves it without inventing a
      thickness. Type a base and it becomes an interval (a deliberate claim); clear it again and it
      goes back to a point.
- [ ] Type a base ABOVE the top and press Save. It should be **refused with a message naming the
      plate**, not silently swapped — a reversed pair is a typo worth seeing.

**Worth knowing:** the table only ever shows the ACTIVE delivery of each dataset, like every other
reader in the app. A superseded delivery is untouched by a shift — switch which one is live in
Data Sets… first.

**Still open (your call):** whether thin sections should move automatically when you re-register the
core they were cut from. You said yes but tentatively, so nothing does that yet — the shift above is
deliberate and visible. Say the word and it becomes automatic.

---

## One shift per barrel, and the core remembers it (2026-07-31)

Data ▸ Tools ▾ ▸ **Register Depth…**, the new table at the bottom.

- [ ] Open it on a cored well. Below the single-shift tools there is now **One shift per barrel**.
      A note tells you whether this core has already been moved, and by how much — so a second
      pass is never applied on top of a forgotten first one.
- [ ] Fill a range (top, base) for the first barrel and press **Propose**. It runs the same match
      as before but only over that range, fills in the shift, and draws that barrel's own
      correlogram. **Add a barrel** for the next one.
- [ ] If pieces moved inside a barrel, just split it: two shorter ranges, a different shift each.
- [ ] **Apply all barrels**, then **Ctrl+Z**. Every plug should return exactly where it started.
- [ ] Try shifts that would cross — push an upper barrel down past the one below it. It should
      **refuse, name the two plugs that would cross, and change nothing**. Deeper rock must never
      end up above shallower rock.
- [ ] Try two ranges that overlap. Also refused: a plug can only belong to one barrel. Ranges that
      just touch (2000–2010 and 2010–2020) are fine.

**The part worth testing on real data:** import an XRD or CEC table AFTER you have shifted the core,
still at the depths the lab wrote. The core now remembers where it started, so those samples can be
placed where that rock actually is — including where one barrel moved further than its neighbour,
because the correction is interpolated between plugs rather than applied as one number.

Outside the cored interval there is nothing to go on, so the correction is held from the nearest
end and those samples are marked as extrapolated rather than quietly placed.

**Note on older projects:** a project made before today gets the new record filled in as "no shift
yet". Any shifting you did before this exists is not recoverable, so the core is treated as
delivered where it currently sits. From here on it is tracked.

---

## New data can follow the shifted core (2026-07-31)

Data ▸ **Import Aux…** has a new tick-box: **"These depths came from the core report"**.

This is the payoff for the depth record. A lab sends XRD or CEC written at the depths from the
original core report. If you have since registered that core against the log, those depths are out
by however far the core moved — and the samples would be attributed to the wrong rock.

- [ ] Register a well's core first (Tools ▸ Register Depth…), ideally with two barrels moved by
      **different** amounts so the test is a real one.
- [ ] Import an XRD or CEC file written at the ORIGINAL core depths with the box **ticked**. Each
      sample should land on the rock it was measured from — including across the barrel boundary,
      where the correction is worked out between plugs rather than applied as one number.
- [ ] Import the same file with the box **off** and compare. The depths stay exactly as written.
      That is the right behaviour for a file already on the log's depth scale.
- [ ] Check the message after the import. It should say the samples were **placed from the core
      depth record**, and count any that fell **outside the cored interval** — those have nothing
      to go on, so they keep the nearest correction and are reported rather than placed quietly.
- [ ] Tick the box on a well with no core. It should import the depths as written and say **"no
      core to follow"** rather than looking like it mapped something.
- [ ] Tick it on a well whose core has never been shifted. It should say so — the box worked, there
      was simply nothing to correct.

**Deliberately off by default.** Nothing in a delimited text file reliably says which depth scale
it uses, so this is your declaration, not a guess the app makes.

**Not yet offered for SCAL or image imports** — both also arrive at lab-written depths. Say the
word and they get the same tick-box.

---

## SCAL and pictures can follow the core too (2026-07-31)

The same tick-box — **"These depths came from the core report"** — is now on **Import SCAL…** and
the **image import wizard**, not just Import Aux…

- [ ] Register a well's core first, ideally with two barrels moved by different amounts.
- [ ] Import a SCAL file whose plug depths came from the core report, box **ticked**. The points
      should land on the rock they were cut from, and the message should say **"placed from the
      core depth record"**.
- [ ] Import the same file with the box **off**. Depths stay exactly as written.
- [ ] Import thin sections whose filenames or table carry the core report's depths, box ticked.
      Each plate should move to where its plug now sits.
- [ ] Check a **core photograph** with a base depth. It should move, and **keep the thickness it
      was logged with** — a 1 m photo stays 1 m, it does not stretch or flip.
- [ ] Check a **thin section** with no base. It should move and **stay a point sample** — a section
      is cut from one plug and never gains a thickness from being moved.
- [ ] Tick the box on a well with no core, for either import. It should use the depths as written
      and say **"no core to follow"**.

**Worth knowing:** SCAL rows that carry no depth at all are left alone, and the message says so
rather than pretending it placed them.

**Still not automatic:** a delivery already sitting in the project does not move when you re-register
the core afterwards. That is the last piece of your tentative "yes" on pictures following plugs, and
it waits for you to firm it up.

---

## Data already in the project follows a later re-registration (2026-07-31)

The last piece. Until now, re-registering a core moved the plugs and their extras — but the XRD you
imported last month, the SCAL points and the thin sections stayed where they were.

- [ ] Open **Tools ▸ Register Depth…** on a well that already has XRD/CEC, SCAL and pictures. Below
      the plots there is now a list of **everything that can move with the core**.
- [ ] Deliveries you imported with **"these depths came from the core report"** ticked are
      **pre-ticked** here. Anything else is listed but left unticked, marked *"not marked as
      core-depth data"* — a perforation record is on the driller's scale and must not be dragged
      along.
- [ ] Apply a shift. The status line should count **plugs, point samples, Pc points and pictures**
      separately, so you can see everything moved.
- [ ] **Ctrl+Z.** All four should come back together.
- [ ] The same list drives the **per-barrel** Apply — untick something, apply barrels, and it stays
      put.
- [ ] Untick everything and apply: only the plugs move. That is a legitimate choice, not an error.

**Note on older data:** anything imported before today is marked *not* core-depth, because the app
genuinely does not know. It is still listed and you can tick it by hand — but the default leaves it
alone rather than moving data on a guess.

**Worth a real test:** register a core, then check that a thin section you imported months ago has
moved with its plug and still lines up in the log view.

---

## The core carries its own depth history (2026-07-31)

The last piece of the depth work. Next year, "why is this core at this depth?" has an answer in the
project rather than in somebody's memory.

- [ ] Open **Tools ▸ Register Depth…** on a well whose core has never been shifted. At the bottom:
      *"This core has never been shifted. Its plugs are at the depths the laboratory delivered."*
- [ ] Propose a shift and apply it. Reopen the dialog — there is now a line giving the date, the
      delivery, the amount, what it was matched against, and the correlation.
- [ ] **Overrule a proposal**: propose, then type a different amount before applying. The recorded
      correlation should be the one at the amount you APPLIED, not the peak of the scan.
- [ ] **Ctrl+Z**, then reopen. The undo appears as its own line rather than the original vanishing —
      a core you registered, disagreed with and put back is not the same as one you never touched.
- [ ] Apply **per-barrel** shifts where one barrel was proposed and another typed by hand. Each gets
      its own line with its own interval, and the hand-typed one shows a BLANK correlation, not 0.00.
- [ ] A plain **Shift Core…** (Data ▸ Core) also lands in the history, marked *typed by hand*.

**Worth a real test:** register a core today, come back to the well next week, and check the history
tells you what you did and how well it matched — without opening a notebook.

---

## Plate scale and preparation (2026-07-31)

The groundwork for measuring anything on a thin section. Your answer was "sometimes" on both the
scale and the epoxy, so both are things you tell the app rather than things it works out.

- [ ] **Data ▸ Import ▸ Images…** now asks for a **field of view width in mm**, an **impregnation**
      choice and a **stain**. Leave them all blank — the import should work exactly as before.
- [ ] Set a field of view for the delivery, then override **one plate** in the new **FOV mm** column
      of the confirm table. That plate keeps its own value; the rest take the delivery's.
- [ ] **Data ▸ Tools ▾ ▸ Plate Details…** (this is the old Plate Depths… — same dialog, renamed
      because it now holds more than depth). Depths work as before.
- [ ] Hover a plate's **FOV mm** cell. A calibrated plate shows what that works out to in µm/px on
      the stored copy; an uncalibrated one says grain and pore size cannot be measured on it.
- [ ] **Apply to whole delivery** with "All datasets" selected should refuse — a core photograph
      must not inherit the thin sections' magnification. Pick one dataset and it applies.
- [ ] **Ctrl+Z** after that. Plates that had different values before should get their own back, not
      one shared value.
- [ ] **Clear** a plate's field of view (empty the box, Save). It should go back to having no scale.
      A wrongly typed scale you cannot remove is worse than one never entered.

**Why "unknown" is a real setting:** left unknown, the pore measurement will refuse the plate. If it
assumed "not impregnated" it would still return a porosity — built out of blue-ish grains and edge
artefact — and that number would plot against your core helium porosity looking perfectly sensible.

**Worth a real test:** import a delivery where some sections state a scale and some do not, and
check the ones without simply come in blank rather than picking up a neighbour's number.

---

## Pore area from blue epoxy (2026-07-31)

The first real number off a thin section. It needs numpy and Pillow in the app's Python — the dialog
says so up front if they are missing, and nothing else in the app is affected.

- [ ] **Petrophysics ▸ Petrography ▸ Pore Area…** on a well with thin sections.
- [ ] The **Tune on plate** list should grey out any plate you have not declared as blue epoxy, and
      say why — *not stated* or *not impregnated*. Set it in Plate Details… and it becomes selectable.
- [ ] The preview shows your plate with the counted pixels in **red**, and the percentage under it.
      That red area IS what gets measured — it is drawn by the same code, not a separate guess.
- [ ] Move **Hue from / Hue to / Saturation / Brightness** and watch the red change. Tighten
      saturation until stain and grain edges drop out. The starting numbers are a plain blue band,
      not a calibration — they are there so there is something to look at on the first click.
- [ ] **Measure every declared plate** — a table of plate, depth and pore area, plus a warning naming
      every plate left out and why.
- [ ] **Save as point data.** Lands under **PETROGRAPHY / VPORE_TS** at each plate's depth. Check it
      in the Wells pane tree, and put it in a point-data track beside your core porosity.
- [ ] Nothing should be written until you press Save — moving the sliders must leave the project alone.

**On reading the result:** where this disagrees with core helium porosity, the disagreement is
information, not a bug. Microporosity below what the section resolves, plucked grains, epoxy that did
not penetrate. Two honest measurements of different things.

**Not done on purpose:** no despeckling. Cleaning up a mask needs a brush size in pixels, and pixels
mean a different physical size on every plate — including the ones with no scale at all. The speckle
stays visible so you can judge it.

**Worth a real test:** run it on a delivery you have point-counted by hand and see how close it lands.

---

## Measuring a plate's own scale bar (2026-07-31)

For plates that print a scale bar instead of stating the field of view. This is what makes grain
size and pore size possible at all.

- [ ] **Data ▸ Tools ▾ ▸ Plate Details…**, then the **⇹** button in a plate's FOV column.
- [ ] Switch to **Actual size** and scroll to the bar. This is worth doing — one pixel out on a
      100-pixel bar is 1%, and looking closer is the only cure.
- [ ] Drag from one end of the bar to the other. A red line with end caps shows exactly where you
      landed, and the readout gives the bar as a percentage of the plate.
- [ ] Type what the bar reads (500 µm, 1 mm, whatever is printed) and the field of view appears,
      with the µm/px it works out to.
- [ ] **Use this scale** fills the FOV box in the table. Press the row's **Save** to keep it — the
      measuring does not write anything by itself.
- [ ] Tick **Apply to every plate of this delivery** before accepting if they were all shot at the
      same magnification. Each plate keeps its own impregnation and stain — only the scale changes.
- [ ] **Ctrl+Z** after that undoes the whole delivery.
- [ ] Press **Esc** in the middle of measuring. Nothing should be written or left half-done.

**Why it does not care about zoom:** the bar is measured as a *share of the picture*, not in pixels.
If the bar is a quarter of the plate's width and reads 500 µm, the plate is 2 mm across — and that
stays true whether you are looking at it fit-to-window or at full size, and whether the stored copy
was shrunk on import or not.

**Worth a real test:** measure the same bar twice, once fitted and once at actual size, and check you
get the same answer to within your own hand.

---

## Pore geometry (2026-07-31)

The shape and size of the individual pores, not just how much of the plate they cover. Needs scipy
as well as numpy and Pillow.

- [ ] **Petrophysics ▸ Petrography ▸ Pore Area…**, tune the colour band as before, then tick
      **Also measure each pore's shape and size**.
- [ ] **Measure every declared plate.** The table gains **Pores**, **Aspect** and **Roundness** —
      and, for plates that carry a scale, **D10 / D50 / D90 in µm**.
- [ ] A plate with no scale should show its shape numbers and leave the µm columns **blank** — not
      zero, not a pixel figure wearing a micron label.
- [ ] **Save as point data**: PORE_N, PORE_ASPECT, PORE_SHAPE at every plate, plus PORE_D10/D50/D90
      only where a scale existed. Check the Wells pane tree.
- [ ] Raise **Smallest pore (pixels)** and watch the pore count drop. That number is in pixels on
      purpose — it says what your picture can resolve, not how big a pore is in the rock.

**How to read the numbers.** Roundness is 4πA/P²: 1.00 is a circle, lower is more ragged. Aspect is
the long axis over the short axis of the equivalent ellipse: 1.00 is round, higher is elongated. D50
is the pore diameter that splits the pore AREA in half — area-weighted on purpose, because that is
what a capillary-pressure curve fills. A count-weighted median would be dominated by the smallest
specks your scan can see.

**Two things it deliberately throws away.** Pores touching the edge of the plate are excluded (their
real size is unknown, and keeping them would drag the distribution small), and blobs below the
minimum are dropped as speckle. Both are counted so you can see how many.

**Worth a real test:** compare D50 against the pore throat radii from your SCAL Pc curves on the same
plugs. They measure different things — bodies against throats — so they should NOT match, but they
should move together, and where they do not is worth a look.

---

## Plug QC — the petrography numbers meet an independent measurement (2026-07-31)

The check the last three items were building toward. Everything Part 2 measures is a number nothing
else in the app could test; this pairs it with the core.

- [ ] **Petrophysics ▸ Petrography ▸ Plug QC…** (also in the workspace ＋ menu, "Plug QC (core vs
      petrography)"). It opens on **CPOR against VPORE_TS** if the well has both — the section
      against the plug it was cut from, which is the question the pane exists for.
- [ ] **Compare.** Check the table: pairs, Pearson r, Spearman ρ, and the median of each axis.
- [ ] **Read the medians first.** A 0.19 beside an 18.2 means one delivery is a fraction and the
      other is percent — nothing here converts a unit, so this is where you would see it.
- [ ] Turn the **reference line** to **1:1** for this pair. Porosity against porosity is the same
      quantity twice, so the line means something and the axes go square. Points below it are
      sections reading leaner than the plug.
- [ ] **Now the harder one:** X = **SCAL — pore-throat radius**, Y = **PORE_D50**. A **Mercury
      saturation** box appears; 35% is the Winland r35 convention. Leave the reference line at
      **None**.
- [ ] Tick **Log X** and **Log Y**. Both quantities run over decades, so this is the honest picture.
      Watch **Spearman stay exactly the same** while Pearson moves — that is the point of showing
      both, and it is why Spearman is the one to quote.
- [ ] **Read the exclusions line.** It names why each sample was left out: no partner within the
      tolerance, no depth, no recorded interfacial tension, a Pc curve that never reached 35%.
- [ ] **Hover a point.** It names the well and depth, and says either "same depth" or how far apart
      the two deliveries were paired across.

**What the numbers mean.** Pearson asks whether the cloud is a straight line. Spearman asks only
whether the two move together, and does not care about the shape or the axis scale. For a pore
BODY against a pore THROAT you want Spearman: bodies are always larger than the throats that drain
them, so they must never fall on one line, but a rock with bigger bodies had better have bigger
throats. A high Spearman with a poor Pearson is the healthy answer there. On porosity against
porosity it is the other way round — there you want Pearson near 1 and the cloud on the 1:1 line.

**On the depth tolerance.** The default 0.15 is one standard 6-inch sample. If almost nothing pairs,
do NOT widen it — a core off by a whole sample interval will happily pair with its neighbour and
look fine. Register the core first (**Data ▸ Tools ▾ ▸ Register Depth…**); the pane's own note says
so when it finds nothing.

**The throat radius.** Washburn, r = 2σcosθ/Pc, using the σcosθ your laboratory recorded on that
delivery — a plug with none is excluded by name rather than converted with an assumed mercury
system. Pc is interpolated in log Pc, and a curve that stopped before 35% mercury is excluded rather
than extrapolated.

**Worth a real test:** run it on a well where you already trust the core. If VPORE_TS sits
systematically below CPOR, that is the Delesse relation meeting a real section — under-counted thin
epoxy, or a plate that is not representative of the plug. Either way the size of the gap is a number
you now have.

---

## Grain size — apparent, with Wicksell as a tick (2026-07-31)

Your D3 answer, shipped. Family B completes Part 2 of the image plan. Needs scipy, like the pore
geometry.

- [ ] **Petrophysics ▸ Petrography ▸ Pore Area…**, tune the colour band as usual, then tick
      **Also outline each grain and measure its size**. Two more fields appear —
      **Smallest grain (pixels)** and **Grain separation (pixels)** — plus the Wicksell tick.
- [ ] **Look at the preview before the table.** The grains are drawn as yellow outlines over the
      same picture. This is the one thing you cannot judge from a number: a section chopped into
      fifty slivers and one sensibly split into twelve grains produce equally plausible tables.
- [ ] **Drag Grain separation up and down** and watch the outlines. Small = more, thinner grains;
      large = fewer, fatter ones. Stop where it matches what you see down the microscope.
- [ ] **Measure.** The table gains **Grains** and **Contact**, and on plates with a scale
      **GD50 app µm** and **Sort app φ**.
- [ ] Tick **Also report Wicksell-corrected sizes** and measure again. Two more columns,
      **GD50 W µm** and **Sort W φ**.
- [ ] A plate with no scale should show **Grains** and **Contact** and leave every µm and φ column
      **blank** — phi is a logarithm of millimetres, so sorting needs a scale as much as a diameter
      does.
- [ ] **Save as point data** and check the Wells tree: GRAIN_N, GRAIN_ASPECT, GRAIN_CONTACT, then
      GRAIN_D10/D50/D90_APP and GRAIN_SORT_APP, and the _W set if you asked for it. **There is no
      plain GRAIN_D50** — that is deliberate.

**Contact is the number to read first.** It is the fraction of a grain's outline that touches
another grain rather than open pore. Where your sand is loose it will be near zero and the outlines
are real. Where it is cemented or has quartz overgrowths there is no pore between the grains for
the picture to see, and the algorithm places a boundary at the narrowest point anyway. Above 0.7
the run tells you so, and those sizes are a description of the fabric rather than a grain-size
analysis. No correction fixes that — Wicksell corrects for the sectioning, not for outlines that
were never visible.

**Sorting is Folk & Ward in phi**, the same measure your core descriptions use: under 0.35 is very
well sorted, 0.35–0.50 well sorted, 0.50–0.71 moderately well sorted, and so on. Phi runs backwards
from millimetres — bigger phi is finer rock.

**What Wicksell actually changes, and it is not what you would expect.** A random cut through a
grain rarely goes through its centre, so sections look small and badly sorted. But the size effect
is smaller than the textbooks make it sound: about 13% on the median, and because the diameters here
are area-weighted (which on a section is the same as weighing, like a sieve) most of even that is
already absorbed. The real damage is to **sorting** — a perfectly sorted rock reads about 0.19 phi
from its sections alone. So use the correction when the sorting number is what you are quoting, and
do not expect it to move D50 much.

**Worth a real test:** compare GRAIN_D50_APP against a sieve or laser grain-size analysis on the
same plugs, in **Plug QC**. Both are volume-weighted, so they should be directly comparable — and
where they are not, Contact will usually tell you why.

---

## Stained carbonate — mineral fractions from a declared stain (2026-07-31)

Family A2. Needs no scipy — it is a colour rule like the pore band, so it runs on every plate
including the uncalibrated ones.

- [ ] **Plate Details…** first: each stained plate needs its **Stain** field filled in with what
      your laboratory actually applied. A plate with the field blank is refused by name, and one
      whose stain does not match the scheme is refused too, naming both.
- [ ] **Pore Area…**, tick **Also read the stain**. Pick the scheme. Two ship: **Alizarin red S**
      (calcite stains, dolomite does not — Friedman 1959) and **Alizarin red S + potassium
      ferricyanide** (the combined stain, which also separates the ferroan phases — Dickson 1966).
- [ ] The class list below it is **editable**. The mineral identifications are published; the
      colours are not — what a stained calcite photographs as depends on your dye batch, your lamp
      and your scanner. Tune the hue ranges against the preview.
- [ ] **Measure.** The table gains one column per mineral plus **Unclassified**. Check that every
      row sums with the pore area to 100%.
- [ ] **Read Unclassified first.** It is the rock that fell in no band. If it is large the mineral
      columns are a partial answer, and the run says so above 25%.
- [ ] **Save** and check the Wells tree: MIN_CALCITE, MIN_DOLOMITE, MIN_FERROAN_CALCITE,
      MIN_FERROAN_DOLOMITE, MIN_UNCLASS.

**The one trap, and it will bite you if your sections are both impregnated and stained.** Blue-dyed
epoxy is blue. Under Dickson's combined stain, ferroan dolomite is turquoise. They are the same
colour to a hue rule, and the pore rule runs first — so ferroan dolomite gets counted as porosity.

On a test plate built as exact quarters, the default epoxy band (180–260°) returned **pore 50% and
ferroan dolomite 0%**. Porosity doubled, a mineral erased, and both numbers entirely believable.
Narrowing the epoxy band to 210–260° returned **pore 25% and ferroan dolomite 25%**, which is the
truth.

The run detects the overlap and names the affected mineral in the notes. It does **not** fix it —
which of the two bands to narrow is your judgement, looking at your plate. If you see that note,
tune before you trust either number.

**Worth a real test:** compare MIN_CALCITE + MIN_DOLOMITE against your XRD on the same plugs, in
**Plug QC**. XRD is by weight and a section is by area, but the two should track — and where they
do not, Unclassified usually explains it.

---

## Mineral classifier — trained on your own point counts (2026-07-31)

Family A3, the last one. Quartz against feldspar in plane light is not a colour problem, so this is
a classifier you train. Needs scikit-learn as well as scipy.

- [ ] **Petrophysics ▸ Petrography ▸ Mineral Classifier…**
- [ ] Type a mineral name and **Add mineral**. Repeat for each one you want to separate.
- [ ] **Click on the plate** to label what is under the pointer — the same act as point counting.
      The chip for each mineral shows its running count. **Undo last label** takes one back.
- [ ] Switch plates and keep labelling. Labels from every plate in the delivery train one model.
- [ ] **Save labels.** They persist with the project, so you can come back and add more.
- [ ] **Train and apply.** Read the per-mineral table FIRST, then the fractions.
- [ ] **Train, apply and save** writes CLS_QUARTZ, CLS_FELDSPAR … to the PETROGRAPHY dataset. Check
      the Wells tree.

**Read the recall column before anything else.** It is the fraction of held-out clicks the model got
right for that mineral, and a row below 0.70 is coloured. **A low recall means that mineral's
percentage is noise** — the model cannot see it, and no amount of confident decimal places changes
that. The overall accuracy can be 90% while one mineral is at 0.4, which is why the table is per
class.

**The check is honest by construction.** The model is scored on clicks it has never seen — and
grouped by click, so the pixels around a click can never be split between training and testing.
That is the difference between an accuracy you can trust on a new plate and one that just measures
how well the model memorised your clicks.

**What it can and cannot do.** It sees colour and local texture. Cloudy altered feldspar against
clear quartz at the same colour is exactly the case it handles — tested, and it separated them
perfectly. Two minerals that genuinely look identical in your images it will NOT separate, and it
tells you so: labelling one uniform material as two minerals gave a held-out accuracy of 41%, near
chance, with both classes flagged.

**Nothing is pre-trained, and the model does not travel.** It learned your lamp, your white balance
and your scanner along with your minerals. A differently photographed delivery needs its own labels.

**On the item names.** These are `CLS_` and the stain results are `MIN_`. That is deliberate — a
fraction from a published stain identification and one from your own classifier are different
claims, and a report has to be able to say which it quoted.

**Worth a real test:** label a section you have already point counted by hand, then compare the
fractions against your own count. That is the only calibration that means anything here.

---

## A click that needs a well now says so (2026-07-31)

ROADMAP T-IMP-05. Small, but it is the kind of thing that wastes ten minutes and looks like a bug.

- [ ] With **no well selected**, click each of: Export LAS, Import DLIS, Import SCAL, Import
      deviation, Import Aux, Import pictures, Data Sets, Shift Core, Well header.
- [ ] Each should open a small dialog naming that action and saying no well is selected, with what
      to do about it. Previously the only sign was one line in the status bar.
- [ ] **OK** closes it. Select a well and the same click should go straight through with no dialog.

Nothing changed about what the actions do — only about how they refuse.

---

## Run on a real petrography delivery — and what it changed (2026-07-31)

I ran the pore-area rule over a real carbonate thin-section delivery: 134 photomicrographs, one
laboratory, one well, one report. Three things came out of it, and one of them changed the code.

**Before you click anything, the part that has no fix yet.** Your plates arrive inside an Excel
workbook — one worksheet per plate, with the well, the depth in feet, the plug number and the
magnification typed into cells, and the pictures pasted on top. Import pictures… takes a folder of
files, so it cannot read a single one of them. I had to lift them out of the workbook by hand
before anything here could see them. That is the real first barrier, and it is the next thing worth
building.

- [ ] Confirm that is how your petrography usually arrives, or whether some laboratories send you
      loose JPEGs with the depth in the filename.

**What changed.** The app already refuses a plate nobody declared impregnated. It had nothing to
say about a plate that WAS impregnated but photographed under a different light — and that is most
of a real delivery. Across those 134 plates the overall colour ran from orange through green to
violet. On one blue-cast plate the rule returned **97% porosity**. On a green-cast plate from the
same core it returned 6%. Twenty-eight plates came back above 50% porosity. None of them failed;
all of them would have been saved at a real depth.

The new rule is: **if the picture is mostly the colour you called pore, it is not a porosity.** Rock
is mostly rock, so on a plate the band is reading correctly the typical pixel is a grain. When the
typical pixel is pore-coloured, the band is matching the background.

- [ ] Run **Pore Area…** on a delivery where the plates were not all photographed alike. Plates the
      rule cannot read should appear in the table in orange with a ⚠ on the percentage.
- [ ] Hover one. The tooltip should name that plate's own median hue and say the band is matching
      the background.
- [ ] The number is still shown — that is deliberate, it is what you tune the band against — but
      **Save** should not store those plates. Check the point data afterwards: only the readable
      plates should be there.
- [ ] The notes should say how many plates were affected, and — when the delivery was shot under
      more than one light — that its colours span too wide a range for one band, so it should be
      measured in groups.
- [ ] Tune the band on one flagged plate using the preview until the mask sits on the pores. The
      warning should clear and that plate should become storable.

On my run this took what would have been stored from a 0–97% spread down to 0–39%, median 12%. That
is a believable carbonate.

**What is still refused, correctly.** Your delivery states `5x` and `10x`, not a field of view in
micrometres — and magnification alone cannot be converted without the camera and tube factors. So
every size in microns stays blank on these plates. Some sheets carry a scale bar as a separate
little graphic beside the picture; Calibrate (⇹) can only use a bar that is inside the picture
itself.

- [ ] Check whether your plates ever have the scale bar burned into the photograph rather than
      pasted next to it. That decides whether ⇹ is usable on your deliveries at all.

**Still open, and I did not guess at it.** A delivery can mix thin sections with SEM plates in one
folder. A colour rule run over a grey SEM image returns **0.0%** — which looks like a perfectly
reasonable answer for a tight rock, and is the mirror of the 97% case. The obvious test did not
separate them on this data, so I shipped nothing rather than a threshold I could not defend.

---

## Import plates straight from a petrography workbook (2026-07-31)

This is the barrier the last run hit. Your plates live inside the workbook, one worksheet per plate,
with the depth typed in a cell — so **Import pictures…** now reads the workbook itself.

- [ ] **Data ▸ Import pictures…** with a well selected. The file dialog should now offer
      *Plates and petrography workbooks* and let you pick a `.xlsx` directly.
- [ ] Pick one of your petrography photo-sheet books. The wizard should open with every plate
      already listed and **the depth filled in from the sheet** — not guessed from a filename.
- [ ] The depth unit should already be set to what the sheets say (ft on the ones I tested), not to
      your display unit.
- [ ] Read the note block above the table. It should list what was left out and why: decorations
      dropped per sheet, sheets that state two magnifications, sheets whose header has no depth.
- [ ] Check a few rows against the workbook itself. Panel A and panel B of one sheet are two
      separate plates at the same depth — that is deliberate, only you can say which is plane light
      and which is crossed nicols.
- [ ] Import, then open the well's picture track. The plates should sit at the depths the report
      gives them.

On the two books I tested this gave **152 plates, every one with a depth**.

**The old `.xls` is refused.** You'll get a message naming the file and telling you to Save As
`.xlsx` in Excel first. That's 107 of the 165 petrography workbooks on this machine, so it will come
up. The reason: the pictures can be pulled out of an `.xls`, but working out which *worksheet* each
one came from — and the worksheet is where the depth is — needs a lot more work, and guessing it
would hang a plate off the wrong sand.

- [ ] Try selecting an `.xls` and confirm the message says what to do rather than doing nothing.
- [ ] Save one as `.xlsx` and confirm it then imports normally.

**Tell me if the Save As route is too tedious** and I'll scope reading `.xls` properly.

**Magnification is carried but not used.** Your sheets say `5x`, `10x`, `2.5x`. That can't become a
field of view without your microscope's camera and tube factors, so sizes in microns stay blank
until you enter a real scale. The wizard says so rather than staying silent about it.

- [ ] Confirm you'd want a per-delivery "camera width and tube factor" setting that turns
      magnification into a scale automatically — or whether you'd rather always measure a bar.

---

## The whole road, run on your own core (2026-07-31)

I ran the petrography suite end to end on a delivered carbonate photo book — workbook in, plates at
their stated depths, pore area measured, then checked against the petrographer's own point-counted
visible porosity for the same samples. Two things came out of it: one delivery format that was
vanishing, and an honest answer about whether the measurement works.

### Your vector plate books now import

One of the two books holds its photomicrographs as **EMF** (vector) rather than JPEG. It was
importing as **zero plates** — 53 sheets, 106 pictures, nothing, and almost no explanation. The
library reading the workbook silently discards picture formats it cannot decode. It now reads the
pictures straight out of the file instead, so they all come through.

- [ ] Import the vector book (the one whose PDF twin shows magenta-stained plates). You should get
      **106 plates** rather than an empty list.
- [ ] Check the note block. Sheets whose header states no depth are counted and named — those
      plates come in without a depth rather than borrowing one from the sheet above.
- [ ] Confirm the plates display in the picture track and print in a composite.

Both books together now give **258 plates** where you previously got 152.

### The measurement does not yet agree with the petrographer, and here is the number

On the blue-epoxy book: 35 samples paired against the point-count table. **Counted median 14%,
measured median 6.8%, rank agreement -0.09.** No colour band anywhere in the range fixes it.

The reason is that your plates are not colour-consistent. Across one core, one laboratory, one
report, the plates' own median hue spans **289 degrees** — some fields are green-cast, some
blue-cast. On a green-cast plate the rule found 0.04% where you counted 15%; on a blue-cast plate
it found 31% where you counted 9%. It is measuring the photograph, not the rock.

- [ ] Open Pore Area, tune the band on ONE plate with the preview, and look at how badly it fits
      the next plate along. That is the problem in one click.

**Where the plates ARE colour-consistent it works.** Restricted to the blue-cast group with a band
tuned to them: rank agreement **0.62** on 10 plates. So the method is sound and the instruction is
real: measure a delivery in colour groups, not in one pass.

**One warning worth having.** On the green-cast group I could tune a band until the measured median
landed on your counted median almost exactly — 15.7 against 15.0 — while the per-plate agreement
stayed at -0.10. **Tuning until the average looks right is the wrong way to tune it.** Judge the
band on the preview and on agreement, never on the mean.

- [ ] Does this match your own experience of these plates — that the lighting varied between
      sessions? If the laboratory can re-export them under one white balance, that is worth more
      than anything I can do in software.

### Still open, deliberately not guessed

A plate cast AWAY from the band returns a fraction near zero — which looks exactly like a tight
rock. I can see the signature (0.04% against your 15%) but I cannot pick the cut-off that separates
it from a genuinely tight section without inventing a number.

- [ ] Tell me whether you would rather it REFUSED a suspiciously empty measurement on a section you
      declared impregnated, or kept storing it. Refusing costs you the odd real tight plate; keeping
      it ships the odd wrong number that looks fine.

## One band, many lamps — the colour fix (2026-07-31)

You said "yes but conditional" to refusing an empty measurement, and asked for the colour problem
fixed properly. Both are here.

**The fix: name a reference plate.** Pore Area… now has a **Reference plate** picker under the
tuning plate. Leave it at *none* and nothing changes — every plate is read exactly as delivered,
which is right for a delivery shot in one session. Pick the plate your band reads correctly and
every other plate is colour-corrected onto it before the band is applied.

What that means in petrophysical terms: the app measures your reference plate's matrix colour and
puts every other plate's matrix colour in the same place, then applies your band. It corrects the
lamp, not the rock. It deliberately does **not** use the textbook grey-balance, because a
blue-epoxy section genuinely is blue — the more porous the more so — and grey-balancing would
flatten the porosity signal itself.

- [ ] Open Pore Area, tune the band on one plate, set that plate as the Reference, then use **Tune
      on plate** to preview a differently-cast plate. The band should now sit on its epoxy.
- [ ] Run it over the delivery and look at the new **Shift** column — how far each plate's light
      sat from your reference's. A plate that moved a long way is one to look at.
- [ ] Compare the result against your petrographer's point count again. Before: rank agreement
      -0.09 over 35 samples. If the correction is doing its job this should move.

**The conditional refusal.** A plate whose band claims less than one pore's worth of pixels is now
refused — but **only when you have named a reference plate.** Without one there is no evidence the
band finds epoxy anywhere in the delivery, so an empty answer might just mean you haven't tuned it
yet, and refusing would refuse your first click. Once a reference is named, that plate is your
statement that the band works, and a plate showing nothing after being corrected onto it is either
nonporous or mis-corrected — and the picture cannot say which.

- [ ] Check the refused rows read sensibly (orange, with the reason on hover). If a genuinely tight
      section of yours gets refused, tell me — that is the cost of the conservative call and I want
      to know how often you actually pay it.

**A reference plate that is itself mostly the colour you called pore is refused outright**, by
name, before anything runs. Everything is anchored to it, so a mistake there would be inherited by
the whole delivery and would look consistent everywhere.

- [ ] Try setting a badly cast plate as the reference and confirm you get a clear refusal rather
      than a delivery of plausible nonsense.

### Still open

Whether ONE reference can serve plates spanning 289 degrees at all. The correction gets less exact
the further a plate has to move. How far is too far is a judgement to read off the Shift column and
the preview — I have not invented a cut-off for it.

- [ ] After a run, tell me the largest Shift you saw on a plate whose preview still looked right.
      That is the number I would need before any automatic cut-off could exist.

## What the correction was actually worth on your rock (2026-07-31)

I ran the point-count comparison again with a reference plate, on your own delivery. Two things
came out of it, and the first is a bug I had just shipped.

**The correction was anchoring on the wrong thing.** I had it measure each plate's overall colour
and correct that onto the reference. But a plate with more blue epoxy in the field of view HAS a
bluer overall colour — so the correction was partly cancelling the porosity itself. It is now
anchored on the matrix only: the pixels the band did not claim.

The difference on your plates, against the petrographer's point count over 45 plugs:

| | rank agreement |
|---|---|
| no correction | 0.19 |
| corrected, anchored on the whole plate (this morning's version) | 0.05 |
| corrected, anchored on the matrix (now) | 0.20 |

- [ ] Nothing to click for this one — but if you ran Pore Area with a reference plate before
      reading this, re-run it. The stored numbers came from the wrong anchor.

**The honest verdict: it stops the measurement being wrong, it does not make it right.** Rank
agreement with your petrographer is around 0.2, and sweeping 57 different bands the best I could
reach was 0.25 without the correction and 0.36 with it — and that best-of number is fitted on the
same data it is scored on, so it is a ceiling, not an accuracy.

**But the measurement itself is repeatable.** Your delivery photographed two separate fields of
view of every plug. The two agree with each other at **0.85**, while agreeing with the point count
at 0.10–0.27. So the pictures are fine and the measurement is stable — the disagreement with the
point count is systematic, not noise.

- [ ] Does that match your expectation? A point count ticks visible pores under a grid; the colour
      rule counts every blue pixel including microporous haze the counter would not tick. If you
      think that is the whole gap, say so and I will stop trying to close it with colour.

**One number that says the colour cast is not just a lamp.** Two photographs of the SAME plug,
taken minutes apart, differ in overall colour by up to 66 degrees of hue. That is far more than a
white balance can explain, and it is why a single reference plate cannot rescue the whole delivery.

- [ ] If you can ask the laboratory anything about these plates, ask whether the camera was on
      auto white balance. That would explain it exactly, and it is a setting, not a re-shoot.

---

## A seventh of a delivery was landing on the wrong rock (2026-07-31)

Reading your petrography books to set up the helium comparison, 18 of one book's 129 plate sheets
came out at depths of 33 to 71 feet — on a well cored at 4,600 and 7,000 feet.

They are the sheets that write the depth the Indonesian way:

```
a sheet writing a decimal point   :  6980.71 FT/ 301     read as 6980.71 ft    correct
a sheet writing a decimal comma   :  7016,54 FT / 337    read as 54 ft         wrong
```

The reader looks for a number with a unit after it. A comma decimal splits the number in two, so
`7016` was thrown away for carrying no unit and `54 FT` matched instead. Nothing failed. The plate
was simply stored at 54 feet, which is a perfectly plausible shallow depth, on rock 7,000 feet away.

It now reads both conventions, and the 103 sheets that use a decimal point are untouched.

- [ ] Re-import any petrography workbook you have already brought in, and check the depth column in
      the wizard before you commit it. If a plate is sitting in the wrong sand, this is why.

**One sheet in 129 still reads wrong and I have left it that way on purpose.** It writes
`7033,50/354 FT (CORE)` — the unit sits on the plug number instead of the depth, so it reads 354 ft.
Every rule that would fix that case breaks a commoner one (a cell reading `PLATE 12, DEPTH 4633.50
FT` would then read 12). The wizard's editable depth table is the defence, and a 354 sitting among
7,000s is easy to spot there.

---

## Your point count is not the yardstick I was treating it as (2026-07-31)

I have been judging the colour rule against your petrographer's visible-porosity count and getting
poor agreement. So I checked the count itself against the laboratory's helium porosity, on the same
45 plugs. Neither of those measurements is mine.

**They agree with each other at rank 0.505** — and the point count reads a median 14.5% against
helium's 24.8%.

That gap is the expected one: a point count ticks pores visible under an optical grid, while helium
fills every connected pore including micropores far below what a microscope can resolve. In a
carbonate that is a large difference. So 0.505 is roughly the ceiling for this rock, and
"disagrees with the point count" was never on its own evidence that the colour rule is wrong.

**Against helium, the colour rule reaches 0.575 with no correction and 0.67–0.69 with it** — better
than the point count manages. But that headline is inflated and I want to be straight about it.

- [ ] Nothing to click. This one is a number to know.

### What survives when you look inside a single core

Your delivery spans two cored intervals with very different rock — the shallow one runs about 25%
helium porosity, the deep one about 5%. A correlation computed across both looks strong largely
because it tells a porous carbonate from a tight one, which you already know before you start.

Scored **inside** each interval, against helium:

| | shallow core | deep core |
|---|---|---|
| colour rule, no correction | 0.01 | 0.27 |
| colour rule, corrected | 0.19 | 0.49 |
| your petrographer's count | 0.51 | not counted |

So the honest reading is:

- **The colour correction earns its place.** It lifts agreement inside both cores, measured against
  an independent laboratory rather than against the point count. In the deep core it roughly
  doubles, 0.27 to 0.49.
- **It does not yet beat your petrographer where both exist.** In the shallow core the count reaches
  0.51 and the colour rule 0.19.
- **The cross-delivery 0.69 is mostly two different rocks.** Do not quote it.

- [ ] The deep core has no point count at all, and there the colour rule reaches 0.49 against helium.
      If that interval matters to you, this is the case where the tool is doing work nobody has done
      by hand. Worth a look at those plates.

### Which plate you pick as the reference matters more than the colour band

This is the part I did not expect, and it is the most useful thing to come out of the run.

Sweeping three reference plates drawn from each core's own plates, scored inside that core against
helium:

| reference plate | shallow core | deep core |
|---|---|---|
| a pale one | 0.11 | 0.30 |
| a middling one | **0.24** | **0.53** |
| a strongly cast one | 0.20 | 0.15 |
| no correction at all | 0.01 | 0.27 |

In the deep core that is a **three-and-a-half-fold spread** from one choice — and the worst pick
(0.15) is worse than not correcting at all (0.27). So the reference plate is a bigger lever than the
band you spend time tuning, and right now the dialog gives you nothing to pick it with except the
preview.

- [ ] When you next run Pore Area with a reference, try two or three different reference plates and
      see how much the numbers move. If they move a lot, that is the tool telling you the delivery
      needs splitting into groups, not that the band is wrong.

Giving each core its own reference did beat using one for the whole delivery — 0.24 against 0.19 in
the shallow core, 0.53 against 0.49 in the deep one. So **measure them in groups** is real advice.
It is a refinement, though, not the missing piece.

## You can now see which reference plate is the right one (2026-07-31)

Last session's finding was that picking the reference plate moved the answer more than tuning the
colour band did — 0.11 to 0.53 in your deep core, with the worst pick worse than not correcting at
all — and that nothing in the dialog let you see that. It does now.

**Check against** sits directly under Reference plate in Pore Area…, and defaults to your core
porosity where the well has it. After each run you get a line like:

> Agreement with CPOR — core plugs: rank 0.49 over 11 plug(s).

and, once you have tried more than one setting, a small table of everything tried this session with
the best in bold:

| Setting tried | Plugs | Rank agreement |
|---|---|---|
| none · band 180–260° | 11 | 0.27 |
| a strongly cast plate · band 180–260° | 11 | 0.15 |
| **a mid-tone plate · band 180–260°** | 11 | **0.53** |

Those three numbers are your deep core, from last session's sweep — no correction 0.27, a bad
reference 0.15 which is worse than doing nothing, and a good one 0.53. Before this they all looked
the same on screen. (The setting column names whichever plate you actually picked; the descriptions
here are just so the row reads.)

Three things to know about reading it.

**Use the rank column, not the straight-line one.** A thin section always reads below the plug's
helium porosity because helium finds micropores the microscope cannot see. That offset is real and
it is not an error, and only the rank figure ignores it and asks the question you actually care
about: does the tool put the plugs in the same order the laboratory does.

**Watch the Plugs column.** If it changes when you change a setting, the two runs were scored on
different rock — a different set of plates got refused — and the rows are not a fair comparison. The
non-comparable rows go orange and are never bolded, so a number that only rose because the awkward
plugs dropped out cannot be mistaken for an improvement.

**Nothing is written by this.** It is measured on exactly the plates that a Save would store, but it
runs whether you save or not, so you can tune freely.

- [ ] Open Pore Area… on a well that has both plates and core porosity, and check that Check
      against comes up already set to CPOR.
- [ ] Run once with no reference plate, then two or three times with different references, and see
      whether the table spreads the way your deep core did.
- [ ] If a reference scores below the no-reference run, that reference is making things worse —
      worth knowing before it goes into a report.
- [ ] On a well with no core, check the run still works and simply says there is nothing to check
      against.

## Each cored interval can have its own reference plate (2026-07-31)

Last session gave the reference plate a dial. This session gives it a second one, because the same
numbers said one reference was not always enough: giving each of your cored intervals its own plate
scored better than a single delivery-wide one in **both** of them (0.19 → 0.24 shallow, 0.49 → 0.53
deep). That was measured before this existed, by running each interval as a separate job. Now it is
one run — and Check against will tell you whether it helped on the well in front of you rather than
you having to take my word for the two above.

**Per-interval references** sits directly under Reference plate in Pore Area…. Press *+ Interval
with its own reference*, type a depth range and pick the plate that range should be corrected onto.
Leave either end blank for "from the top of the well" or "down to total depth" — that is how a cored
interval at either end is actually described, and a blank is not a missing number.

The plate table gains a **Reference** column whenever more than one plate served, beside the Shift
column. A shift of 40° means nothing until you know which plate it is 40° from.

**Intervals may touch but not cross.** `2000–2010` next to `2010–2020` is fine and the shared depth
goes to whichever you listed first. A real overlap is refused before anything is measured, because
inside it the reference a section got would come down to the order of the list — you would get two
different answers from the same settings with nothing on screen saying why.

**A section no interval reaches falls back to the Reference plate above.** If you have not set one,
that section is refused **by name** rather than measured uncorrected. That is deliberate: the
empty-answer guard only works on a corrected plate, so an uncorrected one sitting in the same saved
delivery as corrected ones would have quietly lost it, and nothing downstream could tell them apart.

**Fractions from two different intervals are only as comparable as their two reference plates are.**
The run says so, and lists which plate served which range. Compare intervals on the agreement figure
rather than by reading their medians against each other.

- [ ] Set up two intervals matching your two cored sections, each with a mid-tone plate of its own,
      and check the Reference column shows each section corrected onto its own interval's plate.
- [ ] Compare that run against a single delivery-wide reference in the settings table, and see
      whether the split earns its place on your rock the way it did on mine.
- [ ] Leave a gap between the intervals with no Reference plate set, and check the sections in the
      gap are named in Left out rather than quietly measured.
- [ ] Type two overlapping intervals and check the run refuses with both ranges named, before it
      measures anything.
- [ ] Check that touching intervals (`…–2500` and `2500–…`) are accepted, and that a plate sitting
      exactly on 2500 goes to the one listed first.

## Core slab photographs: clean them up, then read a trace off them (2026-07-31)

Two new things under Data ▸ Tools ▾ ▸ **Condition Core Photos…**, and they are meant to be used in
that order.

### Cleaning the photograph

Everything is done by looking at the picture rather than typing numbers at it:

- The delivery is a **strip of thumbnails** across the top. Click one to work on it. A green dot
  marks the ones already conditioned **in the project** — not the one you are fiddling with now.
- **Crop** by dragging a rectangle on the photograph. Drag again inside it to crop further.
- **Pick a grey**, then click the colour card, the grey tray or a white label. Everything shifts so
  that patch reads neutral, and the swatch beside the button shows what you clicked.
- **Straighten, Brightness, Contrast, Colour, Warmth, Green/magenta** are sliders whose track shows
  the gradient they move along — you can see which way amber is before you touch it.
- **Hold to compare** shows the photograph as delivered for as long as you hold the button.
- The **histogram** under the picture is the exposure check: a wall at either end is detail that is
  now pure black or pure white and cannot come back.

**Nothing is destroyed.** The photograph as imported is kept the first time you apply, everything
afterwards is re-rendered from it, and **Reset this photo** puts it back byte for byte — including
its shape, so a cropped picture goes back to its full frame rather than being stretched into the
cropped one's box.

**Apply this light to the whole run** copies the colour half only. Every picture keeps its own
straightening and crop, because the box sits differently on the bench in every frame.

### Reading a trace off it

Below the sliders, **Read the trace** averages the pixels down the core and draws three tracks:

- **DARK** — how dark the rock is. Follows shale in most clastic sections.
- **RED** — redness, which picks up oxidation, red beds and some oil staining.
- **TEX** — how much the colour varies across the core, which is a lamination and heterogeneity
  measure.

Tell it **which way depth runs** (across the picture or down it), whether the box was photographed
**deepest-end-first**, and **how many rows of core** are in the frame. Then **Save as curves** writes
them as `CPHOTO_DARK`, `CPHOTO_RED` and `CPHOTO_TEX`.

**They are not called VSH, and they never will be.** Darkness follows shale without being a shale
volume — the same dark band is organic mudstone in one core and oil stain in another. Anything named
VSH is read by every module in the app as a shale volume, so an uncalibrated one under that name
would be a wrong answer that computes and plots. Calibrate it against your own GR first.

**Check against does exactly that**, and it defaults to GR. It reports a SIGNED number: darkness and
gamma should both rise into shale, so a strongly NEGATIVE darkness is a finding rather than a weak
result — usually the depth axis is the other way round. The run says so when it sees it.

Two things worth knowing. **Crop to exactly the core before reading a trace** — the picture's top
and base depths are taken to span the frame end to end, so a tray or a tape left in the crop is read
as rock. And **equal rows are an approximation**: a real box has unequal rows and gaps, so for a
careful job crop to one row and run it per row.

- [ ] Open it on a well with core photographs and check the thumbnail strip fills in as you scroll,
      and that clicking one loads that picture.
- [ ] Pick a grey on a colour card or a grey tray and see whether the cast comes out the way you
      would have corrected it by hand.
- [ ] Crop a box down to just the core, apply, and check the picture in a log-view image track and
      in a printed composite — they should both show the cropped, corrected version.
- [ ] Press Reset this photo and check it comes back exactly as delivered, at its full size.
- [ ] Apply this light to the whole run on a run shot in one session, and check each box keeps its
      own crop.
- [ ] Read a trace off a box with GR as the check. Does DARK agree with your gamma? If it comes back
      strongly negative, try Deepest end first and see whether it flips.
- [ ] On a four-row core box, compare 4 rows in one go against cropping to one row at a time — is
      the equal-lane approximation good enough on your boxes?
- [ ] Save the curves and put CPHOTO_DARK in a log track beside GR.

## Square up a box shot from an angle, and three detail corrections (2026-08-01)

Two more things in Data ▸ Tools ▾ ▸ **Condition Core Photos…**

### Square up

For a box photographed from one end rather than straight above. Press **Square up**, drag the four
handles onto the corners of the core itself, then press **Done**. The picture is stretched back to
the shape the box really is.

This is not the same as Straighten and Straighten cannot do it. A box shot from an angle is a
trapezoid — the far end is drawn shorter than the near end — so a depth read straight down the
frame runs fast at one end and slow at the other, and every sample in between lands at a depth that
is wrong by an amount which changes along the core. Rotating a trapezoid just gives you a tilted
trapezoid.

The squared-up picture is a different SHAPE from the one that arrived, and that is correct: the
delivered shape was already wrong, and the new proportions are measured from the corners you
placed. (This is deliberately the opposite of what happens to a thin section, whose delivered shape
is the truth and is never stretched.)

While you are dragging, the picture is shown as the camera framed it — otherwise you would be
pointing at corners in a photograph that had already been squared up to them.

### Detail: Local contrast, Denoise, Sharpen

Three new sliders, grouped apart from the colour ones because they do something different.

- **Local contrast** lifts the shadowed end of a box towards the lit end, tile by tile, instead of
  brightening the whole picture. This is the one for a box lit from one side.
- **Denoise** takes out speckle and dust without softening the grain boundary next to it.
- **Sharpen** lifts real edges — bedding, grain boundaries, fractures.

**They change what Read the trace measures, and there is a warning under them that turns orange
when one is active.** Local contrast roughly HALVES the darkness contrast between clean sand and
mudstone, so an equalised box and a plain one no longer read on the same scale — the trace still
follows the rock, but a calibration against GR fitted on one will not hold on the other. Sharpening
inflates TEX and denoising suppresses it. Read the trace off photographs corrected for light and
framing only; use these three to make a picture readable.

Read the trace also names any photograph carrying one of the three, so a run cannot quietly mix
equalised and plain boxes.

- [ ] Take a box photographed from one end, Square up, and check the core comes out as a rectangle
      with the ends the same width.
- [ ] Read the trace off that box before and after squaring up — does DARK agree with GR better
      once the depth axis is linear?
- [ ] Try Square up on a box that was already shot square: the handles start at the frame corners,
      so leaving them there should change nothing.
- [ ] Apply this light to the whole run on a photo you have squared up, and check the other boxes
      did NOT get its corners.
- [ ] Push Local contrast up on a box lit from one side. Does the shadowed end become readable?
- [ ] Then Read the trace on that box and check the run names it in the notes.
- [ ] Denoise a grainy photograph and check the grain boundaries are still sharp — if they soften,
      say so, that would mean the filter is wrong.
- [ ] Check a preview against the applied result: a denoise or sharpen judged on screen should look
      the same once saved, not weaker.

## The core photograph beside the logs (2026-08-01)

**Condition Core Photos… ▸ Build depth strips**, then open a log view and choose the new **Core**
layout.

Build depth strips takes each box, cuts it into its rows of core, turns each row so it runs
downwards, and stacks them into one tall picture covering that box's own depth interval. It uses the
same **Depth runs / Rows of core / Deepest end first** settings as Read the trace, sitting right
above it — get those right once and both the picture and the curve are right.

The strips land in a new picture dataset called **CORE STRIP**. Building again replaces the last
one, so you can try 4 rows, look at it, try 2, and not end up with a pile of half-built deliveries.

The built-in **Core** layout shows GR, the strip, CPHOTO_DARK and the neutron-density crossover
side by side. You can also add the strip to any layout of your own: add an Image track, set its
dataset to CORE STRIP, mode to Depth and Fit to **Fill the track**.

Two things worth knowing. **Crop to exactly the core first** — the strip is stretched over the
box's depth interval end to end, so a tray or a tape left in the crop is drawn as rock. And
**gaps stay gaps**: each box keeps its own interval, so a break between two core runs shows as a
break rather than being closed up.

- [ ] Build strips off a real core-photograph delivery and open the Core layout. Does the core run
      the right way down the page?
- [ ] Check a box you know — is row 2 below row 1, and does each row start where the last one
      finished?
- [ ] Scroll to a break between two core runs. Is the gap still there?
- [ ] Print a composite with the Core layout. Does the printed strip match what the screen showed?
- [ ] Try Deepest end first on a box that was photographed the other way up, rebuild, and check the
      strip flips the whole box rather than just each row.
- [ ] Rebuild with a different row count and confirm the old strips are replaced, not added to.
- [ ] Put the strip beside GR and see whether the dark bands line up with the gamma peaks. If they
      are shifted by a constant, that is a core depth shift — Data ▸ Tools ▾ ▸ Register Depth…

## Register the core off its own photograph (2026-08-01)

**Data ▸ Tools ▾ ▸ Register Depth…** now offers the core photograph's trace in the reference list,
as **Core photo — CPHOTO_DARK**, alongside the plug columns and the point datasets.

This is usually the best reference you have. Core plugs give you a few dozen samples a foot apart;
the photograph gives a reading every few millimetres down the whole cored interval, which is what a
cross-correlation actually wants. Read the trace and Save as curves first, then open Register
Depth… and pick it.

**If the result comes back negative, do not accept it.** Darkness should rise with gamma, because
clay is both dark and radioactive. A negative best match nearly always means the boxes are laid out
the other way up rather than that the core is shifted — go back to Condition Core Photos, tick
Deepest end first, save the trace again, and re-run. The run says this in the notes rather than
quietly proposing a shift, because a correlogram cannot tell an upside-down box from a real depth
error.

### Fixed at the same time

**Saved trace curves were unreadable.** They were being stored at the photograph's own sampling
rather than on the well's depth frame, and the app joins computed curves to that frame by an exact
depth match — so CPHOTO_DARK was saved, was reported as saved, and then came back empty to every
module, plot and log track. If you saved trace curves before today and could not find them
anywhere, that was why. Save them again and they will be there.

They are now written on the well's own depth frame, each sample being the average of the photograph
samples inside it rather than one of them picked out. Depths outside the cored interval stay blank.

- [ ] Read a trace, Save as curves, then check CPHOTO_DARK really appears — in a log track, in the
      crossplot curve list, and in Register Depth…
- [ ] Register a core against CPHOTO_DARK and compare the shift with what you get from the plug
      porosity. Does the photograph give a sharper peak on the correlogram?
- [ ] Deliberately set the wrong lay-out (untick Deepest end first on a reversed box), save, and
      run Register Depth… — does it refuse in words rather than propose a shift?
- [ ] Apply a shift found from the photograph and check the core plugs, the extras and the plates
      all move with it.

## The UV frame beside the white-light one (2026-08-01)

In **Condition Core Photos…**, next to Hold to compare, there is now a delivery picker and a **Hold
for the pair** button. Pick your UV delivery there (it opens on one automatically if its name says
UV) and hold the button to see the same depth under ultraviolet.

It matches on depth, not on filename, so it works whatever the two deliveries are called. Each frame
is shown with its own conditioning — a UV frame under a white-light photograph's white balance would
be a picture of the correction rather than of the fluorescence.

**Build depth strips now shows the dataset it writes to**, pre-filled from the source name: build
off CORE PHOTO and it suggests CORE STRIP, build off CORE PHOTO UV and it suggests CORE STRIP UV. So
you can build both and put them in two tracks side by side in a log view — white light beside
fluorescence, both at true depth.

- [ ] Open a well with both deliveries. Does the pair picker land on the UV one by itself?
- [ ] Hold for the pair on a box you know has a show. Is the fluorescence where you expect it?
- [ ] Check a box where the two deliveries were framed differently — does it still find the right
      UV frame?
- [ ] Build strips off both deliveries into two dataset names, then put both in one log view beside
      GR. Do they line up with each other and with the log?
- [ ] Condition the UV delivery separately (it usually wants a different exposure) and check the
      white-light one was not touched.

## Thin sections get the core photographs' workspace (2026-08-01)

**Petrophysics ▸ Petrography ▸ Condition Plates…** is the same workspace the core photographs use —
thumbnail strip, drag to crop, click a grey to fix the cast, sliders whose tracks show the colour
they move through, hold to compare, histogram. It opens on a thin-section delivery instead of a core
one, and the trace and depth-strip section is hidden, because a section is cut from one plug and
covers no interval.

**Condition first, then measure.** Pore Area reads the conditioned plate, so a cast fixed here is a
cast the measurement never has to fight.

### Pore Area: the band is a colour now

The four number boxes are gone. In their place:

- A **hue wheel** with two draggable ends. The part of the wheel your band accepts is the bright
  part; everything else is dimmed. Drag an end past the other and the band wraps through red, which
  is allowed.
- **At least this vivid** and **at least this bright** are sliders showing the colours they move
  through, at your band's own hue.
- A **swatch** of roughly what a pixel has to look like to be counted.
- The numbers are still there beside them, and still typable.

**Pick the pore colour** — press it, then click a pore on the plate below. The band re-centres on
that colour keeping the width you set, and the floors drop just enough that the pixel you clicked is
inside it. It reads the plate WITHOUT the red mask on it, so clicking inside the mask still samples
the rock.

**Hold to compare** shows the plate without the mask, so you can see what the band claimed against
what is actually there.

- [ ] Open Condition Plates… on a thin-section delivery. Does the filmstrip fill in, and does
      cropping and white-balancing work the way it does on core photos?
- [ ] Fix a plate's colour cast there, then run Pore Area on it. Is the band easier to set?
- [ ] Use Pick the pore colour on a clear blue pore. Does the band land somewhere sensible?
- [ ] Now click a grain by mistake — does the band move somewhere obviously wrong, so you can see
      it did what you asked?
- [ ] Drag the wheel's ends and watch the preview. Is it easier to judge than typing degrees?
- [ ] Try a band wrapped through red (drag the high end below the low one) and check the preview
      agrees with the wheel.
- [ ] Hold to compare on a plate you are unsure about. Does the mask sit on the pores?

## Pick a plate by looking at it (2026-08-01)

Both **Pore Area…** and **Mineral Classifier…** now show the delivery as a strip of thumbnails
above the picture, the same way Condition Plates… does. Click a tile to work on that plate; the
dropdown is still there underneath if you prefer it.

In **Pore Area**, a plate that cannot be measured is greyed out with the reason when you hover it —
"not impregnated", or "preparation not stated". It is still clickable, so you can preview what the
band would claim on it before deciding whether to declare it in Plate Details.

In the **Mineral Classifier**, each tile shows how many clicks you have already placed on that
plate. That is the thing a filename list cannot tell you: which plates you have counted and which
you have not.

- [ ] Open Pore Area on a mixed delivery. Are the undeclared plates obviously greyed?
- [ ] Hover one — does it say why?
- [ ] Click a greyed one and press Preview. You should still see what the band would claim, even
      though it will not be stored.
- [ ] In the Mineral Classifier, click through a few plates and place labels. Do the counts on the
      tiles keep up?
- [ ] Close the classifier and reopen it. Do the counts come back?
- [ ] On a large delivery (a hundred plates or more), scroll the strip — does it stay responsive,
      loading thumbnails as you go?

## Four silent successes, from the triage (2026-08-01)

From `docs/review_triage.md` — findings 13, 17, 19 and 20. All four were places where something
reported success having done nothing, or half of something.

**An equation script that throws on some depths now says so.** Run a Rhai equation that raises on
part of the interval — the run still succeeds and the curve is still written, but the summary line
now ends with a warning naming how many samples of how many threw. A curve that is simply holed
because its inputs were missing does NOT warn, which is the point: the warning only fires where the
script had real numbers and could not answer.

- [ ] Write an equation that throws on part of a well (e.g. `if gr > 60.0 { throw "high" } gr/100`)
      and run it. Does the status line say how many samples raised?
- [ ] Run one that cannot throw over an interval with washed-out GR. Does it stay quiet?
- [ ] Does the Processing panel show that well as a warning rather than a plain tick?

**A workflow chain that dies no longer locks the project.** If a chain's worker stops
unexpectedly, it now reports a failure instead of sitting at Running forever — and Open Project,
New Project and Compact Project work again. Previously the only way out was restarting the app.

- [ ] Run a chain to completion, then Open Project. (Control — this always worked.)
- [ ] Cancel one mid-run, then Open Project.
- [ ] If a chain ever does die on you, check the Workflow Builder says it stopped and that the
      results are incomplete, rather than sitting at Running.

**Curve Edit refuses an empty box instead of writing 0.** Right-click a curve in a log view →
Edit. With "Set constant" chosen, clearing the Value box used to write **0.0** over the interval —
a perfectly normal-looking reading of very clean rock. It now refuses, in the dialog, naming the
field.

- [ ] Edit → Set constant, clear Value, press Apply. Do you get a warning in the dialog and
      nothing written?
- [ ] Type `abc` in Value. Same?
- [ ] Clear Top as well. Does the message name both fields?
- [ ] Choose Blank (erase) and clear Value. It should go through — Blank does not use Value.
- [ ] Type a real number. Does it still apply normally?

**Editing a well that has been deleted now fails.** With the Wells grid open, delete that well in
the Wells & Tops pane, then edit one of its cells. It used to report success and push an undo
entry for a change that never happened.

- [ ] Do exactly that. Do you get "that well is no longer in the project… refresh the Wells grid"?
- [ ] Does the ordinary case — editing a well that IS there — still work?

## Three things wrong with the report PDF, from the triage (2026-08-01)

Findings 12, 15 and 18. All three were invisible to whoever exported the report.

**A batch export no longer loses a well.** If two wells share a name — or two different names come
out the same once the filename is cleaned up — the second used to overwrite the first, and the app
still said it had written both. Now the second gets `_2` on the end and you get one file per well.
The first one keeps the plain name.

- [ ] Batch-export a report over a set of wells that includes two with the same name. Do you get
      two files, one of them ending `_2_report.pdf`?
- [ ] Does the count in the status line match the number of files in the folder?
- [ ] If a well fails to render, does the error name the WELL now rather than a long id?

**The cover states the interval the well is logged over.** It used to state whatever depth window
the composite was printed at — so setting a 5 m window put "Interval: 1005.0 – 1010.0 m" on the
cover of a report whose pay table still covered every zone in the well. If you set a print window,
it is now stated separately underneath, as "Log pages printed over…".

- [ ] Render a report with no depth window. Does the cover's interval match the well's logged
      range?
- [ ] Set a depth window and render again. Does the cover keep the full interval and add the
      printed-over line?
- [ ] Tick Tables only with a window set. The printed-over line should NOT appear — there are no
      log pages for it to describe.

**Every page carries the Made in SandiBumi mark.** The cover had it and the composite pages had
it, but the methodology, zone-parameter and pay-summary pages did not — so a pay summary
photocopied on its own was unattributed.

- [ ] Render a report and look at the bottom of the methodology, zone-parameter and pay-summary
      pages.
- [ ] If the pay summary runs to more than one page, check the second page too.
- [ ] Is it small and pale enough not to compete with the table? Say if you would rather it were
      cover-only — it is one line to take back out.

## Cutoffs and empty runs, from the triage (2026-08-01)

Findings 8, 7 and 10.

**A permeability cutoff now survives a chain that models permeability.** In Monte Carlo, adding a
`perm_coates` step to a chain used to switch the PERM cutoff off silently — so the study most
likely to want that cutoff was the one that never got it. The numbers looked like a cutoff that
had been applied and simply not bitten.

- [ ] Run a Monte Carlo chain that READS permeability from the project, with a PERM cutoff set
      high enough to bite. Does the pay drop?
- [ ] Insert a permeability model into the same chain and run it again with the same cutoff. Does
      it bite the same way now?
- [ ] Lower the cutoff below the modelled permeability. Does the pay come back?

**A well with no permeability still escapes the PERM cutoff — but it now says so.** Whether an
uncored well should be excluded from a permeability cutoff or exempted from it is your call and
nothing about it has changed. What has changed is that the report and the dashboard now tell you
which wells escaped, instead of adding their full pay in silently beside wells the cutoff was
applied to.

- [ ] Run a pay summary with a PERM cutoff over a mix of wells, some with a PERM curve and some
      without. In the Field Dashboard, is there an orange line naming the wells with no
      permeability?
- [ ] Export a report for one of those wells with a PERM cutoff set. Is there a note under the pay
      table saying the cutoff was not applied and its net pay is not comparable?
- [ ] Export a report with no PERM cutoff at all. The note should NOT appear.
- [ ] **Tell me which way you want the rule itself to go** — should an uncored well be excluded
      from a permeability cutoff, or exempted from it? The flag makes either honest; only you can
      say which is right.

**A failed run no longer fills the Curve Catalog with blank curves.** Running rocktyping on a well
with no permeability used to report the failure AND write all eight of its output curves into the
catalog, blank from top to bottom. The catalog could then no longer tell "this was never run" from
"this ran and could not answer".

- [ ] Run rocktyping on a well with porosity but no permeability. Does it report the failure?
- [ ] Check the Curve Catalog — are RQI/FZI/RT and the rest absent, rather than present and empty?
- [ ] Give that well a permeability curve and run it again. Do all eight curves appear with real
      values?
- [ ] Check the Processing history still records the failed attempt — that is where the record of
      "a run happened" belongs now.

## The VSH dropdown now says which Larionov is which (2026-08-01)

Finding 21, and the bookkeeping of 11 and 14.

**The two Larionov options are the ones that matter.** They differ by a digit in their name and by
more than half again in their answer at mid-range gamma — 0.330 against 0.216 at IGR 0.5 — which is
right where the VSH cutoff decides net pay. The dropdown used to show bare ids, and the manual test
plan (the only other place it was written down) had the rock ages the wrong way round.

- [ ] Open **Petrophysics ▸ VSH ▾ ▸ VSH from Gamma Ray**. Does the method dropdown read
      `LARINOV1 — Larionov, Mesozoic and older` and `LARINOV2 — Larionov, Tertiary / unconsolidated`?
- [ ] Mahakam is Miocene, so `LARINOV2` should be the one you reach for. Does the label make that
      obvious without looking anything up?
- [ ] Run with one, then re-open the pane. Does it come back on the same choice?
- [ ] Hover a saved run in the Curve Catalog — the tooltip lists params. Is the stored value still
      the bare id (`LARINOV2`), so an old run still reads correctly?
- [ ] Every other module's option dropdowns should be unchanged (bare ids). Check one, e.g. Porosity
      ▸ Density-Neutron.
- [ ] `LARINOV3` deliberately claims no rock age — it shows its coefficient instead, because nothing
      in the repo cites a source for that form. Say if you know one and I will label it properly.

**Two plan corrections, no code:**

- [ ] T-PETRO-02 step 1 now names the right rock age, and its Expected line pairs 0.33 with
      Larionov-older rather than Larionov-Tertiary.
- [ ] T-ADV-13's "Mark **Fail — known**" instruction is struck. Saturation-Height on a deviated well
      really does measure from the survey now, so following the old paragraph would have logged a
      working feature as broken.

**One item left your sign-off list:** legacy-multimin RECON_ERR. It never needed a decision — the
module is retired and refuses to start, and SandiMin already detects the exactly-determined case and
returns a note saying RECON is forced to zero. The one thing worth your eye:

- [ ] Run SandiMin with only RHOB + NPHI (plus unity) so it is exactly determined. Is the `dof` note
      hard to miss in the pane, or does it sit somewhere you would scroll past? A warning nobody
      reads is the same as no warning.

---

## Condition — curve conditioning as modules (2026-08-05)

Five new modules in a new **Condition** ribbon group: Despike, Smooth, Clip, Fill Gaps, Flip
Polarity. Built as modules rather than as an editor, so they are multi-well, zone-overridable,
chainable, mask-aware and log-set-versioned from the first run, with the dialog auto-generated.
Full reasoning in `docs/plan_data_tools.md`.

- [ ] **Petrophysics ribbon ▸ Condition ▾** — five entries, and the group sits before VSH. Does the
      icon read as a spiky trace being flattened, or as noise?
- [ ] Open **Despike**. WINDOW opens EMPTY with the placeholder "set a value" — your call, since
      what counts as a spike rather than a thin bed has no value that is right in two basins. Press
      Run without filling it: does it refuse in the pane, naming WINDOW, with the cursor landing in
      that field?
- [ ] The method dropdown offers all four rules you asked for. Do the labels say enough to pick one
      without reading a manual?
- [ ] Run HAMPEL on a GR with a known spike. **Set WINDOW narrower than the thinnest bed you mean to
      keep** — that is the whole discriminator. Did the spike go and the beds stay?
- [ ] Try a WINDOW of about two samples. It should REFUSE, not run: over three samples the spread
      the test measures against is set by the spike itself. Does the refusal point you at ABS?
- [ ] Every module writes `<input>_C` unless you type your own name in **OUT**. Type `GR_ED` and
      check that is what lands in the Curve Catalog.
- [ ] Type `GR` in OUT. It should be REFUSED. This one is worth reading: a curve stored under a
      standard mnemonic is written, counted and reported, and then nothing reads it back — the raw
      log wins. Is the message clear about why?
- [ ] `<OUT>_SPK` is written beside each despiked curve, 1 where a sample was replaced. Put it on a
      log view beside the curve. Is that the right way to see what the filter took, or would you
      rather have a count in the run report?
- [ ] **Smooth** — MEAN, MEDIAN and SAVGOL. Over an interval with real curvature, does SAVGOL keep
      the peak where a MEAN flattens it?
- [ ] Smooth a curve that has a hole in it. The hole must still be there afterwards: smoothing does
      not fill gaps, on purpose. Confirm nothing appeared across it.
- [ ] **Clip** defaults to BLANK rather than CLAMP — a reading outside the range is usually a
      reading the tool could not make, and clamping leaves a real number where there is no
      measurement. Is BLANK the right default for your work, or do you reach for CLAMP more?
- [ ] Leave Clip's MAX empty and set only MIN. The upper side should genuinely not be a bound.
- [ ] **Fill Gaps** — set MAX_GAP, run, then plot `<OUT>_FILL`. Every invented sample is marked and
      nothing else is. Is the flag curve worth the extra entry in the catalog, or would you rather
      it were optional-off?
- [ ] Fill Gaps must never fill a hole at the very top or bottom of a curve — that is extrapolating
      past where the tool logged. Check one.
- [ ] **Flip Polarity** on an SP. `<OUT>_PIV` carries the pivot actually used. Flipping the result
      again with the same pivot should give the original back exactly.
- [ ] Put a Condition step into a **Workflow** chain and set a per-step OUT name in the grid. Does
      the text cell behave like the other override cells (bold when overridden, Set-all working)?
- [ ] Anything here you would rather have as a separate editor pane with a before/after preview —
      that is the next increment, and this is the moment to redirect it.

---

## Every tool now names its log set — and the word is "log set" (2026-08-05)

You said you had forgotten what a set refers to. The reason is that the UI never said it: the
store, the backend and the docs all say **log set**, while the two dialogs that offered one
called it a "constellation", abbreviated to "cons". One word now, everywhere.

Before this, exactly two surfaces of nineteen let you choose a version. ML, SandiMin, the
saturation-height fit, the cutoff sweep, the pay summary, the facies tie, Lorenz, results QC and
every deliverable read whatever the current values happened to be — and ML, SandiMin and the core
photo trace hardcoded where their output went (`ML`, `SANDIMIN`, nowhere).

- [ ] **Petrophysics ▸ any module** — the two rows now read "Input log set" and "Output log set".
      Does that connect with what the Curve Catalog shows, in a way "cons" did not?
- [ ] **Curve Catalog** — the section heading now reads "Log sets", and the search box says so too.
- [ ] Run a module into a log set called `TEST`, then run another module with **Input log set =
      TEST**. Does the second read the first's values rather than the current ones?
- [ ] **ML Models…** now has both rows. Train a model with Input log set = FINAL, then re-run your
      porosity, then apply the saved model. It should still be reading FINAL — the whole reason
      for saving a model is that it can be reapplied to the same rock.
- [ ] ML output used to land in a set called `ML` with no way to change it. Type your own name.
- [ ] **SandiMin** — same two rows; output used to be forced to `SANDIMIN`.
- [ ] **Cutoffs & Summary**, **Report…**, **Workbook…**, **Deck…** — each has an Input log set.
      This is the one that matters for a client deliverable: a report that cannot name the version
      of the interpretation it quotes cannot be reproduced. Render the same report against two sets
      and check the numbers actually differ.
- [ ] **Photo Log…** now writes into a log set (default `CPHOTO`). Before, the trace curves had no
      version at all, so each re-read silently replaced the last and nothing recorded which
      conditioning produced them. Check the Curve Catalog shows a version after a Save.
- [ ] **Calibrate S…**, **Plug QC…**, **Pore Area…** and the petrography family read point data and
      pictures rather than curves, so they have no log set — say if you expected one there.
- [ ] Anywhere the two rows appear in a place that reads awkwardly, say so — they are one shared
      control, so moving them is one change rather than nineteen.

---

## Frame — blocking, and the permeability trap (2026-08-05)

Two modules in a new **Frame** ribbon group: **Block (Upscale)** and **Bed Detect**.

Resample and Regularize are deliberately NOT here. A module's outputs are written at the run's own
depth frame, so changing how often a well is sampled cannot be a module — it would have to write a
different depth column, which belongs to the well rather than to one curve. That comes with Intake.

- [ ] **Petrophysics ribbon ▸ Frame ▾** — two entries. The group sits after Condition.
- [ ] **Block** on PHIE with OPT_BEDS = INTERVAL and a 1 m interval. Every sample of a block should
      carry that block's one value.
- [ ] Put the blocked curve in a log track and **set its draw style to Step** (right-click the curve
      → edit). Without that the view draws a diagonal between two block values, which is a gradient
      the data never measured. Should Block set that automatically? It cannot today — the module
      writes a curve, the layout owns the style — say if you want them linked.
- [ ] **The one to check carefully: OPT_STAT on a permeability.** Block your PERM over a laminated
      interval three times — MEAN, GEOMETRIC, HARMONIC. On a real sand-shale they should differ by
      orders of magnitude, not percent, and MEAN should be the highest every time. An arithmetic
      upscale hands a simulator a permeability the rock does not have and nothing downstream reads
      as wrong. Is the dropdown wording enough to make somebody stop and think?
- [ ] MEAN is the default because it is right for porosity and for every volume fraction. Say if
      you would rather it had no default and refused until chosen, like the despike window.
- [ ] **OPT_BEDS = CLASS** pointing at FACIES — each run of a constant class is one bed, so the
      boundaries are where the rock changes. Check a facies boundary lines up with a block edge.
- [ ] **OPT_BEDS = ZONES** — one value per marker interval, which is what a zone-parameter table
      wants. Needs tops on the well; it refuses by name if there are none.
- [ ] **OPT_BEDS = AUTO** — boundaries found from the curve itself. Run **Bed Detect** first and
      put its output in a track as class blocks, so you can SEE the beds before anything is
      averaged over them. Over-segmentation is what a step-finder gets wrong, and a blocked curve
      computed from beds nobody checked looks perfectly reasonable.
- [ ] MIN_BED has no default, same as the despike window. Does the refusal say enough?
- [ ] `<OUT>_BED` rides beside the blocked curve, carrying the bed number. Useful, or clutter?

---

## Statistics — five tables (2026-08-05)

**Petrophysics ▸ Batch ▸ Statistics…**, or the ＋ menu. One pane, five tabs, sharing a well scope,
an input log set and a per-zone toggle. Every one is a pure read — nothing here writes a curve.

- [ ] **Curve Summary** — one row per well × zone × curve. Pick several curves (Ctrl-click) and
      set your own percentiles. Is the **Missing** column worth its width? It is there because a
      mean over 12 samples of a 400-sample zone is not the zone's mean, and nothing else in the
      row would say so.
- [ ] Find a well that never entered a zone. Every statistic on that row should be **empty, not
      zero** — Excel's AVERAGE and COUNT skip a blank and treat a zero as data.
- [ ] **Pair Summary** — two curves against each other. Both Pearson and Spearman are reported
      because they answer different questions: Pearson only makes sense when both axes are the
      same quantity. Try PHIE against core porosity, then PHIE against PERM, and see which
      coefficient you actually trust in each case.
- [ ] **Fit** — least squares on as many predictors as you like, with log10 on either side. The
      one to read is **R² (blind well)**: it refits leaving each well out in turn and scores on
      the well it never saw. Compare it with the plain R² on a permeability transform — if the
      gap is large the fit is memorising your wells.
- [ ] Fit refuses two predictors that carry the same information rather than returning coefficients
      nobody can interpret. Try PHIE and PHIT together.
- [ ] With fewer than three wells the blind figure says "needs 3+ wells" rather than showing a
      number. Is that clear enough, or should it refuse the run?
- [ ] **Versus** — the same curves in two log sets. This is the first thing in the app that uses
      log-set provenance for anything. Run a module, then compare its set against the previous one.
      The **Only reference / Only this** columns are the ones to watch: a re-run that gained or
      lost an interval matters more than one that shifted values slightly, and a mean difference
      over the common depths says nothing about it.
- [ ] **Thickness** — this is the one you asked to be its own tool. Four ways to count:
      - FLAG on `FLAG_PAY` — check it agrees with the Cutoffs & Summary net. It should, because it
        reads the same flag curve rather than re-applying the cutoffs.
      - CLASS on FACIES — thickness per facies without writing a flag for each.
      - CUTOFF — type a condition. Does one condition cover your work, or do you need several
        ANDed? The backend already takes a list; the pane offers one.
      - MARKER — gross between tops, for an isopach.
- [ ] **Gross TVD / Net TVD are blank on a well with no survey**, and that is deliberate: a
      vertical well and an unsurveyed deviated one look identical in the data. On a deviated well
      check the TVD columns really are smaller than the MD ones — measured thickness overstates
      true vertical by about 30% at 40 degrees, which is a reserves error that reads as a good well.
- [ ] **Copy as CSV** under each table. Should these also go into the Workbook export as their own
      sheets? The table definition is already in the shape `office.rs` renders.

---

## Intake — one importer, and the grid is the control (2026-08-05)

**Data ribbon ▸ Intake…**, or the ＋ menu. Replaces the table-shaped importers: core tables, point
data, and any delimited text with mixed column types. LAS and DLIS keep their own path — they are
self-describing formats with headers and units built in, not tables.

It is a FRONT END, not a second write path. `import_core_table` already owns well routing, the
foot-to-metre conversion, the percent rule, per-well replace, depth dedup and carrying unclaimed
columns into point data. Intake builds the mapping and calls it, so the two can never disagree.

- [ ] **Data ▸ Intake…** — choose a real delivery. Does the grid read the file the way you would?
- [ ] Each column header carries its **proposed role** and, on hover, the reason. Overrule one and
      watch the tint follow. Are the nine roles the right nine, or is something missing?
- [ ] The **Delimiter**, **Skip lines** and **Decimal** controls re-read the grid live. Try a file
      with a title block above the headers and skip past it.
- [ ] **The decimal control matters.** A delivery that writes `7016,54` alongside `6980.71` is not
      hypothetical — one of your petrography workbooks did exactly that, and reading only the dot
      convention put a seventh of it at 54 feet. Left on "(decide per value)" the rightmost
      separator wins; a genuinely ambiguous `1,234` is read as 1.234 and reported.
- [ ] **Cells outlined in red** sit in a numeric column and did not parse — a stray unit, a
      spreadsheet's `#N/A`, the wrong decimal convention. An EMPTY cell is not flagged, because a
      blank is a missing measurement. Is the outline visible enough on your themes?
- [ ] A column no measurement role claims becomes **Point item** and is carried into `aux_data` at
      the plug depths — lithology text, Kv/Kh, oil shows. Only **Ignore** drops one. Check they
      land: Wells pane ▸ well ▸ Point data.
- [ ] **Paste from clipboard** — copy a block out of Excel and paste. It takes the identical parse
      and commit path as a file, so anything true of one is true of the other.
- [ ] Choose several files at once. The mapping is confirmed ONCE, on the first, and applied by
      header name to all of them — a delivery split across files is one delivery with one shape.
      Say if you have deliveries where that is not true.
- [ ] **Delivery set** — the files chosen together are one delivery, auto-suffixed per well so an
      import never overwrites.
- [ ] Without a DEPTH role the Import button stays disabled and the pane says why, rather than
      failing after you press it.

**Not yet built, and named rather than quietly missing:**

- [x] **Wide and Block array layouts** — shipped, see below.
- [x] **Curve role** — shipped, see below.
- [x] **Templates** — shipped, see below.
- [ ] Import Aux… is gone. Core, SCAL and Tops imports are still in the Data ribbon; say the word
      and they go too, once Intake has earned it on your own deliveries.

---

## Intake: arrays, logs and saved mappings (2026-08-05)

**Layout** is a new field, and it is a DECLARATION — a wide table and a long one are both
rectangles of numbers and nothing in the characters says which is which.

- [ ] **Wide**: a porous-plate Pc table, one row per plug, a column per pressure step. Mark the
      DEPTH column, name the array (`PC_SW`), import. Every other column header is read as its own
      axis value, so `0.5, 1, 2, 4, 8` become the pressures.
- [ ] Put a `TOTAL` column on the end. It should be DROPPED and NAMED in the notes — counted as a
      bin it would be a saturation at an invented pressure, right where a Thomeer fit is most
      sensitive.
- [ ] Try a header written `100 psi`. The unit should be stripped and the number read.
- [ ] **Block**: several tables stacked with the header repeated. Import with Block ticked and the
      repeats should be stripped. Import the SAME file WITHOUT it and you get one extra row whose
      saturations are 1, 2, 4 — the header read as a measurement. That is what the flag prevents.
- [ ] A block whose depth is on a label line above each table rather than in a column is NOT read,
      and says so. If your deliveries look like that, tell me and it becomes the next increment.
- [ ] Import the same array twice under one delivery name. The second must land as `NAME_1`, not
      replace the first — check both are still there.
- [ ] **Curve role**: a CSV of continuous logs (GR every 15 cm). Mark the column `Log curve` and
      it goes to the curve store where modules can read it, not to point data. Check it appears in
      the Wells pane under its delivery set and is selectable as a module input.
- [ ] One file carrying BOTH logs and a lithology description should import as one delivery.
- [ ] **Saved mapping**: set your roles, name it, Save. Read next quarter's file and Apply. It
      matches by column NAME, so a delivery that gained a column does not shift every role one to
      the right — and the new column is listed rather than silently ignored.

---

## Naming every output curve, at the run (2026-08-05)

Every module pane gains an **Output curves** card: one box per curve the run will write, with the
name it will be written under, plus a **prefix all** box for a trial run. This replaces the
"Output prefix" field of the earlier increment and the "Output curve name" field the Condition and
Frame families carried — one control, forty modules.

- [ ] Open **Despike**. The two boxes should read `GR_C` and `GR_C_SPK` before you touch anything.
      Change the input curve to RHOB and both should follow.
- [ ] Type `GR_ED` into the first box. The flag box should follow to `GR_ED_SPK` rather than
      stranding at `GR_C_SPK`.
- [ ] Type `GR` into it. It should REFUSE, in the pane, naming the curve — GR is read from the raw
      log first, so a conditioned copy stored under that name is never the one anything reads.
- [ ] Clear the box. It should go back to `GR_C`, not to an unnamed curve.
- [ ] Put `TEST_` in the prefix box and run. Every curve should land prefixed, the real ones
      untouched.
- [ ] A renamed or prefixed step in a **Monte Carlo** study should REFUSE by name — the study
      resolves its cutoffs from the declared output names.
- [ ] The same boxes are in the workflow builder's per-step editor (the ⚙ expander).

---

## Reframe — a set with its own sampling (2026-08-05)

**Data ▸ Sampling ▸ Reframe…** This is the answer to the 0.1523 m well in a 0.5 m field.

Worth knowing before you try it: **every curve read in this app is an exact depth match onto the
well's standard grid**. A 0.1524 m delivery attached to a well whose grid came from a 0.5 m LAS
therefore contributes almost nothing today — no error, no warning, just a curve that reads mostly
MISSING. That is what this fixes, in both directions.

- [ ] Press **Check sampling** first. It should tell you what each well is ALREADY sampled at —
      a number nothing else in the app shows. A well already at the target is marked, because
      re-framing it would resample every curve for nothing.
- [ ] Re-frame a fine well onto 0.5 into a set of your own naming. The ORIGINAL must be untouched:
      open the well's log view and confirm it still draws at its own sampling.
- [ ] Point a module's **Input log set** at the new set and run it. The whole run should happen at
      0.5 — including the standard curves, resampled onto it, so nothing pairs a 0.5 m PHIE with a
      0.1524 m GR.
- [ ] Check a laminated interval: a downsample should AVERAGE the interval, not pick one sample out
      of it. A facies or flag curve should come back as one of its own classes.
- [ ] Re-frame a permeability with **Geometric** and again with **Arithmetic** and compare. They
      should differ by orders of magnitude on laminated rock, and arithmetic will always read
      highest.
- [ ] Re-framing to the SAME set name should give you a new VERSION of it, not a second set.

---

## Normalize — one tool, any curve (2026-08-05)

**Petrophysics ▸ Curve Conditioning ▸ Normalize.** `GR Normalization` is gone from the pickers —
it is now a preset of this and delegates to the same code, so saved chains still run.

- [ ] Normalize a GR with the same reference pair you used before. The answer must be identical to
      what `gr_normalize` gave.
- [ ] Normalize an NPHI, a DT, an RHOB. Same tool, same fields.
- [ ] Run it with **Space = LOG** on a resistivity. Compare against LINEAR: on a curve spanning
      decades the linear map crushes the middle into the bottom of the range.
- [ ] Leave `REF_LOW`/`REF_HIGH` blank. It must refuse, naming them — a reference pair from one
      basin is the wrong pair in another and the output looks plausible either way.
- [ ] **MEAN_SD** should run unconfigured: mean 0, spread 1 is a definition, not somebody's field
      calibration.

---

## Intake replaces Import Aux (2026-08-05)

**Import Aux… is gone.** Intake is the route for point data. Core and SCAL imports stay.

- [ ] Import an XRD or CEC table through Intake with a dataset name of your own. It should land as
      point data.
- [ ] Now check the well's **core** afterwards — the plugs and the φ-k cloud must be exactly as
      they were. Before this increment, importing a lab table through Intake wrote an empty core
      delivery and made it ACTIVE, which would have silently emptied Plug QC, Register Depth and
      the S-factor fit.
- [ ] Tick **"These depths came from the core report"** on a well whose core you have registered.
      The samples should land on the corrected depths, and the result should SAY it followed the
      core. (This was read from the form and dropped by the backend until now.)
- [ ] Anything the old Aux dialog did that Intake cannot — say so and it goes back on the list.

---

## Statistics: three means (2026-08-05)

- [ ] Curve Summary now shows **Geom** and **Harm** beside **Mean**. On a permeability they should
      differ by orders of magnitude; on a porosity they should be close.
- [ ] A curve with a zero or negative sample should leave both blank rather than computing them
      over the positive samples only.

---

## The sand/shale curve off the white-light trace (2026-08-05)

The first of the two you parked from the UV round. **Photo Log ▸ Sand / shale ▸ Write CPHOTO_LITH.**

- [ ] Read a white-light box with the box ticked and the cut left blank. It should propose a cut
      from this core's own darkness (Otsu) and say so, with how many samples landed in the darker
      class.
- [ ] Look at it beside GR in the built-in **Core** layout. It is a blocks curve — 0 lighter,
      1 darker — so a correlation panel can consume it.
- [ ] Type your own cut and re-read. The note should say "as given" rather than "Otsu".
- [ ] Switch Light to **Ultraviolet**. The row should disappear: under UV the brightness IS the
      fluorescence, so cutting it in two would name an oil show a rock type.
- [ ] Read a box of one lithology. It should REFUSE rather than invent a contact through the
      middle of it.
- [ ] It is called `CPHOTO_LITH` and never `VSH` or `LITH` — the same dark band is mudstone in one
      core and oil stain in another, and a curve under a name every module reads as lithology
      would be an uncalibrated answer that computes and plots.
- [ ] Nothing smooths it. If it flickers sample to sample, run **Frame ▸ Block** with
      OPT_STAT = MODE — the one upscale that carries a class code whole. Tell me if you would
      rather it had its own minimum bed thickness.

---

## The unfold for dipping beds (2026-08-05)

The second of the two. **Photo Log ▸ Unfold dipping beds**, stated as a depth DROP across the core
rather than an angle — an angle needs the core's diameter, which nothing here stores, while the
drop is read straight off the picture: note one contact's depth at each edge and subtract.

- [ ] Find a box with an obviously dipping contact. Read it flat first and look at where
      `CPHOTO_DARK` crosses — it should ramp across roughly the drop.
- [ ] Enter the drop and read again. The ramp should collapse to a step. Measured on a synthetic
      1 m dip it went to under a third of its width while both the sand and the mudstone read the
      same either side — the correction changes where the boundary is, never what is beside it.
- [ ] Sign: POSITIVE means the bedding sits deeper at the RIGHT edge. If the contact gets worse,
      try the other sign before anything else.
- [ ] The corner triangles at each barrel's ends have no rock in them and come back MISSING —
      never filled from the edge row, never wrapped from the other end of the barrel.
- [ ] It is DECLARED by default — and **Propose…** beside the field now scans a range of dips and
      shows how sharply the core reads at each. See the next section.

---

## The three open items, closed (2026-08-05)

Your "solve em" on the completion report. Each had been left as a stated limit; each is now done.

### A block keyed by a label line — Intake ▸ Layout: Block

A per-plug delivery that writes `PLUG 12  4633.5 ft` above each table instead of carrying the depth
in a column now imports. The depth is **the number carrying a UNIT**, which is the rule a plate
workbook's header cell is already read by — on a caption that also names a plug, nothing else tells
the two apart.

- [ ] Import a block-shaped Pc or NMR export whose blocks are captioned. Check each block's rows
      land on the caption's depth, and that the captions themselves are not stored as samples.
- [ ] A caption with no unit is refused BY NAME and the run says why. Confirm you get the reason
      rather than a plausible wrong depth.
- [ ] The control worth seeing once: read the same file WITHOUT Block and every row imports with
      no depth at all — which looks like a clean read of plugs that never had depths. That silent
      version is what this replaces.
- [ ] If a caption carries an interval (`2103.4 m to 2104.1 m`) the FIRST depth is used and the run
      names it as a duplicate — see the next section, which is your answer to this.

### A plug sits at one depth — Intake, import result (2026-08-05)

Your call: *"it should be 1 plug number only, should warn user if duplicate"*. So a caption naming
two depths is a duplicate rather than an interval to choose an end of, and — the part that turned
out to matter more — **two samples at one depth cannot both be stored**. An array holds one vector
per depth, so the second is refused by the database with a message naming nothing you typed. The
run now names the depth instead.

- [ ] Import a block file whose captions repeat a depth. The result names that depth and the line
      goes red, rather than reporting a sample count that the failed write contradicts.
- [ ] A caption carrying two rows under it gets the same warning — that is two plugs at one depth,
      arrived at from inside one caption instead of across two.
- [ ] A DEPTH column with a repeated depth is caught by the same rule, with no Block layout needed.
- [ ] **The control, and the one worth a minute:** a clean delivery says nothing at all, and a
      multi-well file where two WELLS share a depth also says nothing. A warning that fires on good
      files is one you would rightly start ignoring.
### The Wide/Block preview — Intake, under the grid (2026-08-05)

Your "give it preview so user can see the examples". Pick a file, set Layout to Wide or Block, and
a second table appears under the grid showing **what reading it as an array produced** — the depth
each sample landed on, and what the header row became as an axis. The grid above shows the file's
own text; this shows what was made of it.

- [ ] On a captioned Block file, check the depths in the preview match the captions. This is the
      only place on screen that says a caption was understood at all.
- [ ] The axis header shows the parsed number beside the text it came from — `100 psi` reading as
      100. Worth one look on a delivery that writes units into the header row.
- [ ] Duplicated rows are tinted red and the duplicate note is red. Hover a tinted row for why.
- [ ] It says "Showing 40 of 4,000" on a big file — and a duplicate at row 900 is **still shown**,
      carrying its real row number. Worth testing on a big delivery: a preview that stopped at 40
      would be useless on exactly the file that needs it.
- [ ] **A bug this uncovered, worth confirming:** a Block file keyed only by captions has no DEPTH
      column, and the Import button used to stay disabled — so that whole feature could not be
      reached from the pane. It now enables once the preview finds depths. If you ever had a
      captioned file "do nothing", this was why.

### The array write cannot half-finish (2026-08-05)

Found while building the check above, and the reason it matters is that it was invisible. Replacing
an array curve is delete-then-write, and it was not one transaction — so a write that failed part
way left the old curve deleted and the new one incomplete. On a Monte Carlo matrix that is not a
crash, it is a realization set quietly missing depths, with every percentile read off it computed
from a smaller population than the study ran.

- [ ] Import a Wide/Block file with a repeated depth. It is now refused **by name** — the message
      says which depth — instead of the old database error naming an internal table.
- [ ] Then check the curve that was already stored under that name is still whole. That is the
      half that used to be lost.
- [ ] Re-run a Monte Carlo with realizations persisted, over a well that already has them. It
      should replace cleanly, as before — this change must not alter that.

### A minimum bed thickness — Photo Log ▸ Sand / shale, third box

- [ ] Read `CPHOTO_LITH` with the box blank first: every one-sample flicker is kept, and the run
      says so. That is still the default, because no thickness is right in two cores.
- [ ] Enter your thinnest meaningful bed. Beds below it are absorbed into the rock around them and
      the count is reported. Check the beds you would actually log survive.
- [ ] A thin stretch with unphotographed core on BOTH sides is left alone and counted separately —
      it is a short barrel, not a flicker, and there is no neighbouring rock to absorb it into.

### Propose a dip — Photo Log ▸ Unfold dipping beds ▸ Propose…

- [ ] Press it on a box with a dipping contact. You get the whole scan drawn, not just a number:
      one sharp peak means the dip is determined.
- [ ] **A flat scan is the answer that matters.** If the core has no bedding contrast, every
      candidate scores alike, the run says FLAT, and the peak is noise. Leave the unfold at zero.
- [ ] Hatched slots are candidates that sheared away too much core to be compared — not poor
      scores. Without that floor, sliding the core off its own frame would win the scan.
- [ ] **Use N** only fills the box. Nothing is applied until you read the trace, exactly like
      accepting a depth-registration shift.
- [ ] Measured on a synthetic 1 m dip it proposes 1.0, and on the same picture with a horizontal
      contact it proposes 0.0.

### A machine with no WebGPU is told so by name — log view

- [ ] Hard to click through unless a field laptop actually lacks WebGPU, so recorded for the
      day one does: instead of a dim one-line "viewer disabled", the log view now shows a card
      that names what failed (WebGPU), says the rest of the application still works — plots,
      dialogs, imports, exports are all 2D and unaffected — and states the fix (update the GPU
      driver and the WebView2 Runtime, then reopen the view). The status line gets the message
      too, as the record.
- [ ] Verified in the browser by removing WebGPU from the page and building a real log view
      panel: the card appears with the underlying reason on its last line ("WebGPU is not
      supported in this environment"), styled from the theme's own tokens so every skin
      renders it correctly.

### One home for the rules — AGENTS.md, .cursorrules, CLAUDE.md

Nothing to click through here; these are judgement calls to accept or redirect.

- [ ] **AGENTS.md and .cursorrules are pointers now, not copies.** AGENTS.md had fallen 383
      lines behind CLAUDE.md — it was missing the whole log-set and Intake contracts, so
      Codex was working from the 2026-08-01 rules with nothing in either file saying so.
      .cursorrules had never carried rules 6 to 11 at all, which include "the frontend never
      sends SQL for writes" and "Python runs as a subprocess". Checked first that nothing
      would be lost: only two lines existed in AGENTS.md and not in CLAUDE.md, and both were
      older wordings of lines CLAUDE.md has since updated.
- [ ] **Rule 5 no longer argues with the rest of the file.** It said "no extensive unit test
      blocks unless explicitly requested" while the same file cites "Pinned by <test>" 42
      times and the repo holds 775 of them. It now says what you actually do: no ceremonial
      coverage, but one named test per contract, pinned from both sides where a lazier
      implementation would otherwise pass, and #[ignore] for anything needing an optional
      package. Read it and tell me if that is your intent.
- [ ] **The subagent ladder now lives in one place.** The machine-level rules and this repo
      both carried a tier table and they had drifted — the global one has four tiers with a
      distinct opus tier and requires announcing each delegation, this one had three tiers and
      no announcement rule. CLAUDE.md now points at the global ladder and keeps only what is
      specific to this repo: that the cost driver is the vcvars verify loop, which files carry
      silent-wrongness risk, and what is never delegated.
- [ ] A stale cross-reference went with it: "physics defaults per collaboration rule 5" had
      pointed at the PR rule ever since that rule was inserted. It reads rule 6 now.
- [ ] CLAUDE.md is still 282 KB and still loads in full every session. That is the one audit
      finding not addressed here, because where to draw the line needs your call.

### CLAUDE.md carries the contract; the reasoning moved to docs/

The last audit finding, the one left for your call: CLAUDE.md was 283 KB and every byte of it
loaded on every session, most of it dated build records for work that had already shipped and been
accepted. It is 95 KB now — a third of the size — and nothing was deleted.

- [ ] Open `CLAUDE.md`. Everything that governs how the code is written is still there, in the
      same words: the eleven implementation rules, the write discipline, the store contracts, the
      Organic design system, the dockview traps, the dev commands, the collaboration protocol.
- [ ] The new **The build record** section sits after Open-path hardening. Each line in it is a
      contract a session must not break — the ones that are wrong *silently* if broken. Read a
      few and check I kept the right half: e.g. "a fit is the algebraic inverse of the module's
      own equation", "`core_data.depth_orig` is the record and nothing ever shifts it", "an
      apparent answer and a corrected one get different item names".
- [ ] Six new files in `docs/` hold the reasoning that moved: `record_calibration.md`,
      `record_core_depth.md`, `record_petrography.md`, `record_core_imaging.md`,
      `record_data_tools.md`, `record_fixes.md`. Open one and check a section you remember —
      the text is byte-identical to what CLAUDE.md carried, moved by script rather than retyped,
      and verified line by line against the original.
- [ ] The section headed **Current state (2026-07-20)** is now **Shipped capability, and the
      conventions it set**. It had been describing Phase 9 as "STARTED" for work the same file
      later recorded as finished, and a section labelled current that is not is its own hazard.
      Its lead paragraph now says plainly to trust the build record and `ROADMAP.md` over the
      phase labels.
- [ ] Nothing that stayed moved. `## Refusing a click that needs a well`, the Organic design
      system and the launch screen are standing rules rather than dated records, so they stayed
      put — and the launch screen now sits next to the design system, where it belongs.
- [ ] One thing to watch, and the reason to click through rather than take my word: the split was
      done by line range. If a future session finds a contract in a `record_*.md` that CLAUDE.md
      should have kept, that is the failure mode — tell me and it comes back up.

### Reframe: regularize, and putting several wells on one frame

Two capabilities the roadmap had listed as shipped since the day they were planned, and which
did not exist. They are in the **Reframe** pane, not the Frame module group — a module returns a
vector aligned to whatever frame it was handed, so it cannot change the sampling at all, which is
why `block` upscales by replacing values at the well's own depths.

- [ ] **Regularize.** Open Reframe, pick a source, choose *Make the existing sampling uniform* and
      leave the step box empty. It takes the source's own median spacing. The point is that a
      log delivered at a wobbling 0.1524 stays at 0.1524 — reading the number off the probe and
      typing it back is only a chance to get it wrong. Put a number in the box and that wins.
- [ ] **Put every well on one frame.** Tick *Align* with several wells selected. Every well comes
      out on the same top, base and step, not merely the same step. This was a real defect and not
      just a missing feature: a plain step re-frame anchored each well on its own first depth, so
      ten wells re-framed at 0.5 landed on 1500.00, 1500.50 … and 1498.25, 1498.75 …. Every read
      in this app is an exact depth match, so nothing downstream could line those wells up — the
      exact failure Reframe exists to fix, reappearing one level up. Check a crossplot or a
      multi-well overlay across the aligned set.
- [ ] **Depths a well has no data for come back MISSING**, and the run says so in its notes. The
      shared frame spans every selected well, so a shallow well acquires empty rows at the deep
      end. That is the same rule `match_well` already followed — a borrowed frame is taken whole.
- [ ] **Regularize plus Align without a step is refused**, by name, rather than run. Each well has
      its own spacing, so adopting one would quietly make that well the standard for the field.
      State the step and it proceeds.
- [ ] `match_well` and `match_set` ignore the Align tick, because they are already aligned by
      construction — the frame is borrowed whole from one place.

### ML: the blind-well score was not blind

The leaderboard's headline number — the blind-well score, the one honest figure this product offers
where three vendors offer none — was optimistic by construction. Two separate leaks, both fixed.

- [ ] **The scaler had seen the held-out well.** `StandardScaler` was fitted over the whole pooled
      matrix and the folds were cut afterwards, so on a three-well run the well being held out
      contributed roughly a third of the centring it was supposed to be blind to. There is now one
      scaler per fold, fitted on that fold's training rows and nothing else. Re-run a leaderboard
      you have run before: **the scores will move, and downward is the correct direction.**
- [ ] **Feature importance was measured on the training data.** A second model was fitted over
      everything and permuted against everything — no split at all — and its result was printed in
      the same table row as the blind score. Importance is now measured on each fold's held-out
      rows by that fold's own model.
- [ ] **Importance bars carry a whisker now**, the spread between wells. This is the one to look at
      on real data: a feature can have the second-highest mean and a whisker reaching back to zero,
      which means it carried in one well and nowhere else. That is not a predictor, and the old bare
      bar said it was. Features whose mean sits inside their own spread are dimmed.
- [ ] **The leaderboard no longer crowns a winner it cannot separate.** Where the top rows are
      within their combined fold-to-fold spread, they are marked as a group and a line says the run
      does not separate them, instead of bolding whichever sorted first. Check this against a run
      where two algorithms are genuinely close — it should refuse to pick.
- [ ] The tie test is a plain "gap wider than the summed spreads", not a t-test. Folds are wells and
      there are rarely more than a handful, so a test needing assumptions the data cannot support
      would be a second false precision on top of the first. Tell me if you want it stricter.
- [ ] `docs/PRD_v2/24_ml-advanced.md` had recorded that this importance "is correctly cross-validated
      at the group level". It never was. The document is corrected in place and says what it used to
      claim, so the correction is auditable rather than quietly overwritten.

### ML: the leaderboard ranked models nobody was going to fit

`SB-MLA-026`. The hyperparameters were written out twice — once in the training runner, once in the
leaderboard — and the two had drifted. The leaderboard's whole purpose is to be trusted for a
choice, so a ranking of different models is not a degraded ranking; it is a ranking of the wrong
things, presented cleanly.

- [ ] **Set `degree` to 3 on Linear and press Compare.** It used to be ranked as a straight line,
      because the leaderboard's copy had no polynomial branch at all. Now the row scores the cubic
      you would actually fit. This is the divergence with the largest consequence — it changed which
      row won.
- [ ] Two more that were quieter: the gradient-boosting fallback ran at 100 iterations in the
      leaderboard against the run's 300, and `SVC` was built without `probability=True`, which makes
      scikit-learn fit internal Platt scaling — a different estimator, not a different output.
- [ ] **The leaderboard now takes your settings at all.** It previously accepted no parameter map,
      so every candidate was ranked at library defaults however you had configured the run.
- [ ] **New "Settings" column: `yours` on one row, `defaults` on the rest.** Your settings belong to
      the algorithm the dialog is showing, so that row uses them and the others are scored at the
      defaults the run would fit for them. Applying one algorithm's `C` to every row would re-rank
      estimators against a number nobody chose for them — but a mixed table that does not say so is
      the thing this rule exists to prevent. Check the column names the algorithm you had selected.
- [ ] The fix is one shared definition both runners are composed from, not two copies kept in step.
      Syncing copies would have fixed these three and left the mechanism that produced them. The
      test names every estimator and asserts each is in the shared fragment and in **neither**
      runner body, so a runner that embeds it and then shadows it still fails.

## ML — an unclusterable well now fails instead of quietly writing an empty curve (2026-08-07)

`SB-MLA-013`. A well that cannot be labelled — no input curve carries a reading, or fewer complete
samples than clusters requested — used to return the pre-allocated all-NaN vector as a **success**.
On a log view an all-missing track is indistinguishable from one that was never computed, so the
failure was not merely silent; it was disguised as an absence of work. Both engines had it.

- [ ] **Run electrofacies over a well with a washed-out interval where nothing overlaps** (or set K
      higher than the number of complete samples). It now refuses that well by name, naming the
      cause. Previously the run reported success and drew an empty FACIES track.
- [ ] **The message says WHICH emptiness it is** — "no depth carries every input curve at once" vs
      "the run mask excluded all N depths". They call for opposite fixes: go and find the missing
      curve, or widen the mask. The old wording, "no complete samples in this well", said both.
- [ ] **Run ML clustering over a whole field where some wells are good and some are empty.** The run
      itself still succeeds; the empty wells are listed as refused. This is the only case that
      matters in practice — a run where *no* well has data was already refused outright, which is
      why this never showed up in testing.
- [ ] **Nothing is written for a refused well**: no curve, and no log-set version allocated. A run
      that reports failure must not also version an interpretation — the rule the rest of the app
      already follows. Check the log set catalog after a run with refusals.
- [ ] **The results table was rebuilt around this.** It leads with a tally ("37 of 40 wells written
      — 3 refused, listed first below"), puts refused wells at the top, tints them, and identifies
      every well by **name** rather than by the UUID it used to print. A refusal is a result, not a
      footnote; on a 200-well run it would otherwise be three lines somewhere in the middle.

## ML — automatic train/blind split by percentage of the DATA (2026-08-07)

Jauhar asked for it directly: give five wells and a percentage, and have SandiBumi choose the split.
It did not exist — "Train wells" was a manual checklist, and the only validation number on offer
(`r2_cv5`) was a random 5-fold over **pooled samples**, which puts consecutive depths from one well
on both sides of the fold. The model was being scored on rock it had already seen a metre away.

Built first as a percentage of the WELL COUNT, which Jauhar corrected the same day: *"not 30% of
wells, but from 30% of total data those 5 wells gave"*. He is right, and it is not cosmetic — on
five wells of 3000/1000/500/300/200 samples, "two of the five wells" is anywhere from 12% to 68% of
the actual rock depending on which two the shuffle draws. The percentage now targets **samples**,
and the well subset is chosen to reach it.

- [ ] **Tick "Hold wells back as a blind test", set a percentage — of SAMPLES.** SandiBumi picks
      whole wells until about that share of the pooled samples is held back. Try it on wells of very
      different length: the number of wells held out will change to suit, which is the whole point.
- [ ] **The result reports the share it REACHED, not the one you asked for**, with the sample count
      beside each side's well names. Whole wells are lumpy and the target is often unreachable; when
      the miss is 5 points or more it says so in a line of its own, and explains that holding wells
      back whole is what makes the share coarse. Two wells can be a third of the field or a
      twentieth of it — the names alone never said which.
- [ ] **It still holds back WHOLE WELLS, never loose samples.** A 70/30 split of pooled samples
      would leak the same way the old CV did — at 0.1524 m sampling the row above and the row below
      are all but the same rock. With fewer than 2 training wells the control says so rather than
      pretending.
- [ ] **The pre-run line no longer predicts a well count**, deliberately: how many samples each well
      contributes is not known until the curves are read and the mask applied, and a number the
      dialog guessed and the run then contradicted would be worse than no number. It says what is
      being aimed at, and warns that few wells make the steps coarse.

## ML — the second split mode: random rows, stratified (2026-08-07)

Jauhar again, third time and explicit: *"real 30% of data, from existing assume 10000 of data,
random sample 3000 data from there with similar statistic taken to be tested as blind"*. So the
conventional ML hold-out now exists beside the by-well one. **They answer different questions and
neither is a better version of the other**, which is why it is a segmented choice rather than a
default with an override.

- [ ] **Tick the blind test, then switch "held back as" to Random rows.** The percentage becomes
      exact — 3000 of 10 000, not "about 3000". Hover each option: whole wells answers *will this
      work on the next well I drill*, random rows answers *has this learned the relationship in
      these wells*.
- [ ] **The draw is STRATIFIED, not flat.** Each stratum (the class for a classifier, a decile of
      the target for a regressor) contributes its own 30%, so the blind set carries the same
      distribution as the whole. A flat draw can put a thin coal wholly on one side.
- [ ] **"How alike the two sides are" is the evidence, and it can fail.** Every input and the target
      are compared, scaled by the fitted side's own spread — an absolute difference cannot be
      compared across GR and porosity. A row past a quarter of a standard deviation is marked in the
      warn colour and the note below says the blind set is not representative. **Try a classifier
      with a very rare facies** — that is the case that should light up.
- [ ] **Every label on the panel changes with the mode, because the same word would be a lie.** In
      sample mode it reads "R² on the blind rows … drawn from wells the model also trained on", and
      the agreement line says the model learned the relationship *in these wells* rather than "the
      fit travels to wells it has not seen". Check both modes and confirm neither claims the other's
      guarantee.
- [ ] **Cross-validation stays grouped by well in BOTH modes.** That is the point: a sample-mode run
      carries one score that can leak and one that cannot, side by side. If they disagree badly, the
      model is memorising depth neighbours.
- [ ] **The mode is in the run record** ("Settings this run actually used"), beside the seed — the
      same number means a different thing under each, so a record without it cannot be re-run.

- [ ] **It warns when a side is too thin** — "A blind set of one well is one opinion, not a spread",
      "Fitting on one well is a model of that well". Both are legitimate runs, so neither is refused.
- [ ] **The seed is yours and it is stated.** Same seed, same wells — a blind score you cannot
      re-run is a blind score you cannot quote.
- [ ] **The results name the wells, not just the count.** "Which wells?" is always the next question
      after a blind score, and a percentage does not answer it.
- [ ] **Three scores side by side, labelled by what they are a score OF**: on the fitted wells, in
      cross-validation, on the blind wells. **Check the gap line underneath** — that gap is the part
      of the fit that does not travel, and it is the number an experienced eye actually reads.
- [ ] **The blind wells still get their predicted curve.** The model is deliberately NOT refitted on
      them afterwards, so you can put that curve beside core in a well the model never saw. Refitting
      would make the curve in-sample and leave the reported score describing a model that no longer
      exists.
- [ ] **Cross-validation is now grouped by well too** (`GroupKFold`), with the scaler refitted inside
      each fold. Where there is only one well it says "random folds within ONE well — not a blind
      score" instead of printing a number that looks like validation.
- [ ] Clustering and reduction do not offer the control: they are fitted on the very wells they are
      applied to, so "held out" could not mean anything there.

## Class curves are never averaged or interpolated (2026-08-07)

`SB-MLA-055`. A facies code is a name that happens to be written as a number. The mean of facies 1
and facies 4 is 2.5, which is not a facies — and it plots as a block track, exports to LAS and reads
back into the next module without complaint. Nothing downstream can tell it is wrong.

Re-framing already guessed well: `looks_discrete` picked MODE for anything that looked like a code
scheme. But it only ever ran on the **Auto** method, so choosing Interpolate or Mean explicitly went
straight through, and there was no record anywhere saying a curve *is* a class curve.

- [ ] **Run Electrofacies (or GMM Facies), then re-frame that well with the method set to MEAN or
      INTERPOLATE.** The FACIES curve comes back resampled by MODE/NEAREST anyway, and the run notes
      say so by name: which curve, what was asked, what was used. A substitution you cannot see is
      the thing this exists to prevent.
- [ ] **The probability curve is NOT protected.** `gmm_facies` writes FACIES_GMM beside FPROB; FPROB
      is an ordinary continuous probability and must stay averageable. Check it still resamples by
      MEAN.
- [ ] **Rename the output, or set an output prefix, and re-run.** The declaration follows the name
      the run actually wrote (TEST_LITHO, not FACIES), so a renamed class curve is still protected.
- [ ] **An ordinary curve is untouched.** A caliper that happens to read whole inches still looks
      discrete to the guesser — set it to MEAN and it stays MEAN. A guess may pick the default;
      only a declaration overrides a decision you made.

### The three modules that would have averaged it anyway (2026-08-07)

Re-framing was only one of the doors. **Frame ▸ Block** could upscale a FACIES curve by MEAN, and
**Condition ▸ Smooth** and **Condition ▸ Despike** would run on it without comment. Worse: the Core
Photos pane already tells you in so many words to *"use Frame ▸ Block with OPT_STAT = MODE, the one
upscale that carries a class code whole"* — and Block **had no MODE option**, so following the
application's own advice fell through to the arithmetic mean.

- [ ] **Block a FACIES curve — MODE is now in the "How a bed's value is taken" list**, and it gives
      the bed's commonest code. That is the fix the Core Photos pane has been pointing at.
- [ ] **Choose MEAN, GEOMETRIC, HARMONIC or MEDIAN on that same curve and it is refused by name**,
      with the reason and the fix in the message. MEDIAN is refused with the rest deliberately: it
      interpolates, so an even-count bed of {1, 2} returns 1.5.
- [ ] **MIN and MAX are allowed.** They land on a sample that really occurs, and a class scheme
      ordered by shaliness has an order even where it has no arithmetic.
- [ ] **Smooth and Despike refuse a class curve outright** and point at Block ▸ MODE. There is no
      safe version of either: smoothing means producing values *between* the ones measured, and on a
      class log a lone code between two others is a thin bed, not a spike — nothing in the numbers
      tells them apart, so a "cleaned" facies log is one with its thinnest beds quietly deleted.
- [ ] **Block, Smooth and Despike an ordinary curve and nothing has changed.** The rule fires on the
      declaration, never on how the values look.

## A model now says how well it travels, and which rows made it (2026-08-07)

`SB-MLA-003` / `SB-MLA-009`. Two things a predicted curve could not tell you before.

**How well the model travels.** A net-pay number computed from a predicted permeability whose
blind-well R² was 0.31 is a different claim from one computed from a measured permeability — and
nothing downstream could tell which it had received. The saved-models list now carries a **blind
score pill** on every row: `blind R2 0.61` in green, amber or red, with the wells, the rows and the
protocol in the tooltip.

The important half is the other one. A model fitted without holding anything back reads **"not
blind-tested"**, in neutral colour, and shows *no number*. Its training score is not a measurement
of how it travels, and putting one there would be the failure this exists to prevent: a delivered
project once showed a training correlation of 0.99 on a curve whose blind-well range was 0.31–0.70.

- [ ] **Open ML Models ▸ Saved models.** Every model carries a pill. Models you fitted before today
      say "not blind-tested" — correctly, because they were not.
- [ ] **Fit one with *Hold wells back as a blind test* ticked, save it, and look at the row.** The
      pill shows the blind score. Hover it: it names how many wells were held back and whether the
      split answers "will this work on the next well" (whole wells) or only "is the relationship
      learnable here" (random rows).
- [ ] **Apply that model to new wells.** The curve records the *same* blind score — it is copied
      from the model, not recomputed, so a curve made by applying a model says what a curve made by
      the fit says.
- [ ] **Check a curve's record** in the Database Inspector ▸ log sets: `params_json` now carries the
      blind block and the training fingerprint beside the model name.

**Which rows made it.** "Trained on 12 wells, 4,300 samples" does not pin a re-run: the same wells
at a later log-set version are *different rows* with the same names and often the same count. Each
saved model now carries a fingerprint of its exact training matrix, in the row tooltip.

- [ ] **Fit the same configuration twice without changing any data.** The fingerprint matches.
- [ ] **Edit one input curve sample and re-fit.** It differs — even though the well list, the
      sample count and the curve list are all unchanged. That is the case the well list cannot see.

## Fitting permeability in log space, without the number changing meaning (2026-08-07)

`SB-MLA-035`. Permeability spans decades, so it is fitted as `log10(k)` — that is where the
relation to porosity is a straight line. **ML Models ▸ Predict a continuous log** now offers that
choice directly: a new **Fit target as** control with *As measured* and *log10*.

The reason it is a control rather than something you do by hand is what happens next. Whatever the
model is fitted on is what the model *predicts* — so a log-fitted run predicts `log10(mD)`, not mD.
Written under the name you typed, in a table headed mD, that is a number no reader can catch: a
permeability mean of **−0.4** is not an error state, it is 0.398 mD in log units, and the rows
around it read −0.4, 1.2, 2.8 and look like a plausible spread. It renders, it prints, and it
reaches a client deck.

So a log-fitted run writes **two curves**, and says so before you press Run:

- `<name>_LOG10` — what the model actually predicted, in log units.
- `<name>` — its back-transform, in the target's own units.

- [ ] **Fit a permeability against your usual inputs with *log10* selected.** Two curves appear in
      the output list. Put both on a log view: the `_LOG10` one spans roughly 0–4, the plain one
      spans the decades.
- [ ] **Export that well to LAS and open the header.** `<name>_LOG10` carries `log10(mD)` and
      `<name>` carries `mD` — provided the target curve had a unit when it was imported. This is the
      one place the units leave the building, so it is the one worth checking.
- [ ] **Read the score panel.** It now says *"Scored in log10(mD) — the space the model was fitted
      in"* above the R² rows. An R² in log space is usually **lower** than the same model's R² in mD,
      and that is not the model getting worse — the linear-space number is flattered by the few
      largest permeabilities, which is exactly why the log fit is the right one.
- [ ] **Press Compare with *log10* selected.** The leaderboard ranks in log space too, and its note
      says so. Compare the ranking against the same leaderboard with *As measured*: on real
      permeability the winner often changes. The log-space one is the ranking that matches what
      Run will fit.
- [ ] **If any of your plugs read zero permeability**, the run reports how many were dropped. Zero
      has no logarithm, and the alternative — flooring it to some small number — is a value nobody
      chose that would drag the low end of the fit.
- [ ] **Switch to a classification or clustering task.** The control disappears; a class label has
      no logarithm.

## One k-means, and one seed default — RESULTS CHANGE (2026-08-07)

`SB-MLA-023` / `SB-MLA-024`. SandiBumi has two k-means engines: the built-in one behind
**Facies ▸ Electrofacies** and **GMM Facies**, and scikit-learn's behind **ML Models ▸ clustering**.
They were set up differently — 8 restarts and a 100-iteration cap in the built-in one against
scikit-learn's 10 and 300, and no convergence tolerance at all on the built-in side. Restart count
and iteration cap are exactly the two settings that decide *which* of the several clusterings the
data supports is the one you get, so **the same curves, the same K and the same seed gave two
different facies schemes depending on which door you came in**, with nothing on either screen
saying so.

Both now run one definition: 10 restarts, a 300-iteration cap, and scikit-learn's convergence
tolerance implemented natively. The values are scikit-learn's own documented defaults rather than
anything invented here, and both moves are in the safe direction — restarts keep the best result by
inertia, so 10 can only find a fit at least as good as 8 did, and the higher cap only affects runs
that had not finished converging at 100.

- [ ] **Re-run Electrofacies on a well you have clustered before, at the same K and seed.** The
      answer may differ slightly from the old one. Where it does, the new one is the better fit —
      it is the lowest-inertia result of more restarts run further.
- [ ] **The seed default is now 42 in every module.** Electrofacies and GMM Facies used to default
      to **7**; the ML suite has always used 42. **This changes what you get from pressing Run
      without touching the seed field.** An old run recorded its seed, so it is still reproducible —
      type 7 back in. Check a saved log set's parameters if you need to reproduce one.
- [ ] **Cluster the same curves both ways** — Facies ▸ Electrofacies, then ML Models ▸ clustering ▸
      k-means over the same well, same K, same seed. On rock where the grouping is clear-cut they
      should now agree. They are still two engines drawing random numbers differently, so on genuinely
      ambiguous data they can land on different local answers; what is fixed is that they no longer
      differ *by configuration*.

## ML — the run records what it actually used, defaults included (2026-08-07)

`SB-MLA-001`. The record kept against every run was the settings you *typed*, which is the one set
of numbers that needs no reporting — you have them. The values that decide a result and appear
nowhere on screen are the ones nobody supplied: `seed` above all, which chooses which of the several
clusterings the data supports is the one you got. A record you cannot re-run from is not a record.

- [ ] **Run any model and open "Settings this run actually used"** under the results. Every
      parameter the run read is there, defaulted rows marked and sorted first — they are the only
      rows you have not already seen.
- [ ] **A defaulted value names where the default came from**, so "who chose 200 trees" has an
      answer. The difference between deciding 200 and having 200 decided for you is invisible in a
      report six months later.
- [ ] **A clamped value states both numbers** — t-SNE perplexity narrowed against a small sample
      count reads "12.3 (asked for 30)". A request the code quietly narrowed would otherwise be
      recorded as the number you typed.
- [ ] **This is what gets persisted**, not the supplied set: check the log set's parameters and a
      saved model's record. That is the half that makes a re-run reconstructable.
- [ ] Native electrofacies/GMM are NOT covered yet — they report through the module framework,
      which has no parameter record. Their `SEED` fallback is still silent. Separate increment.

## The report says which curves a model predicted (2026-08-07)

`SB-MLA-010`. A predicted permeability looks exactly like a measured one on a track: smooth,
plausible, in the right units. By the time it has been through a cutoff and into a hydrocarbon pore
volume, nothing on the page says a model made it. Until now the lineage stopped at the database —
the run was recorded, and the deliverable did not mention it.

Both documents now carry a **Machine-learning provenance** section, immediately after the
methodology table: the PDF (**Plot ▸ Deliverables ▸ Report…**) and its editable Word twin
(**Save Word…**). Six columns — the curve and what it is a prediction of, the model and algorithm,
the inputs in the order they were fitted in, what it was trained on, how well it travels, and the
log set, run date and training fingerprint.

Above the table, printed and not assumed: these curves were **predicted, not measured and not
computed by a petrophysical equation**, and every number derived from them inherits the blind
performance stated beside them.

- [ ] **Run a model on a well, then generate that well's report.** The section is there, and it
      names the model you used.
- [ ] **Re-run the same model, or a different one, over the same curve name, and re-generate.** The
      table still has one row, describing the run that made the curve now in the report — not both.
      This is the half worth checking: a table naming a superseded run credits a model that did not
      make the number on the page, which is worse than no table at all.
- [ ] **Look at the blind column.** A model fitted with wells held back reads its blind score and
      how many wells; one fitted without reads "not blind-tested" and shows no number.
- [ ] **Save the Word version and compare it against the PDF.** Same rows, same wording, same
      caveat. They are built from one definition, so they cannot drift — and the editable document
      is the one a client actually opens, so the caveat has to be in it.
- [ ] **Generate a report for a well with no ML curves.** No section at all — not an empty table
      under a heading implying there is a model somewhere.
- [ ] LAS export does not carry this yet. The honest place for it there is a `~Other` block, and
      that is a separate increment.

## A saved model says what rock it learned from — and every log is offered as an input (2026-08-07)

`SB-MLA-002`, `SB-MLA-004`, `SB-MLA-005`, plus a bug you found.

**The bug first, because it was costing you inputs.** The ML dialog has always had a checkbox per
curve, but the list it was built from only ever contained the six standard columns and whatever
modules had computed. It never looked at the imported curve store — so a well delivered with fifteen
logs offered you five, and the extra runs, PEF, CALI, spectral GR and everything else looked as
though the app could not use them. It always could: the fetch path reads them by mnemonic and has
done since the generic store shipped. Only the picker was blind. Every imported mnemonic is now in
the list, tagged with its unit and where it came from, and you tick as many as you want.

**What a saved model now records about its own training rock.** Three facts that were missing, all
of them things a re-run has to match and none of them derivable from the well list:

- **Which log set each well's rows were read from**, per well — because the input set resolves per
  well. Ask for `FINAL` across a field where three wells do not have one, and those three quietly
  read live values while the rest read stored ones. The record says so, and the run tells you at the
  time, by name.
- **The mask, by name, and what it took out per well.** Masked samples and samples dropped for a
  missing curve are counted separately: they call for opposite fixes, and a single "rows not used"
  number reads as the mask's doing when often it is a curve nobody logged. The run reports the total
  with the worst well named.
- **The interpreter and every library that took part** — Python, numpy, scipy, scikit-learn, joblib
  and xgboost. The model file is a pickle, so joblib is the one that actually reads it back, and it
  is the component nobody thinks of.

**And a "has drifted" tag on the model row, before you apply anything.** If the log set a model
learned from has been superseded or deleted, or the libraries on this machine have moved since it
was fitted, the row says so and the tooltip names what changed. This had to be at pick time rather
than at run time: a run can only report its own runtime after it has predicted, and by then the
curves are written.

- [ ] **Open ML Models on a well with imported logs.** Every mnemonic in the well is in the input
      list, not just GR/RHOB/NPHI/DT/RT/CALI. Each shows its unit and whether it was imported or
      computed. Tick several and train — the ones you ticked are the ones it uses.
- [ ] **Train a supervised model with a log set selected, save it, then hover its row in Saved
      models.** The tooltip names the set and version it read from.
- [ ] **Train one with a mask curve.** The run message says how much it excluded, as a percentage,
      and names the well that lost the most. The tooltip names the mask curve.
- [ ] **Train one with no mask.** It says "no mask was applied" — not blank, and not confusable
      with a mask that ran and flagged nothing (which says exactly that instead).
- [ ] **Re-run the module that made a training input, so its log set version moves. Reopen ML
      Models.** The old model's row carries a warning naming the well and the version step. This is
      the one worth checking: it is the difference between a model you can reproduce and one you
      cannot, and nothing else on the row would tell you.
- [ ] **Look at a model saved before today.** No warnings at all. A record that did not exist is not
      a mismatch, and a tag that fires on every old model is a tag you would stop reading.

## The same run twice gives the same curves — proven, not promised (2026-08-07)

`SB-MLA-008`, the last of the P0 group.

The competitive finding behind this one is blunt: on a pooled five-well set at K = 15, an unseeded
clustering gives different cluster numbers every time it runs, so a facies track in a delivered
report cannot be reproduced. Nobody else offers a guarantee here. We now do, and it is **measured
rather than asserted** — every algorithm across all four tasks runs twice through real scikit-learn
and the two results are compared **bit for bit**, not to a tolerance. Fifteen configurations, all
identical. A tolerance would hide exactly the drift the check exists to catch, and comparing values
rather than bits would let a sample that turned into a gap on the second pass pass as "both missing,
both fine".

Every run has a seed on the record even when you never touched the seed box, because the runner
records the default it used along with the fact that it was a default. So the failure above cannot
happen here by omission.

**Where we do not promise, we say so before you press Run.** One case qualifies today, and it is
our own code rather than a claim about somebody's library: `Gradient boosting` fits XGBoost where
XGBoost is installed and substitutes scikit-learn's version where it is not — recorded as the same
algorithm name either way. Same request, same seed, same wells, a different estimator depending on
the machine. Choose it on a machine without XGBoost and a line under the algorithm says exactly
that. This machine does not have XGBoost, so you should see it.

Changed libraries, changed rows and a superseded input set are the other three ways a re-run stops
matching — those are not properties of an algorithm and are reported separately, on the model row.

- [ ] **Pick Regression ▸ Gradient boosting.** A line appears under the algorithm saying XGBoost is
      not installed and this will fit the scikit-learn substitute. Switch to another algorithm and
      it disappears — it is not a permanent banner.
- [ ] **Run the same clustering twice on the same wells with the same seed.** The facies numbers are
      identical, not merely similar. Change the seed and they are not.
- [ ] **Run the same regression twice.** Same curve, and the same reported R², to every digit.

## The ML pane, rebuilt as four sections (2026-08-07)

Your seven points from the click-through, in order.

**1 — The style.** Every control you struck through (Task, Algorithm, Target curve, Mask, Output
curve) was a bare dropdown inheriting the browser's own look, while the controls beside them looked
right because they are segmented pills that carry their own styling. `.form-control` is the design
system's input style and exactly one control in the pane was using it. They all do now.

**2 — Four sections.** **Input · Data QC · Model · Results**, in the order the work is done. One
scrolling column had grown to a dozen rows in no useful order — the algorithm at the top, its
parameters two thirds down, the output curve below that — so setting up a run meant scrolling past
everything twice. The section strip stays put while a long Results panel scrolls under it, and
**Run Model** is a footer outside the sections, reachable from all four.

**3 — One algorithm list.** The Task dropdown is gone. The algorithm picker now groups by what you
are predicting: **Continuous log**, **Discrete log**, **Electrofacies**, **Reduction**. Random
Forest appears under both supervised groups, which is honest — it is two estimators with one idea.
Picking the algorithm sets the task, so changing your mind no longer resets your choice.

**4 — Data QC that knows what it is checking for.** Every finding is about the data **and the
model**, never the data alone, because "is this data good" has no answer: four orders of magnitude
between two curves is fatal to k-means and irrelevant to a random forest. Same wells, same curves —
SVR gets a red *"RES_DEEP would swamp every other input, turn on Standardize"*; Random Forest gets a
green *"scale does not matter to this estimator"*. It also reports how many rows can actually reach
the fit, which curve caps that, which wells are missing which curve, whether your target looks like
a class code or a continuous log, and whether K is more classes than the data can carry.

**5 — Parameters.** They were always editable. What you could not see was which ones you had
changed — a grid of numbers looks the same whether they are your settings or the library's. A
changed field now marks itself and offers **Reset to defaults**, and each field's tooltip names its
default. The run has always recorded the difference; the form now shows it.

**6 — Predicted vs measured, per model.** In Results, under the comparison charts. Every point is a
prediction made by a model that had not seen that row, so the picture answers the same question the
score does. Pick which model you are looking at from the dropdown on the panel. **Coloured by
well** — that is the reading the score cannot give you.

**7 — R² and RMSE across models.** Two bar panels side by side, never one axis, because higher is
better for one and lower for the other. Where they disagree the panel says why.

- [ ] **Open ML Models.** Four sections across the top, and every dropdown and field looks like the
      rest of the application.
- [ ] **Open the Algorithm list.** Four groups by what you are predicting; Random Forest under both
      supervised ones. Pick a clustering algorithm — the target curve and train wells disappear,
      because there is no target.
- [ ] **Pick your curves and wells on Input, then open Data QC.** It measures that selection.
      Read the row-count line first: it is what everything else is read against.
- [ ] **Switch between Support Vector Regression and Random Forest and re-check.** The scale finding
      changes from red to green on the same data. That is the point of the section.
- [ ] **Change a parameter.** Its label goes accent-coloured and a Reset appears.
- [ ] **Run Compare algorithms on a real target.** The Results section gets the leaderboard, the
      dot-and-whisker score chart, the R²/RMSE panels, the predicted-vs-measured crossplot and the
      cross-model "which curve carries" panel.
- [ ] **On the crossplot, step through the models.** A good one sits on the dashed 1:1 line. Look
      for one whose cloud is tight but rotated off it (mis-scaled, correctable) or flat at the mean
      (learned nothing). Both can score the same R².
- [ ] **Look at the well colours on the crossplot.** This is the one worth your time: a blind R² of
      0.7 over three wells can be 0.9, 0.85 and 0.1, and the third well is the one that says whether
      the curve travels.

## ML - one model where a curve exists, a smaller one where it does not (2026-08-07)

Your cross-check: *"assume user have 4 curves, model should still run even 1 curves only half depth
coverage, (model only predict using 3 curves on the other half depth coverage)"*. It did not. A row
reached the fit only where EVERY input had a value, so one curve logged over half the interval
deleted the other half of all four - in the training and in the prediction. On a field where each
well is missing something different, the intersection can be nearly empty while every individual
curve looks well logged.

**Model section, "Partial coverage": Fit a model per available-input pattern.** Off by default,
because with it on the curve is made by more than one model and you have to be told which.

How it decides. It looks at which inputs are actually present at each depth, takes the patterns that
really OCCUR (not every possible subset - four curves would be fifteen hypothetical models), and
fits one model per pattern. **Each depth is then predicted by the largest model whose curves it
carries**: where all four exist, the four-curve model; where one is short, a three-curve model
fitted on every row carrying those three - including the four-curve rows, which carry them too.

What it will not do, and why each refusal is there:

- **A segment with fewer than 30 training rows is not fitted**, and the depths it would have covered
  are left blank. Named in the result with its row count. A model fitted on eighteen plugs is not a
  weaker answer, it is a different kind of object, and shipping one under the same curve name would
  make the curve's quality vary down its length with nothing recording where.
- **The scores are never averaged.** Each segment reports its own blind score on its own rows. An
  R2 over both would describe neither, and the lower one is not the worse model - it is the one that
  had fewer curves to work with.
- **Each segment saves its own model**, suffixed `_3CURVE` / `_4CURVE`, so a saved artifact says
  from its name which curves it needs.
- **The curve is written ONCE**, at the end, with every segment recorded in its provenance - because
  "which model produced this curve" genuinely has more than one answer along its length.

Data QC changes with the switch. With it off, a short curve is a warning that names the three ways
out - drop the curve, drop the wells, or turn this on. With it on, the same curve is green and says
the run will fit a model with it and a model without. The row-count headline also changes, and there
is one case worth knowing: if the thinnest curve is your TARGET (core permeability usually is), this
does NOT lift the cap. Every model is fitted against the target, so no segment can see a depth the
target does not reach. What you buy there is coverage of the PREDICTION, not more training data -
and the panel says so rather than leaving you to work it out.

- [ ] **Pick four curves where one is short, and run without the switch.** Data QC warns, and names
      the switch as one of the three ways out.
- [ ] **Turn it on and re-check Data QC.** The same finding goes green and says what will happen.
- [ ] **Run it.** Results opens with one card per model: the inputs it uses, how many depths it
      predicts, what share of the curve that is, how many rows fitted it, and its own blind score
      with the protocol spelled out.
- [ ] **Read the two cards against each other.** They are not a ranking. The four-curve card should
      predict the interval where all four exist; the three-curve card the rest.
- [ ] **Look at the written curve in a log view.** It should be continuous across the depth where
      the short curve stops - that is the whole point - with no step at the boundary that is not
      geology.
- [ ] **Save the model and look at the Saved models list.** Two entries, `_4CURVE` and `_3CURVE`.
- [ ] **Force a skip**: choose an input present in only a handful of rows. That segment should
      appear as a warn-tinted card stating its row count and that its depths were left blank,
      never silently vanish.

## ML - writing the output at the target sampling (2026-08-07)

Your item 4: *"each log has different resolution, sometimes it looks low frequency such resistivity,
sometimes high such rxo, gr, or nphi. Result should adjust their frequency to log target"*, then
*"writing output at target sampling"*.

A model fitted against a target read every 0.5 m predicts at every INPUT depth, so it wrote a value
every 0.1524 m. That curve claims three times the vertical resolution anything it learned from ever
had, and on a log view it is indistinguishable from a log a tool actually ran at that rate.

**Model section, "Output resolution": As predicted | Target sampling.** Open Data QC once and the
target's own measured sampling is filled into the block-thickness box - the median across your
training wells, not the mean, because one well logged at a different rate would otherwise offer you
a spacing no tool ever ran at. It stays editable; nothing is decided for you.

What it does: one value per interval, held across the interval. **The depth frame does not change.**
Computed curves are read back by exact depth match, so a curve written at its own coarser sampling
would land on depths the well does not have and read back all-missing - which is why re-framing is
Reframe's job and this is the same discipline Block (Upscale) already follows. The consequence for
you: **set the curve's draw style to Step in the curve editor**, or the log view draws a gradient
between two block values that nothing measured. The run says so in its notes.

Three rules inside it:

- **Blocks are anchored on an absolute depth grid**, not on each well's first sample. Anchored per
  well, two wells would get the same block thickness at different block boundaries, so a bed sitting
  mid-block in one well straddles a boundary in the next. The numbers stay plausible and stop being
  comparable.
- **A class curve takes the block's commonest CODE, never a mean.** The mean of facies 1 and facies 4
  is 2.5, which is not a facies. A `_PROB` curve beside it is a real number and averages normally.
- **A depth the model did not answer stays missing.** It never inherits a value from its block.

Under a log10 target transform the block mean runs in log space, which makes it the **geometric mean
of the millidarcies** - the standard permeability upscale - by construction rather than by a special
case.

A saved model records the resolution it was made to write at, and **applying it later inherits that**
along with its log set. A fit reviewed as a 0.5 m answer, propagated at the input sampling, would be
a curve at a different resolution from the one you signed off, under the same model's name.

- [ ] **Open Data QC with a coarse target selected** (core permeability, or a blocked log). Then open
      Model: the block thickness box is already filled with its measured spacing.
- [ ] **Run once As predicted and once at Target sampling**, into two output curve names, and put
      both in a log view. The second should be visibly blocky and the first should not.
- [ ] **Set the blocked curve's draw style to Step** (right-click the curve in the log view). Before
      you do, it draws sloping lines between block values - that is the display lying, not the data.
- [ ] **Check the depth frame did not change**: the blocked curve should have a sample at every depth
      the unblocked one does, just repeated within each block.
- [ ] **Save the model, then Apply it to other wells.** The result notes should say it was written at
      the same resolution, without you setting anything.
- [ ] **Try it on a classification run.** The class curve should hold whole codes - never a 2.5
      between facies 2 and 3 - while its `_PROB` companion averages.
### Three surfaces that were painting with a token this app does not have

`--panel` is not a token in this design system; the surface tokens are `--bg-app`, `--bg-panel`,
`--bg-panel-alt` and `--bg-hover`. Three rules referenced one that is declared nowhere, and a
`var()` naming a token nothing declares does not fall back to anything — it resolves to
transparent. **Check these in the DARK theme.** The light theme is the reason all three survived
since they were written: a transparent card on the cream ground looks about right, and only on
dark does a see-through panel become obvious.

- [ ] **The Intake pane's column-mapping grid.** Load a delivery and scroll the preview past the
      first screen. The header row carries the role pickers and stays put while the rows scroll
      under it — it should be solid, with the role tints readable on top of it. It had been
      see-through, so the numbers from row 180 were sliding through the pickers you were setting.
- [ ] **The same pane's array preview**, below the grid. Same check, same header.
- [ ] **The Statistics panel.** Scroll a long table; the Well / Zone / Item header should be
      solid. This one was not in the original report — it was found by listing every custom
      property the stylesheet declares and diffing that against every `var()` in it. It looked
      defended, because it named a fallback: the fallback was undeclared too, so it collapsed
      exactly the same way. Worth knowing that check exists, since this is twice now.
- [ ] **The Photo Log pane's fluorescence cards** (Advance ▸ Core Imaging ▸ Core Photos…, with
      more than one kind of show declared). Each kind sits in its own card; on dark the cards
      should read as cards rather than as one flat column.
- [ ] The role tints in the Intake header are unchanged in meaning — still the same colour per
      role, still translucent so they read as a tint *of* the header rather than a colour of
      their own. They now have something opaque underneath to tint, which is what was missing.
      If any tint now looks heavier or flatter than you remember, say so.
- [ ] **A fourth, found independently the same day and fixed on the ML branch**: the coverage-segment
      cards in the ML pane's Results. Same token, same white-on-dark result. Mentioned here so the
      count in this section's title is not read as the whole tally.

## ML pane, round 3 — the five points from your click-through (2026-08-07)

**1 + 2 — Run Model moved, and propagation became its own section.** These were one change: the
same question ("what does this button apply to") answered by putting each button with the choices it
consumes. Run Model was a pane FOOTER visible from every section — standing under Data QC it read as
acting on what Data QC was showing. It is now last in **Model**, after everything it consumes.

**Model Distribution** is a fifth section with its own well scope, interval, log sets, output name
and mask. Its mask is its own on purpose: those are different wells, and a bad-hole flag is a
property of the hole it was computed in. What it does NOT restate is the model's features and their
order — those travel inside the artifact, and letting a caller restate them would invite them to
differ. It prints them instead.

**Tops bounding** confines both the rows a model learns from and the depths it writes. Pick a marker
and it fills two editable depth boxes; the run always sends numbers, so what was used stays
recoverable after somebody moves the tops. Three rules, all silent when wrong: an open side stays
OPEN (the deepest marker runs to TD, and reading a missing base as "no window" would widen the run
back to the whole well under a zone's name); the base is EXCLUSIVE while the top is inclusive, so two
abutting zones cannot both claim the sample on their shared marker; and a NaN depth is in no window.
**The limitation, stated in the control:** markers are read from the SELECTED well and applied as
depths to every well in the run. Resolving per well sounds more correct, but a well lacking the
marker would fall back to its whole length and join the fit as a different population.

**3 — Three algorithm groups.** Universal (Random Forest, Support Vector — the families the runner
fits both ways, listed once with a Continuous/Discrete control beside them), Continuous only,
Discrete only. Electrofacies clustering sits under Discrete because it writes class codes; PCA and
t-SNE under Continuous because they write component curves. Nothing was dropped.

**4 — Copy / Image / SVG / Print** under the score chart and the predicted-vs-measured crossplot.

**5 — sampling is not resolution.** You are right, and this is the half that needs no decision from
you: every supervised run now MEASURES how much the prediction wiggles against the measured target it
learned from, and says so when it falls short. A prediction is always smoother than its target
because the model can only carry through the detail its inputs contain — a curve read over feet
cannot produce detail measured over inches. **Nothing synthesizes the missing detail**, because that
produces a log that looks better resolved without being better known. Two questions on that are
waiting for you.

- [ ] **Open ML Models.** Five sections. Run Model is at the bottom of **Model** and nowhere else.
- [ ] **On Input, pick a marker in Interval.** Top and Base fill in; the deepest marker leaves Base
      empty (runs to TD). Type over them and the marker selection clears — the depths win.
- [ ] **Run bounded.** The written curve should be blank above the top and below the base, and the
      notes should say what it was confined to.
- [ ] **Check the base marker's own sample.** It belongs to the zone BELOW. Run two abutting zones
      into two curves and no depth should appear in both.
- [ ] **Save a model, then go to Model Distribution.** Pick it: the note names the curves it needs
      and the order. Choose a DIFFERENT well set and a different interval, give the curve its own
      name, Distribute. It must not inherit the fit's wells or interval.
- [ ] **Open the Algorithm list.** Three groups. Random Forest and Support Vector appear once, under
      Universal, with Continuous/Discrete beside the picker.
- [ ] **Switch Support Vector to Discrete.** The picker keeps its name and the parameters drop to the
      classifier's set. Then pick Random Forest — it should STAY on Discrete.
- [ ] **Run a regression where the target is much sharper than the inputs** (core porosity from GR
      and deep resistivity). The notes should say the prediction is smoother than the measured log,
      and by roughly how much. Read a thin bed off it and see whether you agree.
- [ ] **Export a chart.** Copy, Image, SVG and Print under the score chart and the crossplot. Open
      the SVG somewhere else — the colours must survive leaving the app.

**Section order — Distribution now follows Results.** Propagating a model you have not looked at is
the one move this pane should not make easy, so the section that spends that judgement comes after
the one that supplies it. A successful run now jumps you to Results; a failed one deliberately does
NOT, because the failure message lives on the Model tab.

**Three from the ML PRD backlog.** A cancelled run's log sets now say they came from a cancelled run
(a partial set is not corrupt and not empty — it looks exactly like a finished run over fewer wells).
A sample DBSCAN rejects is now its own class rather than a gap, so a hole in a facies curve means one
thing only: never evaluated. And a model a delivered curve still cites cannot be deleted without a
word.

- [ ] **Run a model, watch where you land.** A successful run should drop you on Results. Make one
      fail (uncheck every input curve, or pick a target no well carries) — you should STAY on Model,
      with the reason under the button.
- [ ] **Model Distribution is now the last tab**, after Results.
- [ ] **Start a run over many wells and press Cancel partway.** The wells that were cut say
      "cancelled". Then open the log set on a well that DID get written — it should say it came from
      a cancelled run and how many wells of how many were covered. Without that mark it would be
      indistinguishable from a clean run over the wells you kept.
- [ ] **Run DBSCAN clustering** with a tight eps so it rejects a good fraction. On the log view the
      rejected samples should be a NEUTRAL GREY block, not one of the facies colours, and the
      crossplot legend should say **Rejected** rather than F-1. Gaps in that curve should now mean
      only "no input data here".
- [ ] **Print that same DBSCAN curve to a composite/PDF.** The grey must survive to paper — this is
      the one that would have quietly printed an outlier as real rock.
- [ ] **Delete a saved model that produced a curve you still have.** It must refuse and NAME the
      wells and curves. Say yes to the second question, then look at that curve's provenance — it
      should say the model was DELETED, not just print its name. Delete a model nothing used: that
      one should just go, with no second question.

**Item 5's second half, and a PRD sweep.** Spectral matching and two curves, as you chose. Plus
twelve more PRD requirements, all of the same family: a run that did something other than what was
asked and did not say so. And an outage found while writing it up — the Python runner was passed on
the command line, Windows caps that near 32 KB, and adding comments broke every ML feature at once.

- [ ] **Tick "Also write a spectrally textured copy" and run a regression.** You get TWO curves:
      `<name>` unchanged, and `<name>_SIM` with the frequency content the target has. Plot both.
      `_SIM` should look like a real log; `<name>` should look like a prediction. **The point is that
      the honest one is the plainer one.** Read the note under the run — it should say the detail is
      one realisation of many, right in its statistics and arbitrary in its placement.
- [ ] **Check the means agree.** `_SIM` must not shift the level of the curve, only add wiggle.
- [ ] **Try to use `<name>_SIM` as an input curve for another model.** It must REFUSE and tell you to
      use the plain curve. This is the one that would otherwise quietly train a model on invented
      detail.
- [ ] **Run the same fit twice with the same seed.** `_SIM` must be identical both times — it is a
      simulation, not a random doodle.
- [ ] **Run PCA twice.** PC1 must not flip sign between runs. Check `loadings` in the results — it
      should tell you what each component is made of, per input curve.
- [ ] **Run a clustering with k larger than you have samples**, and one with k too high for the data.
      The first should say it clamped k; the second should say fewer clusters came back than asked.
      Neither should quietly return a different number of clusters than you typed.
- [ ] **Run k-means with a very low iteration cap** if you can — the result should say it did NOT
      converge, rather than handing you labels that look final.
- [ ] **Look at any score in the results.** Every one should now be able to tell you what it is a
      score OF — fitted rows, folds of the same wells, or wells the model never saw. A one-well run
      should say its cross-validation is not a blind score.
- [ ] **Open a curve made by ML on a well that was NOT in the training set** and read its provenance.
      It should say this well did not train the model, so its curve is an extrapolation.
- [ ] **Sanity check that ML still runs at all.** This is the outage check: any fit, any algorithm. If
      you see "The filename or extension is too long", something reverted.

## ML — Also run, in a row that fits, with each algorithm's own settings (2026-08-07)

Your three points from the screenshot.

**The space.** Also run was a column of full-width checkboxes — about 168px for seven algorithms, in
a pane whose whole job is the panel beside it. It is now a row of chips that wrap: 50px, two rows at
the usual pane width. Same algorithms, same ticks.

**The missing settings.** Each chip carries a **⚙**. Open it and you get that algorithm's own
parameters, the same fields the main picker shows. Only what you actually CHANGE is stored and sent —
an untouched field stays a default and is recorded as one. That distinction is not cosmetic: a
co-run that silently posted the full default set would make every comparison read as hand-tuned, and
the run record could no longer tell your choice from the runner's. A **dot** on a chip marks the
algorithms carrying settings of their own, and the note under the row names them.

**7 versus 4.** You counted seven algorithms available for a continuous task and only four offered
as co-runs. The three missing ones are Analysis (PCA), t-SNE and the clustering family: they have no
target curve, so there is nothing to score a prediction against, and putting them on a leaderboard
beside a regression would be comparing two different questions. The row now says that itself rather
than leaving you to count.

- [ ] **Open ML Models → Model.** Also run should be a wrapping row of chips, not a column. Count the
      rows it costs you.
- [ ] **Tick two or three.** Then open one chip's ⚙ — the other ticks must survive. (They used to be
      cleared by the re-render.)
- [ ] **Change a parameter in the ⚙ panel.** The dot should appear on THAT chip immediately, and the
      note under the row should name it.
- [ ] **Change it back to the default value.** The dot should go — nothing is stored, so nothing is
      claimed.
- [ ] **Run with two tuned co-runs.** Open the run record for each: the parameter you changed should
      read as yours, everything else as a default.
- [ ] **Read the note about the algorithms that cannot be co-run.** Does the reason match what you
      expected when you counted seven?

## Panes that fill the space they are given (2026-08-07)

Your point about resizing. These panes were pinned to a 620px column — the right width for a
one-shot fit form, and wrong for a pane carrying tables, a confusion matrix and a leaderboard. In a
wide window the card simply ended and the dock showed past it. **ML, Facies tie-in, Monte Carlo,
Lorenz and HFU now fill their pane.** Thomeer and saturation-height deliberately do not: they really
are a column of fields, and widening them would only stretch the fields.

**Filling is not stretching**, which is the part worth checking. Lifting the cap on its own gave a
1210px-wide Output curve box — worse than the gap it filled. So single-line controls keep the width
they always had, and where the pane is wide enough the **ML form gains a second column** instead.
That is measured against the pane, not the window, so a narrow ML pane in a wide window stays one
column, which is the normal case in a docked workspace.

- [ ] **Open ML Models and drag its splitter wider.** The card should grow with the pane, not stop
      at a fixed column with grey to the right of it.
- [ ] **Keep widening.** Around the point where two readable columns fit, the form should go to two
      columns and get shorter — you should see more of it without scrolling.
- [ ] **Check nothing looks stretched.** The Algorithm picker and the Output curve box should stay a
      sensible width; the notes, the co-run chips, the tables and the charts are the things that
      should be using the extra room.
- [ ] **Narrow it back down.** It should return to one column cleanly, with no clipped controls.
- [ ] **Maximise the window with ML docked beside the Wells pane.** The ML pane can be narrow in a
      wide window — it should stay one column there, not split because the *window* is wide.
- [ ] **Same widening check on Facies tie-in, Monte Carlo, Lorenz and HFU.** Tables and plots should
      use the width.

## Facies tie-in: two percentages, and a threshold that is yours (2026-08-07)

**A purity and a recognition rate are different numbers.** The tie-in reported one percentage per
reference class — of the samples core calls RT2, how many the model found — and called it purity.
Geolog reports the *other* axis for the same table and calls it a recognition rate. They disagree
whenever the classes are imbalanced: a small rock type can be perfectly found (100% by row) while
the label the model gives it is mostly something else (18% by column). Both are true of the same
cell, and a bare "72%" in a report says nothing about which was meant.

You now get both, as two tables titled by the question each answers, and the confusion matrix has a
**Counts / % of reference / % of predicted** control with the denominator stated under it.

**The acceptance threshold ships empty and stays empty.** The method note says to accept the mapping
when dominant-class purity is above a threshold and names no value; nothing SandiBumi holds names
one either. So there is no default. Leave it blank and you get the purity with no verdict and a line
saying the bar is yours to set for this field. Fill it in and the verdict is recorded together with
the number it was judged against.

**Core plugs that landed nowhere are counted.** Plugs are matched to the nearest log sample within a
metre and dropped when there is none. That rule, and how many plugs it dropped, now print with the
result — a k variance reduction over nine of ninety plugs used to look identical to one over ninety.

- [ ] **Run a tie-in on a well where one rock type is much rarer than the others.** Do the two
      tables disagree for that class? They should — that disagreement is the point.
- [ ] **Switch the matrix between Counts, % of reference and % of predicted.** Does the caption under
      it always tell you what the numbers are divided by?
- [ ] **Run with the threshold empty.** No verdict, and a line saying why.
- [ ] **Set it to something you would actually accept and re-run.** Verdict appears; check the
      Processing history records the threshold as well as the purity.
- [ ] **Type 90 instead of 0.9 — or rather, check the field is in percent.** It should refuse
      anything that is not a purity, rather than reading it as "never accept".
- [ ] **Run on a well whose core extends below the logged interval.** Does it say how many plugs
      found no sample in range?

## ML: a curve on the wrong depth grid now says so (2026-08-07)

The one that would have cost you a day. Inputs are joined to the run frame by **exact depth
equality** — nothing is interpolated, snapped or gap-filled. So a curve delivered on 0.1524 m, used
in a run whose frame is 0.5 m, coincides at no depth at all: it is fully logged, fully stored, and
reads as absent. Every count the run reported treated it exactly like a curve the well never had,
and the message said "missing input curve" — which sends you hunting for a log that is sitting
right there.

The run notes now tell the two apart, and name Reframe as the fix. This reads the same measurement
the **Data QC** section already shows per curve, so the note and the panel cannot disagree.

- [ ] **Fit a model using a curve that came in on a different sampling from the well's frame** (an
      old run, a re-framed set, a vendor delivery at 0.1524). The notes should name that curve, say
      it exists but landed nowhere, quote both spacings and point at Reframe.
- [ ] **Now try one the well genuinely does not have.** That must still read as a *missing* curve —
      different problem, different fix.
- [ ] **Run an ordinary fit where everything shares the frame.** No framing note at all. If it fires
      on every run it stops being read.

## A normalization basis that holds still when you add a well (2026-08-07)

This one is worth reading before you use it, because the problem it fixes leaves no trace.

**What was happening.** When inputs are standardized, the mean and spread come from the wells in
the run. Add one well to a model-build set and every mean and every spread is recomputed — so every
boundary expressed in them moves, **including in the wells you did not touch**. A DBSCAN `eps` of
0.5 is 0.5 standard deviations of whatever happened to be selected that day. Your earlier
interpretation does not come back, nothing tells you anything changed, and both answers look
equally sensible on screen. Geolog offers a fixed basis for exactly this; IP has no equivalent.

**What you get now.** In **Data QC → Normalization**, under the standardize tick, a choice:

- **From the data** — what every run did before, unchanged. The note now says plainly that the
  basis moves when the well set does.
- **Fixed limits** — each curve normalized onto 0–1 against a low and a high *you* set. Those do
  not move when the well set does, so a boundary found today still means the same thing next month.

**The limits are yours and stay empty.** GR normalized 0–150 and the same GR normalized 0–200 give
different clusters and both look right, so SandiBumi will not pick them. A run whose inputs are not
all covered is refused, naming the ones that are not.

**And a retrain now tells you if the space moved.** Refit a model under a name you have used before
and the run says how far the basis shifted — in standard deviations of the *old* basis, because
that is the unit any threshold you carried over is already in.

- [ ] **Open Data QC.** Under the standardize tick: a **From the data / Fixed limits** control, with
      a line explaining each.
- [ ] **Untick standardize.** The basis control should disappear — it means nothing when nothing is
      being normalized.
- [ ] **Pick Fixed limits.** A row per ticked input curve, each with an empty low and high.
- [ ] **Tick another input curve on the Input tab, come back.** The table should have gained that
      curve, and anything you already typed should still be there.
- [ ] **Press Run with a box still empty.** It must refuse and name the curve — not run with a zero.
- [ ] **Fill them in and run a clustering.** Read the notes: they should say the basis is your fixed
      limits and that `eps` is in fractions of each declared range.
- [ ] **Now the real test.** Fit a model on a set of wells with **From the data** and save it. Add
      one more well — ideally a shalier or cleaner one — and refit under the *same* name. The run
      should tell you the feature space was rescaled, name the curve that moved most, and say by how
      much. **Then judge whether your earlier boundaries still mean what you thought.**
- [ ] **Do the same with Fixed limits.** No rescale message, because nothing moved. That is the
      whole point of the feature.
- [ ] **Apply a saved fixed-limits model to a new well.** It must use the same limits it was fitted
      with, not recompute anything.

## The synthetic log cannot cheat by copying itself (2026-08-07)

Nothing to click for this one — it is a guard, and the reason it is worth knowing about is that the
failure it prevents looks like success.

**Synthetic Log (KNN Predict)** works by finding, for each depth, the most similar depths elsewhere
in the well and averaging what the target curve reads there. If it were allowed to count a sample as
its own neighbour, then at K = 1 the closest match to any depth would be that depth itself, at
distance zero — so the synthetic would reproduce the raw curve **exactly**, every time. The error
would be zero, any set of predictors would look perfect, and a synthetic RHOB would simply echo the
washed-out log it was there to replace, quietly defeating MAX_RAW.

The guard has always been there. What was missing was anything to keep it there: it is one line, of
the kind a later reader removes as redundant. There is now a test that fails loudly without it — I
checked by disabling the line, and it reports **60 of 60 samples reproduced exactly** rather than
drifting slightly off.

- [ ] **If you use Synthetic Log with MAX_RAW on a washed-out RHOB**, sanity-check the result against
      a good-hole interval: the synthetic should differ from the raw curve everywhere, not track it.
      If it ever looks identical to the input, that is the failure above and worth telling me about.

## Where the vendors disagree, the disagreement is on screen (2026-08-07)

Three packages ship three different values for the number of clusters, and none of them tells you the
other two exist. None of them can, either — no vendor has standing to publish a competitor's
defaults. SandiBumi sells no competing default, so it is the one tool that can put them side by side,
and that is now what the **K** field does.

- [ ] **Open Electrofacies (Facies ribbon) and look under the K field.** There should be a collapsed
      line reading *Shipped values elsewhere (5) — this number is not settled*. Expand it.
- [ ] **Read the four positions.** IP advises 15–20 as a first-stage count and 4–5 consolidated;
      Techlog ships a hard 5 in two separate modules; Geolog states none at all. Each names its
      product and the document it was read from.
- [ ] **Check that Geolog's row is there at all.** "None stated" is an entry, not a blank to be
      dropped — it is the row that says the number is not settled. Two vendors alone would read as
      agreement.
- [ ] **Check where SandiBumi's own 5 sits.** It must be in the list and it must be **last**, marked
      as ours, and it must say plainly that it is a starting point rather than a fitted number.
- [ ] **Open GMM Facies and the ML pane.** Same panel, same four values. They read one registry, so
      the three places can never disagree about one number.
- [ ] **Run a clustering with K left at 5, then again at 17.** The run notes should say which cited
      values your choice agrees with — 17 agrees with IP's first-stage range, 5 agrees with Techlog
      and with IP's consolidated range. A range counts as agreement: if you typed 17 you did take
      IP's advice, and a record that only matched exact numbers would say you invented it.
- [ ] **Type something nobody ships — 9, say.** It must say it agrees with none of them, and still
      run. This is a record, not a gate.
- [ ] **Confirm Geolog is never quoted as endorsing anything**, whatever you type. A vendor that
      ships no default cannot be cited as approving your choice.

What is deliberately NOT here: no vendor algorithm, table or wording. A shipped default is one
documented fact about a product, cited to the page documenting it. If an entry needed a lookup table
to make sense, it would be the wrong entry.

## The leaderboard's score and its plus-or-minus are now the same number (2026-08-07)

Found by running the ML pane against a real five-well delivery — predicting bulk density from GR,
TVDSS and deep resistivity — and then reproducing the result in scikit-learn outside SandiBumi to
see whether the numbers held up. They did not, in a specific and correctable way.

The leaderboard was computing its headline score as **one R² over every out-of-fold row at once**,
and its `±` as **the spread of the per-well fold scores**. Two different statistics, printed as one
figure. So a row reading `0.327 ± 0.094` was telling you that a typical well scores about 0.33 when
a typical well actually scored **0.216**. The pooled number is higher because the wells sit at
different density levels, and the contrast *between* wells lands in its denominator as variance the
model gets credit for explaining — even though no well's own detail was predicted any better. On
five wells the flattery was 0.11 R². It grows with how different your wells are, which is to say it
grows exactly where a field study needs the number most.

Two things it broke that were not obvious. The **tie rule** — which greys out a winner the fold
spread cannot separate — was comparing a pooled centre against fold spreads, so it was measuring a
gap in one currency against a ruler in another. And the leaderboard disagreed with the **training
run**: the run reports `r2_cv` as the mean of its folds, so a model you picked at 0.33 would report
about 0.22 the moment you actually ran it, and nothing said why.

Both scores still ship. Neither is wrong — they answer different questions — so the table now has
two labelled columns and says which is which.

- [ ] **Run a Compare (leaderboard) over 3+ wells.** The score column is now headed **R² (per well)**
      with a separate **R² (pooled)** column beside it. Read the sentence under the table: it should
      say the first answers *what will the next well score* and the second *how good is the
      field-wide curve*.
- [ ] **Check the pooled column is the higher of the two** on wells that differ in character. If
      they are nearly equal your wells are alike, which is itself worth knowing.
- [ ] **Note that ranking follows the per-well column**, not pooled. A model that merely spreads the
      field's own contrast should no longer float to the top.
- [ ] **Now the cross-check that matters.** Pick the top model, run it for real with a blind split,
      and compare the run's `r2_cv` against the leaderboard's per-well score. **They should be close.**
      Before this they differed by about 0.11 with no explanation offered.
- [ ] **Look at the score chart's whiskers.** They are drawn as score ± spread, so they were
      previously centred on a number the spread did not describe. They should now sit around the
      per-well score.
- [ ] **If two rows are greyed as tied**, that judgement is now made in one currency. Worth a second
      look at any run where the tie call previously seemed off.

Verified by breaking it: with the old conflated line restored, the new test reports **0.985** where
the honest per-well answer on the same fixture is poor.

## The blind draw is judged on its whole shape, not on its mean (2026-08-07)

Jauhar, on reading the blind-vs-train gap: *"the lottery blind not only cover same single
statistic, but should cover all, such p10 and p90, std, mean, modus, and skewness"*. He is right,
and the old table could not have caught it.

"How alike the two sides are" compared **mean and standard deviation only**. Two sets can agree
exactly on both and still be completely different rock — a unimodal clean sand against a bimodal
sand-shale pair, or two sets differing entirely in which tail is long. When that happens the blind
score is a statement about a population the model was never fitted to, and nothing downstream can
tell, because the score is one number.

It matters most where it is easiest to miss. A whole-well hold-out on a handful of wells is a
lottery: on a real five-well set, the ten possible two-well draws spanned **0.64 R²**, from +0.32 to
−0.32, with the same model and the same data. This table is the only place you can see which ticket
you drew.

So it now reports **n, mean, sd, P10, P50, P90, mode and skew** for both sides, two rows per curve —
fitted above, blind below — and flags a curve on **any** of them, not on the mean alone.

Three things worth knowing about how it is computed. The percentiles come from `distribution.rs`,
the same shared statistics core every other percentile in SandiBumi uses, so they agree with the
histogram panel and the Field Dashboard by construction rather than by a third implementation being
kept in step by hand. The **mode** has no meaning on continuous data without a binning, so both
sides are histogrammed over their **combined** range at one resolution — a mode read off two
different binnings would not be a comparison. **Skew** is Fisher-Pearson g1, the same number
`scipy.stats.skew` returns, so it can be checked.

It also fixes something small that had been there all along: the curves are **named** now. The
runner is only told the feature names when it is saving a model, so the table used to read
`x0`, `x1`, `x2`.

- [ ] **Run a model with a blind split** and open the split box. The balance table should have eight
      statistics and two rows per curve, and the curves should be named — GR, RHOB, TVDSS — not x0.
- [ ] **Check a curve you know is skewed** (deep resistivity is the usual one). Its fitted and blind
      skew are unlikely to match, and that is the finding.
- [ ] **Read the sentence under the table.** It should name the worst disagreement and say **which
      statistic on which curve** produced it — not just "the two sides differ".
- [ ] **The important case:** a curve flagged in red whose *means* look almost identical. That is
      exactly what the old table passed as representative, and the reason for the change.
- [ ] **Re-run with a different split seed** and watch the table change. If it changes a lot, your
      blind score is a lottery ticket and should be quoted with that in mind — prefer the
      leaderboard's leave-one-well-out, which uses every well as the test set once.
- [ ] **Narrow the pane.** The balance table scrolls sideways in its own box below about 620px; the
      pane must not stretch and the window must never scroll sideways.

Known and NOT fixed here: the three-score table above it (train / CV / blind) still overflows a pane
narrower than about 440px. Pre-existing, unrelated to this change, and worth a separate look.

## A model that gave up no longer looks like a model that lost (2026-08-07)

Three things found by running the ML pane against real wells, now fixed.

### The optimiser giving up was invisible

On the real five-well run the neural net returned **R² of −50.9** and sat in the leaderboard looking
like a candidate that had simply done badly. It had not done badly — it had never finished
training. `MLPRegressor` stops at `max_iter` whether or not it has converged, and scikit-learn says
so loudly through a `ConvergenceWarning`. SandiBumi was throwing that away: warnings go to stderr,
and the runner reads only the last stderr line, as the error.

A half-trained model and a poor model are indistinguishable from the score, and they call for
opposite responses — one needs more iterations, the other needs different inputs.

- [ ] **Run ML Models with the neural net (ANN) and a low max_iter** — 50 will do it. The run should
      now tell you it did not converge, quote scikit-learn's own wording, and suggest raising
      max_iter or standardising the inputs.
- [ ] **Raise max_iter until the message stops.** That is the point at which the score starts
      describing the model your settings actually asked for.
- [ ] **Run a Compare with ANN among the candidates.** Its row should be greyed with an inline
      **⚠ did not converge**, and hovering it should say in how many folds. It keeps its place in
      the table — the fit did produce something — but it must not read as an ordinary poor result.
- [ ] **Check the other rows are untouched.** Only a model that actually hit its limit is marked.

### "Set" means two different things, and picking the wrong one was silent

Choosing **FPROOH** as the input set produced *"5 of 5 training wells have no log set named
'FPROOH'"* — correct, and useless: import sets and log sets are different stores. A **log set** is a
version of an interpretation (RAW / EDIT / FINAL, written by a module run). An **import set** is a
delivery of measured curves, named in the LAS wizard. The run read exactly the right rows anyway,
because import sets resolve by mnemonic — so the note looked like a problem and was not one.

The message now explains that, **but only when no well matched at all**. A version genuinely missing
from some wells is patchy across a field — some were re-run, some were not — and telling that user
they picked the wrong *kind* of set would be a confident wrong answer.

- [ ] **Pick an input set that is the name of a LAS delivery** rather than an interpretation version.
      The note should explain the difference and say the rows you got are the ones you wanted.
- [ ] **Now run a module on some wells but not others, then use that log set.** The note must
      name the wells and must NOT claim you chose the wrong kind of set.

### The curves in the balance table are named

Covered in the section above; it was the same root cause — the runner is only told the feature
names when it is saving a model.

## An input curve can be logged before the fit, and the model remembers (2026-08-08)

Deep resistivity spans decades. A fit on the raw column is carried by its largest few values, and
standardising does **not** fix that — a z-score recentres a skewed variable, it does not unskew it.
Both incumbents let you log an input before a fit; SandiBumi could not. Now it can, per input curve,
in the Model section under **Input transform**: as measured / log₁₀ / ln / √.

**The important part is not the log. It is that the model remembers it.** A model fitted on
log₁₀(RT) and later applied to raw RT returns numbers that are in range, confident and wrong — the
scaler absorbs none of it, and there is nothing downstream that could catch it. So the transform
rides inside the saved model file alongside the scaler, exactly as the curve ORDER already does. On
apply it is read from the model and applied by the model; a caller that states a different one is
refused by name rather than quietly overridden.

Three decisions worth knowing:

**A value the transform cannot represent becomes missing.** A zero or negative resistivity under a
log is dropped, never nudged by a small number nobody chose and never clamped to a floor. Those are
already invalid measurements, and inventing a value for them would put a fabricated number into the
fit. The count is reported per curve, because losing rows silently is how a training set shrinks
without anyone noticing.

**Nothing is ever suggested.** Every curve defaults to *as measured*. Whether a resistivity should
be logged is your call about your own data, and a default log would change every existing run's
answer without saying so.

**The list is deliberately short.** Each of the four is a transform a petrophysicist already applies
by hand to these curves. A free-text expression box here would be a second equation engine sitting
where the real one already lives — and a curve transformed by an arbitrary expression could not be
re-applied from a saved model with any confidence.

- [ ] **Set log₁₀ on a deep resistivity and run a regression.** Compare the blind score against the
      same run with everything *as measured*. On the wells I tested log₁₀ actually scored **worse**,
      which is a real answer about that data — the point is that you can now find out.
- [ ] **Check the run notes** for how many samples the transform dropped, if any. Zero or negative
      resistivities are the usual cause.
- [ ] **Save a model fitted with a transform**, then apply it to other wells. It must use the same
      transform without being told — you should not have to set anything on the apply.
- [ ] **The one that matters:** confirm the applied curve is sensible on a well that was not in
      training. A model that had silently dropped its log would still produce a smooth, plausible
      curve — which is exactly why this is enforced inside the model file rather than around it.
- [ ] **Try a transform on a curve, then untick that curve from the inputs and run.** It should
      refuse and name the curve rather than quietly ignoring the setting.
- [ ] **Leave everything as measured and run.** The result must be identical to before this existed.

## A run that covered part of the field now says so, and the fit has a ceiling (2026-08-08)

Two halves of SB-MLA-065, both about the same thing: a batch run reports its outcome in a panel
that is closed by Monday, while the curve it wrote is still there.

### The fit could run forever, and now you set the limit

`SVR` and `SVC` were built without `max_iter`, and scikit-learn's default there is **-1 — no
limit**. Those two get slow very fast as the pooled sample count grows, and the fit is the one
phase of a run with no progress and no working Cancel: the app just looks frozen, and you cannot
tell "working hard" from "stuck". Both now take a **max iterations** setting.

**It is an iteration count, not a stopwatch, and that was your call** — *"everything we can do and
report in sandibumi, it should be re-producible"*. Stopping after 500 iterations gives the same
model on every machine. Stopping after ten minutes gives a different model on a faster laptop, and
a curve nobody else could reproduce.

**The default stays -1, so nothing you have run before changes its answer.** What changes is that
the setting exists, says what -1 costs, and is recorded with the run. When a finite limit is hit,
scikit-learn raises the same `ConvergenceWarning` that now produces the *"this fit did not
converge"* message — so the reporting was already built.

- [ ] **Set max iterations on Support Vector Regression** — try something small like 200 on a
      decent-sized run. You should get the "did not converge" message, naming the limit.
- [ ] **Leave it at -1 and confirm nothing changed** versus a run you made before today.
- [ ] **Check the run's recorded parameters** show the value you set, so the run can be repeated.

### A run that skipped wells no longer looks complete

A cancelled run was already marked. The gap was the run that **finishes normally**: you run 80
wells, 12 have no usable samples, the run succeeds and tells you so — and leaves 68 log sets that
are indistinguishable from a complete run over a smaller well selection, because the set name and
the module string are the ones a complete run writes.

Those sets are now stamped as covering part of the field, with the counts and a plain-language
note, in the same place the cancelled mark lives — **not** a second mechanism, because two places
recording "this set does not cover the field" is one place that eventually stops being updated.

The two marks stay distinguishable on purpose: *cancelled* and *some wells had no data* call for
opposite responses — re-run it, versus go and look at those wells.

- [ ] **Run ML over a well selection where some wells lack an input curve.** The run should succeed,
      and the note should say how many wells produced nothing.
- [ ] **Open the Curve Catalog on a well that DID get the curve.** Its set should carry the mark
      saying the field is covered in part.
- [ ] **Run over wells that all have the inputs.** No mark at all — if a complete run got stamped,
      the mark would stop meaning anything.
- [ ] **Cancel a run part-way** and confirm you get the *cancelled* wording, not this one.

### Not done

SB-MLA-065 also mentioned a wall-clock backstop for algorithms with no iteration count. Deferred
deliberately: a default that refuses would change what your existing runs do, and a default that
does not refuse adds nothing — so the cap would have to be a number you choose, which is its own
decision.

## One Ward criterion instead of two copies of it (2026-08-08)

The Ward minimum-variance criterion — the thing that decides where one flow unit ends and the next
begins — existed **twice**, once in the HFU tool and once in the Lorenz tool. Not two similar
routines: the same dynamic program, line for line, differing only in what each handed back. Each
also carried its own copy of the backtracking step that turns the table into cluster numbers.

Two copies of one criterion is two places for it to drift, and the drift would be **silent** — both
would go on producing a plausible-looking partition, and nothing would report that the two tools had
started disagreeing about the same arithmetic.

There is now one implementation, in the shared statistics core beside the percentiles and the
histogram, and both tools call it.

### The part that is not shared, and should not be

What genuinely differs between the two is **the order the values are put in before the criterion
runs**, and that changes the geological question completely:

- **HFU** sorts FZI **by value**. A cluster is then a rock TYPE — plugs from anywhere in the well
  with similar flow character group together.
- **Lorenz** keeps **depth order**. A cluster is then an interval of HOLE — a flow unit with a top
  and a base.
- The ML pane's agglomerative clustering has **no ordering constraint** at all.

Same criterion, three questions. So the ordering is now *declared* by whichever tool is calling,
travels with the result, and is named — `ward:sorted-value`, `ward:depth-contiguous`, `ward:free`.
The HFU result says which one produced it, in words.

- [ ] **Run HFU with the Ward method** on a core set you have run before, and confirm the units come
      out the same as they used to. This was a refactor — the numbers must not have moved.
- [ ] **Read the note on that result.** It should name `ward:sorted-value` and say that a unit here
      is a rock type, not an interval of hole.
- [ ] **Run the Lorenz flow-unit segmentation** on a well you have run before. Same check: the unit
      boundaries must be identical to what you had.
- [ ] **The thing worth understanding rather than clicking:** the two tools now share the same
      arithmetic and still give different answers on the same well, because one sorts by value and
      the other keeps depth order. That difference is real and intended.

## A clustering method that picks its own number of facies (2026-08-08)

This is the answer to the MRGC question rather than a new idea. MRGC's real selling point is that it
finds the natural number of electrofacies and copes with clusters of very different size and
density — a thin coal beside a thick sand. Everything else about it we already had, and its
published description is not complete enough to implement without guessing.

**HDBSCAN does the same job from a fully public method**, ships inside scikit-learn, and can be
named in a report with no argument about where it came from. It is now in the Clustering list.

What it does differently from the three already there:

- **It is not told how many facies to find.** K-means, GMM and hierarchical all need K up front, and
  the answer changes with it. HDBSCAN reads it off the data and tells you what it found. **The K
  setting is ignored, and the run says so** rather than leaving a number you typed sitting there
  looking honoured.
- **It tolerates thin units beside thick ones.** K-means minimises within-cluster spread, so given
  one more cluster than the rock has it splits the big group rather than finding the small one.
  DBSCAN needs a single density setting to suit both at once, and no value does.
- **It refuses samples rather than forcing them.** Like DBSCAN, a sample it will not assign is
  written as rejected — a finding about that rock, distinct from "never evaluated".

The setting that does the work is **min samples per facies**. It is a sample count, which at a
half-foot sampling makes it a thickness question: 25 samples is about twelve feet. The smallest unit
worth naming in your field is your call, so nothing infers it.

**One thing to read carefully.** HDBSCAN also writes a `_PROB` curve, and it is **not** the same
quantity as the GMM one. GMM's is a posterior across components — "how sure am I this is component
3 rather than 4". HDBSCAN's is a membership **strength** within the cluster it already chose — 1.0
at the dense core of a unit, falling toward 0 at its edge, and it says nothing about which other
cluster the sample might belong to. Read the second as the first and a facies track looks uncertain
exactly where it is most certain. The run states which one it produced.

- [ ] **Run Clustering → HDBSCAN** on a well with an obvious thin bed. Check the reported cluster
      count against what you would have picked, and whether the thin bed came out as its own facies.
- [ ] **Compare against k-means at the same K** it chose. The interesting case is where they differ.
- [ ] **Check the rejected percentage.** A track that is 40% rejected is telling you the min-samples
      setting is too high for this well, or that standardisation is off.
- [ ] **Turn standardisation off and re-run.** You should get a warning that the density estimate is
      then in the curves' own mixed units and the largest-range curve dominates.
- [ ] **Read the `_PROB` curve's stated meaning** in the run result before using it.

## Predicting a missing log into wells that never had it (k-NN propagation)

This is the leg you pointed at: PEF exists in a handful of wells, GR and RHOB exist everywhere,
and you want a PEF in the wells that never ran the tool.

**What was actually missing.** `log_predict` could never do this. It builds its training set from
the *same well's* samples where the target is present, so it fills PEF gaps in a well that already
has PEF — it cannot predict PEF in a well that has none. It is a gap-filler that reads as a
predictor. The ML suite could do the cross-well job, but only with tree ensembles, and an ensemble
predicts an average of averages: it regresses toward the mean and flattens exactly the contrast a
synthetic curve is made to recover.

**k-NN is now in the regression list.** It returns a blend of *measured* target values from rock
with similar inputs, so the values it writes are values the rock actually had. Train it on the wells
that have PEF, save the model, apply it to the wells that don't.

**Three curves come with it, and they are the point.** Beside the prediction you get:

- `_MIN` and `_MAX` — the smallest and largest **measured** target value among the k nearest rocks.
  This is not a confidence interval and not the prediction plus-or-minus anything. It says "the
  closest k rocks we have measured had PEF between 3.1 and 4.8", so it is lopsided wherever the
  neighbourhood is lopsided, which is the honest picture.
- `_DIST` — the mean distance to those neighbours in standardised space. **This is the one the
  product never had.** Near zero means the training set contains rock like this and the value is an
  interpolation between things somebody measured. Large means nothing in the training set looks like
  this, and the prediction there is an extrapolation the model has no basis for. Every predicted
  curve we have ever written was quotable with no such statement attached.

The run also reports `knn_dist_train_p50` and `p90` — what the *fitted* rock's own neighbour
distances looked like. That is the scale to read `_DIST` against, because 0.8 is unremarkable in one
feature set and off the end of the world in another.

- [ ] **Train on the wells that have PEF, apply to a well that doesn't.** Confirm a PEF curve
      appears where there was none, and that it is not a flat line.
- [ ] **Put `_DIST` on the log view beside the prediction.** Find an interval where it spikes and
      check what that rock is — that is the model telling you it is guessing there.
- [ ] **Compare `_DIST` against the reported p90.** Anything well above it should be treated as
      uninterpreted rather than as a prediction.
- [ ] **Shade between `_MIN` and `_MAX`** (crossover fill) and read the width. A wide band over a
      zone means the neighbours disagreed, which is a different problem from being far away.
- [ ] **Set neighbours to 1 and re-run.** The band should collapse onto the prediction exactly —
      that is what proves the band is the neighbours' own values and not a fitted interval.
- [ ] **Compare the k-NN curve against the same run with Random Forest.** The RF version should be
      visibly smoother; decide for yourself which one you would defend in a report.
- [ ] **Save the model and re-apply it in a later session.** The band and distance must come back
      identically — the fitted targets travel inside the artifact.

## The cluster table — reading what a clustering run actually found

**"Cluster", not "facies", from here on.** A cluster becomes a facies only once you have merged and
named it. Before that it is a group of samples, and the same model is as likely to be feeding a
propagated curve as a lithology track — calling the object "facies" is what made this look like an
electrofacies feature and kept it from being built.

**The numbers already existed and none of them were readable.** `cluster_sizes` was emitted by the
runner and rendered nowhere except as a JSON blob in the generic metrics rows; the per-cluster means
were computed only to order the labels, then thrown away. So a clustering run told you how good the
separation was (silhouette) without telling you what it had separated.

A clustering run now shows one row per cluster: a colour swatch matching the log track and the
crossplot, the sample count with a share bar behind it, and then one column per input curve. Each
cell is the cluster's **mean**, with its **P10–P90** beneath.

**The range is there because a column of means cannot support the decision you make from this
table.** Two clusters with the same mean and no overlap are two rocks that happen to average alike.
Two with the same mean and overlapping ranges are one rock split in half. A mean alone makes those
identical, and merging is exactly the choice between them.

Built in Rust rather than in the Python runner, so it uses the same percentile definition as every
histogram and box plot in the product, and so the columns carry your curve names — the runner is
only told those when a model is being saved, which for clustering never happens.

- [ ] **Run any clustering and read the table.** The ids run from one end of the first curve to the
      other; confirm that ordering makes sense for the curve you put first.
- [ ] **Look for two clusters with similar means.** Check their P10–P90. Overlapping ranges are your
      merge candidates; separated ranges are two real rocks that happen to average alike.
- [ ] **Check the numbers are in the curve's own units** — GR should read like GR, not like a
      z-score. Cross-check one against a histogram of the same interval.
- [ ] **Deliberately over-cluster** (say k = 20 on a well you know) and see whether the table makes
      the merge decisions obvious. That is the workflow this is meant to support.
- [ ] **Run DBSCAN or HDBSCAN** and confirm the rejected count is stated below the caption and that
      rejected samples are excluded from every column rather than dragging a cluster's mean.
- [ ] **Compare a swatch colour against the FACIES track** in a log view — they should be the same
      colour for the same id.

## What is each curve worth? (ranked predictor combinations)

Compare ▸ **Every combination (what is each curve worth?)**. The leaderboard already scored
algorithm × curve-subset combinations; what it never did was enumerate them, or answer the question
you would actually ask before the next well: **which logs do I need to run?**

**The answer is best-with minus best-without.** For each curve: the best score reachable by any
combination that includes it, minus the best reachable by any combination without it. A curve near
zero is one the others already cover for — you can stop running that tool.

**This is deliberately not permutation importance**, which is what the leaderboard already showed.
Importance asks how much *one model* leans on a curve, and that understates any curve with a
stand-in: drop RHOB from a run that also has DT and a tree ensemble simply leans on DT, so RHOB
reads unimportant while the field would genuinely lose nothing by not logging it. Same conclusion,
but arrived at by re-fitting without the curve rather than by shuffling it inside one model.

**Two readings that are easy to confuse and must not be.** A curve showing `+0.000` was scored
without and added nothing. A curve showing `—` was in *every* combination scored, so its value was
never measured at all. Those are opposite findings and they would send a logging decision in
opposite directions, so the second is never printed as a zero.

Scored on the same whole-well GroupKFold as the rest of the leaderboard — held-out **wells**, not
held-out rows — so the number answers "what will the next well score", which is the question a
logging decision turns on.

There are 2^n − 1 combinations, so a cap is unavoidable. They are enumerated **largest first**, so
what a cap drops is always the smallest combinations, and the full set plus every drop-one set are
always scored first. The run says how many it reached and whether the per-curve answer is complete.

- [ ] **Run it on a target you already understand** (say PEF from GR/RHOB/NPHI/DT). Check the
      ranking against your own expectation before trusting it on something you don't.
- [ ] **Find a curve worth ~0.00** and confirm from the leaderboard that dropping it really does
      leave the top score unchanged.
- [ ] **Watch for a negative worth.** A curve that makes the best model *worse* is a real result —
      usually noise, a bad splice, or a curve that is missing over half the interval.
- [ ] **Check the `—` rows.** With few curves everything gets dropped at least once; with many, the
      cap may not reach that far, and the note says so.
- [ ] **Compare the ranking against the importance column** in the leaderboard below. Where they
      disagree, the disagreement is the interesting part — that is a curve with a stand-in.
- [ ] **Re-run with a different seed.** The ranking should be stable; if it is not, the differences
      between those curves are inside the noise and none of them is worth much.

## Input curves: numbered slots instead of a checkbox grid

The grid was unusable on a real delivery — sixty-odd curves in two columns with the four you wanted
scattered through it. It is now **Input log 1 / Input log 2 / + input log**, each a dropdown, with a
× to remove one.

**It was also wrong, and that is the half worth knowing.** The old help text said "class 0 = lowest
mean of the FIRST checked curve (put GR first)" — and there was no way to do it. The order came from
the catalog's own listing, not from the order you ticked. So the one instruction that field carried
could not be followed, and anyone who believed it got their clusters numbered by whichever curve the
catalog happened to list first.

The slot number is now the column index the model actually sees. That matters in three places: for
clustering, class 0 is the lowest mean of Input log 1; a saved model records the order and refuses a
run that reorders it; and the curve-worth table reads the same list.

A curve already taken by another slot is greyed out in the rest, so the same log cannot be entered
twice. Removing the last slot leaves one blank rather than an empty form.

- [ ] **Open ML on a well with many imported curves** and confirm the pane is now readable.
- [ ] **Put GR in slot 1, run a clustering**, and check class 0 really is the cleanest rock. Then
      move GR to slot 2 and confirm the numbering changes — that is the contract being visible.
- [ ] **Try to pick the same curve twice.** It should be greyed out in the other slot.
- [ ] **Remove every slot.** One blank row should remain rather than an empty form.
- [ ] **Check Data QC, the fixed-limits table and the transform picker** all follow the slots — they
      are per input curve and should update as you change one.
- [ ] **Save a model, then reorder the slots and apply it.** It must refuse rather than predict.

## The ML sub-tabs actually switch now

You reported the sub-panes stacked all on the first one and the tabs did nothing. They did — and the
switching code was never the problem.

`.ml-section[hidden] { display: none }` was there, so the guard was not missing. It just **lost**:
`.dock-ml .ml-section { display: grid }` has the same specificity and sits later in the stylesheet,
so it won the tie. Every section then took `display: grid` whatever its `hidden` attribute said, all
five stacked under Input, and clicking a tab set an attribute that changed nothing. It only failed
in the docked pane, which is why it looked fine anywhere else.

The pairing is now repeated beside the rule that outranks it, with a note that any future `display`
rule in that block needs the same line — a guard 1,800 lines away is one nobody editing that block
will think about.

- [ ] **Click every sub-tab.** Exactly one section at a time, and the content should change.
- [ ] **Widen the pane past ~1060px.** The active section should go to two columns; prose, tables
      and charts still span the full width.
- [ ] **Check the dock tabs** (Database Inspector / Machine Learning) switch again. With five
      sections stacked the pane was enormously tall, which is the likely cause — but that is a
      hypothesis, so if they still misbehave say so and it gets its own look.

## One well picker, and the Input tab in reading order

**The Train wells grid is gone.** RUN ON is now the only well selector, and the run both fits and
predicts on it. On a sixteen-well field the two lists were the same names twice with no visible rule
for which governed what.

It was also almost never a real choice. A well contributes training rows only where the **target
curve exists**, so listing a well with no target as a training well already did nothing; and
listing one with a target but leaving it out of RUN ON meant fitting on rock you then refused to
predict. The genuine "fit here, predict there" case is the **saved model** — fit on the wells that
carry the curve, then propagate that artifact from Model Distribution, which is the route you
described.

The one thing this gives up: holding back a well that *has* the target without also giving up its
prediction. That is what the blind split is for, and it already holds out whole wells.

**The Input tab now reads in the order you asked:**

`Input log set → Input curves → Output log set → Output curve → Target curve → Mask → Interval → RUN ON`

The log-set pickers used to sit at the bottom of the tab, a screen away from the curves they govern
— which is the "set and curve feel disconnected" complaint. Which set you read from decides which
version of PHIE the model sees, so it belongs directly above the curve list. Output curve moved over
from the Model tab to sit beside the output set: where it lands and what it is called are one step.

- [ ] **Run a supervised fit** with wells in scope that have no target curve. Confirm it trains on
      the ones that do and still predicts for all of them.
- [ ] **Check the blind-split wording** follows the RUN ON count as you change scope.
- [ ] **Run a clustering.** RUN ON must still be visible — only Target curve hides for an
      unsupervised task.
- [ ] **Check the Model tab** no longer has Output curve, and does not look gappy without it.
- [ ] **Confirm the reading order** matches the list above on a narrow pane and on a wide one (the
      Input tab goes two-column past ~1060px).
