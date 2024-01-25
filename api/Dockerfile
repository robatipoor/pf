# build stage
FROM rust:latest as builder

WORKDIR /workspace

RUN apt-get update && apt-get install lld clang -y

COPY . .

RUN cargo build --bin api --release

# deploy stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends openssl ca-certificates && apt-get clean

# create workspace directory
WORKDIR /workspace

COPY api/settings settings

# copy binary and configuration files
COPY --from=builder /workspace/target/release/api .

# expose port
EXPOSE 8080

ENV RUST_LOG info

# run the binary
ENTRYPOINT ["./api","--settings","settings/base.toml"]
