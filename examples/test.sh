#!/bin/bash

set -Cue

EXAMPLES=$(pwd)

# First, check the buildability of all examples
for directory in ./*/; do
    if [ "$(basename $directory)" != "target" ]; then
        cd $directory
        cargo check
        cd ..
    fi
done

# Now, additionally run tests for each example if needed

cd $EXAMPLES/static_files && \
    cargo test

cd $EXAMPLES/jwt && \
    cp .env.sample .env && \
    cargo test

cd $EXAMPLES/uibeam && \
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
