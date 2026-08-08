# PRD v2 spine corrections pending

This file records verified corrections for a separate spine-maintenance session. Chapter-writing
lanes do not edit the spine directly.

## SP-001 — Tier-C implementation policy

- **Spine location:** `docs/PRD_v2/00_INDEX.md` §0.4, current line 102.
- **Current claim:** Tier C is never implemented, approximated or reverse-engineered.
- **Verified source:** `docs/PRD_v2/CONTRACT.md:146-169`, amended 2026-08-07, says that
  reconstruction is prohibited but a real Tier-C user need requires independently derived
  capability from public literature, primary sources and first principles.
- **Direction wrong:** the spine is stale and over-prohibitive. It incorrectly bars the capability;
  the binding contract bars the derivation path and requires independent derivation.

## SP-002 — Unconventional gas-content scope

- **Spine location:** `docs/PRD_v2/01_PRODUCT.md` §4.3, current line 190.
- **Current claim:** the shipped unconventional suite includes gas-in-place.
- **Verified source:** `src-tauri/src/unconventional.rs:247-281` explicitly defines per-sample gas
  content in scf/ton and names those intensive outputs `GIP_*`; lines 286-364 take no thickness or
  area and produce no Bcf volume.
- **Direction wrong:** the spine overstates the shipped physical scope. Intensive gas content ships;
  extensive areal gas-in-place does not.
