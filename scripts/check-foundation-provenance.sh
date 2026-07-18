#!/usr/bin/env bash
set -euo pipefail

matches="$({
  git grep -n -i -E 'blob\.handy\.computer|handy\.computer|github\.com/cjpais/Handy|com\.pais\.handy|CJ-Signing|cjpais-dev' -- \
    ':!LICENSE' ':!NOTICE.md' ':!docs/**' ':!PLANNING/**' \
    ':!scripts/check-foundation-provenance.sh' || true
})"

if [[ -n "$matches" ]]; then
  printf '%s\n' 'Forbidden upstream runtime identity found:' "$matches" >&2
  exit 1
fi

printf '%s\n' 'Foundation provenance gate passed.'
