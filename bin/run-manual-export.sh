#!/usr/bin/env bash

# cleanup
docker volume rm -f osm-test-vol

# build
docker volume create osm-test-vol

# run the export
echo "" >  ./pg_dump.log
echo "" >  ./pg_dump.error.log
chmod go-rwx ./db/pgpass

docker run --rm --name osm-admin -it \
  --network osm-test-net \
  -v ${PWD}/db/pgpass:/root/.pgpass \
  -v osm-test-vol:/var/lib/osm/ \
  -v ${PWD}/pg_dump.log:/var/log/osm/pg_dump.log \
  -v ${PWD}/pg_dump.error.log:/var/log/osm/pg_dump.error.log \
  -v ${PWD}/output/:/var/lib/osm/output \
  navigatorsguild/osm-admin \
  --verbose \
  export \
  --dump /var/lib/osm/dump \
  --output /var/lib/osm/output/result.osm.pbf \
  --output-format pbf \
  --compression-level 0 \
  --host osm-test-db \
  --port 5432 \
  --database openstreetmap \
  --user openstreetmap \
  --no-password

