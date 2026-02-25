use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logging() {
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true);

    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,blog_server=debug"));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    tracing::info!("Logging initialized");
}
