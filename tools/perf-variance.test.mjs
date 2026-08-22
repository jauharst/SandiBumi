// Pins the two refusals in `perf-variance.mjs`, and pins them from BOTH sides.
//
// Both refusals guard the same failure: a spread that is not variance, reported as though it were.
// A tool that refused every input would satisfy the two refusal tests perfectly and be useless, so
// the third test asserts that a well-formed pair is ACCEPTED and that the floor is the arithmetic
// it claims to be. Neither half passes alone.

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const tool = path.join(path.dirname(fileURLToPath(import.meta.url)), 'perf-variance.mjs');

/** One `perf_baseline` transcript: the size header plus `print_table`'s three-column rows. */
function transcript(wells, rows) {
  const lines = [
    '================ SandiBumi performance baseline ================',
    `wells ${wells} x 1562 samples = ${wells * 1562} samples`,
    'build: release',
    '',
    '== READ PATH (backend half of a click) ==',
    'OPERATION                              MEDIAN        MIN        MAX  PRODUCED',
  ];
  for (const [label, median, produced] of rows) {
    lines.push(
      label.padEnd(34)
      + `${median.toFixed(1)}ms`.padStart(10)
      + `${(median * 0.98).toFixed(1)}ms`.padStart(11)
      + `${(median * 1.02).toFixed(1)}ms`.padStart(11)
      + `  ${produced}`,
    );
  }
  return lines.join('\n') + '\n';
}

function run(transcripts) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'sb-variance-'));
  const files = transcripts.map((text, i) => {
    const p = path.join(dir, `run${i + 1}.txt`);
    fs.writeFileSync(p, text, 'utf8');
    return p;
  });
  const result = spawnSync(process.execPath, [tool, ...files], { encoding: 'utf8' });
  fs.rmSync(dir, { recursive: true, force: true });
  return result;
}

test('a_variance_report_refuses_to_average_two_different_experiments', () => {
  const result = run([
    transcript(10, [['field dashboard (pay summary)', 57.0, '30 rows']]),
    transcript(100, [['field dashboard (pay summary)', 110.1, '300 rows']]),
  ]);
  assert.notEqual(result.status, 0, 'a 10-well run and a 100-well run must not be averaged');
  assert.match(result.stderr, /different sizes/);
  // The spread between those two is ~1.9x and is entirely project size, not variance. Reporting it
  // as a noise floor would make every real scaling finding disappear into invented noise.
  assert.doesNotMatch(result.stdout, /NOISE FLOOR/);
});

test('a_timing_taken_on_a_failed_operation_is_refused_rather_than_averaged', () => {
  const result = run([
    transcript(100, [['chain 1/4 vsh_gr, all wells', 860.9, '100/100 wells ok']]),
    transcript(100, [['chain 1/4 vsh_gr, all wells', 41.2, '0/100 FAILED - first: no such curve']]),
  ]);
  assert.notEqual(result.status, 0, 'a stopwatch on a failed operation is not a timing');
  assert.match(result.stderr, /failed operations/);
  assert.doesNotMatch(result.stdout, /NOISE FLOOR/);
});

test('matching_runs_are_accepted_and_the_noise_floor_is_max_over_min', () => {
  const result = run([
    transcript(100, [['module vsh_gr, 1 well', 100.0, '1/1 wells ok']]),
    transcript(100, [['module vsh_gr, 1 well', 150.0, '1/1 wells ok']]),
  ]);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /NOISE FLOOR/);
  // 150 / 100. Pinned as a number so a floor that silently became a ratio of medians, or a
  // percentage, or a standard deviation, fails here rather than being read as a max/min.
  assert.match(result.stdout, /1\.50x/);
});

test('a_transcript_holding_more_than_one_experiment_is_refused_and_a_section_title_is_not_one', () => {
  // `cargo test --release perf_baseline` matches the module path by SUBSTRING, so it ran all four
  // tests in perf_baseline_test at once and cargo scheduled them concurrently. The rows looked
  // entirely normal; only the extra banners said so. Measured 2026-08-23: that put the 100-well
  // chain at ~83 s against ~36 s for the same command when the module held one test.
  const twoExperiments = transcript(100, [['module vsh_gr, 1 well', 67.1, '1/1 wells ok']])
    + '\n============ THE READ/WRITE SPLIT INSIDE A MODULE RUN ============\n'
    + 'wells 100 x 1562 samples ; rayon threads: 32\n';
  const refused = run([twoExperiments, twoExperiments]);
  assert.notEqual(refused.status, 0, 'timings taken while another heavy test ran are not comparable');
  assert.match(refused.stderr, /contains 2 experiments/);

  // The other side, and the reason this rule is not simply "any line of = signs": ONE experiment
  // prints several `== SECTION ==` titles of its own, and refusing those would refuse every clean
  // transcript there is. The fixture above carries one, and it must still be accepted.
  const clean = run([
    transcript(100, [['module vsh_gr, 1 well', 100.0, '1/1 wells ok']]),
    transcript(100, [['module vsh_gr, 1 well', 110.0, '1/1 wells ok']]),
  ]);
  assert.equal(clean.status, 0, clean.stderr);
  assert.match(clean.stdout, /NOISE FLOOR/);
});
