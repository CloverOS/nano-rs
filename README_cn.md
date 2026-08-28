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
        <li><a href="#api文档生成">Api文档生成(基于utoipa)</a></li>
        <li><a href="#api信息收集生成">Api信息收集生成</a></li>
        <li><a href="#日志与请求链路追踪">日志与请求链路追踪</a></li>
        <li><a href="#模块化示例">模块化示例</a></li>
    </ol>
    </li>
    <li><a href="#其他">其他</a>
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

MSRV >= 1.94

### 安装

```shell
cargo add nano-rs
```

该命令安装 crates.io 上最新的正式版本。如需在下个版本发布前使用当前开发线上的功能，请依赖 `main`：

```shell
cargo add nano-rs --git https://github.com/CloverOS/nano-rs --branch main
```

## 快速开始

### 路由注册自动生成

- 添加构建依赖

```toml
[build-dependencies]
nano-rs = { git = "https://github.com/CloverOS/nano-rs", branch = "main" }
nano-rs-build = { git = "https://github.com/CloverOS/nano-rs", branch = "main" }
```

- 添加生成组件 build.rs

```rust
use std::error::Error;
use nano_rs::axum::generator::gen_route::AxumGenRoute;
use nano_rs::core::NanoBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    NanoBuilder::new(None).gen_api_route(AxumGenRoute::new());
    Ok(())
}
```

- 将配置文件添加到你想要的目录（在示例中，它被放置在 [etc/config.yaml](https://github.com/CloverOS/nano-rs/blob/main/example/etc/config.yaml)）

```yaml
port: 8888
name: example
host: 127.0.0.1
```

- 在项目的任何地方用宏编写你的API代码（例如，在api/pet下），关于宏，请参考 [示例](https://github.com/CloverOS/nano-rs/blob/main/example/src/api)

```rust
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::rest::RestResp;
use nano_rs::{biz_ok, get};

#[get(path = "/store/name", layers = ["crate::layers::auth::auth_token1"])]
pub async fn get_store_name() -> Result<RestResp<String>, ServerError> {
    biz_ok!("Doggy Store".to_string())
}
```

- 构建项目。相关输入发生变化时，Cargo 会重新运行 `build.rs`。

```shell
cargo build
```

- 然后你会在你的 `src/` 中得到一个名为 `routes.rs` 的文件。
- 不要编辑 `routes.rs`，因为每次构建时它都会被覆盖。
- 编辑 `main.rs`。 (参考示例中的项目结构)

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
        // 可信反向代理设置 X-Real-IP 时可使用该来源。
        .add_secure_client_ip_source_layer(ClientIpSource::XRealIp)
        .run()
        .await;
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
nano-rs = { git = "https://github.com/CloverOS/nano-rs", branch = "main", features = ["utoipa_axum"] }
nano-rs-build = { git = "https://github.com/CloverOS/nano-rs", branch = "main" }
utoipa = { version = "5.5.0", features = ["axum_extras"] }
```

- 在 build.rs 中添加生成组件

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

- OpenAPI 现在支持 **两种写法**，并且旧的 `#[utoipa::path(...)]` 写法完全兼容。
- 旧写法（继续支持）：完整编写 utoipa 注解。参考 [utoipa](https://github.com/juhaku/utoipa/tree/master/examples/todo-axum)。

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

- 新简写（推荐）：直接写 `#[get]/#[post]`，nano-rs 会根据入参提取器和返回类型自动推导 OpenAPI。

```rust
/// Query pet by id
#[get(path = "/store/pet", tag = "Store")]
pub async fn get_query_pet_name(
    Query(query): Query<QueryPet>,
) -> Result<RestResp<Pet>, ServerError> { ... }
```

- 支持模块级默认分组（减少每个接口重复写 `tag`）：

```rust
/// @tag Store
pub mod store;
```

- 使用模块级 `/// @tag ...` 后，该模块内的接口可以省略 `tag`。
- 如果存在多层嵌套模块，会继承最近父模块的 tag；如果子模块自己声明了 `/// @tag ...`，则优先使用子模块的 tag。
- 除非处理函数同时使用了 `#[utoipa::path(...)]`，否则路由宏必须提供 `path = "..."`；旧写法会使用 utoipa 中的路径生成路由。
- 如果两个注解都提供路径，两者必须完全一致。其他 utoipa 元数据会被保留，但路径不一致会给出明确错误并中止构建。
- 构建项目。相关输入发生变化时，Cargo 会重新运行 `build.rs`。

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
nano-rs = { git = "https://github.com/CloverOS/nano-rs", branch = "main" }
nano-rs-build = { git = "https://github.com/CloverOS/nano-rs", branch = "main" }
```

- 添加 build.rs

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

- 这将在 `src/` 中生成 `api_info.rs`，你可以使用 `get_api_info()` 获取收集到的接口信息。
- `routes.rs`、`doc.rs` 和 `api_info.rs` 都是生成文件，请勿手动修改，后续构建可能覆盖它们。

### 日志与请求链路追踪

日志文件按级别在 `log.level` 下分别配置：

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

- 日志文件按天滚动。`max_files` 对每个日志级别独立生效，并包含当前文件；省略该字段或设置为 `0` 时不清理旧文件。
- 请求链路追踪会复用非空的入站 `x-request-id`；没有可用值时生成 UUID，并将最终 ID 写入响应头和请求日志。
- `ignore_resource` 会同时匹配配置的 HTTP 方法，以及实际请求路径或 Axum 路由模板，例如 `/pets/{id}`。
- 启用响应体日志时，SSE 流和 WebSocket 升级会跳过响应体缓冲，避免阻塞流式响应或协议升级。
- 如需注入自定义 tracing subscriber layer，可向 `nano_rs::tracing::init_tracing_with_layers` 传入 `nano_rs::tracing::DynTracingLayer`，同时保留内置日志配置。

### 模块化示例

跨 API、model 和 layer crate 的路由及 OpenAPI 生成方式可参考 [`example_modular`](https://github.com/CloverOS/nano-rs/tree/main/example_modular)。

## 其他

### SeaOrm从数据库生成postgresql注释
- 因为SeaOrm不支持从postgresql读取注释到实体类，所以我们提供了一个工具来生成注释。
- 添加构建依赖
```toml
[build-dependencies]
nano-rs-extra = { git = "https://github.com/CloverOS/nano-rs", branch = "main", features = ["postgres"] }
tokio = { version = "1.53.1", features = ["full"] }
```
- 添加 build.rs
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

有关建议功能（和已知问题）的完整列表，请参阅[未解决的问题](https://github.com/CloverOS/nano-rs/issues)。


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
