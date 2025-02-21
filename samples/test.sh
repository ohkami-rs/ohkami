#!/bin/bash

set -Ceu

SAMPLES=$(pwd)

cd $SAMPLES/openapi-schema-enums && \
    cargo run && \
    diff openapi.json openapi.json.sample
test $? -ne 0 && exit 150 || :

cd $SAMPLES/openapi-schema-from-into && \
    cargo run && \
    diff openapi.json openapi.json.sample
test $? -ne 0 && exit 151 || :

cd $SAMPLES/openapi-tags && \
    cargo run && \
    diff openapi.json openapi.json.sample
test $? -ne 0 && exit 152 || :

cd $SAMPLES/petstore && \
    cargo build && \
    cd client && \
        npm install && \
        cd .. && \
    (timeout -sKILL 5 cargo run &) && \
    sleep 1 && \
    diff openapi.json openapi.json.sample && \
    cd client && \
        npm run gen && \
        npm run main
# FIXME
# this is a little flaky; sometimes cause connection refused
test $? -ne 0 && exit 153 || :

cd $SAMPLES/readme-openapi && \
    cargo build && \
    (timeout -sKILL 1 cargo run &) && \
    sleep 1 && \
    diff openapi.json openapi.json.sample
test $? -ne 0 && exit 154 || :

cd $SAMPLES/realworld && \
    docker compose up -d && \
    sleep 5 && \
    sqlx migrate run && \
    cargo test && \
    docker compose down
test $? -ne 0 && exit 155 || :

cd $SAMPLES/streaming && \
    cargo build && \
    (timeout -sKILL 1 cargo run &) && \
    sleep 1 && \
    diff openapi.json openapi.json.sample
test $? -ne 0 && exit 156 || :

cd $SAMPLES/worker-bindings && \
    cargo check
test $? -ne 0 && exit 157 || :

cd $SAMPLES/worker-durable-websocket && \
    cargo check
test $? -ne 0 && exit 158 || :

cd $SAMPLES/worker-with-openapi && \
    cp wrangler.toml.sample wrangler.toml && \
    (test -f openapi.json || echo '{}' >> openapi.json) && \
    npm run openapi -- --skip-login && \
    diff openapi.json openapi.json.sample && \
    # FIXME : to generic way (this is a stopgap for testing
    # this functionality in at least my local env)
    (test $(whoami) = kanarus && \
        npm run openapi && \
        cp openapi.json.loggedin.sample tmp.json && \
        sed -i "s/{{ ACCOUNT_NAME }}/kanarus/" tmp.json && \
        diff openapi.json tmp.json \
        ; (test -f tmp.json && rm tmp.json) \
    || :)
test $? -ne 0 && exit 159 || :
