#!/bin/bash

set -Cue

EXAMPLES=$(pwd)

cd $EXAMPLES/static_files && \
    cargo test

cd $EXAMPLES/jwt && \
    cp .env.sample .env
cd $EXAMPLES/jwt && \
    cargo test 2>&1 | grep 'Unexpected end of headers' \
    && echo '---> expected error' \
    || exit 1
cd $EXAMPLES/jwt && \
    OHKAMI_REQUEST_BUFSIZE=4096 cargo test

cd $EXAMPLES/html_layout && \
    cargo build && \
    (timeout -sKILL 5 cargo run &) && \
    sleep 1 && \
    CONTENT_TYPE_COUNT=$(curl -i 'http://localhost:5555' 2>&1 | grep -i 'content-type' | wc -l) && \
    if [ $CONTENT_TYPE_COUNT -eq 1 ]; then
        echo '---> ok'
    else
        echo '---> multiple content-type headers found (or something else went wrong)'
        exit 1
    fi
