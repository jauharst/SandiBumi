import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { isGeneratedFileCurrent } from './generated-artifact.mjs';

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
export const REGISTRY_PATH = 'registry/unit-registry.json';
export const OUTPUT_PATHS = [
  'src-tauri/src/generated/unit_registry.rs',
  'src/generated/unitRegistry.ts',
  'docs/generated/UNIT_REGISTRY.md',
  'verification/unit-registry.json',
];

function fail(message) {
  throw new Error(`unit registry: ${message}`);
}

function unique(items, label) {
  const seen = new Set();
  for (const item of items) {
    if (seen.has(item)) fail(`duplicate ${label} '${item}'`);
    seen.add(item);
  }
}

export function registryDigest(registry) {
  return crypto.createHash('sha256').update(`${JSON.stringify(registry)}\n`, 'utf8').digest('hex');
}

export function validateRegistry(registry) {
  if (registry.schema_version !== 1) fail(`unsupported schema version ${registry.schema_version}`);
  if (!registry.registry_version || typeof registry.registry_version !== 'string') {
    fail('registry_version is required');
  }
  for (const key of ['quantity_kinds', 'families', 'convertible_families', 'unit_tokens', 'rules']) {
    if (!Array.isArray(registry[key])) fail(`${key} must be an array`);
  }

  unique(registry.quantity_kinds.map((kind) => kind.id), 'quantity kind');
  unique(registry.quantity_kinds.map((kind) => kind.rust_variant), 'Rust quantity variant');
  const kinds = new Map(registry.quantity_kinds.map((kind) => [kind.id, kind]));
  const tokens = new Map();
  unique(registry.unit_tokens.map((unit) => unit.token), 'unit token');
  for (const unit of registry.unit_tokens) {
    if (!kinds.has(unit.quantity_kind)) {
      fail(`unit '${unit.token}' has unknown quantity kind '${unit.quantity_kind}'`);
    }
    tokens.set(unit.token, unit);
  }
  for (const unit of registry.unit_tokens) {
    const canonical = tokens.get(unit.canonical_unit);
    if (!canonical) fail(`unit '${unit.token}' names absent canonical token '${unit.canonical_unit}'`);
    if (canonical.quantity_kind !== unit.quantity_kind) {
      fail(
        `unit '${unit.token}' is ${unit.quantity_kind} but canonical '${unit.canonical_unit}' is ${canonical.quantity_kind}`,
      );
    }
  }

  unique(registry.families.map((family) => family.family), 'family');
  unique(registry.families.flatMap((family) => family.aliases), 'family alias');
  unique(
    registry.families.flatMap((family) => family.alias_patterns ?? []),
    'family alias pattern',
  );
  const families = new Set(registry.families.map((family) => family.family));
  for (const family of registry.families) {
    if (!Array.isArray(family.aliases)
      || !Array.isArray(family.alias_patterns ?? [])
      || !Array.isArray(family.excluded_alias_patterns ?? [])) {
      fail(`family '${family.family}' aliases and patterns must be arrays`);
    }
    for (const pattern of [
      ...(family.alias_patterns ?? []),
      ...(family.excluded_alias_patterns ?? []),
    ]) {
      if (typeof pattern !== 'string' || !pattern.includes('*') || pattern !== pattern.toUpperCase()) {
        fail(`family '${family.family}' has invalid uppercase wildcard pattern '${pattern}'`);
      }
    }
    if (!kinds.has(family.quantity_kind)) {
      fail(`family '${family.family}' has unknown quantity kind '${family.quantity_kind}'`);
    }
    const canonical = tokens.get(family.canonical_unit);
    if (!canonical) {
      fail(`family '${family.family}' names absent canonical unit '${family.canonical_unit}'`);
    }
    if (canonical.quantity_kind !== family.quantity_kind) {
      fail(
        `family '${family.family}' declares ${family.quantity_kind} but canonical '${family.canonical_unit}' is ${canonical.quantity_kind}`,
      );
    }
  }
  unique(registry.convertible_families, 'convertible family');
  for (const family of registry.convertible_families) {
    if (!families.has(family)) fail(`convertible family '${family}' is absent from families`);
  }

  for (const rule of registry.rules) {
    const from = tokens.get(rule.from_unit);
    const to = tokens.get(rule.to_unit);
    if (!from || !to) {
      fail(`rule '${rule.from_unit}' -> '${rule.to_unit}' names an absent unit token`);
    }
    if (from.quantity_kind !== to.quantity_kind) {
      fail(
        `rule '${rule.from_unit}' -> '${rule.to_unit}' changes dimension from ${from.quantity_kind} to ${to.quantity_kind}`,
      );
    }
    if (!rule.derivation || !rule.factor_expression || !rule.offset_expression) {
      fail(`rule '${rule.from_unit}' -> '${rule.to_unit}' lacks arithmetic custody`);
    }
  }
  return registry;
}

function rustString(value) {
  return JSON.stringify(value);
}

function renderRust(registry, digest) {
  const kindById = new Map(registry.quantity_kinds.map((kind) => [kind.id, kind.rust_variant]));
  const families = registry.families.map((family) =>
    `    FamilySpec { family: ${rustString(family.family)}, canonical_unit: ${rustString(family.canonical_unit)}, quantity_kind: QuantityKind::${kindById.get(family.quantity_kind)}, aliases: &[${family.aliases.map(rustString).join(', ')}], alias_patterns: &[${(family.alias_patterns ?? []).map(rustString).join(', ')}], excluded_alias_patterns: &[${(family.excluded_alias_patterns ?? []).map(rustString).join(', ')}] },`,
  );
  const tokens = registry.unit_tokens.map((unit) =>
    `    UnitTokenSpec { token: ${rustString(unit.token)}, quantity_kind: QuantityKind::${kindById.get(unit.quantity_kind)}, canonical_unit: ${rustString(unit.canonical_unit)} },`,
  );
  const rules = registry.rules.map((rule) => [
    '    UnitRule {',
    `        families: &[${rule.families.map(rustString).join(', ')}],`,
    `        from_unit: ${rustString(rule.from_unit)},`,
    `        to_unit: ${rustString(rule.to_unit)},`,
    `        factor: ${rule.factor_expression},`,
    `        offset: ${rule.offset_expression},`,
    `        derivation: ${rustString(rule.derivation)},`,
    `        automatic: ${rule.automatic},`,
    '    },',
  ].join('\n'));
  return `// GENERATED by tools/unit-registry.mjs from ${REGISTRY_PATH} — DO NOT EDIT.\n\
pub const UNIT_REGISTRY_VERSION: &str = ${rustString(registry.registry_version)};\n\
pub const UNIT_REGISTRY_SHA256: &str = ${rustString(digest)};\n\
\n\
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]\n\
#[serde(rename_all = "snake_case")]\n\
pub enum QuantityKind {\n${registry.quantity_kinds.map((kind) => `    ${kind.rust_variant},`).join('\n')}\n}\n\
\n\
pub const FAMILIES: &[FamilySpec] = &[\n${families.join('\n')}\n];\n\
\n\
pub const CONVERTIBLE_FAMILIES: &[&str] = &[${registry.convertible_families.map(rustString).join(', ')}];\n\
\n\
pub const UNIT_TOKENS: &[UnitTokenSpec] = &[\n${tokens.join('\n')}\n];\n\
\n\
pub const UNIT_RULES: &[UnitRule] = &[\n${rules.join('\n')}\n];\n`;
}

function renderTypeScript(registry, digest) {
  const families = registry.families.map(({
    family,
    canonical_unit,
    quantity_kind,
    aliases,
    alias_patterns = [],
    excluded_alias_patterns = [],
  }) => ({
    family,
    canonicalUnit: canonical_unit,
    quantityKind: quantity_kind,
    aliases,
    aliasPatterns: alias_patterns,
    excludedAliasPatterns: excluded_alias_patterns,
  }));
  const units = registry.unit_tokens.map(({ token, quantity_kind, canonical_unit }) => ({
    token,
    quantityKind: quantity_kind,
    canonicalUnit: canonical_unit,
  }));
  const rules = registry.rules.map((rule) => `  {
    families: ${JSON.stringify(rule.families)},
    fromUnit: ${JSON.stringify(rule.from_unit)},
    toUnit: ${JSON.stringify(rule.to_unit)},
    factor: ${rule.factor_expression},
    offset: ${rule.offset_expression},
    derivation: ${JSON.stringify(rule.derivation)},
    automatic: ${rule.automatic},
  }`).join(',\n');
  return `// GENERATED by tools/unit-registry.mjs from ${REGISTRY_PATH} — DO NOT EDIT.\n\
export const UNIT_REGISTRY_VERSION = ${JSON.stringify(registry.registry_version)};\n\
export const UNIT_REGISTRY_SHA256 = ${JSON.stringify(digest)};\n\
export const UNIT_REGISTRY_FAMILIES = ${JSON.stringify(families, null, 2)} as const;\n\
export const UNIT_REGISTRY_UNITS = ${JSON.stringify(units, null, 2)} as const;\n\
export const UNIT_REGISTRY_RULES = [\n${rules}\n] as const;\n\
export const UNIT_REGISTRY_POPULATION = {\n\
  families: UNIT_REGISTRY_FAMILIES.length,\n\
  aliases: UNIT_REGISTRY_FAMILIES.reduce((sum, family) => sum + family.aliases.length, 0),\n\
  aliasPatterns: UNIT_REGISTRY_FAMILIES.reduce((sum, family) => sum + family.aliasPatterns.length, 0),\n\
  aliasExclusions: UNIT_REGISTRY_FAMILIES.reduce((sum, family) => sum + family.excludedAliasPatterns.length, 0),\n\
  units: UNIT_REGISTRY_UNITS.length,\n\
  rules: UNIT_REGISTRY_RULES.length,\n\
} as const;\n\
function registryAliasPatternMatches(pattern: string, value: string): boolean {
  const pieces = pattern.split("*");
  let cursor = 0;
  for (let index = 0; index < pieces.length; index += 1) {
    const piece = pieces[index];
    if (!piece) continue;
    const found = value.indexOf(piece, cursor);
    if (found < 0 || (index === 0 && !pattern.startsWith("*") && found !== 0)) return false;
    cursor = found + piece.length;
  }
  const tail = pieces[pieces.length - 1] ?? "";
  return pattern.endsWith("*") || value.endsWith(tail);
}\n\
export function unitRegistryFamilyFor(mnemonic: string): string | null {
  const key = mnemonic.trim().toUpperCase();
  const exact = UNIT_REGISTRY_FAMILIES.find((entry) =>
    entry.aliases.some((alias) => alias === key));
  if (exact) return exact.family;
  let resolved: string | null = null;
  for (const entry of UNIT_REGISTRY_FAMILIES) {
    if (entry.excludedAliasPatterns.some((pattern) => registryAliasPatternMatches(pattern, key))) continue;
    if (!entry.aliasPatterns.some((pattern) => registryAliasPatternMatches(pattern, key))) continue;
    if (resolved && resolved !== entry.family) return null;
    resolved = entry.family;
  }
  return resolved;
}\n`;
}

function renderMarkdown(registry, digest) {
  const familyRows = registry.families.map((family) =>
    `| \`${family.family}\` | \`${family.quantity_kind}\` | \`${family.canonical_unit}\` | ${family.aliases.map((alias) => `\`${alias}\``).join(', ')} | ${(family.alias_patterns ?? []).map((alias) => `\`${alias}\``).join(', ')} | ${(family.excluded_alias_patterns ?? []).map((alias) => `\`${alias}\``).join(', ')} |`,
  );
  const unitRows = registry.unit_tokens.map((unit) =>
    `| \`${unit.token}\` | \`${unit.quantity_kind}\` | \`${unit.canonical_unit}\` |`,
  );
  return `<!-- GENERATED by tools/unit-registry.mjs from ${REGISTRY_PATH} — DO NOT EDIT. -->\n\
# Canonical curve and unit registry\n\
\n\
- Registry version: \`${registry.registry_version}\`\n\
- Source SHA-256: \`${digest}\`\n\
- Population: ${registry.families.length} families, ${registry.families.reduce((sum, family) => sum + family.aliases.length, 0)} exact aliases, ${registry.families.reduce((sum, family) => sum + (family.alias_patterns ?? []).length, 0)} vendor alias patterns, ${registry.families.reduce((sum, family) => sum + (family.excluded_alias_patterns ?? []).length, 0)} pattern exclusions, ${registry.unit_tokens.length} unit tokens, ${registry.rules.length} conversion rules\n\
\n\
## Families\n\
\n\
| Family | Quantity kind | Canonical unit | Exact mnemonic aliases | Vendor alias patterns | Pattern exclusions |\n\
|---|---|---|---|---|---|\n\
${familyRows.join('\n')}\n\
\n\
## Unit tokens\n\
\n\
| Observed token | Quantity kind | Canonical interpretation |\n\
|---|---|---|\n\
${unitRows.join('\n')}\n`;
}

function renderManifest(registry, digest) {
  return `${JSON.stringify({
    schema_version: 1,
    registry_version: registry.registry_version,
    source_sha256: digest,
    populations: {
      families: registry.families.length,
      aliases: registry.families.reduce((sum, family) => sum + family.aliases.length, 0),
      alias_patterns: registry.families.reduce((sum, family) => sum + (family.alias_patterns ?? []).length, 0),
      alias_exclusions: registry.families.reduce((sum, family) => sum + (family.excluded_alias_patterns ?? []).length, 0),
      units: registry.unit_tokens.length,
      rules: registry.rules.length,
    },
    families: registry.families,
    unit_tokens: registry.unit_tokens,
  }, null, 2)}\n`;
}

export function generateArtifacts(registry) {
  validateRegistry(registry);
  const digest = registryDigest(registry);
  return new Map([
    [OUTPUT_PATHS[0], renderRust(registry, digest)],
    [OUTPUT_PATHS[1], renderTypeScript(registry, digest)],
    [OUTPUT_PATHS[2], renderMarkdown(registry, digest)],
    [OUTPUT_PATHS[3], renderManifest(registry, digest)],
  ]);
}

export function writeArtifacts(root, artifacts) {
  for (const [relative, content] of artifacts) {
    const target = path.join(root, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, content, 'utf8');
  }
}

export function checkArtifacts(root, artifacts) {
  for (const [relative, expected] of artifacts) {
    const target = path.join(root, relative);
    if (!fs.existsSync(target)) fail(`generated output is missing: ${relative}`);
    if (!isGeneratedFileCurrent(fs.readFileSync(target, 'utf8'), expected)) {
      fail(`generated output is stale: ${relative}`);
    }
  }
}

export function loadRegistry(root = repo) {
  return validateRegistry(JSON.parse(fs.readFileSync(path.join(root, REGISTRY_PATH), 'utf8')));
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const registry = loadRegistry(repo);
  const artifacts = generateArtifacts(registry);
  if (process.argv.includes('--check')) {
    checkArtifacts(repo, artifacts);
    process.stdout.write(`Unit registry ${registry.registry_version} is current across ${artifacts.size} generated consumers.\n`);
  } else {
    writeArtifacts(repo, artifacts);
    process.stdout.write(`Generated ${artifacts.size} unit-registry consumers from ${REGISTRY_PATH}.\n`);
  }
}
