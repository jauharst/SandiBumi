/**
 * Source-owned inventory of asynchronous operations that can replace or refetch a live plot.
 *
 * Keep this list aligned with the five plot surfaces named by SB-PLT-029's owning chapter and
 * T27's log-view viewport refetch. A new asynchronous plot load is incomplete until it appears
 * here and creates its token at the actual await boundary.
 */
export const PLOT_ASYNC_LOAD_REGISTRY = [
  { id: "workspace-plot-build", owner: "src/ui/workspace.ts" },
  { id: "histogram-data-refetch", owner: "src/ui/histogramPanel.ts" },
  { id: "histogram-context-refetch", owner: "src/ui/histogramPanel.ts" },
  { id: "crossplot-data-refetch", owner: "src/ui/crossplotPanel.ts" },
  { id: "crossplot-core-refetch", owner: "src/ui/crossplotPanel.ts" },
  { id: "crossplot-context-refetch", owner: "src/ui/crossplotPanel.ts" },
  { id: "pickett-data-refetch", owner: "src/ui/pickettPanel.ts" },
  { id: "pickett-context-refetch", owner: "src/ui/pickettPanel.ts" },
  { id: "correlation-data-refetch", owner: "src/ui/correlationPanel.ts" },
  { id: "correlation-well-refetch", owner: "src/ui/correlationPanel.ts" },
  { id: "vega-data-refetch", owner: "src/ui/vegaPanel.ts" },
  { id: "vega-selector-refetch", owner: "src/ui/vegaPanel.ts" },
  { id: "vega-editor-load", owner: "src/ui/vegaPanel.ts" },
  { id: "vega-resize", owner: "src/ui/vegaPanel.ts" },
  { id: "logview-viewport-refetch", owner: "src/ui/viewportRefetch.ts" },
] as const;

export type PlotAsyncOperationId = (typeof PLOT_ASYNC_LOAD_REGISTRY)[number]["id"];

export interface PlotAsyncGenerationToken {
  readonly operation: PlotAsyncOperationId;
  readonly generation: number;
}

export type PlotAsyncCommitOutcome = "applied" | "stale";

/** Create the explicit, immutable identity that must cross an asynchronous plot boundary. */
export function beginPlotAsyncGeneration(
  operation: PlotAsyncOperationId,
  generation: number,
): PlotAsyncGenerationToken {
  if (!Number.isSafeInteger(generation) || generation < 1) {
    throw new Error(`${operation} requires a positive safe-integer generation`);
  }
  return Object.freeze({ operation, generation });
}

/** A token is current only while both its generation and its owning panel are current. */
export function isPlotAsyncGenerationCurrent(
  token: PlotAsyncGenerationToken,
  currentGeneration: number,
  disposed = false,
): boolean {
  return !disposed && token.generation === currentGeneration;
}

/**
 * Commit one asynchronously produced value. Stale disposable results are torn down before the
 * active callback can run, so callers cannot accidentally put disposal after panel mutation.
 */
export function commitPlotAsyncGeneration<T>(
  token: PlotAsyncGenerationToken,
  currentGeneration: number,
  disposed: boolean,
  value: T,
  handlers: {
    apply(value: T): void;
    disposeStale?(value: T): void;
  },
): PlotAsyncCommitOutcome {
  if (!isPlotAsyncGenerationCurrent(token, currentGeneration, disposed)) {
    handlers.disposeStale?.(value);
    return "stale";
  }
  handlers.apply(value);
  return "applied";
}
