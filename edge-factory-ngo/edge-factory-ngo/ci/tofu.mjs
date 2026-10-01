// Runs `tofu` in the prod root with the reviewed backend file and in-memory encryption config.
import { readFileSync } from 'node:fs';
import { randomBytes } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { root, infra, assert } from './common.mjs';

const args = process.argv.slice(2);
const local = process.env.TOFU_LOCAL_VALIDATION === '1';
if (local) {
  assert(['init', 'validate', 'providers'].includes(args[0]), 'Local validation cannot read or write state');
  if (args[0] === 'init') assert(args.includes('-backend=false'), 'Local init requires -backend=false');
} else if (args[0] === 'init' && !args.includes('-backend=false')) {
  args.push('-backend-config=backend.tfbackend');
}
const key = process.env.TOFU_ENCRYPTION_KEY || (local ? randomBytes(32).toString('hex') : '');
assert(/^[0-9a-f]{64}$/.test(key), 'Provide a 32-byte encryption key as 64 lowercase hex characters');
const encryption = readFileSync(`${root}/ci/encryption.hcl`, 'utf8').replace('__ENCRYPTION_KEY__', key);
const result = spawnSync('tofu', args, {
  cwd: infra,
  env: { ...process.env, TF_ENCRYPTION: encryption, TF_IN_AUTOMATION: 'true', TF_INPUT: 'false' },
  stdio: 'inherit'
});
if (result.error) throw result.error;
process.exit(result.status ?? 1);
