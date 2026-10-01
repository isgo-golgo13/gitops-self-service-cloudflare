import { mkdirSync, writeFileSync, appendFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { root, infra, tfvars, json, sha256, git, requireEnv, assert, targetDigest, requestFingerprints } from './common.mjs';

const event = json(requireEnv('GITHUB_EVENT_PATH'));
const pr = event.pull_request;
assert(pr && pr.head.repo.full_name === requireEnv('GITHUB_REPOSITORY'), 'Only trusted internal PRs are supported');
const plan = `${root}/artifacts/approved.tfplan`;
const version = JSON.parse(execFileSync('tofu', ['version', '-json'], { encoding: 'utf8' })).terraform_version;
const meta = {
  schema: 4, repository: requireEnv('GITHUB_REPOSITORY'), pr: pr.number, head_sha: pr.head.sha,
  planned_commit: git('rev-parse', 'HEAD'), planned_tree: git('rev-parse', 'HEAD^{tree}'),
  run_id: requireEnv('GITHUB_RUN_ID'), run_attempt: requireEnv('GITHUB_RUN_ATTEMPT'), created_at: new Date().toISOString(),
  tofu_version: version, target_sha256: targetDigest(), lock_sha256: sha256(`${infra}/.terraform.lock.hcl`),
  tfvars_sha256: sha256(tfvars), requests_sha256: requestFingerprints(), plan_sha256: sha256(plan)
};
mkdirSync(`${root}/artifacts`, { recursive: true });
writeFileSync(`${root}/artifacts/metadata.json`, `${JSON.stringify(meta, null, 2)}\n`);
appendFileSync(requireEnv('GITHUB_STEP_SUMMARY'), [`PR #${pr.number}; planned tree: ${meta.planned_tree}`, `Plan SHA-256: ${meta.plan_sha256}`, `Requests: ${Object.keys(meta.requests_sha256).join(', ')}`, 'Apply refuses a different merged tree, backend, lockfile, request file, tool version, or stale plan.'].join('\n\n'));
