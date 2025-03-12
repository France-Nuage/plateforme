FROM node:22-alpine AS base

RUN apk add --no-cache bash curl firefox
RUN wget -O /usr/local/bin/wait-for-it.sh https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh \
    && chmod +x /usr/local/bin/wait-for-it.sh

WORKDIR /app

COPY package.json package-lock.json* ./

COPY packages/ ./packages/
COPY apps/tests/ ./apps/tests/

RUN npm ci
RUN npx playwright install --with-deps firefox

COPY <<EOF /app/entrypoint.sh
#!/bin/bash
set -e

# Function to check if a URL returns HTTP 200
check_http_status() {
  local url=\$1
  local max_attempts=10
  local attempt=1
  local wait_time=5

  echo "Checking if \$url is available..."
  
  while [ \$attempt -le \$max_attempts ]; do
    echo "Attempt \$attempt of \$max_attempts..."
    status_code=\$(curl -s -o /dev/null -w "%{http_code}" \$url || echo "failed")
    
    if [ "\$status_code" = "200" ]; then
      echo "\$url is available (HTTP 200)"
      return 0
    else
      echo "\$url returned status \$status_code, waiting \$wait_time seconds..."
      sleep \$wait_time
      attempt=\$((attempt + 1))
    fi
  done
  
  echo "ERROR: \$url did not become available after \$max_attempts attempts"
  return 1
}

# Wait for host:port using wait-for-it
if [ -n "\$WAIT_HOSTS" ]; then
  IFS=',' read -ra HOSTS <<< "\$WAIT_HOSTS"
  for host_port in "\${HOSTS[@]}"; do
    IFS=':' read -r host port <<< "\$host_port"
    echo "Waiting for \$host:\$port..."
    /usr/local/bin/wait-for-it.sh "\$host:\$port" -t 120
    echo "\$host:\$port is available"
  done
fi

# Additional health check for API and platform
if [ -n "\$API_URL" ]; then
  check_http_status "\$API_URL/health" || exit 1
fi

if [ -n "\$PLATFORM_URL" ]; then
  check_http_status "\$PLATFORM_URL/health" || exit 1
fi

echo "All services are ready, starting tests..."
exec npm run test
EOF

RUN chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/entrypoint.sh"]
