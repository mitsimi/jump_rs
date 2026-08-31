use maud::{Markup, html};

use crate::web::icons::{Icon, icon, icon_with_label};

use super::feedback::field_error;

pub fn transfer_modal(error: Option<&str>) -> Markup {
    html! {
        dialog
            class="modal modal__large"
            aria-labelledby="transfer-modal-title"
            onclick="jumpCloseModalOnBackdrop(event)" {
                (modal_header())
                div class="transfer__modal-body" {
                    (transfer_tabs())

                    div class="transfer__content" {
                        (export_panel())
                        (import_panel(error))
                    }
                    (format_hint())
                }
        }
    }
}

fn modal_header() -> Markup {
    html! {
        div class="modal__header" {
            h2 id="transfer-modal-title" class="modal__title" {
                "DATA TRANSFER"
            }
            button
                class="modal__close-btn"
                type="button"
                onclick="jumpCloseModal()"
                aria-label="Close" {
                (icon_with_label(Icon::X))
            }
        }
    }
}

fn transfer_tabs() -> Markup {
    html! {
        div class="transfer__tabs" role="tablist" aria-label="Transfer mode" {
            button
                id="transfer-export-tab"
                type="button"
                role="tab"
                class="transfer__tab transfer__tab--active"
                aria-selected="true"
                aria-controls="transfer-export-panel"
                tabindex="0"
                onclick="jumpShowTransferTab('export')"
                onkeydown="jumpHandleTransferTabKeydown(event)" {
                "EXPORT"
            }
            button
                id="transfer-import-tab"
                type="button"
                role="tab"
                class="transfer__tab"
                aria-selected="false"
                aria-controls="transfer-import-panel"
                tabindex="-1"
                onclick="jumpShowTransferTab('import')"
                onkeydown="jumpHandleTransferTabKeydown(event)" {
                "IMPORT"
            }
        }
    }
}

fn export_panel() -> Markup {
    html! {
        div
            id="transfer-export-panel"
            class="transfer__export-section"
            role="tabpanel"
            aria-labelledby="transfer-export-tab" {
            div class="transfer__export-icon" { (icon(Icon::Download)) }
            p class="transfer__description" {
                "Export all registered devices to a JSON file. The file contains device names, MAC addresses, IP addresses, ports, and descriptions."
            }
            a class="transfer__action-btn" href="/devices/export" download {
                (icon(Icon::Download))
                "DOWNLOAD JSON"
            }
        }
    }
}

fn import_panel(error: Option<&str>) -> Markup {
    html! {
        form
            id="transfer-import-panel"
            class="transfer__import-section"
            role="tabpanel"
            aria-labelledby="transfer-import-tab"
            hidden
            hx-post="/devices/import"
            hx-target="#device-grid"
            hx-swap="outerHTML"
            hx-status:4xx="target:#modal-root swap:innerHTML" {
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
                    label class="form-label" for="import-payload" { "JSON device data" }
                    textarea
                        class="transfer__json-input"
                        name="payload"
                        id="import-payload"
                        rows="6"
                        placeholder="[{\"name\":\"Device\",\"mac_address\":\"aa:bb:cc:dd:ee:ff\",\"ip_address\":\"192.168.1.100\",\"port\":9}]" {}
                    button class="transfer__action-btn" type="submit" {
                        (icon(Icon::Upload))
                        "Import devices"
                    }
                }
            }
        }
    }
}

fn format_hint() -> Markup {
    html! {
        div class="transfer__footer" {
            div class="transfer__format-hint" {
                "Expected JSON format:"
                code { "[{\"name\":\"Name\",\"mac_address\":\"aa:bb:cc:dd:ee:ff\",\"ip_address\":\"1.2.3.4\",\"port\":9,\"description\":\"...\"}]" }
            }
        }
    }
}
