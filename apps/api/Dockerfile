# Build Stage
FROM node:20-alpine AS build

WORKDIR /usr/src/app

# Copy monorepo root files (package.json, package-lock.json, turbo.json)
COPY ./package.json ./package-lock.json ./turbo.json ./

# Copy workspace files (apps and packages)
COPY ./apps ./apps
COPY ./packages ./packages

# Install all dependencies (including dev dependencies) in the build stage
RUN npm install

# Build the app (specific to the "api" app in the monorepo)
RUN npx turbo run build --filter=api

# Production Stage
FROM node:20-alpine

WORKDIR /usr/src/app

# Copy the root package.json and package-lock.json to the production image
COPY ./package.json ./package-lock.json ./

# Copy the app-specific package.json files (if needed for production dependencies)
COPY ./apps/api/package*.json ./apps/api/

# Copy the built files from the build stage
COPY --from=build /usr/src/app/apps/api/build ./build

# Install only production dependencies (omit dev dependencies)
RUN npm install --omit=dev --production

# Expose the API's port
EXPOSE 3000

# Use non-root user for better security
USER node

# Start the application
CMD ["node", "build/bin/server.js"]
