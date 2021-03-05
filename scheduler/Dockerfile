FROM rust:1.49 as builder

# RUN USER=root cargo new --bin nettu_scheduler
WORKDIR ./nettu_scheduler
COPY ./ ./
RUN cargo build --release


FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8000

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /nettu_scheduler/target/release/server ${APP}/nettu_scheduler

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./nettu_scheduler"]
