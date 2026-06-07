# SPEC-008 : Gestion des values et secrets

## Resume

Les values de deploiement Helm sont construites a partir de 3 couches avec une
priorite stricte. Les secrets ne transitent jamais par la base de donnees : ils
sont stockes exclusivement dans des K8s Secrets.

## 3 couches de values

1. **user_values** (client, stocke en DB dans `user_values` JSONB)
   - Fournies par l'utilisateur via le formulaire console
   - Exemples : domain, smtp.host, smtp.port, timezone, parametres applicatifs
   - Persistees pour etre reutilisees lors des upgrades

2. **plan values_override** (plan, stocke en DB dans `managed.service_plan.values_override` JSONB)
   - Injectees automatiquement selon le plan selectionne
   - Exemples : limites CPU/RAM, nombre de replicas, taille du stockage
   - Appliquees par-dessus les user_values

3. **platform_values** (genere au runtime par `build_platform_values()`)
   - Non-overridable par l'utilisateur ni le plan
   - Contenu actuel : `persistence.storageClass` et `cnpg.storageClass` (si database_engine defini)
   - Valeur : variable d'environnement `MANAGED_DEFAULT_STORAGE_CLASS`

## Priorite de merge

```
platform > plan > user
```

- `merge_helm_values(user, plan)` -> user_plus_plan
- `merge_helm_values(user_plus_plan, platform)` -> values finales
- `deep_merge` recursif : les valeurs scalaires de la couche superieure ecrasent celles de la couche inferieure

## Secrets (valeurs sensibles)

- Les secrets ne sont JAMAIS stockes en base de donnees
- Le `configurable_values_schema` annote les champs secrets avec `"format": "password"`
- A la creation/mise a jour, l'API recoit le payload complet du client
- Le helper `splitUserAndSecretValues(schema, formData)` separe :
  - Champs normaux -> `user_values` (DB)
  - Champs `format: password` -> `secret_values` (K8s Secret)
- Les secrets sont ecrits dans un K8s Secret nomme `{release_name}-secrets`
- Les templates du chart doivent referencer ce secret directement

## Secrets concernes

- `smtp.password` : mot de passe SMTP
- `oidc.client_secret` : secret client SSO/OIDC
- `s3.access_key`, `s3.secret_key` : credentials S3

## Formulaire console (rjsf)

- Le formulaire est genere dynamiquement depuis `configurable_values_schema` (JSON Schema) + `ui_schema` (rjsf)
- Stack : `@rjsf/core` + `@rjsf/chakra-ui` + `@rjsf/validator-ajv8`
- Le split user/secret est transparent pour l'utilisateur (un seul formulaire)
- Les champs password utilisent le widget password natif de rjsf
