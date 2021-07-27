FROM ubuntu:20.04

# Update default packages
RUN apt-get update

# Get Ubuntu packages
RUN apt-get install -y \
	build-essential \
	curl \
	pkg-config \
	libssl-dev
RUN apt-get dist-upgrade



# Update new packages
RUN apt-get update

# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

# RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
# Add .cargo/bin to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /home/rust/

COPY . .

# USER root
# RUN chown -R rust:rust .
# USER rust

ENV DATABASE_URL "postgresql://postgres:postgres@172.17.0.1:5432/nettuscheduler"

# RUN cargo test
RUN cargo build 
# RUN cargo build --release
RUN echo $(ls -1 ./target)
RUN exit 1
