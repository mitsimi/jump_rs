use maud::{Markup, html};

use crate::models::Device;
use crate::web::icons::{Icon, icon, icon_with_label};

use super::feedback::field_error;

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

fn empty_state() -> Markup {
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

fn device_card(device: &Device) -> Markup {
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
                    hx-swap="innerHTML"
                    aria-label="Edit device"
                    title="Edit device" {
                    (icon(Icon::Pencil))
                    span class="device-card__action-label" { "Edit" }
                }
                button
                    class="btn btn-danger device-card__icon-action"
                    type="button"
                    hx-post={ "/devices/" (device.id) "/delete" }
                    hx-target="#device-grid"
                    hx-swap="outerHTML"
                    hx-confirm={ "Remove " (device.name) "?" }
                    aria-label="Remove device"
                    title="Remove device" {
                    (icon(Icon::Trash2))
                    span class="device-card__action-label" { "Remove" }
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
        dialog
            class="modal modal__medium"
            aria-labelledby="device-modal-title"
            onclick="jumpCloseModalOnBackdrop(event)" {
                div class="modal__header" {
                    h2 id="device-modal-title" class="modal__title" { (title) }
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
                            input id="device-name" class="form-input" name="name" placeholder="e.g., Gaming PC" value=(name) required autofocus;
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
