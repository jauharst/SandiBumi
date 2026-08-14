import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {
  checkArtifacts,
  generateArtifacts,
  loadRegistry,
  OUTPUT_PATHS,
  registryDigest,
  validateRegistry,
  writeArtifacts,
} from './unit-registry.mjs';

// CORRECTNESS — SB-INS-019 / SB-INS-T24. The equal version/population result and both
// release refusals are specified by 27_ip-install-blockers.md sections 4 and 6. This test
// derives all four consumers independently from the reviewed source fixture; it does not
// treat current generated output as its expected value.
test('one_versioned_registry_generates_equal_runtime_ui_documentation_and_test_populations_and_release_refuses_drift', () => {
  const registry = loadRegistry();
  const digest = registryDigest(registry);
  const artifacts = generateArtifacts(registry);
  assert.deepEqual([...artifacts.keys()], OUTPUT_PATHS);

  const manifest = JSON.parse(artifacts.get('verification/unit-registry.json'));
  assert.equal(manifest.registry_version, registry.registry_version);
  assert.equal(manifest.source_sha256, digest);
  assert.deepEqual(manifest.populations, {
    families: registry.families.length,
    aliases: registry.families.reduce((sum, family) => sum + family.aliases.length, 0),
    units: registry.unit_tokens.length,
    rules: registry.rules.length,
  });
  for (const relative of OUTPUT_PATHS.slice(0, 3)) {
    const output = artifacts.get(relative);
    assert.match(output, new RegExp(registry.registry_version));
    assert.match(output, new RegExp(digest));
  }
  assert.deepEqual(manifest.families, registry.families);
  assert.deepEqual(manifest.unit_tokens, registry.unit_tokens);

  const rust = artifacts.get('src-tauri/src/generated/unit_registry.rs');
  for (const family of registry.families) {
    assert.ok(
      rust.includes(
        `FamilySpec { family: ${JSON.stringify(family.family)}, canonical_unit: ${JSON.stringify(family.canonical_unit)},`,
      ),
    );
  }
  for (const unit of registry.unit_tokens) {
    assert.ok(
      rust.includes(`UnitTokenSpec { token: ${JSON.stringify(unit.token)},`),
    );
  }

  const typescript = artifacts.get('src/generated/unitRegistry.ts');
  const tsFamilies = JSON.parse(
    typescript.match(/UNIT_REGISTRY_FAMILIES = ([\s\S]+?) as const;/u)[1],
  );
  const tsUnits = JSON.parse(
    typescript.match(/UNIT_REGISTRY_UNITS = ([\s\S]+?) as const;/u)[1],
  );
  assert.deepEqual(
    tsFamilies,
    registry.families.map(({ family, canonical_unit, quantity_kind, aliases }) => ({
      family,
      canonicalUnit: canonical_unit,
      quantityKind: quantity_kind,
      aliases,
    })),
  );
  assert.match(typescript, /export const UNIT_REGISTRY_RULES = \[/u);
  for (const rule of registry.rules) {
    assert.match(typescript, new RegExp(`fromUnit: ${JSON.stringify(rule.from_unit)}`));
    assert.match(typescript, new RegExp(`toUnit: ${JSON.stringify(rule.to_unit)}`));
    assert.match(typescript, new RegExp(`derivation: ${JSON.stringify(rule.derivation).replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`));
  }
  assert.deepEqual(
    tsUnits,
    registry.unit_tokens.map(({ token, quantity_kind, canonical_unit }) => ({
      token,
      quantityKind: quantity_kind,
      canonicalUnit: canonical_unit,
    })),
  );

  const documentation = artifacts.get('docs/generated/UNIT_REGISTRY.md');
  for (const family of registry.families) {
    assert.ok(
      documentation.includes(`| \`${family.family}\` | \`${family.quantity_kind}\` |`),
    );
  }
  for (const unit of registry.unit_tokens) {
    assert.ok(
      documentation.includes(`| \`${unit.token}\` | \`${unit.quantity_kind}\` | \`${unit.canonical_unit}\` |`),
    );
  }

  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'sandibumi-unit-registry-'));
  try {
    writeArtifacts(root, artifacts);
    assert.doesNotThrow(() => checkArtifacts(root, artifacts));
    fs.appendFileSync(path.join(root, 'src/generated/unitRegistry.ts'), '\n// drift\n', 'utf8');
    assert.throws(() => checkArtifacts(root, artifacts), /generated output is stale/u);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }

  const wrongDimension = structuredClone(registry);
  wrongDimension.families[0].quantity_kind = 'length';
  assert.throws(() => validateRegistry(wrongDimension), /declares length.*gamma_ray/u);
});
