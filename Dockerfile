FROM alpine:3.20

RUN apk add --no-cache ca-certificates curl

WORKDIR /app

ARG VERSION=latest

RUN if [ "$VERSION" = "latest" ]; then \
      VERSION=$(curl -sL "https://api.github.com/repos/fepfitra/flaggers-bot/releases/latest" | grep -oP '"tag_name":\s*"\K[^"]+'); \
    fi && \
    curl -sL "https://github.com/fepfitra/flaggers-bot/releases/download/${VERSION}/flaggers_bot-linux-x86_64" -o /usr/local/bin/flaggers_bot && \
    chmod +x /usr/local/bin/flaggers_bot

RUN mkdir -p /root/.config/flaggers_bot

ENTRYPOINT ["flaggers_bot"]
CMD ["run"]
