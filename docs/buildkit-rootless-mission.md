# Mission : Migration vers BuildKit Rootless

## Contexte

### Situation actuelle

La CI GitLab utilise actuellement **Docker-in-Docker (DinD)** pour builder les
images :

```yaml
image: docker:29-cli
services:
  - docker:29-dind
```

Cette approche nécessite que le runner Kubernetes exécute les pods en mode
**privileged**, ce qui pose des problèmes de sécurité :

```text
┌─────────────────────────────────────────────────────────┐
│                   Kubernetes Node                       │
│  ┌───────────────────────────────────────────────────┐  │
│  │              CI Pod (privileged: true)            │  │
│  │  ┌─────────────┐    ┌─────────────────────────┐   │  │
│  │  │ docker:cli  │───▶│ docker:dind (sidecar)   │   │  │
│  │  │             │    │                         │   │  │
│  │  │ buildx      │    │ Docker Daemon           │   │  │
│  │  │ commands    │    │ (accès root complet)    │   │  │
│  │  └─────────────┘    └─────────────────────────┘   │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘

Problèmes :
- Le pod a accès root au node
- Risque d'évasion de conteneur
- Non compatible avec les politiques de sécurité strictes
```

### Objectif

Migrer vers **BuildKit avec le driver Kubernetes** pour :

1. Supprimer le besoin de `privileged: true` sur le runner
2. Améliorer la sécurité du cluster
3. Préparer l'infrastructure pour les futures évolutions (chart Helm)

```text
┌─────────────────────────────────────────────────────────┐
│                   Kubernetes Node                       │
│  ┌───────────────────────────────────────────────────┐  │
│  │           CI Pod (privileged: false)              │  │
│  │  ┌─────────────────────────────────────────────┐  │  │
│  │  │              buildx client                  │  │  │
│  │  │  docker buildx create --driver=kubernetes   │  │  │
│  │  └──────────────────┬──────────────────────────┘  │  │
│  └─────────────────────┼─────────────────────────────┘  │
│                        │ API K8s                        │
│                        ▼                                │
│  ┌───────────────────────────────────────────────────┐  │
│  │         BuildKit Pod (éphémère, rootless)         │  │
│  │  ┌─────────────────────────────────────────────┐  │  │
│  │  │              moby/buildkit                  │  │  │
│  │  │         (userspace, pas de daemon)          │  │  │
│  │  └─────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘

Avantages :
- Pas de privilèges élevés
- Isolation complète des builds
- Pods de build éphémères et nettoyés automatiquement
```

## Solution technique : BuildKit Kubernetes Driver

### Pourquoi ce choix ?

| Critère | Docker-in-Docker | Kaniko | BuildKit K8s Driver |
|---------|------------------|--------|---------------------|
| Maintenance | Active | Abandonnée | Active (Docker Inc.) |
| Privilèges requis | Root/privileged | Aucun | Aucun |
| Compatibilité Dockerfile | Totale | Partielle | Totale |
| Cache registry | Oui | Oui | Oui |
| Multi-stage builds | Oui | Oui | Oui |
| BuildKit features | Oui | Non | Oui (natif) |
| Intégration buildx | Native | Non | Native |

Le **BuildKit Kubernetes Driver** est la solution recommandée car :

- Intégré nativement à `docker buildx` (pas de nouvel outil à apprendre)
- Supporte toutes les fonctionnalités Dockerfile
- Compatible avec le cache registry existant
- Crée des pods éphémères qui se nettoient automatiquement
- Maintenu activement par Docker Inc.

### Comment ça fonctionne ?

```text
1. Le job CI crée un "builder" avec le driver kubernetes
   $ docker buildx create --driver=kubernetes --name=k8s-builder

2. Buildx demande à l'API Kubernetes de créer un pod BuildKit
   [CI Pod] ──▶ [API K8s] ──▶ [BuildKit Pod]

3. Le build s'exécute dans le pod BuildKit (rootless)
   [CI Pod] ◀──────────────▶ [BuildKit Pod]
              gRPC connection

4. L'image est poussée directement vers le registry
   [BuildKit Pod] ──▶ [registry.gitlab.com]

5. Le pod BuildKit est supprimé automatiquement
```

## Prérequis

### Permissions Kubernetes

Le ServiceAccount du runner GitLab doit pouvoir créer des pods. Créer un
Role/RoleBinding si nécessaire :

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: buildkit-role
  namespace: gitlab-runner  # Adapter au namespace du runner
rules:
  - apiGroups: [""]
    resources: ["pods"]
    verbs: ["create", "delete", "get", "list", "watch"]
  - apiGroups: [""]
    resources: ["pods/exec"]
    verbs: ["create"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: buildkit-rolebinding
  namespace: gitlab-runner
subjects:
  - kind: ServiceAccount
    name: gitlab-runner  # Adapter au nom du SA
    namespace: gitlab-runner
roleRef:
  kind: Role
  name: buildkit-role
  apiGroup: rbac.authorization.k8s.io
```

### Image CI

Remplacer `docker:29-cli` par une image avec `buildx` et `kubectl` :

```dockerfile
# Proposition : créer une image CI custom
FROM docker:29-cli

# Installer kubectl pour le driver kubernetes
RUN apk add --no-cache curl \
    && curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" \
    && install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl \
    && rm kubectl

# Buildx est déjà inclus dans docker:29-cli
```

Ou utiliser une image existante comme `alpine/docker-cli` avec kubectl.

## Plan de migration

### Vue d'ensemble

```text
Phase 0 : Préparation
    │
    ▼
Phase 1 : Proof of Concept
    │   └── Migrer "protocol" (service le plus simple)
    │
    ▼
Phase 2 : Services Node.js
    │   ├── Migrer "node-sdk"
    │   └── Migrer "console"
    │
    ▼
Phase 3 : Services de test
    │   └── Migrer "system-tests"
    │
    ▼
Phase 4 : Services Rust (les plus complexes)
    │   └── Migrer "rust-services" (controlplane, authz-worker, synchronizer)
    │
    ▼
Phase 5 : Finalisation
        ├── Supprimer DinD
        ├── Passer privileged: false
        └── Documentation finale
```

### Ordre de migration justifié

| Ordre | Service | Justification |
|-------|---------|---------------|
| 1 | protocol | Dockerfile simple, pas de dépendances, build rapide |
| 2 | node-sdk | Node.js simple, similaire au suivant |
| 3 | console | Node.js avec build args, teste les paramètres |
| 4 | system-tests | Dépend de node-sdk, valide le multi-stage |
| 5 | rust-services | Le plus complexe : 3 targets, cache partagé |

---

## Phase 0 : Préparation

### Objectifs

- Comprendre la CI actuelle
- Préparer l'environnement
- Créer une branche de travail

### Étapes

#### 0.1 Analyser la CI actuelle

Lire et comprendre `.gitlab-ci.yml` :

```bash
# Fichiers à étudier
.gitlab-ci.yml              # Pipeline principale
docker-compose.yml          # Pour comprendre les services
controlplane/Dockerfile     # Multi-target Rust
console/Dockerfile          # Multi-stage Node.js
protocol/Dockerfile         # Simple
```

Questions à se poser :

- Comment le cache est-il géré ?
- Quelles sont les dépendances entre les jobs ?
- Comment les variables d'environnement sont-elles utilisées ?

#### 0.2 Créer la branche de travail

```bash
git checkout -b feat/buildkit-rootless
```

#### 0.3 Vérifier les permissions K8s

Depuis un pod du runner (ou en local avec le même kubeconfig) :

```bash
# Tester si on peut créer des pods
kubectl auth can-i create pods --namespace=gitlab-runner

# Si non, demander l'application du RBAC (voir section Prérequis)
```

#### 0.4 Préparer l'image CI (optionnel)

Si une image custom est nécessaire, la créer et la pousser :

```bash
# Dans un nouveau dossier ci-image/
docker build -t registry.gitlab.com/.../ci-image:latest .
docker push registry.gitlab.com/.../ci-image:latest
```

---

## Phase 1 : Proof of Concept - Protocol

### Objectifs

- Valider que le driver Kubernetes fonctionne
- Établir le pattern de migration
- Documenter les problèmes rencontrés

### État actuel du job

```yaml
"build / protocol":
  stage: build
  script:
    - |
      docker buildx build \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/protocol:buildcache \
        --cache-to type=registry,ref=${CI_REGISTRY_IMAGE}/protocol:buildcache,mode=max \
        --tag ${CI_REGISTRY_IMAGE}/protocol:${CI_COMMIT_REF_SLUG} \
        --push \
        --file protocol/Dockerfile \
        protocol
```

### Nouveau job avec driver Kubernetes

```yaml
"build / protocol":
  stage: build
  image: docker:29-cli
  # Plus besoin du service DinD !
  # services:
  #   - docker:29-dind
  variables:
    # Désactiver les variables Docker TLS (plus de daemon local)
    DOCKER_HOST: ""
    DOCKER_TLS_CERTDIR: ""
  before_script:
    # Authentification au registry
    - echo "$CI_REGISTRY_PASSWORD" | docker login -u "$CI_REGISTRY_USER" --password-stdin $CI_REGISTRY

    # Créer le builder avec le driver kubernetes
    - |
      docker buildx create \
        --driver=kubernetes \
        --driver-opt=namespace=${BUILDKIT_NAMESPACE:-gitlab-runner} \
        --driver-opt=rootless=true \
        --name=k8s-builder \
        --use

    # Attendre que le builder soit prêt
    - docker buildx inspect --bootstrap
  script:
    - |
      docker buildx build \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/protocol:buildcache \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/protocol:buildcache-${CI_COMMIT_REF_SLUG} \
        --cache-to type=registry,ref=${CI_REGISTRY_IMAGE}/protocol:buildcache-${CI_COMMIT_REF_SLUG},mode=max \
        --cache-to type=registry,ref=${CI_REGISTRY_IMAGE}/protocol:buildcache,mode=max \
        --tag ${CI_REGISTRY_IMAGE}/protocol:${CI_COMMIT_REF_SLUG} \
        --push \
        --file protocol/Dockerfile \
        protocol
  after_script:
    # Nettoyer le builder
    - docker buildx rm k8s-builder || true
  rules:
    - changes:
        - protocol/**/*
        - .gitlab-ci.yml
```

### Points de validation

- [ ] Le job démarre sans erreur
- [ ] Le pod BuildKit est créé dans le namespace
- [ ] Le build s'exécute correctement
- [ ] L'image est poussée sur le registry
- [ ] Le cache fonctionne (2ème build plus rapide)
- [ ] Le pod BuildKit est nettoyé après le build

### Debugging

Si problème, vérifier :

```bash
# Voir les pods BuildKit créés
kubectl get pods -n gitlab-runner -l app=buildkit

# Logs du pod BuildKit
kubectl logs -n gitlab-runner -l app=buildkit

# Décrire le pod en cas d'erreur
kubectl describe pod -n gitlab-runner -l app=buildkit
```

---

## Phase 2 : Services Node.js

### 2.1 Migration de node-sdk

Similaire à protocol, mais vérifier :

- Le multi-stage build fonctionne
- Les node_modules sont correctement cachés

```yaml
"build / node-sdk":
  stage: build
  image: docker:29-cli
  variables:
    DOCKER_HOST: ""
    DOCKER_TLS_CERTDIR: ""
  before_script:
    - echo "$CI_REGISTRY_PASSWORD" | docker login -u "$CI_REGISTRY_USER" --password-stdin $CI_REGISTRY
    - |
      docker buildx create \
        --driver=kubernetes \
        --driver-opt=namespace=${BUILDKIT_NAMESPACE:-gitlab-runner} \
        --driver-opt=rootless=true \
        --name=k8s-builder \
        --use
    - docker buildx inspect --bootstrap
  script:
    - |
      docker buildx build \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/node-sdk:buildcache \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/node-sdk:buildcache-${CI_COMMIT_REF_SLUG} \
        --cache-to type=registry,ref=${CI_REGISTRY_IMAGE}/node-sdk:buildcache-${CI_COMMIT_REF_SLUG},mode=max \
        --cache-to type=registry,ref=${CI_REGISTRY_IMAGE}/node-sdk:buildcache,mode=max \
        --tag ${CI_REGISTRY_IMAGE}/node-sdk:${CI_COMMIT_REF_SLUG} \
        --push \
        --file node-sdk/Dockerfile \
        node-sdk
  after_script:
    - docker buildx rm k8s-builder || true
  rules:
    - changes:
        - node-sdk/**/*
        - .gitlab-ci.yml
```

### 2.2 Migration de console

Plus complexe car utilise des `--build-arg` :

```yaml
"build / console":
  stage: build
  image: docker:29-cli
  variables:
    DOCKER_HOST: ""
    DOCKER_TLS_CERTDIR: ""
    TARGET: development
  before_script:
    - echo "$CI_REGISTRY_PASSWORD" | docker login -u "$CI_REGISTRY_USER" --password-stdin $CI_REGISTRY
    - |
      docker buildx create \
        --driver=kubernetes \
        --driver-opt=namespace=${BUILDKIT_NAMESPACE:-gitlab-runner} \
        --driver-opt=rootless=true \
        --name=k8s-builder \
        --use
    - docker buildx inspect --bootstrap
  script:
    - |
      docker buildx build \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/console:buildcache \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/console:buildcache-${CI_COMMIT_REF_SLUG} \
        --cache-to type=registry,ref=${CI_REGISTRY_IMAGE}/console:buildcache-${CI_COMMIT_REF_SLUG},mode=max \
        --cache-to type=registry,ref=${CI_REGISTRY_IMAGE}/console:buildcache,mode=max \
        --build-arg VITE_OIDC_CLIENT_ID=${OIDC_CLIENT_ID:-francenuage} \
        --build-arg VITE_OIDC_PROVIDER_NAME=${OIDC_PROVIDER_NAME:-keycloak} \
        --build-arg VITE_OIDC_PROVIDER_URL=${OIDC_PROVIDER_URL:-https://keycloak.test/realms/francenuage} \
        --build-arg VITE_APPLICATION_DEFAULT_MODE=${VITE_APPLICATION_DEFAULT_MODE:-rpc} \
        --build-arg VITE_CONTROLPLANE_URL=${CONTROLPLANE_URL:-https://controlplane.test} \
        --target ${TARGET} \
        --tag ${CI_REGISTRY_IMAGE}/console:${CI_COMMIT_REF_SLUG}-${TARGET} \
        --push \
        --file console/Dockerfile \
        .
  after_script:
    - docker buildx rm k8s-builder || true
  rules:
    - changes:
        - console/**/*
        - node-sdk/**/*
        - .gitlab-ci.yml
```

### Points de validation Phase 2

- [ ] node-sdk build et push OK
- [ ] console build avec build-args OK
- [ ] Les caches sont partagés correctement
- [ ] Les targets (development/release) fonctionnent

---

## Phase 3 : Services de test

### Migration de system-tests

```yaml
"build / system-tests":
  stage: build
  image: docker:29-cli
  variables:
    DOCKER_HOST: ""
    DOCKER_TLS_CERTDIR: ""
  before_script:
    - echo "$CI_REGISTRY_PASSWORD" | docker login -u "$CI_REGISTRY_USER" --password-stdin $CI_REGISTRY
    - |
      docker buildx create \
        --driver=kubernetes \
        --driver-opt=namespace=${BUILDKIT_NAMESPACE:-gitlab-runner} \
        --driver-opt=rootless=true \
        --name=k8s-builder \
        --use
    - docker buildx inspect --bootstrap
  script:
    - |
      docker buildx build \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/system-tests:buildcache \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/system-tests:buildcache-${CI_COMMIT_REF_SLUG} \
        --cache-to type=registry,ref=${CI_REGISTRY_IMAGE}/system-tests:buildcache-${CI_COMMIT_REF_SLUG},mode=max \
        --cache-to type=registry,ref=${CI_REGISTRY_IMAGE}/system-tests:buildcache,mode=max \
        --tag ${CI_REGISTRY_IMAGE}/system-tests:${CI_COMMIT_REF_SLUG} \
        --push \
        --file system-tests/Dockerfile \
        .
  after_script:
    - docker buildx rm k8s-builder || true
  rules:
    - changes:
        - system-tests/**/*
        - node-sdk/**/*
        - .gitlab-ci.yml
```

---

## Phase 4 : Services Rust

### La complexité des services Rust

Le job `rust-services` est le plus complexe car :

1. **3 targets différents** : controlplane, authz-worker, synchronizer
2. **Cache partagé** : Un seul build compile le workspace, les suivants réutilisent
3. **Dépendances de build** : Le premier build exporte le cache, les autres l'importent

### Stratégie de migration

Deux options :

**Option A : 3 jobs séparés avec cache partagé**

Chaque service a son propre job, mais ils partagent le même cache.

**Option B : 1 job avec builds séquentiels (recommandé)**

Garder la logique actuelle dans un seul job.

### Migration du job rust-services

```yaml
"build / rust-services":
  stage: build
  image: docker:29-cli
  variables:
    DOCKER_HOST: ""
    DOCKER_TLS_CERTDIR: ""
    TARGET: development
  before_script:
    - echo "$CI_REGISTRY_PASSWORD" | docker login -u "$CI_REGISTRY_USER" --password-stdin $CI_REGISTRY
    - |
      docker buildx create \
        --driver=kubernetes \
        --driver-opt=namespace=${BUILDKIT_NAMESPACE:-gitlab-runner} \
        --driver-opt=rootless=true \
        --name=k8s-builder \
        --use
    - docker buildx inspect --bootstrap
  script:
    - |
      set -e

      # Build 1: controlplane (exporte le cache du workspace)
      docker buildx build \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/rust-services:buildcache \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/rust-services:buildcache-${CI_COMMIT_REF_SLUG} \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/controlplane:${CI_COMMIT_REF_SLUG}-${TARGET} \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/controlplane:master-${TARGET} \
        --cache-to type=registry,ref=${CI_REGISTRY_IMAGE}/rust-services:buildcache-${CI_COMMIT_REF_SLUG},mode=max \
        --cache-to type=registry,ref=${CI_REGISTRY_IMAGE}/rust-services:buildcache,mode=max \
        --target ${TARGET} \
        --tag ${CI_REGISTRY_IMAGE}/controlplane:${CI_COMMIT_REF_SLUG}-${TARGET} \
        --push \
        --file controlplane/Dockerfile \
        controlplane

      # Build 2: authz-worker (réutilise le cache)
      docker buildx build \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/rust-services:buildcache \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/rust-services:buildcache-${CI_COMMIT_REF_SLUG} \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/authz-worker:${CI_COMMIT_REF_SLUG}-${TARGET} \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/authz-worker:master-${TARGET} \
        --target ${TARGET}-authz-worker \
        --tag ${CI_REGISTRY_IMAGE}/authz-worker:${CI_COMMIT_REF_SLUG}-${TARGET} \
        --push \
        --file controlplane/Dockerfile \
        controlplane

      # Build 3: synchronizer (réutilise le cache)
      docker buildx build \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/rust-services:buildcache \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/rust-services:buildcache-${CI_COMMIT_REF_SLUG} \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/synchronizer:${CI_COMMIT_REF_SLUG}-${TARGET} \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/synchronizer:master-${TARGET} \
        --target ${TARGET}-synchronizer \
        --tag ${CI_REGISTRY_IMAGE}/synchronizer:${CI_COMMIT_REF_SLUG}-${TARGET} \
        --push \
        --file controlplane/Dockerfile \
        controlplane
  after_script:
    - docker buildx rm k8s-builder || true
  rules:
    - changes:
        - controlplane/**/*
        - .gitlab-ci.yml
```

### Points de validation Phase 4

- [ ] Les 3 images sont buildées correctement
- [ ] Le cache est partagé entre les builds
- [ ] Le 2ème et 3ème build sont significativement plus rapides
- [ ] Les targets development et release fonctionnent

---

## Phase 5 : Finalisation

### 5.1 Factoriser le code commun

Créer un template YAML pour éviter la duplication :

```yaml
# Template commun pour tous les builds
.buildkit_template:
  image: docker:29-cli
  variables:
    DOCKER_HOST: ""
    DOCKER_TLS_CERTDIR: ""
  before_script:
    - echo "$CI_REGISTRY_PASSWORD" | docker login -u "$CI_REGISTRY_USER" --password-stdin $CI_REGISTRY
    - |
      docker buildx create \
        --driver=kubernetes \
        --driver-opt=namespace=${BUILDKIT_NAMESPACE:-gitlab-runner} \
        --driver-opt=rootless=true \
        --name=k8s-builder \
        --use
    - docker buildx inspect --bootstrap
  after_script:
    - docker buildx rm k8s-builder || true

# Utilisation
"build / protocol":
  extends: .buildkit_template
  stage: build
  script:
    - |
      docker buildx build \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/protocol:buildcache \
        # ... reste du build
```

### 5.2 Supprimer DinD

Une fois tous les jobs migrés, supprimer de `.gitlab-ci.yml` :

```yaml
# SUPPRIMER ces lignes globales
services:
  - docker:29-dind

# SUPPRIMER ces variables
variables:
  DOCKER_HOST: tcp://docker:2376
  DOCKER_TLS_CERTDIR: "/certs"
  DOCKER_TLS_VERIFY: 1
  DOCKER_CERT_PATH: "$DOCKER_TLS_CERTDIR/client"
```

### 5.3 Configurer le runner en non-privileged

Modifier la configuration du runner GitLab (dans le Helm chart du runner ou
le config.toml) :

```toml
[[runners]]
  [runners.kubernetes]
    privileged = false
    # ... autres options
```

### 5.4 Nettoyer les jobs legacy

Supprimer les jobs de compatibilité qui ne sont plus nécessaires :

```yaml
# À SUPPRIMER une fois la migration validée
"build / controlplane":
  # ...
  rules:
    - when: never

"build / authz-worker":
  # ...
  rules:
    - when: never

"build / synchronizer":
  # ...
  rules:
    - when: never
```

---

## Tests de validation finale

### Checklist complète

#### Fonctionnel

- [ ] Tous les jobs de build passent
- [ ] Les images sont poussées sur le registry
- [ ] Le cache fonctionne (builds suivants plus rapides)
- [ ] Les tests unitaires passent
- [ ] Les tests système passent
- [ ] Le déploiement prod fonctionne

#### Sécurité

- [ ] Le runner fonctionne avec `privileged: false`
- [ ] Aucun pod ne demande de capabilities root
- [ ] Les pods BuildKit sont éphémères et nettoyés

#### Performance

- [ ] Temps de build comparable ou meilleur
- [ ] Pas de pods BuildKit orphelins

### Script de validation

```bash
#!/bin/bash
# validate-migration.sh

echo "=== Validation de la migration BuildKit ==="

# 1. Vérifier qu'aucun DinD n'est utilisé
echo "Checking for DinD references..."
if grep -r "docker:.*-dind" .gitlab-ci.yml; then
    echo "WARN: DinD still referenced in .gitlab-ci.yml"
else
    echo "OK: No DinD references"
fi

# 2. Vérifier que tous les jobs utilisent le template
echo "Checking buildkit template usage..."
if grep -c "extends: .buildkit_template" .gitlab-ci.yml; then
    echo "OK: Jobs use buildkit template"
fi

# 3. Vérifier les variables Docker
echo "Checking Docker variables..."
if grep "DOCKER_HOST: tcp://" .gitlab-ci.yml; then
    echo "WARN: Old DOCKER_HOST still present"
else
    echo "OK: No legacy DOCKER_HOST"
fi

echo "=== Validation complete ==="
```

---

## Livrables attendus

### Code

1. **`.gitlab-ci.yml` modifié** avec tous les jobs migrés
2. **Template `.buildkit_template`** factorisé
3. **(Optionnel) Image CI custom** si nécessaire

### Documentation

1. **README dans `docs/`** expliquant :
   - L'architecture BuildKit Kubernetes driver
   - Comment ajouter un nouveau job de build
   - Comment débugger les problèmes de build

2. **Schéma d'architecture** (format Mermaid ou draw.io) :
   - Flux du build avec le driver Kubernetes
   - Interactions entre les pods

3. **Mise à jour du CLAUDE.md** avec les nouvelles conventions de build

### Validation

1. **Pipeline verte sur master** après merge
2. **Runner en `privileged: false`** confirmé

---

## Ressources

### Documentation officielle

- [BuildKit GitHub](https://github.com/moby/buildkit)
- [Docker Buildx Kubernetes Driver](https://docs.docker.com/build/builders/drivers/kubernetes/)
- [GitLab CI/CD with Kubernetes](https://docs.gitlab.com/runner/executors/kubernetes/)

### Commandes utiles

```bash
# Voir les builders disponibles
docker buildx ls

# Inspecter un builder
docker buildx inspect k8s-builder

# Voir les pods BuildKit
kubectl get pods -n gitlab-runner -l app=buildkit

# Logs en temps réel
kubectl logs -f -n gitlab-runner -l app=buildkit

# Supprimer un builder bloqué
docker buildx rm k8s-builder --force
```

### Troubleshooting

| Problème | Cause probable | Solution |
|----------|----------------|----------|
| `permission denied` | RBAC manquant | Appliquer le Role/RoleBinding |
| `context deadline exceeded` | Timeout réseau | Augmenter `--driver-opt=timeout=` |
| Pod BuildKit pending | Pas de ressources | Vérifier les quotas du namespace |
| Cache non utilisé | Registry auth | Vérifier le login Docker |
| Build lent | Pas de cache | Vérifier les `--cache-from` |

---

## Contact

En cas de blocage :

1. Utilise Claude Code pour analyser les erreurs
2. Consulte les logs Kubernetes
3. Demande de l'aide si bloqué plus de 2h sur le même problème

Bonne migration !
