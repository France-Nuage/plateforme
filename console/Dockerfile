ARG NODE_BASE_IMAGE=registry.france-nuage.fr/library/node:22-alpine
ARG NGINX_BASE_IMAGE=registry.france-nuage.fr/library/nginx:1.27-alpine

FROM ${NODE_BASE_IMAGE} AS base

RUN apk add --no-cache curl

# Build node-sdk first
WORKDIR /node-sdk
COPY ./node-sdk/package.json ./node-sdk/package-lock.json ./
RUN npm ci
COPY ./node-sdk .
RUN npm run build

# Build console
WORKDIR /app
COPY ./console/package.json ./console/package-lock.json ./
RUN npm ci
COPY ./console .

FROM base AS development
WORKDIR /app
CMD ["npm", "run", "dev"]

FROM base AS build
WORKDIR /app
# Environment-specific settings are resolved at runtime via /config.js.
RUN npm run build

FROM ${NGINX_BASE_IMAGE} AS release
COPY --from=build /app/dist /usr/share/nginx/html
COPY ./console/nginx.conf /etc/nginx/conf.d/default.conf
COPY ./console/docker-entrypoint.sh /docker-entrypoint.d/40-runtime-config.sh
RUN chmod +x /docker-entrypoint.d/40-runtime-config.sh
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
