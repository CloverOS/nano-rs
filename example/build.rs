use std::error::Error;

use utoipa::openapi::{ContactBuilder, InfoBuilder, ServerBuilder};

use nano_rs::axum::gen::gen_api_info::AxumGenApiInfo;
use nano_rs::axum::gen::gen_doc::AxumGenDoc;
use nano_rs::axum::gen::gen_route::AxumGenRoute;
use nano_rs_build::core::NanoBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    NanoBuilder::new(None)
        .gen_api_route(AxumGenRoute::new())
        .gen_api_doc(AxumGenDoc::new()
            .set_info(InfoBuilder::new()
                .title("Pet")
                .description(Some("Pet Api Server"))
                .terms_of_service(Some("https://example.com"))
                .contact(Some(ContactBuilder::new()
                    .name(Some("Pet"))
                    .email(Some("pet@gmail.com"))
                    .build()))
                .version("v1")
                .build())
            .add_server(ServerBuilder::new()
                .url("")
                .description(Some("dev"))
                .build())
            .add_server(ServerBuilder::new()
                .url("https://example.com")
                .description(Some("prod"))
                .build())
            .build())
        .gen_api_info(AxumGenApiInfo::new());
    Ok(())
}

