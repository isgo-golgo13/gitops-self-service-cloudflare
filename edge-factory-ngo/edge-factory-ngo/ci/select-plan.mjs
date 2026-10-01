import { github, requireEnv, assert, output } from './common.mjs';

const repo = requireEnv('GITHUB_REPOSITORY');
const merged = requireEnv('GITHUB_SHA');
const prs = await github(`/repos/${repo}/commits/${merged}/pulls?per_page=100`);
const candidates = prs.filter(pr => pr.merged_at && pr.merge_commit_sha === merged && pr.base.ref === 'main');
assert(candidates.length === 1, 'Expected one merged PR for this main commit; direct pushes are not deployable');
const pr = await github(`/repos/${repo}/pulls/${candidates[0].number}`);
assert(pr.head.repo?.full_name === repo, 'Fork PR deployments are not supported');

// Restrict the search to the exact workflow, event, repository, PR and head SHA.
let runs = [];
for (let page = 1; page <= 10; page++) {
  const result = await github(`/repos/${repo}/actions/workflows/plan.yml/runs?event=pull_request&head_sha=${pr.head.sha}&per_page=100&page=${page}`);
  runs.push(...result.workflow_runs.filter(run =>
    run.head_repository?.full_name === repo && run.head_sha === pr.head.sha &&
    run.pull_requests.some(item => item.number === pr.number)
  ));
  if (result.workflow_runs.length < 100) break;
}
runs.sort((a, b) => Number(b.id) - Number(a.id));
const run = runs[0];
assert(run && run.status === 'completed' && run.conclusion === 'success', 'Latest matching plan run is not successful');
assert(Date.parse(run.updated_at) <= Date.parse(pr.merged_at), 'The successful plan must predate PR merge');
const name = `prod-plan-${pr.head.sha}-${run.id}-${run.run_attempt}`;
const result = await github(`/repos/${repo}/actions/runs/${run.id}/artifacts?per_page=100`);
const artifacts = result.artifacts.filter(a => a.name === name && !a.expired);
assert(artifacts.length === 1, 'Expected one unexpired artifact from the selected plan attempt');
output('run_id', run.id);
output('run_attempt', run.run_attempt);
output('artifact_id', artifacts[0].id);
output('pr', pr.number);
output('head_sha', pr.head.sha);
