FROM node:22-alpine AS base

RUN apk add ca-certificates

WORKDIR /node-sdk
COPY ./node-sdk .
RUN npm ci
RUN npm run build

WORKDIR /app
COPY ./system-tests/package.json ./system-tests/package-lock.json ./
RUN npm install
COPY ./system-tests .

CMD ["npm", "run", "test:ui"]

