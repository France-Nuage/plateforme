# Fichiers modifiés et créés

## 📝 Fichiers créés

### 1. Migration SQL
```
controlplane/migrations/20260126000000_add_soft_delete_to_instances.sql
```
- Ajoute la colonne `deleted_at` à la table `instances`
- Crée les indexes pour optimiser les requêtes

## ✏️ Fichiers modifiés

### 1. Modèle Instance
```
controlplane/frn-core/src/compute/instance.rs
```

**Modifications**:
- Ajout du champ `deleted_at` avec l'attribut `#[fabrique(soft_delete)]`
- Modification d'une requête SQL pour exclure les instances supprimées (`AND deleted_at IS NULL`)

### 2. Synchroniseur
```
controlplane/synchronizer/src/lib.rs
```

**Modifications**:
- Ajout de l'import: `use chrono::Utc;`
- Ajout du champ `deleted_at: None` dans la construction des instances
- Ajout de la logique complète de soft delete:
  - Récupère les IDs des instances synchronisées
  - Récupère toutes les instances non supprimées du hyperviseur
  - Identifie les instances disparues
  - Marque les instances disparues comme supprimées avec `deleted_at = now()`
  - Sauvegarde les changements en base de données

## 📊 Statistiques des changements

- **Fichiers créés**: 1 (migration SQL)
- **Fichiers modifiés**: 2 (Instance, Synchronizer)
- **Fichiers de documentation créés**: 2 (IMPLEMENTATION_TEST.md, IMPLEMENTATION_SUMMARY.md)
- **Lignes ajoutées**: ~50 (logique de soft delete + migration + documentation)

## 🔄 Flux de synchronisation avec soft delete

```
1. Synchronisation commence
   ↓
2. Récupère les instances du hyperviseur (distant_instances)
   ↓
3. Pour chaque instance distante:
   - Crée ou met à jour en base de données
   - `deleted_at` = NULL (réactivation si était supprimée)
   ↓
4. Récupère toutes les instances non supprimées en base pour ce hyperviseur
   ↓
5. Compare avec la liste des instances distantes
   ↓
6. Identifie les instances qui ont disparu du hyperviseur
   ↓
7. Marque ces instances comme supprimées (`deleted_at` = now())
   ↓
8. Sauvegarde en base de données
   ↓
9. Continue pour le prochain hyperviseur
   ↓
10. Synchronisation terminée
```

## ✅ Validation

Tous les changements ont été validés et respectent:

- ✅ La specification de l'issue (soft delete des instances disparues)
- ✅ Le pattern Fabrique `#[fabrique(soft_delete)]`
- ✅ Les bonnes pratiques (indexes, logging)
- ✅ La sécurité (pas de faux positifs lors d'offline des hyperviseurs)
- ✅ La récupération (réactivation possible si instance réapparaît)

## 🚀 Déploiement

Pour déployer ces changements:

1. Exécuter la migration SQL:
```sql
-- Appliquée automatiquement par le système de migration
```

2. Compiler le code:
```bash
cd controlplane
cargo build --release
```

3. Redémarrer les services:
```bash
docker-compose down
docker-compose up -d
```

4. Vérifier les logs du synchroniseur:
```bash
docker-compose logs -f synchronizer
```

Vous devriez voir des messages comme:
```
"Soft deleting instances not found on hypervisor"
```

## 📌 Points à retenir

- La suppression est **soft delete** (pas de suppression physique en BD)
- Les instances peuvent être réactivées
- Les performances sont optimisées avec les indexes
- Le logging permet de tracer les actions
- Aucun impact sur les APIs existantes
