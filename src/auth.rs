use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    Form, Router,
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use maud::{DOCTYPE, Markup, html};
use parking_lot::Mutex;
use serde::Deserialize;

use crate::config::AuthConfig;

const SESSION_COOKIE: &str = "jumpers_session";
// A valid bcrypt hash used when the submitted username does not exist, so both
// failed-login paths perform the same expensive password check.
const DUMMY_PASSWORD_HASH: &str = "$2b$12$UdLYoJ5lgPsC0RKqYH/jMua7zIn0g9kPqWmhYayJYLaZQ/FTmH2/u";

#[derive(Clone)]
pub struct AuthState(Arc<AuthStateInner>);

struct AuthStateInner {
    users: HashMap<String, String>,
    sessions: Mutex<HashMap<String, Session>>,
    secure_cookie: bool,
    session_expiry: std::time::Duration,
    session_cookie_max_age: time::Duration,
}

struct Session {
    username: String,
    expires_at: Instant,
}

impl AuthState {
    pub fn from_config(config: &AuthConfig) -> Result<Option<Self>, String> {
        if !config.enabled {
            return Ok(None);
        }

        let users = parse_users(&config.users)?;
        if users.is_empty() {
            return Err("auth.enabled is true, but auth.users is empty".to_string());
        }
        if config.session_expiry_seconds == 0 {
            return Err("auth.session_expiry_seconds must be greater than zero".to_string());
        }
        let session_expiry = Duration::from_secs(config.session_expiry_seconds);
        let session_cookie_max_age = time::Duration::try_from(session_expiry)
            .map_err(|_| "auth.session_expiry_seconds is too large".to_string())?;
        if Instant::now().checked_add(session_expiry).is_none() {
            return Err("auth.session_expiry_seconds is too large".to_string());
        }

        Ok(Some(Self(Arc::new(AuthStateInner {
            users,
            sessions: Mutex::new(HashMap::new()),
            secure_cookie: config.secure_cookie,
            session_expiry,
            session_cookie_max_age,
        }))))
    }

    fn authenticated_user(&self, jar: &CookieJar) -> Option<String> {
        let token = jar.get(SESSION_COOKIE)?.value();
        let mut sessions = self.0.sessions.lock();
        let session = sessions.get(token)?;
        if session.expires_at <= Instant::now() {
            sessions.remove(token);
            return None;
        }
        let username = session.username.clone();
        drop(sessions);
        Some(username)
    }

    fn session_cookie(&self, token: String) -> Cookie<'static> {
        Cookie::build((SESSION_COOKIE, token))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(self.0.secure_cookie)
            .max_age(self.0.session_cookie_max_age)
            .build()
    }

    fn removal_cookie(&self) -> Cookie<'static> {
        Cookie::build(SESSION_COOKIE)
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(self.0.secure_cookie)
            .build()
    }
}

fn parse_users(value: &str) -> Result<HashMap<String, String>, String> {
    let mut users = HashMap::new();
    for entry in value
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        let (username, hash) = entry.split_once(':').ok_or_else(|| {
            format!("invalid auth user {entry:?}; expected username:password_hash")
        })?;
        if username.trim().is_empty() || hash.trim().is_empty() {
            return Err("auth usernames and password hashes must not be empty".to_string());
        }
        validate_bcrypt_hash(username, hash)?;
        if users
            .insert(username.trim().to_string(), hash.trim().to_string())
            .is_some()
        {
            return Err(format!("duplicate auth username {username:?}"));
        }
    }
    Ok(users)
}

fn validate_bcrypt_hash(username: &str, hash: &str) -> Result<(), String> {
    if !(hash.starts_with("$2a$") || hash.starts_with("$2b$") || hash.starts_with("$2y$")) {
        return Err(format!(
            "auth user {username:?} does not have a bcrypt password hash"
        ));
    }

    let parts = hash.parse::<bcrypt::HashParts>().map_err(|err| {
        format!("auth user {username:?} has an invalid bcrypt password hash: {err}")
    })?;
    if !(4..=31).contains(&parts.get_cost()) {
        return Err(format!("auth user {username:?} has an invalid bcrypt cost"));
    }

    let (_, payload) = hash
        .rsplit_once('$')
        .ok_or_else(|| format!("auth user {username:?} has an invalid bcrypt password hash"))?;
    let (salt, digest) = payload.split_at(22);
    let salt = bcrypt::BASE_64
        .decode(salt)
        .map_err(|_| format!("auth user {username:?} has an invalid bcrypt password hash"))?;
    let digest = bcrypt::BASE_64
        .decode(digest)
        .map_err(|_| format!("auth user {username:?} has an invalid bcrypt password hash"))?;
    if salt.len() != 16 || digest.len() != 23 {
        return Err(format!(
            "auth user {username:?} has an invalid bcrypt password hash"
        ));
    }

    Ok(())
}

pub fn public_router(state: AuthState) -> Router {
    Router::new()
        .route("/login", get(login_page).post(login))
        .route("/logout", post(logout))
        .with_state(state)
}

pub async fn require_authentication(
    State(state): State<AuthState>,
    jar: CookieJar,
    request: Request<Body>,
    next: Next,
) -> Response {
    let is_api = request.uri().path().starts_with("/api/");
    let basic_credentials = is_api.then(|| parse_basic_credentials(&request)).flatten();
    let basic_is_valid = if let Some((username, password)) = basic_credentials {
        verify_credentials(&state, &username, password).await
    } else {
        false
    };
    if state.authenticated_user(&jar).is_some() || basic_is_valid {
        return next.run(request).await;
    }

    let is_htmx = request.headers().contains_key("HX-Request");
    if is_api {
        return (
            StatusCode::UNAUTHORIZED,
            [(
                "WWW-Authenticate",
                "Basic realm=\"Jumpers\", charset=\"UTF-8\"",
            )],
            "authentication required",
        )
            .into_response();
    }
    if is_htmx {
        return (StatusCode::UNAUTHORIZED, [("HX-Redirect", "/login")]).into_response();
    }
    Redirect::to("/login").into_response()
}

fn parse_basic_credentials(request: &Request<Body>) -> Option<(String, String)> {
    let encoded = request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Basic "))?;
    let decoded = BASE64.decode(encoded).ok()?;
    let credentials = String::from_utf8(decoded).ok()?;
    let (username, password) = credentials.split_once(':')?;

    Some((username.to_string(), password.to_string()))
}

async fn verify_credentials(state: &AuthState, username: &str, password: String) -> bool {
    let hash = state
        .0
        .users
        .get(username.trim())
        .map_or_else(|| DUMMY_PASSWORD_HASH.to_string(), Clone::clone);
    let username_exists = state.0.users.contains_key(username.trim());

    tokio::task::spawn_blocking(move || bcrypt::verify(password, &hash).unwrap_or(false))
        .await
        .unwrap_or(false)
        && username_exists
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login_page(State(state): State<AuthState>, jar: CookieJar) -> Response {
    if state.authenticated_user(&jar).is_some() {
        return Redirect::to("/").into_response();
    }
    login_view(None).into_response()
}

async fn login(
    State(state): State<AuthState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Response {
    let valid = verify_credentials(&state, &form.username, form.password).await;

    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            login_view(Some("Invalid username or password")),
        )
            .into_response();
    }

    let token = nanoid::nanoid!(48);
    let mut sessions = state.0.sessions.lock();
    sessions.retain(|_, session| session.expires_at > Instant::now());
    sessions.insert(
        token.clone(),
        Session {
            username: form.username.trim().to_string(),
            expires_at: Instant::now() + state.0.session_expiry,
        },
    );
    drop(sessions);
    (jar.add(state.session_cookie(token)), Redirect::to("/")).into_response()
}

async fn logout(State(state): State<AuthState>, jar: CookieJar) -> Response {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        state.0.sessions.lock().remove(cookie.value());
    }
    (jar.remove(state.removal_cookie()), Redirect::to("/login")).into_response()
}

fn login_view(error: Option<&str>) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Sign in · Jumpers" }
                link rel="stylesheet" href="/static/app.css";
            }
            body {
                main class="auth-shell" {
                    section class="auth-card" {
                        div class="auth-brand" { "JUMP" span { "ERS" } }
                        p class="auth-kicker" { "Wake-on-LAN Control" }
                        h1 { "Sign in" }
                        @if let Some(error) = error { p class="auth-error" role="alert" { (error) } }
                        form method="post" action="/login" class="auth-form" {
                            label for="username" { "Username" }
                            input id="username" name="username" type="text" autocomplete="username" required autofocus;
                            label for="password" { "Password" }
                            input id="password" name="password" type="password" autocomplete="current-password" required;
                            button type="submit" class="btn btn-primary" { "Sign in" }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, header},
        routing::get,
    };
    use tower::ServiceExt;

    fn enabled_state() -> AuthState {
        let hash = bcrypt::hash("correct horse", 4).unwrap();
        AuthState::from_config(&AuthConfig {
            enabled: true,
            users: format!("alice:{hash}"),
            secure_cookie: false,
            session_expiry_seconds: 60,
        })
        .unwrap()
        .unwrap()
    }

    fn protected_app(state: AuthState) -> Router {
        let protected = Router::new()
            .route("/private", get(|| async { "ok" }))
            .route("/api/private", get(|| async { "ok" }))
            .route_layer(axum::middleware::from_fn_with_state(
                state.clone(),
                require_authentication,
            ));
        Router::new().merge(public_router(state)).merge(protected)
    }

    #[test]
    fn parses_tinyauth_style_users() {
        let alice = bcrypt::hash("alice-password", 4).unwrap();
        let users = parse_users(&format!(
            "alice:{alice},bob:$2y$04$I.jf5EIXCDllPCFc0zv0kuXcgTswNFNYGjxr9Wo/TcP.TMpIZoq4O"
        ))
        .unwrap();
        assert_eq!(users.len(), 2);
        assert!(users.contains_key("alice"));
        assert!(users.contains_key("bob"));
    }

    #[test]
    fn rejects_non_bcrypt_hashes() {
        assert!(parse_users("alice:plaintext").is_err());
    }

    #[test]
    fn rejects_malformed_bcrypt_hashes() {
        assert!(parse_users("alice:$2b$12$not-a-complete-hash").is_err());
        assert!(
            parse_users("alice:$2b$12$!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!")
                .is_err()
        );
    }

    #[test]
    fn rejects_session_expiry_that_cannot_be_represented() {
        let config = AuthConfig {
            enabled: true,
            users: format!("alice:{}", bcrypt::hash("password", 4).unwrap()),
            secure_cookie: false,
            session_expiry_seconds: u64::MAX,
        };
        assert!(AuthState::from_config(&config).is_err());
    }

    #[test]
    fn disabled_auth_ignores_auth_specific_values() {
        let config = AuthConfig {
            enabled: false,
            users: "not-a-valid-user".to_string(),
            secure_cookie: false,
            session_expiry_seconds: u64::MAX,
        };
        assert!(AuthState::from_config(&config).unwrap().is_none());
    }

    #[tokio::test]
    async fn redirects_pages_but_returns_unauthorized_for_api() {
        let app = protected_app(enabled_state());
        let page = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/private")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(page.status(), StatusCode::SEE_OTHER);
        assert_eq!(page.headers().get(header::LOCATION).unwrap(), "/login");

        let api = app
            .oneshot(
                Request::builder()
                    .uri("/api/private")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(api.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn valid_login_cookie_unlocks_protected_routes() {
        let app = protected_app(enabled_state());
        let login = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/login")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from("username=alice&password=correct+horse"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(login.status(), StatusCode::SEE_OTHER);
        let cookie = login
            .headers()
            .get(header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Lax"));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/private")
                    .header(header::COOKIE, cookie.split(';').next().unwrap())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn valid_basic_auth_unlocks_api_only() {
        let app = protected_app(enabled_state());
        let credentials = BASE64.encode("alice:correct horse");
        let api = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/private")
                    .header(header::AUTHORIZATION, format!("Basic {credentials}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(api.status(), StatusCode::OK);

        let page = app
            .oneshot(
                Request::builder()
                    .uri("/private")
                    .header(header::AUTHORIZATION, format!("Basic {credentials}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(page.status(), StatusCode::SEE_OTHER);
    }

    #[tokio::test]
    async fn invalid_basic_auth_is_challenged() {
        let app = protected_app(enabled_state());
        let credentials = BASE64.encode("alice:wrong");
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/private")
                    .header(header::AUTHORIZATION, format!("Basic {credentials}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.headers().get(header::WWW_AUTHENTICATE).unwrap(),
            "Basic realm=\"Jumpers\", charset=\"UTF-8\""
        );
    }
}
