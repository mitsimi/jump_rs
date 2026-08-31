use maud::{Markup, html};

use crate::models::Device;
use crate::web::icons::{Icon, icon};

use super::devices::device_grid;

#[derive(Debug, Clone, Copy)]
pub enum ToastKind {
    Success,
    Error,
}

impl ToastKind {
    const fn class_name(self) -> &'static str {
        match self {
            Self::Success => "toast__toast toast__success",
            Self::Error => "toast__toast toast__error",
        }
    }

    const fn icon(self) -> Icon {
        match self {
            Self::Success => Icon::Check,
            Self::Error => Icon::AlertCircle,
        }
    }
}

pub fn field_error(message: &str) -> Markup {
    html! {
        div class="error-message error-message--block" role="alert" { (message) }
    }
}

fn toast(kind: ToastKind, message: &str) -> Markup {
    html! {
        div
            class={ (kind.class_name()) " toast__show" } {
            span class="toast__icon" { (icon(kind.icon())) }
            span class="toast__message" { (message) }
        }
    }
}

fn toast_partial(kind: ToastKind, message: &str) -> Markup {
    html! {
        hx-partial hx-target="#toast-root" hx-swap="beforeend" {
            (toast(kind, message))
        }
    }
}

fn clear_modal_partial() -> Markup {
    html! {
        hx-partial hx-target="#modal-root" {}
    }
}

pub fn grid_with_toast(devices: &[Device], kind: ToastKind, message: &str) -> Markup {
    html! {
        hx-partial hx-target="#device-grid" hx-swap="outerMorph" {
            (device_grid(devices))
        }
        (toast_partial(kind, message))
        (clear_modal_partial())
    }
}

pub fn toast_fragment(kind: ToastKind, message: &str) -> Markup {
    toast_partial(kind, message)
}
