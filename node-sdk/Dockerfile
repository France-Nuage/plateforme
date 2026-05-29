ARG NODE_BASE_IMAGE=registry.france-nuage.fr/library/node:22-alpine
FROM ${NODE_BASE_IMAGE} AS base
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
CMD ["npm", "run", "build"]
