import { healthSnapshot, type HealthSnapshot } from "../ipc";

/** Performance Monitor (Phase 11): a compact Petrel-PHM-style panel of four resource gauges —
 *  CPU, MEM System, USER Objects, GDI Objects — colour-coded green/yellow/red so a resource
 *  leak or CPU/memory-pressure state is obvious at a glance. Polls a cheap backend snapshot on
 *  a timer and updates the bars in place (no rebuild → no flicker). Metrics are Windows-only;
 *  where a value is unavailable the gauge shows "n/a". */
export async function buildHealthContent(
  _setStatus: (text: string) => void,
): Promise<{ el: HTMLElement; dispose: () => void }> {
  const el = document.createElement("div");
  el.className = "health-panel";

  interface Gauge {
    key: "mem_system" | "cpu_load" | "user_objects" | "gdi_objects";
    label: string;
    hint: string;
  }
  const GAUGES: Gauge[] = [
    { key: "cpu_load", label: "CPU", hint: "Total CPU utilisation across all cores" },
    { key: "mem_system", label: "MEM System", hint: "System memory in use" },
    { key: "user_objects", label: "USER Objects", hint: "This process's USER handles vs the 10,000 per-process ceiling" },
    { key: "gdi_objects", label: "GDI Objects", hint: "This process's GDI handles vs the 10,000 per-process ceiling" },
  ];

  const rows = new Map<Gauge["key"], { fill: HTMLElement; val: HTMLElement }>();
  for (const g of GAUGES) {
    const row = document.createElement("div");
    row.className = "health-row";
    const label = document.createElement("span");
    label.className = "health-label";
    label.textContent = g.label;
    label.title = g.hint;
    const barWrap = document.createElement("div");
    barWrap.className = "health-bar-wrap";
    const fill = document.createElement("div");
    fill.className = "health-bar-fill";
    barWrap.appendChild(fill);
    const val = document.createElement("span");
    val.className = "health-val";
    val.textContent = "…";
    row.append(label, barWrap, val);
    el.appendChild(row);
    rows.set(g.key, { fill, val });
  }

  const note = document.createElement("div");
  note.className = "health-note";
  note.textContent = "Green < 60% · Yellow 60–80% · Red > 80%";
  el.appendChild(note);

  const level = (v: number): string => (v >= 80 ? "health-red" : v >= 60 ? "health-yellow" : "health-green");

  async function refresh(): Promise<void> {
    const s: HealthSnapshot | null = await healthSnapshot().catch(() => null);
    for (const g of GAUGES) {
      const r = rows.get(g.key)!;
      const v = s ? s[g.key] : null;
      if (v == null || !Number.isFinite(v)) {
        r.fill.style.width = "0%";
        r.fill.className = "health-bar-fill";
        r.val.textContent = "n/a";
        continue;
      }
      const pct = Math.max(0, Math.min(100, v));
      r.fill.style.width = `${pct}%`;
      r.fill.className = `health-bar-fill ${level(v)}`;
      let extra = "";
      if (g.key === "user_objects" && s?.user_count != null) extra = ` (${s.user_count})`;
      if (g.key === "gdi_objects" && s?.gdi_count != null) extra = ` (${s.gdi_count})`;
      r.val.textContent = `${v.toFixed(0)}%${extra}`;
    }
  }

  await refresh();
  // Health changes slowly; 1.5s keeps it live without wasting IPC.
  const timer = window.setInterval(() => void refresh(), 1500);
  return { el, dispose: () => window.clearInterval(timer) };
}
