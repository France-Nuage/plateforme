# Build Stage
FROM node:22-alpine AS build

WORKDIR /usr/src/app

# Copy monorepo root files (package.json, package-lock.json, turbo.json)
COPY ./package.json ./package-lock.json ./turbo.json ./

# Copy workspace files (apps and packages)
COPY ./apps ./apps
#COPY ./packages ./packages

# Install all dependencies (including dev dependencies) in the build stage
RUN npm install

# Build the app (specific to the "api" app in the monorepo)
RUN npx turbo run build --filter=api

# Development Stage
FROM node:22-alpine AS development

RUN mkdir -p /usr/src/app/apps

WORKDIR /usr/src/app/apps/api

COPY ./apps/api/package*.json /usr/src/app/apps/api

RUN npm install

COPY ./apps/api /usr/src/app/apps/api

EXPOSE 3000

CMD ["npm", "run", "dev"]

# Production Stage
FROM node:22-alpine AS release

WORKDIR /usr/src/app

# Copy the root package.json and package-lock.json to the production image
COPY ./package.json ./package-lock.json ./

# Copy the app-specific package.json files (if needed for production dependencies)
COPY ./apps/api/package*.json ./apps/api/

# Copy the built files from the build stage
COPY --from=build /usr/src/app/apps/api/build ./build

# Install only production dependencies (omit dev dependencies)
RUN npm install --omit=dev --production --ignore-scripts

# Expose the API's port
EXPOSE 3000

# Use non-root user for better security
USER node

# Start the application
CMD ["node", "build/bin/server.js"]
