# edge-factory-go - the front door. Bare verbs only.
#   make check      tofu fmt + validate (offline), kustomize build of gitops/, cargo check
#   make test       consistency tests + cargo test
#   make request    the SSIP terminal; writes infra/live/prod/requests/<name>.json
#   make push       publish this repository as the OCI artifact Flux reconciles from (the release)
#   make bootstrap  day 0, once: Flux + bootstrap source on your k3s (needs TOFU_ENCRYPTION_KEY, TF_VAR_registry)
#   make status     what Flux and the tofu-controller are doing
#   make clean
# Nothing is ever written to /tmp; transient output goes to artifacts/ and .test/.
.DEFAULT_GOAL := check
SHELL := /usr/bin/env bash
.SHELLFLAGS := -euo pipefail -c
ARTIFACT ?= oci://ghcr.io/isgo-golgo13/edge-factory-go/tree
REV      := $(shell git rev-parse --short HEAD 2>/dev/null || echo dev)
BRANCH   := $(shell git branch --show-current 2>/dev/null || echo main)

.PHONY: check test request push bootstrap status clean

check:
	tofu fmt -check -recursive infra
	for root in infra/live/prod infra/live/control-plane/flux; do tofu -chdir=$$root init -backend=false -input=false >/dev/null && tofu -chdir=$$root validate; done
	for dir in flux-system platform/controllers platform/secrets workloads/edge-factory; do kubectl kustomize gitops/$$dir >/dev/null; done
	$(MAKE) -C ssip check

test:
	node --test tests/consistency.test.mjs
	$(MAKE) -C ssip test

request:
	$(MAKE) -C ssip build
	./ssip/target/release/edgefactory --mode tofu --repo .

push:
	test -z "$$(git status --porcelain)" || { echo "commit first: the artifact must equal a commit"; exit 1; }
	flux push artifact $(ARTIFACT):$(REV) --path=. --source="$$(git config --get remote.origin.url)" --revision="$(BRANCH)@sha1:$$(git rev-parse HEAD)"
	flux tag artifact $(ARTIFACT):$(REV) --tag $(BRANCH)

# Day 0. $(shell) would flatten newlines, so TF_ENCRYPTION is built inside the recipe.
bootstrap:
	test -n "$$TOFU_ENCRYPTION_KEY" || { echo "set TOFU_ENCRYPTION_KEY (64 hex chars from OpenBao) before make bootstrap"; exit 1; }
	export TF_ENCRYPTION="$$(sed "s/__ENCRYPTION_KEY__/$$TOFU_ENCRYPTION_KEY/" ci/encryption.hcl)"; \
	tofu -chdir=infra/live/control-plane/flux init -backend-config=backend.tfbackend -input=false && \
	tofu -chdir=infra/live/control-plane/flux apply -input=false

status:
	flux get sources oci -n flux-system
	flux get kustomizations -n flux-system
	flux get helmreleases -n flux-system
	kubectl get terraforms.infra.contrib.fluxcd.io -n flux-system

clean:
	rm -rf artifacts .test infra/live/*/.terraform infra/live/control-plane/*/.terraform
