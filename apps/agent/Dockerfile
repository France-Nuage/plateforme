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

# Build the app (specific to the "agent" app in the monorepo)
RUN npx turbo run build --filter=agent

# Production Stage
FROM node:22-alpine

WORKDIR /usr/src/app

# Copy the root package.json and package-lock.json to the production image
COPY ./package.json ./package-lock.json ./

# Copy the app-specific package.json files (if needed for production dependencies)
COPY ./apps/agent/package*.json ./apps/agent/

# Copy the built files from the build stage
COPY --from=build /usr/src/app/apps/agent/dist ./dist

# Install only production dependencies (omit dev dependencies)
RUN npm install --omit=dev --production


# Use non-root user for better security
USER node

# Start the application
CMD ["node", "dist/france-nuage-agent.js"]
