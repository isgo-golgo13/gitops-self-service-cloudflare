# edge-factory-xp (Crossplane 2 self-service on your k3s) - the front door. Bare verbs only.
#   make check      tofu fmt + validate, crossplane validate, cargo check
#   make test       consistency tests + cargo test
#   make build      Configuration package (xpkg) + SSIP image
#   make publish    push + cosign-sign to GHCR
#   make bootstrap  day 0, once: Crossplane on the k3s (needs TOFU_ENCRYPTION_KEY, TF_VAR_registry, TF_VAR_cosign_public_key)
#   make register   day 0: ImageConfig, Configuration, policy, environment, ProviderConfig, SSIP
#   make request    the SSIP terminal in xp mode (talks to the service); --demo works offline
#   make status
#   make clean
.DEFAULT_GOAL := check
SHELL := /usr/bin/env bash
.SHELLFLAGS := -euo pipefail -c
REGISTRY ?= ghcr.io/isgo-golgo13/edge-factory-xp
VERSION  ?= v1.0.0
CONFIG_IMAGE := $(REGISTRY)/configuration-edgefactory:$(VERSION)

.PHONY: check test build publish bootstrap register request status clean

check:
	tofu fmt -check -recursive infra
	tofu -chdir=infra/live/control-plane/crossplane init -backend=false -input=false >/dev/null && tofu -chdir=infra/live/control-plane/crossplane validate
	crossplane beta validate platform/apis platform/examples
	$(MAKE) -C ssip check

test:
	node --test tests/consistency.test.mjs
	$(MAKE) -C ssip test

build:
	mkdir -p target
	crossplane xpkg build --package-root=platform --examples-root=platform/examples --package-file=target/configuration-edgefactory.xpkg
	$(MAKE) -C ssip image

publish: build
	crossplane xpkg push --package-files=target/configuration-edgefactory.xpkg $(CONFIG_IMAGE)
	cosign sign --yes --tlog-upload=false --key "$$COSIGN_KEY" $(CONFIG_IMAGE)
	podman push $(REGISTRY)/edgefactory-ssip:1.0.0
	cosign sign --yes --tlog-upload=false --key "$$COSIGN_KEY" $(REGISTRY)/edgefactory-ssip:1.0.0

bootstrap:
	test -n "$$TOFU_ENCRYPTION_KEY" || { echo "set TOFU_ENCRYPTION_KEY (64 hex chars from OpenBao) before make bootstrap"; exit 1; }
	export TF_ENCRYPTION="$$(sed "s/__ENCRYPTION_KEY__/$$TOFU_ENCRYPTION_KEY/" ci/encryption.hcl)"; \
	tofu -chdir=infra/live/control-plane/crossplane init -backend-config=backend.tfbackend -input=false && \
	tofu -chdir=infra/live/control-plane/crossplane apply -input=false

register:
	kubectl apply -f platform/runtime/imageconfig.yaml
	kubectl apply -f platform/runtime/configuration.yaml
	kubectl wait --for=condition=Healthy configuration/edgefactory --timeout=10m
	kubectl apply -f platform/policy/
	kubectl apply -f platform/environment/
	kubectl apply -f platform/runtime/provider-config.yaml -f platform/runtime/ssip.yaml

request:
	$(MAKE) -C ssip build
	./ssip/target/release/edgefactory --mode xp

status:
	kubectl get configuration,provider,function
	kubectl get environmentconfigs
	kubectl get appstacks -A
	kubectl get managed -A 2>/dev/null || true
	kubectl -n edgefactory-system get deploy,pods

clean:
	rm -rf target .test infra/live/control-plane/*/.terraform
