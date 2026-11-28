use std::fs;
use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Initializes structured stdout + daily rotating file logging in `log_dir`.
/// Returns a `WorkerGuard` that must stay alive for the duration of `main`.
pub fn init_logging(log_dir: &Path) -> Result<WorkerGuard, std::io::Error> {
    fs::create_dir_all(log_dir)?;

    let file_appender = tracing_appender::rolling::daily(log_dir, "middleware.log");
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,esp32_clock_middleware=debug"));

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_names(false);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_writer(non_blocking_file);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}
