import { appState, type BrushSelection, type TopInterval } from "../state";

/** Every event that can make a quantitative plot stale without changing its panel identity. */
export const PLOT_INVALIDATION_KINDS = [
  "theme",
  "dataRevision",
  "interval",
  "selection",
  "size",
] as const;

export interface PlotSize {
  width: number;
  height: number;
}

/** appState observables are current-value sources: subscribe synchronously supplies one snapshot. */
export interface CurrentValueSource<T> {
  subscribe(listener: (value: T) => void): () => void;
}

export interface PlotInvalidationSources {
  theme: CurrentValueSource<number>;
  dataRevision: CurrentValueSource<number>;
  interval: CurrentValueSource<TopInterval | null>;
  selection: CurrentValueSource<BrushSelection | null>;
  size: CurrentValueSource<PlotSize>;
}

export interface PlotInvalidationHandlers {
  theme(version: number): void;
  dataRevision(version: number): void;
  interval(interval: TopInterval | null): void;
  selection(selection: BrushSelection | null): void;
  size(size: PlotSize): void;
  /** Invalidates generation tokens and cancels queued frames/timers owned by the panel. */
  cancelPending(): void;
}

export interface PlotInvalidationContract {
  dispose(): void;
}

/**
 * Subscribe one plot to the complete invalidation vocabulary.
 *
 * Each source's synchronous current snapshot is deliberately swallowed: a panel initializes from
 * that snapshot while it builds, and only later changes invalidate it. This prevents one builder
 * from double-fetching at construction while another silently chooses different initial semantics.
 */
export function subscribePlotInvalidationContract(
  sources: PlotInvalidationSources,
  handlers: PlotInvalidationHandlers,
): PlotInvalidationContract {
  let disposed = false;
  const subscribeChanges = <T>(source: CurrentValueSource<T>, handler: (value: T) => void): (() => void) => {
    let currentSnapshot = true;
    return source.subscribe((value) => {
      if (currentSnapshot) {
        currentSnapshot = false;
        return;
      }
      if (!disposed) handler(value);
    });
  };
  const unsubscribers = [
    subscribeChanges(sources.theme, handlers.theme),
    subscribeChanges(sources.dataRevision, handlers.dataRevision),
    subscribeChanges(sources.interval, handlers.interval),
    subscribeChanges(sources.selection, handlers.selection),
    subscribeChanges(sources.size, handlers.size),
  ];

  return {
    dispose(): void {
      if (disposed) return;
      disposed = true;
      for (const unsubscribe of unsubscribers) unsubscribe();
      handlers.cancelPending();
    },
  };
}

/** A ResizeObserver presented as the same current-value source used by application state. */
function elementSizeSource(target: HTMLElement): CurrentValueSource<PlotSize> {
  return {
    subscribe(listener): () => void {
      let current: PlotSize = { width: target.clientWidth, height: target.clientHeight };
      listener(current);
      let frame = 0;
      const observer = new ResizeObserver(() => {
        const next = { width: target.clientWidth, height: target.clientHeight };
        if (next.width === current.width && next.height === current.height) return;
        current = next;
        if (frame) return;
        // The frame reads `current` when it RUNS, so a drag that fires the observer twenty times
        // coalesces to the last size rather than the one that opened the frame - which is the
        // whole point of deferring. A second variable held in lockstep with this one said the
        // two could differ; they never could, and a reader had to prove that for themselves.
        frame = requestAnimationFrame(() => {
          frame = 0;
          listener(current);
        });
      });
      observer.observe(target);
      return () => {
        observer.disconnect();
        if (frame) cancelAnimationFrame(frame);
      };
    },
  };
}

/** Register a live panel against the one application invalidation contract. */
export function registerPlotInvalidationContract(
  sizeTarget: HTMLElement,
  handlers: PlotInvalidationHandlers,
): PlotInvalidationContract {
  return subscribePlotInvalidationContract(
    {
      theme: appState.themeVersion,
      dataRevision: appState.dataVersion,
      interval: appState.selectedInterval,
      selection: appState.brushedDepths,
      size: elementSizeSource(sizeTarget),
    },
    handlers,
  );
}
