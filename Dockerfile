FROM rust:slim-buster as builder

RUN apt update && apt install -y pkg-config libssl-dev

WORKDIR /app
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN cargo build --release

# ---

FROM debian:buster-slim

RUN apt-get update && apt-get install -y libssl-dev

COPY --from=builder /app/target/release/server-monitor ./

CMD ["./server-monitor"]