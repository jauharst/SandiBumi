# PRD v2 spine corrections pending

This file records verified corrections for a separate spine-maintenance session. Chapter-writing
lanes do not edit the spine directly.

## SP-001 — Tier-C implementation policy

- **Status:** **RESOLVED 2026-08-08** by commit `1ae005d`.
- **Spine location:** `docs/PRD_v2/00_INDEX.md` §0.4, current line 102.
- **Former claim:** Tier C is never implemented, approximated or reverse-engineered.
- **Verified source:** `docs/PRD_v2/CONTRACT.md:146-169`, amended 2026-08-07, says that
  reconstruction is prohibited but a real Tier-C user need requires independently derived
  capability from public literature, primary sources and first principles.
- **Resolution:** §0.4 now bars transcription, reconstruction, approximation from vendor outputs and
  reverse engineering while requiring independently derived capability where a real user need is
  established, with SandiBumi's own method, defaults, limitations, tests and supported `Betters:`
  statement.

## SP-002 — Unconventional gas-content scope

- **Spine location:** `docs/PRD_v2/01_PRODUCT.md` §4.3, current line 190.
- **Current claim:** the shipped unconventional suite includes gas-in-place.
- **Verified source:** `src-tauri/src/unconventional.rs:247-281` explicitly defines per-sample gas
  content in scf/ton and names those intensive outputs `GIP_*`; lines 286-364 take no thickness or
  area and produce no Bcf volume.
- **Direction wrong:** the spine overstates the shipped physical scope. Intensive gas content ships;
  extensive areal gas-in-place does not.

## SP-003 — Omovie Sonic Saturation independent-derivation evidence

- **Chapter location:** `docs/PRD_v2/12_saturation.md` §7.4 and ESC-10.
- **Draft class:** **C-1**, because CONTRACT §2.2 names Omovie Sonic Saturation under US 12,242,011
  B2. This remains a draft legal classification until the claims are read.
- **Gap:** neither the patent claims nor the relevant published sonic-saturation literature is held
  in the saturation evidence package. The real user need, a lawful independent route, supported
  `Betters:`, defaults, limitations and an owning `SB-SAT-*` requirement/test package therefore
  cannot yet be stated.
- **What closes it:** acquire and review the claims and primary literature, establish the user need,
  then either write the independently derived package or state why no qualifying need exists.

## SP-004 — Mineral-solver sonic bridge independent-derivation evidence

- **Chapter location:** `docs/PRD_v2/13_mineral-solver.md` §7.5 and ACQ-11.
- **Draft class:** **C-2**, because dossier G-6 identifies IP's Wyllie ↔ Hunt-Raymer `Cp` bridge as a
  proprietary vendor-fitted four-term regression and identifies no patent claim.
- **Gap:** the lawful candidate route — the full nonlinear sonic response, without the fitted bridge
  or its coefficients — is supported at capability level, but the primary Wyllie (1956/1958) and
  Raymer–Hunt–Gardner (SPWLA 1980) papers are not held. Method limitations, defaults, a fully
  supported `Betters:` statement and an owning `SB-MIN-*` requirement/test package remain unwritten.
- **What closes it:** acquire the named papers, derive and document SandiBumi's own method and bounds,
  then add analytic, boundary, continuity and regression tests before implementation is required.

## SP-005 — Missing record of the superseded PRD-v1 follow-on documents

- **Spine location:** `docs/PRD_v2/06_SEQUENCING_AND_GATES.md` §26, outside this lane's ownership.
- **Verified fact:** `docs/V1_SCOPE.md`, `docs/RELEASE.md` and `docs/TARGET_ARCHITECTURE.md` exist on
  unmerged branch `docs/prd-and-security-hardening` at commit `18da8b0`; `docs/ARCHITECTURE.md` does
  not. `docs/PRD_v2/00_INDEX.md` §0.1 now records Jauhar's stated 2026-08-08 decision to discard the
  three existing documents as superseded by PRD v2.
- **Gap:** §26's “Decisions already made” table does not currently contain that discard decision,
  although the task direction says it is recorded there.
- **What closes it:** in a spine-maintenance lane, add the 2026-08-08 discard/supersession decision
  to §26 so the index citation and consolidated decision register agree.
