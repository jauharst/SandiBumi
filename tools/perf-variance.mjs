#!/usr/bin/env node
// Turns several runs of `perf_baseline` into a variance report.
//
// WHY THIS EXISTS. The harness prints a median of its own repetitions, which measures how steady
// one operation is INSIDE one run. It says nothing about how steady the same operation is BETWEEN
// runs, and between-run drift is what actually invalidates a comparison - a fix measured on Tuesday
// against a baseline taken on Monday. Passes 1-3 of the performance brief published growth ratios
// and scaling exponents built from single runs at each size, on a machine observed to put the same
// 100-well chain at 36 s and at 56 s. This computes the number that says which of those ratios
// mean anything.
//
//   node tools/perf-variance.mjs run1.txt run2.txt run3.txt run4.txt run5.txt
//
// It PARSES ONLY. It never spawns cargo, so it cannot accidentally become a second harness that
// disagrees with the first one.
//
// The output column that matters is NOISE FLOOR = max/min across runs. A claimed speed-up smaller
// than that ratio is not distinguishable from doing nothing, on this machine, for that operation.

import fs from 'node:fs';

const files = process.argv.slice(2);
if (files.length < 2) {
  console.error('usage: node tools/perf-variance.mjs <run1.txt> <run2.txt> [...] (2 or more)');
  process.exit(2);
}

// `print_table` writes: {label:<34} {median:>9.1}ms {min:>9.1}ms {max:>9.1}ms  {produced}
// The label is matched non-greedily and the three ms fields anchor the line, so a label that
// exactly fills its 34 columns - which leaves only one separating space - still parses.
const ROW = /^(.*?)\s+([\d.]+)ms\s+([\d.]+)ms\s+([\d.]+)ms(?:\s+(.*))?$/;
const HEADER = /wells\s+(\d+)\s*x\s*(\d+)\s+samples/i;
// Every harness in perf_baseline_test.rs opens with its own `==== title ====` banner. More than
// one in a transcript means more than one test ran, and cargo runs tests CONCURRENTLY by
// default - so the timings were taken under load the operation does not normally carry. Four or
// more = signs, because print_table's SECTION titles use exactly two and are not experiments.
const BANNER = /^={4,}\s+\S.*\S\s+={4,}$/;

function parseRun(path) {
  const text = fs.readFileSync(path, 'utf8');
  const size = text.match(HEADER);
  const rows = new Map();
  const banners = [];
  for (const line of text.split(/\r?\n/)) {
    if (BANNER.test(line.trim())) banners.push(line.trim());
    const m = ROW.exec(line);
    if (!m) continue;
    const label = m[1].trim();
    // Section rules, the header row, and any line whose "label" is empty are not operations.
    if (!label || label.startsWith('-') || label === 'OPERATION') continue;
    rows.set(label, { median: Number(m[2]), produced: (m[5] ?? '').trim() });
  }
  return {
    path,
    size: size ? `${size[1]} wells x ${size[2]} samples` : 'UNDECLARED',
    rows,
    banners: [...new Set(banners)],
  };
}

const runs = files.map(parseRun);

// Refuse to average two different experiments. Comparing a 10-well run with a 100-well one would
// produce a spread that is not variance at all, and it would look exactly like variance.
const sizes = [...new Set(runs.map((r) => r.size))];
if (sizes.length !== 1) {
  console.error(`refusing to combine runs of different sizes: ${sizes.join(' | ')}`);
  process.exit(1);
}

// Found the hard way, 2026-08-23: `cargo test --release perf_baseline` is a SUBSTRING match on
// the full test path, so it matched all four tests in perf_baseline_test and ran them at once.
// The 100-well chain came out at ~83 s against the ~36 s the same command produced when that
// module held one test. Nothing in the transcript says so except the extra banners - the rows
// look completely normal - which is exactly why this is a refusal and not a warning.
for (const run of runs) {
  if (run.banners.length > 1) {
    console.error(
      `refusing to report variance: ${run.path} contains ${run.banners.length} experiments, so its\n`
      + 'timings were taken while the others were running. Re-run with an exact filter, e.g.\n'
      + '  cargo test --release --lib perf_baseline_test::perf_baseline -- --exact --ignored --nocapture\n'
      + run.banners.map((b) => '  ' + b).join('\n'),
    );
    process.exit(1);
  }
}

// A timing taken on an operation that failed is a stopwatch on the failure - the exact defect the
// harness was written to avoid. If any run reports one, say so and stop.
const failures = [];
for (const run of runs) {
  for (const [label, row] of run.rows) {
    if (/FAILED|error/i.test(row.produced)) failures.push(`${run.path}: ${label} -> ${row.produced}`);
  }
}
if (failures.length > 0) {
  console.error('refusing to report variance over runs containing failed operations:');
  for (const f of failures) console.error('  ' + f);
  process.exit(1);
}

const median = (xs) => {
  const s = [...xs].sort((a, b) => a - b);
  const mid = s.length >> 1;
  return s.length % 2 ? s[mid] : (s[mid - 1] + s[mid]) / 2;
};

// Only operations EVERY run reported can be compared; one missing sample would silently change
// which runs a spread is computed over.
const labels = [...runs[0].rows.keys()].filter((l) => runs.every((r) => r.rows.has(l)));
const dropped = [...new Set(runs.flatMap((r) => [...r.rows.keys()]))].filter((l) => !labels.includes(l));

const stats = labels.map((label) => {
  const values = runs.map((r) => r.rows.get(label).median);
  const med = median(values);
  const min = Math.min(...values);
  const max = Math.max(...values);
  return { label, values, med, min, max, floor: min > 0 ? max / min : Infinity };
});
stats.sort((a, b) => b.floor - a.floor);

console.log(`\n============ BETWEEN-RUN VARIANCE - ${sizes[0]}, ${runs.length} runs ============`);
console.log('Each cell is that run\'s own median. NOISE FLOOR is max/min across runs.\n');
console.log(
  `${'OPERATION'.padEnd(34)}${'MEDIAN'.padStart(11)}${'MIN'.padStart(11)}${'MAX'.padStart(11)}${'NOISE FLOOR'.padStart(13)}`,
);
for (const s of stats) {
  console.log(
    s.label.padEnd(34)
    + `${s.med.toFixed(1)}ms`.padStart(11)
    + `${s.min.toFixed(1)}ms`.padStart(11)
    + `${s.max.toFixed(1)}ms`.padStart(11)
    + `${s.floor.toFixed(2)}x`.padStart(13),
  );
}

if (dropped.length > 0) {
  console.log(`\nNOT COMPARED - reported by some runs and not others: ${dropped.join(', ')}`);
}

const worst = stats[0];
const best = stats[stats.length - 1];
console.log(`\nSteadiest: ${best.label} at ${best.floor.toFixed(2)}x.`);
console.log(`Noisiest:  ${worst.label} at ${worst.floor.toFixed(2)}x.`);
console.log(
  '\nHOW TO READ THE FLOOR: on this machine, for that operation, a claimed improvement smaller\n'
  + 'than its floor is indistinguishable from changing nothing. A ratio measured ACROSS project\n'
  + 'sizes carries the floors of both endpoints, so it must clear roughly floor x floor before it\n'
  + 'is a scaling finding rather than a pair of unlucky runs.',
);
