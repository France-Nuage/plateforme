FROM node:18.19-slim AS builder

WORKDIR /app

# Installation des dépendances
COPY package*.json ./
COPY tsconfig.json ./
RUN npm ci
RUN npm install -g typescript

# Build de l'application
COPY src ./src
RUN npm run build

# Production stage
FROM node:18.19-slim

WORKDIR /app

# Création d'un utilisateur non-root
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nodeuser

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
# Configuration de l'environnement
ENV NODE_ENV=production

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
# Copie des fichiers nécessaires uniquement
COPY package*.json ./
RUN npm ci --only=production
COPY --from=builder /app/dist ./dist

# Compiler les fichiers TypeScript en JavaScript
RUN tsc
# Copy the built files from the build stage
COPY --from=build /usr/src/app/apps/agent/dist ./dist
# Changement d'utilisateur pour plus de sécurité
USER nodeuser

# Exposer le port si nécessaire (ajustez selon votre app)
EXPOSE 3333
# Install only production dependencies (omit dev dependencies)
RUN npm install typescript -g
RUN npm install --omit=dev --ignore-scripts
# Healthcheck (optionnel mais recommandé)
HEALTHCHECK --interval=30s --timeout=3s \
  CMD node -e "process.exit(1)"

# Démarrer l'application (ajustez si le point d'entrée est différent)

# Use non-root user for better security
USER node

# Start the application
# Point d'entrée de l'application
CMD ["node", "dist/france-nuage-agent.js"]