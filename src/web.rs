pub mod error;
pub mod forms;
pub mod icons;
pub mod views;

use axum::{
    Form, Router,
    extract::{Extension, Path},
    http::{
        StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    },
    response::{IntoResponse, Response},
    routing::{get, post},
};

use crate::{
    arp::ArpError,
    error::ApiError,
    storage::SharedStorage,
    web::{
        error::{WebError, WebResult, api_result, form_result},
        forms::{ArpLookupForm, DeviceForm, ImportDevicesForm},
        views::ToastKind,
    },
};

pub fn router() -> Router {
    Router::new()
        .route("/", get(home))
        .route("/devices", get(devices_fragment).post(create_device))
        .route("/devices/new", get(new_device_modal))
        .route("/devices/{id}/edit", get(edit_device_modal))
        .route("/devices/{id}/update", post(update_device))
        .route("/devices/{id}/delete", post(delete_device))
        .route("/devices/{id}/wake", post(wake_device))
        .route("/devices/export", get(export_devices))
        .route("/devices/import", post(import_devices))
        .route("/arp-lookup", post(arp_lookup))
        .route("/transfer", get(transfer_modal))
}

async fn home(Extension(storage): Extension<SharedStorage>) -> impl IntoResponse {
    let devices = crate::devices::list_devices(&storage);
    views::home_page(&devices)
}

async fn devices_fragment(Extension(storage): Extension<SharedStorage>) -> impl IntoResponse {
    let devices = crate::devices::list_devices(&storage);
    views::device_grid(&devices)
}

async fn new_device_modal() -> impl IntoResponse {
    views::device_modal(None, None)
}

async fn edit_device_modal(
    Extension(storage): Extension<SharedStorage>,
    Path(id): Path<String>,
) -> WebResult<impl IntoResponse> {
    let device = storage
        .get(&id)
        .ok_or_else(|| WebError::Form(format!("Device not found: {id}")))?;
    Ok(views::device_modal(Some(&device), None))
}

async fn create_device(
    Extension(storage): Extension<SharedStorage>,
    Form(form): Form<DeviceForm>,
) -> Response {
    let req = match form_result(form.into_create_request()) {
        Ok(req) => req,
        Err(err) => return device_form_error(None, &err.message()),
    };

    if let Err(err) = api_result(crate::devices::create_device(&storage, req)) {
        return device_form_error(None, &err.message());
    }

    let devices = crate::devices::list_devices(&storage);
    views::grid_with_toast(&devices, ToastKind::Success, "Device created").into_response()
}

async fn update_device(
    Extension(storage): Extension<SharedStorage>,
    Path(id): Path<String>,
    Form(form): Form<DeviceForm>,
) -> Response {
    let existing = storage.get(&id);
    let req = match form_result(form.into_update_request()) {
        Ok(req) => req,
        Err(err) => return device_form_error(existing.as_ref(), &err.message()),
    };

    if let Err(err) = api_result(crate::devices::update_device(&storage, &id, req)) {
        return device_form_error(existing.as_ref(), &err.message());
    }

    let devices = crate::devices::list_devices(&storage);
    views::grid_with_toast(&devices, ToastKind::Success, "Device updated").into_response()
}

async fn delete_device(
    Extension(storage): Extension<SharedStorage>,
    Path(id): Path<String>,
) -> Response {
    if let Err(err) = api_result(crate::devices::delete_device(&storage, &id)) {
        return (
            err.status_code(),
            views::toast_fragment(ToastKind::Error, &err.message()),
        )
            .into_response();
    }

    let devices = crate::devices::list_devices(&storage);
    views::grid_with_toast(&devices, ToastKind::Success, "Device removed").into_response()
}

async fn wake_device(
    Extension(storage): Extension<SharedStorage>,
    Path(id): Path<String>,
) -> Response {
    match api_result(crate::devices::wake_device(&storage, &id)) {
        Ok(()) => views::toast_fragment(ToastKind::Success, "Wake signal sent").into_response(),
        Err(err) => (
            err.status_code(),
            views::toast_fragment(ToastKind::Error, &err.message()),
        )
            .into_response(),
    }
}

async fn arp_lookup(Form(form): Form<ArpLookupForm>) -> Response {
    let ip = form.ip_address.trim();
    let current_mac = form.mac_address.unwrap_or_default();
    if ip.is_empty() {
        return views::mac_lookup_error(&current_mac, "Enter an IP address first").into_response();
    }

    match crate::devices::arp_lookup(ip) {
        Ok(mac) => views::mac_lookup_controls(&mac).into_response(),
        Err(ApiError::Arp(err @ ArpError::NotDirectlyConnected { .. })) => {
            views::mac_lookup_error_with_hint(&current_mac, &err.to_string(), err.hint())
                .into_response()
        }
        Err(err) => {
            let err = WebError::Api(err);
            views::mac_lookup_error(&current_mac, &err.message()).into_response()
        }
    }
}

async fn transfer_modal() -> impl IntoResponse {
    views::transfer_modal(None)
}

async fn import_devices(
    Extension(storage): Extension<SharedStorage>,
    Form(form): Form<ImportDevicesForm>,
) -> Response {
    let req = match form_result(form.into_import_requests()) {
        Ok(req) => req,
        Err(err) => return transfer_error(&err.message()),
    };

    let count = req.len();
    if let Err(err) = api_result(crate::devices::import_devices(&storage, req)) {
        return transfer_error(&err.message());
    }

    let devices = crate::devices::list_devices(&storage);
    views::grid_with_toast(
        &devices,
        ToastKind::Success,
        &format!("Imported {count} device(s)"),
    )
    .into_response()
}

async fn export_devices(Extension(storage): Extension<SharedStorage>) -> Response {
    let exported = crate::devices::export_devices(&storage);
    let body = match serde_json::to_string_pretty(&exported) {
        Ok(body) => body,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                views::error_page(StatusCode::INTERNAL_SERVER_ERROR, &err.to_string()),
            )
                .into_response();
        }
    };
    let date = time::OffsetDateTime::now_utc().date();
    let filename = format!("jump-devices-{date}.json");

    (
        [
            (CONTENT_TYPE, "application/json; charset=utf-8".to_string()),
            (
                CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        body,
    )
        .into_response()
}

fn device_form_error(device: Option<&crate::models::Device>, message: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        [("HX-Retarget", "#modal-root")],
        views::device_modal(device, Some(message)),
    )
        .into_response()
}

fn transfer_error(message: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        [("HX-Retarget", "#modal-root")],
        views::transfer_modal(Some(message)),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request},
    };
    use tempfile::TempDir;
    use tower::ServiceExt;

    fn app() -> (Router, SharedStorage, TempDir) {
        let _ = crate::config::init();
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("devices.json");
        let storage = SharedStorage::load(path.to_str().unwrap()).unwrap();
        (crate::app::build_service(storage.clone()), storage, dir)
    }

    async fn response_text(response: Response) -> String {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    fn form_request(uri: &str, body: &str) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    #[tokio::test]
    async fn home_renders_shell_and_assets() {
        let (app, _storage, _dir) = app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_text(response).await;
        assert!(body.contains("id=\"device-grid\""));
        assert!(body.contains("/static/vendor/htmx.min.js"));
        assert!(body.contains("/static/vendor/alpine.min.js"));
        assert!(body.contains("href=\"/api/swagger\""));
        assert!(body.contains("API Docs"));
        assert!(body.contains("No Devices Found"));
    }

    #[tokio::test]
    async fn create_device_updates_storage_and_returns_grid() {
        let (app, storage, _dir) = app();
        let response = app
            .oneshot(form_request(
                "/devices",
                "name=Gaming%20PC&mac_address=AA%3ABB%3ACC%3ADD%3AEE%3AFF&ip_address=192.168.1.10&port=9&description=Main",
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(storage.get_all().len(), 1);
        let body = response_text(response).await;
        assert!(body.contains("Gaming PC"));
        assert!(body.contains("Device created"));
        assert!(body.contains("hx-swap-oob"));
    }

    #[tokio::test]
    async fn create_device_validation_error_keeps_storage_empty() {
        let (app, storage, _dir) = app();
        let response = app
            .oneshot(form_request(
                "/devices",
                "name=Gaming%20PC&mac_address=bad&port=9",
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(storage.get_all().is_empty());
        let body = response_text(response).await;
        assert!(body.contains("Invalid MAC address format"));
    }

    #[tokio::test]
    async fn import_valid_json_updates_grid() {
        let (app, storage, _dir) = app();
        let payload = "%5B%7B%22name%22%3A%22Imported%22%2C%22mac_address%22%3A%22AA%3ABB%3ACC%3ADD%3AEE%3AFF%22%2C%22ip_address%22%3A%22192.168.1.20%22%2C%22port%22%3A9%2C%22description%22%3A%22From%20JSON%22%7D%5D";
        let response = app
            .oneshot(form_request(
                "/devices/import",
                &format!("payload={payload}"),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(storage.get_all().len(), 1);
        let body = response_text(response).await;
        assert!(body.contains("Imported"));
        assert!(body.contains("Imported 1 device(s)"));
    }

    #[tokio::test]
    async fn export_devices_returns_download_json() {
        let (app, _storage, _dir) = app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/devices/export")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/json; charset=utf-8"
        );
        assert!(
            response
                .headers()
                .get(CONTENT_DISPOSITION)
                .unwrap()
                .to_str()
                .unwrap()
                .contains("attachment; filename=\"jump-devices-")
        );
    }

    #[tokio::test]
    async fn preserved_api_and_swagger_routes_resolve() {
        let (app, _storage, _dir) = app();

        let api_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/devices")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(api_response.status(), StatusCode::OK);

        let docs_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/docs/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(docs_response.status(), StatusCode::OK);

        let swagger_response = app
            .oneshot(
                Request::builder()
                    .uri("/api/swagger")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            swagger_response.status().is_redirection() || swagger_response.status().is_success()
        );
    }
}
