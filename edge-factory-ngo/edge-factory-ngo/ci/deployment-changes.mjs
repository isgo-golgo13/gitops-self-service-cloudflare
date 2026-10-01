// This credential-free classification lets docs-only / ssip-only PRs pass required CI
// without invoking a production plan or apply. Production credentials are only
// available in the subsequent plan/apply jobs.
import { json, github, requireEnv, assert, output } from './common.mjs';

const repo = requireEnv('GITHUB_REPOSITORY');
const event = json(requireEnv('GITHUB_EVENT_PATH'));
let pr = event.pull_request;
if (!pr) {
  const sha = requireEnv('GITHUB_SHA');
  const prs = await github(`/repos/${repo}/commits/${sha}/pulls?per_page=100`);
  const merged = prs.filter(item => item.merged_at && item.merge_commit_sha === sha && item.base.ref === 'main');
  assert(merged.length === 1, 'Expected an approved merged PR; direct pushes are not deployable');
  pr = merged[0];
}
assert(pr.head.repo?.full_name === repo, 'Only trusted internal PRs are supported');
const relevant = path => {
  return path.startsWith('infra/') || path.startsWith('ci/') || path.startsWith('security/') ||
    ['.github/workflows/plan.yml', '.github/workflows/apply.yml', '.pre-commit-config.yaml', '.tflint.hcl'].includes(path);
};
let required = false;
let count = 0;
for (let page = 1; page <= 30; page++) {
  const files = await github(`/repos/${repo}/pulls/${pr.number}/files?per_page=100&page=${page}`);
  count += files.length;
  required ||= files.some(file => relevant(file.filename) || (file.previous_filename && relevant(file.previous_filename)));
  if (files.length < 100) break;
}
// GitHub caps this endpoint at 3000 files. At the cap, fail closed into a plan.
if (count >= 3000) required = true;
output('required', String(required));
console.log(required ? 'Deployment inputs or controls changed: plan/apply is required.' : 'No deployment change: terminal/documentation work remains independent.');
