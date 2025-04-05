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

FROM nginx:1.27-alpine AS release
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
