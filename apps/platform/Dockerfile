#FROM node:22-alpine as development
#WORKDIR /usr/src/app
#COPY package*.json .
#RUN npm install
#COPY . .
#EXPOSE 3000
#CMD ["npm", "run", "dev"]

# Build Stage
FROM node:22-alpine AS build

WORKDIR /usr/src/app

# Copy monorepo root files (package.json, package-lock.json, turbo.json)
COPY ./package.json ./package-lock.json ./turbo.json ./

# Copy workspace files (apps and packages)
COPY ./apps ./apps
COPY ./packages ./packages

# Install all dependencies (including dev dependencies) in the build stage
RUN npm install

# Build the app (specific to the "platform" app in the monorepo)
RUN npx turbo run build --filter=platform

# Development Stage
FROM node:22-alpine AS development

RUN mkdir -p /usr/src/app/apps

WORKDIR /usr/src/app/apps/platform

COPY ./apps/platform/package*.json /usr/src/app/apps/platform
COPY ./packages /usr/src/app/packages

RUN npm install

COPY ./apps/platform /usr/src/app/apps/platform

EXPOSE 3000

CMD ["npm", "run", "dev"]