# ADR : Authentification BFF client confidentiel (cookie chiffré) pour la console

> **Statut : Accepté** — remplace (supersedes) l'[ADR 002](002-authentication.md).
>
> Cet ADR décrit l'architecture d'authentification **réellement déployée**. Le flux
> SPA publique + PKCE + tokens en `sessionStorage` + `Authorization: Bearer` gRPC de
> l'ADR 002 a été retiré du code.

## Contexte et Enjeux

France Nuage migre son IAM vers **FerrisKey**, qui ne supporte **pas PKCE** sur sa
surface OAuth RP. Une SPA publique ne peut donc plus se protéger par PKCE pour
l'échange de code. La console navigateur devient un **client confidentiel** : le
control-plane (Rust, gRPC-web) détient `client_id` + `client_secret`, exécute
l'échange `authorization_code` **côté serveur**, et le navigateur ne reçoit qu'un
**cookie de session httpOnly chiffré** — aucun token en JavaScript.

Le flux est **agnostique du provider** : il fonctionne contre le Keycloak actuel
comme contre FerrisKey, ce qui en fait le chemin de migration sûr. Il est activé par
configuration : les routes `/auth/*` n'existent que lorsqu'un `OIDC_CLIENT_SECRET`
est fourni. L'ancien frontend SPA/PKCE a été supprimé — en l'absence du secret, le
BFF n'est pas monté et la console ne peut pas s'authentifier (une erreur de
déploiement à éviter, pas un repli gracieux vers un flux legacy).

## Décision d'Architecture Adoptée

### BFF client confidentiel (control-plane)

- **Échange de code server-side** (`client_secret_post`) : le control-plane appelle
  le `token_endpoint` du provider avec `client_id` + `client_secret` ; le navigateur
  ne voit jamais ni le secret ni les tokens.
- **Découverte OIDC** au démarrage (`.well-known/openid-configuration`) pour
  résoudre `authorization_endpoint`, `token_endpoint`, `issuer`,
  `end_session_endpoint`.
- **Endpoints** (montés sur l'origine control-plane, à côté de gRPC-web) :
  - `GET /auth/login` — génère `state` (CSRF) + `nonce` (anti-rejeu), les stocke en
    cookies httpOnly courts, redirige vers l'`authorization_endpoint` (scope
    `openid profile email offline_access`).
  - `GET /auth/callback` — valide `state`, échange `code` → tokens, valide l'id_token
    (signature via le cache JWKS partagé, puis `iss`, `aud`, `exp`, `nonce`), scelle
    le cookie de session, redirige vers la console.
  - `GET /auth/refresh` — ouvre le cookie scellé, échange son `refresh_token` contre
    des tokens frais, re-valide l'id_token, re-scelle le cookie (nouvel `exp`,
    refresh token éventuellement rotaté). **Échoue fermé** (cookie effacé + 401) sur
    toute erreur, jamais un 500 ni un succès silencieux.
  - `GET /auth/me` — renvoie l'identité de session + le flag `isAdmin` autoritatif,
    en appliquant l'`exp` interne (court) du cookie.
  - `GET|POST /auth/logout` — efface le cookie et redirige vers l'`end_session_endpoint`
    du provider (RP-initiated logout).

### Cookie de session `frn_session`

- **Chiffré et auto-contenu** : un `SessionPayload { refresh_token, sub, email, exp }`
  scellé avec l'AEAD audité de `frn_crypto` (**XChaCha20-Poly1305**) et un AAD fixe
  (`frn-session-cookie-v1`) qui le sépare des autres ciphertexts (ex. kubeconfigs).
  L'id_token « gras » n'est jamais stocké ; le payload reste minimal pour tenir sous
  la limite navigateur de ~4 KB.
- **Clé** : `AUTH_COOKIE_KEY` (base64, 32 octets), la même que le chemin cookie gRPC
  de `IAM`, pour que les deux s'accordent sur une clé unique. Rotation : voir le
  [runbook de rotation](runbook-auth-cookie-key-rotation.md).
- **Attributs** : `HttpOnly`, `Path=/`, `Secure` (hors dev http local), `SameSite`
  (`Lax` par défaut ; `None; Secure` requis en déploiement cross-site — garde de
  démarrage qui alerte sinon). `Max-Age` = **fenêtre de refresh** (`SESSION_MAX_TTL`,
  défaut 12 h) ; l'`exp` **interne** du payload est la durée courte de l'access/id-token.
- **Bornage** : le cookie scellé est borné, prefixe de nom inclus (`frn_session=` +
  valeur ≤ ~4 KB) ; au-delà, on **échoue bruyamment** (`SessionTooLarge`) plutôt que
  d'expédier un cookie que le navigateur droppe en silence.

### gRPC-web authentifié par cookie

- Les appels gRPC-web du navigateur portent automatiquement le cookie
  (`credentials: 'include'`) ; le control-plane les authentifie en ouvrant le cookie
  scellé dans `IAM::principal`. **Aucun token n'est exposé au JavaScript.**
- **CORS credentialed** : origine console **exacte** (pas de wildcard, illégal avec
  `Access-Control-Allow-Credentials`), listes explicites de méthodes/headers.

### Autorisation — flag admin autoritatif en base

Le flag `isAdmin` est lu depuis la **base de données** du control-plane
(`users.is_admin`, via `User::find_or_create_one_by_email`), exactement la même
source que le RPC `Profile.GetCurrentUser` — **jamais** un rôle porté par un token.

### Contrat d'erreur `?auth_error=<reason>`

Un `/auth/callback` rejeté redirige la console vers son origine avec un code
machine-lisible `?auth_error=<reason>` (au lieu d'échouer sur une page texte de
l'origine control-plane). Les raisons — `state`, `nonce`, `exchange`, `no_id_token`,
`validation`, `session` — réutilisent le code déjà émis comme label de métrique, de
sorte que la redirection et la métrique ne divergent jamais. `session` distingue une
erreur **serveur** de scellement/dimensionnement d'une erreur de **validation**
d'id_token (elles ne polluent pas la même alerte).

### Observabilité

Compteurs Prometheus exposés sur `GET /metrics` (même écouteur HTTP que gRPC-web) :
`auth_login_total`, `auth_callback_reject_total{reason}`, `auth_refresh_total{result}`.
Les valeurs de label sont des `&'static str` typés — jamais un token, un email ou une
clé.

## Sécurité et Bonnes Pratiques

- **CSRF** (`state`) et **anti-rejeu** (`nonce`) validés en temps constant sur le
  round-trip de login.
- **Validation cryptographique d'abord** : la signature de l'id_token est vérifiée
  avant tout décodage des claims (le payload n'est décodé qu'une fois signé).
- **`exp` appliqué sur le chemin cookie** : `IAM::principal`, `/auth/me` et
  `/auth/refresh` refusent une session dont l'`exp` interne a expiré.
- **Fail-closed partout** sur `/auth/refresh` (pas de cookie, déchiffrement KO,
  refresh token absent/rejeté, id_token invalide → cookie effacé + 401).
- **Refresh borné** : un `/auth/refresh` touche le provider **au plus une fois** (pas
  de tempête de retry).

## Conséquences

- **Positif** : aucun token en JavaScript (surface XSS réduite), secret confidentiel
  jamais exposé au navigateur, chemin de migration Keycloak → FerrisKey sans PKCE,
  autorisation admin non falsifiable (source DB), observabilité de bout en bout.
- **Coûts / limites** : l'échange de code et le refresh passent par le control-plane
  (état de session porté par un cookie chiffré, pas de store serveur). La rotation de
  `AUTH_COOKIE_KEY` invalide **toutes** les sessions vivantes (pas de fenêtre
  bi-clé) — voir le [runbook de rotation](runbook-auth-cookie-key-rotation.md).
