# osm-admin

## Overview
osm-admin is an [Open Street Map (OSM)](http://www.openstreetmap.org) data administration tool. At the first stage it 
focuses on efficient import of *.osm.pbf files into an `apidb` schema on Postgresql database for use by the 
[openstreetmap-website](https://github.com/openstreetmap/openstreetmap-website). In the future support for 
database export and manipulation of *.osm.pbf dumps will be added.

## Status
The project is in initial stages of development.

## Usage

### Clone
Clone the git repository 

### Test
Requires docker and wget on your system. Runs docker compose based integration test. Downloads a malta-latest.osm.pbf 
from http://download.geofabrik.de/europe/malta-latest.osm.pbf
```bash
cd osm-admin
./bin/run-tests.sh
```

### Build
```bash
docker build -t osm-admin:0.1.0
touch touch pg_restore.log
touch touch pg_restore.error.log
docker run --rm --name osm-admin -it \
  -v ${PWD}/pg_restore.log:/var/log/osm/pg_restore.log \
  -v ${PWD}/pg_restore.error.log:/var/log/osm/pg_restore.error.log \
  -v ${PWD}/data/malta-latest.osm.pbf:/var/lib/osm/input/malta-latest.osm.pbf \
  osm-admin:0.1.0 import \
  --input /var/lib/osm/input/malta-latest.osm.pbf \
  --input-format pbf \
  --output /var/lib/osm/output/malta-latest \
  --host localhost \
  --port 5432 \
  --user openstreetmap \
  --password
```

### Develop
See instructions for setting up the [development](https://github.com/navigatorsguild/osm-admin/wiki/Development) environment.

## Credits
 - [osmosis](https://github.com/openstreetmap/osmosis) - a powerful OSM data processor.
   - used to bootstrap the development environment
 - [osmium](https://github.com/osmcode/libosmium) - an very fast OSM data processor.
     - used to bootstrap the development environment
 - [Open Street Map (OSM)](http://www.openstreetmap.org)
   - [apidb](https://wiki.openstreetmap.org/wiki/Databases_and_data_access_APIs#apidb) schema definition 

## Similar projects
- [osmosis](https://github.com/openstreetmap/osmosis) - a powerful OSM data processor.
- [osmium](https://github.com/osmcode/libosmium) - an very fast OSM data processor.

## License
Apache License Version 2.0. See the LICENSE file.

