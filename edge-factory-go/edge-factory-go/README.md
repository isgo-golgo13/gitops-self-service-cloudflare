# edge-factory-go — the same Cloudflare app stacks, reconciled by the Flux tofu-controller

> New here? Read **docs/TUTORIAL.md** first. Shares `infra/`, `platform/` and `ssip/` with edge-factory-ngo byte for byte.

The control loop is a controller on a small k3s you already run: every five minutes it refreshes
from Cloudflare, plans `infra/live/prod` from the OCI artifact of this repository, applies if the
plan is not empty, and corrects drift. No pipeline. Requests are still `requests/<name>.json`
written by the SSIP terminal; the "release" is `make push`.

## Three layers

| Layer | Tool | Runs | Where |
| --- | --- | --- | --- |
| **Substrate** | OpenTofu: `infra/live/control-plane/flux` installs Flux + the bootstrap source on your k3s | once | k3s |
| **Shell + data** | OpenTofu under the tofu-controller: `infra/modules/*` via the `app-stack` profile layer | every 5 min | k3s → Cloudflare |
| **Code + containers** | `wrangler deploy` from the app repo, using `wrangler_bindings` output | per app release | app pipeline |

## Front door

```
make request      # the terminal writes infra/live/prod/requests/<name>.json
git commit && make push         # the committed tree becomes the OCI artifact Flux pulls
make status                      # Flux sources/kustomizations, the Terraform CR
make bootstrap                   # day 0 only
```

The `Terraform` CR (`gitops/workloads/edge-factory/terraform-prod-edge.yaml`): interval 5m,
`approvePlan: auto`, R2 backend override, credentials from OpenBao via External Secrets,
`destroyResourcesOnDeletion: false`. Secrets: `security/README.md`.
