FROM rust:1-alpine AS rust-builder

RUN apk add --no-cache curl

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && touch src/lib.rs src/main.rs && cargo fetch

COPY src/ ./src/

RUN cargo build --release

FROM alpine:3.23 AS runtime

RUN apk add --no-cache iputils-arping iputils-ping iproute2-minimal net-tools

RUN addgroup -g 1000 app && adduser -u 1000 -G app -s /bin/sh -D app

WORKDIR /app

COPY --from=rust-builder /app/target/release/jump_rs /app/
COPY static/ /app/static/

USER app

EXPOSE 3000

ENTRYPOINT ["./jump_rs"]
