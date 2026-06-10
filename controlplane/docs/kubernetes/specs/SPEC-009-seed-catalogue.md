# SPEC-009 : Seed du catalogue

## Resume

Le seed permet d'initialiser le catalogue de services manages en base a partir
de fichiers YAML. C'est un binaire CLI execute manuellement. Les versions de
production ne sont JAMAIS seedees -- seul le CI les enregistre.

## Format YAML

Un fichier par service dans `controlplane/seed/managed/{slug}.yaml` :

```yaml
service:
  slug: vaultwarden
  name: Vaultwarden
  description: "Gestionnaire de mots de passe auto-heberge"
  category: security
  database_engine: cnpg       # optionnel (cnpg ou mariadb)
  icon_url: null              # optionnel
  deploy_target:              # labels requis sur le cluster hote (SPEC-004) ;
    availability: ft          # sans deploy_target le service n'est pas deployable

plans:                        # optionnel, upsert a chaque execution
  - id: vaultwarden-standard
    name: "Vaultwarden Standard"
    status: active
    highlighted: false
    values:                   # values_override Helm (optionnel)
      resources:
        limits:
          cpu: "500m"
          memory: "512Mi"
    entitlements:
      - key: support_level
        label: "Support"
        value: "Email"
    prices:
      monthly: 999            # centimes
      yearly: 10789

dev_mock_version:             # optionnel, dev local uniquement
  chart_version: 0.1.0
  app_version: 1.35.4
  oci_reference: oci://registry.gitlab.com/.../vaultwarden
  configurable_values_schema: {...}
  ui_schema: {...}
```

## Binaire seed_managed

```
DATABASE_URL=postgres://... cargo run --bin seed_managed -- \
    --dir seed/managed [--with-dev-mock]
```

- `--dir <path>` (obligatoire) : repertoire contenant les fichiers YAML
- `--with-dev-mock` (optionnel) : insere aussi les `dev_mock_version` (dev local uniquement)
- Lit `DATABASE_URL` depuis les variables d'environnement

## Idempotence

- Services : `ON CONFLICT (slug) DO UPDATE` -- met a jour name, description, category, deploy_target, etc.
- Plans : upsert par (service_id, slug) -- met a jour name, status, values, entitlements, prix
- Versions mock : `ON CONFLICT (service_id, chart_version) DO UPDATE` -- met a jour oci_ref, schema
- Re-executer le seed plusieurs fois produit le meme resultat

## Rapport

- Le seed retourne un `Vec<SeedReport>` par fichier traite :
  - `service_slug` : slug du service seede
  - `plans_upserted` : nombre de plans inseres/mis a jour
  - `mock_version_inserted` : si la version mock a ete inseree

## Regle de production

- En production, seul le service et les plans sont seedes
- Les versions sont EXCLUSIVEMENT creees par le CI via `RegisterVersion` (SPEC-003)
- Le flag `--with-dev-mock` ne doit JAMAIS etre utilise en production
- Decision en suspens : execution auto au boot du serveur ou manuelle a chaque deploiement

## Dependance

- `serde_yaml = "0.9"` (marque deprecated, mais largement utilise par l'ecosysteme)
- Migration future possible vers `serde_yaml_ng`
