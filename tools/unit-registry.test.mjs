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
    alias_patterns: registry.families.reduce((sum, family) => sum + (family.alias_patterns ?? []).length, 0),
    alias_exclusions: registry.families.reduce((sum, family) => sum + (family.excluded_alias_patterns ?? []).length, 0),
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
    registry.families.map(({
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
    })),
  );
  assert.match(typescript, /export function unitRegistryFamilyFor/u);
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

// CORRECTNESS - AUDIT-2026-08-20 finding 48. `rule.families` is the field the converter
// SELECTS on (curves.rs: `rule.families.contains(&family)`), and it was the one field on a rule
// that nothing validated - unit tokens were checked, convertible_families was checked, this was
// not. A family misspelt here fails nothing: the registry generates, the app boots, and the
// conversion the rule exists to perform never fires, so the curve lands in its declared unit
// under a canonical name. That is the exact silent wrongness the registry is built to prevent.
test('a_rule_naming_a_family_that_does_not_exist_fails_the_registry_instead_of_generating', () => {
  const base = loadRegistry();
  const clone = () => JSON.parse(JSON.stringify(base));

  // The live registry passes as it stands - the check is real, not vacuous.
  assert.doesNotThrow(() => validateRegistry(clone()));

  // A typo in an automatic rule's family is refused by name.
  const typo = clone();
  const automatic = typo.rules.find((rule) => rule.automatic && (rule.families ?? []).length);
  assert.ok(automatic, 'the registry ships at least one automatic rule to mistype');
  automatic.families = ['RHOOB'];
  assert.throws(() => validateRegistry(typo), /RHOOB/, 'a misspelt family must fail the build');

  // QV is named on purpose and only ever on a rule that cannot bind automatically. Flipping
  // that flag is what would make the unbindable family bind, so it is refused too.
  const bound = clone();
  const qv = bound.rules.find((rule) => (rule.families ?? []).includes('QV'));
  assert.ok(qv, 'the QV rule is what the exception exists for');
  assert.equal(qv.automatic, false, 'and it ships non-automatic, per section 7.1 O-2');
  qv.automatic = true;
  assert.throws(() => validateRegistry(bound), /QV/, 'an unbindable family must not bind');

  // The exception cannot outlive its reason in either direction.
  const declared = clone();
  declared.families.push({
    family: 'QV',
    canonical_unit: declared.families[0].canonical_unit,
    quantity_kind: declared.families[0].quantity_kind,
    aliases: [],
  });
  assert.throws(() => validateRegistry(declared), /declared family now/, 'a family that joined must leave the list');

  const dropped = clone();
  dropped.rules = dropped.rules.filter((rule) => !(rule.families ?? []).includes('QV'));
  assert.throws(() => validateRegistry(dropped), /no rule names/, 'an excusal nothing uses must go');
});
