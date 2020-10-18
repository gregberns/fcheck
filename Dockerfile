FROM clux/muslrust as build

WORKDIR /build
COPY Cargo.toml Cargo.lock /build/
RUN mkdir src &&\
    echo "pub fn main() {}" > src/main.rs &&\
    cargo build --release &&\
    rm -rf /build/src &&\
    rm -rf /build/target/x86_64-unknown-linux-musl/release/.fingerprint/fcheck-*

COPY src/ /build/src/

RUN cargo build --release

# ==================================================

FROM debian:stretch as debian-stretch

WORKDIR /fcheck

RUN apt-get update &&\
    apt-get install -y --no-install-recommends wdiff netcat &&\
    apt-get clean && rm -rf /var/lib/apt/lists/*

COPY ./dhall/dhall-to-json /usr/local/bin
COPY ./dhall/dhall-to-yaml /usr/local/bin

RUN mkdir /fcheck/config &&\
    mkdir /fcheck/data &&\
    mkdir /fcheck/output

COPY --from=build /build/target/x86_64-unknown-linux-musl/release/fcheck /app

CMD ["./fcheck"]

# ==================================================

FROM alpine as alpine

WORKDIR /fcheck

# RUN apk update &&\
# diffutils - contains wdiff
# netcat-openbsd - contains netcat
RUN apk add --no-cache diffutils netcat-openbsd

COPY ./dhall/dhall-to-json /usr/local/bin
COPY ./dhall/dhall-to-yaml /usr/local/bin

RUN mkdir /fcheck/config &&\
    mkdir /fcheck/data &&\
    mkdir /fcheck/output

COPY --from=build /build/target/x86_64-unknown-linux-musl/release/fcheck /app

CMD ["./fcheck"]
