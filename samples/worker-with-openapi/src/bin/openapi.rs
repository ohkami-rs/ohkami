//! ```sh
//! CARGO_BUILD_TARGET=(rustc -vV | grep 'host' | sed 's/host: //') cargo run --bin openapi
//! ```

#![cfg(feature="openapi")]

use ohkami::openapi;

fn main() {
    worker_with_openapi::ohkami().spit_out(
        openapi::OpenAPI::json(
            "WorkerWithOpenAPI Server", "0.1.0", [
                openapi::Server::at("http://localhost:8787")
                    .description("dev (miniflare) URL"),
                openapi::Server::at("https://worker-with-openapi.kanarus.workers.dev")
                    .description("production URL"),
            ]
        )
    )
}
