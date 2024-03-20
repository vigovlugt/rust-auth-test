mod auth;
mod router;
mod users;

use crate::router::create_router;
use oauth2::basic::BasicClient;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

struct AppState {
    db: Pool<Sqlite>,
    google_oauth2: BasicClient,
    http_client: reqwest::Client,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().unwrap();

    let db = sqlx::SqlitePool::connect(std::env::var("DATABASE_URL").unwrap().as_str()).await?;

    let google_oauth2 = BasicClient::new(
        oauth2::ClientId::new(std::env::var("GOOGLE_CLIENT_ID").unwrap()),
        Some(oauth2::ClientSecret::new(
            std::env::var("GOOGLE_CLIENT_SECRET").unwrap(),
        )),
        oauth2::AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string())?,
        Some(oauth2::TokenUrl::new(
            "https://oauth2.googleapis.com/token".to_string(),
        )?),
    )
    .set_redirect_uri(oauth2::RedirectUrl::new(
        "http://localhost:3000/login/google/callback".to_string(),
    )?);

    let http_client = reqwest::Client::new();

    let state = AppState {
        db,
        google_oauth2,
        http_client,
    };

    let router = create_router(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!(
        "Listening on: http://localhost:{}",
        listener.local_addr().unwrap().port()
    );

    axum::serve(listener, router).await.unwrap();
    Ok(())
}
