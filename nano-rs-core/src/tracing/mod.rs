
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::config::rest::RestConfig;

/// - 初始化tracing
/// - Init tracing for project
/// #Example
/// ```rust
/// use nano_rs_core::config::rest::RestConfig;
///
/// let rest_config = nano_rs_core::config::init_config_with_cli::<RestConfig>();
/// let _guard = nano_rs_core::tracing::init_tracing(&rest_config);
/// ```
#[allow(dead_code)]
pub fn init_tracing(service_config: &RestConfig) -> (WorkerGuard, WorkerGuard, WorkerGuard) {
    let time_fmt =
        time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:6]");
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(time::macros::offset!(+8), time_fmt);

    let access_file_appender = tracing_appender::rolling::daily("logs", "info.log");
    let (access_appender, access_guard) = tracing_appender::non_blocking(access_file_appender);
    let access_layer = tracing_subscriber::fmt::layer()
        .with_timer(timer.clone())
        .with_ansi(false)
        .with_writer(
            access_appender
                .with_min_level(tracing::Level::INFO)
                .with_max_level(tracing::Level::INFO),
        );

    let error_file_appender = tracing_appender::rolling::daily("logs", "error.log");
    let (error_appender, error_guard) = tracing_appender::non_blocking(error_file_appender);
    let error_layer = tracing_subscriber::fmt::layer()
        .with_timer(timer.clone())
        .with_ansi(false)
        .with_writer(
            error_appender
                .with_min_level(tracing::Level::ERROR)
                .with_max_level(tracing::Level::ERROR),
        );

    let trace_file_appender = tracing_appender::rolling::daily("logs", "trace.log");
    let (trace_appender, trace_guard) = tracing_appender::non_blocking(trace_file_appender);
    let trace_layer = tracing_subscriber::fmt::layer()
        .with_timer(timer.clone())
        .with_ansi(false)
        .with_writer(
            trace_appender
                .with_min_level(tracing::Level::TRACE)
                .with_max_level(tracing::Level::TRACE),
        );

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            (service_config.get_env_filter())
                .into()
        }))
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_timer(timer)
                .with_writer(std::io::stderr),
        )
        .with(access_layer)
        .with(error_layer)
        .with(trace_layer)
        .init();
    let guards = (access_guard, error_guard, trace_guard);
    guards
}

#[allow(dead_code)]
fn get_level_by_string(level: &str) -> tracing::Level {
    match level {
        "info" => {
            tracing::Level::INFO
        }
        "error" => {
            tracing::Level::ERROR
        }
        "trace" => {
            tracing::Level::TRACE
        }
        "debug" => {
            tracing::Level::DEBUG
        }
        "warn" => {
            tracing::Level::WARN
        }
        _ => {
            tracing::Level::INFO
        }
    }
}