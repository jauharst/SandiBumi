/**
 * AUDIT-2026-08-20 finding 68: the frontend's unreferenced exports, owned.
 *
 * The Rust side inventories every warning and every ignored test with an owner, a gate and a
 * rationale (`gate2-hygiene.mjs`). The TypeScript side had nothing equivalent: `noUnusedLocals`
 * does not apply to exports, there is no knip/ts-prune, and the acceptance suite references
 * none of them — so an export with no caller passed every gate silently. Twelve had
 * accumulated, seven of them IPC wrappers, which means seven working backend commands that
 * nothing in the app can reach.
 *
 * DELETING those wrappers would make the capability further out of reach, not nearer. So this
 * gate does not forbid an unreferenced export; it forbids an UNOWNED one, and it re-checks the
 * claim each entry makes:
 *
 *   BACKEND-ROUTE-PENDING   names a backend command, which must still be registered in
 *                           `lib.rs`. If it is not, the wrapper is dead at BOTH ends and is
 *                           not "pending a route" — it is just dead.
 *   SUPERSEDED-BY-SIBLING   names the live export the app reaches instead, which must exist
 *                           and must itself be referenced.
 *   EXPORTED-FOR-SYMMETRY   a helper published beside used siblings; carries only its reason.
 *
 * The list may SHRINK and never silently grow: an entry whose subject is now referenced, or no
 * longer exists, is a stale row and fails.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const REPO = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const INVENTORY = path.join(
  REPO,
  "docs",
  "takeover",
  "evidence",
  "gate2-frontend-export-inventory.json",
);

/** Top-level `export <kind> <name>` declarations. Re-exports and default exports are not
 *  declarations and are deliberately out of scope. */
const DECLARATION =
  /^export\s+(?:async\s+)?(function|const|let|class|interface|type|enum)\s+([A-Za-z_$][\w$]*)/gm;

/** Every top-level declaration, exported or not — a supersession may point at a module-private
 *  helper, which is as live as an exported one. */
const ANY_DECLARATION =
  /^(?:export\s+)?(?:async\s+)?(?:function|const|let|class|interface|type|enum)\s+([A-Za-z_$][\w$]*)/gm;

function walk(dir, extensions, out = []) {
  if (!fs.existsSync(dir)) return out;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === "node_modules") continue;
      walk(full, extensions, out);
    } else if (extensions.some((extension) => entry.name.endsWith(extension))) {
      out.push(full);
    }
  }
  return out;
}

const relative = (file) => path.relative(REPO, file).replaceAll("\\", "/");

/** Every place a name could be used: the app, its tooling, its end-to-end suite, the shell. */
export function readSurfaces(repo = REPO) {
  const declarationFiles = walk(path.join(repo, "src"), [".ts"]);
  const referenceFiles = [
    ...declarationFiles,
    ...walk(path.join(repo, "tools"), [".mjs", ".js", ".ts"]),
    ...walk(path.join(repo, "e2e"), [".mjs", ".js", ".ts"]),
    path.join(repo, "index.html"),
  ].filter((file) => fs.existsSync(file));
  const sources = new Map();
  for (const file of new Set(referenceFiles)) {
    sources.set(relative(file), fs.readFileSync(file, "utf8"));
  }
  return { declarationFiles: declarationFiles.map(relative), sources };
}

/** Where a declaration's own text stops: the start of the next top-level one. A fixed window
 *  runs into the neighbour's invoke, and brace matching from the first `{` lands inside the
 *  RETURN TYPE of any wrapper declared `Promise<{ ... }>` — which is most of them. */
const NEXT_TOP_LEVEL =
  /^(?:export\b|const\b|let\b|function\b|async\b|class\b|interface\b|type\b|enum\b|\/\*\*)/m;

/** The command a wrapper actually invokes, read from the wrapper's OWN text. */
function invokedCommand(text, declarationIndex) {
  const rest = text.slice(declarationIndex);
  const boundary = rest.slice(1).search(NEXT_TOP_LEVEL);
  const body = boundary < 0 ? rest : rest.slice(0, boundary + 1);
  const match = body.match(/invoke(?:<[^(]*>)?\(\s*"([a-z_0-9]+)"/u);
  return match ? match[1] : null;
}

export function unreferencedExports(surfaces = readSurfaces()) {
  const { declarationFiles, sources } = surfaces;
  const found = [];
  for (const file of declarationFiles) {
    const text = sources.get(file);
    for (const match of text.matchAll(DECLARATION)) {
      const [, kind, name] = match;
      const pattern = new RegExp(`\\b${name.replaceAll("$", "\\$")}\\b`, "g");
      let references = 0;
      for (const [other, otherText] of sources) {
        const count = (otherText.match(pattern) ?? []).length;
        // its own declaration is not a use of it
        references += other === file ? Math.max(0, count - 1) : count;
      }
      if (references > 0) continue;
      found.push({
        key: `${file}|${name}`,
        file,
        kind,
        name,
        command: file === "src/ipc.ts" ? invokedCommand(text, match.index) : null,
      });
    }
  }
  return found.sort((left, right) => left.key.localeCompare(right.key));
}

/** Command names registered with Tauri. A wrapper for anything else reaches nothing. */
export function registeredCommands(repo = REPO) {
  const lib = fs.readFileSync(path.join(repo, "src-tauri", "src", "lib.rs"), "utf8");
  const start = lib.indexOf("generate_handler![");
  if (start < 0) throw new Error("lib.rs has no generate_handler! block to read");
  const block = lib.slice(start, lib.indexOf("])", start));
  return new Set([...block.matchAll(/\b([a-z_][a-z0-9_]*)\b/gu)].map((match) => match[1]));
}

export function validateInventory({ observed, inventory, commands, referencedNames }) {
  const errors = [];
  if (inventory.schema_version !== 1) errors.push("inventory schema version must be 1");

  const entries = (inventory.groups ?? []).flatMap((group) =>
    (group.exports ?? []).map((entry) => ({ ...entry, category: group.category, group })),
  );
  const owned = new Map(entries.map((entry) => [entry.key, entry]));
  const seen = new Map(observed.map((item) => [item.key, item]));

  for (const item of observed) {
    if (!owned.has(item.key)) {
      errors.push(
        `unowned unreferenced export ${item.key}: give it an owner and a reason in ` +
          "gate2-frontend-export-inventory.json, or delete it",
      );
    }
  }
  for (const entry of entries) {
    if (!seen.has(entry.key)) {
      errors.push(
        `stale inventory row ${entry.key}: it is referenced now, or gone. Delete the row — ` +
          "this list may shrink, never go stale",
      );
      continue;
    }
    if (entry.category === "BACKEND-ROUTE-PENDING") {
      if (!entry.backend_command) {
        errors.push(`${entry.key} is BACKEND-ROUTE-PENDING and must name its backend command`);
      } else if (!commands.has(entry.backend_command)) {
        errors.push(
          `${entry.key} claims backend command '${entry.backend_command}', which is not ` +
            "registered in lib.rs: the wrapper is dead at BOTH ends, not pending a route",
        );
      } else if (seen.get(entry.key).command !== entry.backend_command) {
        errors.push(
          `${entry.key} invokes '${seen.get(entry.key).command}' but the inventory names ` +
            `'${entry.backend_command}'`,
        );
      }
    }
    if (entry.category === "SUPERSEDED-BY-SIBLING") {
      // A supersession claim is only worth anything if the sibling is LIVE. Two uses beyond its
      // own declaration is the discriminator: one of them may be the dead export calling it, so
      // a sibling reached only from the corpse fails here rather than reading as reassurance.
      // The sibling need not be exported - `bindingAuditedFamilyDisplayDecision` is module-private
      // and is exactly what the panel reaches.
      const uses = (referencedNames.get(entry.superseded_by) ?? 0) - 1;
      if (!entry.superseded_by) {
        errors.push(`${entry.key} is SUPERSEDED-BY-SIBLING and must name the live sibling`);
      } else if (uses < 0) {
        errors.push(
          `${entry.key} says '${entry.superseded_by}' is what the app reaches instead, but ` +
            "nothing declares that name",
        );
      } else if (uses < 2) {
        errors.push(
          `${entry.key} names '${entry.superseded_by}' as its live replacement, but that has ` +
            `${uses} use(s) beyond its own declaration: the supersession is not live`,
        );
      }
    }
    if (!entry.rationale || entry.rationale.trim().length < 20) {
      errors.push(`${entry.key} needs a rationale that says why it still exists`);
    }
  }
  if (inventory.expected_unreferenced_export_count !== observed.length) {
    errors.push(
      `inventory declares ${inventory.expected_unreferenced_export_count} unreferenced ` +
        `exports but observed ${observed.length}`,
    );
  }
  const counted = new Map();
  for (const entry of entries) counted.set(entry.category, (counted.get(entry.category) ?? 0) + 1);
  for (const [category, expected] of Object.entries(inventory.expected_category_counts ?? {})) {
    if ((counted.get(category) ?? 0) !== expected) {
      errors.push(
        `inventory declares ${expected} ${category} rows but carries ${counted.get(category) ?? 0}`,
      );
    }
    counted.delete(category);
  }
  for (const category of counted.keys()) {
    errors.push(`category ${category} has rows but no declared count`);
  }
  return errors;
}

/** Total occurrences of each declared name — exported or not — across every surface. One of
 *  those is the declaration itself; the rest are uses. */
export function nameOccurrences(surfaces) {
  const declared = new Set();
  for (const file of surfaces.declarationFiles) {
    for (const match of surfaces.sources.get(file).matchAll(ANY_DECLARATION)) {
      declared.add(match[1]);
    }
  }
  const counts = new Map();
  for (const name of declared) {
    const pattern = new RegExp(`\\b${name.replaceAll("$", "\\$")}\\b`, "g");
    let total = 0;
    for (const text of surfaces.sources.values()) total += (text.match(pattern) ?? []).length;
    counts.set(name, total);
  }
  return counts;
}

export function checkFrontendExports(repo = REPO) {
  const surfaces = readSurfaces(repo);
  const observed = unreferencedExports(surfaces);
  const referencedNames = nameOccurrences(surfaces);
  const inventory = JSON.parse(fs.readFileSync(INVENTORY, "utf8"));
  return {
    observed,
    errors: validateInventory({
      observed,
      inventory,
      commands: registeredCommands(repo),
      referencedNames,
    }),
  };
}

if (process.argv[1] && import.meta.url.endsWith(path.basename(process.argv[1]))) {
  if (process.argv.includes("--list")) {
    for (const item of unreferencedExports()) {
      console.log(`${item.file}\t${item.kind}\t${item.name}\t${item.command ?? ""}`);
    }
  } else {
    const { observed, errors } = checkFrontendExports();
    if (errors.length > 0) {
      for (const error of errors) console.error(`frontend export inventory: ${error}`);
      process.exit(1);
    }
    console.log(`Frontend export inventory valid: ${observed.length} unreferenced exports owned`);
  }
}
