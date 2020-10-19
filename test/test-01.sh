#!/bin/bash

set -e

# VERSION=$(cat ../version)

# mkdir -p ./bin
# cp ../bin/$VERSION/fcheck ./bin/fcheck

# docker-compose build

# rm -rf ./actual/*
# rm -rf ./output/*

rm -rf ./config/*
cp ./test-01-config.toml ./config/config.toml

docker-compose up --exit-code-from fcheck || (echo "docker-compose failed" && exit 1)

cp ./output/report.json ./output/report-01.json

if [[ ! -f "./actual/dogs.txt" ]]; then
    echo "./actual/dogs.txt is missing"
    exit 1
fi
if [ $(cat ./expected/dogs.txt) != "dogs" ]; then
    echo "dogs.txt are not Equal."
    exit 1
fi

if [[ ! -f "./actual/cats.txt" ]]; then
    echo "./actual/cats.txt is missing"
    exit 1
fi
if [ $(cat ./expected/cats.txt) != "cats" ]; then
    echo "cats.txt are not Equal."
    exit 1
fi

if [[ ! -f "./output/report-01.json" ]]; then
    echo "./output/report-01.json is missing"
    exit 1
fi
# Check that the output report is correct - with jq
OUT_REQ=$(cat output/report-01.json | jq -r .result)
if [[ $OUT_REQ != "success" ]]; then
    echo "./output/report-01.json .result should be success"
    exit 1
fi
