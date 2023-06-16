FROM clux/muslrust:stable AS builder
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release --package benchmark-web-backend

FROM gcr.io/distroless/static:nonroot
COPY --from=builder --chown=nonroot:nonroot /volume/target/x86_64-unknown-linux-musl/release/benchmark-web-backend /app/benchmark-web-backend
CMD [ "/app/benchmark-web-backend" ]
