# SPEC-003 : Enregistrement d'une version depuis le CI

## Resume

Apres avoir publie un chart Helm dans le registry OCI, le pipeline CI enregistre
la version dans le control plane via la RPC `RegisterVersion`. C'est le seul chemin
pour creer une version en production.

## Authentification CI

- Token statique pre-partage, charge depuis la variable d'environnement `CI_SERVICE_TOKEN`
- Le CI envoie le header `Authorization: Bearer <token>`
- Comparaison en temps constant via `subtle::ConstantTimeEq` (protection contre les timing attacks)
- Pas de table service_account : approche simplifiee, un seul token pour le CI
- Si le token est absent ou invalide : gRPC `Unauthenticated`

## Flux d'enregistrement

- Le CI existant fait deja : lint-and-test, version-check, yaml-lint, chart-schema, kubesec, publish-oci
- Apres le publish OCI, un stage `register` appelle `RegisterVersion` avec :
  - `service_slug` : identifie le service cible (doit exister en base)
  - `chart_version` : version semver du chart
  - `app_version` : version de l'application (optionnel)
  - `oci_reference` : reference OCI complete (doit commencer par `oci://`)
  - `configurable_values_schema` : JSON Schema des values configurables par le client (optionnel)
  - `ui_schema` : UI Schema rjsf pour le rendu du formulaire console (optionnel)

## Validations

- `service_slug` non vide, le service doit exister en base
- `oci_reference` non vide, doit commencer par `oci://`
- `configurable_values_schema` doit etre du JSON valide si present
- `ui_schema` doit etre du JSON valide si present

## Idempotence

- Insert avec `ON CONFLICT (service_id, chart_version) DO NOTHING`
- Si la version existe deja : retourne gRPC `AlreadyExists`
- Une version enregistree n'est jamais mise a jour silencieusement (pas de DO UPDATE)

## Conventions attendues cote charts

- Annotation `france-nuage.fr/managed: "true"` dans `Chart.yaml` (selection des charts managed)
- `values.schema.json` : JSON Schema des values configurables
- `values.ui-schema.json` : UI Schema rjsf (optionnel, rendu auto si absent)

## RPC protegees par le meme token

- `RegisterVersion` : enregistrement d'une version
- `SyncPlans` : synchronisation des plans depuis le catalogue YAML
