# edge-factory-ngo - the front door. Bare verbs only.
#   make check     fmt + validate + lint (offline, no credentials) + cargo check
#   make test      CI guard tests + cargo test
#   make request   open the SSIP terminal; writes infra/live/prod/requests/<name>.json
#   make plan      plan against the R2 state bucket (needs credentials in env)
#   make apply     apply a saved, reviewed plan: make apply PLAN=artifacts/approved.tfplan
#   make bindings  write per-app wrangler binding manifests to artifacts/bindings/<app>.json
#   make clean
# Nothing is ever written to /tmp; transient output goes to artifacts/ and .test/.
.DEFAULT_GOAL := check
SHELL := /usr/bin/env bash
.SHELLFLAGS := -euo pipefail -c
PLAN ?= artifacts/approved.tfplan

.PHONY: check test request plan apply bindings clean

check:
	tofu fmt -check -recursive infra
	bash ci/validate-local.sh
	bash ci/lint.sh
	$(MAKE) -C ssip check

test:
	node --test tests/ci.test.mjs
	$(MAKE) -C ssip test

request:
	$(MAKE) -C ssip build
	./ssip/target/release/edgefactory --mode tofu --repo .

plan:
	mkdir -p artifacts
	node ci/tofu.mjs init -lockfile=readonly
	node ci/tofu.mjs plan -lock-timeout=5m -out=../../../$(PLAN)

apply:
	test -f "$(PLAN)"
	node ci/tofu.mjs init -lockfile=readonly
	node ci/tofu.mjs apply -lock-timeout=5m ../../../$(PLAN)

bindings:
	mkdir -p artifacts/bindings
	node ci/tofu.mjs output -json wrangler_bindings | node -e 'const b=JSON.parse(require("fs").readFileSync(0,"utf8"));for(const [n,v] of Object.entries(b)) require("fs").writeFileSync(`artifacts/bindings/$${n}.json`, JSON.stringify(v,null,2))'
	ls artifacts/bindings

clean:
	rm -rf artifacts .test infra/live/prod/.terraform
