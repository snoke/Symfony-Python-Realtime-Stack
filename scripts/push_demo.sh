#!/usr/bin/env bash
set -euo pipefail

HOST="${HOST:-http://localhost:8180}"
USER_ID="${USER_ID:-42}"

curl -sS -X POST "$HOST/api/push-demo/$USER_ID" | cat
