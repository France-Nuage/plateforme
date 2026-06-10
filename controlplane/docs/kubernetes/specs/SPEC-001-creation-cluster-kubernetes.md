# SPEC-001 : Creation d'un cluster Kubernetes

## Resume

Un administrateur plateforme enregistre un cluster Kubernetes dans le control plane.
Le kubeconfig est chiffre au repos et n'est jamais re-expose via l'API (write-only).

## Qui

- Seuls les utilisateurs avec `is_admin = true` peuvent creer/modifier/supprimer un cluster
- L'admin gate est verifiee sur chaque RPC (ListClusters, GetCluster, CreateCluster, UpdateCluster, DeleteCluster)

## Flux de creation

- L'admin fournit : `name` (DNS-label, unique), `description` (optionnel), `kubeconfig` (YAML)
- Le serveur valide le nom (regex `^[a-z0-9]([a-z0-9-]*[a-z0-9])?$`, max 63 chars)
- Le serveur effectue un health check synchrone sur le cluster cible :
  - Parse le kubeconfig, construit un client kube-rs
  - Appelle `GET /version` avec timeouts (connect 5s, read 10s, total 20s)
  - Extrait `api_server_url`, `kubernetes_version`, `platform`
  - `kubernetes_version` est normalisee en semver strict : le prefixe `v` du gitVersion
    (ex: `v1.32.2+k3s1` -> `1.32.2+k3s1`) est retire ; si la valeur n'est pas du semver
    parseable, NULL est stocke (le health check reussit quand meme)
  - Si le cluster est injoignable, la creation echoue immediatement (rien n'est persiste)
- Le kubeconfig est chiffre via envelope encryption (voir section suivante)
- Le cluster est insere en base avec `health_status = healthy`

## Encryption du kubeconfig

- Algorithme : XChaCha20-Poly1305 (AEAD, 256-bit keys, 192-bit nonces)
- Schema envelope encryption a deux couches :
  - Un DEK (Data Encryption Key) aleatoire est genere pour chaque kubeconfig
  - Le kubeconfig est chiffre avec le DEK -> `encrypted_kubeconfig` + `kubeconfig_nonce`
  - Le DEK est chiffre avec le KEK (Key Encryption Key) -> `dek_encrypted` + `dek_nonce`
- Le KEK est charge depuis la variable d'environnement `KUBECONFIG_ENCRYPTION_KEY` (base64, 32 bytes)
- Le KEK ne touche jamais la base de donnees, il est en memoire uniquement (ZeroizeOnDrop)
- AAD (Additional Authenticated Data) = `cluster_id || key_version` -- empeche la transplantation de ciphertext entre lignes
- `key_version` (defaut 1) permet la rotation de cle sans re-chiffrer tout d'un coup

## Kubeconfig write-only

- L'API ne retourne jamais le kubeconfig ni les champs chiffres
- Le proto `KubernetesClusterProto` expose uniquement : id, name, description, api_server_url, ca_fingerprint, kubernetes_version, platform, health_status, timestamps
- Le dechiffrement (`decrypt_kubeconfig`) est reserve aux composants internes (worker de deploiement)

## Mise a jour

- Sans nouveau kubeconfig : seuls name/description sont mis a jour
- Avec nouveau kubeconfig : health check + re-chiffrement complet

## Suppression

- Refusee si des instances de services manages sont encore hebergees sur le
  cluster (`ClusterHasInstances`)
- Sinon, suppression directe de la ligne

## Labels de cluster

- Table `kubernetes.label` : paires cle/valeur reutilisables (ex: `availability=ft`),
  CITEXT (insensibles a la casse), contraintes `length < 50` et charset `[a-zA-Z0-9-]`
- Table de jointure `kubernetes.cluster_label` : attache un label a un cluster
  (attach/detach idempotents)
- Flag `system` : un label `system = true` appartient au control plane ; l'API
  refuse create/delete/attach/detach dessus, meme pour les admins
  (`SystemLabelReadOnly`)
- RPCs admin : ListLabels, CreateLabel, DeleteLabel, AttachClusterLabel,
  DetachClusterLabel ; les labels d'un cluster sont retournes dans
  `KubernetesClusterProto.labels`

## Selection d'un cluster

- `pick_healthy_cluster_matching` : selectionne un cluster `health_status = healthy`
  qui porte TOUS les labels requis par le `deploy_target` du service manage
- Aucun candidat -> erreur typee `NoClusterMatchingDeployTarget` (pas de fallback)
- Plusieurs candidats -> choix aleatoire pour repartir la charge
- Utilise au deploiement d'une instance (voir SPEC-004) ; les projets ne sont
  plus lies a un cluster

## Stockage en base

- Schema `kubernetes`, table `kubernetes.cluster`
- Colonnes chiffrees : `encrypted_kubeconfig`, `kubeconfig_nonce`, `dek_encrypted`, `dek_nonce`
- Colonnes metadonnees : `name`, `description`, `api_server_url`, `ca_fingerprint`, `kubernetes_version`, `platform`
- `kubernetes_version` porte une contrainte CHECK semver strict (NULL autorise) ; la
  normalisation applicative garantit qu'une valeur non parseable devient NULL
- Colonnes sante : `health_status` (enum healthy/unreachable), `last_health_check_at`
- Index sur `key_version` pour les jobs de re-chiffrement lors de rotation de cle
