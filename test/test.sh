#!/bin/bash

set -e

VERSION=$(cat ../version)

mkdir -p ./actual
mkdir -p ./bin
mkdir -p ./config
mkdir -p ./expected
mkdir -p ./output

cp ../bin/$VERSION/fcheck ./bin/fcheck

docker-compose build

rm -rf ./actual/*
rm -rf ./output/*

echo "============== Test-01 =============="
./test-01.sh && echo "Test-01 Successful." || exit 1


echo "============== Test-02 =============="
./test-02.sh && echo "Test-02 Successful." || exit 1


echo "Tests Successful. Push to Prod."
