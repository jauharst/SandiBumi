import type { CurveStyle, Layout, Track, TrackCurveSeries } from "./ipc";
import { classRuns, faciesColor, valueFrac } from "./ui/plotCanvas";
import { appState } from "./state";
import { pxPerUnitAt1to1 } from "./units";
import { trackCurveKey } from "./trackCurveRequest";

const VERTEX_SHADER = /* wgsl */ `
struct Transform {
  topDepth: f32,
  pxPerUnit: f32,
  canvasHeightPx: f32,
  _pad: f32,
};
@group(0) @binding(0) var<uniform> transform: Transform;

@vertex
fn main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
  let pxY = (position.y - transform.topDepth) * transform.pxPerUnit;
  let ndcY = 1.0 - 2.0 * pxY / transform.canvasHeightPx;
  return vec4<f32>(position.x, ndcY, 0.0, 1.0);
}
`;

const FRAGMENT_SHADER = /* wgsl */ `
struct ColorUniform {
  color: vec4<f32>,
};
@group(0) @binding(1) var<uniform> colorU: ColorUniform;

@fragment
fn main() -> @location(0) vec4<f32> {
  return colorU.color;
}
`;

interface ViewState {
  topDepth: number;
  pxPerUnit: number;
}

interface CurveGeometry {
  curveName: string;
  vertexBuffer: GPUBuffer;
  vertexCount: number;
  hidden: boolean;
  // Each curve owns its own color buffer + bind group (written once at load time) rather
  // than sharing one mutated per draw: queue.writeBuffer isn't synchronized with the
  // encoder, so reusing a single buffer across draws in a loop makes every draw see only
  // the last-written color once the GPU actually executes them.
  bindGroup: GPUBindGroup;
  /** Optional shading between the curve and a track edge (triangle-list, alpha-blended). */
  fillVertexBuffer?: GPUBuffer;
  fillVertexCount?: number;
  fillBindGroup?: GPUBindGroup;
}

interface ReadoutSample {
  curveName: string;
  value: number;
}

// CSS px per STORED depth unit at a true 1:1 print scale — 96 CSS px/in ÷ 0.0254 m/in for
// metres, exactly 96 × 12 for feet. A named "1:N" scale is therefore pxPerUnit1to1() / N.
//
// This used to be a metres-only constant, which meant every named scale on a foot-indexed
// well was mislabelled by 3.28× (engineering review F2e). It reads the PROJECT unit, never
// the display unit: "1:200" claims 200 units of rock per unit of paper, so it depends on
// how long a stored unit physically is, not on which unit the reader prefers.
function pxPerUnit1to1(): number {
  return pxPerUnitAt1to1(appState.projectDepthUnit.get());
}
// Open at a 1:2000 overview (≈212 m of section in a 400 px pane) — a real, honest ratio.
// The old 96/100 = 0.96 was labelled "1:100" but was actually ~1:3937, and the dropdown's
// true 1:100 then clamped to 20 (see MAX_PX_PER_UNIT), so 1:20/1:50/1:100 all looked identical.
const defaultPxPerUnit = (): number => pxPerUnit1to1() / 2000;
// Zoom bounds, shared by setScale/zoomAt so every path clamps identically. Max = a true 1:10
// (finer than any preset); min ≈ 1:189000 (frames a whole deep well when zoomed fully out).
const MIN_PX_PER_UNIT = 0.02;
const maxPxPerUnit = (): number => pxPerUnit1to1() / 10;

/**
 * Multi-track hardware-accelerated log viewer. Given a Layout (tracks + per-curve
 * scale/color), per-track pixel weights, and the decimated curve series for a well, lays
 * every curve's line-list geometry out once (CPU side) with the RAW depth value baked into
 * each vertex — the vertex shader converts depth -> pixel -> NDC dynamically from a
 * depth/scale uniform, so panning and zooming never require rebuilding geometry.
 */
export class LogCanvasRenderer {
  private canvas: HTMLCanvasElement;
  private device!: GPUDevice;
  private context!: GPUCanvasContext;
  private pipeline!: GPURenderPipeline;
  private fillPipeline!: GPURenderPipeline;
  private uniformBuffer!: GPUBuffer;
  private bindGroupLayout!: GPUBindGroupLayout;

  private curves: CurveGeometry[] = [];
  private depthMin = 0;
  private depthMax = 1;
  private seriesByName = new Map<string, TrackCurveSeries>();
  /** Horizontal extents (0–1 canvas fractions) per track, for cursor hit-testing. */
  private trackRanges: { title: string; leftFrac: number; rightFrac: number }[] = [];

  private view: ViewState = { topDepth: 0, pxPerUnit: defaultPxPerUnit() };
  private dirty = true;
  private running = false;
  /** Cached clear color read from --bg-panel. getComputedStyle forces a style recalc, and a
   *  drag-pan marks every frame dirty, so we read it once and only re-read on a theme change
   *  (repaint() clears this). The color only changes when the theme does. */
  private cachedBgColor: string | null = null;
  /** Teardown for listeners attached to `window` (they outlive the removed canvas) and
   *  any pending timers, run in dispose() so a closed log view leaks nothing. */
  private cleanups: Array<() => void> = [];

  public onViewSettled: (() => void) | null = null;

  /** Fires after every rendered frame (pan/zoom included) so overlays drawn on a
   *  separate 2D canvas — core points, annotations — stay glued to the view. */
  public onFrameRendered: (() => void) | null = null;
  /** `trackTitle` is the track under the cursor's X (null when off every track). */
  public onCursorMove: ((depth: number | null, samples: ReadoutSample[], trackTitle: string | null) => void) | null = null;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
  }

  async init(): Promise<void> {
    if (!navigator.gpu) {
      throw new Error("WebGPU is not supported in this environment");
    }
    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) {
      throw new Error("Failed to acquire a WebGPU adapter");
    }
    this.device = await adapter.requestDevice();

    const context = this.canvas.getContext("webgpu");
    if (!context) {
      throw new Error("Failed to acquire a WebGPU canvas context");
    }
    this.context = context;

    const format = navigator.gpu.getPreferredCanvasFormat();
    this.context.configure({ device: this.device, format, alphaMode: "opaque" });

    this.uniformBuffer = this.device.createBuffer({
      size: 16,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });

    const vertexModule = this.device.createShaderModule({ code: VERTEX_SHADER });
    const fragmentModule = this.device.createShaderModule({ code: FRAGMENT_SHADER });

    this.bindGroupLayout = this.device.createBindGroupLayout({
      entries: [
        { binding: 0, visibility: GPUShaderStage.VERTEX, buffer: { type: "uniform" } },
        { binding: 1, visibility: GPUShaderStage.FRAGMENT, buffer: { type: "uniform" } },
      ],
    });

    const pipelineLayout = this.device.createPipelineLayout({ bindGroupLayouts: [this.bindGroupLayout] });
    const vertexState: GPUVertexState = {
      module: vertexModule,
      entryPoint: "main",
      buffers: [{ arrayStride: 2 * 4, attributes: [{ shaderLocation: 0, offset: 0, format: "float32x2" }] }],
    };

    this.pipeline = this.device.createRenderPipeline({
      layout: pipelineLayout,
      vertex: vertexState,
      fragment: { module: fragmentModule, entryPoint: "main", targets: [{ format }] },
      primitive: { topology: "line-list" },
    });

    // Curve-fill shading: same shaders, triangle topology, straight alpha blending.
    this.fillPipeline = this.device.createRenderPipeline({
      layout: pipelineLayout,
      vertex: vertexState,
      fragment: {
        module: fragmentModule,
        entryPoint: "main",
        targets: [
          {
            format,
            blend: {
              color: { srcFactor: "src-alpha", dstFactor: "one-minus-src-alpha", operation: "add" },
              alpha: { srcFactor: "one", dstFactor: "one-minus-src-alpha", operation: "add" },
            },
          },
        ],
      },
      primitive: { topology: "triangle-list" },
    });

    this.attachPanHandlers();
    this.attachCursorHandler();
    this.startLoop();
  }

  /** Lays out every track's curves using `trackWeights` (keyed by track title) for column
   *  proportions, and centers the view on the data if this is the first load. */
  loadLayout(
    layout: Layout,
    series: TrackCurveSeries[],
    trackWeights: Map<string, number>,
    preserveDepthRange = false,
  ): void {
    this.seriesByName = new Map(series.map((s) => [s.curve_name, s]));

    if (!preserveDepthRange) {
      let depthMin = Infinity;
      let depthMax = -Infinity;
      for (const s of series) {
        for (const d of s.depth) {
          if (d < depthMin) depthMin = d;
          if (d > depthMax) depthMax = d;
        }
      }
      const isFirstLoad = this.curves.length === 0 && this.depthMin === 0 && this.depthMax === 1;
      if (!Number.isFinite(depthMin) || !Number.isFinite(depthMax) || depthMin === depthMax) {
        depthMin = 0;
        depthMax = 1;
      }
      this.depthMin = depthMin;
      this.depthMax = depthMax;
      if (isFirstLoad) {
        this.view.topDepth = depthMin;
      }
    }

    const previousHidden = new Map(this.curves.map((c) => [c.curveName, c.hidden]));
    for (const c of this.curves) {
      c.vertexBuffer.destroy();
      c.fillVertexBuffer?.destroy();
    }
    this.curves = [];

    const totalWeight = layout.tracks.reduce((sum, t) => sum + (trackWeights.get(t.title) ?? 150), 0) || 1;
    let cumulativeWeight = 0;
    this.trackRanges = [];

    for (const track of layout.tracks) {
      const weight = trackWeights.get(track.title) ?? 150;
      const trackLeftFrac = cumulativeWeight / totalWeight;
      cumulativeWeight += weight;
      const trackRightFrac = cumulativeWeight / totalWeight;
      this.trackRanges.push({ title: track.title, leftFrac: trackLeftFrac, rightFrac: trackRightFrac });
      // Well-diagram, point-data, array-log and image tracks keep their column (so the
      // overlay can draw into it) but contribute no curve geometry — all four live entirely
      // on the 2D overlay canvas.
      const kind = track.kind ?? "curves";
      if (kind === "well_diagram" || kind === "point_data" || kind === "array_log" || kind === "image") continue;
      const trackLeftNdc = -1 + 2 * trackLeftFrac;
      const trackRightNdc = -1 + 2 * trackRightFrac;

      for (const curveStyle of track.curves) {
        const seriesKey = trackCurveKey(curveStyle);
        const s = this.seriesByName.get(seriesKey);
        if (!s) continue;
        if (curveStyle.fill === "blocks") {
          for (const geometry of this.buildBlockGeometries(s, curveStyle, trackLeftNdc, trackRightNdc, seriesKey)) {
            geometry.hidden = previousHidden.get(geometry.curveName) ?? false;
            this.curves.push(geometry);
          }
          continue;
        }
        const geometry = this.buildCurveGeometry(
          s,
          curveStyle,
          track.scale_type,
          trackLeftNdc,
          trackRightNdc,
          seriesKey,
        );
        if (geometry) {
          geometry.hidden = previousHidden.get(geometry.curveName) ?? false;
          this.curves.push(geometry);
        }
        // Crossover shading is its own geometry pair (one colour each side), because the
        // fill pipeline binds a single colour uniform per draw. They carry this curve's
        // name, so hiding the curve hides its shading with it.
        if (curveStyle.fill === "curve") {
          const reference = this.resolveCrossover(track, curveStyle);
          if (reference) {
            for (const g of this.buildCrossoverGeometries(
              s, curveStyle, reference, track.scale_type, trackLeftNdc, trackRightNdc, seriesKey,
            )) {
              g.hidden = previousHidden.get(g.curveName) ?? false;
              this.curves.push(g);
            }
          }
        }
      }
    }

    this.dirty = true;
  }

  /** Toggles a curve's visibility without rebuilding any geometry. */
  setCurveHidden(curveName: string, hidden: boolean): void {
    for (const c of this.curves) {
      if (c.curveName === curveName) c.hidden = hidden;
    }
    this.dirty = true;
  }

  private buildCurveGeometry(
    series: TrackCurveSeries,
    style: {
      curve_name: string;
      color: string;
      min: number;
      max: number;
      draw_style?: "line" | "step";
      // "blocks" never reaches here — loadLayout routes it to buildBlockGeometries.
      // "curve" draws its line here and its shading in buildCrossoverGeometries.
      fill?: "none" | "left" | "right" | "curve" | "blocks";
      fill_color?: string;
      fill_opacity?: number;
    },
    scaleType: "linear" | "log",
    trackLeftNdc: number,
    trackRightNdc: number,
    seriesKey: string,
  ): CurveGeometry | null {
    const n = Math.min(series.depth.length, series.value.length);
    const positions: number[] = [];

    // Shared with the print (`composite.rs::value_frac`) so the screen and the deliverable
    // cannot disagree about where a value sits, or whether it HAS a place at all. `null` is a
    // real answer: a non-positive sample on a log axis has no position, and the substitution
    // this used to make (`Math.max(v, 1e-6)`) drew a permeability of zero as a continuous dip
    // to the track edge - a measurement that was never made - while the print showed a gap.
    const valueToNdcX = (v: number): number | null => {
      const f = valueFrac(v, style.min, style.max, scaleType === "log");
      if (f == null) return null;
      // A continuous curve CLAMPS at the track edge; only its existence is in question above.
      const clamped = Math.max(0, Math.min(1, f));
      return trackLeftNdc + clamped * (trackRightNdc - trackLeftNdc);
    };

    // "step": the sample's value holds all the way down to the next sample before it jumps,
    // so each interval draws two segments (a vertical hold, then a horizontal jump) instead
    // of one diagonal. Same corner the composite exporter builds.
    const step = style.draw_style === "step";
    for (let i = 0; i < n - 1; i++) {
      const v0 = series.value[i];
      const v1 = series.value[i + 1];
      // A step curve HOLDS v0 down to the next sample's depth even when that next sample is
      // missing - which is the stated contract and what the print does. Skipping the whole
      // interval made the last sample before every gap draw as a zero-length tick: a blocked
      // VSH ended one sample short of every washout, and a zone-constant curve lost the bottom
      // of its interval. The gap then starts where the missing sample actually is.
      if (Number.isNaN(v0)) continue;
      if (Number.isNaN(v1) && !step) continue;
      // Y is the RAW depth value — the vertex shader converts it to a pixel/NDC position
      // dynamically from the current pan/zoom uniform, so this buffer never needs rebuilding
      // just because the user panned or changed the vertical scale.
      const x0 = valueToNdcX(v0);
      if (x0 == null) continue;
      const x1 = Number.isNaN(v1) ? null : valueToNdcX(v1);
      const d0 = series.depth[i];
      const d1 = series.depth[i + 1];
      if (step) {
        positions.push(x0, d0, x0, d1);
        // The jump to the next sample only exists if there IS a next sample.
        if (x1 != null) positions.push(x0, d1, x1, d1);
      } else if (x1 != null) {
        positions.push(x0, d0, x1, d1);
      }
    }

    if (positions.length === 0) return null;

    const data = new Float32Array(positions);
    const vertexBuffer = this.device.createBuffer({
      size: data.byteLength,
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });
    this.device.queue.writeBuffer(vertexBuffer, 0, data);

    const colorBuffer = this.device.createBuffer({
      size: 16,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });
    this.device.queue.writeBuffer(colorBuffer, 0, new Float32Array(hexToRgba(style.color)));

    const bindGroup = this.device.createBindGroup({
      layout: this.bindGroupLayout,
      entries: [
        { binding: 0, resource: { buffer: this.uniformBuffer } },
        { binding: 1, resource: { buffer: colorBuffer } },
      ],
    });

    const geometry: CurveGeometry = {
      curveName: seriesKey,
      vertexBuffer,
      vertexCount: data.length / 2,
      hidden: false,
      bindGroup,
    };

    // Shading between the curve and a track edge: two triangles per sample segment,
    // in the same (NDC-x, raw-depth-y) space the line geometry uses.
    if (style.fill === "left" || style.fill === "right") {
      const edgeNdc = style.fill === "left" ? trackLeftNdc : trackRightNdc;
      const fillPositions: number[] = [];
      for (let i = 0; i < n - 1; i++) {
        const v0 = series.value[i];
        const v1 = series.value[i + 1];
        // Mirrors the line loop exactly: the shading may not cover an interval the line does
        // not draw, or a value with no position on the axis. A sample the axis cannot place
        // used to be substituted and then SHADED, which is the same false statement twice.
        if (Number.isNaN(v0)) continue;
        if (Number.isNaN(v1) && !step) continue;
        const x0 = valueToNdcX(v0);
        if (x0 == null) continue;
        // A stepped curve's shading is a rectangle per interval, not a wedge — the held
        // value bounds it on both sides.
        const x1 = step ? x0 : valueToNdcX(v1);
        if (x1 == null) continue;
        const d0 = series.depth[i];
        const d1 = series.depth[i + 1];
        fillPositions.push(x0, d0, edgeNdc, d0, x1, d1, edgeNdc, d0, edgeNdc, d1, x1, d1);
      }
      if (fillPositions.length > 0) {
        const fillData = new Float32Array(fillPositions);
        const fillVertexBuffer = this.device.createBuffer({
          size: fillData.byteLength,
          usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
        });
        this.device.queue.writeBuffer(fillVertexBuffer, 0, fillData);

        const rgba = hexToRgba(style.fill_color ?? style.color);
        rgba[3] = Math.max(0, Math.min(1, style.fill_opacity ?? 0.25));
        const fillColorBuffer = this.device.createBuffer({
          size: 16,
          usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
        });
        this.device.queue.writeBuffer(fillColorBuffer, 0, new Float32Array(rgba));

        geometry.fillVertexBuffer = fillVertexBuffer;
        geometry.fillVertexCount = fillData.length / 2;
        geometry.fillBindGroup = this.device.createBindGroup({
          layout: this.bindGroupLayout,
          entries: [
            { binding: 0, resource: { buffer: this.uniformBuffer } },
            { binding: 1, resource: { buffer: fillColorBuffer } },
          ],
        });
      }
    }

    return geometry;
  }

  /** Discrete class curve (fill: "blocks", e.g. FACIES): contiguous same-class runs become
   *  full-track-width rectangles, one geometry per class so each draw keeps the single-color
   *  uniform contract. Colors come from the shared facies palette, matching the crossplot's
   *  categorical coloring. NaN runs stay empty (background shows through). */
  private buildBlockGeometries(
    series: TrackCurveSeries,
    style: { curve_name: string; fill_opacity?: number },
    trackLeftNdc: number,
    trackRightNdc: number,
    seriesKey: string,
  ): CurveGeometry[] {
    const n = Math.min(series.depth.length, series.value.length);
    if (n === 0) return [];

    const trisByClass = new Map<number, number[]>();
    const pushRun = (cls: number, top: number, bottom: number): void => {
      let tris = trisByClass.get(cls);
      if (!tris) trisByClass.set(cls, (tris = []));
      tris.push(
        trackLeftNdc, top, trackRightNdc, top, trackLeftNdc, bottom,
        trackRightNdc, top, trackRightNdc, bottom, trackLeftNdc, bottom,
      );
    };

    for (const run of classRuns(series.depth, series.value)) {
      pushRun(run.cls, run.top, run.bottom);
    }

    const alpha = Math.max(0, Math.min(1, style.fill_opacity ?? 0.85));
    const out: CurveGeometry[] = [];
    for (const [cls, tris] of trisByClass) {
      const fillData = new Float32Array(tris);
      const fillVertexBuffer = this.device.createBuffer({
        size: fillData.byteLength,
        usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
      });
      this.device.queue.writeBuffer(fillVertexBuffer, 0, fillData);

      const rgba = hexToRgba(faciesColor(cls));
      rgba[3] = alpha;
      const colorBuffer = this.device.createBuffer({
        size: 16,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
      });
      this.device.queue.writeBuffer(colorBuffer, 0, new Float32Array(rgba));

      const bindGroup = this.device.createBindGroup({
        layout: this.bindGroupLayout,
        entries: [
          { binding: 0, resource: { buffer: this.uniformBuffer } },
          { binding: 1, resource: { buffer: colorBuffer } },
        ],
      });

      out.push({
        curveName: seriesKey,
        // No line geometry for block curves — the line pass draws 0 vertices.
        vertexBuffer: this.device.createBuffer({ size: 8, usage: GPUBufferUsage.VERTEX }),
        vertexCount: 0,
        hidden: false,
        bindGroup,
        fillVertexBuffer,
        fillVertexCount: fillData.length / 2,
        fillBindGroup: bindGroup,
      });
    }
    return out;
  }

  /** Finds the reference curve for `fill: "curve"` shading. The reference must be another
   *  curve in the SAME track, because its own min/max is what positions it — that
   *  compatible scaling is the entire meaning of a neutron-density crossover. */
  private resolveCrossover(
    track: Track,
    style: CurveStyle,
  ): { series: TrackCurveSeries; min: number; max: number } | null {
    const to = style.fill_to?.trim().toUpperCase();
    if (!to) return null;
    const matches = track.curves.filter((c) => c.curve_name.trim().toUpperCase() === to);
    const refStyle =
      matches.find((candidate) => (candidate.set_name?.trim() || null) === (style.set_name?.trim() || null)) ??
      matches[0];
    if (!refStyle) return null;
    const series = this.seriesByName.get(trackCurveKey(refStyle));
    if (!series) return null;
    return { series, min: refStyle.min, max: refStyle.max };
  }

  /** Crossover shading (fill: "curve"): the area between this curve and a reference curve,
   *  coloured by which side this curve is on. Two fill-only geometries — one per side —
   *  because the fill pipeline binds a single colour uniform per draw, the same split
   *  buildBlockGeometries uses. Where the pair crosses inside a sample interval the quad is
   *  split at the crossing so the colours meet exactly on the crossover. A NaN on either
   *  curve leaves that interval unshaded; separation is never inferred across a gap. */
  private buildCrossoverGeometries(
    series: TrackCurveSeries,
    style: CurveStyle,
    reference: { series: TrackCurveSeries; min: number; max: number },
    scaleType: "linear" | "log",
    trackLeftNdc: number,
    trackRightNdc: number,
    seriesKey: string,
  ): CurveGeometry[] {
    const toNdc = (v: number, min: number, max: number): number => {
      let frac: number;
      if (scaleType === "log") {
        const logMin = Math.log10(Math.max(min, 1e-6));
        const logMax = Math.log10(Math.max(max, 1e-6));
        frac = (Math.log10(Math.max(v, 1e-6)) - logMin) / (logMax - logMin);
      } else {
        frac = (v - min) / (max - min);
      }
      frac = Math.max(0, Math.min(1, frac));
      return trackLeftNdc + frac * (trackRightNdc - trackLeftNdc);
    };

    const sampleRef = makeSampler(reference.series);
    const step = style.draw_style === "step";
    const leftTris: number[] = [];
    const rightTris: number[] = [];
    const quad = (
      bucket: number[],
      xa0: number, xb0: number, d0: number,
      xa1: number, xb1: number, d1: number,
    ): void => {
      bucket.push(xa0, d0, xb0, d0, xb1, d1, xa0, d0, xb1, d1, xa1, d1);
    };

    const n = Math.min(series.depth.length, series.value.length);
    for (let i = 0; i < n - 1; i++) {
      const d0 = series.depth[i];
      const d1 = series.depth[i + 1];
      const va0 = series.value[i];
      const vb0 = sampleRef(d0);
      if (Number.isNaN(va0) || Number.isNaN(vb0)) continue;
      const a0 = toNdc(va0, style.min, style.max);
      const b0 = toNdc(vb0, reference.min, reference.max);
      let a1 = a0;
      let b1 = b0;
      if (!step) {
        // A stepped curve holds its value across the interval, so both edges stay vertical
        // and the pair can never cross inside one interval.
        const va1 = series.value[i + 1];
        const vb1 = sampleRef(d1);
        if (Number.isNaN(va1) || Number.isNaN(vb1)) continue;
        a1 = toNdc(va1, style.min, style.max);
        b1 = toNdc(vb1, reference.min, reference.max);
      }
      const s0 = a0 - b0;
      const s1 = a1 - b1;
      if (s0 < 0 !== s1 < 0 && s0 !== s1) {
        const t = s0 / (s0 - s1);
        const dm = d0 + (d1 - d0) * t;
        const xm = a0 + (a1 - a0) * t;
        (s0 < 0 ? leftTris : rightTris).push(a0, d0, b0, d0, xm, dm);
        (s1 < 0 ? leftTris : rightTris).push(xm, dm, b1, d1, a1, d1);
      } else {
        quad(s0 < 0 ? leftTris : rightTris, a0, b0, d0, a1, b1, d1);
      }
    }

    const alpha = Math.max(0, Math.min(1, style.fill_opacity ?? 0.3));
    const leftColor = style.fill_color ?? style.color;
    const out: CurveGeometry[] = [];
    for (const [tris, hex] of [
      [leftTris, leftColor],
      [rightTris, style.fill_color2 ?? leftColor],
    ] as [number[], string][]) {
      if (tris.length === 0) continue;
      const fillData = new Float32Array(tris);
      const fillVertexBuffer = this.device.createBuffer({
        size: fillData.byteLength,
        usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
      });
      this.device.queue.writeBuffer(fillVertexBuffer, 0, fillData);

      const rgba = hexToRgba(hex);
      rgba[3] = alpha;
      const colorBuffer = this.device.createBuffer({
        size: 16,
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
      });
      this.device.queue.writeBuffer(colorBuffer, 0, new Float32Array(rgba));

      const bindGroup = this.device.createBindGroup({
        layout: this.bindGroupLayout,
        entries: [
          { binding: 0, resource: { buffer: this.uniformBuffer } },
          { binding: 1, resource: { buffer: colorBuffer } },
        ],
      });

      out.push({
        curveName: seriesKey,
        // Shading only — the curve's own line geometry is built separately.
        vertexBuffer: this.device.createBuffer({ size: 8, usage: GPUBufferUsage.VERTEX }),
        vertexCount: 0,
        hidden: false,
        bindGroup,
        fillVertexBuffer,
        fillVertexCount: fillData.length / 2,
        fillBindGroup: bindGroup,
      });
    }
    return out;
  }

  /** Sets the vertical scale directly (px per depth unit). Prefer setScaleRatio for "1:N". */
  setScale(pxPerUnit: number): void {
    this.view.pxPerUnit = Math.min(maxPxPerUnit(), Math.max(MIN_PX_PER_UNIT, pxPerUnit));
    this.dirty = true;
    this.onViewSettled?.();
  }

  /** Sets a true print-style vertical scale of 1:ratio (e.g. 200 → 1:200). */
  setScaleRatio(ratio: number): void {
    if (ratio > 0 && Number.isFinite(ratio)) this.setScale(pxPerUnit1to1() / ratio);
  }

  /** The current true vertical scale as the N in "1:N" (derived from the live pxPerUnit),
   *  so the UI can show the real scale after a zoom instead of a stale preset. */
  getScaleRatio(): number {
    return pxPerUnit1to1() / this.view.pxPerUnit;
  }

  /** Multiplies the current scale by `factor`, re-centering on the currently visible midpoint. */
  stepZoom(factor: number): void {
    this.zoomAt(this.canvas.clientHeight / 2 || 0.5, factor);
  }

  /** Zooms by `factor` while keeping the depth under `pixelY` (canvas-Y) fixed — the
   *  natural "zoom toward the cursor" gesture used by Ctrl+scroll. */
  zoomAt(pixelY: number, factor: number): void {
    const anchorDepth = this.view.topDepth + pixelY / this.view.pxPerUnit;
    this.view.pxPerUnit = Math.min(maxPxPerUnit(), Math.max(MIN_PX_PER_UNIT, this.view.pxPerUnit * factor));
    this.view.topDepth = anchorDepth - pixelY / this.view.pxPerUnit;
    this.dirty = true;
    this.onViewSettled?.();
  }

  resetView(): void {
    this.view.topDepth = this.depthMin;
    this.view.pxPerUnit = defaultPxPerUnit();
    this.dirty = true;
    this.onViewSettled?.();
  }

  /** Pans (keeping the current scale) so `depth` sits near the top of the viewport —
   *  used when a formation top is selected in the Wells & Tops pane. */
  scrollToDepth(depth: number): void {
    const height = this.canvas.clientHeight || 1;
    this.view.topDepth = depth - 0.08 * (height / this.view.pxPerUnit);
    this.dirty = true;
    this.onViewSettled?.();
  }

  getScale(): number {
    return this.view.pxPerUnit;
  }

  /** Full data depth range (not the currently visible viewport), for the report header. */
  getDataDepthRange(): [number, number] {
    return [this.depthMin, this.depthMax];
  }

  private attachPanHandlers(): void {
    let dragging = false;
    let lastY = 0;
    let settleHandle: number | undefined;

    const scheduleSettled = () => {
      if (settleHandle !== undefined) window.clearTimeout(settleHandle);
      settleHandle = window.setTimeout(() => this.onViewSettled?.(), 150);
    };

    // Both drag and scroll only pan depth — zoom is a deliberate action via the vertical
    // scale selector / zoom buttons, matching how real log-plot software separates the two.
    this.canvas.addEventListener("pointerdown", (e) => {
      dragging = true;
      lastY = e.clientY;
    });
    // A drag must keep panning when the pointer leaves the canvas, so these live on
    // `window` — which means they survive the canvas being removed on panel close. Keep
    // named references and remove them in dispose(), or every closed log view leaks one
    // pointerup + one pointermove that keep firing (and pin this renderer) for the app's life.
    const onWindowPointerUp = () => {
      dragging = false;
    };
    const onWindowPointerMove = (e: PointerEvent) => {
      if (!dragging) return;
      const dy = e.clientY - lastY;
      lastY = e.clientY;
      this.view.topDepth -= dy / this.view.pxPerUnit;
      this.dirty = true;
      scheduleSettled();
    };
    window.addEventListener("pointerup", onWindowPointerUp);
    window.addEventListener("pointermove", onWindowPointerMove);
    this.cleanups.push(() => {
      window.removeEventListener("pointerup", onWindowPointerUp);
      window.removeEventListener("pointermove", onWindowPointerMove);
      if (settleHandle !== undefined) window.clearTimeout(settleHandle);
    });
    this.canvas.addEventListener(
      "wheel",
      (e) => {
        e.preventDefault();
        if (e.ctrlKey) {
          // Ctrl+scroll = zoom the depth scale toward the cursor (up = zoom in).
          const rect = this.canvas.getBoundingClientRect();
          this.zoomAt(e.clientY - rect.top, e.deltaY < 0 ? 1.15 : 1 / 1.15);
        } else {
          // Plain scroll pans depth.
          this.view.topDepth += e.deltaY / this.view.pxPerUnit;
          this.dirty = true;
          scheduleSettled();
        }
      },
      { passive: false },
    );
  }

  /** Converts a pixel-Y offset within the canvas to a depth. */
  private pixelYToDepth(pixelY: number): number {
    return this.view.topDepth + pixelY / this.view.pxPerUnit;
  }

  /** Returns the [top, bottom] depth currently visible in the canvas, for the depth axis. */
  getVisibleDepthRange(): [number, number] {
    const height = this.canvas.clientHeight || 1;
    return [this.pixelYToDepth(0), this.pixelYToDepth(height)];
  }

  /** Horizontal extent of every track as canvas-width fractions, for overlay drawing. */
  getTrackRanges(): { title: string; leftFrac: number; rightFrac: number }[] {
    return this.trackRanges.map((t) => ({ ...t }));
  }

  /** The track whose horizontal extent contains the given canvas-X fraction. */
  private trackAtFrac(xFrac: number): string | null {
    for (const t of this.trackRanges) {
      if (xFrac >= t.leftFrac && xFrac < t.rightFrac) return t.title;
    }
    return null;
  }

  private attachCursorHandler(): void {
    this.canvas.addEventListener("pointermove", (e) => {
      const rect = this.canvas.getBoundingClientRect();
      const pixelY = e.clientY - rect.top;
      const depth = this.pixelYToDepth(pixelY);
      const xFrac = rect.width > 0 ? (e.clientX - rect.left) / rect.width : -1;
      const trackTitle = this.trackAtFrac(xFrac);

      if (depth < this.depthMin || depth > this.depthMax) {
        this.onCursorMove?.(null, [], trackTitle);
        return;
      }

      const samples: ReadoutSample[] = [];
      for (const [name, series] of this.seriesByName) {
        samples.push({ curveName: name, value: nearestValue(series, depth) });
      }
      this.onCursorMove?.(depth, samples, trackTitle);
    });
    this.canvas.addEventListener("pointerleave", () => this.onCursorMove?.(null, [], null));
  }

  /** Requests a redraw on the next frame with the data already loaded — used when only
   *  presentation changed (e.g. theme colors, read from CSS vars at render time). */
  repaint(): void {
    this.cachedBgColor = null; // theme may have changed — re-read --bg-panel next frame
    this.dirty = true;
  }

  private startLoop(): void {
    if (this.running) return;
    this.running = true;
    const frame = () => {
      if (!this.running) return;
      if (this.dirty) {
        this.render();
        this.dirty = false;
        this.onFrameRendered?.();
      }
      requestAnimationFrame(frame);
    };
    requestAnimationFrame(frame);
  }

  stop(): void {
    this.running = false;
  }

  /** Syncs the drawing-buffer size to the element's CSS size after a dock-panel resize.
   *  WebGPU's swapchain picks up the new canvas size on the next getCurrentTexture(). */
  resize(): void {
    const w = this.canvas.clientWidth;
    const h = this.canvas.clientHeight;
    if (w > 0 && h > 0 && (this.canvas.width !== w || this.canvas.height !== h)) {
      this.canvas.width = w;
      this.canvas.height = h;
    }
    this.dirty = true;
  }

  /** Stops the loop and releases GPU resources (called when a dock panel closes). */
  dispose(): void {
    this.stop();
    for (const c of this.cleanups) c();
    this.cleanups = [];
    for (const c of this.curves) {
      c.vertexBuffer.destroy();
      c.fillVertexBuffer?.destroy();
    }
    this.curves = [];
    this.device?.destroy();
  }

  private render(): void {
    const heightPx = this.canvas.clientHeight || 1;
    const uniformData = new Float32Array([this.view.topDepth, this.view.pxPerUnit, heightPx, 0]);
    this.device.queue.writeBuffer(this.uniformBuffer, 0, uniformData);

    const encoder = this.device.createCommandEncoder();
    if (this.cachedBgColor === null) {
      // Detached canvases (inactive dockview tab) report empty computed styles — fall back
      // to the root element so the clear color tracks the active theme, not the light default.
      this.cachedBgColor =
        getComputedStyle(this.canvas).getPropertyValue("--bg-panel").trim() ||
        getComputedStyle(document.documentElement).getPropertyValue("--bg-panel").trim() ||
        "#fbf7ee";
    }
    const clear = hexToRgba(this.cachedBgColor);
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          view: this.context.getCurrentTexture().createView(),
          clearValue: { r: clear[0], g: clear[1], b: clear[2], a: 1 },
          loadOp: "clear",
          storeOp: "store",
        },
      ],
    });
    // Fills first, so curve lines draw on top of the shading.
    pass.setPipeline(this.fillPipeline);
    for (const curve of this.curves) {
      if (curve.hidden || !curve.fillVertexBuffer || !curve.fillBindGroup) continue;
      pass.setBindGroup(0, curve.fillBindGroup);
      pass.setVertexBuffer(0, curve.fillVertexBuffer);
      pass.draw(curve.fillVertexCount!);
    }

    pass.setPipeline(this.pipeline);
    for (const curve of this.curves) {
      if (curve.hidden) continue;
      pass.setBindGroup(0, curve.bindGroup);
      pass.setVertexBuffer(0, curve.vertexBuffer);
      pass.draw(curve.vertexCount);
    }

    pass.end();
    this.device.queue.submit([encoder.finish()]);
  }
}

/** Interpolates a series onto arbitrary depths for crossover shading. Two curves in one
 *  track need not share a sampling — a 0.1 m image-derived curve and a 0.5 m wireline curve
 *  routinely do not — so the reference is read at the styled curve's own depths. Returns NaN
 *  outside the reference's depth range or across a NaN gap, so shading stops where the
 *  reference genuinely stops rather than being extrapolated. The cursor walks forward and
 *  only restarts when the caller sweeps backwards, keeping a full pass linear. */
function makeSampler(series: TrackCurveSeries): (depth: number) => number {
  const d = series.depth;
  const v = series.value;
  const n = Math.min(d.length, v.length);
  let i = 0;
  return (depth: number): number => {
    if (n === 0 || depth < d[0] || depth > d[n - 1]) return NaN;
    if (i > n - 2 || d[i] > depth) i = 0;
    while (i < n - 2 && d[i + 1] < depth) i++;
    const d0 = d[i];
    const d1 = d[i + 1];
    const v0 = v[i];
    const v1 = v[i + 1];
    if (Number.isNaN(v0) || Number.isNaN(v1)) return NaN;
    return d1 === d0 ? v0 : v0 + ((v1 - v0) * (depth - d0)) / (d1 - d0);
  };
}

function nearestValue(series: TrackCurveSeries, depth: number): number {
  const depths = series.depth;
  const n = depths.length;
  if (n === 0) return NaN;
  // depths are sorted ascending (LAS index) — binary-search the insertion point instead of
  // the old O(n) per-pointermove scan, then pick the closer of the two neighbours. Ties go to
  // the earlier index, matching the old strict `dist < closestDist`, so the value is identical.
  let lo = 0;
  let hi = n - 1;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (depths[mid] < depth) lo = mid + 1;
    else hi = mid;
  }
  if (lo > 0 && Math.abs(depths[lo - 1] - depth) <= Math.abs(depths[lo] - depth)) {
    return series.value[lo - 1];
  }
  return series.value[lo];
}

function hexToRgba(hex: string): [number, number, number, number] {
  const clean = hex.trim().replace("#", "");
  if (clean.length < 6) return [0.05, 0.05, 0.07, 1];
  const r = parseInt(clean.substring(0, 2), 16) / 255;
  const g = parseInt(clean.substring(2, 4), 16) / 255;
  const b = parseInt(clean.substring(4, 6), 16) / 255;
  return [r, g, b, 1];
}
