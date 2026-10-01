# Edge Factory NGO — the tutorial (start here)

## 0. In plain words
You tell the platform six things about an app (name, hostname, tier, exposure, database, objects).
The platform turns that into a Worker shell on your domain with WAF and rate limits, plus the data
it needs (R2, D1 or Hyperdrive→Postgres) and, for private apps, a Zero Trust login. You deploy your
Rust code — and the container it runs in — with `wrangler`, into the shell. Nothing provider-shaped
ever appears in your request: the **profile layer** (`infra/modules/app-stack`) is the translator.

## 1. One-time setup
1. A Cloudflare account, one or more zones (`dronegrid.io`), Workers Paid plan if you use Containers.
2. A dedicated R2 bucket `edge-factory-state` + an R2 API token scoped to it (state only).
3. Two scoped Cloudflare API tokens (plan = read, apply = edit; scopes in `security/README.md`).
4. OpenBao with JWT auth trusting GitHub; load `security/openbao-*`; put the tokens and a 64-hex
   `tofu_encryption_key` in `secret/cloudflare/prod/{plan,apply}`.
5. Set `account_id` in `infra/live/prod/prod.auto.tfvars`, the R2 endpoint in `backend.tfbackend`.
6. Branch protection on `main`: review + required check `Deployment PR gate`.

## 2. Ask for an app
```
make request        # Tab/↑↓/Enter; review shows the JSON that will be written
```
It writes `infra/live/prod/requests/<name>.json`. Commit it, open a PR.

## 3. What the PR does
`plan.yml`: classify → check (fmt, validate, lint, 16 guard tests, cargo check; no credentials) →
plan (OIDC→OpenBao 15-min token → scoped Cloudflare token → `tofu plan` against R2 → encrypted plan +
provenance metadata, now fingerprinting every request file) → required gate. Read the plan: it lists
exactly the Cloudflare objects the profile layer chose for your tier and exposure. Merge.

## 4. What the merge does
`apply.yml` downloads *that* plan, runs the 13 provenance checks (tree, lockfile, platform tfvars,
backend, request files, plan bytes, age, newer main, tool version, run identity), gets a fresh
token, applies. Refusal = new PR.

## 5. Deploy your code
```
make bindings        # artifacts/bindings/<app>.json
```
Paste the bindings into your app's `wrangler.jsonc` (`d1_databases`, `r2_buckets`, `hyperdrive`,
`containers`, `routes`) and `wrangler deploy`. The Worker shell already owns the hostname.

## 6. Day 2
| I want to… | Do this |
| --- | --- |
| change tier / exposure / database | edit the request file, PR (the plan shows adds/changes) |
| add an app | `make request` again |
| remove an app | delete its request file, PR (plan shows destroys — read it) |
| change the platform's opinions | edit `local.tier` in `app-stack/main.tf`; one PR changes every app |
| rotate the Cloudflare token | OpenBao only |

## 7. Why this is air-tight, secure and modern
- App teams see six fields; the provider surface (dozens of resources) is behind a map they can read.
- Scoped, short-lived tokens; nothing in GitHub; plan and apply have different token scopes.
- Apply applies exactly what was reviewed — 13 checks, now including every request file.
- WAF, rate limits, TLS floor and Access are not optional per app: they come with the tier.
- Platform owns shell + data; wrangler owns code + containers. Cloudflare designed that split.
- Same six fields are the XRD in edge-factory-xp: the loop can change, the API does not.
