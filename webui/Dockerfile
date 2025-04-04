FROM node:22 AS base
WORKDIR /app
RUN corepack enable
COPY . .
RUN pnpm install

FROM base AS development
WORKDIR /app
CMD ["pnpm", "run", "dev", "--host"]

FROM base AS build
WORKDIR /app
RUN pnpm run build

FROM node:22-alpine AS release
WORKDIR /app
RUN corepack enable
COPY --from=build dist node_modules package.json pnpm-lock.yaml ./
CMD ["pnpm", "run", "preview", "--host"]
