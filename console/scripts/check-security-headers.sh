#!/bin/sh
# Garde-fou de régression sur les en-têtes de sécurité de la console.
#
# Pourquoi : la console (console.france-nuage.fr) porte l'authentification et le
# contrôle des ressources cloud ; elle doit annoncer HSTS sur toutes ses réponses.
# Ce test encode le finding #8043 (volet console) : HSTS était absent, et le piège
# d'héritage `add_header` de nginx l'aurait de toute façon perdu pour la location
# des ressources statiques (nginx n'hérite pas des add_header d'un niveau parent
# dès qu'une location redéclare les siens). Le test échoue AVANT le correctif et
# passe APRÈS.
set -eu

FILE="$(CDPATH= cd "$(dirname "$0")/.." && pwd)/nginx.conf"
[ -f "$FILE" ] || { echo "FAIL: $FILE introuvable"; exit 1; }

# X-Content-Type-Options marque chaque location qui pose le groupe d'en-têtes de
# sécurité. HSTS doit être présent au moins autant de fois : sinon une location
# servant du contenu pose ses en-têtes SANS HSTS (piège d'héritage add_header).
hsts=$(grep -Ec 'add_header[[:space:]]+Strict-Transport-Security' "$FILE" || true)
xcto=$(grep -Ec 'add_header[[:space:]]+X-Content-Type-Options' "$FILE" || true)

[ "$hsts" -ge 1 ] || { echo "FAIL: aucun en-tête Strict-Transport-Security dans nginx.conf"; exit 1; }
[ "$hsts" -ge "$xcto" ] || {
  echo "FAIL: HSTS absent d'au moins une location à en-têtes de sécurité ($hsts/$xcto) — piège d'héritage add_header"
  exit 1
}

# Valeur robuste : max-age >= 1 an et includeSubDomains.
maxage=$(grep -E 'Strict-Transport-Security' "$FILE" | grep -oE 'max-age=[0-9]+' | head -1 | cut -d= -f2)
[ -n "${maxage:-}" ] && [ "$maxage" -ge 31536000 ] || { echo "FAIL: HSTS max-age < 31536000 (1 an)"; exit 1; }
grep -E 'Strict-Transport-Security.*includeSubDomains' "$FILE" >/dev/null || { echo "FAIL: HSTS sans includeSubDomains"; exit 1; }

echo "OK: HSTS présent sur $hsts/$xcto location(s) à en-têtes de sécurité, max-age=$maxage, includeSubDomains"
