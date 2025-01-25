```md
# MonProjetAdonis

Bienvenue dans **API**, une application construite avec le framework [AdonisJS](https://adonisjs.com/). Ce document vous guidera pas à pas pour l'installation, la configuration et l'exécution de ce projet.

## Table des Matières

- [Aperçu](#aperçu)
- [Prérequis](#prérequis)
- [Installation](#installation)
  - [Configuration de la base de données](#configuration-de-la-base-de-données)
- [Utilisation](#utilisation)
  - [Lancement du serveur](#lancement-du-serveur)
- [Scripts utiles](#scripts-utiles)
- [Licence](#licence)

---

## Aperçu

AdonisJS est un framework MVC Node.js permettant de développer des applications web et API performantes. Ce projet sert d’exemple de mise en place d’un environnement de développement et de déploiement pour une application AdonisJS.

## Prérequis

Avant de commencer, assurez-vous que les outils suivants sont installés sur votre machine :

1. **Node.js** (version 14 ou supérieure) : [Télécharger Node.js](https://nodejs.org/)
2. **npm** (généralement inclus avec Node.js)
3. **AdonisJS CLI** (facultatif, mais pratique pour la création de projets et le démarrage rapide) :
   ```bash
   npm i -g @adonisjs/cli
   ```
4. **Base de données** :
  - Nous utilisons Postgresql

## Installation

1. **Cloner le dépôt** :
   ```bash
   git clone git@gitlab.com:getnobullshit/france-nuage/plateforme.git
   cd plateforme/apps/api
   ```

2. **Installer les dépendances** :
   ```bash
   npm install
   ```

### Configuration de la base de données

1. Copiez le fichier `.env.example` en `.env` :
   ```bash
   cp .env.example .env
   ```
2. Dans le fichier `.env`, mettez à jour les variables de configuration de la base de données :
   ```env
   DB_CONNECTION=postgres
   DB_HOST=127.0.0.1
   DB_PORT=5432
   DB_USER=root
   DB_PASSWORD=root
   DB_DATABASE=mon_projet_adonis
   ```
   Ajustez ces valeurs selon votre configuration (MySQL, PostgreSQL, etc.).

3. **Migration de la base de données** :
   ```bash
   node ace migration:run
   ```
   Cela créera les tables nécessaires dans votre base de données.

## Utilisation

### Lancement du serveur

Démarrez le serveur de développement AdonisJS :

```bash
docker compose up && 
node ace serve --hmr
```

Par défaut, l’application tournera sur le port `3333`. Vous pouvez y accéder à l’adresse :  
[http://127.0.0.1:3333](http://127.0.0.1:3333)

## Scripts utiles

- **Démarrer en mode développement** :
  ```bash
  node ace serve --hmr
  ```
- **Exécuter les migrations** :
  ```bash
  node ace migration:run
  ```
- **Créer une migration** :
  ```bash
  node ace make:migration <nom_de_migration>
  ```
- **Créer un contrôleur** :
  ```bash
  node ace make:controller <NomDuContrôleur>
  ```

## Licence

Ce projet est sous licence [MIT](LICENSE). Vous êtes libre de l’utiliser, le modifier et le distribuer à votre convenance, sous réserve d’inclure la licence originale.

---

Merci d’avoir lu ce README. Si vous avez des questions, n’hésitez pas à [ouvrir une issue](https://github.com/mon-utilisateur/MonProjetAdonis/issues) sur ce dépôt !
```
