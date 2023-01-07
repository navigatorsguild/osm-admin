#!/usr/bin/env bash

if [ ! -d ./data ]; then
  mkdir ./data
  wget --directory-prefix data http://download.geofabrik.de/europe/malta-latest.osm.pbf
fi

touch ./pg_restore.log
touch ./pg_restore.error.log
chmod go-rwx ./db/pgpass

docker compose -f ./docker-compose.yaml down --remove-orphans -v
docker compose -f ./docker-compose.yaml build
docker compose -f ./docker-compose.yaml run --service-ports  import-test
