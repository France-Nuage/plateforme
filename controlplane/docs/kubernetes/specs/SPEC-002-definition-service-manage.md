# SPEC-002 : Definition d'un service manage

## Resume

Un service manage est une application deployable par les clients via Helm sur un cluster K8s
multi-tenant partage. Le control plane stocke le catalogue (metadonnees), le registry OCI
stocke les artefacts (charts Helm).

## Structure d'un service manage

- **Service** (`managed.service`) : entree catalogue identifiee par un `slug` unique (kebab-case)
  - Champs : slug, name, description, category, database_engine (optionnel), icon_url, deploy_target
  - Categories : security, collaboration, analytics, database, automation, cms, erp, storage, dashboard
  - Moteurs DB : cnpg (CloudNativePG) ou mariadb
  - `deploy_target` (JSONB) : selecteur de labels resolu au deploiement, objet
    cle/valeur (ex: `{"availability": "ft"}`). Seuls les clusters sains portant
    TOUTES les paires sont eligibles. NULL ou `{}` = service non deployable
    (erreur typee `MissingDeployTarget`). Non expose par l'API catalogue.
  - Soft-delete via `deactivated_at` (null = actif, timestamp = desactive)

- **Version** (`managed.service_version`) : une version precise d'un chart Helm publie par le CI
  - Champs : chart_version (semver), app_version, oci_reference (prefix `oci://`), configurable_values_schema (JSON Schema), ui_schema (rjsf)
  - Contrainte d'unicite : (service_id, chart_version)
  - Soft-delete via `deactivated_at`
  - Les versions sont crees exclusivement par le CI via la RPC `RegisterVersion` (jamais manuellement)

- **Plan** (`managed.service_plan`) : offre tarifaire pour un service
  - Champs : slug (unique par service), name, description, status (active/archived), highlighted
  - `values_override` (JSONB) : valeurs Helm injectees par le plan (ex: limites CPU/RAM, replicas)
  - `entitlements` (JSONB) : tableau `[{key, label, value}]` affiche sur la page pricing
  - `price_monthly_cents`, `price_yearly_cents` : prix en centimes (optionnels)

- **Instance** (`managed.service_instance`) : un deploiement concret pour un projet client
  - Liee a un service, une version, un plan, un projet, une organisation, et au
    cluster qui l'heberge (`cluster_id`, resolu a la creation via le deploy_target)
  - Possede un namespace K8s unique et un release_name Helm unique
  - Stocke les `user_values` (JSONB) non-secrets
  - Statut gere par FSM (voir SPEC-004/005/006)

## FSM des instances

- Etats : provisioning, running, upgrading, failed, deleting, deleted
- Transitions :
  - provisioning -> running (provision_complete) | failed (fail)
  - running -> upgrading (upgrade) | deleting (delete) | failed (fail)
  - upgrading -> running (upgrade_complete) | failed (fail)
  - failed -> provisioning (retry) | deleting (delete)
  - deleting -> deleted (delete_complete) | failed (fail)

## API catalogue (publique, sans authentification)

- `ListServices` : tous les services actifs, tries par nom
- `GetService(slug)` : un service par slug
- `ListVersions(service_slug)` : versions non desactivees, triees par date decroissante
- `ListPlans(service_slug)` : plans actifs, tries par date croissante

## Source de verite

- Le CI/CD est la frontiere de transaction : il pousse d'abord le chart dans le registry OCI, puis enregistre la version via l'API
- Si l'enregistrement echoue, la version existe dans le registry mais n'est pas visible aux clients
- La DB stocke des metadonnees, pas le contenu du chart
