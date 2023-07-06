FROM rust:1.66-bullseye AS builder

WORKDIR /home/osm-admin

RUN apt-get update
RUN apt-get install -y protobuf-compiler

RUN cargo init --vcs none --lib .

COPY ./Cargo.toml ./Cargo.toml
RUN cargo update
COPY ./src ./src
RUN cargo build -r


FROM ubuntu:22.04
RUN apt-get update
RUN apt-get install -y postgresql-client
RUN mkdir -p /var/log/osm
RUN mkdir -p /var/lib/osm
WORKDIR /opt/osm
COPY --from=builder /home/osm-admin/target/release/import-test /opt/osm-admin/bin/import-test

ENTRYPOINT ["/opt/osm-admin/bin/import-test"]

