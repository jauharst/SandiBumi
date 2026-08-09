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
