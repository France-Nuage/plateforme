#!/bin/sh
# Regenerate /config.js from environment variables before nginx starts.
set -eu

CONFIG_FILE=/usr/share/nginx/html/config.js

CONTROLPLANE_URL="${CONTROLPLANE_URL:-https://controlplane.test}"
APPLICATION_MODE="${APPLICATION_MODE:-rpc}"

cat > "$CONFIG_FILE" <<EOF
window.__RUNTIME_CONFIG__ = {
  controlplaneUrl: '${CONTROLPLANE_URL}',
  applicationMode: '${APPLICATION_MODE}',
};
EOF
