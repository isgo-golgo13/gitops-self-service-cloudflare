# edge-factory-xp — gitless GitOps: Cloudflare app stacks as a Crossplane 2 self-service API

> New here? Read **docs/TUTORIAL.md** first. `platform/` and `ssip/` are shared with the OpenTofu variants.

App teams ask for an `AppStack` in the SSIP terminal; the service validates against the XRD,
stores the request as a cosign-signed OCI artifact (freight) in GHCR, and applies the XR;
Crossplane composes the Worker shell, R2, D1/Hyperdrive, Access, WAF and LB through the
**official** `crossplane-contrib/provider-upjet-cloudflare`, per resource, forever. Nothing reads
git at runtime; nothing writes a state file.

## Three layers — why there are `.tf` files in a Crossplane repo

| Layer | Tool | Runs | Where |
| --- | --- | --- | --- |
| **Substrate** | OpenTofu: `infra/live/control-plane/crossplane` installs Crossplane on your k3s | once (`make bootstrap`) | k3s |
| **Product** | Crossplane: `platform/` — the `AppStack` XRD (= `spec.schema.json`), the Composition (= the `app-stack` profile table), EnvironmentConfigs, policy | per request, reconciled per resource | k3s → Cloudflare |
| **Code + containers** | `wrangler deploy` using `status.wranglerBindings` | per app release | app pipeline |

A controller cannot install the cluster it is not yet on; OpenTofu does that once and goes dormant.

## The Worker shell under Crossplane

`provider-upjet-cloudflare` exposes `WorkersScript` (not the newer `cloudflare_worker` parent). The
Composition creates it with a placeholder module and **`managementPolicies` without `Update`**, so
wrangler deploys never get reverted: Crossplane owns existence and deletion; the app team owns code.

## Front door

```
make bootstrap && make build && make publish && make register    # day 0
make request            # terminal in xp mode;  ./ssip/target/release/edgefactory --demo  works offline
make status
```
