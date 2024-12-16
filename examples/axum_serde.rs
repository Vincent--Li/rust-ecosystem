use std::sync::{Arc, Mutex};

use anyhow::Result;
use axum::{
    extract::State,
    routing::{get, patch},
    Json, Router,
};
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{instrument, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{format::FmtSpan, Layer},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer as _,
};

#[derive(Debug, PartialEq, Clone, Serialize, Builder)]
struct User {
    name: String,
    age: u8,
    dob: DateTime<Utc>,
    skills: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct UserUpdate {
    age: Option<u8>,
    skills: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry().with(layer).init();

    let addr = "127.0.0.1:3001";
    let listener = TcpListener::bind(addr).await?;

    let user = UserBuilder::default()
        .name("John".to_string())
        .age(42)
        .dob(Utc::now())
        .skills(vec!["Rust".to_string(), "C++".to_string()])
        .build()
        .unwrap();
    let user = Arc::new(Mutex::new(user));

    let app = Router::new()
        .route("/", get(user_handler))
        .route("/", patch(update_handler))
        .with_state(user);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[instrument]
async fn user_handler(State(user): State<Arc<Mutex<User>>>) -> Json<User> {
    user.lock().unwrap().clone().into()
}

#[instrument]
async fn update_handler(
    State(user): State<Arc<Mutex<User>>>,
    Json(payload): Json<UserUpdate>,
) -> Json<User> {
    let mut user = user.lock().unwrap();
    if let Some(age) = payload.age {
        user.age = age;
    }
    if let Some(skills) = payload.skills {
        user.skills = skills;
    }

    user.clone().into()
}
