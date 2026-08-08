#!/bin/sh
# Seed dev/CI d'un FerrisKey local pour le flow BFF de la console.
#
# Recette validée en live contre ghcr.io/ferriskey/ferriskey-api (2026-08) — chaque
# appel a été vérifié, y compris les pièges qui cassent un seed écrit à l'aveugle :
#  - le client se crée via POST /realms/{realm}/clients (client_type=confidential),
#    le `secret` est TOUJOURS généré par le serveur (un `secret` fourni est ignoré,
#    aucun endpoint set/regenerate) → on l'exporte dans OIDC_CLIENT_SECRET_FILE.
#  - `redirect_uris` dans le corps du create est IGNORÉ : il faut un appel dédié
#    POST /realms/{realm}/clients/{id}/redirects {value,enabled}.
#  - la réponse create-user renvoie l'id sous `.data.id`.
#  - reset-password exige un mot de passe conforme à la policy (≥ 8 + complexité) ;
#    un mot de passe faible (ex. "anvil") renvoie 422.
#
# Idempotent : ré-exécutable (le realm/client/user existants sont réutilisés).
# Requiert : curl, jq. Fail-fast (set -e), aucune sortie silencieuse.
set -eu

FERRISKEY_URL="${FERRISKEY_URL:-http://ferriskey-api:3333}"
ADMIN_REALM="${ADMIN_REALM:-master}"
ADMIN_USERNAME="${ADMIN_USERNAME:-admin}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-admin}"
REALM="${REALM:-france-nuage}"
CLIENT_ID="${CLIENT_ID:-console}"
REDIRECT_URI="${REDIRECT_URI:-https://controlplane.test/auth/callback}"
POST_LOGOUT_URI="${POST_LOGOUT_URI:-https://console.test}"
SEED_USERNAME="${SEED_USERNAME:-wile.coyote}"
SEED_EMAIL="${SEED_EMAIL:-wile.coyote@acme.test}"
SEED_PASSWORD="${SEED_PASSWORD:-Anvil-Coyote-2026!}"
OIDC_CLIENT_SECRET_FILE="${OIDC_CLIENT_SECRET_FILE:-/shared/oidc-client-secret}"

log() { echo "[ferriskey-seed] $*"; }

# 1. Attendre que l'API réponde (borné).
i=0
until curl -sf -o /dev/null "$FERRISKEY_URL/health/ready"; do
  i=$((i + 1))
  [ "$i" -gt 60 ] && { log "FerrisKey API pas prête après 120s"; exit 1; }
  sleep 2
done
log "FerrisKey API prête ($FERRISKEY_URL)"

# 2. Token admin (password grant, realm master, client admin-cli auto-provisionné).
TOKEN=$(curl -sf -X POST \
  "$FERRISKEY_URL/realms/$ADMIN_REALM/protocol/openid-connect/token" \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  -d grant_type=password -d client_id=admin-cli \
  --data-urlencode "username=$ADMIN_USERNAME" \
  --data-urlencode "password=$ADMIN_PASSWORD" | jq -r '.access_token')
[ -n "$TOKEN" ] && [ "$TOKEN" != null ] || { log "login admin échoué"; exit 1; }
log "admin authentifié"

auth() { curl -sf -H "Authorization: Bearer $TOKEN" "$@"; }

# 3. Realm (idempotent).
if auth -o /dev/null "$FERRISKEY_URL/realms/$REALM"; then
  log "realm $REALM déjà présent"
else
  auth -X POST "$FERRISKEY_URL/realms" -H 'Content-Type: application/json' \
    -d "{\"name\":\"$REALM\",\"display_name\":\"France Nuage\"}" >/dev/null
  log "realm $REALM créé"
fi

# 4. Client confidentiel `console` (idempotent). Le secret est généré serveur.
EXISTING=$(auth "$FERRISKEY_URL/realms/$REALM/clients" \
  | jq -r --arg c "$CLIENT_ID" '(.data // .)[] | select(.client_id==$c) | .id' | head -n1)
if [ -n "$EXISTING" ]; then
  CID="$EXISTING"
  log "client $CLIENT_ID déjà présent ($CID) — secret serveur non ré-exposable, recréer le realm pour repartir propre"
else
  RESP=$(auth -X POST "$FERRISKEY_URL/realms/$REALM/clients" \
    -H 'Content-Type: application/json' -d "{
      \"client_id\":\"$CLIENT_ID\",
      \"name\":\"France Nuage Console (BFF)\",
      \"client_type\":\"confidential\",
      \"protocol\":\"openid-connect\",
      \"public_client\":false,
      \"service_account_enabled\":false,
      \"direct_access_grants_enabled\":false,
      \"enabled\":true
    }")
  CID=$(echo "$RESP" | jq -r '.id')
  SECRET=$(echo "$RESP" | jq -r '.secret')
  [ -n "$SECRET" ] && [ "$SECRET" != null ] || { log "pas de secret retourné"; exit 1; }
  mkdir -p "$(dirname "$OIDC_CLIENT_SECRET_FILE")"
  printf '%s' "$SECRET" > "$OIDC_CLIENT_SECRET_FILE"
  log "client $CLIENT_ID créé ($CID), secret écrit dans $OIDC_CLIENT_SECRET_FILE"
fi

# 5. Redirect URI (appel dédié — ignoré dans le corps du create).
auth -X POST "$FERRISKEY_URL/realms/$REALM/clients/$CID/redirects" \
  -H 'Content-Type: application/json' \
  -d "{\"value\":\"$REDIRECT_URI\",\"enabled\":true}" >/dev/null 2>&1 || true
log "redirect_uri $REDIRECT_URI enregistré"

# 6. Utilisateur de dev + mot de passe (policy-conforme).
UID_JSON=$(auth "$FERRISKEY_URL/realms/$REALM/users" \
  | jq -r --arg u "$SEED_USERNAME" '(.data // .)[] | select(.username==$u) | .id' | head -n1)
if [ -z "$UID_JSON" ]; then
  UID_JSON=$(auth -X POST "$FERRISKEY_URL/realms/$REALM/users" \
    -H 'Content-Type: application/json' -d "{
      \"username\":\"$SEED_USERNAME\",\"firstname\":\"Wile\",\"lastname\":\"Coyote\",
      \"email\":\"$SEED_EMAIL\",\"email_verified\":true
    }" | jq -r '.data.id')
  log "user $SEED_USERNAME créé ($UID_JSON)"
fi
auth -X PUT "$FERRISKEY_URL/realms/$REALM/users/$UID_JSON/reset-password" \
  -H 'Content-Type: application/json' \
  -d "{\"credential_type\":\"password\",\"temporary\":false,\"value\":\"$SEED_PASSWORD\"}" >/dev/null
log "mot de passe défini pour $SEED_USERNAME"

log "seed terminé : realm=$REALM client=$CLIENT_ID user=$SEED_USERNAME"
