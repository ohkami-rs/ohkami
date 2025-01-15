#!/bin/bash

set -Ceu

SAMPLES=$(pwd)

cd $SAMPLES/petstore && \
    cargo build && \
    (timeout -sKILL 5 cargo run &) && \
    cd client && \
    npm install && npm run gen && npm run main
test $? -ne 0 && exit 1 || :

cd $SAMPLES/readme-openapi && \
    cargo build && \
    (timeout -sKILL 1 cargo run &) && \
    diff -q openapi.json openapi.json.sample
test $? -ne 0 && exit 2 || :

cd $SAMPLES/realworld && \
    docker compose up -d && \
    sleep 5 && \
    sqlx migrate run && \
    cargo test && \
    docker compose down
test $? -ne 0 && exit 3 || :
