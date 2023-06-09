FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .

# Install certificates
RUN apt-get install -y --no-install-recommends ca-certificates && update-ca-certificates

RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/api_aggregator ./api_aggregator

# runtime
FROM debian:bullseye-slim

RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

COPY --from=builder /usr/src/app/api_aggregator /app/api_aggregator
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

CMD ./api_aggregator
