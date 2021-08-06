FROM ubuntu:20.04

ARG DEBIAN_FRONTEND=noninteractive

# Update default packages
RUN apt-get update

# Get Ubuntu packages
RUN apt-get install -y \
	build-essential \
	curl \
	pkg-config \
	libssl-dev
RUN apt-get -y dist-upgrade



# Update new packages
RUN apt-get update

# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /home/rust/

COPY . .

# USER root
# RUN chown -R rust:rust .
# USER rust

ENV DATABASE_URL "postgresql://postgres:postgres@172.17.0.1:5432/nettuscheduler"

RUN cargo build --release
RUN cp /home/rust/target/release/nettu_scheduler .

ENTRYPOINT ["./nettu_scheduler"]
