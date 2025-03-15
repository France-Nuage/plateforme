#------------------------------------------------------------------------------
# BASE stage
#------------------------------------------------------------------------------
#
# This layer adds container dependencies that are reused across the stages,
# namely [turbo](https://turbo.build).
#
FROM node:22-alpine AS base

RUN apk add --no-cache wget bash curl firefox
RUN npm install --global turbo
RUN npx playwright install firefox


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

RUN npm ci

RUN turbo prune --scope=@france-nuage/tests --docker


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
COPY apps/tests/package.json /app/apps/tests/package.json

# Install the pruned dependency declarations
RUN --mount=type=cache,target=/root/.npm,sharing=locked npm ci

# Copy the pruned sources
COPY --from=pruned /app/out/full/ .
COPY turbo.json turbo.json

# Build all of the app dependencies, but not the app itself for development
RUN turbo run build --no-cache --filter="@france-nuage/tests^..."


#------------------------------------------------------------------------------
# PRODUCTION stage
#------------------------------------------------------------------------------
#
# Sets up a development environment with live reloading for the
# @france-nuage/platform application.
#
FROM base AS production

WORKDIR /app

COPY --from=install /app /app

CMD ["tail", "-f", "/dev/null"]


