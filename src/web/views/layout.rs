use axum::http::StatusCode;
use maud::{DOCTYPE, Markup, html};

use crate::models::Device;
use crate::web::icons::{Icon, icon};

use super::devices::device_grid;

#[allow(clippy::needless_pass_by_value)]
pub fn layout(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                link rel="stylesheet" href="/static/app.css";
                script src="/static/vendor/htmx.min.js" defer {}
                script src="/static/vendor/alpine.min.js" defer {}
                script src="/static/app.js" defer {}
            }
            body {
                div class="container" {
                    (body)
                    (footer())
                }
                div id="modal-root" {}
                div
                    id="toast-root"
                    class="toast__container"
                    aria-live="polite"
                    aria-atomic="false"
                    aria-relevant="additions" {}
            }
        }
    }
}

pub fn home_page(devices: &[Device], username: Option<&str>) -> Markup {
    layout(
        "Jumpers",
        html! {
            (header(username))
            (device_grid(devices))
        },
    )
}

pub fn error_page(status: StatusCode, message: &str) -> Markup {
    layout(
        "Jumpers Error",
        html! {
            (header(None))
            section class="empty-grid" {
                div class="empty-banner" {
                    h2 class="empty-title" { "Request Failed" }
                    div class="empty-text" { (status.as_u16()) " " (message) }
                }
            }
        },
    )
}

fn header(username: Option<&str>) -> Markup {
    html! {
        header class="app-header" {
            div class="app-header__brand" {
                h1 class="app-header__title" {
                    "JUMP" span class="app-header__accent" { "ERS" }
                }
                p class="app-header__subtitle" { "Wake-on-LAN Control" }
            }
            div class="app-header__right" {
                button
                    class="app-header__data-btn"
                    type="button"
                    hx-get="/transfer"
                    hx-target="#modal-root"
                    hx-swap="innerHTML" {
                    (icon(Icon::Database))
                    "Import / Export"
                }
                @if let Some(username) = username {
                    form method="post" action="/logout" {
                        button
                            class="app-header__data-btn app-header__logout-btn"
                            type="submit"
                            aria-label=(format!("Sign out {username}"))
                            title="Sign out" {
                            span class="app-header__username" { (username) }
                            (icon(Icon::LogOut))
                        }
                    }
                }
            }
        }
    }
}

fn footer() -> Markup {
    let api_docs_enabled = crate::config::get().server.api_docs;

    html! {
        footer class="app-footer" {
            div class="app-footer__inner" {
                div class="app-footer__version" {
                    "jump.rs " (env!("CARGO_PKG_VERSION"))
                }
                @if api_docs_enabled {
                    nav class="app-footer__links" aria-label="Footer links" {
                        a class="app-footer__link" href="/api/swagger" { "API Docs" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authenticated_header_shows_username_and_logout_icon() {
        let markup = header(Some("alice")).into_string();

        assert!(markup.contains("alice"));
        assert!(markup.contains("aria-label=\"Sign out alice\""));
        assert!(markup.contains("m16 17 5-5-5-5"));
    }
}
