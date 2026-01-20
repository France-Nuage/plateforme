# Migration BuildKit - Architecture et Guide

## Vue d'ensemble

Ce projet a migré de Docker-in-Docker vers BuildKit sur Kubernetes pour améliorer la sécurité, les performances et la stabilité des builds CI/CD.

## Architecture BuildKit Kubernetes Driver

### Avant: Docker-in-Docker (DinD)

```
┌─────────────────────────────────────┐
│ GitLab Runner Pod                   │
│                                     │
│  ┌──────────────────────────────┐   │
│  │ Job Container                │   │
│  │  docker build ...            │   │
│  └──────────┬───────────────────┘   │
│             │                       │
│  ┌──────────▼───────────────────┐   │
│  │ DinD Sidecar (privileged)    │   │
│  │  Docker Daemon               │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘

Problèmes:
✗ Mode privileged requis 
✗ Pas d'isolation entre jobs
✗ Performances limitées
```

### Après: BuildKit sur Kubernetes

```
┌─────────────────────────────────────┐
│ GitLab Runner Pod                   │
│                                     │
│  ┌──────────────────────────────┐   │
│  │ Job Container                │   │
│  │  docker buildx build ...     │   │
│  └──────────┬───────────────────┘   │
│             │                       │
│             │ kubectl/API call      │
│             ▼                       │
└─────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────┐
│ Namespace: frn-gitlab-runner        │
│                                     │
│  ┌──────────────────────────────┐   │
│  │ BuildKit Pod (rootless)      │   │
│  │  - Job ID: 126893226970      │   │
│  │  - Cache: Registry           │   │
│  │  - Auto-cleanup              │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘

Avantages:
✓ Rootless 
✓ Isolation par job
✓ Meilleure performance
```
## Comment ajouter un nouveau job de build

### Étape 1: Créer votre Dockerfile

```dockerfile
# Exemple: my-service/Dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:20-alpine
COPY --from=builder /app/dist /app
CMD ["node", "/app/server.js"]
```

### Étape 2: Ajouter le job dans `.gitlab-ci.yml`

```yaml
"build / my-service":
  extends: .buildkit_template        
  stage: build
  script:
    - |
      docker buildx build \
        --cache-from type=registry,ref=${CI_REGISTRY_IMAGE}/my-service:buildcache \
        --cache-to type=registry,ref=${CI_REGISTRY_IMAGE}/my-service:buildcache,mode=max \
        --tag ${CI_REGISTRY_IMAGE}/my-service:${CI_COMMIT_REF_SLUG} \
        --push \
        --file my-service/Dockerfile \
        my-service
  rules:
    - changes:
        - my-service/**/*
        - .gitlab-ci.yml
```

## Débugger les problèmes de build

### 1. Vérifier les pods BuildKit

```bash
# Lister tous les pods BuildKit
kubectl get pods -n frn-gitlab-runner | grep k8s-builder

# Détails d'un pod spécifique
kubectl describe pod k8s-builder-126893226970-xxx -n frn-gitlab-runner

# Logs d'un pod BuildKit
kubectl logs k8s-builder-126893226970-xxx -n frn-gitlab-runner

#verifiez le pipeline gitlab
```

## Ressources supplémentaires

- [BuildKit Documentation](https://github.com/moby/buildkit)
- [Docker Buildx Kubernetes Driver](https://docs.docker.com/build/drivers/kubernetes/)
- [GitLab Runner Kubernetes Executor](https://docs.gitlab.com/runner/executors/kubernetes/)

