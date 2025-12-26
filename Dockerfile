FROM rust:1.91-alpine3.22 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock .
RUN mkdir ./src && touch ./src/lib.rs && cargo build --release && rm ./src/lib.rs
COPY . .
RUN cargo build --release

FROM alpine:3.22
COPY --from=builder /app/target/release/fshare /usr/local/bin/fshare
USER nobody:nogroup
CMD ["fshare"]
