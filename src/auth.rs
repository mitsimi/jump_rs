use axum::{
    Json,
    body::Body,
    extract::{FromRequestParts, State},
    http::{Request, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use base64::prelude::*;
use parking_lot::RwLock;
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tracing::{instrument, warn};
use utoipa::ToSchema;

use crate::app::AppState;
use crate::config::AuthConfig;
use crate::error::ErrorResponse;

pub const SESSION_COOKIE_NAME: &str = "session_id";

pub fn validate_auth_config(config: &AuthConfig) -> Result<(), String> {
    if !config.disabled && has_global_wildcard(&config.allow_origins) {
        return Err(
            "Invalid auth.allow_origins: '*' is not allowed when auth.disabled=false. Use explicit origins or single-wildcard patterns such as 'http://localhost:*'.".to_string(),
        );
    }

    Ok(())
}

pub fn has_global_wildcard(allow_origins: &[String]) -> bool {
    allow_origins.iter().any(|origin| origin.trim() == "*")
}

pub fn is_allowed_origin(origin: &str, allow_origins: &[String]) -> bool {
    if allow_origins.is_empty() {
        return false;
    }

    allow_origins
        .iter()
        .any(|allowed| origin_matches(allowed, origin))
}

pub fn origin_matches(allowed: &str, origin: &str) -> bool {
    if allowed == "*" {
        return true;
    }

    if !allowed.contains('*') {
        return normalize_origin(allowed) == normalize_origin(origin);
    }

    let allowed = normalize_origin(allowed);
    let origin = normalize_origin(origin);

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

fn normalize_origin(origin: &str) -> &str {
    origin.trim().trim_end_matches('/')
}

#[derive(Clone)]
pub struct UserStore {
    users: HashMap<String, String>,
}

impl UserStore {
    pub fn from_config(users_config: &str) -> Self {
        let mut users = HashMap::new();

        if users_config.is_empty() {
            return Self { users };
        }

        let normalized = users_config.replace("$$", "$");

        for entry in normalized.split(',') {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }

            if let Some((username, hash)) = entry.split_once(':') {
                let username = username.trim();
                let hash = hash.trim();

                if username.is_empty() {
                    warn!(entry = %entry, "Skipping user entry: empty username");
                    continue;
                }
                if hash.is_empty() {
                    warn!(username = %username, "Skipping user entry: empty password hash");
                    continue;
                }

                users.insert(username.to_string(), hash.to_string());
            } else {
                warn!(entry = %entry, "Skipping user entry: invalid format (expected username:hash)");
            }
        }

        Self { users }
    }

    pub fn verify_password(&self, username: &str, password: &str) -> bool {
        self.users.get(username).map_or_else(
            || {
                bcrypt::verify(
                    password,
                    "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4beWo3z6eHMy3yOS",
                )
                .ok();
                false
            },
            |hash| bcrypt::verify(password, hash).unwrap_or(false),
        )
    }

    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    pub fn user_count(&self) -> usize {
        self.users.len()
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub username: String,
    pub expires_at: SystemTime,
}

#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    timeout: Duration,
}

impl SessionManager {
    pub fn new(timeout_seconds: u64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            timeout: Duration::from_secs(timeout_seconds),
        }
    }

    pub fn create_session(&self, username: &str) -> String {
        let session_id = nanoid::nanoid!(32);
        let session = Session {
            username: username.to_string(),
            expires_at: SystemTime::now() + self.timeout,
        };

        self.sessions.write().insert(session_id.clone(), session);
        session_id
    }

    pub fn validate_session(&self, session_id: &str) -> Option<String> {
        let sessions = self.sessions.read();
        sessions
            .get(session_id)
            .filter(|session| session.expires_at > SystemTime::now())
            .map(|session| session.username.clone())
    }

    pub fn invalidate_session(&self, session_id: &str) {
        self.sessions.write().remove(session_id);
    }

    pub fn cleanup_expired(&self) {
        let now = SystemTime::now();
        self.sessions
            .write()
            .retain(|_, session| session.expires_at > now);
    }
}

#[derive(Clone)]
pub struct AuthState {
    pub user_store: UserStore,
    pub session_manager: SessionManager,
}

impl AuthState {
    pub const fn new(user_store: UserStore, session_manager: SessionManager) -> Self {
        Self {
            user_store,
            session_manager,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthenticatedUser {
    pub username: String,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts.extensions.get().cloned().ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(ErrorResponse {
                    status: "error".to_string(),
                    message: "Authentication required".to_string(),
                }),
            )
        })
    }
}

fn is_public_path(path: &str) -> bool {
    path == "/api/auth/login"
        || path == "/api/auth/logout"
        || path == "/api/auth/status"
        || path == "/api/swagger"
        || path.starts_with("/api/swagger/")
        || path == "/api/docs/openapi.json"
}

fn extract_basic_auth(request: &Request<Body>) -> Option<(String, String)> {
    let auth_header = request.headers().get(header::AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;

    let (scheme, encoded) = auth_str.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("Basic") {
        return None;
    }

    let decoded = String::from_utf8(BASE64_STANDARD.decode(encoded).ok()?).ok()?;
    let (username, password) = decoded.split_once(':')?;
    Some((username.to_string(), password.to_string()))
}

#[instrument(skip_all, fields(path = %request.uri().path()))]
pub async fn auth_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path();

    if !path.starts_with("/api/") {
        return next.run(request).await;
    }

    if state.config.auth.disabled {
        tracing::debug!("Auth disabled, skipping authentication");
        return next.run(request).await;
    }

    state.auth_state.session_manager.cleanup_expired();

    if is_public_path(path) {
        tracing::debug!("Public path, skipping authentication");
        return next.run(request).await;
    }

    if let Some(session_cookie) = jar.get(SESSION_COOKIE_NAME) {
        let session_id = session_cookie.value();
        if let Some(username) = state
            .auth_state
            .session_manager
            .validate_session(session_id)
        {
            request
                .extensions_mut()
                .insert(AuthenticatedUser { username });
            return next.run(request).await;
        }
    }

    if let Some((username, password)) = extract_basic_auth(&request)
        && state
            .auth_state
            .user_store
            .verify_password(&username, &password)
    {
        request
            .extensions_mut()
            .insert(AuthenticatedUser { username });
        return next.run(request).await;
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            status: "error".to_string(),
            message: "Authentication required".to_string(),
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_store_parse_valid() {
        let store = UserStore::from_config("user1:hash1,user2:hash2");
        assert_eq!(store.user_count(), 2);
    }

    #[test]
    fn test_user_store_parse_empty() {
        let store = UserStore::from_config("");
        assert!(store.is_empty());
    }

    #[test]
    fn test_user_store_parse_with_spaces() {
        let store = UserStore::from_config(" user1 : hash1 , user2 : hash2 ");
        assert_eq!(store.user_count(), 2);
    }

    #[test]
    fn test_user_store_skip_invalid() {
        let store =
            UserStore::from_config("user1:hash1,invalid,user2:hash2,:nousername,nopassword:");
        assert_eq!(store.user_count(), 2);
    }

    #[test]
    fn test_user_store_docker_compose_escape() {
        let store = UserStore::from_config("user1:$$2b$$12$$hash");
        assert_eq!(store.user_count(), 1);
        assert!(store.users.get("user1").unwrap().contains("$2b$12$"));
    }

    #[test]
    fn test_session_manager_create_validate() {
        let manager = SessionManager::new(3600);
        let session_id = manager.create_session("testuser");

        let username = manager.validate_session(&session_id);
        assert_eq!(username, Some("testuser".to_string()));
    }

    #[test]
    fn test_session_manager_invalidate() {
        let manager = SessionManager::new(3600);
        let session_id = manager.create_session("testuser");

        manager.invalidate_session(&session_id);
        assert!(manager.validate_session(&session_id).is_none());
    }

    #[test]
    fn test_session_manager_invalid_session() {
        let manager = SessionManager::new(3600);
        assert!(manager.validate_session("nonexistent").is_none());
    }

    #[test]
    fn test_is_public_path() {
        assert!(is_public_path("/api/auth/login"));
        assert!(is_public_path("/api/auth/logout"));
        assert!(is_public_path("/api/auth/status"));
        assert!(is_public_path("/api/swagger"));
        assert!(is_public_path("/api/swagger/"));
        assert!(is_public_path("/api/docs/openapi.json"));

        assert!(!is_public_path("/api/auth/me"));
        assert!(!is_public_path("/api/devices"));
        assert!(!is_public_path("/api/devices/123"));
    }

    #[test]
    fn test_origin_matches_exact() {
        assert!(origin_matches(
            "http://localhost:3000",
            "http://localhost:3000"
        ));
        assert!(origin_matches(
            "http://localhost:3000/",
            "http://localhost:3000"
        ));
        assert!(!origin_matches(
            "http://localhost:3000",
            "http://localhost:5173"
        ));
    }

    #[test]
    fn test_origin_matches_single_wildcard() {
        assert!(origin_matches(
            "http://localhost:*",
            "http://localhost:5173"
        ));
        assert!(origin_matches(
            "https://*.example.com",
            "https://api.example.com"
        ));
        assert!(!origin_matches(
            "http://localhost:*",
            "http://localhost:5173/path"
        ));
    }

    #[test]
    fn test_origin_matches_rejects_multi_wildcard_patterns() {
        assert!(!origin_matches("http://*:*", "http://localhost:5173"));
        assert!(!origin_matches("*://localhost:*", "http://localhost:5173"));
    }

    #[test]
    fn test_validate_auth_config_rejects_global_wildcard_when_enabled() {
        let config = AuthConfig {
            disabled: false,
            users: String::new(),
            session_timeout: 3600,
            allow_insecure_cookies: false,
            allow_origins: vec!["*".to_string()],
        };

        assert!(validate_auth_config(&config).is_err());
    }
}
