#!/bin/bash
set -ex

# used for tests only
psql -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" <<-EOSQL
    CREATE DATABASE openstreetmap;
    CREATE USER openstreetmap SUPERUSER PASSWORD 'openstreetmap';
    GRANT ALL PRIVILEGES ON DATABASE openstreetmap TO openstreetmap;
EOSQL
