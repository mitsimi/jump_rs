# Jumpers

A simple Wake-on-LAN (WoL) web server built with Rust and Axum.

## Features

- Wake devices on your network via HTTP API
- Web-based frontend interface
- JSON-based device storage
- Configurable via file or environment variables
- Optional OpenTelemetry tracing support
- Docker support included

## Quick Start

1. Run the server:

```bash
cargo run
```

2. Access the web interface at `http://localhost:3000`

Optional: copy `config.toml.example` to `config.toml` to customize settings.

## Configuration

Configuration can be provided via `config.toml` or environment variables with the `JUMPERS_` prefix.
Use `JUMPERS_CONFIG` to point at a custom config file path.

| Key                           | Env var                               | Description                                                                                                     | Default        | Example                  |
| ----------------------------- | ------------------------------------- | --------------------------------------------------------------------------------------------------------------- | -------------- | ------------------------ |
| `server.port`                 | `JUMPERS_SERVER_PORT`                 | HTTP port to listen on.                                                                                         | `3000`         | `8080`                   |
| `server.log_level`            | `JUMPERS_SERVER_LOG_LEVEL`            | Log level: `trace`, `debug`, `info`, `warn`, `error`.                                                           | `info`         | `debug`                  |
| `server.log_format`           | `JUMPERS_SERVER_LOG_FORMAT`           | Log output format: `compact`, `json`, `pretty`.                                                                 | `compact`      | `json`                   |
| `auth.allow_insecure_cookies` | `JUMPERS_AUTH_ALLOW_INSECURE_COOKIES` | Allow cookies over HTTP (useful for local dev).                                                                 | `false`        | `true`                   |
| `auth.allow_origins`          | `JUMPERS_AUTH_ALLOW_ORIGINS`          | Explicit origins allowed for auth actions. Supports `*` and single-wildcard patterns like `http://localhost:*`. | `[]`           | `["http://localhost:*"]` |
| `storage.file_path`           | `JUMPERS_STORAGE_FILE_PATH`           | Path to the JSON file storing device data.                                                                      | `devices.json` | `/data/devices.json`     |
| `wol.default_port`            | `JUMPERS_WOL_DEFAULT_PORT`            | Default UDP port for Wake-on-LAN magic packets.                                                                 | `9`            | `7`                      |
| `auth.disabled`               | `JUMPERS_AUTH_DISABLED`               | Disable authentication entirely (recommended until users are configured).                                       | `true`         | `false`                  |
| `auth.users`                  | `JUMPERS_AUTH_USERS`                  | Comma-separated `username:bcrypt_hash` entries. Docker Compose requires `$$` escaping.                            | `""`           | `admin:$2b$12$...`       |
| `auth.session_timeout`        | `JUMPERS_AUTH_SESSION_TIMEOUT`        | Session timeout in seconds.                                                                                     | `86400`        | `3600`                   |
| `otel.endpoint`               | `JUMPERS_OTEL_ENDPOINT`               | OTLP endpoint URL (requires `otlp` feature).                                                                    | _(unset)_      | `http://localhost:4317`  |
| `otel.service_name`           | `JUMPERS_OTEL_SERVICE_NAME`           | Service name for traces.                                                                                        | `jump_rs`      | `my-jump-instance`       |

See `config.toml.example` for a ready-to-edit starting point.

## Docker

Run with Docker Compose:

```bash
docker compose up
```

### Networking Requirements

The Docker setup requires special networking configuration for Wake-on-LAN to function properly:

- **Host Network Mode**: The container uses `network_mode: host` to access the local network directly. This is necessary because Wake-on-LAN magic packets must be sent to the broadcast address of your local network.

- **Network Capabilities**: The container needs `NET_RAW` and `NET_ADMIN` capabilities to:
  - Send raw network packets (Wake-on-LAN magic packets)
  - Perform network operations like MAC address lookups

These settings are already configured in the provided `docker-compose.yml` file.

## Development

Build the project:

```bash
cargo build
```

Run with OpenTelemetry tracing:

```bash
cargo run --features otlp
```

Generate OpenAPI specification:

```bash
cargo run -- --gen-openapi
```

or

```bash
cargo run gen-openapi
```

Generate client for frontend based on OpenAPI specification:

```bash
cd frontend && pnpm gen:openapi
```
