FROM node:22-alpine AS base

RUN apk add --no-cache bash curl
RUN wget -O /usr/local/bin/wait-for-it.sh https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh \
    && chmod +x /usr/local/bin/wait-for-it.sh

WORKDIR /app

COPY package.json package-lock.json* ./

COPY packages/ ./packages/
COPY apps/tests/ ./apps/tests/

RUN npm ci

COPY <<EOF /app/entrypoint.sh
#!/bin/bash
set -e

if [ -n "\$WAIT_HOSTS" ]; then
  IFS=',' read -ra HOSTS <<< "\$WAIT_HOSTS"
  for host_port in "\${HOSTS[@]}"; do
    IFS=':' read -r host port <<< "\$host_port"
    echo "Waiting for \$host:\$port..."
    /usr/local/bin/wait-for-it.sh "\$host:\$port" -t 60
    echo "\$host:\$port is available"
  done
fi

exec npm run test
EOF

RUN chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/entrypoint.sh"]
