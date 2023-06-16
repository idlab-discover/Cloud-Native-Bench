FROM rust:1.68.2-slim-bullseye as builder
WORKDIR /usr/src/build
COPY . .
RUN apt-get update && apt-get install protobuf-compiler -y && cargo install --path benchmark-cpu

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y build-essential pkg-config libelf-dev libssl-dev bc bison flex time procps gcc curl wget xz-utils git make && git clone https://gitlab.com/knurd42/kcbench.git && cd kcbench && make install && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/benchmark-cpu /usr/local/bin/benchmark-cpu
CMD [ "benchmark-cpu" ]
