import { beginPlotAsyncGeneration, isPlotAsyncGenerationCurrent, type PlotAsyncOperationId } from "./plotAsync";

export interface ViewportLoadIdentity {
  sourceKey: string;
  low: number;
  high: number;
  targetPixelHeight: number;
}

export interface TaggedViewportLoad extends ViewportLoadIdentity {
  operation: PlotAsyncOperationId;
  generation: number;
}

export type ViewportRefetchOutcome = "loaded" | "pending" | "applied" | "stale" | "failed";

function valid(identity: ViewportLoadIdentity): boolean {
  return (
    identity.sourceKey.length > 0 &&
    Number.isFinite(identity.low) &&
    Number.isFinite(identity.high) &&
    identity.high > identity.low &&
    Number.isFinite(identity.targetPixelHeight) &&
    identity.targetPixelHeight > 0
  );
}

function signature(identity: ViewportLoadIdentity): string {
  return JSON.stringify([
    identity.sourceKey,
    identity.low,
    identity.high,
    identity.targetPixelHeight,
  ]);
}

/**
 * Owns the disposable data identity behind one depth viewport. The interval is always
 * half-open. A view that leaves the loaded interval, or asks for finer source density,
 * gets a new generation; only that generation may replace the displayed series.
 */
export class ViewportRefetchCoordinator<T> {
  private generation = 0;
  private loaded: ViewportLoadIdentity | null = null;
  private pendingSignature = "";

  reset(): void {
    this.generation += 1;
    this.loaded = null;
    this.pendingSignature = "";
  }

  seedLoaded(identity: ViewportLoadIdentity): void {
    if (!valid(identity)) return;
    this.loaded = { ...identity };
    this.pendingSignature = "";
  }

  private needsRefetch(request: ViewportLoadIdentity): boolean {
    const loaded = this.loaded;
    if (!loaded || loaded.sourceKey !== request.sourceKey) return true;
    if (request.low < loaded.low || request.high > loaded.high) return true;

    const loadedUnitsPerPixel = (loaded.high - loaded.low) / loaded.targetPixelHeight;
    const requestedUnitsPerPixel = (request.high - request.low) / request.targetPixelHeight;
    return requestedUnitsPerPixel < loadedUnitsPerPixel * (1 - Number.EPSILON * 8);
  }

  async refetch(
    request: ViewportLoadIdentity,
    load: (request: TaggedViewportLoad) => Promise<T>,
    apply: (value: T, request: TaggedViewportLoad) => void,
    reportPending: (message: string, request: TaggedViewportLoad) => void,
    reportFailure: (message: string, error: unknown) => void,
  ): Promise<ViewportRefetchOutcome> {
    if (!valid(request)) return "failed";
    if (!this.needsRefetch(request)) return "loaded";

    const requestSignature = signature(request);
    if (requestSignature === this.pendingSignature) return "pending";

    const token = beginPlotAsyncGeneration("logview-viewport-refetch", ++this.generation);
    const tagged: TaggedViewportLoad = { ...request, ...token };
    this.pendingSignature = requestSignature;
    try {
      reportPending(
        `Loading detailed samples for depth [${request.low}, ${request.high}). ` +
          "Existing samples remain visible until the refresh completes.",
        tagged,
      );
      const value = await load(tagged);
      if (!isPlotAsyncGenerationCurrent(token, this.generation)) return "stale";
      apply(value, tagged);
      this.loaded = { ...request };
      this.pendingSignature = "";
      return "applied";
    } catch (error) {
      if (!isPlotAsyncGenerationCurrent(token, this.generation)) return "stale";
      this.pendingSignature = "";
      reportFailure(
        `Could not load detailed samples for depth [${request.low}, ${request.high}). ` +
          "Existing samples remain on screen; pan or zoom to retry.",
        error,
      );
      return "failed";
    }
  }
}
