#!/usr/bin/env bash
set -euo pipefail
TOFU_LOCAL_VALIDATION=1 node ci/tofu.mjs init -backend=false -lockfile=readonly
TOFU_LOCAL_VALIDATION=1 node ci/tofu.mjs validate
