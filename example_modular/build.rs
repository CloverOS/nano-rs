use std::error::Error;

use nano_rs::axum::generator::gen_api_info::AxumGenApiInfo;
use nano_rs::axum::generator::gen_doc::AxumGenDoc;
use nano_rs::axum::generator::gen_route::AxumGenRoute;
use nano_rs::core::NanoBuilder;
use utoipa::openapi::{ContactBuilder, InfoBuilder, ServerBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    NanoBuilder::new(None)
        .gen_api_route(AxumGenRoute::new())
        .gen_api_doc(
            AxumGenDoc::new()
                .set_info(
                    InfoBuilder::new()
                        .title("Modular Pet")
                        .description(Some("Modular nano-rs example"))
                        .terms_of_service(Some("https://example.com"))
                        .contact(Some(
                            ContactBuilder::new()
                                .name(Some("Modular Team"))
                                .email(Some("modular@example.com"))
                                .build(),
                        ))
                        .version("v1")
                        .build(),
                )
                .add_server(
                    ServerBuilder::new()
                        .url("http://127.0.0.1:8899")
                        .description(Some("dev"))
                        .build(),
                )
                .build(),
        )
        .gen_api_info(AxumGenApiInfo::new());
    Ok(())
}
