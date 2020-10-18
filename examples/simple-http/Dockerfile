FROM debian:stretch as debian-stretch

WORKDIR /fcheck

RUN wget https://github.com/gregberns/fcheck/releases/download/0.3.0/fcheck

# Install any needed libraries
# * `wdiff` has been quite useful for checking text files
# * `netcat` ensures a service's port is open before fcheck starts tests
RUN apt-get update &&\
    apt-get install -y --no-install-recommends wdiff netcat &&\
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Config is for the `fcheck-config.toml` file
RUN mkdir /fcheck/config &&\
    # The data directory is used for `actual` and `expected` folders to diff test results
    mkdir /fcheck/data &&\
    # The output directory can be used for the output report
    mkdir /fcheck/output

# For local testing, its helpful to just mount the `fcheck-config.toml`, then the container isn't rebuilt every time
CMD ["/bin/fcheck", "--config-file", "/fcheck/config/fcheck-config.toml", "--report-file", "/output/report.json"]
