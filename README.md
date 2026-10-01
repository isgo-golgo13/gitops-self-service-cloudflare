# gitops-self-service-cloudflare
OpenTofu TF and Crossplane Self-Service TUI CLI driven (using Rust ratatui) workflows for Cloudflare Resources (CF CDN, CF Secret Store, Vault, CF D1 Postgres Storage, CF R2 for application and OpenTofu TF state storage, CF Workers, CF Containers. The project provides self-service architectures across three environments for three different clients at certain levels of automated IaC provisioning stages.

- Non-GitOps (no Kubernetes-native GitOps drift-detection/auto-reconciliation) controller in the hotpath such as OpenTofu Flux Controller and this pipeline runs a low-cognitive higher-friction process of a GitHub Actions CI/CD with short-lived OIDC credentials with no OpenTofu Flux controller (no Terraform and GitRepository/OCIRepository CRDs). **This version** includes with a Rust `ratatui` TUI CLI catalog system to drive the CF infrastructure into the live canvas. This version includes OpenTofu logical mapping of self-service low-friction provider inputs that directly would fail the provider provisioning process unless these low-congitive inputs are mapped internally to actual Cloudflare provider inputs done using a middleware intercepting process of OpenTofu TF map data structures to translate these friendly inputs to the actual OpenTofu TF provider. This process is natively provided in Crossplane using `patch-and-transform` in a day 0 platform engineering step sequence of providing inventories registred to Crossplane as templated Helm charts of OpenAPI v3 spec Crossplane 2.0 XRDs (the namespaced scope OpenAPI YAML interfaces for self-served clients) and their defined controllers that actually do the tethering of Crossplane Cloudflare MR (Managed Resources) through Compositions where the `patch-and-transform` is done. 

The project structure of this first approach is provided here.

```shell
edge-factory-ngo
├── edge-factory-ngo
│   ├── Makefile
│   ├── README.md
│   ├── ci
│   │   ├── auth.mjs
│   │   ├── common.mjs
│   │   ├── deployment-changes.mjs
│   │   ├── encryption.hcl
│   │   ├── lint.sh
│   │   ├── plan-metadata.mjs
│   │   ├── select-plan.mjs
│   │   ├── tofu.mjs
│   │   ├── validate-local.sh
│   │   └── verify-plan.mjs
│   ├── containers
│   │   ├── app.Containerfile
│   │   └── ssip.Containerfile
│   ├── docs
│   │   ├── TUTORIAL.md
│   │   └── svg
│   │       └── ef-ngo.svg
│   ├── dot.gitignore
│   ├── dot.pre-commit-config.yaml
│   ├── dot.tflint.hcl
│   ├── infra
│   │   ├── live
│   │   │   └── prod
│   │   │       ├── backend.tfbackend
│   │   │       ├── main.tf
│   │   │       ├── outputs.tf
│   │   │       ├── prod.auto.tfvars
│   │   │       ├── providers.tf
│   │   │       ├── requests
│   │   │       │   ├── README.md
│   │   │       │   ├── admin.json
│   │   │       │   └── billing-api.json
│   │   │       ├── variables.tf
│   │   │       └── versions.tf
│   │   └── modules
│   │       ├── app-stack
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       ├── cf-access
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       ├── cf-d1
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       ├── cf-hyperdrive
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       ├── cf-load-balancer
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       ├── cf-r2
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       ├── cf-worker
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       └── cf-zone-baseline
│   │           ├── main.tf
│   │           ├── outputs.tf
│   │           └── variables.tf
│   ├── platform
│   │   ├── apis
│   │   │   ├── composition-appstack.yaml
│   │   │   └── xrd-appstack.yaml
│   │   ├── crossplane.yaml
│   │   ├── environment
│   │   │   ├── envconfig-platform.yaml
│   │   │   └── envconfig-zone-dronegrid-io.yaml
│   │   ├── examples
│   │   │   └── appstack.yaml
│   │   ├── policy
│   │   │   ├── mrd-activation.yaml
│   │   │   ├── rbac-app-team.yaml
│   │   │   └── vap-freight.yaml
│   │   ├── runtime
│   │   │   ├── configuration.yaml
│   │   │   ├── imageconfig.yaml
│   │   │   └── provider-config.yaml
│   │   └── spec.schema.json
│   ├── security
│   │   ├── README.md
│   │   ├── openbao-apply-policy.hcl
│   │   ├── openbao-apply-role.json.example
│   │   ├── openbao-plan-policy.hcl
│   │   └── openbao-plan-role.json.example
│   ├── ssip
│   │   ├── Cargo.toml
│   │   ├── Makefile
│   │   └── crates
│   │       ├── edgefactory-core
│   │       │   ├── Cargo.toml
│   │       │   └── src
│   │       │       ├── api.rs
│   │       │       ├── freight.rs
│   │       │       ├── lib.rs
│   │       │       ├── schema.rs
│   │       │       └── xr.rs
│   │       ├── edgefactory-ssip
│   │       │   ├── Cargo.toml
│   │       │   └── src
│   │       │       ├── auth.rs
│   │       │       ├── main.rs
│   │       │       ├── routes.rs
│   │       │       └── state.rs
│   │       └── edgefactory-tui
│   │           ├── Cargo.toml
│   │           └── src
│   │               ├── client.rs
│   │               ├── main.rs
│   │               └── ui.rs
│   └── tests
│       └── ci.test.mjs
├── ppt-slides
│   ├── Edge-Factory-NGO.pptx
│   └── svg
│       └── ef-ngo.svg
└── static-assets
    ├── 01-start.svg
    ├── 02-choose.svg
    ├── 03-review.svg
    ├── 04-dispatch.svg
    ├── 05-provisioning.svg
    └── 06-provisioned.svg
```

- GitOps (a Kubernetes-native GitOps drift-detection/auto-reconciliation) controller in the hotpath such as OpenTofu Flux Controller and this pipeline runs a low-cognitive lower-friction process of a GitHub Actions CI/CD with short-lived OIDC credentials with OpenTofu Flux controller (Terraform and GitRepository/OCIRepository CRDs).
**This version** This version includes with a Rust `ratatui` TUI CLI catalog system to drive the CF infrastructure into the live canvas. This version includes OpenTofu logical mapping of self-service low-friction provider inputs that directly would fail the provider provisioning process unless these low-congitive inputs are mapped internally to actual Cloudflare provider inputs done using a middleware intercepting process of OpenTofu TF map data structures to translate these friendly inputs to the actual OpenTofu TF provider. This process is natively provided in Crossplane using `patch-and-transform` in a day 0 platform engineering step sequence of providing inventories registred to Crossplane as templated Helm charts of OpenAPI v3 spec Crossplane 2.0 XRDs (the namespaced scope OpenAPI YAML interfaces for self-served clients) and their defined controllers that actually do the tethering of Crossplane Cloudflare MR (Managed Resources) through Compositions where the `patch-and-transform` is done.


The project structure of this second approach is provided here.

```shell
edge-factory-go
├── edge-factory-go
│   ├── Makefile
│   ├── README.md
│   ├── ci
│   │   └── encryption.hcl
│   ├── containers
│   │   ├── app.Containerfile
│   │   └── ssip.Containerfile
│   ├── docs
│   │   ├── TUTORIAL.md
│   │   └── svg
│   │       └── ef-go.svg
│   ├── dot.gitignore
│   ├── dot.pre-commit-config.yaml
│   ├── gitops
│   │   ├── README.md
│   │   ├── flux-system
│   │   │   ├── edge-factory.yaml
│   │   │   ├── kustomization.yaml
│   │   │   ├── platform-controllers.yaml
│   │   │   └── platform-secrets.yaml
│   │   ├── platform
│   │   │   ├── controllers
│   │   │   │   ├── external-secrets.yaml
│   │   │   │   ├── helmrepositories.yaml
│   │   │   │   ├── kustomization.yaml
│   │   │   │   └── tofu-controller.yaml
│   │   │   └── secrets
│   │   │       ├── cloudflare-credentials.yaml
│   │   │       ├── clustersecretstore.yaml
│   │   │       ├── kustomization.yaml
│   │   │       ├── state-credentials.yaml
│   │   │       └── tofu-encryption.yaml
│   │   └── workloads
│   │       └── edge-factory
│   │           ├── kustomization.yaml
│   │           └── terraform-prod-edge.yaml
│   ├── infra
│   │   ├── live
│   │   │   ├── control-plane
│   │   │   │   └── flux
│   │   │   │       ├── backend.tfbackend
│   │   │   │       ├── flux.auto.tfvars
│   │   │   │       ├── main.tf
│   │   │   │       ├── outputs.tf
│   │   │   │       ├── providers.tf
│   │   │   │       ├── variables.tf
│   │   │   │       └── versions.tf
│   │   │   └── prod
│   │   │       ├── backend.tfbackend
│   │   │       ├── main.tf
│   │   │       ├── outputs.tf
│   │   │       ├── prod.auto.tfvars
│   │   │       ├── providers.tf
│   │   │       ├── requests
│   │   │       │   ├── README.md
│   │   │       │   ├── admin.json
│   │   │       │   └── billing-api.json
│   │   │       ├── variables.tf
│   │   │       └── versions.tf
│   │   └── modules
│   │       ├── app-stack
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       ├── cf-access
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       ├── cf-d1
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       ├── cf-hyperdrive
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       ├── cf-load-balancer
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       ├── cf-r2
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       ├── cf-worker
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       ├── cf-zone-baseline
│   │       │   ├── main.tf
│   │       │   ├── outputs.tf
│   │       │   └── variables.tf
│   │       └── flux-tofu-controller
│   │           ├── chart
│   │           │   ├── Chart.yaml
│   │           │   ├── templates
│   │           │   │   ├── source.yaml
│   │           │   │   └── sync.yaml
│   │           │   └── values.yaml
│   │           ├── main.tf
│   │           ├── outputs.tf
│   │           └── variables.tf
│   ├── platform
│   │   ├── apis
│   │   │   ├── composition-appstack.yaml
│   │   │   └── xrd-appstack.yaml
│   │   ├── crossplane.yaml
│   │   ├── environment
│   │   │   ├── envconfig-platform.yaml
│   │   │   └── envconfig-zone-dronegrid-io.yaml
│   │   ├── examples
│   │   │   └── appstack.yaml
│   │   ├── policy
│   │   │   ├── mrd-activation.yaml
│   │   │   ├── rbac-app-team.yaml
│   │   │   └── vap-freight.yaml
│   │   ├── runtime
│   │   │   ├── configuration.yaml
│   │   │   ├── imageconfig.yaml
│   │   │   └── provider-config.yaml
│   │   └── spec.schema.json
│   ├── security
│   │   ├── README.md
│   │   ├── openbao-k8s-policy.hcl
│   │   └── openbao-k8s-role.json.example
│   ├── ssip
│   │   ├── Cargo.toml
│   │   ├── Makefile
│   │   └── crates
│   │       ├── edgefactory-core
│   │       │   ├── Cargo.toml
│   │       │   └── src
│   │       │       ├── api.rs
│   │       │       ├── freight.rs
│   │       │       ├── lib.rs
│   │       │       ├── schema.rs
│   │       │       └── xr.rs
│   │       ├── edgefactory-ssip
│   │       │   ├── Cargo.toml
│   │       │   └── src
│   │       │       ├── auth.rs
│   │       │       ├── main.rs
│   │       │       ├── routes.rs
│   │       │       └── state.rs
│   │       └── edgefactory-tui
│   │           ├── Cargo.toml
│   │           └── src
│   │               ├── client.rs
│   │               ├── main.rs
│   │               └── ui.rs
│   └── tests
│       └── consistency.test.mjs
├── ppt-slides
│   ├── Edge-Factory-GO.pptx
│   └── svg
│       └── ef-go.svg
└── static-assets
    ├── 01-start.svg
    ├── 02-choose.svg
    ├── 03-review.svg
    ├── 04-dispatch.svg
    ├── 05-provisioning.svg
    └── 06-provisioned.svg
```

   
- GitOps Crossplane controller (no OpenTofu as infra except in its role of declaratively creating the Kubernetes IaC control plane cluster. Crossplane is itself an auto-reconciler and detects drift at Git level or in OCI registries. 
**This version** This version includes with a Rust `ratatui` TUI CLI catalog system to drive the CF infrastructure into the live canvas. This version includes Crossplane 2.0 logical mapping of self-service low-friction provider inputs that directly would fail the provider provisioning process unless these low-congitive inputs are mapped internally to actual Cloudflare provider inputs done using a middleware function intercepting process of Crossplane XRD spec (.spec part) to translate these friendly inputs to the actual Crossplane CF provider. This process is natively provided in Crossplane using `patch-and-transform` in a day 0 platform engineering step sequence of providing inventories registered to Crossplane as templated Helm charts of OpenAPI v3 spec Crossplane 2.0 XRDs (the namespaced scope OpenAPI YAML interfaces for self-served clients) and their defined controllers that actually do the tethering of Crossplane Cloudflare MR (Managed Resources) through Compositions where the `patch-and-transform` is done.


## TUI Self-Service Workflows for All Versions

The following sections cover a Rust-native terminal UI (TUI) self-service deployment workflow platform engineering architects would design and provide to organizational units that require a low-cogntive low-friction workflow to self-serve their own infrastructure in Cloudflare. This technique is logically and equally applied to GCP and AWS. The TUI is low friction however a Rust WASM native Leptos or Dixous UI with an Axum web service could equally or pair with the TUI if that was ever required.

## TUI Self-Service for OpenTofu + Packer Design for Cloudflare Workflows (Non-K8s GitOps - NGO)

**Stage 1**

![stage 1](edge-factory-ngo/static-assets/01-start.svg)

**Stage 2**

![stage 2](edge-factory-ngo/static-assets/02-choose.svg)

**Stage 3**

![stage 3](edge-factory-ngo/static-assets/03-review.svg)

**Stage 4**

![stage 4](edge-factory-ngo/static-assets/04-dispatch.svg)

**Stage 5**

![stage 5](edge-factory-ngo/static-assets/05-provisioning.svg)

**Stage 6**

![stage 6](edge-factory-ngo/static-assets/06-provisioned.svg)


## TUI Self-Service for OpenTofu + Packer Design for Cloudflare Workflows (K8s GitOps - GO)

**Stage 1**

![stage 1](edge-factory-go/static-assets/01-start.svg)

**Stage 2**

![stage 2](edge-factory-go/static-assets/02-choose.svg)

**Stage 3**

![stage 3](edge-factory-go/static-assets/03-review.svg)

**Stage 4**

![stage 4](edge-factory-go/static-assets/04-dispatch.svg)

**Stage 5**

![stage 5](edge-factory-go/static-assets/05-provisioning.svg)

**Stage 6**

![stage 6](edge-factory-go/static-assets/06-provisioned.svg)


## TUI Self-Service for Crossplane + Packer Design for Cloudflare Workflows (K8s GitOps - GLGO)

**Stage 1**

![stage 1](edge-factory-xp/static-assets/01-start.svg)

**Stage 2**

![stage 2](edge-factory-xp/static-assets/02-choose.svg)

**Stage 3**

![stage 3](edge-factory-xp/static-assets/03-review.svg)

**Stage 4**

![stage 4](edge-factory-xp/static-assets/04-dispatch.svg)

**Stage 5**

![stage 5](edge-factory-xp/static-assets/05-provisioning.svg)

**Stage 6**

![stage 6](edge-factory-xp/static-assets/06-provisioned.svg)






## Executing the Factories and the TUI

Provided here is the compilation process using the provided Makefile per factory project including the steps to compile the Rust native IaC provisioning self-service TUI.

### The Packer CI and OpenTofu - NGO Version

Prerequisites on the workstation: `tofu` (≥ 1.10), `node` (≥ 22), `tflint`, `cargo` (Rust ≥ 1.85, edition 2024), `wrangler`. On Cloudflare: one account, one zone, a dedicated R2 bucket `edge-factory-state` with its own API token, and two scoped API tokens (plan = read, apply = edit; scopes in `edge-factory-ngo/security/README.md`) stored in OpenBao under `secret/cloudflare/prod/{plan,apply}` with a 64-hex `tofu_encryption_key`. No secret is ever stored in GitHub; `ci/auth.mjs` exchanges the GitHub OIDC token for a 15-minute OpenBao token at run time.

```shell
cd edge-factory-ngo/edge-factory-ngo
# dotfiles ship renamed: restore them once
mv dot.gitignore .gitignore && mv dot.pre-commit-config.yaml .pre-commit-config.yaml && mv dot.tflint.hcl .tflint.hcl

make check        # tofu fmt + validate (offline), tflint, cargo check of the TUI workspace
make test         # 16 CI guard tests (plan provenance, request-file fingerprints, routing) + cargo test
```

Compile and run the Rust self-service TUI (`ssip/` workspace: `edgefactory-core`, `edgefactory-tui`, `edgefactory-ssip`):

```shell
make -C ssip check                         # cargo check --workspace
make -C ssip build                         # cargo build --release  ->  ssip/target/release/edgefactory
./ssip/target/release/edgefactory --demo   # walk the screens with a built-in catalog, no account needed
make request                               # the real thing: --mode tofu, validates against platform/spec.schema.json
                                           # and writes infra/live/prod/requests/<name>.json
```

Deploy through the two-stage pipeline (the request file is the only thing an app team commits):

```shell
git add infra/live/prod/requests/<name>.json && git commit -m "<name>" && gh pr create
#   plan.yml : check -> plan (OIDC -> OpenBao -> read-scoped token -> tofu plan against R2) -> gate
#   review the plan: it lists every Cloudflare object the app-stack profile layer chose for the tier/exposure
#   merge    : apply.yml verifies 13 provenance fingerprints and applies exactly that plan with the edit-scoped token
make bindings                              # after apply: artifacts/bindings/<app>.json for the app's wrangler.jsonc
```

Image build in this variant: the Rust application container is built from `containers/app.Containerfile` and shipped **by the app repository** with `wrangler deploy` into the Worker shell the pipeline created (Cloudflare Containers are deployed through wrangler, not the Terraform provider). Local plan/apply without GitHub (break-glass only): `make plan` and `make apply PLAN=artifacts/approved.tfplan` with `CLOUDFLARE_API_TOKEN`, `AWS_ACCESS_KEY_ID/SECRET` (R2) and `TOFU_ENCRYPTION_KEY` in the environment.

### The Packer CI and OpenTofu - GO Version

Prerequisites: everything from NGO plus a Kubernetes cluster you already run (a small k3s is enough; kubeconfig context `k3s-edge`), `flux`, `kubectl`, and an OpenBao Kubernetes-auth mount for that cluster (`edge-factory-go/security/README.md`). The control loop is the Flux tofu-controller; there is no GitHub Actions workflow in this variant.

```shell
cd edge-factory-go/edge-factory-go
mv dot.gitignore .gitignore && mv dot.pre-commit-config.yaml .pre-commit-config.yaml

make check        # tofu fmt + validate for prod and the control-plane root, kustomize build of gitops/, cargo check
make test         # 5 consistency tests (CR paths, ExternalSecrets, backend override == backend.tfbackend, versions) + cargo test
```

Day 0, once — Flux and the bootstrap source on the cluster (OpenTofu runs here and then goes dormant):

```shell
export TF_VAR_registry='{host="ghcr.io",username="<user>",password="<token>"}'   # from OpenBao
export TOFU_ENCRYPTION_KEY=<64 hex from OpenBao>
make bootstrap    # infra/live/control-plane/flux: Flux (Helm) + OCIRepository(tree:main) + Kustomization(gitops/flux-system)
make status       # Flux then installs tofu-controller + External Secrets, syncs the Cloudflare token / R2 keys / encryption
                  # block from OpenBao, and creates the Terraform CR prod-edge (interval 5m, approvePlan auto)
```

Compile and run the TUI exactly as in NGO (`make -C ssip build`, `--demo`, `make request` writes `infra/live/prod/requests/<name>.json`). The release is a push of the committed tree as an OCI artifact:

```shell
git commit -am "<name>"
make push         # flux push artifact oci://ghcr.io/<you>/edge-factory-go/tree:<sha>, tagged :main
make status       # within ~6 minutes the controller has planned and applied; drift is corrected on the next loop
kubectl -n flux-system get secret prod-edge-outputs -o jsonpath='{.data.wrangler_bindings}' | base64 -d   # bindings for wrangler.jsonc
```

Image build: as in NGO, the application container is built from `containers/app.Containerfile` and deployed with `wrangler deploy` from the app repository into the shell the controller created.

### The Packer CI and Crossplane - GLGO Version

Prerequisites: a Kubernetes cluster (k3s, context `k3s-edge`), `crossplane` CLI, `cosign`, `podman`, `kubectl`, a GHCR credential, a cosign key in OpenBao Transit with its public half exported, and an OpenBao Kubernetes-auth mount (`edge-factory-xp/security/README.md`). The provider is the official `crossplane-contrib/provider-upjet-cloudflare`; nothing is generated locally.

```shell
cd edge-factory-xp/edge-factory-xp
mv dot.gitignore .gitignore && mv dot.pre-commit-config.yaml .pre-commit-config.yaml

make check        # tofu fmt + validate (control-plane root), crossplane beta validate platform/, cargo check
make test         # 7 consistency tests: XRD == spec.schema.json == app-stack inputs, MR kinds activated, SSIP constants == admission policy
```

Day 0 — platform engineering stocks the inventory (run in this order, once):

```shell
export TF_VAR_registry='{host="ghcr.io",username="<user>",password="<token>"}'
export TF_VAR_cosign_public_key="$(cat cosign.pub)"
export TOFU_ENCRYPTION_KEY=<64 hex>
make bootstrap    # infra/live/control-plane/crossplane: Crossplane 2.4 on the cluster, signature verification on
make build        # crossplane xpkg build of platform/ (XRD + Composition + deps)  +  the SSIP service image (containers/ssip.Containerfile)
make publish      # push + cosign-sign the Configuration package and the SSIP image to GHCR
make register     # ImageConfig, Configuration (pulls the provider + 3 functions), policy (RBAC, admission, MR activation),
                  # EnvironmentConfigs (account, zones, tier opinions), ProviderConfig (token via External Secrets), SSIP Deployment
make status
```

Day 1 — an app team compiles and runs the TUI in `xp` mode (same binary as NGO/GO, different mode):

```shell
make -C ssip build
./ssip/target/release/edgefactory --demo                      # offline walkthrough of all screens
export EDGEFACTORY_TOKEN=$(<your SSO login>)                   # OIDC bearer for the SSIP service
make request                                                  # --mode xp: choose -> review -> Enter
#   the SSIP service verifies the token, validates against the XRD, pushes a cosign-signed freight artifact to GHCR,
#   applies the AppStack; the admission policy admits it only from the SSIP identity with a digest;
#   Crossplane matches the registered XRD + Composition and composes the managed resources
#   the watch screen shows hostname, Worker, D1/R2/Hyperdrive, Access, LB and, when Ready, the wrangler bindings
```

Image build: the application container is deployed with `wrangler deploy` into the `WorkersScript` shell Crossplane created; the shell is composed with `managementPolicies` without `Update`, so a wrangler deploy is never reverted by the controller.






## References 

#### OpenTofu 
- https://opentofu.org/
- https://opentofu.org/docs/cli/code/
- https://search.opentofu.org/provider/opentofu/cloudflare/latest

#### OpenTofu Flux Controller
- https://github.com/flux-iac/tofu-controller
- https://flux-iac.github.io/tofu-controller/getting_started/
- https://flux-iac.github.io/tofu-controller/

#### Packer
- https://developer.hashicorp.com/packer
- https://developer.hashicorp.com/hcp/docs/packer

#### Crossplane
- https://www.crossplane.io/
- https://www.upbound.io/
- https://github.com/cdloh/provider-cloudflare

#### OpenTelemetry
- https://opentelemetry.io/

#### OpenBao (Vault)
- https://openbao.org/

#### OpenChoreo
- https://openchoreo.dev/

#### FluxCD
- https://fluxcd.io/

#### Rust
- https://rust-lang.org/
- https://ratatui.rs/
- https://leptos.dev/
- https://dioxuslabs.com/

#### Cloudflare
- https://www.cloudflare.com/
- https://www.cloudflare.com/products/d1/
- https://www.cloudflare.com/products/r2/
- https://www.cloudflare.com/products/containers/
- https://www.cloudflare.com/products/dns/
- https://www.cloudflare.com/products/cdn/
- https://www.cloudflare.com/products/workers/





