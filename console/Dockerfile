FROM node:22-alpine AS base

RUN apk add curl

WORKDIR /node-sdk
COPY ./node-sdk .
RUN npm ci
RUN npm run build

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
ARG VITE_OIDC_CLIENT_ID=france-nuage-console
ENV VITE_OIDC_CLIENT_ID=${VITE_OIDC_CLIENT_ID}
ARG VITE_OIDC_PROVIDER_NAME=france-nuage
ENV VITE_OIDC_PROVIDER_NAME=${VITE_OIDC_PROVIDER_NAME}
ARG VITE_OIDC_PROVIDER_URL=https://auth.france-nuage.fr/realms/france-nuage
ENV VITE_OIDC_PROVIDER_URL=${VITE_OIDC_PROVIDER_URL}
RUN npm run build

FROM nginx:1.27-alpine AS release
COPY --from=build /app/dist /usr/share/nginx/html
COPY ./console/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
