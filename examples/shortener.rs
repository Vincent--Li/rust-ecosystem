use anyhow::Result;
use axum::{
    extract::{Path, State}, http::{header::LOCATION, HeaderMap, StatusCode}, response::IntoResponse, routing::{get, post}, Json, Router
};
use derive_builder::Builder;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[derive(Debug, Deserialize, Serialize, Builder)]
struct ShortenReq {
    url: String,
}

#[derive(Debug, Deserialize, Serialize, Builder)]
struct ShortenResp {
    url: String,
}

#[derive(Debug, Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Debug, FromRow)]
struct UrlRecord {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

const LISTEN_ADDR: &str = "localhost:9876";

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let url = "postgres://vincent:vincent@localhost:5432/shortener";
    // PgPool 本身是Arc的, 所以不需要再套Arc
    let state = AppState::try_new(url).await?;
    info!("connected to database: {url}");
    let addr = "127.0.0.1:9876";
    let listener = tokio::net::TcpListener::bind(LISTEN_ADDR).await?;
    info!("listening on {}", LISTEN_ADDR);

    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

// async 不能漏
async fn shorten(
    State(state): State<AppState>,
    Json(data): Json<ShortenReq>, // post body 只能consumey一次, 所以要放在最后一个
) -> Result<impl IntoResponse, StatusCode> {
    let id = state
        .shorten(&data.url)
        .await
        .map_err(|e| {
            warn!("shorten failed: {}", e);
            StatusCode::UNPROCESSABLE_ENTITY
        })?;
    let body = Json(ShortenResp {
        url: format!("http://{}/{}", LISTEN_ADDR, id),
    });
    Ok((StatusCode::CREATED, body))
}

async fn redirect(State(state): State<AppState>, Path(id): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let url = state
        .get_url(&id)
        .await
        .map_err(|_|StatusCode::NOT_FOUND)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        LOCATION,
        url.parse().unwrap(),
    );
    Ok((StatusCode::PERMANENT_REDIRECT, headers ))
}

impl AppState {
    async fn try_new(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await?;
        // create table if not exists
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS urls (
                id CHAR(6) PRIMARY KEY,
                url TEXT NOT NULL UNIQUE
            )"#,
        )
        .execute(&pool)
        .await?;
        Ok(Self { db: pool })
    }

    async fn shorten(&self, url: &str) -> Result<String> {
        let id = nanoid!(6);
        let row: UrlRecord = sqlx::query_as("INSERT INTO urls (id, url) VALUES ($1, $2) ON CONFLICT(url) DO update set url=excluded.url RETURNING id")
            .bind(&id)
            .bind(url)
            .fetch_one(&self.db)
            .await?;
        info!("Shortened {} to {}", row.url, row.id);
        Ok(row.id)
    }

    async fn get_url(&self, id: &str) -> Result<String> {
        let record: UrlRecord = sqlx::query_as("SELECT url FROM urls WHERE id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await?;
        Ok(record.url)
    }
}
