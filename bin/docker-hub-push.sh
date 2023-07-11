#!/usr/bin/env bash

TAG=`git for-each-ref refs/tags --sort=-taggerdate --format='%(refname:lstrip=2)' | tail -n 1`

docker buildx build . \
  --builder=container \
  --push \
  --platform=linux/amd64,linux/arm64/v8 \
  -t navigatorsguild/osm-admin:latest \
  -t navigatorsguild/osm-admin:${TAG}
