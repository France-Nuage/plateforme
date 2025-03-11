#------------------------------------------------------------------------------
# BASE stage
#------------------------------------------------------------------------------
#
# This layer adds container dependencies that are reused across the stages,
# namely [turbo](https://turbo.build).
#
FROM node:22-alpine AS base

RUN npm install --global turbo

#------------------------------------------------------------------------------
# PRUNED stage
#------------------------------------------------------------------------------
#
# This stage copies over the project files and runs
# [turbo prune](https://turbo.build/repo/docs/reference/prune) for generating a
# *partial monorepo* for the current app.
#
FROM base AS pruned

WORKDIR /app

COPY . .

RUN turbo prune --scope=@france-nuage/control-plane --docker

#------------------------------------------------------------------------------
# INSTALL stage
#------------------------------------------------------------------------------
#
# Installs dependencies and prepares the codebase for development and build
# stages.
#
FROM base AS install

WORKDIR /app

# Copy the pruned dependency declarations as well as the docker optimizations
COPY --from=pruned /app/out/json/ .
COPY --from=pruned /app/out/package*.json /app/
COPY apps/control-plane/package.json /app/apps/control-plane/package.json

# Install the pruned dependency declarations
RUN --mount=type=cache,target=/root/.npm,sharing=locked npm ci

# Copy the pruned sources
COPY --from=pruned /app/out/full/ .
COPY turbo.json turbo.json

# Build all of the app dependencies, but not the app itself for development
RUN turbo run build --no-cache --filter="@france-nuage/control-plane^..."

#------------------------------------------------------------------------------
# DEVELOPMENT stage
#------------------------------------------------------------------------------
#
# Sets up a development environment with live reloading for the
# @france-nuage/control-plane application.
#
FROM base AS development

WORKDIR /app

COPY --from=install /app /app

CMD ["turbo", "run", "dev", "--filter", "@france-nuage/control-plane"]

#------------------------------------------------------------------------------
# BUILD stage
#------------------------------------------------------------------------------
#
# Builds the application, compiling all necessary assets and code
#
FROM base AS build

WORKDIR /app

# Copy the sources and dependencies from the install stage
COPY --from=install /app /app

# Build the application
RUN turbo run build --filter=@france-nuage/control-plane

#------------------------------------------------------------------------------
# RELEASE stage
#------------------------------------------------------------------------------
#
# Creates a minimal production image with only the necessary runtime files.
#
FROM node:22-alpine AS release

WORKDIR /app/apps/control-plane

# Copy only the necessary node_modules and build for runtime
COPY --from=build /app/apps/control-plane/build /app/apps/control-plane/build
COPY --from=build /app/packages /app/packages
COPY --from=build /app/node_modules /app/node_modules

EXPOSE 3000

USER node

CMD ["node", "build/bin/server.js"]

