# Utiliser une image Node.js comme base
FROM node:18

# Installer TypeScript globalement
RUN npm install -g typescript

# Définir le répertoire de travail
WORKDIR /app

# Copier les fichiers de configuration
COPY package*.json tsconfig.json ./

# Installer les dépendances
# Build Stage
FROM node:22-alpine AS build

WORKDIR /usr/src/app

# Copy monorepo root files (package.json, package-lock.json, turbo.json)
COPY ./package.json ./package-lock.json ./turbo.json ./

# Copy workspace files (apps and packages)
COPY ./apps ./apps
#COPY ./packages ./packages

# Install all dependencies (including dev dependencies) in the build stage
RUN npm install

# Copier les fichiers sources
COPY src ./src
# Build the app (specific to the "agent" app in the monorepo)
RUN npx turbo run build --filter=agent

# Production Stage
FROM node:22-alpine

WORKDIR /usr/src/app

# Copy the root package.json and package-lock.json to the production image
COPY ./package.json ./package-lock.json ./

# Copy the app-specific package.json files (if needed for production dependencies)
COPY ./apps/agent/package*.json ./apps/agent/

# Compiler les fichiers TypeScript en JavaScript
RUN tsc
# Copy the built files from the build stage
COPY --from=build /usr/src/app/apps/agent/dist ./dist

# Exposer le port si nécessaire (ajustez selon votre app)
EXPOSE 3333
# Install only production dependencies (omit dev dependencies)
RUN npm install typescript -g
RUN npm install --omit=dev --ignore-scripts

# Démarrer l'application (ajustez si le point d'entrée est différent)

# Use non-root user for better security
USER node

# Start the application
CMD ["node", "dist/france-nuage-agent.js"]
