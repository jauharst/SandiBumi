import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { hashRequirementIds } from './gate1-final-audit.mjs';
import { parseCsv } from './takeover-ledger.mjs';

const defaultRepo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const actionModes = ['IMPLEMENT-OR-REFUSE', 'REMEDIATE', 'PROVE', 'RETAIN'];

export function deriveGate2ActionMode(row) {
  if (row.as_built_status === 'ABSENT') return 'IMPLEMENT-OR-REFUSE';
  if (row.as_built_status === 'PARTIAL' || row.as_built_status === 'PRESENT-DIVERGENT') return 'REMEDIATE';
  if (row.as_built_status === 'PRESENT-OK' && row.test_class === 'CORRECTNESS') return 'RETAIN';
  return 'PROVE';
}

function pilotRequirementIds(pilotManifest) {
  return (pilotManifest.capability_groups ?? []).flatMap((group) => group.requirement_ids ?? []);
}

function countActionModes(requirements) {
  const counts = Object.fromEntries(actionModes.map((mode) => [mode, 0]));
  for (const requirement of requirements) {
    const mode = deriveGate2ActionMode(requirement);
    if (Object.hasOwn(counts, mode)) counts[mode] += 1;
  }
  return counts;
}

export function validateGate2Program(program, pilotManifest, ledgerRows, options = {}) {
  const errors = [];
  if (program.schema_version !== 1) errors.push('Gate 2 program schema version must be 1');
  if (!['PLANNED', 'ACTIVE', 'COMPLETE'].includes(program.state)) {
    errors.push('Gate 2 program state must be PLANNED, ACTIVE, or COMPLETE');
  }
  if (!/^[0-9a-f]{40}$/u.test(program.baseline_merge_commit ?? '')) {
    errors.push('Gate 2 baseline merge commit must be a full 40-character SHA');
  }

  const approvedIds = pilotRequirementIds(pilotManifest);
  if (pilotManifest.state !== 'APPROVED') errors.push('pilot scope must be APPROVED before Gate 2 planning');
  if (pilotManifest.included_requirement_count !== approvedIds.length) {
    errors.push(`pilot scope declares ${pilotManifest.included_requirement_count} rows but contains ${approvedIds.length}`);
  }
  const approvedScopeHash = options.approvedScopeHash ?? hashRequirementIds(approvedIds);
  if (program.approved_scope_sha256 !== approvedScopeHash) {
    errors.push(`Gate 2 scope hash ${program.approved_scope_sha256} does not match approved ${approvedScopeHash}`);
  }

  const approvedSet = new Set(approvedIds);
  const pilotGroups = new Map((pilotManifest.capability_groups ?? []).map((group) => [group.id, group]));
  const ledgerById = new Map(ledgerRows.map((row) => [row.requirement_id, row]));
  const seen = new Set();
  const gate2Requirements = [];
  const trancheIds = new Set();

  if (!Array.isArray(program.tranches) || program.tranches.length === 0) {
    errors.push('Gate 2 program must contain at least one tranche');
  }
  for (const [index, tranche] of (program.tranches ?? []).entries()) {
    const trancheId = typeof tranche.id === 'string' ? tranche.id.trim() : '';
    if (trancheId === '') errors.push(`Gate 2 tranche ${index + 1} has no id`);
    if (trancheIds.has(trancheId)) errors.push(`duplicate Gate 2 tranche ${trancheId}`);
    trancheIds.add(trancheId);
    if (typeof tranche.title !== 'string' || tranche.title.trim() === '') {
      errors.push(`Gate 2 tranche ${trancheId || index + 1} has no title`);
    }
    let trancheRequirementIds = [];
    if (Array.isArray(tranche.requirement_ids)) {
      trancheRequirementIds = tranche.requirement_ids;
    } else if (typeof tranche.pilot_group === 'string') {
      const group = pilotGroups.get(tranche.pilot_group);
      if (!group) {
        errors.push(`Gate 2 tranche ${trancheId || index + 1} names unknown pilot group ${tranche.pilot_group}`);
      } else {
        const groupIds = new Set(group.requirement_ids ?? []);
        const exclusions = Array.isArray(tranche.exclude_ids) ? tranche.exclude_ids : [];
        for (const exclusion of exclusions) {
          if (!groupIds.has(exclusion)) {
            errors.push(`Gate 2 tranche ${trancheId || index + 1} excludes ${exclusion} outside ${tranche.pilot_group}`);
          }
        }
        const excluded = new Set(exclusions);
        trancheRequirementIds = [...groupIds].filter((requirementId) => !excluded.has(requirementId));
      }
    }
    if (trancheRequirementIds.length === 0) {
      errors.push(`Gate 2 tranche ${trancheId || index + 1} has no requirements`);
      continue;
    }
    for (const requirementId of trancheRequirementIds) {
      if (seen.has(requirementId)) errors.push(`duplicate routed requirement ${requirementId}`);
      seen.add(requirementId);
      const row = ledgerById.get(requirementId);
      if (!approvedSet.has(requirementId)) errors.push(`unapproved Gate 2 requirement ${requirementId}`);
      if (!row) {
        errors.push(`Gate 2 requirement ${requirementId} is absent from the ledger`);
        continue;
      }
      if (row.release_disposition !== 'PILOT-BLOCKER') {
        errors.push(`Gate 2 requirement ${requirementId} is not a PILOT-BLOCKER`);
      }
      gate2Requirements.push(row);
    }
  }

  const later = Array.isArray(program.later_gate_only) ? program.later_gate_only : [];
  for (const requirement of later) {
    const requirementId = requirement.requirement_id;
    if (seen.has(requirementId)) errors.push(`duplicate routed requirement ${requirementId}`);
    seen.add(requirementId);
    if (!approvedSet.has(requirementId)) errors.push(`unapproved later-gate requirement ${requirementId}`);
    if (!ledgerById.has(requirementId)) errors.push(`later-gate requirement ${requirementId} is absent from the ledger`);
    if (!['G3', 'G4', 'G5'].includes(requirement.owner_gate)) {
      errors.push(`${requirementId} has invalid later owner ${requirement.owner_gate}`);
    }
    if (requirement.state !== 'NOT-OWNED-BY-G2') {
      errors.push(`${requirementId} later-gate state must be NOT-OWNED-BY-G2`);
    }
    if (typeof requirement.reason !== 'string' || requirement.reason.trim() === '') {
      errors.push(`${requirementId} later-gate route has no reason`);
    }
  }

  for (const requirementId of approvedIds) {
    if (!seen.has(requirementId)) errors.push(`approved requirement ${requirementId} is not routed`);
  }
  for (const requirementId of seen) {
    if (!approvedSet.has(requirementId)) errors.push(`routed requirement ${requirementId} is outside the approved scope`);
  }

  if (program.gate2_requirement_count !== gate2Requirements.length) {
    errors.push(`Gate 2 declares ${program.gate2_requirement_count} rows but routes ${gate2Requirements.length}`);
  }
  if (program.later_gate_requirement_count !== later.length) {
    errors.push(`later gates declare ${program.later_gate_requirement_count} rows but route ${later.length}`);
  }
  if (seen.size !== approvedSet.size) {
    errors.push(`routed requirement cardinality ${seen.size} does not match approved ${approvedSet.size}`);
  }

  const gate2Ids = new Set(gate2Requirements.map((row) => row.requirement_id));
  const completed = Array.isArray(program.completed_requirements) ? program.completed_requirements : [];
  const blocked = Array.isArray(program.blocked_requirements) ? program.blocked_requirements : [];
  const progressSeen = new Set();
  for (const requirementId of [...completed, ...blocked]) {
    if (!gate2Ids.has(requirementId)) errors.push(`progress row ${requirementId} is not owned by Gate 2`);
    if (progressSeen.has(requirementId)) errors.push(`progress row ${requirementId} is both completed and blocked`);
    progressSeen.add(requirementId);
  }
  if (program.state === 'COMPLETE' && progressSeen.size !== gate2Ids.size) {
    errors.push(`complete Gate 2 program accounts for ${progressSeen.size} outcomes but owns ${gate2Ids.size}`);
  }

  const derivedCounts = countActionModes(gate2Requirements);
  for (const mode of actionModes) {
    if (program.action_mode_counts?.[mode] !== derivedCounts[mode]) {
      errors.push(`${mode} count ${program.action_mode_counts?.[mode]} does not match routed ${derivedCounts[mode]}`);
    }
  }

  if (errors.length > 0) throw new Error(errors.join('; '));
  return {
    valid: true,
    approved: approvedIds.length,
    gate2: gate2Requirements.length,
    later: later.length,
    tranches: program.tranches.length,
    action_mode_counts: derivedCounts,
  };
}

export function checkGate2Program(repo = defaultRepo) {
  const ledgerRows = parseCsv(fs.readFileSync(path.join(repo, 'docs', 'takeover', 'requirements.csv'), 'utf8'));
  const pilotManifest = JSON.parse(fs.readFileSync(path.join(repo, 'docs', 'takeover', 'pilot-scope.json'), 'utf8'));
  const program = JSON.parse(fs.readFileSync(path.join(repo, 'docs', 'takeover', 'gate2-program.json'), 'utf8'));
  return validateGate2Program(program, pilotManifest, ledgerRows);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const result = checkGate2Program();
    console.log(
      `Gate 2 program valid: ${result.gate2}/${result.approved} owned here, ${result.later} later-gate-only, ${result.tranches} tranches`,
    );
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
