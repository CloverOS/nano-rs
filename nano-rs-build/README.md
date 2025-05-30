## nano-rs-build

#### for Axum
- Add build.rs into your project
```rust
use std::error::Error;
use nano_rs_build::core::NanoBuilder;
use nano_rs::axum::generator::gen_route::AxumGenRoute;

fn main() -> Result<(), Box<dyn Error>> {
    NanoBuilder::new(None).gen_api_route(AxumGenRoute::new());
    Ok(())
}
```

more information please see [nano-rs](https://github.com/CloverOS/nano-rs)