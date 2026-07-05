use axum::http::StatusCode;
use maud::{DOCTYPE, Markup, html};

use crate::models::Device;
use crate::web::icons::{Icon, icon, icon_with_label};

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
                (body)
            }
        }
    }
}

pub fn home_page(devices: &[Device]) -> Markup {
    layout(
        "Jumpers",
        html! {
            div class="container" {
                (header())
                (device_grid(devices))
            }
            div id="modal-root" {}
            div id="toast-root" class="toast__container" {}
        },
    )
}

pub fn error_page(status: StatusCode, message: &str) -> Markup {
    layout(
        "Jumpers Error",
        html! {
            div class="container" {
                (header())
                section class="empty-grid" {
                    div class="empty-banner" {
                        div class="empty-title" { "Request Failed" }
                        div class="empty-text" { (status.as_u16()) " " (message) }
                    }
                }
            }
        },
    )
}

pub fn header() -> Markup {
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
            }
        }
    }
}

pub fn device_grid(devices: &[Device]) -> Markup {
    html! {
        section id="device-grid" class="device-grid__section" {
            div class="device-grid__header" {
                h2 class="device-grid__title" { "Controlled Devices" }
                button
                    class="device-grid__add-btn"
                    type="button"
                    hx-get="/devices/new"
                    hx-target="#modal-root"
                    hx-swap="innerHTML" {
                    (icon(Icon::Plus))
                    span { "Add Device" }
                }
            }

            @if devices.is_empty() {
                (empty_state())
            } @else {
                div class="device-grid__grid" {
                    @for device in devices {
                        (device_card(device))
                    }
                }
            }
        }
    }
}

pub fn empty_state() -> Markup {
    html! {
        div class="device-grid__empty-grid" {
            div class="device-grid__empty-banner" {
                div class="device-grid__empty-icon" { (icon(Icon::Plus)) }
                div class="device-grid__empty-title" { "No Devices Found" }
                div class="device-grid__empty-text" {
                    "Add your first device to start controlling your network wake capabilities."
                }
            }
        }
    }
}

pub fn device_card(device: &Device) -> Markup {
    let short_id: String = device.id.chars().take(8).collect();
    html! {
        article class="device-card" id={ "device-" (device.id) } {
            div class="device-card__header" {
                div class="device-card__name" { (device.name) }
                div class="device-card__id" { (short_id) }
            }

            div class="device-card__info" {
                div class="device-card__info-row" {
                    span class="device-card__label" { "MAC" }
                    span class="device-card__value device-card__value--mac" { (device.mac_address) }
                }
                @if let Some(ip_address) = &device.ip_address {
                    div class="device-card__info-row" {
                        span class="device-card__label" { "IP" }
                        span class="device-card__value" { (ip_address) }
                    }
                }
                div class="device-card__info-row" {
                    span class="device-card__label" { "Port" }
                    span class="device-card__value" { (device.port) }
                }
                @if let Some(description) = &device.description {
                    div class="device-card__info-row" {
                        span class="device-card__label" { "Note" }
                        span class="device-card__value" { (description) }
                    }
                }
            }

            div class="device-card__actions" {
                button
                    class="btn btn-primary"
                    type="button"
                    hx-post={ "/devices/" (device.id) "/wake" }
                    hx-target="#toast-root"
                    hx-swap="beforeend" {
                    (icon(Icon::Power))
                    "Wake"
                }
                button
                    class="btn btn-secondary"
                    type="button"
                    hx-get={ "/devices/" (device.id) "/edit" }
                    hx-target="#modal-root"
                    hx-swap="innerHTML" {
                    (icon(Icon::Pencil))
                    "Edit"
                }
                button
                    class="btn btn-danger"
                    type="button"
                    hx-post={ "/devices/" (device.id) "/delete" }
                    hx-target="#device-grid"
                    hx-swap="outerHTML"
                    hx-confirm={ "Remove " (device.name) "?" } {
                    (icon(Icon::Trash2))
                    "Remove"
                }
            }
        }
    }
}

pub fn device_modal(device: Option<&Device>, error: Option<&str>) -> Markup {
    let is_edit = device.is_some();
    let title = if is_edit { "Edit Device" } else { "Add Device" };
    let action = device.map_or_else(
        || "/devices".to_string(),
        |device| format!("/devices/{}/update", device.id),
    );
    let name = device.map_or("", |device| device.name.as_str());
    let mac = device.map_or("", |device| device.mac_address.as_str());
    let ip = device
        .and_then(|device| device.ip_address.as_deref())
        .unwrap_or("");
    let port = device.map_or_else(
        || crate::config::get().wol.default_port.to_string(),
        |device| device.port.to_string(),
    );
    let description = device
        .and_then(|device| device.description.as_deref())
        .unwrap_or("");

    html! {
        div
            class="modal__overlay"
            x-data="{}"
            onclick="if (event.target === this) jumpCloseModal()" {
            div class="modal modal__medium" role="dialog" aria-modal="true" {
                div class="modal__header" {
                    h2 class="modal__title" { (title) }
                    button
                        class="modal__close-btn"
                        type="button"
                        onclick="jumpCloseModal()"
                        aria-label="Close" {
                        (icon_with_label(Icon::X))
                    }
                }
                form
                    hx-post=(action)
                    hx-target="#device-grid"
                    hx-swap="outerHTML" {
                    div class="modal__body" {
                        @if let Some(error) = error {
                            (field_error(error))
                        }
                        div class="form-group" {
                            label class="form-label" for="device-name" { "Device Name" }
                            input id="device-name" class="form-input" name="name" placeholder="e.g., Gaming PC" value=(name) required;
                        }
                        div class="form-group" {
                            label class="form-label" for="mac-address" {
                                "MAC Address"
                                span class="form-hint" { "AA:BB:CC:DD:EE:FF" }
                            }
                            div id="mac-lookup-result" class="mac-input-wrapper" {
                                (mac_lookup_controls(mac))
                            }
                        }
                        div class="form-row" {
                            div class="form-group form-group--flush" {
                                label class="form-label" for="ip-address" {
                                    "IP Address " span class="form-hint" { "(optional)" }
                                }
                                input id="ip-address" class="form-input" name="ip_address" placeholder="192.168.1.100" value=(ip);
                            }
                            div class="form-group form-group--flush" {
                                label class="form-label" for="device-port" { "Port" }
                                input id="device-port" class="form-input" type="number" min="1" max="65535" name="port" placeholder="9" value=(port);
                            }
                        }
                        div class="form-group form-group--spaced" {
                            label class="form-label" for="description" {
                                "Description " span class="form-hint" { "(optional)" }
                            }
                            input id="description" class="form-input" name="description" placeholder="Notes about this device..." value=(description);
                        }
                    }
                    div class="modal__footer" {
                        button
                            type="button"
                            class="btn btn-secondary"
                            onclick="jumpCloseModal()" {
                            "Cancel"
                        }
                        button type="submit" class="btn btn-primary" {
                            @if is_edit { "Save Changes" } @else { "Add Device" }
                        }
                    }
                }
            }
        }
    }
}

pub fn mac_lookup_controls(mac: &str) -> Markup {
    html! {
        div class="mac-input-control" {
            input
                id="mac-address"
                class="form-input form-input--with-action"
                name="mac_address"
                placeholder="AA:BB:CC:DD:EE:FF"
                value=(mac)
                required;
            button
                type="button"
                class="lookup-btn"
                hx-post="/arp-lookup"
                hx-include="closest form"
                hx-target="#mac-lookup-result"
                hx-swap="innerHTML" {
                (icon(Icon::Search))
                "Lookup"
            }
        }
    }
}

pub fn mac_lookup_error(mac: &str, error: &str) -> Markup {
    mac_lookup_error_with_hint(mac, error, None)
}

pub fn mac_lookup_error_with_hint(mac: &str, error: &str, hint: Option<&str>) -> Markup {
    html! {
        (mac_lookup_controls(mac))
        span class="error-message error-message--inline" {
            span { (error) }
            @if let Some(hint) = hint {
                span class="tooltip" {
                    button
                        class="tooltip__trigger"
                        type="button"
                        aria-label="Why this happens"
                        aria-describedby="mac-lookup-error-hint" {
                        (icon(Icon::Info))
                    }
                    span id="mac-lookup-error-hint" class="tooltip__content" role="tooltip" {
                        (hint)
                    }
                }
            }
        }
    }
}

pub fn transfer_modal(error: Option<&str>) -> Markup {
    html! {
        div
            class="modal__overlay"
            x-data="{}"
            onclick="if (event.target === this) jumpCloseModal()" {
            div class="modal modal__large" role="dialog" aria-modal="true" {
                div class="modal__header" {
                    h2 class="modal__title" { "DATA TRANSFER" }
                    button
                        class="modal__close-btn"
                        type="button"
                        onclick="jumpCloseModal()"
                        aria-label="Close" {
                        (icon_with_label(Icon::X))
                    }
                }
                div class="transfer__modal-body" {
                    div class="transfer__tabs" {
                        button id="transfer-export-tab" type="button" class="transfer__tab transfer__tab--active" onclick="jumpShowTransferTab('export')" { "EXPORT" }
                        button id="transfer-import-tab" type="button" class="transfer__tab" onclick="jumpShowTransferTab('import')" { "IMPORT" }
                    }

                    div class="transfer__content" {
                        div id="transfer-export-panel" class="transfer__export-section" {
                            div class="transfer__export-icon" { (icon(Icon::Download)) }
                            p class="transfer__description" {
                                "Export all registered devices to a JSON file. The file contains device names, MAC addresses, IP addresses, ports, and descriptions."
                            }
                            a class="transfer__action-btn" href="/devices/export" download {
                                (icon(Icon::Download))
                                "DOWNLOAD JSON"
                            }
                        }

                        form
                            id="transfer-import-panel"
                            class="transfer__import-section"
                            hidden
                            hx-post="/devices/import"
                            hx-target="#device-grid"
                            hx-swap="outerHTML" {
                            @if let Some(error) = error {
                                (field_error(error))
                            }
                            div class="transfer__import-options" {
                                div
                                    id="transfer-drop-zone"
                                    class="transfer__drop-zone"
                                    ondragover="jumpImportDragOver(event)"
                                    ondragleave="jumpImportDragLeave(event)"
                                    ondrop="jumpImportDrop(event)" {
                                    input
                                        id="json-file"
                                        class="transfer__file-input"
                                        type="file"
                                        accept=".json"
                                        onchange="jumpLoadImportFile(event.target.files[0])";
                                    button
                                        class="transfer__upload-btn"
                                        type="button"
                                        onclick="document.getElementById('json-file').click()" {
                                        (icon(Icon::Upload))
                                        "Upload JSON File"
                                    }
                                    div class="transfer__drop-text" {
                                        span id="transfer-file-name" { "or drag and drop file here" }
                                    }
                                }

                                div class="transfer__divider" { span { "OR" } }

                                div class="transfer__import-option" {
                                    textarea
                                        class="transfer__json-input"
                                        name="payload"
                                        id="import-payload"
                                        rows="6"
                                        placeholder="[{\"name\":\"Device\",\"mac_address\":\"aa:bb:cc:dd:ee:ff\",\"ip_address\":\"192.168.1.100\",\"port\":9}]" {}
                                    button
                                        class="transfer__action-btn"
                                        type="submit" {
                                        (icon(Icon::Upload))
                                        "Import devices"
                                    }
                                }
                            }
                        }
                    }
                    div class="transfer__footer" {
                        div class="transfer__format-hint" {
                            "Expected JSON format:"
                            code { "[{\"name\":\"Name\",\"mac_address\":\"aa:bb:cc:dd:ee:ff\",\"ip_address\":\"1.2.3.4\",\"port\":9,\"description\":\"...\"}]" }
                        }
                    }
                }
            }
        }
    }
}

pub fn field_error(message: &str) -> Markup {
    html! {
        div class="error-message error-message--block" { (message) }
    }
}

pub fn toast(kind: ToastKind, message: &str) -> Markup {
    html! {
        div
            class={ (kind.class_name()) " toast__show" } {
            span class="toast__icon" { (icon(kind.icon())) }
            span class="toast__message" { (message) }
        }
    }
}

pub fn toast_oob(kind: ToastKind, message: &str) -> Markup {
    html! {
        div id="toast-root" hx-swap-oob="beforeend" {
            (toast(kind, message))
        }
    }
}

pub fn clear_modal_oob() -> Markup {
    html! {
        div id="modal-root" hx-swap-oob="innerHTML" {}
    }
}

pub fn grid_with_toast(devices: &[Device], kind: ToastKind, message: &str) -> Markup {
    html! {
        (device_grid(devices))
        (toast_oob(kind, message))
        (clear_modal_oob())
    }
}

pub fn toast_fragment(kind: ToastKind, message: &str) -> Markup {
    toast(kind, message)
}
