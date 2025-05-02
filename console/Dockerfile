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
RUN npm run build

FROM nginx:1.27-alpine AS release
COPY --from=build /app/out /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
