FROM alpine:3.22

RUN apk add --no-cache git curl

RUN curl -sSL "https://github.com/bufbuild/buf/releases/latest/download/buf-Linux-$(uname -m)" \
  -o /usr/local/bin/buf && \
  chmod +x /usr/local/bin/buf

WORKDIR /workspace

RUN git clone --depth 1 https://github.com/googleapis/googleapis.git googleapis

COPY . francenuage

CMD ["buf", "breaking", "googleapis", "--against=francenuage"]
