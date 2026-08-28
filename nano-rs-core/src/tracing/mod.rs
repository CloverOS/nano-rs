use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::Registry;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::logger::{LOG_LEVEL, LogFileConfig};
use crate::config::rest::RestConfig;

pub type DynTracingLayer = Box<dyn tracing_subscriber::Layer<Registry> + Send + Sync + 'static>;

/// - 初始化tracing
/// - Init tracing for project
/// #Example
/// ```no_run
/// use nano_rs_core::config::rest::RestConfig;
///
/// let rest_config = nano_rs_core::config::init_config_with_cli::<RestConfig>();
/// let _guards = nano_rs_core::tracing::init_tracing(&rest_config);
/// ```
#[allow(dead_code)]
pub fn init_tracing(rest_config: &RestConfig) -> Vec<WorkerGuard> {
    init_tracing_with_layers(rest_config, vec![])
}

/// - 初始化tracing并注入扩展layer
/// - Init tracing with custom layers
/// #Example
/// ```no_run
/// use nano_rs_core::config::rest::RestConfig;
///
/// let rest_config = nano_rs_core::config::init_config_with_cli::<RestConfig>();
/// let _guards = nano_rs_core::tracing::init_tracing_with_layers(&rest_config, vec![]);
/// ```
#[allow(dead_code)]
pub fn init_tracing_with_layers(
    rest_config: &RestConfig,
    extra_layers: Vec<DynTracingLayer>,
) -> Vec<WorkerGuard> {
    let env_filter = build_env_filter(rest_config);
    let (layers, guards) = build_tracing_layers(rest_config, extra_layers);
    // Keep EnvFilter as a dedicated outer layer so it globally gates all log layers.
    tracing_subscriber::registry()
        .with(layers)
        .with(env_filter)
        .init();
    guards
}

fn build_env_filter(rest_config: &RestConfig) -> tracing_subscriber::EnvFilter {
    if let Some(level) = rest_config.log.level.clone() {
        tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(level.get_tracing_level().into())
    } else {
        tracing_subscriber::EnvFilter::new(rest_config.get_env_filter())
    }
}

fn build_tracing_layers(
    rest_config: &RestConfig,
    extra_layers: Vec<DynTracingLayer>,
) -> (Vec<DynTracingLayer>, Vec<WorkerGuard>) {
    let time_fmt = time::macros::format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:6]"
    );
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(time::macros::offset!(+8), time_fmt);

    let mut layers: Vec<DynTracingLayer> = vec![];
    let mut guards = vec![];

    let stderr_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_timer(timer.clone())
        .with_writer(std::io::stderr);
    layers.push(Box::new(stderr_layer));

    for level in LOG_LEVEL {
        let log_file_config = rest_config
            .log
            .clone()
            .level
            .unwrap_or(crate::config::logger::Level::default())
            .get_log_file_config(level)
            .unwrap_or(LogFileConfig::default());
        if log_file_config.file.unwrap_or(true) {
            let mut file_appender_builder = RollingFileAppender::builder()
                .rotation(Rotation::DAILY)
                .filename_prefix(log_file_config.get_default_prefix(level));
            if let Some(max_files) = log_file_config.max_files {
                file_appender_builder = file_appender_builder.max_log_files(max_files);
            }
            let file_appender = file_appender_builder
                .build(log_file_config.dir.as_deref().unwrap_or("logs"))
                .expect("initializing rolling file appender failed");
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
            layers.push(Box::new(layer));
            guards.push(guard);
        }
    }
    layers.extend(extra_layers);

    (layers, guards)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use tracing::Subscriber;
    use tracing_subscriber::layer::Context;

    fn test_rest_config() -> RestConfig {
        use crate::config::logger::{Level, LogConfig, LogFileConfig};

        RestConfig {
            port: 8080,
            name: "tracing_test".to_string(),
            host: Some("127.0.0.1".to_string()),
            mode: None,
            time_out: None,
            base_path: "".to_string(),
            body_limit: 4 * 1024 * 1024,
            log: LogConfig {
                level: Some(Level {
                    trace: Some(LogFileConfig {
                        dir: None,
                        prefix: None,
                        file: Some(false),
                        max_files: None,
                    }),
                    debug: Some(LogFileConfig {
                        dir: None,
                        prefix: None,
                        file: Some(false),
                        max_files: None,
                    }),
                    info: Some(LogFileConfig {
                        dir: None,
                        prefix: None,
                        file: Some(false),
                        max_files: None,
                    }),
                    warn: Some(LogFileConfig {
                        dir: None,
                        prefix: None,
                        file: Some(false),
                        max_files: None,
                    }),
                    error: Some(LogFileConfig {
                        dir: None,
                        prefix: None,
                        file: Some(false),
                        max_files: None,
                    }),
                }),
                ..Default::default()
            },
            rpc: None,
            prometheus: None,
        }
    }

    struct CounterLayer {
        hit: Arc<AtomicUsize>,
    }

    impl<S> tracing_subscriber::Layer<S> for CounterLayer
    where
        S: Subscriber,
    {
        fn on_event(&self, _event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
            self.hit.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn build_layers_without_extra_has_no_file_guards_when_disabled() {
        let rest_config = test_rest_config();
        let (_, guards) = build_tracing_layers(&rest_config, vec![]);
        assert_eq!(guards.len(), 0);
    }

    struct TempLogDir(std::path::PathBuf);

    impl TempLogDir {
        fn new(test_name: &str) -> Self {
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "nano-rs-{test_name}-{}-{unique}",
                std::process::id()
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for TempLogDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn build_info_file_layer(dir: &std::path::Path, max_files: usize) {
        let mut rest_config = test_rest_config();
        let info = rest_config
            .log
            .level
            .as_mut()
            .and_then(|level| level.info.as_mut())
            .unwrap();
        info.dir = Some(dir.to_string_lossy().into_owned());
        info.file = Some(true);
        info.max_files = Some(max_files);

        let (layers, guards) = build_tracing_layers(&rest_config, vec![]);
        drop(layers);
        drop(guards);
    }

    fn create_old_info_logs(dir: &std::path::Path) {
        for date in ["2000-01-01", "2000-01-02", "2000-01-03"] {
            std::fs::write(dir.join(format!("info.log.{date}")), date).unwrap();
        }
    }

    fn count_info_logs(dir: &std::path::Path) -> usize {
        std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .filter(|name| name.to_string_lossy().starts_with("info.log."))
            .count()
    }

    #[test]
    fn max_files_prunes_only_matching_level_files() {
        let dir = TempLogDir::new("max-files");
        create_old_info_logs(dir.path());
        let error_log = dir.path().join("error.log.2000-01-01");
        std::fs::write(&error_log, "error").unwrap();

        build_info_file_layer(dir.path(), 2);

        assert_eq!(count_info_logs(dir.path()), 2);
        assert!(error_log.exists());
    }

    #[test]
    fn zero_max_files_keeps_existing_logs() {
        let dir = TempLogDir::new("zero-max-files");
        create_old_info_logs(dir.path());

        build_info_file_layer(dir.path(), 0);

        assert_eq!(count_info_logs(dir.path()), 4);
    }

    #[test]
    fn extra_layer_receives_events() {
        let rest_config = test_rest_config();
        let hit = Arc::new(AtomicUsize::new(0));
        let extra: DynTracingLayer = Box::new(CounterLayer { hit: hit.clone() });

        let (layers, _guards) = build_tracing_layers(&rest_config, vec![extra]);
        let subscriber = tracing_subscriber::registry().with(layers);

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("custom-layer-hit");
        });

        assert!(hit.load(Ordering::SeqCst) >= 1);
    }
}
