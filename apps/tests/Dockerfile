FROM mcr.microsoft.com/playwright:v1.50.1-noble

WORKDIR /app/apps/tests

# Copy package.json and turbo.json files for dependency installation
COPY package.json turbo.json ./
COPY apps/tests/package.json ./apps/tests/

# Install dependencies
RUN npm install

# Copy the test files
COPY apps/tests/ ./apps/tests/

# Define a health check to indicate when the container is ready
HEALTHCHECK --interval=5s --timeout=5s --start-period=5s --retries=3 \
  CMD ["node", "-e", "process.exit(0)"]

# Set command to wait for other services to be healthy before running tests
CMD ["sh", "-c", "echo 'Waiting for services to be healthy...' && sleep 5 && npm run test"]