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
ARG VITE_APPLICATION_DEFAULT_MODE=rpc
ENV VITE_APPLICATION_DEFAULT_MODE=${VITE_APPLICATION_DEFAULT_MODE}
ARG VITE_CONTROLPLANE_URL=https://controlplane.france-nuage.fr
ENV VITE_CONTROLPLANE_URL=${VITE_CONTROLPLANE_URL}
# OIDC est géré server-side par le BFF (control-plane) : le SPA n'a plus aucune
# config OIDC (le flow public PKCE + oidc-client-ts a été retiré).
RUN npm run build

FROM ${NGINX_BASE_IMAGE} AS release
COPY --from=build /app/dist /usr/share/nginx/html
COPY ./console/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
