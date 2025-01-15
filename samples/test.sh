#!/bin/bash

set -Ceu

SAMPLES=$(pwd)

cd $SAMPLES/petstore && \
    cargo build && \
    timeout -sKILL 3 cargo run &
cd $SAMPLES/petstore/client && \
    sleep 1 && \
    npm install && npm run gen && npm run main
test $? -ne 0 && exit 1 || :

cd $SAMPLES/readme-openapi && \
    timeout -sKILL 1 cargo run &
cd $SAMPLES/readme-openapi && \
    sleep 1 && \
    diff -q openapi.json openapi.json.sample
test $? -ne 0 && exit 2 || :

cd $SAMPLES/realworld && \
    docker compose up -d && \
    sleep 5 && \
    sqlx migrate run && \
    cargo test && \
    docker compose down
test $? -ne 0 && exit 3 || :
