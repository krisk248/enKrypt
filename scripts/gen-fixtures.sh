#!/usr/bin/env bash
# Regenerate the GnuPG interoperability test fixtures used by the Rust suite.
# These contain a THROWAWAY test key (passphrase "test1234") and are not
# committed to the repo. Requires GnuPG 2.x.
#
#   ./scripts/gen-fixtures.sh
#
# Produces crypto-core/tests/fixtures/{seckey,pubkey,msg}.asc

set -euo pipefail

DEST="$(cd "$(dirname "$0")/.." && pwd)/crypto-core/tests/fixtures"
mkdir -p "$DEST"

GNUPGHOME="$(mktemp -d)"
export GNUPGHOME
trap 'rm -rf "$GNUPGHOME"' EXIT

cat > "$GNUPGHOME/params" <<'EOF'
%echo generating enKrypt interop fixture key
Key-Type: eddsa
Key-Curve: ed25519
Key-Usage: sign,cert
Subkey-Type: ecdh
Subkey-Curve: cv25519
Subkey-Usage: encrypt
Name-Real: GnuPG Fixture
Name-Email: fixture@example.com
Expire-Date: 0
Passphrase: test1234
%commit
%echo done
EOF

gpg --batch --pinentry-mode loopback --gen-key "$GNUPGHOME/params"

printf 'Hello from GnuPG interop!\n' \
  | gpg --batch --yes --armor --trust-model always \
        --encrypt -r fixture@example.com > "$DEST/msg.asc"

gpg --batch --pinentry-mode loopback --passphrase test1234 \
    --armor --export-secret-keys fixture@example.com > "$DEST/seckey.asc"

gpg --armor --export fixture@example.com > "$DEST/pubkey.asc"

echo "Wrote fixtures to $DEST"
