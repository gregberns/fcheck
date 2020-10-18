#!/bin/bash

set -e

VERSION=$(cat ../version)

mkdir -p ./bin
cp ../bin/$VERSION/fcheck ./bin/fcheck

docker-compose build

rm -rf ./actual/*
rm -rf ./output/*

docker-compose up --exit-code-from fcheck || (echo "docker-compose failed" && exit 1)

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

if [[ ! -f "./output/report.json" ]]; then
    echo "./output/report.json is missing"
    exit 1
fi
# Check that the output report is correct - with jq
OUT_REQ=$(cat output/report.json | jq -r .result)
if [[ $OUT_REQ != "success" ]]; then
    echo "./output/report.json .result should be success"
    exit 1
fi

echo "Tests Successful. Push to Prod."
