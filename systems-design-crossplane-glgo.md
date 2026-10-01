# Systems design — Edge Factory XP (Crossplane 2 self-service, gitless GitOps)

## 1. Purpose and position

XP is the self-service target. App teams never touch OpenTofu or a managed resource: they submit
an `AppStack` through the SSIP terminal; the platform's pre-registered XRD + Composition pair turns
it into Cloudflare resources; Crossplane reconciles each resource forever. Nothing reads Git at
runtime; every request is a signed OCI artifact before it is a Kubernetes object.

## 2. Day 0 / day 1 split (the whole model)

| | Day 0 — platform engineering | Day 1 — app teams |
| --- | --- | --- |
| Does | `make bootstrap` (Crossplane on k3s), authors the XRD + Composition, `make build/publish/register` | `edgefactory --mode xp`: six fields, review, Enter |
| Owns | The API surface, the opinions table (EnvironmentConfigs), provider config, policy, the SSIP service | Their namespace, their `AppStack`s, their code (`wrangler deploy`) |
| Cannot | — | create managed resources, choose a Cloudflare attribute, bypass the SSIP identity |

Self-service has a ceiling, and the ceiling is the inventory: only what day 0 registered can be
served on day 1.

## 3. Components

| Component | Responsibility | Where |
| --- | --- | --- |
| XRD `AppStack` (v2, namespaced) | The friendly spec as an OpenAPI v3 schema with CEL rules; `status` is the window back (hostname, worker, data IDs, wrangler bindings) | `platform/apis/xrd-appstack.yaml` |
| Composition (pipeline) | `function-environment-configs` → `function-go-templating` → `function-auto-ready`; the opinions table rendered into MRs | `platform/apis/composition-appstack.yaml` |
| EnvironmentConfigs | `edgefactory-platform` (account, region, identity domains, tiers), one `zone-<zone>` per zone | `platform/environment` |
| Provider | Official `crossplane-contrib/provider-upjet-cloudflare` (namespaced `.m` groups); ten kinds activated | `platform/policy/mrd-activation.yaml`, `runtime/provider-config.yaml` |
| Guards | RBAC by kind; `ValidatingAdmissionPolicy` (SSIP identity + freight digest); ImageConfig with cosign verification | `platform/policy`, `runtime/imageconfig.yaml` |
| SSIP service (Axum) | OIDC verify → validate against the live XRD → push signed freight (ORAS + cosign) → server-side apply the XR | `ssip/crates/edgefactory-ssip` |
| SSIP terminal (ratatui) | Catalog from the cluster; choose / origins / review / watch | `ssip/crates/edgefactory-tui` |
| Day-0 substrate | OpenTofu installs Crossplane 2.4 with signature verification on a k3s; dormant afterwards | `infra/live/control-plane/crossplane` |

## 4. Request lifecycle

1. The terminal fetches the catalog (XRD spec schema, zones, tiers) from the SSIP service.
2. On Enter the service: verifies the OIDC bearer; validates `spec` against the XRD's OpenAPI v3 schema; pushes `{config: requester, time, schema digest; layer: the XR YAML}` to GHCR and cosign-signs it; applies the `AppStack` with annotations `…/freight: sha256:…` and `…/requested-by`.
3. The API server admits only from the SSIP ServiceAccount with a well-formed digest (admission policy).
4. Crossplane: XRD registered for the kind? Composition whose `compositeTypeRef` matches? Load EnvironmentConfigs by label (`zone`, the platform table); render MRs; set `Ready` when all are ready.
5. The provider reconciles each MR against Cloudflare on its poll interval; `status.atProvider` is the truth.
6. Status flows back into the XR; the terminal shows it and the wrangler bindings.

## 5. The Worker shell under Crossplane

The upjet provider exposes `WorkersScript` (not the newer `cloudflare_worker` parent). The
Composition creates it with a placeholder module and `managementPolicies: [Observe, Create, Delete,
LateInitialize]` — no `Update` — so a `wrangler deploy` is never reverted. Crossplane owns existence
and deletion; the app team owns the version. Containers ship inside the version via wrangler.

## 6. Security model

- Everything the cluster trusts is OCI and signed: the Configuration package, the SSIP image, every request. Signature verification runs at pull time (`ImageConfig`; alpha, enabled deliberately).
- No human can create an MR: RBAC exposes only `appstacks`; admission requires the SSIP identity; only ten provider kinds exist in the cluster.
- The Cloudflare token reaches the provider through External Secrets from OpenBao; the SSIP service reaches GHCR and the cosign key the same way.
- No state file, no lock, nothing to back up.

## 7. Failure modes

| Condition | Behaviour |
| --- | --- |
| Request fails schema | 422 from the service; nothing pushed, nothing applied |
| GHCR unavailable | freight push fails; nothing applied — the ledger is a precondition |
| Human `kubectl apply` of an `AppStack` | denied by the admission policy |
| A kind nobody registered | rejected by the API server (no XRD) |
| Dashboard change to a composed resource | corrected on the next observe of that one MR |
| Composition change | new package version; existing XRs re-render on their next reconcile |

## 8. Verified / not verified

Every manifest parses; 7 consistency tests (XRD = `spec.schema.json` = `app-stack` inputs; MR kinds =
activation policy; SSIP constants = admission policy; dependencies declared; no placeholders). Not yet
run on a live cluster: the Composition against the provider's generated field names (`readReplication`,
`caching`, `origin` shapes), the SSIP service's kube/OCI paths, ImageConfig's key-based authority shape.
