# Builder image
FROM rust:1.85.1-slim-bullseye as builder

RUN apt-get update && apt-get install -y \
    libudev-dev \
    clang \
    pkg-config \
    libssl-dev \
    build-essential \
    cmake \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release --bin jito-bell

# Final image
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/jito-bell /usr/local/bin/jito-bell

COPY jito_bell_config.yaml /etc/jito-bell/jito_bell_config.yaml

# Only one ENTRYPOINT line
ENTRYPOINT ["jito-bell"]

