#!/bin/bash

set -Cue

HOST_TUPLE=(rustc -Vv | grep 'host' | sed 's/host: //')

CARGO_BUILD_TARGET="$HOST_TUPLE" cargo run --bin openapi
