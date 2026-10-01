// Repository consistency: the GitOps tree and the OpenTofu tree must agree with each other.
// No cluster, no network, nothing in /tmp.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync, existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const read = p => readFileSync(join(root, p), 'utf8');
const yamlFiles = dir => readdirSync(join(root, dir)).filter(f => f.endsWith('.yaml')).map(f => join(dir, f));

test('every Terraform CR path exists under infra/live', () => {
  for (const f of yamlFiles('gitops/workloads/edge-factory')) {
    const m = read(f).match(/^\s*path:\s*(\S+)/m);
    if (m) assert.ok(existsSync(join(root, m[1])), `${f}: ${m[1]} missing`);
  }
});

test('every secret a runner pod references is produced by an ExternalSecret', () => {
  const produced = new Set(yamlFiles('gitops/platform/secrets').flatMap(f => [...read(f).matchAll(/kind: ExternalSecret[\s\S]*?metadata:\s*\{?\s*name:\s*([\w-]+)/g)].map(m => m[1])));
  for (const f of yamlFiles('gitops/workloads/edge-factory')) {
    for (const [, name] of read(f).matchAll(/secretRef:\s*\n\s+name:\s*(\S+)/g)) {
      assert.ok(produced.has(name), `${f}: secret ${name} has no ExternalSecret`);
    }
  }
});

test('the CR backend override equals infra/live/prod/backend.tfbackend', () => {
  const cr = read('gitops/workloads/edge-factory/terraform-prod-edge.yaml');
  const backend = read('infra/live/prod/backend.tfbackend');
  for (const [, key, value] of backend.matchAll(/^(\w+)\s*=\s*(.+)$/gm)) {
    assert.ok(cr.includes(value.trim()), `backend override lacks ${key} = ${value.trim()}`);
  }
});

test('tofu-controller chart, controller image and runner image share one version', () => {
  const hr = read('gitops/platform/controllers/tofu-controller.yaml');
  const chart = hr.match(/version:\s*"(\d+\.\d+\.\d+)"/)[1];
  const tags = [...hr.matchAll(/tag:\s*v(\d+\.\d+\.\d+)/g)].map(m => m[1]);
  const runner = read('gitops/workloads/edge-factory/terraform-prod-edge.yaml').match(/tf-runner:v(\d+\.\d+\.\d+)/)[1];
  assert.deepEqual(new Set([chart, ...tags, runner]), new Set([chart]));
});

test('placeholder hosts use internal.template-domain and never example', () => {
  const walk = d => readdirSync(join(root, d), { withFileTypes: true }).flatMap(e =>
    e.name.startsWith('.') || e.name === 'node_modules' ? [] : e.isDirectory() ? walk(join(d, e.name)) : [join(d, e.name)]);
  for (const f of walk('.')) {
    if (/\.(svg|pptx|png|lock)$/.test(f) || f.endsWith('consistency.test.mjs')) continue;
    assert.ok(!/internal\.example|template-domain|vsphere/.test(read(f)), `${f}: still carries a VMware-era placeholder`);
  }
});
