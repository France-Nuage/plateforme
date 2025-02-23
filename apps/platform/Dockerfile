#FROM node:22-alpine as development
#WORKDIR /usr/src/app
#COPY package*.json .
#RUN npm install
#COPY . .
#EXPOSE 3000
#CMD ["npm", "run", "dev"]

# Étape de construction
FROM node:22-alpine AS builder

# Installation des dépendances système nécessaires
RUN apk add --no-cache libc6-compat python3 make g++

# Définition du répertoire de travail
WORKDIR /usr/src/app

# Copie des fichiers depuis le contexte platform
COPY package*.json ./

# Installation des dépendances
RUN npm install

# Copie du reste du code de l'application
COPY . .

# Build de l'application
RUN npm run build

# Étape de développement
FROM node:22-alpine AS development

WORKDIR /usr/src/app
COPY --from=builder /usr/src/app ./

# L'application démarre en mode développement
CMD ["npm", "run", "dev"]

# Étape de production
FROM node:20-alpine AS production

WORKDIR /usr/src/app

# Copie des fichiers de production depuis l'étape de build
COPY --from=builder /usr/src/app/.output ./.output
COPY --from=builder /usr/src/app/package*.json ./

# Installation des dépendances de production
ENV NODE_ENV=production
RUN npm install --production

# Configuration de l'application
ENV NUXT_HOST=0.0.0.0
ENV PORT=3000

EXPOSE 3000
EXPOSE 39709

CMD ["node", ".output/server/index.mjs"]