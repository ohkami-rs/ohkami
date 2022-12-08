FROM rust:1.65.0 as builder
ENV CARGO_TARGET_DIR=/tmp/target
WORKDIR /app
COPY ./Cargo.toml ./ohkami/Cargo.toml
COPY ./Cargo.lock ./ohkami/Cargo.lock
COPY ./src ./ohkami/src
COPY ./examples/db/rust/Cargo.toml ./Cargo.toml
COPY ./examples/db/rust/Cargo.lock ./Cargo.lock
RUN mkdir ./src \
 && echo 'fn main() {}' > ./src/main.rs \
 && cargo build \
 && rm ./src/main.rs
COPY ./examples/db/rust/src/main.rs ./src/main.rs
RUN cargo build --locked --release

FROM debian:buster-slim
ENV LC_CTYPE=ja_JP.utf8 \
    LANG=ja_JP.utf8
WORKDIR /app
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
COPY --from=builder /tmp/target/release/db ./examle_server
CMD [ "./examle_server" ]