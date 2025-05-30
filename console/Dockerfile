FROM node:22 AS base
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .

FROM base AS development
WORKDIR /app
CMD ["npm", "run", "dev"]

FROM base AS build
WORKDIR /app
ARG VITE_CONTROLPLANE_URL=https://controlplane.france-nuage.fr
ENV VITE_CONTROLPLANE_URL=${VITE_CONTROLPLANE_URL}
ARG VITE_OIDC_CLIENT_ID=b637aaac8a74582baf4f2e54bd05d3ea530d3985b818597a5a83a73ce2287e78
ENV VITE_OIDC_CLIENT_ID=${VITE_OIDC_CLIENT_ID}
ARG VITE_OIDC_PROVIDER_NAME=gitlab
ENV VITE_OIDC_PROVIDER_NAME=${VITE_OIDC_PROVIDER_NAME}
ARG VITE_OIDC_PROVIDER_URL=https://gitlab.com
ENV VITE_OIDC_PROVIDER_URL=${VITE_OIDC_PROVIDER_URL}
RUN npm run build

FROM nginx:1.27-alpine AS release
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
