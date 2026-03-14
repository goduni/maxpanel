FROM rust:1.94-alpine AS builder

RUN apk add --no-cache musl-dev curl

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx

# Cache dependencies with dummy sources
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs && \
    SQLX_OFFLINE=true cargo build --release && \
    rm -rf src

COPY src ./src
COPY migrations ./migrations

RUN touch src/main.rs src/lib.rs && SQLX_OFFLINE=true cargo build --release

FROM alpine:3.22

RUN apk add --no-cache ca-certificates wget && \
    addgroup -S app && adduser -S app -G app

COPY --from=builder /app/target/release/max-dashboard-backend /usr/local/bin/app
COPY --from=builder /app/migrations /app/migrations

USER app
EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=5s --retries=3 \
    CMD wget -qO- http://localhost:8080/health || exit 1

CMD ["app"]
