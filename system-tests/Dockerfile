FROM node:22-alpine AS base

RUN apk add --no-cache ca-certificates

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

CMD ["npm", "run", "test:ui"]

