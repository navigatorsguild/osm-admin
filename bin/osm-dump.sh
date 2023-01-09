#!/usr/bin/env bash

mkdir -p ./result
rm -rf ./result/*
docker run --network host -v ${PWD}/db/pgpass:/root/.pgpass -v ${PWD}/result:/var/lib/pg_dump/result -it --rm postgres:15-bullseye pg_dump --host localhost --port 5432 --username openstreetmap --no-password --file /var/lib/pg_dump/result --format d -d openstreetmap --compress 0 --table public.nodes --table public.node_tags --table public.ways --table public.way_nodes --table public.way_tags --table public.relations --table public.relation_members --table public.relation_tags --table public.changesets --table public.changeset_tags --table public.users