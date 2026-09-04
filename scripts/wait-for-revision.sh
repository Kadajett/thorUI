#!/usr/bin/env bash
set -euo pipefail

base_url=${1:?base URL is required}
expected=${2:?expected revision is required}
attempts=${3:-20}
delay=${4:-2}
revision=""

for ((attempt = 1; attempt <= attempts; attempt++)); do
  revision=$(curl --fail --silent --show-error "${base_url%/}/version.json" | jq -r .revision || true)
  if [[ "$revision" == "$expected" ]]; then
    exit 0
  fi
  echo "Waiting for $expected; received $revision (attempt $attempt/$attempts)"
  sleep "$delay"
done

echo "Expected revision $expected but received $revision" >&2
exit 1
