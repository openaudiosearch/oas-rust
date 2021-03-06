# use the cargo-chef base image
FROM lukemathwalker/cargo-chef:latest-rust-1.53.0 as base
RUN apt-get update && apt-get install -y libssl-dev

# prepare the cargo-chef build
FROM base as planner
WORKDIR app
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

# build dependencies with cargo-chef
FROM base as cacher
WORKDIR app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# build the main binary
FROM base as builder
WORKDIR app
COPY . .
# copy the built dependencies from previous image
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release --bin oas

# build the main image
FROM debian:stable-slim as runtime
RUN apt-get update && apt-get install -y libssl-dev curl
WORKDIR app
COPY --from=builder /app/target/release/oas /usr/local/bin
CMD ["/usr/local/bin/oas", "server"]
