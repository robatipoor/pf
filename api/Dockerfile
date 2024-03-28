# build stage
FROM rust:latest as builder

WORKDIR /workspace

RUN apt-get update && apt-get install lld clang -y

COPY . .

RUN cargo build --bin pf-api --release

# deploy stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends openssl ca-certificates && apt-get clean

# create workspace directory
WORKDIR /workspace

COPY api/settings settings

# copy binary and configuration files
COPY --from=builder /workspace/target/release/pf-api .

# expose port
EXPOSE 8080

ENV RUST_LOG info
ENV PF__SERVER__HOST '0.0.0.0'

# run the binary
ENTRYPOINT ["./pf-api","--settings","settings/base.toml"]
