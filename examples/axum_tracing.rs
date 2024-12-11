use std::time::Duration;

use axum::{extract::Request, routing::get, Router};
use opentelemetry::{trace::{self, TracerProvider as _}, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime::Tokio, trace::{RandomIdGenerator, Tracer, TracerProvider}, Resource};
use tokio::{
    join, net::TcpListener, time::{sleep, Instant}
};
use tracing::{
    debug, info,
    instrument,
    level_filters::LevelFilter,
    warn,
};
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

    // opentelemetry
    let tracer = init_tracer()?;
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .with(telemetry)
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

#[instrument(fields(http.uri = req.uri().path(), http.method = req.method().as_str()))]
async fn index_handler(req: Request) -> &'static str {
    debug!("index handler started");
    sleep(Duration::from_millis(10)).await;
    let ret = long_task().await;
    info!(http.status_code = 200, "index handler completed");
    ret
}

#[instrument]
async fn long_task() -> &'static str {
    let start = Instant::now();
    sleep(Duration::from_millis(112)).await;
    let t1 = task1();
    let t2 = task2();
    let t3 = task3();
    join!(t1, t2, t3);
    let elapsed = start.elapsed().as_millis();
    warn!(app.task_duration = elapsed, "task takes too long");
    "Hello, world!".into()
}

#[instrument]
async fn task1() {
    sleep(Duration::from_millis(20)).await;
}

#[instrument]
async fn task2() {
    sleep(Duration::from_millis(50)).await;
}

#[instrument]
async fn task3() {
    sleep(Duration::from_millis(500)).await;
}

fn init_tracer() -> anyhow::Result<Tracer> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        // tonic 是grpc库
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        // .with_timeout(Duration::from_secs(3))
        // .with_metadata(map)
        .build()?;
    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, Tokio)
        // 给Service命名
        .with_resource(Resource::new(vec![KeyValue::new(
          "service.name",
          "axum-tracing",
      )]))
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(32)
        .with_max_attributes_per_span(64)
        .build();
    let tracer = provider.tracer("my-service-tracer");
    Ok(tracer)
}
