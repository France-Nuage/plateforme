FROM node:22-alpine AS base

RUN apk add --no-cache bash curl firefox

WORKDIR /app

COPY package.json package-lock.json* ./

COPY packages/ ./packages/
COPY apps/tests/ ./apps/tests/

RUN npm ci

# Install Firefox browser for Playwright
RUN npx playwright install firefox

CMD ["tail", "-f", "/dev/null"]
