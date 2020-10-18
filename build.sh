#!/bin/bash

VERSION=$(cat version)
echo $VERSION

TAG=fcheck:alpine-$VERSION

echo "Start Build"
docker build --target alpine -t $TAG . || exit $?
echo "End Build"

source_path=/app
destination_path=./bin/$VERSION/fcheck

mkdir -p $destination_path

container_id=$(docker create $TAG)
docker cp $container_id:$source_path $destination_path
docker rm $container_id
