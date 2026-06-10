# SPEC-006 : Suppression d'une instance

## Resume

Un utilisateur supprime une instance de service manage. Le control plane transite
la FSM vers `deleting` et planifie un workflow de suppression qui nettoie toutes
les ressources Kubernetes et les relations d'acces.

## Prerequis

- L'utilisateur doit avoir la permission `Delete` sur l'instance
- L'instance doit etre en statut `running` ou `failed`

## Flux de suppression (API)

- L'utilisateur fournit : instance_id
- Le serveur charge l'instance et son cluster_id (le cluster ou la release vit, persiste a la creation ; pas de nouveau matching de labels)
- Transition FSM `running -> deleting` (ou `failed -> deleting`)
- Calcul des values mergees existantes (user + platform) pour le rollback eventuel
- Planification du workflow `DeleteManagedService`
- Retour immediat avec status `deleting`

## Workflow de suppression (worker)

1. **HelmUninstallOp** : desinstalle le chart Helm
   - Commande : `helm uninstall {release} --namespace {ns}`
   - Idempotent : "not found" = deja supprime
   - **Rollback** : `helm install` avec le meme chart, version et values (reinstallation complete)
   - Le chart_reference et les values sont stockes dans l'operation pour permettre ce rollback

2. **DeleteK8sSecretOp** : supprime le K8s Secret
   - Lit et capture le contenu du secret avant suppression (pour rollback)
   - Idempotent : HTTP 404 = deja supprime
   - **Rollback** : recree le secret avec les donnees capturees

3. **DeleteNamespaceOp** : supprime le namespace K8s
   - Idempotent : HTTP 404 = deja supprime
   - **Rollback** : recree le namespace avec les labels originaux

4. **DeleteRelationshipsOp** : supprime la relation SpiceDB `project#parent@managed_service_instance`
   - **Rollback** : reecrit la relation

5. **UpdateInstanceStatusOp** : transition FSM `deleting -> deleted` (evenement `delete_complete`)

## Ordre des operations

- L'ordre est intentionnel : d'abord desinstaller l'application (Helm), puis nettoyer les ressources (secret, namespace), puis les relations d'acces
- Le namespace est supprime en dernier cote K8s car il contient potentiellement d'autres ressources creees par le chart
