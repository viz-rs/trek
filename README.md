# Trek

Fast, effective, minimalist web framework for Rust. Based on [Futures], [Hyper], [Tokio] or [Async-std].

[![CI Status](https://github.com/trek-rs/trek/workflows/ci/badge.svg)](https://github.com/trek-rs/trek/actions)

<!-- [![Build Status](https://travis-ci.org/trek-rs/trek.svg?branch=master)](https://travis-ci.org/trek-rs/trek) -->
<!-- [![Latest version](https://img.shields.io/crates/v/trek.svg)](https://crates.io/crates/trek) -->
<!-- [![Documentation](https://docs.rs/trek/badge.svg)](https://docs.rs/trek) -->
<!-- ![License](https://img.shields.io/crates/l/trek.svg) -->

## Features

## Hello Trek

```rust
#[macro_use]
extern crate log;

use trek::Trek;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let app = Trek::new();

    if let Err(e) = app.run("127.0.0.1:8000").await {
        error!("Error: {}", e);
    }
}
```

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

## Thanks

Some ideas from [Tide], [Actix], [Rocket], [Warp], [Phoenix], [Echo], [Gin]. Thanks for these excellent projects.

## BTW

Thanks to Brent Houghton <houghton.brent@gmail.com>. He is the first owner
of the [trek] package on [Crates.io].

[trek]: https://crates.io/crates/trek
[crates.io]: https://crates.io/
[futures]: https://rust-lang-nursery.github.io/futures-rs/
[hyper]: https://hyper.rs/
[tokio]: https://tokio.rs/
[async-std]: https://async.rs/
[tide]: https://github.com/http-rs/tide
[actix]: https://actix.rs/
[rocket]: https://rocket.rs/
[warp]: https://github.com/seanmonstar/warp
[phoenix]: https://phoenixframework.org/
[echo]: https://echo.labstack.com/
[gin]: https://gin-gonic.com/
