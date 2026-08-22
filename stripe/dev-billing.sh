#!/usr/bin/env bash
#
# dev-billing.sh — Relaie les webhooks Stripe vers le controlplane en dev local.
#
# En local, le controlplane n'a pas d'URL publique que Stripe puisse appeler :
# ce script est l'équivalent, côté poste de dev, du sidecar `stripe listen` que
# le chart Helm déploie dans les environnements éphémères. Il :
#
#   1. Charge les variables Stripe depuis le `.env` racine.
#   2. Vérifie les prérequis (Stripe CLI, clé sandbox).
#   3. Récupère le secret de signature webhook (`whsec_...`) de `stripe listen`
#      et le persiste dans `.env` (STRIPE_WEBHOOK_SECRET).
#   4. Démarre `stripe listen`, qui relaie les événements Stripe vers l'endpoint
#      webhook local du controlplane.
#
# La réconciliation du catalogue n'est PAS faite ici : le controlplane la fait
# à son démarrage (catalog::sync_at_boot) dès que STRIPE_SECRET_KEY est présent.
# Pour re-synchroniser à la main après avoir édité le catalogue, utilise la
# sous-commande dédiée : `docker compose exec controlplane server catalog sync`.
#
# Le controlplane VALIDE la signature des webhooks : le secret fourni par
# `stripe listen` change à chaque compte/session, il doit donc être connu du
# controlplane. Si ce script met à jour STRIPE_WEBHOOK_SECRET dans `.env`, il
# faut relancer le controlplane pour qu'il prenne la nouvelle valeur — le script
# le signale explicitement plutôt que de le faire dans ton dos.
#
# Usage :
#   ./stripe/dev-billing.sh
#
# Prérequis :
#   - Stripe CLI installée et authentifiée sur la sandbox "test"
#     (https://stripe.com/docs/stripe-cli ; `stripe login`).
#   - Stack de dev démarrée : `docker compose up -d`.
#   - `.env` racine renseigné (voir `.env.example`), avec au minimum
#     STRIPE_SECRET_KEY pointant sur la sandbox (sk_test_...).

set -euo pipefail

# Racine du repo = dossier parent de ce script.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ENV_FILE="${REPO_ROOT}/.env"

# Endpoint webhook local du controlplane. Le handler HTTP écoute sur le port
# conteneur 8081 (voir server/src/application.rs), exposé sur l'hôte en 50053
# (voir docker-compose.yml). Il n'est PAS derrière Traefik/controlplane.test,
# qui ne route que le port gRPC 80 — on cible donc directement le port publié.
WEBHOOK_URL="${STRIPE_DEV_WEBHOOK_URL:-http://localhost:50053/webhooks/stripe}"

log()  { printf '\033[1;34m[dev-billing]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[dev-billing]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[dev-billing]\033[0m %s\n' "$*" >&2; exit 1; }

# ─────────────────────────────────────────────────────────────────────────────
# 1. Prérequis
# ─────────────────────────────────────────────────────────────────────────────
# La CLI Stripe est requise pour relayer les webhooks. Pas besoin de `stripe
# login` : on lui passe STRIPE_SECRET_KEY (sk_test_) via --api-key plus bas.
command -v stripe >/dev/null 2>&1 \
  || die "Stripe CLI introuvable. Installe-la : https://stripe.com/docs/stripe-cli (aucun 'stripe login' requis, on utilise STRIPE_SECRET_KEY)."

command -v docker >/dev/null 2>&1 \
  || die "docker introuvable."

[ -f "${ENV_FILE}" ] \
  || die "Fichier .env absent à la racine. Copie .env.example en .env et renseigne STRIPE_SECRET_KEY."

# ─────────────────────────────────────────────────────────────────────────────
# 2. Chargement du .env
# ─────────────────────────────────────────────────────────────────────────────
# On exporte les variables du .env dans l'environnement du script.
log "Chargement de ${ENV_FILE}"
set -a
# shellcheck disable=SC1090
source "${ENV_FILE}"
set +a

: "${STRIPE_SECRET_KEY:?STRIPE_SECRET_KEY manquant dans .env (attendu: sk_test_... de la sandbox)}"

case "${STRIPE_SECRET_KEY}" in
  sk_test_*) : ;;
  sk_live_*) die "STRIPE_SECRET_KEY est une clé LIVE. En dev, utilise impérativement la sandbox de test (sk_test_...)." ;;
  *)         warn "STRIPE_SECRET_KEY n'a pas le préfixe sk_test_ attendu — vérifie que tu pointes bien la sandbox." ;;
esac

# On passe la clé sandbox explicitement à `stripe listen` (--api-key). La CLI
# Stripe préfère la variable d'environnement STRIPE_API_KEY à toute autre source
# (y compris `stripe login`) : sans ça, un dev qui a une STRIPE_API_KEY dans son
# shell (p. ex. la clé restreinte du serveur MCP opencode) verrait `stripe
# listen` cibler le mauvais compte, silencieusement. --api-key ne suffit pas si
# STRIPE_API_KEY est définie, donc on la neutralise pour ces appels.
STRIPE_CLI=(env -u STRIPE_API_KEY stripe --api-key "${STRIPE_SECRET_KEY}")

# ─────────────────────────────────────────────────────────────────────────────
# 3. Récupère le secret webhook et le persiste dans .env
# ─────────────────────────────────────────────────────────────────────────────
log "Récupération du secret de signature webhook (stripe listen --print-secret)…"
WEBHOOK_SECRET="$("${STRIPE_CLI[@]}" listen --print-secret 2>/dev/null || true)"

case "${WEBHOOK_SECRET}" in
  whsec_*) : ;;
  *)       die "Impossible de récupérer un secret whsec_ via 'stripe listen --print-secret'. Vérifie STRIPE_SECRET_KEY (sk_test_ de la sandbox)." ;;
esac

CURRENT_SECRET="${STRIPE_WEBHOOK_SECRET:-}"
if [ "${CURRENT_SECRET}" = "${WEBHOOK_SECRET}" ]; then
  log "STRIPE_WEBHOOK_SECRET déjà à jour dans .env."
else
  log "Mise à jour de STRIPE_WEBHOOK_SECRET dans ${ENV_FILE}."
  if grep -q '^STRIPE_WEBHOOK_SECRET=' "${ENV_FILE}"; then
    # Remplace la ligne existante (in-place, compatible macOS/BSD et GNU sed).
    sed -i.bak "s|^STRIPE_WEBHOOK_SECRET=.*|STRIPE_WEBHOOK_SECRET=${WEBHOOK_SECRET}|" "${ENV_FILE}"
    rm -f "${ENV_FILE}.bak"
  else
    printf '\nSTRIPE_WEBHOOK_SECRET=%s\n' "${WEBHOOK_SECRET}" >> "${ENV_FILE}"
  fi

  warn "Le controlplane doit être relancé pour charger le nouveau STRIPE_WEBHOOK_SECRET :"
  warn "    docker compose up -d controlplane"
  warn "Relance ce script ensuite, ou redémarre le controlplane dans un autre terminal avant de payer."
fi

# ─────────────────────────────────────────────────────────────────────────────
# 4. Démarre le forward des webhooks
# ─────────────────────────────────────────────────────────────────────────────
log "Démarrage de 'stripe listen' → ${WEBHOOK_URL}"
log "Laisse ce terminal ouvert : il relaie les événements Stripe vers le controlplane."
exec "${STRIPE_CLI[@]}" listen --forward-to "${WEBHOOK_URL}"
