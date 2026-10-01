# Secrets (NGO)

Two OpenBao KV v2 paths, read by `ci/auth.mjs` through GitHub OIDC → OpenBao JWT auth:

| Path | Fields |
| --- | --- |
| `secret/data/cloudflare/prod/plan` | `cloudflare_api_token` (read-only scoped token), `r2_state_access_key`, `r2_state_secret_key`, `tofu_encryption_key` |
| `secret/data/cloudflare/prod/apply` | same fields with a write-scoped token; optional `postgres_origins` (JSON map for Hyperdrive) |

**Cloudflare API token scopes (apply):** Account — Workers Scripts:Edit, Workers R2 Storage:Edit, D1:Edit,
Hyperdrive:Edit, Access: Apps and Policies:Edit, Load Balancing: Monitors and Pools:Edit;
Zone (listed zones only) — Zone:Read, DNS:Edit, Workers Routes:Edit, Zone WAF:Edit, Zone Settings:Edit,
Load Balancers:Edit. The plan token gets the `:Read` counterparts.

**R2 state:** a dedicated bucket `edge-factory-state` and an R2 API token (Object Read & Write on that
bucket only). Never the application buckets.
