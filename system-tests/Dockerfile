FROM node:22-alpine AS base
WORKDIR /app
COPY . .
RUN npm install
CMD ["npm", "run", "test:ui"]

