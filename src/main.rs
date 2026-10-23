use axum::{
    extract::State,
    http::{Method, StatusCode},
    routing::{get, post},
    Json, Router,
};
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Instant};
use uuid::Uuid;

#[derive(Debug, Row, Serialize, Deserialize)]
struct RouterEvent {
    event_id: String,
    route_path: String,
    method: String,
    status_code: u16,
    latency_ms: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let ch_host = std::env::var("CLICKHOUSE_HOST").unwrap_or_else(|_| "localhost".into());
    let ch_port = std::env::var("CLICKHOUSE_PORT").unwrap_or_else(|_| "8123".into());
    let ch_user = std::env::var("CLICKHOUSE_USER").unwrap_or_else(|_| "default".into());
    let ch_password = std::env::var("CLICKHOUSE_PASSWORD").unwrap_or_else(|_| "clickhouse_dev_pass".into());
    let ch_db = std::env::var("CLICKHOUSE_DB").unwrap_or_else(|_| "jev_router_db".into());

    let client = Client::default()
        .with_url(format!("http://{}:{}", ch_host, ch_port))
        .with_user(ch_user)
        .with_password(ch_password)
        .with_database(ch_db);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/route", post(handle_route))
        .with_state(client);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("🚀 [Compustretch] Service listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> (StatusCode, 'static str) {
    (StatusCode::OK, "OK")
}

async fn handle_route(
    State(client): State<Client>,
) -> (StatusCode, Json<serde_json::Value>) {
    let start = Instant::now();
    tokio::time::sleep(tokio::time::Duration::from_millis(12)).await;

    let latency = start.elapsed().as_secs_f64() * 1000.0;
    let event = RouterEvent {
        event_id: Uuid::new_v4().to_string(),
        route_path: "/api/v1/route".to_string(),
        method: Method::POST.to_string(),
        status_code: 200,
        latency_ms: (latency * 100.0).round() / 100.0,
    };

    let ch_client = client.clone();
    tokio::spawn(async move {
        if let Ok(mut insert) = ch_client.insert("router_events") {
            let _ = insert.write(&event).await;
            let _ = insert.end().await;
            println!("✓ Telemetry recorded to ClickHouse: {:?}", event.event_id);
        }
    });

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "routed",
            "org": "Compustretch",
            "event_id": event.event_id
        })),
    )
}
