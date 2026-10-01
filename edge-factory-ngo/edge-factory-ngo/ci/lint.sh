#!/usr/bin/env bash
set -euo pipefail
repo_root=$(pwd)
tflint --chdir=infra/modules/app-stack --config="${repo_root}/.tflint.hcl"
tflint --chdir=infra/modules/cf-worker --config="${repo_root}/.tflint.hcl"
tflint --chdir=infra/live/prod --config="${repo_root}/.tflint.hcl"
