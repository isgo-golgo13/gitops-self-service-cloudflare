# gitops/ — what Flux reconciles, in order
`flux-system/` (three Kustomizations with dependsOn) → `platform/controllers` (tofu-controller, External Secrets)
→ `platform/secrets` (OpenBao → Cloudflare token, R2 keys, encryption block) → `workloads/edge-factory` (the `Terraform` CR `prod-edge`).
No image factory: code and containers are deployed by wrangler from the app repository.
