use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::logger::{LOG_LEVEL, LogFileConfig};
use crate::config::rest::RestConfig;

/// - 初始化tracing
/// - Init tracing for project
/// #Example
/// ```rust
/// use nano_rs_core::config::rest::RestConfig;
///
/// let rest_config = nano_rs_core::config::init_config_with_cli::<RestConfig>();
/// let _guards = nano_rs_core::tracing::init_tracing(&rest_config);
/// ```
#[allow(dead_code)]
pub fn init_tracing(rest_config: &RestConfig) -> Vec<WorkerGuard> {
    let time_fmt =
        time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:6]");
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(time::macros::offset!(+8), time_fmt);

    let mut layers = vec![];
    let mut guards = vec![];
    for level in LOG_LEVEL {
        let log_file_config = rest_config.log.clone()
            .level.unwrap_or(crate::config::logger::Level::default()).get_log_file_config(level)
            .unwrap_or(LogFileConfig::default());
        if log_file_config.clone().file.unwrap_or(true) {
            let file_appender = tracing_appender::rolling::daily(log_file_config.clone().dir.unwrap_or("logs".to_string()),
                                                                 log_file_config.get_default_prefix(level));
            let tracing_level = log_file_config.get_tracing_level(level);
            let (appender, guard) = tracing_appender::non_blocking(file_appender);
            let layer = tracing_subscriber::fmt::layer()
                .with_timer(timer.clone())
                .with_ansi(rest_config.log.clone().ansi.unwrap_or(false))
                .with_writer(
                    appender
                        .with_min_level(tracing_level)
                        .with_max_level(tracing_level),
                );
            layers.push(layer);
            guards.push(guard);
        }
    }

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(rest_config.get_env_filter()))
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_timer(timer)
                .with_writer(std::io::stderr),
        )
        .with(layers)
        .init();
    guards
}