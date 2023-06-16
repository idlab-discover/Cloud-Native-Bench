FROM clux/muslrust:stable AS builder
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release --package benchmark-operator

FROM alpine
RUN apk --update add helm
COPY --from=builder --chown=nonroot:nonroot /volume/target/x86_64-unknown-linux-musl/release/benchmark-operator /app/benchmark-operator
CMD [ "/app/benchmark-operator" ]
