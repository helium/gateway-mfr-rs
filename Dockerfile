FROM rust:1.71.0

WORKDIR /usr/src/gateway-mfr-rs

COPY . .

RUN cargo build
