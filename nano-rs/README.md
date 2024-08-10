[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]

<div align="center">
  <a href="https://github.com/CloverOS/nano-rs">
    <img src="https://github.com/CloverOS/nano-rs/raw/main/images/logo.png" alt="Logo" width="120" height="120">
  </a>

</div>

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;nano-rs is a lightweight, non-invasive, convention-over-configuration Rust Web service component, aimed at
providing a fast and efficient development experience. By reducing the burden of configuration, it allows you to focus more on implementing business logic.

---

- **Lightweight**: Ensures fast startup and low resource consumption through a streamlined design.
- **Non-invasive**: The framework's design allows for seamless integration into existing Rust projects without fear of interfering with business logic.
- **Convention Over Configuration**: The framework adopts an "intelligent defaults" approach, providing pre-set configurations for common scenarios. This means
  you can get started with little to no configuration, yet it still offers ample configuration options to suit specific needs ensuring ultimate flexibility and
  extensibility.
- **Business Focused**: Our goal is to simplify the development process of Web services. By reducing the tedious and miscellaneous configuration tasks, you can
  concentrate more on implementing the business logic, thus improving the efficiency and quality of project development.

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
    </ol>
    </li>
    <li><a href="others">Others</a>
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

MSRV >= 1.66

### Installation

```shell
  cargo add nano-rs
```

## Quick Start

### Router Auto Gen

- Add build dependencies

```toml
[build-dependencies]
nano-rs = "0.1.3"
nano-rs-build = "0.1.2"
```

- Add gen component build.rs

```rust
use std::error::Error;
use nano_rs_build::core::NanoBuilder;
use nano_rs::axum::gen::gen_route::AxumGenRoute;

fn main() -> Result<(), Box<dyn Error>> {
    NanoBuilder::new(None).gen_api_route(AxumGenRoute::new());
    Ok(())
}
```

- Add the configuration file to your desired directory (in the example, it is placed
  in [etc/config.yaml](https://github.com/CloverOS/nano-rs/blob/master/example/etc/config.yaml))

```yaml
port: 8888
name: example
host: 127.0.0.1
```

- Write your API code anywhere in project with marco (for example, under api/pet), for macros, please refer
  to [example](https://github.com/CloverOS/nano-rs/blob/master/example/src/api)

```rust
#[get(path = "/store/name", layers = ["crate::layers::auth::auth_token1"])]
pub async fn get_store_name() -> Result<RestResp<String>, ServerError> {
    biz_ok("Doggy Store".to_string())
}
```

- Run build once (only needed for the project's first compilation)

```shell
cargo build
```

- Then you will get file named `routes.rs` in your `src/`.
- Do not edit `routes.rs` as it will be overwritten every time you build.
- Edit `main.rs`. (refer to the project structure in the example)

```rust
use axum::Router;
use nano_rs_core::config::rest::RestConfig;
use axum_client_ip::SecureClientIpSource;
use nano_rs_extra::axum::start::AppStarter;

#[tokio::main]
async fn main() {
  let rest_config = nano_rs_core::config::init_config_with_cli::<RestConfig>();
  let _guards = nano_rs_core::tracing::init_tracing(&rest_config);
  let service_context = ServiceContext {
    rest_config: rest_config.clone(),
  };
  let app = Router::new();
  AppStarter::new(app, rest_config)
      .add_log_layer()
      ///if use nginx proxy,you can use SecureClientIpSource::XRealIp
      .add_secure_client_ip_source_layer(SecureClientIpSource::XRealIp)
      .run()
      .await;
}

#[derive(Clone)]
pub struct ServiceContext {
  pub rest_config: RestConfig,
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
nano-rs = "0.1.3"
nano-rs-build = "0.1.2"
utoipa = { version = "4.2.3", features = ["axum_extras"] }
```

- Add gen component to build.rs

```rust
use std::error::Error;
use nano_rs_build::core::NanoBuilder;
use nano_rs::axum::gen::gen_route::AxumGenRoute;

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

- Write utoipa code,see [example](https://github.com/CloverOS/nano-rs/blob/main/example/src/api/pet/store.rs),more document please refer
  to [utoipa](https://github.com/juhaku/utoipa/tree/master/examples/todo-axum)

```rust
/// Get pet by id
#[utoipa::path(
    get,
    path = "/store/pet",
    tag = "Store",
    params(QueryPet),
    responses(
        (status = 200, body = Pet)
    )
)]
#[get(path = "/store/pet", group = "Store")]
pub async fn get_query_pet_name(Query(query): Query<QueryPet>) -> Result<RestResp<Pet>, ServerError> {
    biz_ok(Pet {
        id: query.id,
        name: "Doggy".to_string(),
        tag: None,
        inline: None,
        meta: Meta { name: "Doggy".to_string(), age: 1 },
    })
}
```

- Run build once (only needed for the project's first compilation)

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
nano-rs = "0.1.3"
nano-rs-build = "0.1.2"
```

- Add build.rs

```rust
fn main() -> Result<(), Box<dyn Error>> {
    NanoBuilder::new(None)
        .gen_api_route(AxumGenRoute::new())
        .gen_api_info(AxumGenApiInfo::new());
    Ok(())
}
```

- This will gen `api_info.rs` in your `src/` for collect your all api info,and you can use `get_api_info()` to get all api info.

### SeaOrm Postgresql Doc Generation
- Cause seaorm does not support postgresql doc generation, so we provide a way to generate it.
- Add build dependencies
```toml
[build-dependencies]
nano-rs-extra = { version = "0.1.4" }
tokio = { version = "1.34.0", features = ["full"] }
```
- Add build.rs
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let database_url = "postgres://test:test@localhost/test".to_string();
    let database = "test".to_string();
    let schema = None;
    GenComments::new(None, database_url, database, schema).gen_comments().await?;
    Ok(())
}
```
- Run build once (only needed for the project's first compilation)
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

For a full list of proposed features (and known issues), please see the [open issues](https://github.com/CloverOSe/nano-rs/issues).

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