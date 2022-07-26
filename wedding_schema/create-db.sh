#!/bin/bash
set -e

psql --dbname postgres <<-EOSQL
drop database if exists wedding;

create database wedding;

grant all privileges on all tables in schema public to wedding;
EOSQL
