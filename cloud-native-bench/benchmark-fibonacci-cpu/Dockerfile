FROM rust:1.68.2-slim-bullseye
WORKDIR /usr/src/build
COPY . .
RUN apt-get update && apt-get install protobuf-compiler -y && cargo install cargo-criterion && cargo install --path benchmark-fibonacci-cpu && rm -rf target
WORKDIR /usr/src/build/benchmark-fibonacci-cpu
CMD ["benchmark-fibonacci-cpu"]
