import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { isGeneratedFileCurrent } from './generated-artifact.mjs';

const tools = path.dirname(fileURLToPath(import.meta.url));

// CORRECTNESS — a committed generated file is checked out with CRLF under git's `core.autocrlf`
// (the Windows default, and the state of every fresh clone and new worktree) while every
// generator writes LF. The green gate compares the two, so the comparison must ignore line
// endings — and must still catch real staleness, which is the whole reason the check exists.
//
// Pinned from BOTH sides deliberately: a comparison that always returned `true` would satisfy the
// CRLF case on its own while silently retiring every freshness gate in the repository, and a raw
// byte comparison satisfies the staleness case on its own while failing an untouched checkout.
// Neither assertion alone distinguishes a working check from a broken one.
test('a_committed_generated_file_checked_out_with_crlf_is_current_while_real_staleness_is_still_stale', () => {
  const generated = '# Title\n\n| a | b |\n| 1 | 2 |\n';
  const checkedOutOnWindows = generated.replaceAll('\n', '\r\n');

  // The line endings differ on every line; the content does not. This is a clean checkout.
  assert.notEqual(checkedOutOnWindows, generated, 'the fixture must actually differ byte for byte');
  assert.equal(
    isGeneratedFileCurrent(checkedOutOnWindows, generated),
    true,
    'a CRLF checkout of identical content is current, not stale',
  );

  // Mixed endings within one file — what an editor that rewrites only touched lines leaves behind.
  assert.equal(
    isGeneratedFileCurrent('# Title\r\n\n| a | b |\r\n| 1 | 2 |\n', generated),
    true,
    'mixed line endings are still the same content',
  );

  // A real regeneration: one cell changed. Must be stale under either spelling.
  const stale = generated.replace('| 1 | 2 |', '| 1 | 3 |');
  assert.equal(isGeneratedFileCurrent(stale, generated), false, 'changed content is stale');
  assert.equal(
    isGeneratedFileCurrent(stale.replaceAll('\n', '\r\n'), generated),
    false,
    'normalizing line endings must not normalize away a real difference',
  );

  // A trailing-newline difference is a real difference — generators emit one deliberately.
  assert.equal(
    isGeneratedFileCurrent(generated.trimEnd(), generated),
    false,
    'a missing trailing newline is staleness, not formatting',
  );
});

// CORRECTNESS — the rule above lived as three private copies plus one absence, and the absence
// (`gen-third-party-licenses.mjs`) was the generator that failed the gate in a fresh worktree. A
// fifth generator written tomorrow will reach for a raw `!==` for exactly the same reason the
// fourth did, so the drift is pinned rather than trusted: every generator with a freshness check
// routes through the one helper, and none carries its own line-ending normalization.
test('every_generator_freshness_check_routes_through_the_one_helper_instead_of_its_own_copy', () => {
  const generators = [
    'gen-third-party-licenses.mjs',
    'gen-derived-overlays.mjs',
    'unit-registry.mjs',
    'generate-verification-matrix.mjs',
  ];

  for (const generator of generators) {
    const source = fs.readFileSync(path.join(tools, generator), 'utf8');
    assert.match(
      source,
      /import \{ isGeneratedFileCurrent \} from ['"]\.\/generated-artifact\.mjs['"]/u,
      `${generator} must compare through the shared helper`,
    );
    assert.doesNotMatch(
      source,
      /replaceAll\('\\r\\n'|replace\(\/\\r\\n\/g/u,
      `${generator} carries its own line-ending normalization again`,
    );
  }
});
