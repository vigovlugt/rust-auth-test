use axum_extra::extract::CookieJar;
use chrono::Utc;

use self::session::{get_session, update_session, SESSION_DURATION};

pub mod handlers;
pub mod session;

pub async fn validate_session(
    db: &sqlx::Pool<sqlx::Sqlite>,
    jar: CookieJar,
) -> Result<String, Box<dyn std::error::Error>> {
    let session_id = match jar.get("session_id") {
        Some(session_id) => session_id.value(),
        None => return Err("No session_id cookie found".into()),
    };

    let session = get_session(db, session_id).await?;

    if session.expires_at < Utc::now() {
        return Err("Session expired".into());
    }

    if session.expires_at - Utc::now() < SESSION_DURATION / 2 {
        update_session(&db, session_id).await?;
    }

    Ok(session.user_id)
}
