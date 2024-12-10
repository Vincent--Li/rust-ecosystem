use std::time::Duration;

use axum::{routing::get, Router};
use tokio::{
    net::TcpListener,
    time::{sleep, Instant},
};
use tracing::{debug, info, instrument, level_filters::LevelFilter, warn};
use tracing_appender::non_blocking;
use tracing_subscriber::{
    fmt::{format::FmtSpan, Layer},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer as _,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化tracing, 也可以使用Layer的方式
    // tracing_subscriber::fmt::init();
    let console = Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    // 指定日志保存位置
    let file_appender = tracing_appender::rolling::daily("/tmp/axum_tracing/logs", "ecosystem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file = Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .with_writer(non_blocking)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .init();

    // 启动监听
    let add = "0.0.0.0:8081";
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/long", get(long_task));

    let listener = TcpListener::bind(add).await?;
    info!("Listening on {}", add);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[instrument]
async fn index_handler() -> &'static str {
    debug!( "index handler started");
    sleep(Duration::from_millis(10)).await;
    let ret = long_task().await;
    info!(http.status = 200, "index handler completed");
    ret
}

#[instrument]
async fn long_task() -> &'static str {
    let start = Instant::now();
    sleep(Duration::from_millis(112)).await;
    let elapsed = start.elapsed().as_millis();
    warn!(app.task_duration = elapsed, "task takes too long");
    "Hello, world!".into()
}
