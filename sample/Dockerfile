FROM rust:1.48.0-slim-buster

ENV CARGO_TARGET_DIR=/tmp/target \
    DEBIAN_FRONTEND=noninteractive \
    LC_CTYPE=ja_JP.utf8 \
    LANG=ja_JP.utf8

RUN apt update && apt install -y \
    locales \
    libpq-dev \
    gnupg \
    pkg-config \
    curl \
    build-essential \
    git \
    wget \
 && echo "ja_JP UTF-8" > /etc/locale.gen \
 && locale-gen 

WORKDIR /app

CMD [ "cargo", "run", "--example", "hello" ]