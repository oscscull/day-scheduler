#!/usr/bin/env bash
# Creates the updater signing key and the two CI secrets releases are signed with. Run once by the
# repo owner, from anywhere. The key's password is kept only encrypted, in `.env.release` (dotenvx,
# committed); what decrypts it stays outside the repo, next to the signing key in ~/.tauri.
# Read it back with: dotenvx get TAURI_SIGNING_PRIVATE_KEY_PASSWORD -f .env.release -fk ~/.tauri/quarterdeck-release.env.keys
set -euo pipefail

REPO=oscscull/quarterdeck
DIR=$HOME/.tauri
KEY=$DIR/quarterdeck-updater.key
ENV_KEYS=$DIR/quarterdeck-release.env.keys
ROOT=$(cd "$(dirname "$0")/.." && pwd)
CONF=$ROOT/src-tauri/tauri.conf.json

if [ -e "$KEY" ]; then
  echo "$KEY already exists. Move it away first if you mean to replace it." >&2
  exit 1
fi
mkdir -p "$DIR" && chmod 700 "$DIR"

read -rsp "Password for the updater key: " PW && echo
read -rsp "Again: " AGAIN && echo
if [ "$PW" != "$AGAIN" ]; then
  echo "The passwords differ." >&2
  exit 1
fi

dotenvx set TAURI_SIGNING_PRIVATE_KEY_PASSWORD "$PW" \
  --no-native --no-armor -f "$ROOT/.env.release" -fk "$ENV_KEYS" >/dev/null
chmod 600 "$ENV_KEYS"

(cd "$ROOT" && npx --yes @tauri-apps/cli signer generate --ci -w "$KEY" -p "$PW") >/dev/null
chmod 600 "$KEY"

gh secret set TAURI_SIGNING_PRIVATE_KEY --repo "$REPO" < "$KEY"
printf '%s' "$PW" | gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo "$REPO"
unset PW AGAIN

KEY_PUB=$KEY.pub CONF=$CONF node -e '
  const fs = require("fs");
  const conf = JSON.parse(fs.readFileSync(process.env.CONF, "utf8"));
  conf.plugins.updater.pubkey = fs.readFileSync(process.env.KEY_PUB, "utf8").trim();
  fs.writeFileSync(process.env.CONF, JSON.stringify(conf, null, 2) + "\n");
'
echo "Done: secrets set, public key written to src-tauri/tauri.conf.json, password saved encrypted in .env.release."
