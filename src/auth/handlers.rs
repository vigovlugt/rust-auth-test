use crate::auth::session::create_session;
use crate::users::repository::get_user_by_provider_id;
use crate::AppState;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::cookie::SameSite;
use axum_extra::extract::CookieJar;
use oauth2::reqwest::async_http_client;
use oauth2::TokenResponse;
use std::sync::Arc;

pub async fn handle_auth_url(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), StatusCode> {
    let (auth_url, csrf_token) = match provider.as_str() {
        "google" => state
            .google_oauth2
            .authorize_url(oauth2::CsrfToken::new_random)
            .add_scope(oauth2::Scope::new("email".to_string()))
            .add_scope(oauth2::Scope::new("profile".to_string()))
            .url(),
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let jar = jar.add(
        Cookie::build(Cookie::new("csrf_token", csrf_token.secret().to_owned()))
            .http_only(true)
            .same_site(SameSite::Lax)
            .build(),
    );
    println!("Auth URL: {}", auth_url);

    return Ok((jar, Redirect::temporary(&auth_url.to_string())));
}

#[derive(serde::Deserialize)]
pub struct AuthCallbackQuery {
    code: String,
    state: String,
}

pub async fn handle_auth_callback(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    jar: CookieJar,
    query: Query<AuthCallbackQuery>,
) -> Result<(CookieJar, Redirect), StatusCode> {
    let csrf_token = match jar.get("csrf_token") {
        Some(csrf_token) => csrf_token,
        None => return Err(StatusCode::BAD_REQUEST),
    };
    if csrf_token.value() != query.state {
        return Err(StatusCode::BAD_REQUEST);
    }

    let token = state
        .google_oauth2
        .exchange_code(oauth2::AuthorizationCode::new(query.code.clone()))
        .request_async(async_http_client)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    #[derive(serde::Deserialize)]
    struct UserInfo {
        sub: String,
        email: String,
        name: String,
        email_verified: bool,
    }

    let user_info = state
        .http_client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(token.access_token().secret().to_owned())
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .json::<UserInfo>()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_id = get_user_by_provider_id(&state.db, &provider, &user_info.sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_id = match user_id {
        Some(user_id) => user_id,
        None => {
            let user_id = crate::users::repository::create_user(
                &state.db,
                &user_info.email,
                &provider,
                &user_info.sub,
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            user_id
        }
    };

    let session = create_session(&state.db, &user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jar = jar.add(
        Cookie::build(Cookie::new("session_id", session.id))
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(false)
            .path("/")
            .build(),
    );

    return Ok((jar, Redirect::temporary("/")));
}
