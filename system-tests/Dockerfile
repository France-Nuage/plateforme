FROM node:22-alpine AS base

RUN apk add ca-certificates

WORKDIR /app

COPY package*.json ./

RUN npm install

COPY . .

CMD ["npm", "run", "test:ui"]

