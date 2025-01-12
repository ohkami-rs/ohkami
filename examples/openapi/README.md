# Example Project for Ohkami's `openapi` feature

## Setup

- Recent Rust toolchain
- Node.js >= 22.6.0 ( for `--experimental-strip-types` )

## How to play

First, run the Ohkami app:

```sh
cargo run
```

Then you'll see `openapi.json` generated at the project root!

Now you can fetch the Ohkami in a type-safe way:

```sh
# (another terminal window)

cd client
npm install

# generate type-safe API client from openapi.json
npm run gen

# run client app to interact with Ohkami
npm run main
```
