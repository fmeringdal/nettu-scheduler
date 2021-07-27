FROM ekidd/rust-musl-builder as builder

WORKDIR /home/rust/

COPY . .

USER root
RUN chown -R rust:rust .
USER rust

ENV DATABASE_URL "postgresql://postgres:postgres@172.17.0.1:5432/nettuscheduler"

# RUN cargo test
RUN cargo build --release

# Size optimization
RUN strip target/x86_64-unknown-linux-musl/release/nettu_scheduler

# Start building the final image
FROM scratch
WORKDIR /home/rust/
COPY --from=builder /home/rust/target/x86_64-unknown-linux-musl/release/nettu_scheduler .

ENTRYPOINT ["./nettu_scheduler"]
