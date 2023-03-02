FROM rust:1.67-slim AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

#------------

FROM gcr.io/distroless/cc

COPY --from=builder /app/target/release/meta-cross /meta-cross

ENTRYPOINT ["/meta-cross"]