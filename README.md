<div align="center">
    <h1>ohkami</h1>
</div>

ohkami *- [狼] means wolf in Japanese -* is **simple** and **non macro-based** web framework for Rust.

<br/>

## Features
- *simple*: Less things to learn / Less code to write / Less time to hesitate.
- *non macro-based*: No need for using macros.

<br/>

## Quick start
1. Add dependencies:

```toml
[dependencies]
ohkami = { git = "https://github.com/kana-rus/ohkami", version = "0.1" }
# publishing to crates.io is in progress
```

2. Write your first code with ohkami:

```rust
use ohkami::prelude::*;

fn main() -> Result<()> {
    Server::setup()
        .GET("/", |_| async {Response::OK(Body::text("Hello, world!"))})
        .serve_on(":3000")
}
```

3. If you're interested in okami, learn more by [examples](https://github.com/kana-rus/ohkami/tree/main/examples) and documents(**WIP**)!

<br/>

## Development
ohkami is on very early stage now. Please don't use for any produntions.

<br/>

## License
This project is under MIT LICENSE ([LICENSE-MIT](https://github.com/kana-rus/ohkami/blob/main/LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)).
