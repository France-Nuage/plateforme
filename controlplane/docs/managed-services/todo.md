# Services Manages - TODO

## Contexte

France Nuage propose des services manages (Vaultwarden, Nextcloud, n8n, Metabase, etc.) deployes via Helm sur un cluster K8s multi-tenant partage (isolation par namespace via Capsule). Les charts Helm existent dans un repo Git separe (`~/Downloads/charts/charts/`), et le CI GitLab publie deja en OCI sur le GitLab Container Registry.

Le but : permettre aux clients de deployer des instances de services manages depuis la console, avec versioning, montee de version, et une source de verite unique entre le registry OCI et Postgres.

## Decisions prises

### Architecture

- **Source de verite unique** : le pipeline CI/CD est la frontiere de transaction atomique. Il pousse d'abord le chart dans le registry OCI, puis enregistre la version via l'API du control plane. Si l'enregistrement echoue, la version existe dans le registry mais n'est pas visible aux clients.
- **Registry OCI** : GitLab Container Registry (deja en place, le CI publie deja avec `helm push`).
- **Pas de duplication** : la DB stocke des metadonnees (nom, version, reference OCI, schema de config), pas le contenu du chart. Le registry stocke les artefacts.

### Modele de donnees

- **3 tables** dans un schema `managed` : `managed_service`, `managed_service_version`, `managed_service_instance`.
- **`deactivated_at TIMESTAMPTZ?`** au lieu de `is_active BOOLEAN` (null = actif, set = date de desactivation). Sur `managed_service` et `managed_service_version`.
- **Enums PostgreSQL** : `managed_service_category` (security, collaboration, analytics, database, automation, cms, erp, storage, dashboard), `managed_database_engine` (cnpg, mariadb).
- **Contraintes CHECK** sur : slug (regex kebab-case), chart_version (semver), oci_reference (prefix oci://), namespace (max 253, regex K8s), release_name (max 53, regex Helm).
- **FSM** via `lib_fsm` existante pour le statut des instances : provisioning, running, upgrading, failed, deleting, deleted.
- **Reference vers `lib_fsm.state_machine`** sur `managed_service_instance.status` (meme pattern que `workflow.execution.status`).

### Tiers -- supprimes

Decision 2026-05-18 : les tiers (Starter/Pro/Enterprise) sont **retires** du modele. Le merge des values cote control plane ne combine plus que `user_values` (client) + `platform_values` (genere). Les ressources CPU/RAM/stockage seront soit codees en dur dans les charts, soit exposees comme champs `user_values` standard.

### Secrets

- **Les secrets ne transitent jamais par Postgres.** Option B retenue : K8s Secrets dans le namespace de l'instance.
- Le `configurable_values_schema` de chaque version annote les champs secrets avec `"format": "password"`.
- L'API recoit le payload complet du client, separe config/secrets selon le schema, stocke la config dans `user_values` JSONB, et cree/met a jour un K8s Secret `{release_name}-secrets` dans le namespace.
- Secrets concernes : `smtp.password`, `oidc.client_secret`, `s3.access_key`, `s3.secret_key`.

### Configuration - 3 couches de values au deploiement

1. **user_values** (client, DB) : domain, smtp.host, smtp.port, timezone, parametres applicatifs, et eventuellement CPU/RAM/stockage si exposes par le chart.
2. **secret_values** (client, K8s Secret) : smtp.password, oidc.client_secret, cles S3.
3. **platform_values** (genere, non-overridable) : ingress.host, namespace, backup S3 bucket (`frn-cnpg-backup-managed-{org}-{service}-{env}`), storageClass, anti-affinite zone, securityContext.

Priorite de merge : platform > secrets > user.

### Workflow engine

- Le deploiement passe par le systeme de workflow/FSM existant (`workflow.execution`).
- 3 workflows : `DeployManagedService`, `UpgradeManagedService`, `DeleteManagedService`. (`ScaleManagedService` retire avec les tiers.)
- Nouvelles operations :
  - `CreateNamespaceOp`, `DeleteNamespaceOp`
  - `CreateK8sSecretOp`, `UpdateK8sSecretOp`, `DeleteK8sSecretOp`
  - `HelmInstallOp`, `HelmUpgradeOp`, `HelmUninstallOp` (shell out vers CLI helm)
  - `UpdateInstanceStatusOp`, `UpdateInstanceVersionOp`
- Le worker shell out vers le CLI helm (pas de crate Rust mature).

### API

- Catalogue (public) : `GET /api/v1/managed-services`, `GET /api/v1/managed-services/{slug}`, `GET /api/v1/managed-services/{slug}/versions`.
- Administration (CI) : `POST /api/v1/managed-services/{slug}/versions` (appele par le CI apres publish OCI).
- Instances (par projet) : `POST /api/v1/projects/{project}/instances`, `GET .../instances`, `GET .../instances/{id}`, `POST .../instances/{id}/upgrade`, `DELETE .../instances/{id}`.

### CI

- Le CI existant (`~/Downloads/charts/.gitlab-ci.yml`) fait deja : lint-and-test, version-check, yaml-lint, chart-schema, kubesec, publish-oci.
- A ajouter : stage `register` qui appelle `POST /api/v1/managed-services/{slug}/versions` avec chart_version, app_version, oci_reference, configurable_values_schema, ui_schema.
- A ajouter dans chaque chart candidat :
  - annotation `france-nuage.fr/managed: "true"` dans `Chart.yaml` (pour selectionner les charts managed dans le stage register).
  - `values.schema.json` (JSON Schema des values configurables par le client, validation).
  - `values.ui-schema.json` (rjsf UI Schema, rendu console). Optionnel : si absent, la console rend avec rjsf en mode auto.

### UI dynamique (formulaire de deploiement)

- Stack retenue : `@rjsf/core` + `@rjsf/chakra-ui` + `@rjsf/utils` + `@rjsf/validator-ajv8` (la console est deja sur Chakra UI v3).
- Pattern inspire de Netir (`~/Documents/Dev/netir/platform/netir-front-v2/app/components/custom/forms/`) mais sans i18n (la console n'a pas de couche i18n actuellement) et sans les widgets metiers Netir.
- Format livre par chaque chart : `schema` (JSON Schema) + `uiSchema` (rjsf UI Schema), exactement comme la sortie Netir.
- Stockage cote control plane : nouveau champ `ui_schema JSONB` sur `managed.service_version`.
- Convention UI a documenter dans `~/Downloads/charts/CLAUDE.md` :
  - widgets autorises : `text`, `password`, `textarea`, `checkbox`, `updown`, `select`, `radio`, `email`, `uri`, `hostname`.
  - `ui:title` + `ui:description` obligatoires sur les champs publics.
  - `ui:order` racine obligatoire.
  - Sections optionnelles : `ui:options.expandable: true, collapsed: true`.
- Etape 0 a valider : compatibilite `@rjsf/chakra-ui` avec Chakra UI v3 (l'ecosysteme rjsf cible historiquement v2). Si KO, fork local dans `console/src/components/forms/rjsf-chakra/` adapte a l'API v3.

### Auth CI (service account)

- Nouvelle table `iam.service_account` (id, name, hashed_token Argon2id, scopes JSONB, created_at, last_used_at, revoked_at).
- Scope minimal : `managed.versions:write`.
- Format token : `frn_sa_<random64>`. Imprime une seule fois a la creation.
- Interceptor gRPC : si Authorization Bearer commence par `frn_sa_`, bypass OIDC et charge le service account. Sinon flux OIDC normal.
- Provisioning CLI : sous-commande `frn-server service-account create --name charts-ci --scopes managed.versions:write`.

## Catalogue de services

14 services manages identifies :

| Service | Categorie | DB | Complexite |
|---------|-----------|-----|-----------|
| Vaultwarden | security | CNPG 3r | faible |
| Nextcloud | collaboration | CNPG 3r + Redis | haute |
| n8n | automation | CNPG 2r + Redis | moyenne |
| Metabase | analytics | CNPG 2r | moyenne |
| Matrix (Synapse) | collaboration | CNPG 2r + Redis | haute |
| Odoo | erp | CNPG 2r + Redis | haute |
| OnlyOffice | collaboration | CNPG 2r + Redis | moyenne |
| Directus | cms | CNPG + Redis | moyenne |
| DocuSeal | security | CNPG 2r + Redis | faible |
| Homarr | dashboard | CNPG 2r | faible |
| Matomo | analytics | MariaDB Operator | moyenne |
| SFTPgo | storage | CNPG 3r + HAProxy | moyenne |
| Suite Numerique | collaboration | CNPG + Redis + MinIO + OpenSearch | tres haute |
| PostgreSQL | database | CNPG direct | faible |

3 charts exclus (infra interne) : Headscale, Hoop, Pangolin.

Patterns communs : 13/14 utilisent CNPG, 8/14 utilisent Redis, tous les ingress desactives par defaut, anti-affinite zone partout, convention backup S3 `frn-cnpg-backup-managed-{org}-{app}-{env}`.

## Taches

### Phase 1 - Fondations DB + API catalogue

- [x] Migration : types enum (`managed_service_category`, `managed_database_engine`)
- [x] Migration : table `managed_service`
- [x] Migration : table `managed_service_version`
- [x] Migration : table `managed_service_instance` + FSM `managed_service_instance_status` dans lib_fsm (etats : provisioning, running, upgrading, failed, deleting, deleted)
- [x] Entites Rust : `ManagedService`, `ManagedServiceVersion`, `ManagedServiceInstance`
- [x] Repositories Rust : CRUD pour les 3 tables
- [x] Service Rust : `ManagedServiceCatalogService` (lecture catalogue)
- [x] Routes API catalogue : ListServices, GetService, ListVersions (gRPC)
- [x] Route API enregistrement : RegisterVersion (authentification par service token CI)
- [x] Tests d'integration pour les routes catalogue

### Phase 2 - Workflows + deploiement

- [x] Nouvelles operations : `CreateNamespaceOp`, `DeleteNamespaceOp`
- [x] Nouvelles operations : `CreateK8sSecretOp`, `UpdateK8sSecretOp`, `DeleteK8sSecretOp`
- [x] Nouvelles operations : `HelmInstallOp`, `HelmUpgradeOp`, `HelmUninstallOp`
- [x] Nouvelles operations : `UpdateInstanceStatusOp`, `UpdateInstanceVersionOp`
- [x] Workflow : `DeployManagedServiceWorkflow` (create namespace, create secret, helm install, write relationships, update status)
- [x] Workflow : `UpgradeManagedServiceWorkflow` (update secret, helm upgrade, update version, update status)
- [x] Workflow : `DeleteManagedServiceWorkflow` (helm uninstall, delete secret, delete namespace, cleanup relationships, update status)
- [x] Routes API instances : POST instances, GET instances, GET instances/{id}, POST instances/{id}/upgrade, DELETE instances/{id}
- [x] Logique de merge user + platform values
- [x] Tests d'integration pour les routes instances

### Phase 3 - Console UI

- [x] Page catalogue (liste des services avec categories, recherche, filtres)
- [x] Page detail service (description, versions)
- [x] Formulaire de deploiement (formulaire dynamique genere depuis `values_schema`)
- [x] Page gestion instances (liste, statut, actions upgrade/delete avec polling)
- [x] Page detail instance (statut FSM, config, upgrade, delete)

### Phase 3.5 - Retrait des tiers (2026-05-18)

Decision : on ne maintient pas d'offres tiers (Starter/Pro/Enterprise). Code supprime :

- DB : table `managed.service_tier`, enum `managed_tier_slug`, colonne `tier_id` sur `service_instance` (migration `20260513` editee en place).
- Backend : entites `ManagedServiceTier` / `ManagedTierSlug`, helpers `list_tiers_by_service*`, `find_tier_by_id`, `update_instance_tier`, variant d'erreur `TierNotFound`, workflow `ScaleManagedService`, operation `UpdateInstanceTierOp`. `merge_helm_values` simplifie a (user, platform).
- Proto + RPC : `ManagedServiceTierProto`, champ `tiers` sur `ManagedServiceProto`, `tier_id` sur `CreateInstanceRequest` et `ManagedServiceInstanceProto`. Numerotation des champs reorganisee.
- Seed : section `tiers:` dans `seed/managed/*.yaml`, struct `TierSeed`, `upsert_tier`, champ `tiers_upserted` dans `SeedReport`.
- Tests : helpers `seed_managed_service_tier`, parametre `tier_id` de `seed_managed_service_instance`, test `test_get_managed_service_includes_tiers`.
- Console : composant `ManagedServiceTierCard`, selecteur tier dans `managed-service-deploy.page.tsx`, helpers `extractTierSpecs` / `formatMonthlyPrice` / `ManagedServiceSpec` / `SPEC_LABELS`, fixture `managedServiceTier`.
- SDK : type `ManagedServiceTier`, champ `tiers` sur `ManagedService`, champ `tierId` sur `ManagedServiceInstance` et `CreateManagedInstanceInput`.

### Phase 4 - CI + charts + UI dynamique

#### 4.1 - Auth CI (token env)

Decision 2026-05-25 : approche simplifiee, pas de table service_account. Un simple token en variable d'environnement (`CI_SERVICE_TOKEN`) compare en constant-time (`subtle::ConstantTimeEq`) sur la RPC `RegisterVersion`.

- [x] Variable d'env `CI_SERVICE_TOKEN` lue dans `Config::from_env()`
- [x] Fonction `authenticate_bearer` avec comparaison constant-time (`frn-rpc/src/auth.rs`)
- [x] Guard `authenticate_ci()` sur `ManagedServicesRpc::register_version`
- [x] Tests integration : token valide ok, token absent ko, mauvais token ko, duplicate ko, service inconnu ko, JSON invalide ko, OCI invalide ko

#### 4.2 - Schema rjsf cote control plane

- [x] Migration : ajouter `ui_schema JSONB` (nullable) sur `managed.service_version` (`20260515120000_add_ui_schema_to_managed_service_version.sql`)
- [x] Proto : etendre `RegisterVersionRequest` avec `optional string ui_schema = 6` + champ 8 dans `ManagedServiceVersionProto`
- [x] Adapter `register_version` (core) pour persister `ui_schema`
- [x] Adapter `find_version` / `list_versions` pour exposer `ui_schema` dans les reponses publiques (via mapping `From<&ManagedServiceVersion>` enrichi)
- [x] Tests integration : register avec/sans ui_schema, rejet JSON ui_schema invalide, round-trip register -> list (3 nouveaux tests). 21/21 tests managed verts.
- [x] **Side fix** : helper `seed_managed_service` patche le bug pre-existant de `fabrique-derive 0.2` ou `#[fabrique(soft_delete)]` ne force pas `None` a la creation. Tous les helpers qui creent une entite avec soft_delete doivent appeler `.deactivated_at(None)` explicitement. A repercuter sur les autres factories du code (utilisateurs, organisations, etc.) si on rencontre le meme symptome ("not found" alors que le seed semble ok).

#### 4.3 - UI dynamique console (rjsf + Chakra)

- [x] Verifier compatibilite `@rjsf/chakra-ui` avec Chakra UI v3 : `@rjsf/chakra-ui@6.5.2` cible bien `@chakra-ui/react >=3.16.1` (la console est sur `^3.24.2`). Pas de fork necessaire.
- [x] Ajouter deps : `@rjsf/core@^6.5.2`, `@rjsf/utils@^6.5.2`, `@rjsf/validator-ajv8@^6.5.2`, `@rjsf/chakra-ui@^6.5.2`, `chakra-react-select@^6.1.3` (peer de chakra-ui)
- [x] SDK : ajouter `uiSchema?: string` au modele `ManagedServiceVersion` + mapping dans `managed-service.rpc.ts`
- [x] Composant `RjsfDeployForm` (`console/src/components/forms/rjsf-deploy-form.tsx`) : consomme `schema` + `uiSchema` optionnel, applique le split user/secret au onChange, no submit button (parent owns submit)
- [x] Helper `split-secrets.ts` (`console/src/components/forms/lib/split-secrets.ts`) : `collectSecretPaths(schema)` + `splitUserAndSecretValues(schema, formData)` -> separe les champs `format: password` dans un bucket secret distinct
- [x] Brancher `RjsfDeployForm` dans `managed-service-deploy.page.tsx` en remplacement de l'ancien `DynamicValuesForm` maison
- [x] Suppression du composant maison `dynamic-values-form.tsx` + helpers orphelins (`formatSchemaFieldLabel`, `groupSchemaFieldsByPrefix`, `getSchemaGroupLabel`, `FORM_GROUP_LABELS`)
- [x] Validation : `pnpm tsc --noEmit` OK, `pnpm lint` OK, `pnpm build` OK
- [ ] Tests visuels sur Vaultwarden + un deuxieme service au schema plus riche (n8n ou Metabase) **bloque sur 4.5/4.6 : pas de version reelle en DB tant que le seed et le CI charts ne sont pas en place. Le rendu peut etre teste via une version factice inseree manuellement.**
- [ ] Optionnel (a la demande) : widgets custom `password-widget` (toggle show/hide), `hostname-widget` (suffixe `.apps.france-nuage.fr`), `cpu-widget`/`memory-widget` (sliders) -- non implementes pour l'instant, le rendu par defaut de rjsf+chakra-ui couvre les cas standards (text, password natif, checkbox, select, updown).
- [ ] Optionnel : hook `use-form-errors.ts` pour mapping erreurs backend -> rjsf ErrorSchema -- pas necessaire tant qu'on n'a pas de retour d'erreur structure cote deploy.
- [ ] Optionnel : helper `form-conditions.ts` (`ui:showIf` / `ui:hideIf`) -- aucun chart n'en a besoin pour l'instant.

#### 4.4 - Stage `register` cote charts

- [ ] Stage `register` dans `~/Downloads/charts/.gitlab-ci.yml` (post `publish-oci`, filtre par annotation `france-nuage.fr/managed: true`)
- [ ] Variables CI masquees + protegees : `CONTROLPLANE_URL`, `CI_SERVICE_TOKEN`
- [ ] Documenter conventions UI Schema dans `~/Downloads/charts/CLAUDE.md` (widgets autorises, regles, exemples)

#### 4.5 - Charts (14 services)

- [ ] Pour chaque chart : annotation `france-nuage.fr/managed: "true"` dans `Chart.yaml`
- [ ] Pour chaque chart : `values.schema.json` (14 fichiers)
- [ ] Pour chaque chart : `values.ui-schema.json` (14 fichiers, format rjsf)
- [ ] Pour chaque chart : adapter les templates pour consommer le K8s Secret `{release}-secrets` plutot que des values en clair
- [ ] Pilote bout en bout : Vaultwarden (le plus simple)

#### 4.6 - Seed catalogue

- [x] Format de seed : un YAML par service dans `controlplane/seed/managed/{slug}.yaml` (section `service` + bloc optionnel `dev_mock_version` reserve au dev local). Section `tiers` retiree avec le retrait des offres.
- [x] Module `frn-core/src/managed/seed.rs` : parse YAML, upsert idempotent (`ON CONFLICT (slug) DO UPDATE` sur `managed.service`, `ON CONFLICT DO NOTHING` sur `managed.service_version` pour le mock)
- [x] Binaire `controlplane/server/src/bin/seed_managed.rs` : `--dir <path>` + `--with-dev-mock` optionnel, lit `DATABASE_URL` en env
- [x] Pilote Vaultwarden : YAML (service + dev_mock_version avec ui_schema rjsf valide). Service + version mock crees, re-run idempotent.
- [ ] Decision a figer plus tard : execution auto au boot du server (Application::run) ou manuelle a la deploy. Pour l'instant manuel.
- [ ] Repercuter sur les 13 autres services apres validation du pilote
- [ ] Tests d'integration du module `seed` (parse YAML KO, upsert deux fois, idempotence dev_mock_version) -- a ecrire si le pattern est valide en production

**Bonus side fix** : `serde_yaml = "0.9"` ajoute en dependance directe de `frn-core` (etait dispo en transitive uniquement). Le crate est marque deprecated mais activement utilise par l'ecosysteme. Migration future possible vers `serde_yaml_ng` si besoin.

**Pour exercer la UI rjsf+Chakra sans le CI charts** :

```
cd controlplane && DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/postgres \
    SQLX_OFFLINE=true cargo run --bin seed_managed -- --dir seed/managed --with-dev-mock
```

Le formulaire de deploiement Vaultwarden affiche maintenant les champs `domain`, `signupsAllowed`, `smtp.*`, `sso.*` avec leurs titres FR, le `password` SMTP et le `clientSecret` SSO etant automatiquement routes en `secretValues`.

## Fichiers de reference

- Diagrammes HTML : `docs/managed-services/er-diagram.html`, `state-machine.html`, `architecture-flow.html`
- Document complet : `docs/managed-services/index.html`
- Charts source : `~/Downloads/charts/charts/` (repo Git separe)
- CI existant : `~/Downloads/charts/.gitlab-ci.yml`
- Conventions charts : `~/Downloads/charts/CLAUDE.md` (CNPG, MariaDB Operator, Redis, anti-affinite, backup S3)
- Workflow engine existant : `workflow/src/` (workflows, operations, FSM, repository)
- Migration FSM existante : `migrations/20260512120000_create_workflow_engine.sql`
