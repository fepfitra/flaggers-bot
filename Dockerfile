FROM rust:1.85-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM alpine:3.20

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/flaggers_bot /usr/local/bin/flaggers_bot

RUN mkdir -p /root/.config/flaggers_bot

ENV PATH="/usr/local/bin:${PATH}"

ENTRYPOINT ["flaggers_bot"]
CMD ["run"]
