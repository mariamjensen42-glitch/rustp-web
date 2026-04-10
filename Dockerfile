FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /app/target/release/blog-api .
COPY .env .
COPY blog.db .
COPY migrations/ ./migrations/

EXPOSE 8080
CMD ["./blog-api"]
