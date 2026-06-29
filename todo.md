# TODO - Alignement specs Kubernetes / schéma cible

## 1. Modèle de données cluster -- FAIT

Décision : pas de nouvelle colonne, `kubernetes_version` est conservée avec validation/normalisation semver.

- [x] Normalisation semver dans le service (`normalize_kubernetes_version` dans `frn-core/src/kubernetes.rs`) : prefixe `v` retiré, valeur non parseable -> NULL
- [x] Contrainte CHECK semver intégrée à la migration de création `20260529120000_create_kubernetes_clusters.sql` (pas encore en prod, pas de migration séparée) + atlas.sum régénéré
- [x] Renommage `last_seen_at` -> `last_health_check_at` : la base utilise déjà `last_health_check_at`, c'est le diagramme qui doit être aligné. Aucune migration nécessaire.
- [x] Tests : normalisation à la création (`v1.31.4+k3s1` -> `1.31.4+k3s1`), valeur non-semver -> NULL, normalisation à l'update
- [x] SPEC-001 mise à jour (flux de création + stockage en base)

## 2. Labels et ciblage des clusters -- FAIT

Décisions : `projects.cluster_id` supprimé complètement (migration 20260530 réécrite, pas en prod) au profit de `cluster_id` NOT NULL sur l'instance ; `deploy_target` NULL/vide = erreur explicite `MissingDeployTarget` (pas de fallback) ; labels `system` gérés uniquement par le control plane (l'API refuse create/delete/attach/detach, même pour les admins).

### Modèle de données

- [x] Migration `20260610120000_create_kubernetes_cluster_labels.sql` : `kubernetes.label` (CITEXT key/value, `system` BOOLEAN, contraintes `length < 50` + charset `[a-zA-Z0-9-]`, UNIQUE(key, value)) + `kubernetes.cluster_label` (jointure, ON DELETE CASCADE) + extension citext
- [x] Entités `KubernetesLabel` et `KubernetesClusterLabel` + service `KubernetesLabels` (`frn-core/src/kubernetes/label.rs`) : CRUD admin-gated, attach/detach idempotents, erreur `SystemLabelReadOnly`
- [x] `deploy_target` JSONB sur `managed.service` (intégré à la migration 20260513, CHECK objet JSON) + entité + validation (`parse_deploy_target`)
- [x] API gRPC : ListLabels/CreateLabel/DeleteLabel/AttachClusterLabel/DetachClusterLabel + `labels` dans `KubernetesClusterProto` (hydratation en une requête, pas de N+1)

### Sélection de cluster au déploiement

- [x] `pick_random_healthy_cluster` remplacé par `pick_healthy_cluster_matching` (cluster healthy portant TOUS les labels du deploy_target, comparaison CITEXT insensible à la casse via cast `::citext`)
- [x] Résolution déplacée de la création du projet vers `create_instance` : `projects.cluster_id` supprimé (entité, proto, RPC, `Error::NoClusterAvailable`), `cluster_id` NOT NULL porté par `managed.service_instance` (+ vue), upgrade/delete utilisent `instance.cluster_id`
- [x] `delete_cluster` refuse si des instances sont hébergées (`ClusterHasInstances` remplace `ClusterHasProjects`)
- [x] Aucun candidat -> `NoClusterMatchingDeployTarget` ; deploy_target absent/vide -> `MissingDeployTarget` ; plusieurs candidats -> choix aléatoire

### Tests et docs

- [x] Tests : 8 tests labels (CRUD, casse CITEXT, longueur/charset, system read-only, attach/detach idempotents, cascade) + 6 tests matching (tous les labels, candidat manquant, aucun cluster, unhealthy ignoré, multi-candidats, insensibilité à la casse) + create_instance (no match, deploy_target manquant, propagation au workflow) ; suite server complète verte
- [x] SPEC-001 (labels + sélection + suppression), SPEC-002 (deploy_target + cluster_id instance), SPEC-004 (prérequis + résolution), SPEC-005/006 (cluster_id de l'instance), SPEC-009 (deploy_target dans le seed)
- [x] Seed : `deploy_target: {availability: ft}` sur les 4 services YAML (les clusters étant enregistrés via l'API, leurs labels s'attachent via AttachClusterLabel ; pas de cluster seedé en SQL)

## 3. IAM -- FAIT

Décision : slug CITEXT comme PRIMARY KEY pour organizations et projects (remplacement des UUID). Migration complète : toutes les FK, SpiceDB, protos, entités, services, RPC handlers. Charset `[a-zA-Z-]`, length < 50, immutables (pas d'endpoint update). Le derive macro `Resource` supporte maintenant `#[resource(id)]` pour designer n'importe quel champ comme identifiant.

### Migration et modèle de données

- [x] Migration `20260629120000_iam_slug_primary_keys.sql` : extension citext, conversion slug TEXT -> CITEXT PK sur organizations, ajout slug CITEXT PK sur projects, migration de toutes les FK (hypervisors, instances, zero_trust_networks, organization_user, organization_service_account, invitations, managed.service_instance, organizations.parent_slug), recreation des indexes et de la vue managed.service_instance_view
- [x] Contraintes CHECK : `length(slug) <= 49`, `slug ~ '^[a-zA-Z]([a-zA-Z-]*[a-zA-Z])?$'` sur organizations et projects
- [x] Extension citext activee (CREATE EXTENSION IF NOT EXISTS citext)
- [x] Slug comme PRIMARY KEY : organizations.slug et projects.slug remplacent les UUID. Toutes les FK dependantes migrees (10+ tables)

### Entités et services

- [x] Derive macro `Resource` : support `#[resource(id)]` pour designer le champ slug comme identifiant (frn-derive)
- [x] Organization : `id: Uuid` supprime, `slug: String` PK avec `#[resource(id)]`, `parent_id` -> `parent_slug`, `generate_organization_slug()` (lettres + tirets, max 49 chars)
- [x] Project : `id: Uuid` supprime, `slug: String` PK avec `#[resource(id)]`, `organization_id` -> `organization_slug`, `generate_project_slug()` = `{org_slug}-{name_slug}`
- [x] Instance : `project_id: Uuid` -> `project_slug: String`
- [x] Hypervisor : `organization_id: Uuid` -> `organization_slug: String`
- [x] ManagedServiceInstance/View : `project_id` -> `project_slug`, `organization_id` -> `organization_slug`
- [x] Invitation : `organization_id: Uuid` -> `organization_slug: String`
- [x] ZeroTrustNetwork : `organization_id: Uuid` -> `organization_slug: String`

### Protos et RPC

- [x] resourcemanager.proto : Organization et Project sans `id`, slug comme identifiant, `organization_slug` au lieu de `organization_id`
- [x] compute.proto : `project_slug` et `organization_slug` dans Instance, Hypervisor, CreateInstanceRequest, UpdateInstanceRequest
- [x] managed.proto : `project_slug` et `organization_slug` dans ManagedServiceInstanceProto, CreateInstanceRequest, ListInstancesRequest
- [x] iam.proto : `organization_slug` dans Invitation et CreateInvitationRequest
- [x] infrastructure.proto : `organization_slug` dans ZeroTrustNetwork
- [x] RPC handlers : suppression de tous les `Uuid::parse_str()` sur org/project IDs, passage direct des slugs String

### SpiceDB et workflows

- [x] SpiceDB relationships utilisent les slugs comme resource IDs (via Resource::id() -> slug)
- [x] Workflow deploy/delete : `project_id: Uuid` -> `project_slug: String`
- [x] Synchronizer : `project_id` -> `project_slug`

### Immuabilité et tests

- [x] Slugs immuables : pas d'endpoint Update sur organizations ni projects (read-only par absence d'API de modification)
- [x] Tests existants adaptes au nouveau schema slug
- [x] Cache .sqlx/ vide : a regenerer avec `cargo sqlx prepare` apres application de la migration
