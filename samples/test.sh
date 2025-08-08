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
    cargo check && \
    wasm-pack build --target nodejs --dev --no-opt --no-pack --no-typescript && \
    node dummy_env_test.js
test $? -ne 0 && exit 157 || :

cd $SAMPLES/worker-bindings-jsonc && \
    cargo check
test $? -ne 0 && exit 158 || :

cd $SAMPLES/worker-durable-websocket && \
    cargo check
test $? -ne 0 && exit 159 || :

cd $SAMPLES/worker-with-global-bindings && \
    npm run openapi
test $? -ne 0 && exit 160 || :

cd $SAMPLES/worker-with-openapi && \
    cp wrangler.toml.sample wrangler.toml && \
    (test -f openapi.json || echo '{}' >> openapi.json) && \
    npm run openapi && \
    diff openapi.json openapi.json.sample && \
    sed -i -r 's/^#\[ohkami::worker.*]$/#[ohkami::worker({ title: "Ohkami Worker with OpenAPI", version: "0.1.1", servers: [] })]/' ./src/lib.rs && \
    npm run openapi && \
    diff openapi.json openapi.json.manual-title-version-empty_servers.sample && \
    sed -i -r 's/^#\[ohkami::worker.*]$/#[ohkami::worker({ title: "Ohkami Worker with OpenAPI", version: "0.1.2", servers: [{url: "https:\/\/example.example.workers.dev"}] })]/' ./src/lib.rs && \
    npm run openapi && \
    diff openapi.json openapi.json.manual-title-version-nonempty_servers.sample && \
    sed -i -r 's/^#\[ohkami::worker.*]$/#[ohkami::worker({servers: [{url: "https:\/\/example.example.workers.dev"}]})]/' ./src/lib.rs && \
    npm run openapi && \
    diff openapi.json openapi.json.manual-only_nonempty_servers.sample && \
    sed -i -r 's/^#\[ohkami::worker.*]$/#[ohkami::worker]/' ./src/lib.rs # reset to default
test $? -ne 0 && exit 161 || :
