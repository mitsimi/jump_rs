FROM lukemathwalker/cargo-chef:0.1.77-rust-1-alpine3.23@sha256:a4900bc89ce6a34cb1183f25e73858f6b53d004e2bfaaaae54d706468552ad7e AS chef

WORKDIR /app

# ========================================

FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ========================================

FROM chef AS rust-builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo build --release --locked --bin jump_rs

# ========================================

FROM alpine:3.23 AS runtime

RUN apk add --no-cache iputils-arping iputils-ping iproute2-minimal net-tools

RUN addgroup -g 1000 app && adduser -u 1000 -G app -s /bin/sh -D app

WORKDIR /app

COPY --from=rust-builder /app/target/release/jump_rs /app/
COPY static/ /app/static/

USER app

EXPOSE 3000

ENTRYPOINT ["./jump_rs"]
