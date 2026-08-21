import assert from 'node:assert/strict';
import test from 'node:test';

import { checkFrontendExports, validateInventory } from './frontend-exports.mjs';

/** One owned wrapper, and the surroundings that make its claim true. */
function fixture(overrides = {}) {
  const observed = overrides.observed ?? [
    { key: 'src/ipc.ts|fixtureWrapper', file: 'src/ipc.ts', name: 'fixtureWrapper', command: 'fixture_command' },
  ];
  const inventory = overrides.inventory ?? {
    schema_version: 1,
    expected_unreferenced_export_count: observed.length,
    expected_category_counts: { 'BACKEND-ROUTE-PENDING': 1 },
    groups: [
      {
        category: 'BACKEND-ROUTE-PENDING',
        exports: [
          {
            key: 'src/ipc.ts|fixtureWrapper',
            backend_command: 'fixture_command',
            rationale: 'A wrapper for a registered command that no control in the interface reaches yet.',
          },
        ],
      },
    ],
  };
  return {
    observed,
    inventory,
    commands: overrides.commands ?? new Set(['fixture_command']),
    referencedNames: overrides.referencedNames ?? new Map(),
  };
}

test('the repository owns every unreferenced frontend export', () => {
  const { errors, observed } = checkFrontendExports();
  assert.deepEqual(
    errors,
    [],
    'every unreferenced export must carry an owner and a reason in the inventory',
  );
  assert.ok(observed.length > 0, 'the scanner found nothing at all, which means it stopped working');
});

test('an unreferenced export nobody claimed fails by name', () => {
  const base = fixture();
  const errors = validateInventory({
    ...base,
    observed: [
      ...base.observed,
      { key: 'src/ui/newPanel.ts|orphan', file: 'src/ui/newPanel.ts', name: 'orphan', command: null },
    ],
  });
  assert.ok(
    errors.some((e) => e.includes('unowned unreferenced export src/ui/newPanel.ts|orphan')),
    `a brand new dead export must be refused, got: ${errors.join(' | ')}`,
  );
});

test('a wrapper whose backend command left lib.rs is dead at both ends', () => {
  // The whole reason these rows are KEPT rather than deleted is that the command behind them
  // still works. Once it is unregistered that justification is gone, and the row must stop
  // reading as "a capability waiting for a button".
  const errors = validateInventory({ ...fixture(), commands: new Set(['some_other_command']) });
  assert.ok(
    errors.some((e) => e.includes('dead at BOTH ends')),
    `an unregistered command must be reported, got: ${errors.join(' | ')}`,
  );
});

test('a wrapper is checked against the command it actually invokes', () => {
  const base = fixture();
  base.inventory.groups[0].exports[0].backend_command = 'fixture_other_command';
  const errors = validateInventory({ ...base, commands: new Set(['fixture_command', 'fixture_other_command']) });
  assert.ok(
    errors.some((e) => e.includes("invokes 'fixture_command'") && e.includes("'fixture_other_command'")),
    `a row naming a registered but WRONG command must be refused, got: ${errors.join(' | ')}`,
  );
});

test('a supersession reached only from the export it replaces is refused', () => {
  // A row saying "the app uses X instead" is reassurance. If X is itself reached only from the
  // corpse, the capability is not reachable at all and the row has hidden that.
  const observed = [
    { key: 'src/ui/fixturePanel.ts|fixtureRamp', file: 'src/ui/fixturePanel.ts', name: 'fixtureRamp', command: null },
  ];
  const inventory = {
    schema_version: 1,
    expected_unreferenced_export_count: 1,
    expected_category_counts: { 'SUPERSEDED-BY-SIBLING': 1 },
    groups: [
      {
        category: 'SUPERSEDED-BY-SIBLING',
        exports: [
          {
            key: 'src/ui/fixturePanel.ts|fixtureRamp',
            superseded_by: 'fixtureRampEx',
            rationale: 'The narrow form; the extended one takes the options the properties dialog exposes.',
          },
        ],
      },
    ],
  };
  // declaration + exactly one use, and that use is the dead export calling it
  const deadEnd = validateInventory({
    observed,
    inventory,
    commands: new Set(),
    referencedNames: new Map([['fixtureRampEx', 2]]),
  });
  assert.ok(
    deadEnd.some((e) => e.includes('the supersession is not live')),
    `a sibling reached only from the corpse must be refused, got: ${deadEnd.join(' | ')}`,
  );

  // the other side: a genuinely live sibling passes, so the check cannot be satisfied by
  // refusing everything
  const live = validateInventory({
    observed,
    inventory,
    commands: new Set(),
    referencedNames: new Map([['fixtureRampEx', 3]]),
  });
  assert.deepEqual(live, [], 'a sibling with a use beyond the dead export must be accepted');
});

test('a row whose subject is referenced again is stale and fails', () => {
  const errors = validateInventory({ ...fixture(), observed: [] , inventory: { ...fixture().inventory, expected_unreferenced_export_count: 0, expected_category_counts: {} } });
  assert.ok(
    errors.some((e) => e.includes('stale inventory row src/ipc.ts|fixtureWrapper')),
    `this list may shrink, and a row left behind must say so, got: ${errors.join(' | ')}`,
  );
});

test('a row must say why its subject still exists', () => {
  const base = fixture();
  base.inventory.groups[0].exports[0].rationale = 'dead code';
  const errors = validateInventory(base);
  assert.ok(
    errors.some((e) => e.includes('needs a rationale')),
    `a placeholder rationale must be refused, got: ${errors.join(' | ')}`,
  );
});
