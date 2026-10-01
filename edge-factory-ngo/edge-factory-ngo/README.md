# edge-factory-ngo — Cloudflare app stacks with OpenTofu, delivered by a two-stage GitHub Actions pipeline

> New here? Read **docs/TUTORIAL.md** first.

Self-service Cloudflare application stacks — a Worker shell (fronting a Rust app or a Cloudflare
Container), R2 objects, D1 or Hyperdrive→Postgres, Zero Trust Access, managed WAF and rate
limits, and a load balancer only when there are external origins — from a **six-field friendly
spec**. The platform's opinions live in one profile layer; app teams never see a Cloudflare
attribute. NGO = the control loop is GitHub Actions (PR plan → exact-plan apply).

## Three layers, three cadences

| Layer | Tool | Runs | Where |
| --- | --- | --- | --- |
| **Substrate** | none — a Cloudflare account, GitHub, OpenBao already exist | — | site |
| **Shell + data** (this repo) | OpenTofu: `infra/modules/*` composed by the `app-stack` profile layer | per pull request | `plan.yml` / `apply.yml` |
| **Code + containers** | `wrangler deploy` from the app repository, using the binding manifest this repo outputs | per app release | the app team's pipeline |

Cloudflare's own split: the platform provisions the Worker shell and bindings; the app team deploys
versions. `cloudflare_worker` carries settings, not code, so the two never fight. Containers are
not in the Terraform provider (verified at 5.26.0); they are part of the wrangler deploy.

## The friendly spec = OpenTofu's patch-and-transform

`platform/spec.schema.json` is the user-facing API. `infra/modules/app-stack` is the profile layer:
`local.tier` and `local.exposure` are **maps**, so every opinion is data you can read:

| you write | the platform does |
| --- | --- |
| `tier: small / medium / large` | observability sampling 0.1 / 0.5 / 1; D1 read replication off / on / on; R2 cold-tiering never / 90 d / 180 d; rate limit 300 / 1200 / 6000 rpm; Access session 24 h / 12 h / 8 h; container max instances 2 / 5 / 20 |
| `exposure: public` | proxied custom domain + managed WAF (Cloudflare + OWASP) + per-host rate limit |
| `exposure: private` | all of the above + Zero Trust Access (allowed identity domains) |
| `exposure: hybrid` | all of public + a load balancer with health checks over your `origins` |
| `database: d1` / `postgres` | a D1 database with replication per tier / a Hyperdrive config to your external Postgres |
| `objects: true` | an R2 bucket with multipart cleanup and tier-based cold storage |
| `container: true` | the binding manifest carries a `containers` entry sized per tier |

Requests are files: `infra/live/prod/requests/<name>.json` (`{"apps": {"<name>": {…spec}}}`), written
by the SSIP terminal (`make request`) or by hand; the root merges them with `for_each`. Rulesets are
one-per-zone, so the root aggregates every app's rate limit into one `cf-zone-baseline` per zone.

The same spec is the XRD in `edge-factory-xp`; the same opinions table is its EnvironmentConfig.

## Layout

```
infra/modules/   cf-worker  cf-r2  cf-d1  cf-hyperdrive  cf-access  cf-zone-baseline  cf-load-balancer  app-stack
infra/live/prod/ main.tf (for_each over requests/*.json)  prod.auto.tfvars (platform values)  backend.tfbackend (R2)
platform/        spec.schema.json (the API) + the Crossplane package used by edge-factory-xp (same spec)
ssip/            Rust: edgefactory terminal (--mode tofu writes requests/), service used by XP
containers/      ssip.Containerfile, app.Containerfile (reference Rust app image for Cloudflare Containers)
ci/              OIDC→OpenBao auth, tofu wrapper (R2 backend + encryption), plan provenance (13 checks)
.github/         plan.yml (PR), apply.yml (merge)       security/   OpenBao roles/policies, token scopes
tests/           16 guard tests, no network, nothing in /tmp
```

## Front door

```
make check / make test
make request                  # the terminal: writes infra/live/prod/requests/<name>.json
git add … && git commit && gh pr create       # plan.yml plans, you review, merge → apply.yml applies
make bindings                 # after apply: artifacts/bindings/<app>.json for the app team's wrangler.jsonc
```

## State and secrets

State in a **dedicated R2 bucket** through the S3 backend (`backend.tfbackend`), AES-GCM encrypted
client-side (pbkdf2 passphrase from OpenBao via `TF_ENCRYPTION`). Native locking relies on R2
conditional writes — verify on first `init`. No secret in GitHub: `ci/auth.mjs` exchanges the OIDC
JWT for a 15-minute OpenBao token and reads a **scoped** Cloudflare API token (`security/README.md`).

## Verified / not verified

`tofu validate` passes against cloudflare/cloudflare 5.26.0; a full `tofu plan` covering public+D1+R2,
private+Access, and hybrid+Hyperdrive+LB evaluates to the API call. Not yet applied to a real account:
the custom-domain ↔ Worker ordering, ruleset managed-rule IDs and R2 lockfile behaviour are the first
things to confirm. A free Cloudflare account + one zone is enough to run this end to end.
