// SB-PLT-024 route 2 (DEC-078): a derived overlay may only replace digitized coordinates
// when the published equation, fed the cited constants, reproduces the digitized chart
// within the tolerance of digitizing a printed page. This suite is that proof, plus the
// freshness gate on the generated module — the same two-sided discipline as the
// third-party-licence inventory.
import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { CONSTANTS, GD, LITH2, RHG, derivedOverlays, renderModule } from "./gen-derived-overlays.mjs";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const DT_TOL_US_FT = 0.4; // digitization tolerance on a 40-130 us/ft printed axis
const RHO_TOL_G_CC = 0.005; // the density graduations are exact arithmetic; tolerance is slack

/** Extract one digitized overlay's curves from the generated chartOverlays.ts source. */
function digitizedCurves(id) {
  const source = fs.readFileSync(path.join(repo, "src", "ui", "chartOverlays.ts"), "utf8");
  const start = source.indexOf(`id: "${id}"`);
  assert.ok(start >= 0, `digitized overlay ${id} not found`);
  const end = source.indexOf("id: \"", start + 10);
  const block = source.slice(start, end < 0 ? undefined : end);
  const curves = [];
  const curveRe = /\{ name: "([^"]+)", labelEvery: \d+, grads: \[((?:\[[^\]]*\],?)+)\] \}/g;
  for (const match of block.matchAll(curveRe)) {
    const grads = [...match[2].matchAll(/\[([^\]]*)\]/g)].map((g) =>
      g[1].split(",").map(Number),
    );
    curves.push({ name: match[1], grads });
  }
  assert.ok(curves.length > 0, `no digitized curves parsed for ${id}`);
  return curves;
}

test("the_derived_por22_time_average_curves_reproduce_the_digitized_chart_within_digitization_tolerance", () => {
  const derived = derivedOverlays().find((d) => d.id === "por22_ta");
  assert.ok(derived, "por22_ta must be a derived overlay");
  const digitized = digitizedCurves("por22_ta");
  assert.deepEqual(
    derived.curves.map((c) => c.name).sort(),
    digitized.map((c) => c.name).sort(),
    "the derivation must cover exactly the lithologies the printed chart draws",
  );
  for (const curve of derived.curves) {
    const twin = digitized.find((c) => c.name === curve.name);
    assert.ok(twin, `digitized twin missing for ${curve.name}`);
    assert.equal(
      curve.grads.length,
      twin.grads.length,
      `${curve.name}: the derivation must graduate exactly where the printed chart does`,
    );
    for (const [phi, dt, rho] of curve.grads) {
      const twinGrad = twin.grads.find((g) => g[0] === phi);
      assert.ok(twinGrad, `${curve.name}: digitized chart has no graduation at ${phi} p.u.`);
      assert.ok(
        Math.abs(dt - twinGrad[1]) <= DT_TOL_US_FT,
        `${curve.name} at ${phi} p.u.: derived dt ${dt} vs digitized ${twinGrad[1]} exceeds ${DT_TOL_US_FT} us/ft`,
      );
      assert.ok(
        Math.abs(rho - twinGrad[2]) <= RHO_TOL_G_CC,
        `${curve.name} at ${phi} p.u.: derived rho ${rho} vs digitized ${twinGrad[2]} exceeds ${RHO_TOL_G_CC} g/cc`,
      );
    }
  }
});

test("the_derived_por22_field_observation_curves_are_the_rhg_1980_algorithm_not_a_tracing_of_the_printed_curve", () => {
  // DEC-079: the printed FO set traces RHG 1980's hand-drawn empirical transform, which
  // the paper's own algorithms only approximate. Jauhar ruled the overlay ships the
  // PUBLISHED ALGORITHM at the paper's stated constants. This test pins both halves:
  // the curves ARE the algorithm (independent arithmetic below), they still track the
  // printed chart at low porosity where the paper says the algorithm duplicates it, and
  // the known high-porosity departure is ASSERTED so nobody quietly re-tunes the cited
  // constants to chase the tracing.
  const derived = derivedOverlays().find((d) => d.id === "por22_fo");
  assert.ok(derived, "por22_fo must be a derived overlay");
  const digitized = digitizedCurves("por22_fo");
  assert.deepEqual(
    derived.curves.map((c) => c.name).sort(),
    digitized.map((c) => c.name).sort(),
    "the derivation must cover exactly the lithologies the printed chart draws",
  );

  // The paper's constants, restated here independently of the generator.
  const PAPER = {
    "Quartz sandstone": { vma: 17850, rhoma: 2.65 },
    "Calcite (limestone)": { vma: 20500, rhoma: 2.71 },
    "Dolomite": { vma: 22750, rhoma: 2.87 },
  };
  const VF = 5300;
  const DTF = 1e6 / VF;
  // The algorithm, restated independently: quadratic low branch, fluid-suspension high
  // branch, dt-linear interpolation between the segment-end values across 37..47%.
  const dtRhg = (m, f) => {
    const low = (p) => 1e6 / ((1 - p) ** 2 * m.vma + p * VF);
    const susp = (p) => {
      const rho = m.rhoma - p * (m.rhoma - 1.0);
      const dtma = 1e6 / m.vma;
      return Math.sqrt(rho * p * DTF * DTF + (rho * (1 - p) * dtma * dtma) / m.rhoma);
    };
    if (f <= 0.37) return low(f);
    if (f >= 0.47) return susp(f);
    return ((0.47 - f) / 0.1) * low(0.37) + ((f - 0.37) / 0.1) * susp(0.47);
  };

  for (const curve of derived.curves) {
    const m = PAPER[curve.name];
    const twin = digitized.find((c) => c.name === curve.name);
    assert.ok(twin, `digitized twin missing for ${curve.name}`);
    assert.equal(
      curve.grads.length,
      twin.grads.length,
      `${curve.name}: the derivation must graduate exactly where the printed chart does`,
    );
    for (const [phi, dt, rho] of curve.grads) {
      const f = phi / 100;
      assert.ok(
        Math.abs(dt - dtRhg(m, f)) <= 0.0002,
        `${curve.name} at ${phi} p.u.: derived dt ${dt} is not the RHG algorithm value ${dtRhg(m, f)}`,
      );
      assert.ok(
        Math.abs(rho - (m.rhoma - f * (m.rhoma - 1.0))) <= 0.0002,
        `${curve.name} at ${phi} p.u.: density graduation is not the chart arithmetic`,
      );
      const twinGrad = twin.grads.find((g) => g[0] === phi);
      assert.ok(twinGrad, `${curve.name}: printed chart has no graduation at ${phi} p.u.`);
      if (phi <= 25) {
        // The paper: the algorithms "reasonably duplicate the observed response" here.
        assert.ok(
          Math.abs(dt - twinGrad[1]) <= 1.2,
          `${curve.name} at ${phi} p.u.: derived dt ${dt} vs printed ${twinGrad[1]} exceeds the low-porosity duplication band`,
        );
      }
    }
  }

  // The ruled, documented divergence: at 40 p.u. the algorithm departs from the printed
  // tracing by far more than digitization tolerance. If this ever FAILS, someone tuned
  // the derivation toward the tracing — which DEC-079 forbids without a new ruling.
  const calDerived = derived.curves.find((c) => c.name === "Calcite (limestone)");
  const calDigit = digitized.find((c) => c.name === "Calcite (limestone)");
  const d40 = calDerived.grads.find((g) => g[0] === 40)[1];
  const p40 = calDigit.grads.find((g) => g[0] === 40)[1];
  assert.ok(
    Math.abs(d40 - p40) > 2,
    "the algorithm-vs-tracing gap at 40 p.u. has vanished - the derivation no longer ships the published algorithm",
  );
});

test("the_rhg_derivation_constants_are_exactly_the_paper_stated_values", () => {
  // Raymer, Hunt & Gardner 1980, Summary item 3 + the paper's figures (vf), DEC-079.
  assert.equal(RHG.VF_FT_S, 5300);
  assert.equal(RHG.LOW_END, 0.37);
  assert.equal(RHG.HIGH_START, 0.47);
  assert.deepEqual(
    RHG.LITHS.map((l) => [l.name, l.rhoMa, l.vMaFtS, l.maxPhi]),
    [
      ["Quartz sandstone", 2.65, 17850, 35],
      ["Calcite (limestone)", 2.71, 20500, 40],
      ["Dolomite", 2.87, 22750, 40],
    ],
  );
});

test("the_derivation_constants_are_exactly_the_cited_chart_page_values", () => {
  // tf and rho_f are STATED on Por-22 p. 238; vma values are Por-1 p. 212's printed fan;
  // matrix densities are the classical values the chart's own graduations use (DEC-078).
  assert.equal(CONSTANTS.TF_US_FT, 189.0);
  assert.equal(CONSTANTS.RHO_F, 1.0);
  assert.deepEqual(
    CONSTANTS.LITHS.map((l) => [l.name, l.rhoMa, l.vMaFtS]),
    [
      ["Quartz sandstone", 2.65, 18000],
      ["Calcite (limestone)", 2.71, 21000],
      ["Dolomite", 2.87, 23000],
    ],
  );
});

test("the_derived_lith_pef_curves_are_gardner_dumanoir_physics_and_sit_within_the_tld_tool_response_band", () => {
  // DEC-079: the Lith-3/4 Pe legs are the published Gardner & Dumanoir 1980 physics.
  // The printed charts are Platform Express TLD TOOL charts and carry a small
  // systematic tool-window slope, so the agreement bound here is the observed
  // tool-response band (0.08 Pe, ~1% of the 0-6 axis), not digitization tolerance;
  // the DENSITY leg has no tool component and must be exact chart arithmetic.
  const PE_BAND = 0.08;
  // The paper's table and the Lith-5 legend, restated independently of the generator.
  const MATS = {
    "Quartz sandstone": { U: 4.78, rb: 2.64, rhomaGrad: 2.65 },
    "Calcite (limestone)": { U: 13.8, rb: 2.71, rhomaGrad: 2.71 },
    "Dolomite": { U: 9.0, rb: 2.88, rhomaGrad: 2.87 },
  };
  const FLUIDS = {
    lith3: { Uf: 0.398, rb: 1.0, rhofGrad: 1.0 },
    lith4: { Uf: 1.36, rb: 1.11, rhofGrad: 1.1 },
  };
  const rhoe = (rb) => (rb + 0.1883) / 1.0704; // G&D eq (5) inverse

  for (const id of ["lith3", "lith4"]) {
    const fl = FLUIDS[id];
    const derived = derivedOverlays().find((d) => d.id === id);
    assert.ok(derived, `${id} must be a derived overlay`);
    const digitized = digitizedCurves(id);
    assert.deepEqual(
      derived.curves.map((c) => c.name).sort(),
      digitized.map((c) => c.name).sort(),
      `${id}: the derivation must cover exactly the lithologies the printed chart draws`,
    );
    for (const curve of derived.curves) {
      const m = MATS[curve.name];
      const twin = digitized.find((c) => c.name === curve.name);
      assert.equal(
        curve.grads.length,
        twin.grads.length,
        `${id} ${curve.name}: the derivation must graduate exactly where the printed chart does`,
      );
      for (const [phi, pe, rho] of curve.grads) {
        const f = phi / 100;
        const peCalc =
          (f * fl.Uf + (1 - f) * m.U) / (f * rhoe(fl.rb) + (1 - f) * rhoe(m.rb));
        assert.ok(
          Math.abs(pe - peCalc) <= 0.0002,
          `${id} ${curve.name} at ${phi} p.u.: derived Pe ${pe} is not the G&D physics value ${peCalc}`,
        );
        assert.ok(
          Math.abs(rho - (m.rhomaGrad - f * (m.rhomaGrad - fl.rhofGrad))) <= 0.0002,
          `${id} ${curve.name} at ${phi} p.u.: density graduation is not the chart arithmetic`,
        );
        const twinGrad = twin.grads.find((g) => g[0] === phi);
        assert.ok(twinGrad, `${id} ${curve.name}: printed chart has no graduation at ${phi} p.u.`);
        assert.ok(
          Math.abs(pe - twinGrad[1]) <= PE_BAND,
          `${id} ${curve.name} at ${phi} p.u.: derived Pe ${pe} vs printed ${twinGrad[1]} exceeds the tool-response band`,
        );
        assert.ok(
          Math.abs(rho - twinGrad[2]) <= 0.0005,
          `${id} ${curve.name} at ${phi} p.u.: density ${rho} vs printed ${twinGrad[2]} - the density leg must be exact`,
        );
      }
    }
  }
});

test("the_gardner_dumanoir_constants_are_exactly_the_paper_table_and_lith5_legend_values", () => {
  // G&D 1980 printed table (U and rho_b(log) columns) + chartbook Lith-5 legend p. 198
  // + Lith-3/Lith-4 chart headers (density-scale rho_f). DEC-079.
  assert.equal(GD.RHOE_SCALE, 1.0704);
  assert.equal(GD.RHOE_OFFSET, 0.1883);
  assert.deepEqual(
    GD.MATS.map((m) => [m.name, m.U, m.rbLog, m.rhoMaGrad]),
    [
      ["Quartz sandstone", 4.78, 2.64, 2.65],
      ["Calcite (limestone)", 13.8, 2.71, 2.71],
      ["Dolomite", 9.0, 2.88, 2.87],
    ],
  );
  assert.deepEqual(GD.FLUIDS, {
    fresh: { Uf: 0.398, rbLog: 1.0, rhoFGrad: 1.0 },
    salt: { Uf: 1.36, rbLog: 1.11, rhoFGrad: 1.1 },
  });
});

/** Extract one digitized overlay's points and lines from the generated source. */
function digitizedPointsAndLines(id) {
  const source = fs.readFileSync(path.join(repo, "src", "ui", "chartOverlays.ts"), "utf8");
  const start = source.indexOf(`id: "${id}"`);
  assert.ok(start >= 0, `digitized overlay ${id} not found`);
  const end = source.indexOf('id: "', start + 10);
  const block = source.slice(start, end < 0 ? undefined : end);
  const points = [...block.matchAll(/\{ x: ([\d.-]+), y: ([\d.-]+), label: "([^"]+)" \}/g)].map(
    (m) => ({ x: Number(m[1]), y: Number(m[2]), label: m[3] }),
  );
  const lines = [
    ...block.matchAll(/\{ pts: \[\[([\d.-]+),([\d.-]+)\],\[([\d.-]+),([\d.-]+)\]\](?:, label: "([^"]+)")?(?:, dash: true)? \}/g),
  ].map((m) => ({
    pts: [
      [Number(m[1]), Number(m[2])],
      [Number(m[3]), Number(m[4])],
    ],
    label: m[5],
  }));
  return { points, lines };
}

test("the_derived_lith6_mid_ternary_reproduces_the_digitized_chart_and_keeps_only_the_named_annotations_digitized", () => {
  const U_TOL = 0.12; // digitization tolerance on the 4-17 b/cc Umaa axis
  const RHO_TOL = 0.012; // and on the 2.6-3.1 g/cc rhomaa axis
  const derived = derivedOverlays().find((d) => d.id === "lith6_mid");
  assert.ok(derived, "lith6_mid must be a derived overlay");
  const digitized = digitizedPointsAndLines("lith6_mid");

  // Every derived point sits on its digitized twin within tolerance.
  for (const point of derived.points) {
    const twin = digitized.points.find((p) => p.label === point.label);
    assert.ok(twin, `digitized twin missing for point ${point.label}`);
    assert.ok(
      Math.abs(point.x - twin.x) <= U_TOL && Math.abs(point.y - twin.y) <= RHO_TOL,
      `${point.label}: derived (${point.x}, ${point.y}) vs digitized (${twin.x}, ${twin.y})`,
    );
  }

  const near = (a, b) => Math.abs(a[0] - b[0]) <= U_TOL && Math.abs(a[1] - b[1]) <= RHO_TOL;
  const sameLine = (d, g) =>
    (near(d.pts[0], g.pts[0]) && near(d.pts[1], g.pts[1])) ||
    (near(d.pts[0], g.pts[1]) && near(d.pts[1], g.pts[0]));

  // The three labeled percent edges match by label.
  for (const edge of derived.lines.filter((l) => l.label)) {
    const twin = digitized.lines.find((l) => l.label === edge.label);
    assert.ok(twin, `digitized twin missing for edge ${edge.label}`);
    assert.ok(sameLine(edge, twin), `${edge.label} moved beyond digitization tolerance`);
  }

  // The 20/40/60/80 interior grid matches the digitized unlabeled lines one to one.
  const derivedGrid = derived.lines.filter((l) => !l.label);
  const digitizedGrid = digitized.lines.filter((l) => l.label === undefined);
  assert.equal(derivedGrid.length, 12, "three vertices x four levels");
  assert.equal(
    digitizedGrid.length,
    derivedGrid.length,
    "the digitized chart draws exactly the grid the geometry derives",
  );
  const unmatched = new Set(digitizedGrid);
  for (const line of derivedGrid) {
    const twin = [...unmatched].find((g) => sameLine(line, g));
    assert.ok(
      twin,
      `no digitized twin for derived grid line ${JSON.stringify(line.pts)} - the constant-fraction geometry does not reproduce the chart`,
    );
    unmatched.delete(twin);
  }

  // Only the four annotation labels stay digitized - never a percent edge.
  assert.deepEqual(
    [...(derived.keepDigitizedLineLabels ?? [])].sort(),
    ["Barite", "Gas direction", "Heavy minerals", "Salt"],
  );
});

test("the_derived_lith2_ratio_lines_are_geometry_from_the_printed_boundary_values_and_match_the_digitized_lines_one_to_one", () => {
  // Lith-2 p. 194 labels every boundary line with its own ratio value; a constant-ratio
  // line is the segment from the origin to where it exits the printed frame. The dashed
  // clay/feldspar lines and the region label points are measured-variability graphics
  // with no printed numeric source (Quirein 1982 / Hassan 1976 print them only as
  // figures), so they must stay digitized - pinned from both sides below.
  const K_TOL = 0.001; // the digitized lines are analytic geometry, slack for 4dp rounding
  const TH_TOL = 0.005;
  // The page's printed values and frame, restated independently of the generator.
  const PRINTED = [
    ["Th/K = 25", 25],
    ["Th/K = 12", 12],
    ["Th/K = 3.5", 3.5],
    ["Th/K = 2.0", 2.0],
    ["Th/K = 0.6", 0.6],
    ["Th/K = 0.3", 0.3],
  ];
  const derived = derivedOverlays().find((d) => d.id === "lith2_thk");
  assert.ok(derived, "lith2_thk must be a derived overlay");
  assert.equal(derived.lines.length, PRINTED.length, "one line per printed boundary value");
  assert.equal(derived.points, undefined, "no lith2 point may be derived - none has a printed numeric source");
  assert.equal(derived.curves, undefined, "lith2 has no graduated curves");

  const digitized = digitizedPointsAndLines("lith2_thk");
  const digitizedRatioLines = digitized.lines.filter((l) => l.label?.startsWith("Th/K"));
  assert.equal(
    digitizedRatioLines.length,
    PRINTED.length,
    "the derivation must cover exactly the boundary lines the printed chart draws",
  );
  for (const [label, r] of PRINTED) {
    const end = 5 * r >= 25 ? [25 / r, 25] : [5, 5 * r];
    const line = derived.lines.find((l) => l.label === label);
    assert.ok(line, `derived line missing for printed label ${label}`);
    assert.ok(
      Math.abs(line.pts[0][0]) <= K_TOL && Math.abs(line.pts[0][1]) <= TH_TOL,
      `${label}: a constant-ratio boundary starts at the origin`,
    );
    assert.ok(
      Math.abs(line.pts[1][0] - end[0]) <= K_TOL && Math.abs(line.pts[1][1] - end[1]) <= TH_TOL,
      `${label}: derived end ${JSON.stringify(line.pts[1])} is not the frame exit ${JSON.stringify(end)}`,
    );
    const twin = digitizedRatioLines.find((l) => l.label === label);
    assert.ok(twin, `digitized twin missing for ${label}`);
    assert.ok(
      Math.abs(line.pts[1][0] - twin.pts[1][0]) <= K_TOL &&
        Math.abs(line.pts[1][1] - twin.pts[1][1]) <= TH_TOL &&
        Math.abs(twin.pts[0][0]) <= K_TOL &&
        Math.abs(twin.pts[0][1]) <= TH_TOL,
      `${label}: digitized line ${JSON.stringify(twin.pts)} does not match the printed-value geometry`,
    );
  }

  // Both sides of the keep rule: exactly the two dashed lines stay digitized, and they
  // really exist in the digitized set so the keep-list names real lines.
  assert.deepEqual([...(derived.keepDigitizedLineLabels ?? [])].sort(), ["Clay line", "Feldspar line"]);
  for (const kept of derived.keepDigitizedLineLabels) {
    assert.ok(
      digitized.lines.some((l) => l.label === kept),
      `keep-list names ${kept} but the digitized chart has no such line`,
    );
  }
});

test("the_lith2_derivation_constants_are_exactly_the_printed_chart_page_values", () => {
  // Chartbook Lith-2 p. 194 (former CP-19): the six labeled boundary values and the
  // printed frame. Labels stay verbatim - the page prints "2.0" with its zero.
  assert.equal(LITH2.K_MAX, 5);
  assert.equal(LITH2.TH_MAX, 25);
  assert.deepEqual(
    LITH2.RATIOS.map(({ r, label }) => [r, label]),
    [
      [25, "Th/K = 25"],
      [12, "Th/K = 12"],
      [3.5, "Th/K = 3.5"],
      [2.0, "Th/K = 2.0"],
      [0.6, "Th/K = 0.6"],
      [0.3, "Th/K = 0.3"],
    ],
  );
});

test("the_thirteen_deleted_vendor_definitions_are_gone_from_the_catalog_and_the_six_derived_ones_remain", () => {
  // DEC-082 (2026-08-19): Jauhar's Gate 5 word on the retained register was DELETE.
  // The thirteen vendor definitions (nine tool-response per DEC-078, Por-20's two per
  // DEC-080, Lith-1's two per DEC-081) were removed from chartOverlays.ts; the policy
  // module keeps the DELETION RECORD. This test is the fail-closed release-inventory
  // pin: a deleted id must never reappear in the catalog, the record must name exactly
  // the thirteen ruled ids, and the delete must not have taken a derived definition
  // with it.
  const THIRTEEN = [
    "lith1_pek", "lith1_pethk", "por11", "por12", "por13_aplc", "por13_fplc",
    "por14_aplc", "por14_fplc", "por16", "por18", "por19", "por20_fo", "por20_ta",
  ];
  const policy = fs.readFileSync(path.join(repo, "src", "ui", "chartOverlayPolicy.ts"), "utf8");
  const listMatch = policy.match(/DELETED_VENDOR_OVERLAY_IDS[^=]*=\s*\[([\s\S]*?)\];/);
  assert.ok(listMatch, "DELETED_VENDOR_OVERLAY_IDS not found in chartOverlayPolicy.ts");
  // Strip line comments first: a commented-out entry is NOT in the record.
  const body = listMatch[1].replace(/\/\/[^\n]*/g, "");
  const recorded = [...body.matchAll(/"([a-z0-9_]+)"/g)].map((m) => m[1]);
  assert.deepEqual(
    [...recorded].sort(),
    THIRTEEN,
    "the deletion record must name exactly the thirteen ruled ids",
  );

  const catalog = fs.readFileSync(path.join(repo, "src", "ui", "chartOverlays.ts"), "utf8");
  const catalogIds = [...catalog.matchAll(/id: "([a-z0-9_]+)"/g)].map((m) => m[1]);
  for (const id of THIRTEEN) {
    assert.ok(
      !catalogIds.includes(id),
      `${id} was deleted under DEC-082 and must never reappear in chartOverlays.ts`,
    );
  }
  const derivedIds = derivedOverlays().map((d) => d.id).sort();
  assert.deepEqual(
    derivedIds,
    ["lith2_thk", "lith3", "lith4", "lith6_mid", "por22_fo", "por22_ta"],
    "the derived set must still be exactly the six executed derivations",
  );
  for (const id of derivedIds) {
    assert.ok(
      catalogIds.includes(id),
      `derived definition ${id} must still exist in the catalog - the delete took too much`,
    );
  }
});

test("the_generated_derived_overlay_module_is_current", () => {
  const onDisk = fs.readFileSync(
    path.join(repo, "src", "ui", "chartOverlaysDerived.gen.ts"),
    "utf8",
  );
  // Compare CONTENT, not line endings: a fresh checkout materializes the file with
  // CRLF under git autocrlf, and that must not read as staleness.
  assert.equal(
    onDisk.replace(/\r\n/g, "\n"),
    renderModule().replace(/\r\n/g, "\n"),
    "chartOverlaysDerived.gen.ts is stale - run: node tools/gen-derived-overlays.mjs",
  );
});
