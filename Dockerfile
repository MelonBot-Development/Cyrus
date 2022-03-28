FROM ekidd/rust-musl-builder:1.51.0 AS chef

USER root
RUN cargo install cargo-chef

WORKDIR /Cyrus

FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /Cyrus/recipe.json recipe.json

RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin Cyrus

FROM alpine AS runtime

WORKDIR /Cyrus

COPY voting.toml .
COPY --from=builder /Cyrus/target/x86_64-unknown-linux-musl/release/Cyrus .

ENTRYPOINT [ "Cyrus" ]