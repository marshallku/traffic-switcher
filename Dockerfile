FROM rust:1.80-alpine AS base

WORKDIR /usr/src/traffic_switcher

RUN set -eux; \
    apk add --no-cache musl-dev pkgconfig libressl-dev; \
    rm -rf $CARGO_HOME/registry

COPY Cargo.* .

RUN mkdir src && \
    echo 'fn main() {println!("Hello, world!");}' > src/main.rs && \
    cargo build --release && \
    rm target/release/traffic_switcher* && \
    rm target/release/deps/traffic_switcher* && \
    rm -rf src

FROM base AS builder

COPY src src
RUN cargo build --release

FROM alpine:3.20.2

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/traffic_switcher/target/release/traffic_switcher .

EXPOSE ${PORT}

CMD ["./traffic_switcher"]