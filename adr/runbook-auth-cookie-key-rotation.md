# Runbook : rotation de `AUTH_COOKIE_KEY` (clé de scellement du cookie de session)

Contexte :
[ADR 003 — Authentification BFF client confidentiel](003-bff-authentication.md).

## Ce qu'est la clé

`AUTH_COOKIE_KEY` est une clé **AEAD de 32 octets** (XChaCha20-Poly1305) encodée
en **base64**. Elle scelle et ouvre le cookie de session `frn_session` du BFF,
et le **même** secret sert au chemin cookie gRPC de `IAM` — il n'y a qu'une
seule clé.

En production elle vit dans un **SealedSecret**
(`.Values.secrets.existingSecret`, p.ex. `plateforme-prod-secrets`), sous la clé
`auth-cookie-key`, injectée dans le Deployment control-plane via `secretKeyRef`
(`AUTH_COOKIE_KEY`).

## Impact d'une rotation (à connaître AVANT d'agir)

Le scellement **n'a pas de key-id ni de fenêtre bi-clé** : ouvrir un cookie se
fait avec l'unique clé courante. Donc **changer la clé invalide toutes les
sessions vivantes** — chaque `frn_session` déjà émis échoue à l'ouverture.
Concrètement :

- **Tous les utilisateurs sont déconnectés et doivent se reconnecter une fois.**
- Aucune corruption de données : la reconnexion (login OIDC) reforge un cookie
  valide.
- C'est une **limitation connue** (pas de rotation « sans couture »). L'ajouter
  (key-id
  - fenêtre d'acceptation double-clé) est un travail futur ; aujourd'hui la
    rotation est une opération **interruptive planifiée**, à faire hors pic si
    possible.

Corollaire : ne rotater que si nécessaire (compromission suspectée, politique de
rotation périodique) et prévenir que chaque utilisateur devra se reconnecter.

## Procédure

Toutes les commandes s'exécutent depuis un poste ayant accès `kubectl` au
cluster et `kubeseal` (clé publique du controller sealed-secrets). Remplacer
`<ns>` par le namespace de la release et `<secret>` par
`.Values.secrets.existingSecret`.

1. **Générer une nouvelle clé** (32 octets base64) :

   ```bash
   NEW_KEY="$(openssl rand -base64 32)"
   ```

2. **Sceller la nouvelle valeur** pour la clé `auth-cookie-key` du secret :

   ```bash
   echo -n "$NEW_KEY" | kubeseal --raw --name <secret> \
     --namespace <ns> \
     --controller-name sealed-secrets --controller-namespace kube-system \
     --from-file=/dev/stdin
   ```

3. **Remplacer** la valeur `auth-cookie-key` dans le `SealedSecret`
   (`helm/plateforme/templates/sealed-secret-prod.yaml`, champ
   `spec.encryptedData.auth-cookie-key`) par la sortie ci-dessus, puis appliquer
   le chart (`deploy.sh` / `helm upgrade` habituel du repo — jamais un
   `helm upgrade` brut, cf. règles de déploiement).

4. **Redémarrer le control-plane** pour recharger l'environnement (le secret est
   monté en variable d'env, pas rechargé à chaud) :

   ```bash
   kubectl -n <ns> rollout restart deploy/<release>-controlplane
   kubectl -n <ns> rollout status  deploy/<release>-controlplane --timeout=5m
   ```

5. **Vérifier** : `GET /auth/me` sans cookie renvoie `authenticated: false`, et
   un login complet reforge un cookie qui authentifie un appel gRPC-web. Les
   compteurs `auth_login_total` remontent (reconnexions) ; surveiller
   `auth_refresh_total{result="decrypt_fail"}` — un pic transitoire est attendu
   (les anciens cookies ne s'ouvrent plus), il doit retomber une fois le parc
   reconnecté.

## Notes

- Le chemin cookie gRPC de `IAM` et le BFF lisant la **même** variable d'env, la
  rotation les met à jour ensemble d'office (une seule clé, cohérence garantie).
- Ne jamais committer la clé en clair : uniquement la forme scellée
  (`SealedSecret`). En CI éphémère, la valeur claire passe par
  `secrets.authCookieKey` (IdP de test) ; la prod passe toujours par le
  `SealedSecret`.
