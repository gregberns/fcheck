#!/bin/bash

docker-compose build

rm -rf ./actual/*

docker-compose up --exit-code-from fcheck || (echo "docker-compose failed" && exit 1)

test -f "./actual/dogs.txt" || (echo "./actual/dogs.txt is missing" && exit 1)
if [ $(cat ./expected/dogs.txt) != "dogs" ]; then
    echo "dogs.txt are not Equal."
fi

test -f "./actual/cats.txt" || (echo "./actual/cats.txt is missing" && exit 1)
if [ $(cat ./expected/cats.txt) != "cats" ]; then
    echo "cats.txt are not Equal."
fi

echo "Tests Successful. Push to Prod."
