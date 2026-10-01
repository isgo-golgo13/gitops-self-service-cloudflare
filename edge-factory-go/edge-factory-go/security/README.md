# Secrets (GO)
The k3s cluster authenticates to OpenBao with the Kubernetes auth method (mount `kubernetes-k3s-edge`,
role `edge-factory`, ServiceAccount `external-secrets` in `flux-system`). External Secrets syncs
`secret/data/cloudflare/prod/apply` → `cloudflare_api_token`, `r2_state_access_key`, `r2_state_secret_key`,
`tofu_encryption_key`, optional `postgres_origins` (JSON). Token scopes: see edge-factory-ngo/security/README.md.
