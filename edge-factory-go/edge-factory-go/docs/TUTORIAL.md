# Edge Factory GO — the tutorial (start here)

Same request file, same profile layer, same Cloudflare objects as NGO. Different control loop.

## 1. Day 0 (once)
Your k3s kubeconfig context `k3s-edge`; GHCR credential; OpenBao Kubernetes auth mount `kubernetes-k3s-edge`.
```
export TF_VAR_registry='{host="ghcr.io",username="…",password="…"}'
export TOFU_ENCRYPTION_KEY=<64 hex>
make bootstrap           # Flux + OCIRepository(tree:main) + Kustomization(gitops/flux-system)
```
Flux then installs tofu-controller and External Secrets, syncs the Cloudflare token, R2 keys and
encryption block from OpenBao, and creates the `Terraform` CR `prod-edge`.

## 2. Every change
```
make request             # writes requests/<name>.json
git commit -am "billing-api"
make push                # flux push artifact → ghcr.io/…/edge-factory-go/tree:main
make status
```
Within ~6 minutes the controller has planned and applied. `kubectl -n flux-system get terraform prod-edge`.

## 3. Drift
Change something in the Cloudflare dashboard by hand; the next reconcile puts it back and records it.
Intended? Put it in the request file (or the profile layer), commit, push.

## 4. Deploy code
`kubectl -n flux-system get secret prod-edge-outputs -o jsonpath='{.data.wrangler_bindings}' | base64 -d`
→ paste into `wrangler.jsonc` → `wrangler deploy`.

## 5. Why this is air-tight
Continuous convergence (5 min), OCI artifact as the only runtime source, no Git credential in the
cluster, secrets synced from OpenBao with the encryption block composed server-side, same guards as NGO.
