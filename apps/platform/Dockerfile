#FROM node:22-alpine as development
#WORKDIR /usr/src/app
#COPY package*.json .
#RUN npm install
#COPY . .
#EXPOSE 3000
#CMD ["npm", "run", "dev"]

FROM node:22-alpine AS builder

# Installation des dépendances système nécessaires
RUN apk add --no-cache libc6-compat python3 make g++
RUN corepack enable

# Définition du répertoire de travail
WORKDIR /app

# Copie des fichiers de configuration
COPY . .

# Installation des dépendances avec Turbo
RUN pnpm install
RUN pnpm turbo build --filter=@france-nuage/web...

# Étape de production
FROM node:20-alpine AS development

# Installation des dépendances système nécessaires pour la production
RUN apk add --no-cache libc6-compat
RUN corepack enable

WORKDIR /app

# Copie des fichiers nécessaires depuis l'étape de construction
COPY --from=builder /app/apps/web/.output ./apps/web/.output
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/pnpm-lock.yaml ./pnpm-lock.yaml

# Installation des dépendances de production uniquement
ENV NODE_ENV=production
RUN pnpm install --prod

# Configuration des variables d'environnement
ENV NUXT_HOST=0.0.0.0
ENV PORT=3000

# Exposition du port
EXPOSE 3000

# Commande de démarrage
CMD ["node", "apps/web/.output/server/index.mjs"]