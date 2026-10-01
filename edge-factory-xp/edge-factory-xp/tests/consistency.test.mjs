// The platform package, the policy, the spec schema and the SSIP must agree. No cluster, no network.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const read = p => readFileSync(join(root, p), 'utf8');
const walk = d => readdirSync(join(root, d), { withFileTypes: true }).flatMap(e =>
  e.name.startsWith('.') || e.name === 'target' ? [] : e.isDirectory() ? walk(join(d, e.name)) : [join(d, e.name)]);

const xrd = read('platform/apis/xrd-appstack.yaml');
const comp = read('platform/apis/composition-appstack.yaml');
const lib = read('ssip/crates/edgefactory-core/src/lib.rs');
const vap = read('platform/policy/vap-freight.yaml');

test('XRD is Crossplane v2, namespaced, no claims', () => {
  assert.match(xrd, /apiVersion: apiextensions\.crossplane\.io\/v2/);
  assert.match(xrd, /scope: Namespaced/);
  assert.doesNotMatch(xrd, /claimNames/);
});

test('Composition targets the XRD and is pipeline mode', () => {
  const group = xrd.match(/^\s+group:\s*(\S+)/m)[1];
  const kind = xrd.match(/^\s+kind:\s*(\S+)/m)[1];
  assert.match(comp, new RegExp(`apiVersion: ${group.replace(/\./g, '\\.')}/v1alpha1`));
  assert.match(comp, new RegExp(`kind: ${kind}`));
  assert.match(comp, /mode: Pipeline/);
});

test('every function the Composition calls is a package dependency', () => {
  const deps = read('platform/crossplane.yaml');
  for (const [, fn] of comp.matchAll(/functionRef:\s*\{\s*name:\s*crossplane-contrib-(\S+?)\s*\}/g)) {
    assert.ok(deps.includes(`/${fn}`), `${fn} missing from platform/crossplane.yaml`);
  }
});

test('MR kinds rendered by the Composition are the ones the activation policy enables', () => {
  const mrap = read('platform/policy/mrd-activation.yaml');
  for (const [, api, kind] of comp.matchAll(/apiVersion: (\S+\.cloudflare\.m\.crossplane\.io)\/v1alpha1\s*\n\s*kind: (\S+)/g)) {
    const plural = kind.toLowerCase().endsWith('y') ? kind.toLowerCase().slice(0, -1) + 'ies' : kind.toLowerCase() + 's';
    assert.ok(mrap.includes(`${plural}.${api}`), `${kind} (${api}) not activated`);
  }
});

test('SSIP constants match the XRD and the admission policy', () => {
  const group = xrd.match(/^\s+group:\s*(\S+)/m)[1];
  const plural = xrd.match(/^\s+plural:\s*(\S+)/m)[1];
  assert.ok(lib.includes(`pub const GROUP: &str = "${group}"`));
  assert.ok(lib.includes(`pub const PLURAL: &str = "${plural}"`));
  for (const ann of ['freight', 'requested-by']) {
    assert.ok(lib.includes(`"${group}/${ann}"`), `annotation ${ann} missing in lib.rs`);
    assert.ok(vap.includes(`'${group}/${ann}'`), `annotation ${ann} missing in the admission policy`);
  }
  assert.ok(vap.includes('system:serviceaccount:edgefactory-system:edgefactory-ssip'));
  assert.ok(read('platform/runtime/ssip.yaml').includes('name: edgefactory-ssip'));
});

test('XRD spec equals platform/spec.schema.json (one API, two renderings)', () => {
  const schema = JSON.parse(read('platform/spec.schema.json'));
  for (const k of Object.keys(schema.properties)) assert.match(xrd, new RegExp(`^\\s+${k}:`, 'm'), `XRD lacks ${k}`);
  for (const e of schema.properties.tier.enum) assert.match(read('platform/environment/envconfig-platform.yaml'), new RegExp(`^\\s+${e}:`, 'm'));
  const tfvars = read('../ngo/infra/modules/app-stack/variables.tf');
  for (const k of Object.keys(schema.properties)) assert.match(tfvars, new RegExp(`\\b${k}\\s*=`), `app-stack variables.tf lacks ${k}`);
});

test('no VMware-era placeholders survive', () => {
  for (const f of walk('.')) {
    if (/\.(svg|pptx|png|lock)$/.test(f) || f.endsWith('consistency.test.mjs')) continue;
    assert.ok(!/internal\.example|template-domain|vsphere|vmfactory/i.test(read(f)), `${f}: still carries a VMware-era placeholder`);
  }
});
