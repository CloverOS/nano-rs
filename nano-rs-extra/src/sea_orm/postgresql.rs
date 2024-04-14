use log::LevelFilter;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use nano_rs_core::config::db::DataBaseConfig;

///获取数据库链接配置
fn get_con_config(data_base_config: &DataBaseConfig) -> ConnectOptions {
    let host = format!("{}:{}", data_base_config.host, data_base_config.port);
    let url = if data_base_config.config.is_empty() {
        format!(
            "postgres://{}:{}@{}/{}",
            data_base_config.username,
            data_base_config.password,
            host,
            data_base_config.database
        )
    } else {
        format!(
            "postgres://{}:{}@{}/{}?{}",
            data_base_config.username,
            data_base_config.password,
            host,
            data_base_config.database,
            data_base_config.config
        )
    };
    let mut opt = ConnectOptions::new(url.to_owned());
    opt.max_connections(data_base_config.max_open_conns)
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

///获取postgresql数据库链接
pub async fn get_pg_db(data_base_config: &DataBaseConfig) -> Result<DatabaseConnection, DbErr> {
    let opt = get_con_config(data_base_config);
    let db = Database::connect(opt).await?;
    Ok(db)
}
