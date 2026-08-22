/**
 * Frontend paint harness — pass 1 of the performance brief. It measures and changes nothing.
 *
 * The backend half of a click is timed by `src-tauri/src/perf_baseline_test.rs`. This is the other
 * half: how long the browser takes to turn data it already has into pixels. Nothing in the app has
 * ever measured it — `healthPanel.ts` is a RESOURCE monitor (CPU/MEM/USER/GDI gauges) and cannot
 * answer "how long between the click and the picture".
 *
 * **It is not part of the app and must never be imported by it.** Vite serves the project root, so
 * it is loaded from the dev server on demand:
 *
 *   const h = await import('/tools/perf-frontend.mjs');
 *   await h.runAll();
 *
 * Run it against `npm run tauri dev`'s vite server (port 1420). Every `invoke` failure in a
 * browser tab is benign — this harness never calls the backend, by design: it feeds the draw
 * functions typed arrays directly, so what it reports is paint cost with the data already in hand.
 *
 * ## What a number here means
 *
 * The timed region is the synchronous draw call. For a 2D canvas that is where essentially all the
 * cost is — path building, binning, projection and stroking all happen inside it. GPU compositing
 * afterwards is not included, and neither is the IPC hop that delivered the data. So, like the
 * backend harness, these are LOWER BOUNDS on click-to-paint, and the report says so rather than
 * presenting a sum as the whole truth.
 */

/** Sample counts to sweep. 1_562 is one real logged well on the reference machine; the rest are
 *  10 and 100 wells' worth, which is what a multi-well overlay actually puts on one plot. */
const SIZES = [1_562, 15_620, 156_200];

/** Timed repetitions after a warm-up. The first call pays for JIT and for the canvas's first
 *  backing-store allocation, and including it would report a cost no later click ever pays. */
const REPS = 7;

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
}

/** Deterministic pseudo-random logs, same shape as the Rust harness builds: a slow shaliness
 *  cycle so the values are plausible curves rather than uniform noise. A histogram of noise bins
 *  differently from a histogram of a real log, so noise would measure the wrong thing. */
function syntheticCurves(n) {
  let seed = 1;
  const rand = () => {
    seed = (seed * 1103515245 + 12345) & 0x7fffffff;
    return seed / 0x7fffffff - 0.5;
  };
  const nphi = new Float32Array(n);
  const rhob = new Float32Array(n);
  const gr = new Float32Array(n);
  const depth = new Float32Array(n);
  for (let i = 0; i < n; i += 1) {
    const cycle = Math.sin(i / 140) * 0.5 + 0.5;
    depth[i] = 1500 + 0.1524 * i;
    gr[i] = 25 + 105 * cycle + rand() * 4;
    nphi[i] = 0.12 + 0.3 * cycle + rand() * 0.01;
    rhob[i] = 2.2 + 0.35 * cycle + rand() * 0.02;
  }
  return { depth, gr, nphi, rhob };
}

/** An offscreen-but-attached canvas at a realistic panel size. Attached because a detached canvas
 *  can take a different path in the browser's compositor, and the point is to measure the one the
 *  app takes. */
function makeCanvas(width = 900, height = 600) {
  const canvas = document.createElement("canvas");
  canvas.className = "plot-canvas";
  canvas.style.cssText = `position:fixed;left:-10000px;top:0;width:${width}px;height:${height}px`;
  document.body.appendChild(canvas);
  return canvas;
}

/** Runs `fn` once to warm up, then REPS times, returning the timing summary in milliseconds. */
function bench(label, size, fn) {
  fn(); // warm-up, discarded
  const runs = [];
  for (let i = 0; i < REPS; i += 1) {
    const t0 = performance.now();
    fn();
    runs.push(performance.now() - t0);
  }
  return {
    label,
    size,
    median: +median(runs).toFixed(2),
    min: +Math.min(...runs).toFixed(2),
    max: +Math.max(...runs).toFixed(2),
  };
}

/** Does this browser have WebGPU at all? The log view is the app's only WebGPU surface, so if the
 *  answer is no, its paint cost is simply not measurable here and the report must say that rather
 *  than quietly omitting the row. */
export async function webgpuStatus() {
  if (!("gpu" in navigator)) return "absent (navigator.gpu undefined)";
  try {
    const adapter = await navigator.gpu.requestAdapter();
    return adapter ? "available" : "present but no adapter";
  } catch (err) {
    return `error: ${err}`;
  }
}

/** A three-track layout in the shape the built-in Standard layout uses: GR, then the
 *  neutron-density pair, then resistivity on a log scale. Built here rather than fetched, because
 *  fetching would need the backend and this harness deliberately has none. */
function benchLayout() {
  const curve = (curve_name, color, min, max) => ({ curve_name, color, min, max });
  return {
    name: "perf",
    tracks: [
      { title: "GR", width_weight: 1, scale_type: "linear", curves: [curve("GR", "#2e7d32", 0, 150)] },
      {
        title: "Porosity",
        width_weight: 1.5,
        scale_type: "linear",
        curves: [curve("NPHI", "#1565c0", 0.45, -0.15), curve("RHOB", "#c62828", 1.95, 2.95)],
      },
      { title: "Resistivity", width_weight: 1, scale_type: "log", curves: [curve("RES_DEEP", "#000", 0.2, 2000)] },
    ],
  };
}

/**
 * The log view: TRUE interaction-to-paint, not just a draw call.
 *
 * This is the one measurement here that is not a lower bound on its own leg. `LogCanvasRenderer`
 * marks itself dirty and paints on the next animation frame, then calls `onFrameRendered` — so the
 * clock starts on the interaction and stops when the GPU frame is actually on screen, which is what
 * a user experiences. Every other row in this harness times a synchronous call and stops before
 * compositing.
 *
 * Returns `null` when WebGPU is unavailable, so the report can say the row is missing and why
 * rather than leaving a silent gap.
 */
export async function runLogView(sizes = SIZES) {
  if ((await webgpuStatus()) !== "available") return null;
  const { LogCanvasRenderer } = await import("/src/LogCanvasRenderer.ts");
  const results = [];
  const canvas = makeCanvas(900, 800);
  const renderer = new LogCanvasRenderer(canvas);
  await renderer.init();

  // The renderer normally paints from a `requestAnimationFrame` loop, and a browser PAUSES rAF in
  // a hidden tab — which the measurement pane is, so waiting on `onFrameRendered` hangs forever.
  // Driving `render()` directly and then awaiting the GPU queue measures the same work on a clock
  // that does not depend on the tab being on screen: geometry building and command encoding on the
  // CPU, plus the GPU actually finishing. What it EXCLUDES is the wait for the next vsync, which
  // in the app adds up to one frame interval (~16 ms at 60 Hz) on top of every number below.
  const paint = async () => {
    renderer.render();
    await renderer.device.queue.onSubmittedWorkDone();
  };

  /** One interaction, timed from the call to the GPU finishing the frame it produces. */
  async function interaction(label, size, act) {
    const runs = [];
    for (let i = 0; i < REPS; i += 1) {
      const t0 = performance.now();
      act(i);
      await paint();
      runs.push(performance.now() - t0);
    }
    results.push({
      label,
      size,
      median: +median(runs).toFixed(2),
      min: +Math.min(...runs).toFixed(2),
      max: +Math.max(...runs).toFixed(2),
    });
  }

  for (const size of sizes) {
    const { depth, gr, nphi, rhob } = syntheticCurves(size);
    // RES_DEEP is drawn on a log axis, so it must be positive everywhere or the track measures
    // the renderer's rejection path instead of its drawing path.
    const res = new Float32Array(size);
    for (let i = 0; i < size; i += 1) res[i] = Math.max(0.2, 60 * (1 - (nphi[i] - 0.12) / 0.3) + 1.5);
    const series = [
      { curve_name: "GR", depth, value: gr },
      { curve_name: "NPHI", depth, value: nphi },
      { curve_name: "RHOB", depth, value: rhob },
      { curve_name: "RES_DEEP", depth, value: res },
    ];
    const weights = new Map();

    // Well switch: the layout is reloaded with a different well's curves, then painted.
    await interaction("well switch (paint)", size, () => {
      renderer.loadLayout(benchLayout(), series, weights);
    });
    // Scroll and zoom reuse the loaded geometry — the cost a user pays repeatedly.
    await interaction("log scroll", size, (i) => renderer.scrollToDepth(1500 + i * 20));
    await interaction("log zoom", size, (i) => renderer.zoomAt(400, i % 2 === 0 ? 1.25 : 0.8));
  }

  renderer.dispose();
  canvas.remove();
  return results;
}

export async function runAll() {
  const results = [];
  const canvas = makeCanvas();
  const [{ drawHistogram }, { drawCrossplot }, { drawPickett }] = await Promise.all([
    import("/src/ui/histogramPanel.ts"),
    import("/src/ui/crossplotPanel.ts"),
    import("/src/ui/pickettPanel.ts"),
  ]);

  for (const size of SIZES) {
    const { gr, nphi, rhob } = syntheticCurves(size);
    results.push(bench("histogram", size, () => drawHistogram(canvas, gr, "GR", [])));
    results.push(
      bench("crossplot", size, () => drawCrossplot(canvas, "NPHI", "RHOB", "", nphi, rhob, new Float32Array(0))),
    );
    // Pickett takes (resistivity, porosity). `rhob` stands in as a value source for the
    // resistivity channel purely to give the plot numbers to draw — this measures drawing cost,
    // not petrophysics, and the values are never interpreted. `null` line, no picks: the overlay
    // is a handful of strokes and including it would measure the line, not the point cloud.
    results.push(bench("pickett", size, () => drawPickett(canvas, rhob, nphi, null, [])));
  }

  canvas.remove();
  return { webgpu: await webgpuStatus(), results };
}

/** Renders the result as a fixed-width table, so it can be pasted into a report unchanged. */
export function table(results) {
  const head = `${"PLOT".padEnd(12)}${"POINTS".padStart(9)}${"MEDIAN".padStart(10)}${"MIN".padStart(9)}${"MAX".padStart(9)}`;
  const rows = results.map(
    (r) =>
      `${r.label.padEnd(12)}${String(r.size).padStart(9)}${(`${r.median}ms`).padStart(10)}` +
      `${(`${r.min}ms`).padStart(9)}${(`${r.max}ms`).padStart(9)}`,
  );
  return [head, ...rows].join("\n");
}
