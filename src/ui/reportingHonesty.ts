import type { MlWellResult } from "../ipc";
import { recordProcess } from "../processLog";

export interface MlWriteReportArgs {
  statusLine: Pick<HTMLElement, "textContent">;
  setStatus: (text: string) => void;
  algorithmLabel: string;
  outputs: string[];
  wells: MlWellResult[];
  fallbackTotal: number;
  elapsedMs: number;
}

/** Writes the visible and persistent outcome of an ML run from successful WELL results,
 *  never from the requested scope. An all-failed run deliberately gets no success-history entry. */
export function reportMlWriteOutcome(args: MlWriteReportArgs): { ok: number; total: number } {
  const total = args.wells.length || args.fallbackTotal;
  const ok = args.wells.filter((well) => !well.error).length;
  const outs = args.outputs.join(", ");
  const scope = ok === total ? `${total} well(s)` : `${ok}/${total} well(s)`;
  const needAttention = total - ok;
  args.statusLine.textContent =
    `Done in ${args.elapsedMs} ms → ${outs}` +
    (needAttention > 0 ? ` — ${needAttention} well(s) need attention` : "");
  const status = `${args.algorithmLabel}: wrote ${outs} to ${scope}`;
  args.setStatus(status);
  if (ok > 0) recordProcess("ML", status);
  return { ok, total };
}

/** Reports the Field Dashboard's read-only completion without implying that its returned
 *  statistics persisted FLAG curves. */
export function reportDashboardCompletion(
  status: Pick<HTMLElement, "textContent">,
  wellCount: number,
  rowCount: number,
  flagCount: number,
): void {
  status.textContent =
    `${wellCount} well(s) · ${rowCount} zone-rows across ${flagCount} flag level(s). ` +
    "Stats only — no FLAG curves written; run Cutoffs & Summary to persist flags.";
}
