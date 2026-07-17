# Jumpers

A simple Wake-on-LAN (WoL) web server built with Rust and Axum.

## Features

- Wake devices on your network via HTTP API
- Rust-rendered web interface powered by HTMX
- JSON-based device storage
- Configurable via file or environment variables
- Configurable structured request logging
- Optional built-in username/password authentication
- Docker support included

## Quick Start

1. Copy the example configuration:

```bash
cp config.toml.example config.toml
```

2. Run the server:

```bash
cargo run
```

3. Access the web interface at `http://localhost:3000`

The web UI is served directly by the Rust binary. No separate Node, Vite, or
frontend build step is required.

## Configuration

Configuration can be provided via `config.toml` or environment variables with the `JUMPERS_` prefix.

Example environment variables:

```bash
JUMPERS_SERVER_PORT=8080
JUMPERS_SERVER_LOG_LEVEL=debug
JUMPERS_STORAGE_FILE_PATH=/data/devices.json
```

### Authentication

Built-in authentication is opt-in. Define one or more users as a comma-separated
list of `username:password_hash` entries using bcrypt hashes, matching TinyAuth's
user format:

```toml
[auth]
enabled = true
users = "admin:$2b$12$..."
secure_cookie = true
```

Or configure the same values through the environment:

```bash
JUMPERS_AUTH_ENABLED=true
JUMPERS_AUTH_USERS='admin:$2b$12$...'
JUMPERS_AUTH_SECURE_COOKIE=true
```

Generate a compatible credential with Jumpers:

```bash
jumpers user create --interactive
```

The command prints a `username:password_hash` value suitable for `auth.users`.
Use `--docker` to escape dollar signs for Docker Compose. You can also provide
`--password` for automation, but this exposes the password in shell history.
When Docker Compose interpolates the value, escape each `$` in the hash as `$$`.
Set `secure_cookie = true` whenever users access Jumpers over HTTPS.

Automation can authenticate to API routes with HTTP Basic authentication using
the same configured user credentials:

```bash
curl --user admin:your-password http://localhost:3000/api/devices
```

Basic authentication is accepted only for `/api/*`; it does not bypass the web
login. Always use HTTPS when sending credentials over a network because Basic
authentication encodes credentials but does not encrypt them.

If authentication is disabled (the default), Jumpers does not inspect or require
forward-auth headers. A reverse proxy can therefore protect the whole app using
TinyAuth, Authelia, Authentik, or another forward-auth provider without any
additional Jumpers configuration. Do not expose an unprotected route around the
proxy when relying on forward auth.

See `config.toml.example` for all available options.

## Docker

Run with Docker Compose:

```bash
docker compose up
```

### Networking Requirements

The Docker setup requires special networking configuration for Wake-on-LAN to function properly:

- **Host Network Mode**: The container uses `network_mode: host` to access the local network directly. This is necessary because Wake-on-LAN magic packets must be sent to the broadcast address of your local network.

- **Network Capabilities**: The container needs `NET_RAW` capability to:
  - Send raw network packets (Wake-on-LAN magic packets)
  - Perform network operations like ARP-based MAC address lookups

- **MAC Lookup Limitation**: ARP-based MAC lookup requires layer-2 access to the same LAN as the target device. This works when the container runs with host networking on a Linux Docker host. On Docker Desktop, OrbStack, and other macOS/Windows VM-backed Docker runtimes, the container may still be isolated behind a VM network even with `network_mode: host`; in that case Wake-on-LAN can still work, but MAC lookup may not see devices on the host LAN. Run the binary directly on the host for MAC lookup in that environment.

These settings are already configured in the provided `docker-compose.yml` file.

## Development

### Development tools

This project uses [cargo-run-bin](https://github.com/dustinblackman/cargo-run-bin)
to provide reproducible, project-scoped development tools. After installing Rust
on a new machine, install cargo-run-bin once and then install this project's
tools:

```bash
cargo install cargo-run-bin --locked
cargo bin --install
```

The configured tools can then be run with Cargo aliases:

```bash
cargo nextest run
cargo bacon
cargo release --help
```

Tool versions are pinned in `Cargo.toml` and cached locally in `.bin/`. Running
either alias also installs its tool automatically if it is not already cached.

Build the project:

```bash
cargo build
```

Generate OpenAPI specification:

```bash
cargo emit-openapi
```

or

```bash
cargo gen-openapi
```

The CLI also accepts an explicit output path:

```bash
jumpers openapi generate path/to/openapi.json
```

Swagger UI remains available at `/api/swagger` when the server is running.

Update vendored HTMX and Alpine bundles:

```bash
scripts/update-vendor-js.py
```

The updater downloads the browser bundles from the published npm tarballs and
updates `static/vendor/manifest.json` with version, source, and checksum data.
Specific versions can be pinned when needed:

```bash
scripts/update-vendor-js.py --htmx 2.0.10 --alpine 3.15.12
```

Use check mode to fail when vendored assets are behind the requested versions:

```bash
scripts/update-vendor-js.py --check
```
