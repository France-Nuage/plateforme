#!/bin/sh
# Regenerate /config.js from environment variables before nginx starts.
set -eu

CONFIG_FILE=/usr/share/nginx/html/config.js

CONTROLPLANE_URL="${CONTROLPLANE_URL:-https://controlplane.test}"
OIDC_CLIENT_ID="${OIDC_CLIENT_ID:-francenuage}"
OIDC_PROVIDER_NAME="${OIDC_PROVIDER_NAME:-keycloak}"
OIDC_PROVIDER_URL="${OIDC_PROVIDER_URL:-https://keycloak.test/realms/francenuage}"
APPLICATION_MODE="${APPLICATION_MODE:-rpc}"

cat > "$CONFIG_FILE" <<EOF
window.__RUNTIME_CONFIG__ = {
  controlplaneUrl: '${CONTROLPLANE_URL}',
  oidcClientId: '${OIDC_CLIENT_ID}',
  oidcProviderName: '${OIDC_PROVIDER_NAME}',
  oidcProviderUrl: '${OIDC_PROVIDER_URL}',
  applicationMode: '${APPLICATION_MODE}',
};
EOF
