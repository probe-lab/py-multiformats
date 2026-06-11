#!/usr/bin/env bash
# Refresh the vendored multiformats registries from their canonical repositories.
set -euo pipefail

cd "$(dirname "$0")/.."
curl -fsSL https://raw.githubusercontent.com/multiformats/multicodec/master/table.csv \
    -o data/multicodec-table.csv
curl -fsSL https://raw.githubusercontent.com/multiformats/multibase/master/multibase.csv \
    -o data/multibase-table.csv
git diff --stat data/
