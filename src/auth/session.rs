use chrono::{DateTime, TimeDelta, Utc};

pub const SESSION_DURATION: TimeDelta = match TimeDelta::try_days(30) {
    Some(duration) => duration,
    None => panic!("Failed to create session duration"),
};

pub struct Session {
    pub id: String,
    pub user_id: String,
    pub expires_at: DateTime<Utc>,
}

pub async fn create_session(
    db: &sqlx::Pool<sqlx::Sqlite>,
    user_id: &str,
) -> Result<Session, sqlx::Error> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + SESSION_DURATION;

    sqlx::query!(
        "INSERT INTO user_session (id, user_id, expires_at)
    VALUES ($1, $2, $3)",
        session_id,
        user_id,
        expires_at
    )
    .execute(db)
    .await?;

    Ok(Session {
        id: session_id,
        user_id: user_id.to_string(),
        expires_at,
    })
}

pub async fn get_session(
    db: &sqlx::Pool<sqlx::Sqlite>,
    session_id: &str,
) -> Result<Session, Box<dyn std::error::Error>> {
    let row = sqlx::query!(
        "SELECT id, user_id, expires_at
    FROM user_session
    WHERE id = $1",
        session_id
    )
    .fetch_one(db)
    .await?;

    Ok(Session {
        id: row.id,
        user_id: row.user_id,
        expires_at: row.expires_at.parse()?,
    })
}

pub async fn update_session(
    db: &sqlx::Pool<sqlx::Sqlite>,
    session_id: &str,
) -> Result<Session, Box<dyn std::error::Error>> {
    let expires_at = Utc::now() + SESSION_DURATION;

    sqlx::query!(
        "UPDATE user_session
    SET expires_at = $1
    WHERE id = $2",
        expires_at,
        session_id
    )
    .execute(db)
    .await?;

    Ok(Session {
        id: session_id.to_string(),
        user_id: session_id.to_string(),
        expires_at,
    })
}
