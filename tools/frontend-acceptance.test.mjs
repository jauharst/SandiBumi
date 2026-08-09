import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import { createServer } from "vite";

let server;

before(async () => {
  server = await createServer({
    server: { middlewareMode: true },
    appType: "custom",
    logLevel: "silent",
  });
});

after(async () => {
  await server?.close();
});

async function load(path) {
  return server.ssrLoadModule(path);
}

test("characterizes_finite_statistics_without_population_or_exclusion_metadata", async () => {
  // CHARACTERIZATION — SB-PLT-009 / SB-PLT-T12 supplies the arithmetic fixture and
  // expected count=3, mean=2 and P50=2. The absent disclosure fields describe the
  // current PARTIAL result shape; they are not claimed as the specified final contract.
  const { basicStats } = await load("/src/ui/plotCanvas.ts");
  const stats = basicStats(Float32Array.of(1, 2, 3, Number.NaN, Number.POSITIVE_INFINITY));

  assert.equal(stats.count, 3);
  assert.equal(stats.mean, 2);
  assert.equal(stats.p50, 2);
  assert.deepEqual(
    Object.keys(stats).sort(),
    ["count", "max", "mean", "min", "p5", "p50", "p95", "std"],
    "today's summary has no population, interval, selection, exclusion, percentile-method, or std-estimator metadata",
  );
});

test("characterizes_regression_as_coefficients_without_a_versioned_scientific_record", async () => {
  // CHARACTERIZATION — SB-PLT-010 / SB-PLT-T14 cites the arithmetic fixture
  // y=2+3x for x=1..5. The four-key result is the current PARTIAL payload, not proof
  // of the required model/version/space/exclusion/interval/well/revision record.
  const { fitRegression } = await load("/src/ui/crossplotPanel.ts");
  const fit = fitRegression([1, 2, 3, 4, 5], [5, 8, 11, 14, 17], "linear", "yx");

  assert.ok(fit);
  assert.equal(fit.a, 2);
  assert.equal(fit.b, 3);
  assert.equal(fit.r2, 1);
  assert.equal(fit.n, 5);
  assert.deepEqual(Object.keys(fit).sort(), ["a", "b", "n", "r2"]);
});

test("characterizes_linked_brushing_as_one_ephemeral_scope_with_exact_depth_membership", async () => {
  // CHARACTERIZATION — SB-PLT-018 / SB-PLT-T29 requires two named persistent selections.
  // The replacement and two-field shape below deliberately describe today's single
  // in-memory BrushSelection; they are not presented as the specified coexistence contract.
  const { appState, clearBrush, setBrushedDepths } = await load("/src/state.ts");
  clearBrush();

  setBrushedDepths("scope-a", new Set([100, 100.5]));
  assert.deepEqual([...appState.brushedDepths.get().depths], [100, 100.5]);
  setBrushedDepths("scope-b", new Set([200]));

  const current = appState.brushedDepths.get();
  assert.equal(current.wellId, "scope-b");
  assert.deepEqual([...current.depths], [200], "the second selection replaces the first");
  assert.deepEqual(Object.keys(current).sort(), ["depths", "wellId"]);
  clearBrush();
  assert.equal(appState.brushedDepths.get(), null);
});

test("an_unknown_future_template_field_survives_crossplot_option_normalization", async () => {
  // CORRECTNESS — SB-PLT-025 / SB-PLT-T36 specifies that an unknown future field is
  // preserved or migration refuses; this fixture pins the shipped preservation route.
  const { normalizeCrossplotOptions } = await load("/src/ui/crossplotPanel.ts");
  const future = { schema: 7, semanticBinding: "future-axis-contract" };
  const normalized = normalizeCrossplotOptions({ future_template_field: future });

  assert.deepEqual(normalized.future_template_field, future);
});

test("characterizes_vector_exports_as_labelled_while_the_png_print_path_is_not_labelled_raster", async () => {
  // CHARACTERIZATION — SB-PLT-026 / SB-PLT-T37/T38 requires labelled vector exports
  // and a raster-labelled print route. The unqualified Print label is today's PARTIAL
  // behaviour and must not be mistaken for completion of the export contract.
  const { imageExportMenuEntries, printCanvas } = await load("/src/ui/plotExport.ts");
  const entries = imageExportMenuEntries(
    () => null,
    "plot",
    () => {},
    () => "<svg/>",
    () => ({ width: 1, height: 1, commands: [] }),
  );
  const labels = entries.map((entry) => entry.label);

  assert.ok(labels.includes("Export SVG (vector)…"));
  assert.ok(labels.includes("Export PDF (vector)…"));
  assert.ok(labels.includes("Print…"));
  assert.ok(!labels.some((label) => /raster/i.test(label)));
  assert.match(Function.prototype.toString.call(printCanvas), /image\/png/);
});

test("characterizes_crossplot_static_draw_and_z_colours_as_separately_invalidated_subsets", async () => {
  // CHARACTERIZATION — SB-PLT-028 cites the current crossplot source. These memo and
  // draw boundaries are the shipped PARTIAL subset; the test does not claim that every
  // range, quantile, transform, invariant overlay and transient layer is separated.
  const { buildCrossplotContent } = await load("/src/ui/crossplotPanel.ts");
  const source = Function.prototype.toString.call(buildCrossplotContent);

  assert.match(source, /colorMemo/);
  assert.match(source, /dataGen/);
  assert.match(source, /drawStatic/);
  assert.ok(source.indexOf("drawStatic") < source.lastIndexOf("redraw"));
  assert.match(source, /plot\s*=\s*drawStatic\(canvas,\s*hoverIdx\)/);
});

test("a_superseded_async_plot_build_is_disposed_before_it_can_replace_the_active_panel", async () => {
  // CORRECTNESS — SB-PLT-029 / SB-PLT-T28/T33 cites workspace.ts's generation
  // contract: reverse-order completion may render only the newest generation.
  const { Workspace } = await load("/src/ui/workspace.ts");
  const source = Function.prototype.toString.call(Workspace.prototype.createPlot);
  const generationCheck = source.indexOf("gen !== generation");
  const staleDispose = source.indexOf("content.dispose", generationCheck);
  const activeAppend = source.indexOf("host.appendChild(content.el)");

  assert.ok(source.includes("const gen = ++generation"));
  assert.ok(generationCheck >= 0, "resolved content is guarded by its generation");
  assert.ok(staleDispose > generationCheck, "stale content is disposed inside the guard");
  assert.ok(activeAppend > staleDispose, "the stale-return branch precedes active-panel mutation");
});
