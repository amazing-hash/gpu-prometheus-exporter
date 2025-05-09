FROM rust:latest as build

WORKDIR /app

COPY . .

RUN cargo build --release

FROM ubuntu:latest as production
COPY --from=build /app/target/release/gpu-prometheus-exporter /usr/local/bin/gpu-prometheus-exporter

ENTRYPOINT ["gpu-prometheus-exporter"]

EXPOSE 9835