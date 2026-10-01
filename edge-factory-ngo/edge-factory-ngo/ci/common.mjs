import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, appendFileSync } from 'node:fs';

export const root = process.cwd();
export const infra = `${root}/infra/live/prod`;
export const tfvars = `${infra}/prod.auto.tfvars`;
export const backend = `${infra}/backend.tfbackend`;
export const json = (path) => JSON.parse(readFileSync(path, 'utf8'));
export const sha256 = (path) => createHash('sha256').update(readFileSync(path)).digest('hex');
export const targetDigest = () => sha256(backend);
export const git = (...args) => execFileSync('git', args, { encoding: 'utf8' }).trim();
export const requireEnv = (key) => { if (!process.env[key]) throw new Error(`Missing required environment variable: ${key}`); return process.env[key]; };
export const assert = (condition, message) => { if (!condition) throw new Error(message); };
/** Digest of every app request file, keyed by file name: the reviewed self-service inputs. */
export function requestFingerprints() {
  const dir = `${infra}/requests`;
  return Object.fromEntries(readdirSync(dir).filter(f => f.endsWith('.json')).sort().map(f => [f, sha256(`${dir}/${f}`)]));
}
export function output(key, value) {
  assert(!/[\r\n]/.test(String(value)), 'Multiline output is not accepted');
  appendFileSync(requireEnv('GITHUB_OUTPUT'), `${key}=${value}\n`);
}
export async function github(path) {
  const response = await fetch(`${requireEnv('GITHUB_API_URL')}${path}`, {
    headers: { Authorization: `Bearer ${requireEnv('GH_TOKEN')}`, Accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28' },
    signal: AbortSignal.timeout(30000)
  });
  assert(response.ok, `GitHub API request failed (${response.status})`);
  return response.json();
}
