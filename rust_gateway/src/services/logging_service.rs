use crate::services::settings::Config;

pub(crate) fn init_logging(config: &Config) {
    let level = config.log_level.to_lowercase();
    let filter = tracing_subscriber::EnvFilter::try_new(level)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    if config.log_format == "json" {
        tracing_subscriber::fmt().with_env_filter(filter).json().init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }
}
