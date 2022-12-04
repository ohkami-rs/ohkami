FROM rust:1.65.0 as builder
ENV CARGO_TARGET_DIR=/tmp/target
WORKDIR /app
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./examples ./examples
RUN mkdir src \
 && touch ./src/main.rs \
 && echo 'fn main() {}' > ./src/main.rs \
 && cargo build --release \
 && rm src/main.rs
COPY ./src ./src
RUN cd ./examples/db/ \
 && cargo build --release --locked

FROM debian:buster-slim
ENV LC_CTYPE=ja_JP.utf8 \
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
COPY --from=builder /tmp/target/release/db .
CMD [ "./db" ]
