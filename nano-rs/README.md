[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]

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
    <li><a href="#quick-start">Quick Start</a> </li>
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

Axum

- Add build dependencies

```toml
[build-dependencies]
nano-rs = "0.1.0"
nano-rs-build = "0.1.0"
```

- Add build.rs

```rust
use std::error::Error;
use nano_rs_build::core::NanoBuilder;
use nano_rs::axum::gen::gen_route::AxumGenRoute;

fn main() -> Result<(), Box<dyn Error>> {
    NanoBuilder::new(None).gen_api_route(AxumGenRoute::new());
    Ok(())
}
```

- Add the configuration file to your desired directory (in the example, it is placed in [etc/config.yaml](https://github.com/CloverOS/nano-rs/blob/master/example/etc/config.yaml))

```yaml
port: 8888
name: example
host: 127.0.0.1
```

- Write your API code (for example, under api/pet), for get macros, please refer to [example](https://github.com/CloverOS/nano-rs/blob/master/example/src/api) (documentation coming soon...)
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

- Add main.rs (refer to the project structure in the example)

```rust
use axum::Router;
use nano_rs::axum::start::run;
use nano_rs::config::init_config_with_cli;
use nano_rs::config::rest::RestConfig;
use crate::routes::get_routes;

mod routes;
mod layers;
mod api;

#[tokio::main]
async fn main() {
    let rest_config = init_config_with_cli::<RestConfig>();
    let _guard = nano_rs::tracing::init_tracing(&rest_config);
    let service_context = ServiceContext {
        rest_config: rest_config.clone(),
    };
    let app = Router::new().nest(
        rest_config.base_path.as_str(),
        get_routes(service_context.clone(), rest_config.clone()),
    );
    run(app, rest_config).await
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
- After this, you only need to focus on writing your business logic code; nano-rs will automatically generate routes and register them with axum, allowing you to concentrate solely on implementing business logic.

## Roadmap

- [x] Auto-generate Axum framework routes
- [x] Default configuration for Tracing framework
- [x] Preset common web service configuration (managed via yaml)
- [ ] Auto-generate OpenApi

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

[contributors-shield]: https://img.shields.io/github/contributors/CloverOS/nano-rs.svg?style=for-the-badge

[contributors-url]: https://github.com/CloverOS/nano-rs/graphs/contributors

[forks-shield]: https://img.shields.io/github/forks/CloverOS/nano-rs.svg?style=for-the-badge

[forks-url]: https://github.com/CloverOS/nano-rs/network/members

[stars-shield]: https://img.shields.io/github/stars/CloverOS/nano-rs.svg?style=for-the-badge

[stars-url]: https://github.com/CloverOS/nano-rs/stargazers

[issues-shield]: https://img.shields.io/github/issues/CloverOS/nano-rs.svg?style=for-the-badge

[issues-url]: https://github.com/CloverOS/nano-rs/issues

[license-shield]: https://img.shields.io/github/license/CloverOS/nano-rs.svg?style=for-the-badge

[license-url]: https://github.com/CloverOS/nano-rs/blob/master/LICENSE.txt