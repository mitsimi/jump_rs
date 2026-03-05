use axum::http::Uri;
use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use time::Duration;
use tracing::{info, instrument, warn};
use utoipa::{OpenApi, ToSchema};

use crate::app::AppState;
use crate::auth::{AuthenticatedUser, SESSION_COOKIE_NAME};
use crate::error::ErrorResponse;

#[derive(OpenApi)]
#[openapi(
    paths(login, logout, me, auth_status),
    components(schemas(LoginRequest, LoginResponse, MeResponse, AuthStatusResponse, ErrorResponse)),
    tags(
        (name = "auth", description = "Authentication endpoints")
    )
)]
pub struct AuthApiDoc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(me))
        .route("/api/auth/status", get(auth_status))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "admin")]
    pub username: String,
    #[schema(example = "password123")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    #[schema(example = "success")]
    pub status: String,
    #[schema(example = "admin")]
    pub username: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeResponse {
    #[schema(example = "admin")]
    pub username: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthStatusResponse {
    /// Whether authentication is required (true = auth enabled, false = auth disabled).
    #[schema(example = true)]
    pub auth_required: bool,
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    operation_id = "login",
    tag = "auth",
    summary = "Login with username and password",
    description = "Authenticates a user and returns a session cookie.",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse)
    )
)]
#[instrument(skip_all, fields(username = %payload.username))]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: axum::http::HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> axum::response::Response {
    if !is_same_origin(&headers, &state) {
        warn!(
            origin = ?headers.get(axum::http::header::ORIGIN),
            host = ?headers.get(axum::http::header::HOST),
            "Blocked auth request from mismatched origin"
        );
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "status": "error",
                "message": "Origin not allowed.",
                "code": "origin_not_allowed",
            })),
        )
            .into_response();
    }

    if !state
        .auth_state
        .user_store
        .verify_password(&payload.username, &payload.password)
    {
        warn!("Failed login attempt");
        let removal_cookie = Cookie::build((SESSION_COOKIE_NAME, ""))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(!state.config.auth.allow_insecure_cookies)
            .max_age(Duration::ZERO)
            .build();

        return (
            StatusCode::UNAUTHORIZED,
            jar.remove(removal_cookie),
            Json(serde_json::json!({
                "status": "error",
                "message": "Invalid username or password"
            })),
        )
            .into_response();
    }

    let session_id = state
        .auth_state
        .session_manager
        .create_session(&payload.username);
    let timeout_seconds = state.config.auth.session_timeout;

    let cookie = Cookie::build((SESSION_COOKIE_NAME, session_id))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(!state.config.auth.allow_insecure_cookies)
        .max_age(Duration::seconds(timeout_seconds.cast_signed()))
        .build();

    info!("User logged in successfully");

    (
        StatusCode::OK,
        jar.add(cookie),
        Json(serde_json::json!({
            "status": "success",
            "username": payload.username
        })),
    )
        .into_response()
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    operation_id = "logout",
    tag = "auth",
    summary = "Logout and invalidate session",
    description = "Invalidates the current session and clears the session cookie.",
    responses(
        (status = 200, description = "Logout successful", body = inline(serde_json::Value))
    )
)]
#[instrument(skip_all)]
pub async fn logout(
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    jar: CookieJar,
) -> axum::response::Response {
    if !is_same_origin(&headers, &state) {
        warn!(
            origin = ?headers.get(axum::http::header::ORIGIN),
            host = ?headers.get(axum::http::header::HOST),
            "Blocked auth request from mismatched origin"
        );
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "status": "error",
                "message": "Origin not allowed.",
                "code": "origin_not_allowed",
            })),
        )
            .into_response();
    }

    if let Some(session_cookie) = jar.get(SESSION_COOKIE_NAME) {
        let session_id = session_cookie.value();
        state
            .auth_state
            .session_manager
            .invalidate_session(session_id);
        info!("User logged out");
    }

    let removal_cookie = Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(!state.config.auth.allow_insecure_cookies)
        .max_age(Duration::ZERO)
        .build();

    (
        StatusCode::OK,
        jar.remove(removal_cookie),
        Json(serde_json::json!({
            "status": "success",
            "message": "Logged out successfully"
        })),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    operation_id = "me",
    tag = "auth",
    summary = "Get current authenticated user",
    description = "Returns information about the currently authenticated user.",
    responses(
        (status = 200, description = "Current user info", body = MeResponse),
        (status = 401, description = "Not authenticated", body = ErrorResponse)
    ),
    security(
        ("session_cookie" = []),
        ("basic_auth" = [])
    )
)]
#[instrument(skip_all)]
pub async fn me(user: AuthenticatedUser) -> Json<MeResponse> {
    Json(MeResponse {
        username: user.username,
    })
}

#[utoipa::path(
    get,
    path = "/api/auth/status",
    operation_id = "auth_status",
    tag = "auth",
    summary = "Get authentication status",
    description = "Returns whether authentication is required for this instance.",
    responses(
        (status = 200, description = "Auth status", body = AuthStatusResponse)
    )
)]
#[instrument(skip_all)]
pub async fn auth_status(State(state): State<AppState>) -> Json<AuthStatusResponse> {
    Json(AuthStatusResponse {
        auth_required: !state.config.auth.disabled,
    })
}

fn is_same_origin(headers: &axum::http::HeaderMap, state: &AppState) -> bool {
    let origin = headers
        .get(axum::http::header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    let host = headers
        .get(axum::http::header::HOST)
        .and_then(|value| value.to_str().ok());

    match (origin, host) {
        (None, _) => true,
        (Some(origin), Some(host)) => {
            same_origin_match(origin, host) || is_allowed_origin(origin, state)
        }
        _ => false,
    }
}

fn is_allowed_origin(origin: &str, state: &AppState) -> bool {
    if state.config.auth.allow_origins.is_empty() {
        return false;
    }

    state
        .config
        .auth
        .allow_origins
        .iter()
        .any(|allowed| origin_matches(allowed, origin))
}

fn origin_matches(allowed: &str, origin: &str) -> bool {
    if allowed == "*" {
        return true;
    }

    if !allowed.contains('*') {
        return allowed == origin;
    }

    let allowed = allowed.trim_end_matches('/');
    let origin = origin.trim_end_matches('/');

    if allowed == "*" {
        return true;
    }

    let parts: Vec<&str> = allowed.split('*').collect();
    if parts.len() != 2 {
        return false;
    }

    let prefix = parts[0];
    let suffix = parts[1];
    if !origin.starts_with(prefix) {
        return false;
    }

    if !suffix.is_empty() && !origin.ends_with(suffix) {
        return false;
    }

    if origin.len() < prefix.len() + suffix.len() {
        return false;
    }

    let middle = &origin[prefix.len()..origin.len() - suffix.len()];
    !middle.contains('/')
}

fn same_origin_match(origin: &str, host: &str) -> bool {
    let origin = origin.trim_end_matches('/');
    let origin_uri: Uri = match origin.parse() {
        Ok(uri) => uri,
        Err(_) => return false,
    };
    let Some(origin_authority) = origin_uri.authority() else {
        return false;
    };

    let host_uri: Uri = match format!("http://{host}").parse() {
        Ok(uri) => uri,
        Err(_) => return false,
    };
    let Some(host_authority) = host_uri.authority() else {
        return false;
    };

    if origin_authority.host() != host_authority.host() {
        return false;
    }

    let default_port = match origin_uri.scheme_str() {
        Some("https") => Some(443),
        Some("http") => Some(80),
        _ => None,
    };
    let origin_port = origin_authority.port_u16().or(default_port);
    let host_port = host_authority.port_u16().or(default_port);

    origin_port == host_port
}
