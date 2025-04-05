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
ARG VITE_CONTROLPLANE_URL=https://controlplane.france-nuage.fr
ENV VITE_CONTROLPLANE_URL=${VITE_CONTROLPLANE_URL}
RUN pnpm run build

FROM node:22-alpine AS release
WORKDIR /app
RUN corepack enable
COPY --from=build /app/dist ./dist
COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/package.json /app/pnpm-lock.yaml ./
CMD ["pnpm", "run", "preview", "--host", "--port=80"]
