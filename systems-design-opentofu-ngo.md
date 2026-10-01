# Systems design — Edge Factory NGO (OpenTofu + GitHub Actions, non-GitOps)

## 1. Purpose and position

NGO is the first delivery: self-service Cloudflare application stacks from a six-field friendly
spec, with the control loop a team already understands — a pull request. It is explicitly **not**
GitOps by the OpenGitOps definition (nothing pulls, nothing reconciles continuously); it is a
production-grade pipeline with exact-plan provenance. GO reuses every file in `infra/` and only
changes who runs OpenTofu.

## 2. Components

| Component | Responsibility | Where |
| --- | --- | --- |
| Friendly spec | The user-facing API: zone, hostname, tier, exposure, database, objects, container, origins | `platform/spec.schema.json` |
| SSIP terminal (`edgefactory --mode tofu`) | Renders prompts from the schema, validates, writes `requests/<name>.json` | `ssip/crates/edgefactory-tui` |
| Root | `for_each` over `requests/*.json`; one zone baseline per zone | `infra/live/prod` |
| Profile layer (`app-stack`) | OpenTofu's patch-and-transform: `local.tier` / `local.exposure` maps → provider arguments; outputs status + wrangler bindings | `infra/modules/app-stack` |
| Resource modules | One Cloudflare concern each: `cf-worker`, `cf-r2`, `cf-d1`, `cf-hyperdrive`, `cf-access`, `cf-zone-baseline`, `cf-load-balancer` | `infra/modules/cf-*` |
| Pipeline | `plan.yml` (PR: changes → check → plan → gate), `apply.yml` (merge: select plan → verify → apply) | `.github/workflows`, `ci/` |
| Secrets | GitHub OIDC → OpenBao JWT auth → 15-minute token → scoped Cloudflare API token, R2 keys, encryption secret | `ci/auth.mjs`, `security/` |
| State | Dedicated R2 bucket through the S3 backend, AES-GCM client-side (pbkdf2 passphrase), native S3 lockfile | `backend.tfbackend`, `ci/encryption.hcl` |
| Code deploy | `wrangler deploy` from the app repo using the binding manifest; Containers ship with the Worker version | app repository |

## 3. Data flow

1. App team runs the terminal; the spec is validated against `spec.schema.json`; `requests/<name>.json` is written.
2. PR opened. `deployment-changes.mjs` classifies: only `infra/**` and `apply.yml` need a plan.
3. `check` (no credentials): `tofu fmt/validate`, TFLint, 16 guard tests, `cargo check`.
4. `plan`: OIDC → OpenBao (`edge-prod-plan`) → read-scoped token; `tofu init` against R2; `tofu plan -out`; `metadata.json` records tree hash, lockfile, platform tfvars, backend file, **every request file digest**, plan digest, OpenTofu version, run identity, timestamp.
5. `gate` is the required status check; a human reads the plan — it lists every object the profile layer chose.
6. Merge. `apply.yml` finds the plan run for that exact head, downloads it, runs 13 checks (tree, lockfile, tfvars, backend, requests, plan bytes, age ≤ 24 h, no newer `main`, tool version, repo/PR/run id/attempt), gets an edit-scoped token, applies the saved plan. No `-var-file` at apply.
7. `make bindings` emits `artifacts/bindings/<app>.json`; the app team pastes it into `wrangler.jsonc` and deploys.

## 4. The profile layer (the design's fulcrum)

```
tier     → observability sampling, logpush, D1 read replication, R2 cold-tier days, rate limit rpm, Access session, container max instances
exposure → public: proxied domain + managed WAF + rate limit · private: + Zero Trust Access · hybrid: + load balancer over external origins
database → none | d1 (managed replicated SQLite) | postgres (Hyperdrive config to an external Postgres)
objects  → R2 bucket with multipart cleanup and tier-based cold storage
container→ a `containers` entry in the binding manifest (wrangler deploys it)
```

Every opinion is a map lookup. Changing one line changes every app on the next plan. Rulesets are
per-zone singletons, so the root aggregates all apps' rate limits into one `cf-zone-baseline`.

## 5. Security model

- No secret in GitHub; two non-secret variables (`BAO_ADDR`, `BAO_AUDIENCE`).
- Per-phase OpenBao roles bound to repository id, event, ref and `workflow_ref`; tokens revoked at job end.
- Plan and apply use different Cloudflare token scopes; the plan token cannot write.
- State bucket is dedicated and has its own token; state and plans are encrypted before they reach R2.
- Guards are not optional per app: WAF, rate limits, TLS floor, always-HTTPS come with the tier.

## 6. Failure modes and responses

| Condition | Behaviour |
| --- | --- |
| Something changed between plan and merge | apply refuses (fingerprint mismatch); open a new PR |
| Plan older than 24 h or `main` moved | apply refuses |
| Two apps claim one hostname in a zone | root validation fails at plan |
| `exposure: hybrid` with no origins | module validation fails at plan |
| R2 unreachable | init fails; nothing applied |
| Cloudflare API rejects an argument | plan/apply error with the resource named; fix the profile layer, not the request |

## 7. Verified / not verified

`tofu validate` against cloudflare/cloudflare 5.26.0; a full plan over public+D1+R2+container,
private+Access and hybrid+Hyperdrive+LB evaluates to the API call; 16 guard tests. Not yet applied to
a live account: custom-domain ↔ Worker ordering, managed-ruleset IDs, R2 lockfile semantics.

## 8. Growth

A new opinion = a line in the profile table. A new offering (queue, KV, static site) = a new
`cf-*` module and a field in the schema (additive, defaulted). A new zone = one baseline. The
migration to GO is two files (`control-plane/flux`, the `Terraform` CR) and zero changes in `infra/`.
