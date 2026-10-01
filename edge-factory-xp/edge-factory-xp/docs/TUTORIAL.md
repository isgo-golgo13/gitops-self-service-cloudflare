# Edge Factory XP — the tutorial (start here)

## 1. Day 0 (platform engineering)
```
export TF_VAR_registry='{host="ghcr.io",…}' TF_VAR_cosign_public_key="$(cat cosign.pub)" TOFU_ENCRYPTION_KEY=<64 hex>
make bootstrap           # Crossplane 2.4 on the k3s, signature verification on
make build && make publish   # Configuration (XRD + Composition) + SSIP image, cosign-signed, to GHCR
make register            # ImageConfig, Configuration (pulls provider-upjet-cloudflare + 3 functions), policy, EnvironmentConfigs, ProviderConfig, SSIP
```
Fill `platform/environment/envconfig-platform.yaml` (account id, region, identity domains) and one
`envconfig-zone-<zone>.yaml` per zone (zone id).

## 2. Day 1 (an app team)
```
make request             # or: ./ssip/target/release/edgefactory --demo
```
Choose → (origins if hybrid) → review → Enter. The service verifies your SSO token, validates against
the XRD, pushes the request as signed freight to GHCR, applies the `AppStack`; the admission policy
admits it only from the SSIP identity with a digest. Watch shows hostname, Worker, D1/R2/Hyperdrive,
Access, LB and — once Ready — the wrangler bindings to paste.

## 3. Deploy code
`wrangler deploy` into the shell. Crossplane never updates the script (`managementPolicies` without
Update), so your deploy is never reverted; it still owns existence and deletion.

## 4. Why this is air-tight
No MRs by hand (RBAC + admission policy + MR activation of ten Cloudflare kinds only); every request
a signed, immutable OCI artifact; per-resource reconciliation; no state file; official upjet provider.
