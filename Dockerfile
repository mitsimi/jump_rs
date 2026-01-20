# Stage 1: Build frontend
FROM node:24-alpine AS frontend-builder

WORKDIR /app

COPY frontend/package.json frontend/pnpm-lock.yaml* ./
RUN corepack enable pnpm && pnpm install --frozen-lockfile

COPY frontend/ .
RUN pnpm build

# Stage 2: Build Rust application
FROM rust:1-alpine AS rust-builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && touch src/lib.rs src/main.rs && cargo fetch

COPY src/ ./src/

RUN cargo build --release

# Stage 3: Final runtime image
FROM alpine:3.23 AS runtime

RUN apk add --no-cache libstdc++ openssl ca-certificates iputils net-tools

RUN addgroup -g 1000 app && adduser -u 1000 -G app -s /bin/sh -D app

WORKDIR /app

COPY --from=frontend-builder /static/dist /app/static/dist

COPY --from=rust-builder /app/target/release/jump_rs /app/

USER app

EXPOSE 3000

ENTRYPOINT ["./jump_rs"]
