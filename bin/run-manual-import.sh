#!/usr/bin/env bash

# cleanup
docker stop osm-test-db
docker rm -f osm-test-db
docker network rm osm-test-net
docker volume rm -f osm-test-db-vol
docker volume rm -f osm-test-vol
docker image rm osm-test-db

# build
docker build db -t osm-test-db
docker volume create osm-test-db-vol
docker volume create osm-test-vol
docker network create osm-test-net

# run openstreetmap database
docker run \
  --network osm-test-net \
  --name osm-test-db \
  -v osm-test-db-vol:/var/lib/postgresql/data \
  -e POSTGRES_PASSWORD=openstreetmap \
  -e POSTGRES_HOST_AUTH_METHOD=trust \
  -p 5432:5432 \
  -d osm-test-db

until docker exec -it osm-test-db pg_isready -U postgress
do
  echo "Waiting for osm-test-db"
  sleep 1
done


# copy one of the files below for test import
cp ./tests/fixtures/niue-230109.osm.pbf tests/fixtures/test.osm.pbf
#cp ./tests/fixtures/history-niue-230109.osm.pbf tests/fixtures/test.osm.pbf
#cp ./tests/fixtures/malta-230109.osm.pbf tests/fixtures/test.osm.pbf
#cp ./tests/fixtures/history-malta-230109.osm.pbf tests/fixtures/test.osm.pbf
#cp ./tests/fixtures/denmark-230109.osm.pbf tests/fixtures/test.osm.pbf
#cp ./tests/fixtures/history-denmark-230109.osm.pbf tests/fixtures/test.osm.pbf
#cp ./tests/fixtures/germany-230109.osm.pbf tests/fixtures/test.osm.pbf
#cp ./tests/fixtures/history-germany-230109.osm.pbf tests/fixtures/test.osm.pbf

# run the import
echo "" >  ./pg_restore.log
echo "" >  ./pg_restore.error.log
chmod go-rwx ./db/pgpass

docker run --rm --name osm-admin -it \
  --network osm-test-net \
  -v ${PWD}/db/pgpass:/root/.pgpass \
  -v osm-test-vol:/var/lib/osm/ \
  -v ${PWD}/pg_restore.log:/var/log/osm/pg_restore.log \
  -v ${PWD}/pg_restore.error.log:/var/log/osm/pg_restore.error.log \
  -v ${PWD}/tests/fixtures/test.osm.pbf:/var/lib/osm/input/test.osm.pbf \
  navigatorsguild/osm-admin \
  --verbose \
  import \
  --input /var/lib/osm/input/test.osm.pbf \
  --input-format pbf \
  --output /var/lib/osm/output/test-dump \
  --host osm-test-db \
  --port 5432 \
  --database openstreetmap \
  --user openstreetmap \
  --no-password
