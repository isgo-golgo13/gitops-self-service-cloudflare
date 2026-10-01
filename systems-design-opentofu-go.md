# Systems design — Edge Factory GO (OpenTofu under the Flux tofu-controller, GitOps)

## 1. Purpose and position

GO is the migration target for NGO: identical `infra/`, `platform/` and `ssip/`, but the control
loop is a Kubernetes controller that never stops. It satisfies all four OpenGitOps properties:
declarative (the same OpenTofu), versioned and immutable (an OCI artifact by digest), pulled
automatically (Flux `OCIRepository`), continuously reconciled (the `Terraform` CR every five minutes).

## 2. Components

| Component | Responsibility | Where |
| --- | --- | --- |
| Control-plane cluster | A small k3s (or any) cluster you already run; hosts the loop, not the app | your kubeconfig context |
| Day-0 root | Installs Flux via Helm, the registry pull secret, and a two-template bootstrap chart (`OCIRepository` + `Kustomization`) | `infra/live/control-plane/flux`, `infra/modules/flux-tofu-controller` |
| Flux | `source-controller` pulls `ghcr.io/<you>/edge-factory-go/tree:main` every minute; `kustomize-controller` applies three `Kustomization`s in `dependsOn` order | `gitops/flux-system` |
| tofu-controller | `Terraform` CR `prod-edge`: path `./infra/live/prod`, interval 5m, `approvePlan: auto`, R2 backend override, `destroyResourcesOnDeletion: false` | `gitops/workloads/edge-factory` |
| External Secrets | OpenBao Kubernetes auth → Secrets `cloudflare-credentials`, `state-credentials`, `tofu-encryption` (the encryption block composed server-side) | `gitops/platform/secrets` |
| Profile layer + modules | Byte-identical with NGO | `infra/modules` |
| Release mechanism | `make push` = `flux push artifact` of the committed tree, tagged `:main` | `Makefile`, pre-push hook |

## 3. Data flow

1. Day 0: `make bootstrap` (needs `TF_VAR_registry`, `TOFU_ENCRYPTION_KEY`). Flux comes up, pulls the artifact, installs tofu-controller and External Secrets, syncs secrets, creates `prod-edge`.
2. Day 1: the terminal writes `requests/<name>.json`; commit; `make push` (refuses a dirty tree).
3. Within a minute Flux has the new artifact revision; within five the controller runs a runner pod: `tofu init` (R2), refresh from Cloudflare, plan, apply if non-empty, write `apps` and `wrangler_bindings` into the `prod-edge-outputs` Secret.
4. Drift: a dashboard change is reverted at the next reconcile and recorded in the readable plan and the CR status.
5. The app team reads the outputs Secret for bindings and runs `wrangler deploy`.

## 4. Controls

- `approvePlan: auto` by default; a manual-approval variant (plan, then a human commits the plan id) is the alternative for change windows.
- `flux suspend terraform prod-edge` freezes the loop; `resume` continues.
- `prune: true` on the Kustomizations removes objects whose files were deleted; the CR's `destroyResourcesOnDeletion: false` means deleting the CR never destroys Cloudflare resources.
- The artifact is the only runtime source; the cluster holds no Git credential.

## 5. Security model

- No pipeline identity exists: no GitHub OIDC roles, no runners.
- OpenBao Kubernetes-auth tokens are 15-minute and scoped to the `external-secrets` ServiceAccount.
- The Cloudflare token is edit-scoped to the listed zones; the R2 token is scoped to the state bucket.
- State and plans are encrypted with a pbkdf2 passphrase the runner receives as a composed `TF_ENCRYPTION` block; no key appears in Git, in the artifact or in a Kubernetes object other than the Secret ESO manages.

## 6. Failure modes

| Condition | Behaviour |
| --- | --- |
| Artifact push with uncommitted changes | `make push` refuses |
| Runner cannot reach R2 or Cloudflare | reconcile fails, retried (`retryInterval`), CR shows the error; nothing partially applied beyond OpenTofu's own semantics |
| Secret rotation | ESO refreshes within 15 minutes; next reconcile uses the new value |
| Bad change merged | applied within five minutes — the gate is `make check`/`make test` and branch protection; use manual approval for production change windows |

## 7. Verified / not verified

`tofu validate` for `prod` and the control-plane root; every manifest parses; 5 consistency tests
(CR paths exist, referenced Secrets are produced, backend override equals `backend.tfbackend`,
chart/runner versions agree, no placeholders). Not yet run on a live cluster: the tf-runner's backend
override against R2 and the Flux/ESO chart installs on your k3s.
