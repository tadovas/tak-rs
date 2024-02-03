use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, registry, EnvFilter, Layer};

pub fn init(level_filter: LevelFilter) -> anyhow::Result<()> {
    let layer = fmt::layer().with_filter(
        EnvFilter::builder()
            .with_default_directive(level_filter.into())
            .from_env_lossy(),
    );
    registry().with(layer).try_init()?;
    Ok(())
}
