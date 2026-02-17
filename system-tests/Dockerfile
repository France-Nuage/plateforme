FROM registry.france-nuage.fr/library/node:22-trixie AS base

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# Build node-sdk first
WORKDIR /node-sdk
COPY ./node-sdk/package.json ./node-sdk/package-lock.json ./
RUN npm ci
COPY ./node-sdk .
RUN npm run build

# Build system-tests
WORKDIR /app
COPY ./system-tests/package.json ./system-tests/package-lock.json ./
RUN npm ci
COPY ./system-tests .

# Install Playwright browsers and their system dependencies
RUN npx playwright install --with-deps chromium

CMD ["npm", "run", "test:ui"]

