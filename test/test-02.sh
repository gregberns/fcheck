#!/bin/bash

# set -e

# VERSION=$(cat ../version)

# mkdir -p ./bin
# cp ../bin/$VERSION/fcheck ./bin/fcheck

# docker-compose build

# rm -rf ./actual/*
# rm -rf ./output/*

rm -rf ./config/*
cp ./test-02-config.toml ./config/config.toml

docker-compose up --exit-code-from fcheck || echo "docker-compose failed as expected"

cp ./output/report.json ./output/report-02.json

if [[ ! -f "./output/report-02.json" ]]; then
    echo "./output/report-02.json is missing"
    exit 1
fi
# Check that the output report is correct - with jq
OUT_REQ=$(cat output/report-02.json | jq -r .result)
if [[ $OUT_REQ != "failure" ]]; then
    echo "./output/report-02.json .result should be success"
    exit 1
fi

OUT_REQ=$(cat ./output/report-02.json | jq -r '.setup | .[0].result')
if [[ $OUT_REQ != "failure" ]]; then
    echo "./output/report-02.json .result[0].result should be failure"
    exit 1
fi

