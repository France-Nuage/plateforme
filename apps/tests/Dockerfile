FROM node:22-alpine AS base

RUN apk add --no-cache bash curl firefox
RUN wget -O /usr/local/bin/wait-for-it.sh https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh \
    && chmod +x /usr/local/bin/wait-for-it.sh

WORKDIR /app

COPY package.json package-lock.json* ./

COPY packages/ ./packages/
COPY apps/tests/ ./apps/tests/

RUN npm ci

# Install Firefox browser for Playwright
RUN npx playwright install firefox

CMD ["tail", "-f", "/dev/null"]
