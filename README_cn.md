[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]




<div align="center">
  <a href="https://github.com/CloverOS/nano-rs">
    <img src="images/logo.png" alt="Logo" width="120" height="120">
  </a>
</div>

&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;nano-rs是一个轻量级、无入侵且遵循“约定优于配置”原则的Rust Web服务组件，旨在提供一个快速且高效的开发体验。
通过减少配置的负担，让您更能专注于业务逻辑的实现。

---

- **轻量级**: 通过精简的设计，保证自身的启动与运行速度快速且资源占用极低。
- **无入侵**: 框架的设计允许无缝集成到现有的Rust项目中，无需担心对业务代码产生干扰。
- **约定优于配置**: 框架采用“智能默认”的策略，为常见的应用场景提供了预设配置，这意味着您只需少量的配置或不需要任何配置就能开始工作。同时，它也提供充足的配置选项以满足特定需求，确保了极致的灵活性和可扩展性。
- **专注于业务**: 我们的目标是简化Web服务的开发过程，通过减少配置上的繁琐和杂项，让您能够将更多精力集中于业务逻辑的实现，从而提高整个项目开发的效率和质量。

<details>
  <summary>目录</summary>
  <ol>
    <li>
      <a href="#环境要求">环境要求</a>
    </li>
    <li><a href="#安装">安装</a></li>
    <li><a href="#快速开始">快速开始(Axum)</a>
    <ol>
        <li><a href="#路由注册自动生成">路由注册自动生成</a></li>
        <li><a href="#Api文档生成">Api文档生成(基于utoipa)</a></li>
        <li><a href="#Api信息收集生成">Api信息收集生成</a></li> 
    </ol>
    </li>
    <li><a href="其他">其他</a>
      <ol>
         <li><a href="#seaorm从数据库生成postgresql注释">SeaOrm从数据库生成postgresql注释</a></li>
      </ol>
    </li>
    <li><a href="#路线图">路线图</a></li>
    <li><a href="#许可">许可</a></li>
    <li><a href="#联系方式">联系方式</a></li>
    <li><a href="#鸣谢">鸣谢</a></li>
  </ol>
</details>

### 环境要求

MSRV >= 1.66

### 安装

```shell
  cargo add nano-rs
```

## 快速开始

### 路由注册自动生成

- 添加构建依赖

```toml
[build-dependencies]
nano-rs = "0.1.3"
nano-rs-build = "0.1.2"
```

- 添加生成组件 build.rs

```rust
use std::error::Error;
use nano_rs_build::core::NanoBuilder;
use nano_rs::axum::gen::gen_route::AxumGenRoute;

fn main() -> Result<(), Box<dyn Error>> {
    NanoBuilder::new(None).gen_api_route(AxumGenRoute::new());
    Ok(())
}
```

- 将配置文件添加到你想要的目录（在示例中，它被放置在 [etc/config.yaml](https://github.com/CloverOS/nano-rs/blob/master/example/etc/config.yaml)）

```yaml
port: 8888
name: example
host: 127.0.0.1
```

- 在项目的任何地方用宏编写你的API代码（例如，在api/pet下），关于宏，请参考 [示例](https://github.com/CloverOS/nano-rs/blob/master/example/src/api)

```rust
#[get(path = "/store/name", layers = ["crate::layers::auth::auth_token1"])]
pub async fn get_store_name() -> Result<RestResp<String>, ServerError> {
    biz_ok("Doggy Store".to_string())
}
```

- 运行一次构建（只需要对项目的第一次编译）

```shell
cargo build
```

- 然后你会在你的 `src/` 中得到一个名为 `routes.rs` 的文件。
- 不要编辑 `routes.rs`，因为每次构建时它都会被覆盖。
- 编辑 `main.rs`。 (参考示例中的项目结构)

```rust
use axum::Router;
use nano_rs_core::config::rest::RestConfig;
use axum_client_ip::ClientIpSource;
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
        /// 如果是使用nginx代理，需要使用ClientIpSource::XRealIp
        .add_secure_client_ip_source_layer(ClientIpSource::XRealIp)
        .run()
        .await;
}

#[derive(Clone)]
pub struct ServiceContext {
  pub rest_config: RestConfig, 
}
```

- 运行你的web应用程序

```shell
cargo run -- --config etc/config.yaml
```

- 之后，你只需要专注于编写你的业务逻辑代码；nano-rs 将自动生成路由并将它们注册到 axum，让你只专注于实现业务逻辑。

### Api文档生成

- 添加构建依赖

```toml
[build-dependencies]
nano-rs = { version = "0.1.3", features = ["utoipa_axum"] }
nano-rs-build = "0.1.2"
utoipa = { version = "5.1.1", features = ["axum_extras"] }
```

- 在 build.rs 中添加生成组件

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

- 编写 utoipa 代码，参见 [示例](https://github.com/CloverOS/nano-rs/blob/main/example/src/api/pet/store.rs)，更多文档请参考 [utoipa](https://github.com/juhaku/utoipa/tree/master/examples/todo-axum)

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
#[get()]
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

- 如果启用了 utoipa_axum 特性，则不需要重复编写path和group等代码（除非需要一个中间层），只需编写 utoipa 代码，即可获取 openapi 文档和 axum 路由。
- 运行一次构建（只需要对项目的第一次编译）

```shell
cargo build
```

- 然后你会在你的 `src/` 中得到一个名为 `doc.rs` 的文件。
- 不要编辑 `doc.rs`，因为每次构建时它都会被覆盖。
- 现在你可以使用 `doc.rs` 生成 openapi 文档，参见 [示例](https://github.com/CloverOS/nano-rs/blob/main/example/src/main.rs)

- 运行你的web应用程序

```shell
cargo run -- --config etc/config.yaml
```

### Api信息收集生成

- 添加构建依赖

```toml
[build-dependencies]
nano-rs = "0.1.3"
nano-rs-build = "0.1.2"
```

- 添加 build.rs

```rust
fn main() -> Result<(), Box<dyn Error>> {
    NanoBuilder::new(None)
        .gen_api_route(AxumGenRoute::new())
        .gen_api_info(AxumGenApiInfo::new());
    Ok(())
}
```

- 这将在你的 `src/` 中生成 `api_info.rs`，用于收集你所有的api信息，你可以使用 `get_api_info()` 获取所有api信息。

### SeaOrm从数据库生成postgresql注释
- 因为SeaOrm不支持从postgresql读取注释到实体类，所以我们提供了一个工具来生成注释。
- 添加构建依赖
```toml
[build-dependencies]
nano-rs-extra = { version = "0.1.4" }
tokio = { version = "1.34.0", features = ["full"] }
```
- 添加 build.rs
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
- 运行构建
- 它将会从postgresql数据库中读取你已经填写的注释注入到对应的实体类字段中
- 构建之前
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

- 构建之后
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


## 路线图

- [x] Axum框架路由自动生成
- [x] 默认配置Tracing日志框架
- [x] 预置通用web服务配置（通过yaml管理）
- [x] OpenApi自动生成 (基于 utoipa)

有关建议功能（和已知问题）的完整列表，请参阅[未解决的问题](https://github.com/CloverOSe/nano-rs/issues)。


<!-- LICENSE -->

## 许可

根据 MIT 许可证分发。有关更多信息，请参阅“LICENSE.txt”。



<!-- CONTACT -->

## 联系方式

- a527756694@gmail.com

## 鸣谢

* [Anyhow](https://github.com/dtolnay/anyhow)
* [Axum](https://github.com/tokio-rs/axum)
* [Clap](https://github.com/clap-rs/clap)
* [Tokio](https://github.com/tokio-rs/tokio)
* [Tracing](https://github.com/tokio-rs/tracing)

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->

[contributors-shield]: https://img.shields.io/github/contributors/CloverOS/nano-rs.svg?style=for-the-badge

[contributors-url]: https://github.com/CloverOS/nano-rs/graphs/contributors

[forks-shield]: https://img.shields.io/github/forks/CloverOS/nano-rs.svg?style=for-the-badge

[forks-url]: https://github.com/CloverOS/nano-rs/network/members

[stars-shield]: https://img.shields.io/github/stars/CloverOS/nano-rs.svg?style=for-the-badge

[stars-url]: https://github.com/CloverOS/nano-rs/stargazers

[issues-shield]: https://img.shields.io/github/issues/CloverOS/nano-rs.svg?style=for-the-badge

[issues-url]: https://github.com/CloverOS/nano-rs/issues
