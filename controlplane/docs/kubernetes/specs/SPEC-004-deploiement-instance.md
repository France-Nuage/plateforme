# SPEC-004 : Deploiement d'une instance

## Resume

Un utilisateur cree une instance de service manage depuis la console.
Le control plane valide la demande, insere l'instance en base, et planifie
un workflow de deploiement execute par le worker.

## Prerequis

- L'utilisateur doit avoir la permission `CreateInstance` sur le projet
- Le service, la version et le plan doivent exister et etre actifs
- Le projet doit etre assigne a un cluster Kubernetes sain

## Flux de creation (API)

- L'utilisateur fournit : service_slug, version_id, plan_id, project_id, user_values, secret_values
- Le serveur resout l'organisation et le cluster_id depuis le projet
- Le serveur valide le plan (doit appartenir au service, status = active)
- Generation du namespace : `managed-{org_slug}-{service_slug}-{env}` (suffixe `-{n}` si n > 1, max 63 chars)
- Generation du release_name : `{service_slug}` (suffixe `-{n}` si n > 1, max 53 chars)
- Merge des values (voir SPEC-008)
- Extraction des secrets depuis `secret_values` -> `secret_data` (BTreeMap cle/valeur)
- Insert de l'instance en base avec FSM initialisee a `provisioning`
- Planification du workflow `DeployManagedService` (dans la meme transaction DB)
- Retour immediat avec status `provisioning`

## Workflow de deploiement (worker)

Les etapes s'executent sequentiellement. Chaque etape est rollbackable (voir SPEC-007).

1. **CreateNamespaceOp** : cree le namespace K8s avec les labels instance
   - Labels : `app.kubernetes.io/managed-by=france-nuage`, `france-nuage/service`, `france-nuage/instance`, `france-nuage/project`
   - Idempotent : HTTP 409 (deja existant) = succes

2. **CreateK8sSecretOp** : cree le K8s Secret `{release_name}-secrets` dans le namespace
   - Contient les valeurs sensibles (smtp.password, oidc.client_secret, cles S3)
   - Idempotent : HTTP 409 = succes

3. **HelmInstallOp** : installe le chart Helm
   - Commande : `helm install {release} {oci_ref} --version {ver} --namespace {ns} --values - --wait --timeout 5m0s`
   - Values passees via stdin en JSON
   - Le kubeconfig du cluster cible est ecrit dans un fichier temporaire et passe via `--kubeconfig`
   - Idempotent : "cannot re-use a name" = deja installe

4. **WriteRelationshipsOp** : ecrit la relation SpiceDB `project:{project_id}#parent@managed_service_instance:{instance_id}`
   - Permet le controle d'acces par relation

5. **UpdateInstanceStatusOp** : transition FSM `provisioning -> running` (evenement `provision_complete`)

## Resultat

- Succes : l'instance passe en `running`, le service est accessible via son ingress
- Echec : rollback LIFO des operations reussies, l'instance passe en `failed` apres epuisement des retries
