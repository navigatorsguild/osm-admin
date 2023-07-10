# osm-admin

## Overview
osm-admin is an [Open Street Map (OSM)](http://www.openstreetmap.org) data administration tool. It provides 
efficient import and export of *.osm.pbf files into and from an `apidb` schema on Postgresql database for use by the 
[openstreetmap-website](https://github.com/openstreetmap/openstreetmap-website). 

## Status
The project is in the alpha stage of its first version 0.1.0

## Issues
Issues are welcome and appreciated. Please submit to https://github.com/navigatorsguild/osm-admin/issues

## Roadmap
### v0.2.0
 * Implement synch of OSM changes from https://planet.openstreetmap.org/ - [#1](https://github.com/navigatorsguild/osm-admin/issues/1)
 * merge OSM changes into apidb schema - [#2](https://github.com/navigatorsguild/osm-admin/issues/2)
 
### v0.3.0
* implement option to drop history from export - [#3](https://github.com/navigatorsguild/osm-admin/issues/3)
* index the contents of the DB by S2 Cells - [#4](https://github.com/navigatorsguild/osm-admin/issues/4)
* index the contents by geographical context - [#5](https://github.com/navigatorsguild/osm-admin/issues/5)
* implement regional extracts from OSM database - [#6](https://github.com/navigatorsguild/osm-admin/issues/6)

## Usage
This software is distributed as a docker container at https://hub.docker.com/r/navigatorsguild/osm-admin
All the software that powers the container is located in this GitHub [repository](https://github.com/navigatorsguild/osm-admin) and is available under 
MIT or Apache-2.0 licences

### Pull
```bash
$ docker pull navigatorsguild/osm-admin
```

### Prepare the data
The data can be downloaded from https://planet.openstreetmap.org/pbf/planet-latest.osm.pbf by HTTPS or by torrent.
To get started we recommend using geographical extracts prepared with osmium or downloaded from http://download.geofabrik.de/

```bash
$ osmium getid -r -t planet-latest.osm.pbf r365307 -o malta-boundary-latest.osm
$ osmium extract -p malta-boundary-latest.osm -o malta-latest.osm.pbf planet-latest.osm.pbf
```

### Import
```bash
$ touch touch pg_restore.log
$ touch touch pg_restore.error.log
$ docker volume create osm-admin-vol
$ docker run --rm --name osm-admin -it \
  -v ${PWD}/<PGPASSFILE>:/root/.pgpass \
  -v osm-admin-vol:/var/lib/osm/ \
  -v ${PWD}/pg_restore.log:/var/log/osm/pg_restore.log \
  -v ${PWD}/pg_restore.error.log:/var/log/osm/pg_restore.error.log \
  -v ${PWD}malta-latest.osm.pbf:/var/lib/osm/input/malta-latest.osm.pbf \
  navigatorsguild/osm-admin:latest \
  import \
  --verbose \
  --input /var/lib/osm/input/malta-latest.osm.pbf \
  --input-format pbf \
  --output /var/lib/osm/output/malta-latest \
  --host <OSM_HOST> \
  --port <OSM_PORT> \
  --user <OSM_USER> \
  --no-password
```

Specifying ```--pasword``` will prompt for password. There is an option to use ```--no-password``` for trust 
connections and with pgpass file. Please see an example of PGPASSFILE in ./db/pgpass and the documentation at 
https://www.postgresql.org/docs/current/libpq-pgpass.html 
Please note that the permissions for PGPASSFILE must be 0o600.

### Export
```bash
$ touch touch pg_dump.log
$ touch touch pg_dump.error.log
$ docker volume create osm-admin-vol
$ docker run --rm --name osm-admin -it \
  -v ${PWD}/<PGPASSFILE>:/root/.pgpass \
  -v osm-admin-vol:/var/lib/osm/ \
  -v ${PWD}/pg_dump.log:/var/log/osm/pg_dump.log \
  -v ${PWD}/pg_dump.error.log:/var/log/osm/pg_dump.error.log \
  -v ${PWD}/output/:/var/lib/osm/output \
  navigatorsguild/osm-admin:latest \
  export \
  --verbose \
  --dump /var/lib/osm/dump \
  --output /var/lib/osm/output/result.osm.pbf \
  --output-format pbf \
  --compression-level 0 \
  --host <OSM_HOST> \
  --port <OSM_PORT> \
  --user <OSM_USER> \
  --password"
```

## Develop
See instructions for setting up the [development](https://github.com/navigatorsguild/osm-admin/wiki/Development) environment.

## Experiment
See instructions for setting up the [experimentation](https://github.com/navigatorsguild/osm-admin/wiki/Experiment) environment.

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
MIT OR Apache-2.0

