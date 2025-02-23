FROM node:22-alpine AS build

WORKDIR /usr/src/app

# Copy monorepo root files
COPY package.json package-lock.json turbo.json ./

# Copy workspace files
COPY apps ./apps
COPY packages ./packages

# Install all dependencies
RUN npm install

# Build platform app
RUN npx turbo run build --filter=platform

# Development Stage
FROM node:22-alpine AS development

WORKDIR /usr/src/app

# Copy all files from build stage
COPY --from=build /usr/src/app ./

# Expose port
EXPOSE 3000
EXPOSE 39709

# Start development server
CMD ["npm", "run", "dev", "--workspace=apps/platform"]

# Production Stage
FROM node:22-alpine AS production

WORKDIR /usr/src/app

# Copy built app and dependencies
COPY --from=build /usr/src/app/apps/platform/.output ./.output
COPY --from=build /usr/src/app/package*.json ./

# Install production dependencies only
ENV NODE_ENV=production
RUN npm install --production

# Set environment variables
ENV NUXT_HOST=0.0.0.0
ENV PORT=3000

# Expose ports
EXPOSE 3000
EXPOSE 39709

# Start production server
CMD ["node", ".output/server/index.mjs"]