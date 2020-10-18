#!/bin/bash

VERSION=$(cat version)
echo $VERSION

TAG=fcheck:alpine-$VERSION

echo "Start Build"
docker build --target alpine -t $TAG . || exit $?
echo "End Build"

source_path=/bin/fcheck
destination_dir=./bin/$VERSION

mkdir -p $destination_dir

container_id=$(docker create $TAG)
docker cp $container_id:$source_path $destination_dir
docker rm $container_id
