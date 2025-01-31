# Utiliser une image Node.js comme base
FROM node:18

# Installer TypeScript globalement
RUN npm install -g typescript

# Définir le répertoire de travail
WORKDIR /app

# Copier les fichiers de configuration
COPY package*.json tsconfig.json ./

# Installer les dépendances
RUN npm install

# Copier les fichiers sources
COPY src ./src

# Compiler les fichiers TypeScript en JavaScript
RUN tsc

# Exposer le port si nécessaire (ajustez selon votre app)
EXPOSE 3333

# Démarrer l'application (ajustez si le point d'entrée est différent)
CMD ["node", "dist/france-nuage-agent.js"]
