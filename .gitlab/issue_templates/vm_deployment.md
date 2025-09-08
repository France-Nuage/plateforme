# Déploiement de VM

## Etapes de déploiement

- [ ] Organisation Cloudflare du client effectuée
- [ ] Template de déploiement prêt
- [ ] SDN créé dans le DC concerné
- [ ] VM déployée
- [ ] Tunnel actif
- [ ] Application fonctionnelle
- [ ] Accès SSH à la VM depuis la zone zero trust
- [ ] Application accessible via zone zero trust
- [ ] Transmission des informations d'accès au client

## Informations générales

- **Nom de la VM** : <!-- Nom unique pour identifier la VM -->
- **DC où la déployer** : <!-- Nom du DC (DC01, DC02, ...) -->
- **Projet/Service** : <!-- Projet ou service associé à cette VM -->
- **Demandeur** : <!-- @mention de la personne demandant la VM -->
- **Date de déploiement souhaitée** : <!-- Date à laquelle la VM doit être
déployée -->
- **Durée de vie prévue** : <!-- Courte durée, permanente, etc. -->

## Configuration technique

- **Système d'exploitation** : <!-- Ubuntu 22.04, CentOS 7, etc. -->
- **Ressources requises** :
  - CPU : <!-- Nombre de vCPUs -->
  - RAM : <!-- Quantité de mémoire (ex: 4GB) -->
  - Stockage : <!-- Taille du disque (ex: 50GB) -->
- **Réseau** :
  - VLAN/Subnet : <!-- Identifiant du réseau -->
  - Adresse IP fixe : <!-- Oui/Non -- adresse si oui -->
  - Accès Internet : <!-- Oui/Non -->

## Applications et configurations

- **Applications requises** :

- **Configuration post-déploiement** :
  <!-- Tâches de configuration spécifiques -->
  - [ ] Configuration 1
  - [ ] Configuration 2

## Sécurité

- **Niveau de criticité** : <!-- Faible/Moyen/Élevé -->
- **Données sensibles** : <!-- Oui/Non, préciser si nécessaire -->
- **Comptes utilisateurs requis** :
  <!-- Liste des utilisateurs nécessitant un accès à la VM -->
  - Utilisateur 1 (rôle)
  - Utilisateur 2 (rôle)

## Sauvegarde et résilience

- **Plan de sauvegarde** : <!-- Quotidien/Hebdomadaire/Autre -->
- **RTO/RPO requis** : <!-- Objectifs de temps/point de récupération -->
- **Haute disponibilité** : <!-- Oui/Non -->

## Procédure de validation

- **Critères de succès** :
  <!-- Comment vérifier que le déploiement est réussi -->
  - [ ] Test 1
  - [ ] Test 2
- **Validateur(s)** : <!-- @mention des personnes qui valideront -->

## Documentation

- **Documentation spécifique** : <!-- Lien vers documentation additionnelle -->
- **Contacts techniques** : <!-- @mention des personnes à contacter en cas de
problème -->

/label ~"group::sysadmin" ~"app catalog::Intégration"
