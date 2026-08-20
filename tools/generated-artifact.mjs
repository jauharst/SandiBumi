// One home for the "is this generated file still current?" comparison every generator's
// `--check` mode performs, and for the one rule that comparison has to follow.
//
// **Compare CONTENT, never line endings.** Every generator here writes LF, but git materializes a
// committed text file with CRLF under `core.autocrlf` — which is the default on Windows, and the
// state of every fresh clone and every new worktree. A raw byte comparison therefore reads an
// untouched checkout as STALE and fails the green gate on a machine that has changed nothing. The
// symptom is maximally misleading: it names a file the developer has never opened and tells them
// to regenerate it, and doing so produces no committable diff, because git normalizes the write
// straight back on the way into the index.
//
// This existed as three separate private copies (`unit-registry.mjs`, `gen-derived-overlays.mjs`,
// `generate-verification-matrix.mjs`) and one absence: `gen-third-party-licenses.mjs` compared
// raw bytes and was the one that failed. Four copies of a rule is four chances to miss it, and it
// was missed — so the rule lives here and every generator routes through it. Pinned by
// `tools/generated-artifact.test.mjs`, which also pins that every generator still imports it.

/** Line endings normalized away, so LF and CRLF spellings of one text compare equal. */
function normalizeLineEndings(text) {
  return text.replaceAll('\r\n', '\n');
}

/** Is the generated file's on-disk text current against freshly generated content?
 *
 *  `false` means genuinely stale — the generator's inputs changed and the committed output has
 *  not been regenerated. A difference that is only line endings is never staleness. */
export function isGeneratedFileCurrent(actual, expected) {
  return normalizeLineEndings(actual) === normalizeLineEndings(expected);
}
