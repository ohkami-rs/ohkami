FROM rust:1.65.0 as builder
WORKDIR /app
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN touch ./src/main.rs \
 && echo 'fn main() {}' > ./src/main.rs \
 && cargo build --release \
 && rm src/*.rs
COPY ./src ./src
RUN cargo build --release --example db \
 && mv ./


FROM debian:buster-slim
ENV CARGO_TARGET_DIR=/tmp/target \
    LC_CTYPE=ja_JP.utf8 \
    LANG=ja_JP.utf8
RUN apt update && apt install -y \
    ca-certificates \
    locales \
    libpq-dev \
    gnupg \
    apt-transport-https\
    libssl-dev \
    pkg-config \
    curl \
    build-essential \
    git \
    wget \
 && echo "ja_JP UTF-8" > /etc/locale.gen \
 && locale-gen
COPY --from=builder /tmp/target/release/examples/db .
CMD [ "./db" ]
