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
import { CONSTANTS, derivedOverlays, renderModule } from "./gen-derived-overlays.mjs";

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

test("the_generated_derived_overlay_module_is_current", () => {
  const onDisk = fs.readFileSync(
    path.join(repo, "src", "ui", "chartOverlaysDerived.gen.ts"),
    "utf8",
  );
  assert.equal(
    onDisk,
    renderModule(),
    "chartOverlaysDerived.gen.ts is stale - run: node tools/gen-derived-overlays.mjs",
  );
});
