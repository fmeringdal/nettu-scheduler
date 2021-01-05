FROM rustlang/rust:nightly

RUN cargo install cargo-watch

RUN apt-get update && apt-get -y dist-upgrade
RUN apt-get -y install build-essential libssl-dev libffi-dev
RUN apt install -y netcat


WORKDIR /var/application

EXPOSE 5000