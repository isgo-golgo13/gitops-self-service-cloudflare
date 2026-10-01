import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, symlinkSync, rmSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const source = resolve(fileURLToPath(new URL('..', import.meta.url)));
const scratch = join(source, '.test');

test('saved-plan guards over request files, and workflow routing', async t => {
  mkdirSync(scratch, { recursive: true });
  const root = mkdtempSync(join(scratch, 'ci-'));
  try {
    const infra = join(root, 'infra/live/prod');
    for (const dir of [join(infra, 'requests'), join(root, 'artifacts'), join(root, 'bin')]) mkdirSync(dir, { recursive: true });
    symlinkSync(join(source, 'ci'), join(root, 'ci'), 'dir');
    writeFileSync(join(infra, '.terraform.lock.hcl'), 'lock\n');
    writeFileSync(join(infra, 'prod.auto.tfvars'), 'platform\n');
    writeFileSync(join(infra, 'backend.tfbackend'), 'backend\n');
    writeFileSync(join(infra, 'requests/billing-api.json'), JSON.stringify({ apps: { 'billing-api': { zone: 'dronegrid.io', hostname: 'billing', tier: 'medium', exposure: 'public' } } }));
    writeFileSync(join(root, 'artifacts/approved.tfplan'), 'plan');
    writeFileSync(join(root, 'bin/tofu'), '#!/usr/bin/env node\nprocess.stdout.write(JSON.stringify({terraform_version:process.env.TEST_TOFU_VERSION || "1.12.6"}));', { mode: 0o700 });
    const git = (...a) => execFileSync('git', a, { cwd: root, encoding: 'utf8' }).trim();
    git('init', '-q'); git('add', 'infra'); git('-c', 'user.name=t', '-c', 'user.email=t@dronegrid.io', 'commit', '-qm', 'f');
    const sha = git('rev-parse', 'HEAD');
    writeFileSync(join(root, 'event.json'), JSON.stringify({ pull_request: { number: 7, head: { sha: 'a'.repeat(40), repo: { full_name: 'isgo-golgo13/edge-factory-ngo' } } } }));
    const mock = join(root, 'mock.mjs');
    writeFileSync(mock, 'import { readFileSync } from "node:fs"; globalThis.fetch = async (url) => { const p = new URL(url).pathname; const v = p.endsWith("/branches/main") ? {commit:{sha:process.env.MOCK_MAIN}} : JSON.parse(readFileSync(process.env.MOCK_FILES)); return new Response(JSON.stringify(v), {status:200}); };');
    const env = { ...process.env, PATH: `${join(root, 'bin')}:${process.env.PATH}`, GITHUB_REPOSITORY: 'isgo-golgo13/edge-factory-ngo', GITHUB_SHA: sha, GITHUB_RUN_ID: '9', GITHUB_RUN_ATTEMPT: '1', GITHUB_EVENT_PATH: join(root, 'event.json'), GITHUB_STEP_SUMMARY: join(root, 's.txt'), GITHUB_OUTPUT: join(root, 'o.txt'), GITHUB_API_URL: 'https://api.github.com', GH_TOKEN: 't', PLAN_PR: '7', PLAN_HEAD_SHA: 'a'.repeat(40), PLAN_RUN_ID: '9', PLAN_RUN_ATTEMPT: '1', MOCK_MAIN: sha, MOCK_FILES: join(root, 'files.json') };
    const run = (script, o = {}) => spawnSync(process.execPath, ['--import', mock, join(source, `ci/${script}`)], { cwd: root, env: { ...env, ...o }, encoding: 'utf8' });
    const gen = run('plan-metadata.mjs'); assert.equal(gen.status, 0, gen.stderr);
    const metaPath = join(root, 'artifacts/metadata.json'); const original = JSON.parse(readFileSync(metaPath, 'utf8'));
    await t.test('metadata fingerprints the request files', () => { assert.deepEqual(Object.keys(original.requests_sha256), ['billing-api.json']); assert.equal(run('verify-plan.mjs').status, 0); });
    for (const [label, rel, add] of [['changed request', 'infra/live/prod/requests/billing-api.json', ' '], ['new request', 'infra/live/prod/requests/other.json', '{"apps":{}}'], ['changed backend', 'infra/live/prod/backend.tfbackend', 'x'], ['changed platform tfvars', 'infra/live/prod/prod.auto.tfvars', 'x'], ['changed plan bytes', 'artifacts/approved.tfplan', 'x'], ['changed lockfile', 'infra/live/prod/.terraform.lock.hcl', 'x']]) {
      await t.test(`rejects ${label}`, () => { const p = join(root, rel); const before = (() => { try { return readFileSync(p); } catch { return null; } })(); writeFileSync(p, Buffer.concat([before ?? Buffer.alloc(0), Buffer.from(add)])); assert.notEqual(run('verify-plan.mjs').status, 0); if (before) writeFileSync(p, before); else rmSync(p); });
    }
    for (const [label, change] of [['expired plan', { created_at: '2000-01-01T00:00:00Z' }], ['different tree', { planned_tree: 'b'.repeat(40) }]]) {
      await t.test(`rejects ${label}`, () => { writeFileSync(metaPath, JSON.stringify({ ...original, ...change })); assert.notEqual(run('verify-plan.mjs').status, 0); writeFileSync(metaPath, JSON.stringify(original)); });
    }
    await t.test('rejects changed tool version and superseded main', () => { for (const o of [{ TEST_TOFU_VERSION: '1.11.0' }, { MOCK_MAIN: 'c'.repeat(40) }]) assert.notEqual(run('verify-plan.mjs', o).status, 0); });
    for (const [file, required] of [['ssip/crates/edgefactory-tui/src/ui.rs', false], ['README.md', false], ['infra/live/prod/requests/billing-api.json', true], ['infra/modules/app-stack/main.tf', true], ['.github/workflows/apply.yml', true]]) {
      await t.test(`routes ${file} to deployment=${required}`, () => { writeFileSync(env.MOCK_FILES, JSON.stringify([{ filename: file }])); writeFileSync(env.GITHUB_OUTPUT, ''); const r = run('deployment-changes.mjs'); assert.equal(r.status, 0, r.stderr); assert.equal(readFileSync(env.GITHUB_OUTPUT, 'utf8').trim(), `required=${required}`); });
    }
  } finally { rmSync(root, { recursive: true, force: true }); }
});
