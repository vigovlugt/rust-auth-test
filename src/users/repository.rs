use crate::users::models::User;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

pub async fn get_user(db: &Pool<Sqlite>, user_id: &str) -> Result<User, sqlx::Error> {
    let row = sqlx::query_as!(
        User,
        "SELECT id, email
    FROM user
    WHERE id = $1",
        user_id
    )
    .fetch_one(db)
    .await?;

    Ok(row)
}

pub async fn get_user_by_provider_id(
    db: &Pool<Sqlite>,
    provider: &str,
    provider_user_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    let row = match sqlx::query!(
        "SELECT user_id
    FROM oauth_account
    WHERE provider_id = $1 AND provider_user_id = $2",
        provider,
        provider_user_id
    )
    .fetch_one(db)
    .await
    {
        Ok(row) => row,
        Err(e) => match e {
            sqlx::Error::RowNotFound => return Ok(None),
            _ => return Err(e),
        },
    };

    Ok(Some(row.user_id))
}

pub async fn create_user(
    db: &Pool<Sqlite>,
    email: &str,
    provider: &str,
    provider_user_id: &str,
) -> Result<String, sqlx::Error> {
    let user_id = Uuid::new_v4().to_string();

    let mut tx = db.begin().await?;

    sqlx::query!(
        "INSERT INTO user (id, email)
    VALUES ($1, $2)",
        user_id,
        email
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO oauth_account (provider_id, provider_user_id, user_id)
    VALUES ($1, $2, $3)",
        provider,
        provider_user_id,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(user_id)
}
