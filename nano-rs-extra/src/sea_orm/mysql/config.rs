use log::LevelFilter;
use nano_rs_core::config::db::DataBaseConfig;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;

///获取数据库链接配置
fn get_con_config(data_base_config: &DataBaseConfig) -> ConnectOptions {
    let host = format!("{}:{}", data_base_config.host, data_base_config.port);
    let url = format!(
        "mysql://{}:{}@{}/{}?{}",
        data_base_config.username,
        data_base_config.password,
        host,
        data_base_config.database,
        data_base_config.config
    );
    let mut opt = ConnectOptions::new(url.to_owned());
    opt.max_connections(data_base_config.max_open_conns)
        .acquire_timeout(Duration::from_secs(
            data_base_config.acquire_timeout.unwrap_or(30),
        ))
        .connect_timeout(Duration::from_secs(
            data_base_config.connect_timeout.unwrap_or(10),
        ))
        .sqlx_logging(data_base_config.sqlx_logging);
    match data_base_config.logging_level {
        1 => {
            opt.sqlx_logging_level(LevelFilter::Debug);
        }
        2 => {
            opt.sqlx_logging_level(LevelFilter::Info);
        }
        3 => {
            opt.sqlx_logging_level(LevelFilter::Warn);
        }
        4 => {
            opt.sqlx_logging_level(LevelFilter::Error);
        }
        _ => {
            opt.sqlx_logging_level(LevelFilter::Debug);
        }
    }
    opt
}

///获取mysql数据库链接
pub async fn get_mysql_db(data_base_config: &DataBaseConfig) -> Result<DatabaseConnection, DbErr> {
    let opt = get_con_config(data_base_config);
    let db = Database::connect(opt).await?;
    Ok(db)
}
