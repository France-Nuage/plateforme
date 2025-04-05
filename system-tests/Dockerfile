FROM node:22-alpine AS base
WORKDIR /app
COPY . .
RUN npm install
CMD ["npx", "playwright", "test", "--ui-host=0.0.0.0", --ui-port=9000"]

