use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{self, LogFormat};

pub fn init_tracing() {
    let config = config::get();
    let server_config = &config.server;

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| server_config.log_level.as_filter().into());

    let fmt_layer = match server_config.log_format {
        LogFormat::Json => fmt::layer().json().boxed(),
        LogFormat::Pretty => fmt::layer().pretty().boxed(),
        LogFormat::Compact => fmt::layer().compact().boxed(),
    };

    #[cfg(feature = "otlp")]
    {
        use opentelemetry::KeyValue;
        use opentelemetry::trace::TracerProvider;
        use opentelemetry_otlp::WithExportConfig;
        use opentelemetry_sdk::Resource;

        if let Some(endpoint) = &config.otel.endpoint {
            let service_name = &config.otel.service_name;

            let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .build()
                .expect("Failed to create OTLP exporter");

            let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
                .with_batch_exporter(otlp_exporter, opentelemetry_sdk::runtime::Tokio)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name.clone()),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ]))
                .build();

            let tracer = tracer_provider.tracer(service_name.clone());

            opentelemetry::global::set_tracer_provider(tracer_provider);

            let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .with(otel_layer)
                .init();

            return;
        }
    }

    // Default: logging only (no OTLP export)
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
