[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]

<div align="center">
  <a href="https://github.com/CloverOS/nano-rs">
    <img src="images/logo.png" alt="Logo" width="120" height="120">
  </a>

  [中文文档](README_cn.md)
</div>

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;nano-rs is a lightweight, non-invasive, convention-over-configuration Rust Web service component, aimed at providing a fast and efficient development experience. By reducing the burden of configuration, it allows you to focus more on implementing business logic.

---

- **Lightweight**: Ensures fast startup and low resource consumption through a streamlined design.
- **Non-invasive**: The framework's design allows for seamless integration into existing Rust projects without fear of interfering with business logic.
- **Convention Over Configuration**: The framework adopts an "intelligent defaults" approach, providing pre-set configurations for common scenarios. This means you can get started with little to no configuration, yet it still offers ample configuration options to suit specific needs ensuring ultimate flexibility and extensibility.
- **Business Focused**: Our goal is to simplify the development process of Web services. By reducing the tedious and miscellaneous configuration tasks, you can concentrate more on implementing the business logic, thus improving the efficiency and quality of project development.

<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#environment-requirements">Environment Requirements</a>
    </li>
    <li><a href="#installation">Installation</a></li>
    <li><a href="#quick-start">Quick Start (Axum)</a>
    <ol>
        <li><a href="#router-auto-gen">RouterAutoGen</a></li>
        <li><a href="#openapi-generation">OpenApi Generation(for utoipa)</a></li>
        <li><a href="#apiinfo-generation">ApiInfo Generation</a></li>
        <li><a href="#logging-and-request-tracing">Logging and Request Tracing</a></li>
        <li><a href="#modular-example">Modular Example</a></li>
    </ol>
    </li>
    <li><a href="#others">Others</a>
      <ol>
         <li><a href="#seaorm-postgresql-doc-generation">SeaOrm Postgresql Doc Generation</a></li>
      </ol>
    </li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>

### Environment Requirements

MSRV >= 1.94

### Installation

```shell
cargo add nano-rs
```

This installs the latest release published on crates.io. To use features available on the current development line before the next release, depend on `main`:

```shell
cargo add nano-rs --git https://github.com/CloverOS/nano-rs --branch main
```

## Quick Start

### Router Auto Gen

- Add build dependencies

```toml
[build-dependencies]
nano-rs = { git = "https://github.com/CloverOS/nano-rs", branch = "main" }
nano-rs-build = { git = "https://github.com/CloverOS/nano-rs", branch = "main" }
```

- Add gen component build.rs

```rust
use std::error::Error;
use nano_rs::axum::generator::gen_route::AxumGenRoute;
use nano_rs::core::NanoBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    NanoBuilder::new(None).gen_api_route(AxumGenRoute::new());
    Ok(())
}
```

- Add the configuration file to your desired directory (in the example, it is placed
  in [etc/config.yaml](https://github.com/CloverOS/nano-rs/blob/main/example/etc/config.yaml))

```yaml
port: 8888
name: example
host: 127.0.0.1
```

- Write your API code anywhere in project with marco (for example, under api/pet), for macros, please refer
  to [example](https://github.com/CloverOS/nano-rs/blob/main/example/src/api)

```rust
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::rest::RestResp;
use nano_rs::{biz_ok, get};

#[get(path = "/store/name", layers = ["crate::layers::auth::auth_token1"])]
pub async fn get_store_name() -> Result<RestResp<String>, ServerError> {
    biz_ok!("Doggy Store".to_string())
}
```

- Build the project. Cargo reruns `build.rs` when relevant inputs change.

```shell
cargo build
```

- Then you will get file named `routes.rs` in your `src/`.
- Do not edit `routes.rs` as it will be overwritten every time you build.
- Edit `main.rs`. (refer to the project structure in the example)

```rust
use axum::Router;
use axum_client_ip::ClientIpSource;
use nano_rs::axum::start::AppStarter;
use nano_rs::config::init_config_with_cli;
use nano_rs::config::rest::RestConfig;

#[tokio::main]
async fn main() {
    let rest_config = init_config_with_cli::<RestConfig>();
    let _guards = nano_rs::tracing::init_tracing(&rest_config);
    let log_config = rest_config.log.clone();

    AppStarter::new(Router::new(), rest_config)
        .add_log_layer_with_config(Some(log_config))
        // When a trusted reverse proxy sets X-Real-IP, use this source.
        .add_secure_client_ip_source_layer(ClientIpSource::XRealIp)
        .run()
        .await;
}
```

- Run your web application

```shell
cargo run -- --config etc/config.yaml
```

- After this, you only need to focus on writing your business logic code; nano-rs will automatically generate routes and register them with axum, allowing you
  to concentrate solely on implementing business logic.

### OpenApi Generation

- Add build dependencies

```toml
[build-dependencies]
nano-rs = { git = "https://github.com/CloverOS/nano-rs", branch = "main", features = ["utoipa_axum"] }
nano-rs-build = { git = "https://github.com/CloverOS/nano-rs", branch = "main" }
utoipa = { version = "5.5.0", features = ["axum_extras"] }
```

- Add gen component to build.rs

```rust
use std::error::Error;
use nano_rs::axum::generator::gen_doc::AxumGenDoc;
use nano_rs::axum::generator::gen_route::AxumGenRoute;
use nano_rs::core::NanoBuilder;
use utoipa::openapi::{ContactBuilder, InfoBuilder, ServerBuilder};

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
            .build());
    Ok(())
}
```

- OpenAPI supports **both** styles, and old `#[utoipa::path(...)]` code remains fully compatible.
- Legacy (still supported): write full utoipa annotations. See [utoipa](https://github.com/juhaku/utoipa/tree/master/examples/todo-axum).

```rust
/// Get pet by id
#[utoipa::path(
    get,
    path = "/store/pet",
    tag = "Store",
    params(QueryPet),
    responses((status = 200, body = Pet))
)]
#[get()]
pub async fn get_query_pet_name(Query(query): Query<QueryPet>) -> Result<RestResp<Pet>, ServerError> { ... }
```

- New simplified style (recommended): use `#[get]/#[post]` directly, and nano-rs infers OpenAPI from extractor/response signatures.

```rust
/// Query pet by id
#[get(path = "/store/pet", tag = "Store")]
pub async fn get_query_pet_name(
    Query(query): Query<QueryPet>,
) -> Result<RestResp<Pet>, ServerError> { ... }
```

- Module-level default tag (to avoid repeated `tag = ...` on every handler):

```rust
/// @tag Store
pub mod store;
```

- With module-level `/// @tag ...`, handlers in that module can omit `tag`.
- For nested modules, handlers inherit the nearest ancestor module tag unless the child module declares its own `/// @tag ...`.
- A route macro must provide `path = "..."` unless the handler also has `#[utoipa::path(...)]`; in the legacy form, the utoipa path is used for route
  generation.
- If both annotations provide a path, the paths must be identical. Other utoipa metadata is retained, but a path mismatch fails the build with an explicit
  error.
- Build the project. Cargo reruns `build.rs` when relevant inputs change.

```shell
cargo build
```

- Then you will get file named `doc.rs` in your `src/`.
- Do not edit `doc.rs` as it will be overwritten every time you build.
- Now you can use `doc.rs` to generate openapi document, see [example](https://github.com/CloverOS/nano-rs/blob/main/example/src/main.rs)

- Run your web application

```shell
cargo run -- --config etc/config.yaml
```

### ApiInfo Generation

- Add build dependencies

```toml
[build-dependencies]
nano-rs = { git = "https://github.com/CloverOS/nano-rs", branch = "main" }
nano-rs-build = { git = "https://github.com/CloverOS/nano-rs", branch = "main" }
```

- Add build.rs

```rust
use std::error::Error;

use nano_rs::axum::generator::gen_api_info::AxumGenApiInfo;
use nano_rs::axum::generator::gen_route::AxumGenRoute;
use nano_rs::core::NanoBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    NanoBuilder::new(None)
        .gen_api_route(AxumGenRoute::new())
        .gen_api_info(AxumGenApiInfo::new());
    Ok(())
}
```

- This generates `api_info.rs` in `src/`; use `get_api_info()` to access the collected endpoint metadata.
- `routes.rs`, `doc.rs`, and `api_info.rs` are generated files. Do not edit them by hand because a later build can overwrite them.

### Logging and Request Tracing

Logging is configured per level under `log.level`:

```yaml
log:
  enable_request_body_log: true
  enable_response_body_log: false
  ignore_resource:
    - method: GET
      path: /pets/{id}
  level:
    info:
      file: true
      dir: logs
      max_files: 7
    error:
      file: true
      dir: logs
      max_files: 30
```

- Files roll daily. `max_files` is applied independently to each log level and includes the current file; omit it or set it to `0` to disable pruning.
- The request tracing middleware reuses a non-empty incoming `x-request-id`, otherwise generates a UUID, and writes the selected ID to the response header and
  request log.
- `ignore_resource` matches the configured HTTP method and either the concrete request path or the Axum route template, such as `/pets/{id}`.
- When response-body logging is enabled, SSE streams and WebSocket upgrades bypass body buffering so streaming and upgrades are not blocked.
- Use `nano_rs::tracing::init_tracing_with_layers` with `nano_rs::tracing::DynTracingLayer` values to attach custom tracing subscriber layers while retaining
  the built-in logging configuration.

### Modular Example

See [`example_modular`](https://github.com/CloverOS/nano-rs/tree/main/example_modular) for route and OpenAPI generation across separate API, model, and layer
crates.

## Others

### SeaOrm Postgresql Doc Generation
- Cause seaorm does not support postgresql doc generation, so we provide a way to generate it.
- Add build dependencies
```toml
[build-dependencies]
nano-rs-extra = { git = "https://github.com/CloverOS/nano-rs", branch = "main", features = ["postgres"] }
tokio = { version = "1.53.1", features = ["full"] }
```
- Add build.rs
```rust
use std::error::Error;

use nano_rs_extra::sea_orm::postgres::gen_comments::GenComments;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let database_url = "postgres://test:test@localhost/test".to_string();
    let database = "test".to_string();
    let schema = None;
    GenComments::new(None, database_url, database, schema).gen_comments().await?;
    Ok(())
}
```

- Build the project whenever the entity definitions or database comments change.
- It will inject doc into your entity field from your postgresql database when you already make doc in field.
- Before build
```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::fruit::Entity")]
    Fruit,
}

impl Related<super::fruit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Fruit.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

- After build
```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    /// cake id
    pub id: i32,
    /// cake name
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::fruit::Entity")]
    Fruit,
}

impl Related<super::fruit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Fruit.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```


## Roadmap

- [x] Auto-generate Axum framework routes
- [x] Default configuration for Tracing framework
- [x] Preset common web service configuration (managed via yaml)
- [x] Auto-generate OpenApi (gen [utoipa](https://github.com/juhaku/utoipa) struct)

For a full list of proposed features (and known issues), please see the [open issues](https://github.com/CloverOS/nano-rs/issues).

## License

Distributed under the MIT License. For more information, see `LICENSE.txt`.

## Contact

- a527756694@gmail.com

## Acknowledgments

* [Anyhow](https://github.com/dtolnay/anyhow)
* [Axum](https://github.com/tokio-rs/axum)
* [Clap](https://github.com/clap-rs/clap)
* [Tokio](https://github.com/tokio-rs/tokio)
* [Tracing](https://github.com/tokio-rs/tracing)
* [utoipa](https://github.com/juhaku/utoipa)

[contributors-shield]: https://img.shields.io/github/contributors/CloverOS/nano-rs.svg?style=for-the-badge

[contributors-url]: https://github.com/CloverOS/nano-rs/graphs/contributors

[forks-shield]: https://img.shields.io/github/forks/CloverOS/nano-rs.svg?style=for-the-badge

[forks-url]: https://github.com/CloverOS/nano-rs/network/members

[stars-shield]: https://img.shields.io/github/stars/CloverOS/nano-rs.svg?style=for-the-badge

[stars-url]: https://github.com/CloverOS/nano-rs/stargazers

[issues-shield]: https://img.shields.io/github/issues/CloverOS/nano-rs.svg?style=for-the-badge

[issues-url]: https://github.com/CloverOS/nano-rs/issues
