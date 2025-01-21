#!/bin/bash

set -Ceu

SAMPLES=$(pwd)

cd $SAMPLES/petstore && \
    cargo build && \
    cd client && \
        npm install && \
        cd .. && \
    (timeout -sKILL 5 cargo run &) && \
    sleep 1 && \
    cd client && \
        npm run gen && \
        npm run main
test $? -ne 0 && exit 151 || :

cd $SAMPLES/readme-openapi && \
    cargo build && \
    (timeout -sKILL 1 cargo run &) && \
    sleep 1 && \
    diff -q openapi.json openapi.json.sample
test $? -ne 0 && exit 152 || :

cd $SAMPLES/realworld && \
    docker compose up -d && \
    sleep 5 && \
    sqlx migrate run && \
    cargo test && \
    docker compose down
test $? -ne 0 && exit 153 || :

cd $SAMPLES/worker-with-openapi && \
    cp wrangler.toml.sample wrangler.toml && \
    (test -f openapi.json || echo '{}' >> openapi.json) && \
    npm run openapi
test $? -ne 0 && exit 154 || :
