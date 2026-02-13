FROM alpine:3.23

# Define a fixed version and sha256 to avoid supply chain attacks

ENV BUF_VERSION=v1.56.0
ENV BUF_SHA256=d0e36e933cf5340e34c4184112a489337a03e0b57302fa20bad980a90e600b96

# Configure Alpine mirrors for better reliability
RUN echo "https://dl-cdn.alpinelinux.org/alpine/v3.23/main" > /etc/apk/repositories && \
    echo "https://dl-cdn.alpinelinux.org/alpine/v3.23/community" >> /etc/apk/repositories && \
    apk update --no-cache || (echo "http://dl-4.alpinelinux.org/alpine/v3.23/main" > /etc/apk/repositories && \
    echo "http://dl-4.alpinelinux.org/alpine/v3.23/community" >> /etc/apk/repositories && apk update --no-cache)

RUN apk add --no-cache git curl

RUN curl -sSL "https://github.com/bufbuild/buf/releases/download/${BUF_VERSION}/buf-Linux-x86_64" \
  -o /usr/local/bin/buf && \
  echo "${BUF_SHA256}  /usr/local/bin/buf" | sha256sum -c - && \
  chmod +x /usr/local/bin/buf

WORKDIR /workspace

RUN git clone --depth 1 https://github.com/googleapis/googleapis.git googleapis

COPY . francenuage

CMD ["buf", "breaking", "googleapis", "--against=francenuage"]
