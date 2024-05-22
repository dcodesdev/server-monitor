FROM rust:slim-buster as builder

RUN apt update && apt install -y pkg-config libssl-dev
RUN cargo install sqlx-cli --no-default-features --features sqlite

WORKDIR /app

COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./migrations ./migrations
COPY ./.sqlx/ ./.sqlx 

ENV DATABASE_URL=sqlite://db/db.sqlite

RUN mkdir -p db/
RUN sqlx db create
RUN sqlx migrate run
RUN cargo build --release

# ---

FROM debian:buster-slim

RUN apt-get update && apt-get install -y libssl-dev ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/server-monitor ./
COPY ./migrations ./migrations

CMD ["./server-monitor"]