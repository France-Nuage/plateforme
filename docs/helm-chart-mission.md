# Mission : Création du Chart Helm Plateforme

> **Prérequis** : Cette mission fait suite à la
> [migration BuildKit rootless](./buildkit-rootless-mission.md).
> Assure-toi d'avoir complété cette première phase avant de commencer.

## Contexte

La plateforme France Nuage utilise actuellement Docker Compose pour :

- Le **développement local** (`docker-compose.yml`)
- Les **tests CI** (`docker-compose.yml` + `docker-compose.ci.yml`)
- La **production** (`docker-compose.prod.yml`)

L'objectif est de migrer vers **Kubernetes** via un **chart Helm** qui servira à la
fois pour les tests CI et le déploiement en production.

## Objectifs

1. Créer un chart Helm capable de déployer la stack complète
2. Supporter deux modes : **CI** (tests) et **production**
3. Intégrer le chart dans la pipeline GitLab CI existante
4. Permettre le déploiement automatique en production

## Architecture cible

```text
helm/
└── plateforme/
    ├── Chart.yaml
    ├── values.yaml              # Valeurs par défaut
    ├── values-ci.yaml           # Surcharges pour CI
    ├── values-prod.yaml         # Surcharges pour production
    ├── templates/
    │   ├── _helpers.tpl
    │   ├── NOTES.txt
    │   │
    │   │   # Base de données principale
    │   ├── postgres/
    │   │   ├── deployment.yaml
    │   │   ├── service.yaml
    │   │   └── configmap.yaml
    │   │
    │   │   # Autorisation (SpiceDB)
    │   ├── spicedb/
    │   │   ├── deployment.yaml
    │   │   ├── service.yaml
    │   │   └── migrate-job.yaml
    │   │
    │   │   # Services applicatifs
    │   ├── controlplane/
    │   │   ├── deployment.yaml
    │   │   ├── service.yaml
    │   │   └── migrate-job.yaml    # Atlas migrations
    │   │
    │   ├── authz-worker/
    │   │   └── deployment.yaml
    │   │
    │   ├── synchronizer/
    │   │   └── deployment.yaml
    │   │
    │   │   # CI uniquement
    │   ├── keycloak/
    │   │   ├── deployment.yaml
    │   │   ├── service.yaml
    │   │   └── realm-configmap.yaml
    │   │
    │   └── system-tests/
    │       └── job.yaml            # Job de tests
    │
    └── files/
        ├── keycloak-realm.json     # Import realm Keycloak
        └── spicedb-schema.zed      # Schema d'autorisation
```

## Services à implémenter

### Mode Production (`values-prod.yaml`)

Basé sur `docker-compose.prod.yml` :

| Service        | Type       | Dépendances              |
|----------------|------------|--------------------------|
| postgres       | Deployment | -                        |
| spicedb        | Deployment | postgres                 |
| controlplane   | Deployment | postgres, spicedb        |
| authz-worker   | Deployment | postgres, spicedb        |
| synchronizer   | Deployment | postgres, spicedb        |

### Mode CI (`values-ci.yaml`)

Basé sur `docker-compose.yml` + `docker-compose.ci.yml` :

| Service        | Type       | Dépendances                           |
|----------------|------------|---------------------------------------|
| postgres       | Deployment | -                                     |
| keycloak-db    | Deployment | -                                     |
| keycloak       | Deployment | keycloak-db                           |
| spicedb-db     | Deployment | -                                     |
| spicedb        | Deployment | spicedb-db                            |
| controlplane   | Deployment | postgres, spicedb, keycloak           |
| authz-worker   | Deployment | postgres, spicedb                     |
| synchronizer   | Deployment | postgres, spicedb, keycloak           |
| console        | Deployment | controlplane, keycloak                |
| system-tests   | Job        | controlplane, console (tous services) |

## Correspondance Docker Compose → Kubernetes

### Concepts clés

| Docker Compose          | Kubernetes                              |
|-------------------------|-----------------------------------------|
| `services:`             | `Deployment` + `Service`                |
| `depends_on`            | `initContainers` ou Jobs avec `wait-for`|
| `healthcheck`           | `livenessProbe` / `readinessProbe`      |
| `environment`           | `env` ou `ConfigMap` / `Secret`         |
| `volumes` (data)        | `emptyDir` (CI) ou `PVC` (prod)         |
| `volumes` (config)      | `ConfigMap`                             |
| `expose` / `ports`      | `Service` (ClusterIP)                   |
| `profiles: [donotstart]`| Ressource conditionnelle (`if`)         |

### Exemple de conversion

**Docker Compose** (`docker-compose.prod.yml`):

```yaml
controlplane:
  image: registry.gitlab.com/.../controlplane:release
  depends_on:
    postgres:
      condition: service_healthy
    spicedb:
      condition: service_healthy
  environment:
    DATABASE_URL: postgresql://...
    SPICEDB_URL: http://spicedb:50051
  healthcheck:
    test: ["CMD", "/bin/grpc_health_probe", "-addr=localhost:80"]
    interval: 5s
    timeout: 5s
    retries: 24
  ports:
    - "80:80"
```

**Kubernetes** (`templates/controlplane/deployment.yaml`):

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "plateforme.fullname" . }}-controlplane
  labels:
    {{- include "plateforme.labels" . | nindent 4 }}
    app.kubernetes.io/component: controlplane
spec:
  replicas: 1
  selector:
    matchLabels:
      {{- include "plateforme.selectorLabels" . | nindent 6 }}
      app.kubernetes.io/component: controlplane
  template:
    metadata:
      labels:
        {{- include "plateforme.selectorLabels" . | nindent 8 }}
        app.kubernetes.io/component: controlplane
    spec:
      # Attendre que les dépendances soient prêtes
      initContainers:
        - name: wait-for-postgres
          image: busybox:1.36
          command: ['sh', '-c', 'until nc -z {{ include "plateforme.fullname" . }}-postgres 5432; do sleep 2; done']
        - name: wait-for-spicedb
          image: busybox:1.36
          command: ['sh', '-c', 'until nc -z {{ include "plateforme.fullname" . }}-spicedb 50051; do sleep 2; done']
      containers:
        - name: controlplane
          image: "{{ .Values.controlplane.image.repository }}:{{ .Values.controlplane.image.tag }}"
          ports:
            - name: grpc
              containerPort: 80
              protocol: TCP
          env:
            - name: CONTROLPLANE_ADDR
              value: "0.0.0.0:80"
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: {{ include "plateforme.fullname" . }}-secrets
                  key: database-url
            - name: SPICEDB_URL
              value: "http://{{ include "plateforme.fullname" . }}-spicedb:50051"
            - name: SPICEDB_GRPC_PRESHARED_KEY
              valueFrom:
                secretKeyRef:
                  name: {{ include "plateforme.fullname" . }}-secrets
                  key: spicedb-preshared-key
          # Conversion du healthcheck
          livenessProbe:
            exec:
              command: ["/bin/grpc_health_probe", "-addr=localhost:80"]
            initialDelaySeconds: 10
            periodSeconds: 5
            timeoutSeconds: 5
            failureThreshold: 24
          readinessProbe:
            exec:
              command: ["/bin/grpc_health_probe", "-addr=localhost:80"]
            initialDelaySeconds: 5
            periodSeconds: 5
            timeoutSeconds: 5
            failureThreshold: 3
```

## Gestion des migrations et du seed

### Migrations PostgreSQL (Atlas)

Utiliser un **Job** Kubernetes qui s'exécute avant le déploiement :

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: {{ include "plateforme.fullname" . }}-migrate
  annotations:
    "helm.sh/hook": pre-install,pre-upgrade
    "helm.sh/hook-weight": "1"
    "helm.sh/hook-delete-policy": hook-succeeded
spec:
  template:
    spec:
      restartPolicy: Never
      initContainers:
        - name: wait-for-postgres
          image: busybox:1.36
          command: ['sh', '-c', 'until nc -z {{ include "plateforme.fullname" . }}-postgres 5432; do sleep 2; done']
      containers:
        - name: migrate
          image: arigaio/atlas:latest
          command:
            - atlas
            - schema
            - apply
            - --url=$(DATABASE_URL)?sslmode=disable
            - --to=file:///migrations
            - --auto-approve
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: {{ include "plateforme.fullname" . }}-secrets
                  key: database-url
          volumeMounts:
            - name: migrations
              mountPath: /migrations
      volumes:
        - name: migrations
          configMap:
            name: {{ include "plateforme.fullname" . }}-migrations
```

### Schema SpiceDB (Zed)

Même principe avec un Job pour appliquer le schema :

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: {{ include "plateforme.fullname" . }}-spicedb-schema
  annotations:
    "helm.sh/hook": post-install,post-upgrade
    "helm.sh/hook-weight": "2"
    "helm.sh/hook-delete-policy": hook-succeeded
spec:
  template:
    spec:
      restartPolicy: Never
      containers:
        - name: zed
          image: authzed/zed:latest
          command:
            - zed
            - schema
            - write
            - /schema/schema.zed
          env:
            - name: ZED_ENDPOINT
              value: "{{ include "plateforme.fullname" . }}-spicedb:50051"
            - name: ZED_INSECURE
              value: "true"
            - name: ZED_TOKEN
              valueFrom:
                secretKeyRef:
                  name: {{ include "plateforme.fullname" . }}-secrets
                  key: spicedb-preshared-key
          volumeMounts:
            - name: schema
              mountPath: /schema
      volumes:
        - name: schema
          configMap:
            name: {{ include "plateforme.fullname" . }}-spicedb-schema
```

### Realm Keycloak (CI uniquement)

Keycloak peut importer un realm au démarrage via la variable `KC_SPI_IMPORT_REALM`
ou la commande `start-dev --import-realm` avec un volume monté.

## Structure des Values

### `values.yaml` (défaut)

```yaml
# Configuration globale
global:
  imageRegistry: registry.gitlab.com/getbunker-france-nuage/france-nuage/plateforme
  imageTag: "latest"

# PostgreSQL principal
postgres:
  enabled: true
  image:
    repository: postgres
    tag: "16-alpine"
  auth:
    database: postgres
    username: postgres
    # Le mot de passe sera dans un Secret
  persistence:
    enabled: false  # emptyDir par défaut

# SpiceDB
spicedb:
  enabled: true
  image:
    repository: authzed/spicedb
    tag: "latest"

# Controlplane
controlplane:
  enabled: true
  image:
    repository: ""  # Utilise global.imageRegistry/controlplane
    tag: ""         # Utilise global.imageTag
  config:
    consoleUrl: "https://console.france-nuage.fr"
    oidcUrl: ""

# Authz Worker
authzWorker:
  enabled: true
  image:
    repository: ""
    tag: ""

# Synchronizer
synchronizer:
  enabled: true
  image:
    repository: ""
    tag: ""

# Console (désactivé par défaut, activé en CI)
console:
  enabled: false
  image:
    repository: ""
    tag: ""

# Keycloak (désactivé par défaut, activé en CI)
keycloak:
  enabled: false
  image:
    repository: quay.io/keycloak/keycloak
    tag: "26.0"

# Tests système (désactivé par défaut)
systemTests:
  enabled: false
  image:
    repository: ""
    tag: ""
```

### `values-ci.yaml`

```yaml
global:
  imageTag: "${CI_COMMIT_REF_SLUG}-development"

# Activer Keycloak pour les tests
keycloak:
  enabled: true

# Activer la console
console:
  enabled: true

# Activer les tests
systemTests:
  enabled: true

# Configuration CI spécifique
controlplane:
  config:
    consoleUrl: "http://{{ .Release.Name }}-console"
    oidcUrl: "http://{{ .Release.Name }}-keycloak:8080/realms/francenuage/.well-known/openid-configuration"
```

### `values-prod.yaml`

```yaml
global:
  imageTag: "release"

postgres:
  persistence:
    enabled: true
    size: 10Gi

controlplane:
  config:
    consoleUrl: "https://console.france-nuage.fr"
    oidcUrl: "https://auth.france-nuage.fr/realms/france-nuage/.well-known/openid-configuration"
```

## Intégration GitLab CI

### Modification de `.gitlab-ci.yml`

Ajouter un nouveau stage et des jobs pour Helm :

```yaml
stages:
  - security
  - build
  - code-style
  - unit-tests
  - helm-deploy      # Nouveau stage
  - system-tests
  - documentation
  - release
  - deploy

# Template pour les jobs Helm
.helm_template:
  image: alpine/helm:3.14
  before_script:
    - apk add --no-cache kubectl
    - kubectl config use-context $KUBE_CONTEXT

# Déploiement pour les tests CI
"helm / deploy-ci":
  extends: .helm_template
  stage: helm-deploy
  variables:
    NAMESPACE: "ci-${CI_PIPELINE_ID}"
  script:
    # Créer le namespace
    - kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -

    # Déployer le chart
    - |
      helm upgrade --install plateforme-ci ./helm/plateforme \
        --namespace $NAMESPACE \
        --values ./helm/plateforme/values-ci.yaml \
        --set global.imageTag="${CI_COMMIT_REF_SLUG}-development" \
        --set global.imageRegistry="${CI_REGISTRY_IMAGE}" \
        --wait \
        --timeout 10m
  environment:
    name: ci/$CI_COMMIT_REF_SLUG
    on_stop: "helm / cleanup-ci"
    auto_stop_in: 1 hour

# Nettoyage après les tests
"helm / cleanup-ci":
  extends: .helm_template
  stage: helm-deploy
  variables:
    NAMESPACE: "ci-${CI_PIPELINE_ID}"
  script:
    - helm uninstall plateforme-ci --namespace $NAMESPACE || true
    - kubectl delete namespace $NAMESPACE --ignore-not-found
  when: always
  environment:
    name: ci/$CI_COMMIT_REF_SLUG
    action: stop

# Tests système sur le déploiement Helm
"system tests / helm":
  extends: .helm_template
  stage: system-tests
  needs:
    - "helm / deploy-ci"
  variables:
    NAMESPACE: "ci-${CI_PIPELINE_ID}"
  script:
    # Exécuter les tests via un Job Kubernetes
    - |
      kubectl create job system-tests-${CI_PIPELINE_ID} \
        --namespace $NAMESPACE \
        --image=${CI_REGISTRY_IMAGE}/system-tests:${CI_COMMIT_REF_SLUG} \
        -- npm run test:ci

    # Attendre la fin du job et récupérer les logs
    - kubectl wait --for=condition=complete job/system-tests-${CI_PIPELINE_ID} --namespace $NAMESPACE --timeout=30m
    - kubectl logs job/system-tests-${CI_PIPELINE_ID} --namespace $NAMESPACE
  artifacts:
    when: always
    paths:
      - system-tests/playwright-report/

# Déploiement production (automatique sur master)
"helm / deploy-prod":
  extends: .helm_template
  stage: deploy
  script:
    - |
      helm upgrade --install plateforme ./helm/plateforme \
        --namespace production \
        --values ./helm/plateforme/values-prod.yaml \
        --set global.imageTag="release" \
        --wait \
        --timeout 10m
  environment:
    name: production
    url: https://console.france-nuage.fr
  rules:
    - if: '$CI_COMMIT_BRANCH == "master"'
      when: on_success
```

## Gestion des domaines internes

En CI, les services communiquent via les **noms de Service Kubernetes** :

| Domaine Docker Compose | Service Kubernetes              |
|------------------------|---------------------------------|
| `keycloak.test`        | `{{ .Release.Name }}-keycloak`  |
| `console.test`         | `{{ .Release.Name }}-console`   |
| `controlplane.test`    | `{{ .Release.Name }}-controlplane` |

Les URLs doivent être configurées dynamiquement via les values ou les templates.

## Étapes de réalisation

### Phase 1 : Structure de base

1. Créer la structure de dossiers `helm/plateforme/`
2. Initialiser `Chart.yaml` et `values.yaml`
3. Implémenter les helpers dans `_helpers.tpl`

### Phase 2 : Services d'infrastructure

4. Implémenter PostgreSQL (Deployment + Service)
5. Implémenter SpiceDB (Deployment + Service + migrate Job)
6. Tester le déploiement minimal : `helm install test ./helm/plateforme --dry-run`

### Phase 3 : Services applicatifs

7. Implémenter le Job de migration Atlas
8. Implémenter Controlplane
9. Implémenter Authz-worker
10. Implémenter Synchronizer

### Phase 4 : Services CI

11. Implémenter Keycloak (conditionnel)
12. Implémenter Console (conditionnel)
13. Implémenter le Job system-tests

### Phase 5 : Intégration CI

14. Modifier `.gitlab-ci.yml` pour utiliser Helm
15. Tester le pipeline complet
16. Configurer le déploiement production

## Ressources utiles

### Documentation officielle

- [Helm Documentation](https://helm.sh/docs/)
- [Kubernetes Documentation](https://kubernetes.io/docs/home/)
- [Chart Development Guide](https://helm.sh/docs/chart_template_guide/)

### Exemples de charts

- [Bitnami PostgreSQL](https://github.com/bitnami/charts/tree/main/bitnami/postgresql)
  (référence pour la structure)
- [Charts Helm officiels](https://github.com/helm/charts) (exemples variés)

### Commandes utiles

```bash
# Créer un nouveau chart
helm create helm/plateforme

# Valider la syntaxe
helm lint ./helm/plateforme

# Prévisualiser le rendu
helm template test ./helm/plateforme --values ./helm/plateforme/values-ci.yaml

# Installer en mode debug
helm install test ./helm/plateforme --dry-run --debug

# Déployer localement (avec minikube/kind)
helm install plateforme ./helm/plateforme --namespace test --create-namespace

# Voir les ressources déployées
helm get manifest plateforme

# Mettre à jour
helm upgrade plateforme ./helm/plateforme

# Désinstaller
helm uninstall plateforme
```

## Points d'attention

### Sécurité

- Ne **jamais** commiter de secrets en clair dans les values
- Utiliser des `Secret` Kubernetes pour les mots de passe
- En CI, utiliser les variables GitLab CI/CD

### Performance CI

- Les Jobs de migration doivent utiliser `helm.sh/hook-delete-policy: hook-succeeded`
  pour ne pas bloquer les pipelines suivantes
- Utiliser `--wait` avec un timeout raisonnable
- Nettoyer les namespaces après chaque pipeline

### Debugging

- Utiliser `kubectl logs` pour voir les logs des pods
- Utiliser `kubectl describe pod` pour diagnostiquer les problèmes de démarrage
- Utiliser `helm get values` pour voir la configuration appliquée

## Livrables attendus

1. **Chart Helm fonctionnel** dans `helm/plateforme/`
2. **Documentation** du chart (README.md dans le dossier helm)
3. **Pipeline CI modifiée** avec les jobs Helm
4. **Tests validés** via le nouveau pipeline

## Contact

En cas de questions ou blocages, utilise Claude Code pour t'aider à :

- Comprendre les concepts Kubernetes/Helm
- Débugger les templates
- Convertir les configurations Docker Compose

N'hésite pas à poser des questions !
