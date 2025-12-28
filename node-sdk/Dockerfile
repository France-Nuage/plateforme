FROM node:22-alpine AS base
WORKDIR /app
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm \
  npm ci
COPY . .
CMD ["npm", "run", "build"]
