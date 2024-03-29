FROM rust:1.68.2-slim-bullseye as builder
WORKDIR /usr/src/build
COPY . .
RUN apt-get update && apt-get install protobuf-compiler -y && cargo install --path load-generator-wrk

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y wrk && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/load-generator-wrk /usr/local/bin/load-generator-wrk
CMD [ "load-generator-wrk" ]
