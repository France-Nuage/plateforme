# SPEC-007 : Workflow engine et rollback

## Resume

Le workflow engine est un systeme d'execution durable base sur PostgreSQL et un worker
autonome. Il execute des operations sequentielles reversibles avec retry automatique
et rollback en cas d'echec.

## Architecture

- **Serveur gRPC** (`WorkflowEngine` service) : stocke les workflows en base, gere le locking
- **Worker** (binaire standalone) : poll le serveur, execute les operations, reporte le resultat
- Communication worker <-> serveur via 4 RPCs : `Next`, `Schedule`, `Unlock`, `GetStatus`
- Authentification worker via token partage `WORKER_TOKEN`

## Cycle de vie d'un workflow

- Un workflow est une table `workflow.execution` avec :
  - `definition` (JSONB) : le workflow serialise (type + parametres)
  - `status` : FSM avec etats pending, running, will_retry, completed, failed
  - `soft_try_count` / `hard_try_count` / `max_try_count` : compteurs de retry
  - `locked_until` : verrou optimiste pour le worker
  - `next_retry_at` : prochain essai planifie

- Transitions FSM : pending -> running -> completed | failed | will_retry -> running

## Execution par le worker

Le worker traite un workflow en 5 phases :

1. **Verification des dependances** : si le workflow a des pre-requis, verifie leur statut
   - Si un pre-requis a echoue -> echec immediat
   - Si un pre-requis n'est pas termine -> will_retry (reessai apres le pre-requis)

2. **Planification des pre-requis** : appelle `needed_workflows()`, planifie les sous-workflows

3. **Boucle d'operations** (max 100 tours) :
   - Appelle `next_operations()` sur la definition
   - Si la liste est vide -> les etapes sont terminees
   - Execute toutes les operations du tour en parallele
   - Succes : ajoute les operations a la liste de rollback
   - Echec : rollback LIFO de toutes les operations precedentes

4. **Commit** : appelle `commit()` sur chaque operation (hook pour operations deux-phases)

5. **Workflows suivants** : planifie les follow-ups via `next_workflows()`

## Rollback

- Rollback eager, dans le scope d'une tentative uniquement
- Les operations reussies sont empilees dans une liste (LIFO)
- En cas d'echec d'une operation, rollback de toutes les operations precedentes en ordre inverse
- Chaque operation capture son etat avant modification pour permettre le rollback :
  - `DeleteK8sSecretOp` capture le contenu du secret avant suppression
  - `UpdateK8sSecretOp` capture les donnees precedentes avant ecrasement
  - `HelmUpgradeOp` rollback via `helm rollback {release} 0` (revision precedente)
  - `UpdateInstanceVersionOp` capture la version precedente
- Si un rollback echoue lui-meme, les rollbacks restants sont abandonnes (best-effort)

## Classification des erreurs

- **Transient** (`#[operation_error(transient)]`) : erreurs reseau, I/O, kube
  - N'incremente PAS `hard_try_count` (retry infini tant que max non atteint)
- **Standard** : incremente `hard_try_count`
  - Echec definitif quand `hard_try_count >= max_try_count`
- **Invariant** (`#[operation_error(invariant)]`) : erreurs de serialisation, logique
  - Echec definitif immediat (pas de retry)

## Retry et backoff

- Backoff exponentiel : `2^hard_try_count` secondes, plafonne a 1 heure
- `max_try_count` par defaut : 3 (configure par workflow)
- Timeout par execution : 4 minutes
- Duree du verrou : 5 minutes

## Idempotence des operations

- Toutes les operations sont idempotentes par design :
  - HTTP 409 (Conflict) = deja cree -> succes
  - HTTP 404 (Not Found) = deja supprime -> succes
  - Helm "cannot re-use a name" = deja installe -> succes
  - Helm "not found" = deja desinstalle -> succes
- Cela permet de re-executer un workflow depuis le debut en cas de retry
