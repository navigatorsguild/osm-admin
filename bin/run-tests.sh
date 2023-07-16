#!/usr/bin/env bash

cp ./tests/fixtures/niue-230612.osm.pbf tests/fixtures/test.osm.pbf
#cp ./tests/fixtures/history-niue-230612.osm.pbf tests/fixtures/test.osm.pbf
#cp ./tests/fixtures/malta-230612.osm.pbf tests/fixtures/test.osm.pbf
#cp ./tests/fixtures/history-malta-230612.osm.pbf tests/fixtures/test.osm.pbf
#cp ./tests/fixtures/denmark-230612.osm.pbf tests/fixtures/test.osm.pbf
#cp ./tests/fixtures/history-denmark-230612.osm.pbf tests/fixtures/test.osm.pbf

touch ./pg_restore.log
touch ./pg_restore.error.log
touch ./pg_dump.log
touch ./pg_dump.error.log
chmod go-rwx ./db/pgpass

docker compose -f ./docker-compose.yaml down --remove-orphans -v --rmi local
docker compose -f ./docker-compose.yaml build
#docker compose -f ./docker-compose.yaml run --service-ports  osm-admin-import
#docker compose -f ./docker-compose.yaml run --service-ports  import-test
#docker compose -f ./docker-compose.yaml run --service-ports  osm-admin-export
docker compose -f ./docker-compose.yaml run --service-ports  export-test
