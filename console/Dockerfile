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
ARG NEXT_PUBLIC_CONTROLPLANE_URL=https://controlplane.france-nuage.fr
ENV NEXT_PUBLIC_CONTROLPLANE_URL=${NEXT_PUBLIC_CONTROLPLANE_URL}
ARG NEXT_PUBLIC_CONSOLE_URL=https://console.france-nuage.fr
ENV NEXT_PUBLIC_CONSOLE_URL=${NEXT_PUBLIC_CONSOLE_URL}
ARG NEXT_PUBLIC_OIDC_CLIENT_ID=b637aaac8a74582baf4f2e54bd05d3ea530d3985b818597a5a83a73ce2287e78
ENV NEXT_PUBLIC_OIDC_CLIENT_ID=${NEXT_PUBLIC_OIDC_CLIENT_ID}
ARG NEXT_PUBLIC_OIDC_PROVIDER_NAME=gitlab
ENV NEXT_PUBLIC_OIDC_PROVIDER_NAME=${NEXT_PUBLIC_OIDC_PROVIDER_NAME}
ARG NEXT_PUBLIC_OIDC_PROVIDER_URL=https://gitlab.com
ENV NEXT_PUBLIC_OIDC_PROVIDER_URL=${NEXT_PUBLIC_OIDC_PROVIDER_URL}
RUN npm run build

FROM nginx:1.27-alpine AS release
COPY --from=build /app/out /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
