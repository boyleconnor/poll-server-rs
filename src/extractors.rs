use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::StatusCode;
use axum_extra::extract::SignedCookieJar;
use chrono::Utc;
use crate::auth::UserRole;
use crate::SESSION_COOKIE;
use crate::state::{AppState, Key, FromRef};
use axum::http::request::Parts;
use axum::RequestPartsExt;

pub(crate) struct AuthenticatedUser {
    pub(crate) username: String,
    pub(crate) role: UserRole,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AuthenticatedUser: FromRequest<S>,
    AppState: FromRef<S>,
    S: Send + Sync,
    Key: FromRef<S>
{
    type Rejection = (StatusCode, String);
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let jar: SignedCookieJar = SignedCookieJar::from_request_parts(parts, state).await.map_err(|_| (StatusCode::UNAUTHORIZED, "could not validate authentication cookies".to_string()))?; // FIXME: Fix this error message
        if let Some(cookie) = jar.get(SESSION_COOKIE) {
            let user_sessions = app_state.user_sessions.lock().unwrap();
            if let Some(user_session) = user_sessions.get(cookie.value()) {
                let username = user_session.username.to_string();
                let role = app_state.users.lock().unwrap().get(&username).unwrap().role.clone();
                if user_session.expiration > Utc::now() {
                    Ok(AuthenticatedUser {
                        username,
                        role

                    })
                } else {
                    Err((StatusCode::UNAUTHORIZED, "session expired".to_string()))
                }
            } else {
                Err((StatusCode::UNAUTHORIZED, "not a valid session cookie".to_string()))
            }
        } else {
            Err((StatusCode::UNAUTHORIZED, "you are not signed in".to_string()))
        }
    }
}
