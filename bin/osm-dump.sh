#!/usr/bin/env bash

mkdir -p ./result
rm -rf ./result/*
# begin transaction isolation level repeatable read;
# select  pg_export_snapshot() as snapshot_name, cast(pg_current_xact_id () as text) as transaction_id, cast(current_timestamp as text) as timestamp;
docker run --network host -v ${PWD}/db/pgpass:/root/.pgpass -v ${PWD}/result:/var/lib/pg_dump/result -it --rm postgres:15-bullseye pg_dump -v --host localhost --port 5432 --username openstreetmap --no-password --file /var/lib/pg_dump/result --format d -d openstreetmap --compress 0 --table public.nodes --table public.node_tags --table public.ways --table public.way_nodes --table public.way_tags --table public.relations --table public.relation_members --table public.relation_tags --table public.changesets --table public.users --snapshot="00000003-00000204-1"
# commit