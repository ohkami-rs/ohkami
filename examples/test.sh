#!/bin/bash

set -Cue

EXAMPLES=$(pwd)

cd $EXAMPLES/jwt && \
    cp .env.sample .env
cd $EXAMPLES/jwt && \
    cargo test 2>&1 | grep 'Unexpected end of headers' \
    && echo '---> expected error' \
    || exit 1
cd $EXAMPLES/jwt && \
    OHKAMI_REQUEST_BUFSIZE=4096 cargo test
