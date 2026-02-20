#!/usr/bin/env bash
set -euo pipefail

KEY_DIR="$(dirname "$0")/keys"
mkdir -p "$KEY_DIR"

openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out "$KEY_DIR/dev_private.pem"
openssl rsa -pubout -in "$KEY_DIR/dev_private.pem" -out "$KEY_DIR/dev_public.pem"

echo "Generated: $KEY_DIR/dev_private.pem and $KEY_DIR/dev_public.pem"
