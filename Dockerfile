FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev cargo build-base

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY . .

RUN cargo build --release

FROM alpine:latest

RUN apk add --no-cache libgcc

RUN addgroup -S appgroup && adduser -S appuser -G appgroup

COPY --from=builder /usr/src/app/target/release/gmail_forwarder /usr/local/bin/gmail_forwarder

USER appuser

ENTRYPOINT ["/usr/local/bin/gmail_forwarder"]
