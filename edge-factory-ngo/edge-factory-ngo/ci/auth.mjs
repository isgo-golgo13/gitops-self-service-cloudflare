// GitHub OIDC JWT -> short-lived OpenBao token (JWT auth) -> the phase secret:
// a SCOPED Cloudflare API token, the R2 state key pair, and the state-encryption secret.
import { appendFileSync } from 'node:fs';
import { requireEnv, assert } from './common.mjs';

function expose(key, value) {
  assert(typeof value === 'string' && value.length > 0, `Missing secret field: ${key}`);
  assert(!/[\r\n]/.test(value), `Secret ${key} must be a single-line value`);
  console.log(`::add-mask::${value.replaceAll('%', '%25')}`);
  appendFileSync(requireEnv('GITHUB_ENV'), `${key}=${value}\n`);
}
async function request(url, options) {
  const response = await fetch(url, { ...options, signal: AbortSignal.timeout(30000) });
  assert(response.ok, `Identity service request failed (${response.status})`);
  return response.json();
}
const address = requireEnv('BAO_ADDR').replace(/\/$/, '');
assert(address.startsWith('https://'), 'OpenBao requires HTTPS');
const headers = { 'Content-Type': 'application/json' };
if (process.argv[2] === 'revoke') {
  if (process.env.BAO_TOKEN) {
    const response = await fetch(`${address}/v1/auth/token/revoke-self`, { method: 'POST', headers: { ...headers, 'X-Vault-Token': process.env.BAO_TOKEN }, signal: AbortSignal.timeout(30000) });
    assert(response.ok, `OpenBao token revocation failed (${response.status})`);
  }
} else {
  const oidcUrl = new URL(requireEnv('ACTIONS_ID_TOKEN_REQUEST_URL'));
  oidcUrl.searchParams.set('audience', requireEnv('BAO_AUDIENCE'));
  const oidc = await request(oidcUrl, { headers: { Authorization: `Bearer ${requireEnv('ACTIONS_ID_TOKEN_REQUEST_TOKEN')}` } });
  const login = await request(`${address}/v1/auth/jwt/login`, { method: 'POST', headers, body: JSON.stringify({ role: requireEnv('BAO_ROLE'), jwt: oidc.value }) });
  const token = login.auth.client_token;
  expose('BAO_TOKEN', token);
  const secret = await request(`${address}/v1/${requireEnv('BAO_KV_PATH')}`, { headers: { ...headers, 'X-Vault-Token': token } });
  const f = secret.data.data;
  expose('CLOUDFLARE_API_TOKEN', f.cloudflare_api_token);
  expose('AWS_ACCESS_KEY_ID', f.r2_state_access_key);
  expose('AWS_SECRET_ACCESS_KEY', f.r2_state_secret_key);
  expose('TOFU_ENCRYPTION_KEY', f.tofu_encryption_key);
  if (f.postgres_origins) expose('TF_VAR_postgres_origins', f.postgres_origins);
}
