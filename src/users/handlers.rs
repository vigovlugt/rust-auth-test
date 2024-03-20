use std::sync::Arc;

use crate::{auth::validate_session, AppState};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;

use super::{models::User, repository::get_user};

pub async fn users_me_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> (StatusCode, Json<Option<User>>) {
    let user_id = match validate_session(&state.db, jar).await {
        Ok(user_id) => user_id,
        Err(_) => return (StatusCode::UNAUTHORIZED, Json(None)),
    };

    let user = match get_user(&state.db, &user_id).await {
        Ok(user) => user,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
    };

    return (StatusCode::OK, Json(Some(user)));
}
