import { runQuery, type TablePage } from "../ipc";
import { setStatus } from "../state";

const STARTER = `-- Full DuckDB SQL over the project (read-only).
-- Tables: wells, standard_curves, computed_curves, tops, zones, zone_params, equations, documents
SELECT w.well_name,
       COUNT(*)            AS samples,
       ROUND(AVG(s.gr), 1) AS avg_gr,
       ROUND(MIN(s.depth)) AS top,
       ROUND(MAX(s.depth)) AS bottom
FROM standard_curves s
JOIN wells w USING (well_id)
GROUP BY w.well_name
ORDER BY w.well_name`;

/** Read-only SQL console over the project database — a SQL console's role, but with the
 *  whole of DuckDB SQL (joins, window functions, aggregates, QUALIFY, PIVOT...). */
export class SqlQueryPanel {
  constructor(host: HTMLElement) {
    const root = document.createElement("div");
    root.className = "dbinspector"; // same column layout as the DB inspector
    root.innerHTML = `
      <textarea class="sql-input" spellcheck="false"></textarea>
      <div class="dbi-toolbar">
        <button class="lp-btn primary sql-run">Run (Ctrl+Enter)</button>
        <span class="dbi-pageinfo sql-info"></span>
      </div>
      <div class="dbi-grid sql-grid"><div class="placeholder-note">Results appear here.</div></div>`;
    host.appendChild(root);

    const input = root.querySelector<HTMLTextAreaElement>(".sql-input")!;
    const runBtn = root.querySelector<HTMLButtonElement>(".sql-run")!;
    const info = root.querySelector<HTMLElement>(".sql-info")!;
    const grid = root.querySelector<HTMLElement>(".sql-grid")!;
    input.value = STARTER;

    const run = async () => {
      const sql = input.value.trim();
      if (!sql) return;
      info.textContent = "running…";
      try {
        const page = await runQuery(sql, 1000);
        renderResult(grid, page);
        info.textContent = `${page.total_rows} row(s)`;
      } catch (err) {
        grid.innerHTML = `<div class="placeholder-note">${String(err)}</div>`;
        info.textContent = "";
        setStatus(`Query failed: ${err}`);
      }
    };
    runBtn.addEventListener("click", () => void run());
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && e.ctrlKey) {
        e.preventDefault();
        void run();
      }
      e.stopPropagation(); // keep global undo hotkeys out of the editor
    });
  }
}

function renderResult(grid: HTMLElement, page: TablePage): void {
  const table = document.createElement("table");
  table.className = "dbgrid";
  const thead = document.createElement("thead");
  const headRow = document.createElement("tr");
  for (const col of page.columns) {
    const th = document.createElement("th");
    th.textContent = col;
    headRow.appendChild(th);
  }
  thead.appendChild(headRow);
  table.appendChild(thead);
  const tbody = document.createElement("tbody");
  for (const row of page.rows) {
    const tr = document.createElement("tr");
    for (const cell of row) {
      const td = document.createElement("td");
      td.textContent = cell ?? "";
      if (cell === null) td.classList.add("null");
      tr.appendChild(td);
    }
    tbody.appendChild(tr);
  }
  table.appendChild(tbody);
  grid.innerHTML = "";
  grid.appendChild(table);
}
