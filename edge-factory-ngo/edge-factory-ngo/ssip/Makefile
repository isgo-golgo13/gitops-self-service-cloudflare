# ssip workspace - bare verbs, house standard.
SHELL := /usr/bin/env bash
.SHELLFLAGS := -euo pipefail -c
IMAGE ?= ghcr.io/isgo-golgo13/edge-factory/edgefactory-ssip:1.0.0

.PHONY: touch build check test fmt clippy clean image

touch:
	find crates -name '*.rs' -exec touch {} +

build: touch
	cargo build --release

check:
	cargo check --workspace --all-targets

test:
	cargo test --workspace

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

image:
	podman build -f ../containers/ssip.Containerfile -t $(IMAGE) ..

clean:
	cargo clean
