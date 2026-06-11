#!/usr/bin/env bash
# Refresh the vendored multicodec registry from the canonical repository.
set -euo pipefail

cd "$(dirname "$0")/.."
curl -fsSL https://raw.githubusercontent.com/multiformats/multicodec/master/table.csv \
    -o data/multicodec-table.csv
git diff --stat data/multicodec-table.csv
