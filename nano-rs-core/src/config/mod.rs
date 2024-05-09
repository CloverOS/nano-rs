use clap::Parser;
use serde::Deserialize;
use crate::config::rest::RestConfig;

pub mod read;
pub mod rest;
pub mod db;
pub mod logger;
pub mod etcd;
pub mod rpc;
pub mod prometheus;

/// - 从路径加载配置文件
/// - Load configuration file from config_path
/// # Examples
/// ```
/// use nano_rs_core::config::rest::RestConfig;
/// let rest_config = nano_rs_core::config::init_config::<RestConfig>("etc/config.yaml");
/// ```
#[allow(dead_code)]
pub fn init_config<T: Clone + Default + for<'a> Deserialize<'a>>(config_path: &str) -> T {
    let config = read::read_config(config_path).unwrap_or(T::default());
    config
}

/// - 初始化 Rest 配置与命令行接口
/// - Initialize Rest configuration with command line interface
///  # Examples
///
/// ```
/// let rest_config = nano_rs_core::config::init_rest_config_with_cli();
/// ```
///
#[allow(dead_code)]
pub fn init_rest_config_with_cli() -> rest::RestConfig {
    let cli = Cli::parse();
    let rest_config = read::read_rest_config(cli.config.as_str()).unwrap_or(RestConfig::default());
    rest_config
}

/// - 使用命令行接口初始化配置
/// - Initialize configuration with command line interface
/// # Examples
///
/// ```
/// use nano_rs_core::config::init_config_with_cli;
/// use nano_rs_core::config::rest::RestConfig;
///
/// let config = init_config_with_cli::<RestConfig>();
/// ```
#[allow(dead_code)]
pub fn init_config_with_cli<T: Clone + Default + for<'a> Deserialize<'a>>() -> T {
    let cli = Cli::parse();
    let config = init_config(cli.config.as_str());
    config
}


/// - 命令行接口结构体
/// - Command line interface structure
///
/// # Examples
/// ```bash
/// ./xxx --config etc/config.yaml
/// ./xxx -c etc/config.yaml
/// ```
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// 配置文件路径
    /// config file path
    #[arg(short, long)]
    pub config: String,
}