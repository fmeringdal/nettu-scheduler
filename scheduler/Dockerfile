FROM ubuntu:20.04

ARG DEBIAN_FRONTEND=noninteractive

# Update default packages
RUN apt-get update \
    && apt-get install -y build-essential curl pkg-config libssl-dev \
    && apt-get -y dist-upgrade \
    && rm -rf /var/lib/apt/lists/*

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /home/rust/

COPY . .

ENV DATABASE_URL "postgresql://postgres:postgres@172.17.0.1:5432/nettuscheduler"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && cargo build --release \
    && cp /home/rust/target/release/nettu_scheduler . \
    && cp /home/rust/target/release/migrate . \
    && rm -rf /home/rust/target \
    && rustup self uninstall -y

ENTRYPOINT ["/bin/sh", "-c" , "./migrate && ./nettu_scheduler"]
