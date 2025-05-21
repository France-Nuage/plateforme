# ADR : Architecture d'authentification et d'autorisation avec OpenID Connect pour France-Nuage

## Contexte et Enjeux

France-Nuage souhaite mettre en place une architecture d'authentification moderne et sécurisée pour ses services backend en Rust et applications frontend. Le système doit permettre aux utilisateurs de s'authentifier via des fournisseurs d'identité externes (Google, GitHub, etc.) tout en maintenant un contrôle fin des autorisations côté backend. L'objectif est de fournir une expérience utilisateur fluide avec une Single Page Application (SPA) en NextJS, tout en garantissant la sécurité des API gRPC backend.

Ce besoin s'inscrit dans une optique de **modernité et de sécurité** : en évitant de gérer directement les mots de passe utilisateurs, on réduit les risques de sécurité tout en offrant une expérience d'authentification familière (OAuth/OIDC). Le système doit également s'intégrer dans l'écosystème France-Nuage (technologies open-source, standards ouverts) et respecter les bonnes pratiques de sécurité pour les applications distribuées.

## Options Envisagées

Plusieurs approches ont été étudiées pour construire ce système d'authentification :

### Option 1 : NextAuth.js avec backend intégré

**Description** : Utiliser NextAuth.js avec un backend NextJS traditionnel gérant les sessions via cookies.

**Avantages** :
- Solution tout-en-un bien documentée
- Gestion automatique des sessions
- Support natif de nombreux providers

**Inconvénients** :
- Couplage fort entre frontend et backend NextJS
- Ne s'adapte pas aux services gRPC Rust indépendants
- Complexité pour exposer l'état d'authentification aux services external
- Sessions stateful incompatibles avec l'architecture microservices ciblée

Cette option a été écartée car elle ne correspond pas à l'architecture distribuée souhaitée avec des services Rust indépendants.

### Option 2 : Sessions traditionnelles avec base de données partagée

**Description** : Système de sessions stockées en base de données partagée entre tous les services.

**Avantages** :
- Contrôle total sur la gestion des sessions
- Possibilité de révocation immédiate

**Inconvénients** :
- Nécessite un stockage partagé (Redis/PostgreSQL) accessible par tous les services
- Complexité des appels réseau pour chaque validation
- Point de défaillance unique
- Scaling horizontal plus complexe
- Ne tire pas parti des standards OAuth/OIDC

### Option 3 : Architecture stateless avec validation de tokens côté services

**Description** : Authentification via OIDC (OpenID Connect) avec tokens JWT validés directement par chaque service backend.

**Avantages** :
- Architecture stateless compatible microservices
- Validation locale sans appel réseau
- Standards éprouvés (OAuth 2.0 / OIDC)
- Scalabilité horizontale native
- Indépendance des services

**Inconvénients** :
- Complexité initiale de mise en place
- Gestion de l'expiration des tokens
- Révocation plus complexe (dépend de l'expiration)

Après analyse, **l'option 3 a été retenue** pour sa compatibilité avec l'architecture microservices et gRPC ciblée.

## Décision d'Architecture Adoptée

Nous décidons de mettre en place une **architecture d'authentification stateless basée sur OpenID Connect**, avec les caractéristiques suivantes :

### Frontend SPA (NextJS)

* **Gestion d'état avec Redux Toolkit** pour centraliser l'état d'authentification dans le store global de l'application
* **Flux d'authentification OIDC implicite** : redirection vers le provider (Google, GitHub, etc.) avec `response_type=id_token token`
* **Stockage des tokens en sessionStorage** pour la persistance entre rechargements de page (plus sécurisé que localStorage contre XSS)
* **Validation initiale côté frontend** avec envoi de l'ID token au backend pour confirmation
* **Gestion automatique des redirections** après authentification via une route de callback dynamique `/auth/callback/[provider]`

### Backend Rust (Services gRPC)

* **Validation de tokens JWT côté service** : chaque service Rust valide indépendamment les ID tokens reçus
* **Utilisation de la crate `openidconnect-rs`** pour la validation des tokens selon les standards OIDC
* **Cache des clés publiques JWKS** avec TTL (24h) pour éviter les appels répétés aux providers
* **Support multi-providers** : architecture agnostique permettant Google, GitHub, Facebook, etc.
* **Extraction automatique des informations utilisateur** depuis les claims JWT (sub, email, name, etc.)

### Intégration gRPC avec Authentification

* **Transmission via metadata gRPC** : le frontend envoie l'ID token dans les métadonnées de chaque appel gRPC
* **Intercepteur d'authentification** : middleware Rust validant automatiquement les tokens sur tous les appels protégés
* **Injection des informations utilisateur** dans le contexte de la requête pour utilisation par les handlers

### Configuration des Providers OIDC

* **Google OAuth** : configuration avec `authorized_javascript_origins` pour le flux implicite
* **Support extensible** : table de configuration des providers avec endpoints de découverte automatique
* **Validation des audiences** : vérification que les tokens sont bien destinés aux client IDs France-Nuage

## Flux d'Authentification Détaillé

### 1. Initiation de l'authentification

Le processus commence lorsque l'utilisateur clique sur "Se connecter". Le frontend Redux dispatch une action `initiateLogin()` qui génère une URL OAuth et redirige le navigateur vers le provider d'identité externe.

### 2. Callback et validation

Après authentification, le provider redirige l'utilisateur vers la route de callback avec les tokens dans le fragment d'URL. Le frontend extrait l'ID token et l'envoie au backend Rust pour validation cryptographique.

### 3. Appels API authentifiés

Une fois validé, l'ID token est stocké et utilisé dans les métadonnées de tous les appels gRPC subséquents. L'intercepteur d'authentification valide chaque token et injecte les informations utilisateur dans le contexte.

## Modèle de Données et Configuration

### Structure des informations utilisateur

Les informations utilisateur extraites des claims JWT incluent un identifiant unique (sub), l'email, le nom, l'avatar et le statut de vérification de l'email.

### Configuration des providers

Configuration statique des providers supportés avec leurs endpoints de découverte OIDC. Chaque provider est identifié par son URL d'issuer et son endpoint de configuration well-known.

### Gestion du cache JWKS

Cache en mémoire des clés publiques JWKS avec TTL de 24h par défaut pour optimiser les performances de validation.

## API Frontend (Redux)

### Actions principales

* `initiateLogin(provider)` : Lance le processus d'authentification
* `validateToken(token)` : Valide un token avec le backend
* `logout()` : Nettoie l'état local et les tokens stockés
* `checkAuthStatus()` : Vérifie l'état d'authentification au chargement

### Sélecteurs Redux

Sélecteurs pour accéder aux informations utilisateur, au token d'authentification, au statut de connexion et aux headers d'authentification pour les appels API.

## Sécurité et Bonnes Pratiques

### Côté Frontend

* **Protection XSS** : utilisation de sessionStorage plutôt que localStorage
* **Validation des tokens** : vérification côté backend avant acceptation
* **Gestion des erreurs** : logout automatique en cas de token invalide
* **Nettoyage des URLs** : suppression des tokens des fragments après traitement

### Côté Backend

* **Validation cryptographique** : vérification complète des signatures JWT
* **Vérification des claims** : issuer, audience, expiration
* **Rate limiting** : protection contre les attaques par force brute
* **Logs de sécurité** : traçabilité des tentatives d'authentification

### Protection des API

Chaque endpoint protégé vérifie automatiquement la présence et la validité des informations utilisateur injectées par l'intercepteur d'authentification. Les permissions sont vérifiées avant l'exécution de la logique métier.

## Justifications Techniques

### Choix d'OpenID Connect sur OAuth 2.0 pur

OpenID Connect apporte une **standardisation** des informations d'identité au-dessus d'OAuth 2.0. Les ID tokens JWT contiennent des claims standardisés (sub, email, name) qui simplifient l'extraction des informations utilisateur sans appel supplémentaire à une API userinfo. De plus, OIDC est largement supporté par les providers majeurs (Google, Microsoft, etc.).

### Utilisation des ID tokens pour l'authentification

Bien que certaines documentations (Auth0) déconseillent l'envoi d'ID tokens aux APIs, notre cas d'usage est spécifique : nous construisons un **service de validation d'identité** et non une API de ressources traditionnelle. L'ID token est conçu pour être vérifié par l'application cliente (notre backend fait partie de l'application globale). Cette approche est validée par :

- **Stateless** : pas de stockage de session côté serveur
- **Performance** : validation locale sans appel externe
- **Standards** : utilisation conforme d'OIDC pour la vérification d'identité
- **Sécurité** : validation cryptographique complète des tokens

### Redux Toolkit pour la gestion d'état

Redux Toolkit offre plusieurs avantages pour la gestion de l'authentification :

- **State centralisé** : accessible depuis tous les composants
- **DevTools** : débogage facilité des changements d'état
- **Middleware** : gestion des effets de bord (appels API)
- **Persistence** : facilite la synchronisation avec le stockage local
- **Predictabilité** : flux de données unidirectionnel

### Architecture stateless avec gRPC

Le choix stateless s'impose pour les communications gRPC car :

- **Pas de cookies** : gRPC ne supporte pas les cookies HTTP
- **Metadata** : mécanisme naturel pour transporter l'authentification
- **Microservices** : chaque service peut valider indépendamment
- **Scalabilité** : pas de stockage de session partagé requis