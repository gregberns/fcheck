#!/bin/bash

VERSION=$(cat version)
echo $VERSION

TAG=fcheck:debian-stretch-$VERSION

echo "Start Build"
docker build --target debian-stretch -t $TAG . || exit $?
echo "End Build"

source_path=/bin/fcheck
destination_dir=./bin/$VERSION

mkdir -p $destination_dir

container_id=$(docker create $TAG)
docker cp $container_id:$source_path $destination_dir
docker rm $container_id
