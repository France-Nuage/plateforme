FROM registry.france-nuage.fr/library/node:22-trixie AS base

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libnss3 libnspr4 libatk1.0-0t64 libatk-bridge2.0-0t64 libcups2t64 \
    libdrm2 libxkbcommon0 libxcomposite1 libxdamage1 libxfixes3 \
    libxrandr2 libgbm1 libpango-1.0-0 libcairo2 libasound2t64 \
    fonts-unifont fonts-liberation \
    && rm -rf /var/lib/apt/lists/*

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
RUN npx playwright install chromium

CMD ["npm", "run", "test:ci"]

