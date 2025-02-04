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

# Configuration de l'environnement
ENV NODE_ENV=production

# Copie des fichiers nécessaires uniquement
COPY package*.json ./
RUN npm ci --only=production
COPY --from=builder /app/dist ./dist

# Changement d'utilisateur pour plus de sécurité
USER nodeuser

# Healthcheck (optionnel mais recommandé)
HEALTHCHECK --interval=30s --timeout=3s \
  CMD node -e "process.exit(1)"

# Point d'entrée de l'application
CMD ["node", "dist/france-nuage-agent.js"]