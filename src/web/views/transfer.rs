use maud::{Markup, html};

use crate::web::icons::{Icon, icon, icon_with_label};

use super::feedback::field_error;

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
