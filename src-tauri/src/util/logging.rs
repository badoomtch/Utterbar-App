use std::path::Path;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init(data_dir: &Path) {
    let logs = data_dir.join("logs");
    let _ = std::fs::create_dir_all(&logs);
    let file = RollingFileAppender::new(Rotation::DAILY, logs, "utterbar.log");

    let filter = EnvFilter::try_from_env("UTTERBAR_LOG").unwrap_or_else(|_| EnvFilter::new("info"));

    let console_layer = fmt::layer().with_target(false).with_writer(std::io::stderr);
    let file_layer = fmt::layer().with_ansi(false).with_writer(file);

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .try_init();
}
