use std::error::Error;
use nano_rs_build::core::NanoBuilder;
use nano_rs::axum::gen::gen_route::AxumGenRoute;

fn main() -> Result<(), Box<dyn Error>> {
    NanoBuilder::new(None).gen_api_route(AxumGenRoute::new());
    Ok(())
}
