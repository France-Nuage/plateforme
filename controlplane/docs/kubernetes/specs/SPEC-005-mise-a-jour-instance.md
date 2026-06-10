# SPEC-005 : Mise a jour d'une instance

## Resume

Un utilisateur met a jour une instance vers une nouvelle version de chart
et/ou avec de nouvelles valeurs de configuration. Le control plane transite
la FSM vers `upgrading` et planifie un workflow de mise a jour.

## Prerequis

- L'utilisateur doit avoir la permission `Update` sur l'instance
- L'instance doit etre en statut `running`
- La version cible doit exister et etre active

## Flux de mise a jour (API)

- L'utilisateur fournit : instance_id, version_id, user_values (optionnel), secret_values (optionnel)
- Si user_values n'est pas fourni, les valeurs existantes de l'instance sont conservees
- Le serveur charge l'instance et son cluster_id (le cluster ou la release vit, persiste a la creation ; pas de nouveau matching de labels)
- Transition FSM `running -> upgrading` (evenement `upgrade`)
- Merge des nouvelles values (voir SPEC-008)
- Planification du workflow `UpgradeManagedService`
- Retour immediat avec status `upgrading`

## Workflow de mise a jour (worker)

1. **UpdateK8sSecretOp** : met a jour le K8s Secret avec les nouvelles valeurs secretes
   - Lit et sauvegarde les donnees actuelles du secret avant ecrasement (pour rollback)
   - Remplace le contenu par les nouvelles valeurs

2. **HelmUpgradeOp** : met a jour le chart Helm
   - Commande : `helm upgrade {release} {oci_ref} --version {ver} --namespace {ns} --values - --wait --timeout 5m0s`
   - Idempotent : "no changes since last release" = succes
   - **Rollback** : `helm rollback {release} 0 --wait --timeout 5m0s` (revision 0 = revision precedente)

3. **UpdateInstanceVersionOp** : met a jour le `version_id` sur l'instance en base
   - Sauvegarde la version precedente pour rollback

4. **UpdateInstanceStatusOp** : transition FSM `upgrading -> running` (evenement `upgrade_complete`)

## Rollback natif Helm

- En cas d'echec du helm upgrade, l'operation rollback utilise `helm rollback` qui restaure la revision precedente
- Les secrets K8s sont aussi restaures a leur etat precedent (captures avant modification)
- La version en base est restauree a la version precedente
