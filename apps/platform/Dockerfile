#FROM node:22-alpine as development
#WORKDIR /usr/src/app
#COPY package*.json .
#RUN npm install
#COPY . .
#EXPOSE 3000
#CMD ["npm", "run", "dev"]

# Étape de construction
FROM node:22-alpine AS build

# Installation des dépendances système nécessaires
RUN apk add --no-cache libc6-compat python3 make g++

# Définition du répertoire de travail
WORKDIR /app

# Copie des fichiers essentiels pour l'installation
COPY package*.json ./
COPY turbo.json ./

# Copie des packages locaux
COPY packages/ ./packages/

# Copie de l'application platform
COPY apps/platform/ ./apps/platform/

# Installation avec npm
RUN npm install
RUN npx turbo build --filter=platform

# Étape de production
FROM node:20-alpine AS development

WORKDIR /app

# Copie des fichiers de production depuis l'étape de build
COPY --from=build /app/apps/platform/.output ./.output
COPY --from=build /app/package*.json ./

# Installation des dépendances de production
ENV NODE_ENV=production
RUN npm install --production

EXPOSE 3000

CMD ["node", ".output/server/index.mjs"]