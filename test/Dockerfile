# To support Alpine, need to either add bash or fcheck needs to add support to switch shells
# FROM alpine as alpine

FROM debian:stretch

WORKDIR /app

RUN apt-get update &&\
    apt-get install -y --no-install-recommends wdiff netcat &&\
    apt-get clean && rm -rf /var/lib/apt/lists/*

COPY ./bin/fcheck /bin
COPY ./fcheck-config.toml /bin/fcheck-config.toml

CMD ["/bin/fcheck", "--config-file", "/bin/fcheck-config.toml", "--report-file", "/output/report.json"]
