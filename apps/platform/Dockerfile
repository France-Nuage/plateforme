FROM node:22-alpine AS build

WORKDIR /usr/src/app

# Copy monorepo root files
COPY package.json package-lock.json turbo.json ./

# Copy workspace files
COPY apps/platform ./apps/platform
COPY packages ./packages

# Install dependencies at root level
RUN npm install

# Install dependencies in platform directory
WORKDIR /usr/src/app/apps/platform
RUN npm install

# Return to root directory
WORKDIR /usr/src/app

# Build platform app
RUN npx turbo run build --filter=@france-nuage/platform

# Development Stage
FROM node:22-alpine AS development

WORKDIR /usr/src/app

COPY --from=build /usr/src/app ./

# Install dependencies at root level
RUN npm install

# Install dependencies in platform directory
WORKDIR /usr/src/app/apps/platform
RUN npm install

WORKDIR /usr/src/app

# Expose port
EXPOSE 3000
EXPOSE 39709

CMD ["npm", "run", "dev", "--workspace=apps/platform"]