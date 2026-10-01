import { execFileSync } from 'node:child_process';
import { infra, root, tfvars, json, sha256, git, requireEnv, assert, github, targetDigest, requestFingerprints } from './common.mjs';

const meta = json(`${root}/artifacts/metadata.json`);
const version = JSON.parse(execFileSync('tofu', ['version', '-json'], { encoding: 'utf8' })).terraform_version;
const checks = {
  'metadata format': meta.schema === 4,
  repository: meta.repository === requireEnv('GITHUB_REPOSITORY'),
  'PR number': String(meta.pr) === requireEnv('PLAN_PR'),
  'PR head': meta.head_sha === requireEnv('PLAN_HEAD_SHA'),
  'run ID': meta.run_id === requireEnv('PLAN_RUN_ID'),
  'run attempt': meta.run_attempt === requireEnv('PLAN_RUN_ATTEMPT'),
  'state backend': meta.target_sha256 === targetDigest(),
  'merged source tree': meta.planned_tree === git('rev-parse', 'HEAD^{tree}'),
  'provider lockfile': meta.lock_sha256 === sha256(`${infra}/.terraform.lock.hcl`),
  'platform tfvars': meta.tfvars_sha256 === sha256(tfvars),
  'request files': JSON.stringify(meta.requests_sha256) === JSON.stringify(requestFingerprints()),
  'plan digest': meta.plan_sha256 === sha256(`${root}/artifacts/approved.tfplan`),
  'OpenTofu version': meta.tofu_version === version
};
for (const [label, ok] of Object.entries(checks)) assert(ok, `Rejected plan: ${label} mismatch`);
const age = Date.now() - Date.parse(meta.created_at);
assert(Number.isFinite(age) && age >= 0 && age <= 24 * 60 * 60 * 1000, 'Plan is older than 24 hours or has an invalid timestamp');
const branch = await github(`/repos/${requireEnv('GITHUB_REPOSITORY')}/branches/main`);
assert(branch.commit.sha === requireEnv('GITHUB_SHA'), 'A newer main commit exists; do not deploy an older queued change');
console.log('Approved plan matches the merged tree, backend, requests, provider lockfile and tool version.');
